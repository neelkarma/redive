use anyhow::{anyhow, Context, Result};
use colored::{Color, Colorize};
use indicatif::ProgressBar;
use std::env;
use ureq::AgentBuilder;
use url::Url;

fn status_to_color(status: u16) -> Color {
    match status {
        100..=199 => Color::Cyan,    // Information
        200..=299 => Color::Green,   // Success
        300..=399 => Color::Blue,    // Redirect
        400..=499 => Color::Red,     // Client Error
        500..=599 => Color::Magenta, // Server Error
        _ => Color::White,
    }
}

struct Checker {
    agent: ureq::Agent,
    pb: ProgressBar,
    redirects: i32,
    max_redirects: i32,
}

impl Checker {
    fn new(max_redirects: i32, pb: ProgressBar) -> Self {
        Checker {
            agent: AgentBuilder::new().redirects(0).build(),
            redirects: 0,
            max_redirects,
            pb,
        }
    }

    fn check(&mut self, cur_url: String) -> Result<()> {
        if self.redirects > self.max_redirects {
            return Err(anyhow!("Max redirects exceeded"));
        }

        let res = self.agent.get(&cur_url).call().context(format!(
            "Request to {} failed. Is the URL correct?",
            &cur_url
        ))?;

        self.pb.println(format!(
            "{} {} {}",
            format!("#{}:", self.redirects).bold(),
            format!("{}", res.status())
                .bold()
                .color(status_to_color(res.status())),
            &cur_url.dimmed()
        ));

        if (300..=399).contains(&res.status()) {
            let url = self.get_url(res, &cur_url)?;
            self.redirects += 1;
            return self.check(url);
        }

        self.pb
            .println(format!("{} Redirect(s) -> {}", self.redirects, &cur_url));
        Ok(())
    }

    fn get_url(&self, res: ureq::Response, cur_url: &str) -> Result<String> {
        let location = res
            .header("Location")
            .context("No Location header in 3xx response")?
            .to_string();

        if location.starts_with("http://") || location.starts_with("https://") {
            return Ok(location);
        }

        let mut prev_url = Url::parse(cur_url).context("Failed to parse current url")?;
        prev_url.set_query(None);
        prev_url.set_fragment(None);

        if location.starts_with("/") {
            prev_url.set_path("");
        }

        let res = prev_url
            .join(&location)
            .context("Failed to concatenate relative Location header value to base url")?
            .to_string();

        Ok(res)
    }
}

fn main() -> Result<()> {
    let url = env::args().nth(1).ok_or(anyhow!("No URL provided."))?;
    let max_redirects: i32 = env::args()
        .nth(2)
        .unwrap_or("30".to_string())
        .parse()
        .map_err(|_| anyhow!("The max_redirects must be an integer"))?;

    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(100);
    pb.set_message(format!("Tracing {}", &url));

    let mut checker = Checker::new(max_redirects, pb);
    checker.check(url)
}

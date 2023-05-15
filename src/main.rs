use anyhow::{anyhow, Context, Result};
use colored::{Color, Colorize};
use indicatif::ProgressBar;
use std::env;
use ureq::AgentBuilder;

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

fn main() -> Result<()> {
    let mut url = env::args().nth(1).ok_or(anyhow!("No URL provided."))?;

    let agent = AgentBuilder::new().redirects(0).build();
    let mut count = 1;

    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(100);

    loop {
        pb.set_message(format!("Tracing {}", &url));

        let res = agent
            .get(&url)
            .call()
            .context(format!("Request to {} failed. Is the URL correct?", &url))?;

        pb.println(format!(
            "{} {} {}",
            format!("#{}:", count).bold(),
            format!("{}", res.status())
                .bold()
                .color(status_to_color(res.status())),
            &url.dimmed()
        ));

        // Break on a non-redirect response
        if !(300..=399).contains(&res.status()) {
            break;
        }

        url = res
            .header("Location")
            .context("No Location header in 3xx response")?
            .to_string();

        count += 1;
    }

    pb.println(format!("{} Redirect(s) -> {}", count - 1, &url));

    Ok(())
}

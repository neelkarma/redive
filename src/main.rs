use anyhow::{Error, Result};
use clap::Parser;
use colored::{Color, Colorize};
use indicatif::ProgressBar;
use reqwest::{blocking::ClientBuilder, redirect::Policy, StatusCode, Url};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(help = "URL to trace.")]
    pub url: String,
}

fn get_color_from_code(status: &StatusCode) -> Color {
    if status.is_informational() {
        Color::Cyan
    } else if status.is_success() {
        Color::Green
    } else if status.is_redirection() {
        Color::Blue
    } else if status.is_client_error() {
        Color::Red
    } else if status.is_server_error() {
        Color::Magenta
    } else {
        panic!("Invalid Status Code {}", status.as_u16());
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    let client = ClientBuilder::new().redirect(Policy::none()).build()?;
    let mut url = Url::parse(&args.url)?;
    let mut num = 1;

    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(100);

    loop {
        pb.set_message(format!("Tracing URL {}", &url));

        let res = client.get(url.clone()).send()?;

        pb.println(format!(
            "{} {} {}",
            format!("#{}:", num).bold(),
            format!(" {} ", res.status().as_u16())
                .bold()
                .on_color(get_color_from_code(&res.status())),
            &url
        ));

        if !res.status().is_redirection() {
            break;
        }

        url = Url::parse(
            res.headers()
                .get("Location")
                .ok_or(Error::msg("No Location header in 3xx response"))?
                .to_str()?,
        )
        .expect(&format!("Invalid URL in Location Header"));
        num += 1;
    }

    pb.println(format!("{} Redirection(s) -> {}", num - 1, &url));

    Ok(())
}

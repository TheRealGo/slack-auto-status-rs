use reqwest::Client;
use serde_json::json;
use std::env;
use std::error::Error;
use std::{process::Command, thread::sleep, time::Duration};

fn slack_is_running() -> bool {
    Command::new("pgrep")
        .arg("-x")
        .arg("Slack")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn main() -> Result<(), Box<dyn Error>> {
    let token = env::var("SLACK_TOKEN")?;
    let rt = tokio::runtime::Runtime::new()?;

    let mut was_running = false;

    loop {
        let is_running = slack_is_running();
        if is_running && !was_running {
            rt.block_on(set_slack_status(&token, "勤務中", ":computer:"))?;
            was_running = true;
        } else if !is_running && was_running {
            rt.block_on(set_slack_status(&token, "不在", ":no_entry:"))?;
            was_running = false;
        }

        sleep(Duration::from_secs(60));
    }
}

async fn set_slack_status(
    token: &str,
    status_text: &str,
    emoji: &str,
) -> Result<(), Box<dyn Error>> {
    let profile = json!({
        "status_text": status_text,
        "status_emoji": emoji,
    });

    let body = json!({ "profile": profile });

    let client = Client::new();
    let res = client
        .post("https://slack.com/api/users.profile.set")
        .bearer_auth(token)
        .json(&body)
        .send()
        .await?;

    let resp_json: serde_json::Value = res.json().await?;
    if resp_json["ok"].as_bool().unwrap_or(false) {
        Ok(())
    } else {
        Err(format!("Slack API error: {:?}", resp_json).into())
    }
}

use ::reqwest::StatusCode;
use reqwest::blocking as reqwest;
use std::{error::Error, thread, time::Duration};

#[derive(Debug)]
pub enum HealthStatus {
    /// url is health (don't send email)
    Health(String),
    /// url is dead and fallback is dead (don't send email)
    FallBackDead(String),
    /// url is dead and fallback is health (send email)
    Dead(String),
}

pub struct HealthUrl {
    url: String,
    fallback_url: String,
}

impl HealthUrl {
    pub fn new(url: impl Into<String>, fallback_url: impl Into<String>) -> Self {
        HealthUrl {
            url: url.into(),
            fallback_url: fallback_url.into(),
        }
    }

    pub fn check_health(&self, tries: u8) -> Result<String, Box<dyn Error>> {
        let mut error: Option<Box<dyn Error>> = None;
        let mut ok: Option<String> = None;

        for _ in 0..tries {
            match self.request_to_url(&self.url, Some(&self.fallback_url)) {
                Ok(HealthStatus::Health(msg)) => return Ok(msg),
                Ok(HealthStatus::FallBackDead(msg)) => {
                    ok = Some(msg.into());
                    error = None;
                }
                Ok(HealthStatus::Dead(msg)) => {
                    error = Some(msg.into());
                    ok = None;
                }
                Err(err) => {
                    error = Some(err);
                }
            }

            thread::sleep(Duration::from_secs(1));
        }

        if !error.is_none() {
            Err(error.expect("unwrap error from request_to_url"))
        } else {
            Ok(ok.expect("unwrap ok from request_to_url"))
        }
    }

    fn request_to_url(
        &self,
        input_url: impl AsRef<str>,
        fallback_url: Option<&str>,
    ) -> Result<HealthStatus, Box<dyn Error>> {
        let url = input_url.as_ref();
        println!("Request to url ====> {url}");

        let response = reqwest::get(url)?;

        let status = response.status();
        // let body = response.text()?;

        // println!("Body = {body}");
        // println!("Status = {status}");

        match status {
            StatusCode::OK => {
                let msg = format!("{url} is health");
                // Ok(HealthStatus::Health{ msg, fallback_is_dead: false }) // Ok
                Ok(HealthStatus::Health(msg)) // Ok
            }
            _http_code => {
                if let Some(fallback_url) = fallback_url {
                    match self.request_to_url(fallback_url, None) {
                        Ok(HealthStatus::Health { .. }) => {
                            let msg = format!(
                                "{url} is down, status code = {_http_code}, but fallback {fallback_url} is health"
                            );
                            Ok(HealthStatus::Dead(msg)) // Err
                        }
                        Ok(HealthStatus::Dead(_fallback_url_msg)) => {
                            let msg = format!(
                                "{url} is down, status code = {_http_code}. {_fallback_url_msg}"
                            );
                            Ok(
                                // HealthStatus::Health{ msg, fallback_is_dead: true }
                                HealthStatus::FallBackDead(msg),
                            )
                        }
                        Ok(HealthStatus::FallBackDead(_)) => {
                            panic!("Invalid status - FallBackDead");
                        }
                        Err(error) => Err(error), // Err
                    }
                } else {
                    let msg = format!("Fallback url {url} is down, status code = {_http_code}");
                    Ok(HealthStatus::Dead(msg)) // Err
                }
            }
        }
    }
}

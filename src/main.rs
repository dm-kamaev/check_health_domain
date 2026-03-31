use std::{error::Error, process::ExitCode};

mod private_data;
mod config;
mod health_url;
mod prevent_debug;
mod email_sdk;

use crate::{email_sdk::{EmailDeliverySDK, SendEmail}, health_url::{HealthUrl}};

fn main() -> ExitCode {

  // Anti-debugging (Linux only) - disabled for development
  #[cfg(all(target_os = "linux", feature = "hardening"))]
  {
      if prevent_debug::detect_debugger() {
        eprintln!("Debugger detected via TracerPid!");
        std::process::exit(1);
      }
      prevent_debug::check_ptrace();
  }


  let health_url = HealthUrl::new(config::URL, config::FALLBACK_URL);
  let email_delivery_sdk = EmailDeliverySDK::new();

  match _main(health_url, email_delivery_sdk, config::get_recipients()) {
    Ok(msg) => {
      println!("Result: {msg}");

      ExitCode::SUCCESS
    },
    Err(error) => {
      println!("Error => {error}");

      ExitCode::FAILURE
    }
  }
}

fn _main(health_url: HealthUrl, email_delivery_sdk: impl SendEmail, recipients: Vec<impl Into<String>>) -> Result<String, Box<dyn Error>> {
  let subject = "RM: НЕ ДОСТУПЕН risk-monitoring.ru !!!!";
  match health_url.check_health(3) {
    Ok(msg) => {
      Ok(msg)
    },
    Err(error) => {
      email_delivery_sdk.send(
        recipients,
        subject,
        format!("{error}")
      )?;

      Err(error)
    }
  }
}

// ======== TESTS ========

#[cfg(test)]
mod tests {
    use super::*;
    use crate::email_sdk::{with_mock_sent_messages, SmtpTransportFake};
    use mockito::Server;

    // Helper to get captured messages from the fake transport
    fn get_sent_messages() -> Vec<lettre::Message> {
        with_mock_sent_messages(|messages| messages.borrow().clone())
    }

    fn clear_sent_messages() {
        with_mock_sent_messages(|messages| messages.borrow_mut().clear());
    }

    #[test]
    fn test_main_url_is_health() {
        clear_sent_messages();

        let mut server = Server::new();

        let mock = server.mock("GET", "/")
            .with_status(200)
            .with_body("OK")
            .create();

        let health_url = HealthUrl::new(&server.url(), "https://fallback.example.com");
        let email_sdk = EmailDeliverySDK::<SmtpTransportFake>::_new();

        let result = _main(health_url, email_sdk, vec!["test@example.com", "recipient2@example.com"]);

        assert!(result.is_ok());
        assert!(result.unwrap().contains("is health"));

        assert!(get_sent_messages().is_empty());

        mock.assert();
    }

    #[test]
    fn test_main_url_is_dead_and_fallback_is_health() {
        clear_sent_messages();

        let mut server = Server::new();

        let mock = server.mock("GET", "/")
          .with_status(503)
          .with_body("Service Unavailable")
          .expect(3)
          .create();

        let fallback_path_name = "fallback";
        let fallback_path = format!("/{fallback_path_name}");
        let mock_fallback = server.mock("GET", fallback_path.as_str())
          .with_status(200)
          .with_body("OK")
          .expect(3)
          .create();

        let primary_url = server.url();
        let fallback_url = format!("{}/{fallback_path_name}", server.url());

        let health_url = HealthUrl::new(primary_url, fallback_url);
        let email_sdk = EmailDeliverySDK::<SmtpTransportFake>::_new();

        let result = _main(health_url, email_sdk, vec!["test@example.com", "recipient2@example.com",]);

        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(msg.contains("is down") && msg.contains("fallback"));

        let sent_messages = get_sent_messages();
        assert_eq!(sent_messages.len(), 1);

        mock.assert();
        mock_fallback.assert();
    }


    #[test]
    fn test_main_url_is_dead_and_fallback_is_dead() {
        clear_sent_messages();

        let mut server = Server::new();

        let mock = server.mock("GET", "/")
          .with_status(503)
          .expect(3)
          .with_body("Service Unavailable")
          .create();

        let fallback_path_name = "fallback";
        let fallback_path = format!("/{fallback_path_name}");
        let mock_fallback = server.mock("GET", fallback_path.as_str())
          .with_status(503)
          .expect(3)
          .with_body("Service Unavailable")
          .create();

        let primary_url = server.url();
        let fallback_url = format!("{}/{fallback_path_name}", server.url());

        let health_url = HealthUrl::new(primary_url, fallback_url);
        let email_sdk = EmailDeliverySDK::<SmtpTransportFake>::_new();

        let result = _main(health_url, email_sdk, vec!["test@example.com", "recipient2@example.com"]);

        assert!(result.is_ok());
        let msg = result.unwrap();

        assert!(msg.contains("is down") && msg.contains("fallback"));
        let sent_messages = get_sent_messages();
        assert_eq!(sent_messages.len(), 0);

        mock.assert();
        mock_fallback.assert();
    }


    #[test]
    fn test_retries() {
        clear_sent_messages();

        let mut server = Server::new();

      // ====== Setup retries ======
        // First attempt → fail
        let m1 = server.mock("GET", "/")
            .with_status(500)
            .expect(1)
            .with_body("fail 1")
            .create();

        // Second attempt → fail
        let m2 = server.mock("GET", "/")
            .with_status(500)
            .expect(1)
            .with_body("fail 2")
            .create();

        // Third attempt → success
        let m3 = server.mock("GET", "/")
            .with_status(200)
            .expect(1)
            .with_body("ok")
            .create();

        // ====== Setup fallback ======
        let fallback_path_name = "fallback";
        let fallback_path = format!("/{fallback_path_name}");
        let mock_fallback = server.mock("GET", fallback_path.as_str())
          .with_status(200)
          .expect(2)
          .with_body("OK")
          .create();

        let primary_url = server.url();
        let fallback_url = format!("{}/{fallback_path_name}", server.url());

        let health_url = HealthUrl::new(primary_url, fallback_url);


        let email_sdk = EmailDeliverySDK::<SmtpTransportFake>::_new();

        let result = _main(health_url, email_sdk, vec![
            "test@example.com",
        ]);

        assert!(result.is_ok());
        assert!(result.unwrap().contains("is health"));

        assert!(get_sent_messages().is_empty());

        m1.assert();
        m2.assert();
        m3.assert();
        mock_fallback.assert();
    }

}




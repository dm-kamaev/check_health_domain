use lettre::{
    Message, SmtpTransport, Transport, message::header::ContentType,
    transport::smtp::authentication::Credentials,
};
use std::cell::RefCell;
use std::error::Error;
use std::marker::PhantomData;

use crate::config;

// ===== Interfaces for InputTransport::relay("smtp.gmail.com")?.credentials(credo).build() =====
pub trait SmtpTransportTrait: Send + Sync {
    fn relay(host: &str) -> Result<Self::Builder, Box<dyn Error>>;
    fn send(&self, message: &Message) -> Result<(), Box<dyn Error>>;
    type Builder: TransportBuilder<Transport = Self>;
}

pub trait TransportBuilder: Sized {
    fn credentials(self, credo: Credentials) -> Self;
    fn build(self) -> Self::Transport;
    type Transport: SmtpTransportTrait;
}

pub struct SmtpTransportBuilder {
    host: String,
    credentials: Option<Credentials>,
}

impl TransportBuilder for SmtpTransportBuilder {
    type Transport = SmtpTransport;

    fn credentials(mut self, credo: Credentials) -> Self {
        self.credentials = Some(credo);
        self
    }

    fn build(self) -> Self::Transport {
        let credo = self.credentials.expect("Credentials required");
        SmtpTransport::relay(&self.host)
            .unwrap()
            .credentials(credo)
            .build()
    }
}

impl SmtpTransportTrait for SmtpTransport {
    type Builder = SmtpTransportBuilder;

    fn relay(host: &str) -> Result<Self::Builder, Box<dyn Error>> {
        Ok(SmtpTransportBuilder {
            host: host.to_string(),
            credentials: None,
        })
    }

    fn send(&self, message: &Message) -> Result<(), Box<dyn Error>> {
        Transport::send(self, message)?;
        Ok(())
    }
}

// ===== SmtpTransport Fake Implementation =====

// This stores messages for the current thread (perfect for isolated unit tests)
thread_local! {
    static MOCK_SENT_MESSAGES: RefCell<Vec<Message>> = RefCell::new(Vec::new());
}

/// Public accessor for tests
#[allow(dead_code)]
pub fn with_mock_sent_messages<F, T>(f: F) -> T
where
    F: FnOnce(&RefCell<Vec<Message>>) -> T,
{
    MOCK_SENT_MESSAGES.with(f)
}

#[allow(dead_code)]
pub struct SmtpTransportFake;

#[allow(dead_code)]
pub struct SmtpTransportFakeBuilder {
    credentials: Option<Credentials>,
}

impl TransportBuilder for SmtpTransportFakeBuilder {
    type Transport = SmtpTransportFake;

    fn credentials(mut self, credo: Credentials) -> Self {
        self.credentials = Some(credo);
        self
    }

    fn build(self) -> Self::Transport {
        SmtpTransportFake
    }
}

impl SmtpTransportTrait for SmtpTransportFake {
    type Builder = SmtpTransportFakeBuilder;

    fn relay(_host: &str) -> Result<Self::Builder, Box<dyn Error>> {
        Ok(SmtpTransportFakeBuilder { credentials: None })
    }

    fn send(&self, message: &Message) -> Result<(), Box<dyn Error>> {
        // Save the message to our thread-local storage!
        MOCK_SENT_MESSAGES.with(|messages| {
            messages.borrow_mut().push(message.clone());
        });
        Ok(())
    }
}

// ===== EmailDeliverySDK Implementation =====

pub struct EmailDeliverySDK<T: SmtpTransportTrait = SmtpTransport> {
    // type annotation for InputTransport::relay()...
    _marker: PhantomData<T>,
}

/// for real using
impl EmailDeliverySDK<SmtpTransport> {
    pub fn new() -> Self {
        Self::_new()
    }
}

/// for using test cases
impl<T: SmtpTransportTrait> EmailDeliverySDK<T> {
    pub fn _new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

pub trait SendEmail {
    fn send(
        &self,
        recipients: Vec<impl Into<String>>,
        subject: impl Into<String>,
        mail_body: impl Into<String>,
    ) -> Result<(), Box<dyn Error>>;
}

impl<InputTransport: SmtpTransportTrait> SendEmail for EmailDeliverySDK<InputTransport> {
    fn send(
        &self,
				recipients: Vec<impl Into<String>>,
        subject: impl Into<String>,
        mail_body: impl Into<String>,
    ) -> Result<(), Box<dyn Error>> {
        let subject = subject.into();
        let body = mail_body.into();

        let mut msg_builder = Message::builder()
            .from(config::get_first_user().as_str().parse()?)
            .subject(&subject)
            .header(ContentType::TEXT_PLAIN);

        for recipient_into in recipients {
						let recipient: String = recipient_into.into();
            msg_builder = msg_builder.to(recipient.parse()?);
        }

        let message = msg_builder.body(body)?;

        let pass = config::get_first_userp();
        let credo = Credentials::new(
            config::get_first_user().as_str().to_string(),
            pass.as_str().to_string(),
        );

        let mailer = InputTransport::relay("smtp.gmail.com")?
            .credentials(credo)
            .build();

        mailer.send(&message)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to clear mock state between tests just to be safe
    fn clear_mock_messages() {
        with_mock_sent_messages(|messages| messages.borrow_mut().clear());
    }

    #[test]
    fn test_sdk_send_with_fake_transport_success() {
        clear_mock_messages();

        let sdk = EmailDeliverySDK::<SmtpTransportFake>::_new();

        let result = sdk.send(vec!["test@example.com"],"Test Subject", "Test message");
        assert!(result.is_ok());

        with_mock_sent_messages(|messages| {
            let sent = messages.borrow();
            assert_eq!(sent.len(), 1, "One message should have been sent");
        });
    }
}

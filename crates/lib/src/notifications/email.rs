use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

pub async fn send_email(settings_map: &HashMap<String, String>, subject: &str, message: &str) {
    if settings_map
        .get("smtp_enabled")
        .expect("could not get smtp_enabled")
        .parse::<bool>()
        .unwrap()
    {
        let host: String = settings_map
            .get("smtp_host")
            .expect("could not get smtp_host")
            .to_string();
        // let port: u32 = settings_map.get("smtp_port").expect("could not get smtp_port").parse::<u32>().unwrap();
        // let require_auth: bool = settings_map.get("smtp_require_auth").expect("could not get smtp_require_auth").parse::<bool>().unwrap();
        let username: String = settings_map
            .get("smtp_username")
            .expect("could not get smtp_username")
            .to_string();
        let password: String = settings_map
            .get("smtp_password")
            .expect("could not get smtp_password")
            .to_string();
        let smtp_from: String = settings_map
            .get("smtp_from")
            .expect("could not get smtp_from")
            .to_string();
        let pgduty: String = settings_map
            .get("pgduty")
            .expect("could not get pgduty")
            .to_string();

        // println!("{:#?}", settings_map);
        let mut all_recipients: Vec<String> = vec![];

        settings_map
            .iter()
            .filter_map(|e| match e.0.contains("smtp_recipient") {
                true => Some(e.1),
                false => None,
            })
            .collect::<Vec<_>>()
            .iter()
            .for_each(|e| all_recipients.push(e.to_string()));

        // if message contains ::PAGERDUTY:: add pgduty email to recipients list
        if subject.contains("::PAGERDUTY::") || message.contains("::PAGERDUTY::") {
            println!("PAGERDUTY Triggered\n{}\n{}", subject, message);
            all_recipients.push(pgduty);
        }

        println!("Emailing:: {:#?}", all_recipients);

        all_recipients.iter().for_each(|recipient| {
            let email = Message::builder()
                .from(smtp_from.parse().unwrap())
                .reply_to(smtp_from.parse().unwrap())
                .to(recipient.parse().unwrap())
                .subject(subject)
                .header(ContentType::TEXT_PLAIN)
                .body(String::from(message))
                .unwrap();

            let creds = Credentials::new(username.to_owned(), password.to_owned());

            // Open a remote connection to gmail
            let mailer = SmtpTransport::relay(host.as_str())
                .unwrap()
                .credentials(creds)
                .build();

            // // Send the email
            match mailer.send(&email) {
                Ok(_) => println!("Email sent successfully!"),
                Err(e) => println!("Could not send email: {e:?}"),
            }
            thread::sleep(Duration::new(1, 500000000)); // sleep for 1.5 seconds
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::notifications::email::send_email;
    use config::Config;
    use std::collections::HashMap;

    fn setup_test() -> HashMap<String, String> {
        let settings = Config::builder()
            .add_source(config::File::with_name("config/Settings"))
            .build()
            .unwrap();
        settings
            .try_deserialize::<HashMap<String, String>>()
            .unwrap()
    }

    #[test]
    #[ignore]
    fn test_email() {
        let settings_map = setup_test();
        let rt = tokio::runtime::Runtime::new();
        rt.unwrap()
            .block_on(send_email(&settings_map, "test email", "test message"));
    }
}

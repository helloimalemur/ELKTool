use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;


pub async fn send_email(settings_map: &HashMap<String, String>, subject: &str, message: &str) {
    if settings_map.get("smtp_enabled").expect("could not get smtp_enabled").parse::<bool>().unwrap() {
        let host: String = settings_map.get("smtp_host").expect("could not get smtp_host").to_string();
        let port: u32 = settings_map.get("smtp_port").expect("could not get smtp_port").parse::<u32>().unwrap();
        let require_auth: bool = settings_map.get("smtp_require_auth").expect("could not get smtp_require_auth").parse::<bool>().unwrap();
        let username: String = settings_map.get("smtp_username").expect("could not get smtp_username").to_string();
        let password: String = settings_map.get("smtp_password").expect("could not get smtp_password").to_string();

        // println!("{:#?}", settings_map);

        let recipients: Vec<&String> = settings_map
            .iter()
            .filter_map(|e| match e.0.contains("smtp_recipient") {
                true => Some(e.1),
                false => None
            })
            .collect::<Vec<_>>();

        println!("Emailing:: {:#?}", recipients);

        recipients.iter().for_each(|recipient| {

            let email = Message::builder()
                .from("noreply <noreply@koonts.net>".parse().unwrap())
                .reply_to("noreply <noreply@koonts.net>".parse().unwrap())
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
                Err(e) => panic!("Could not send email: {e:?}"),
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use config::Config;
    use crate::alerts_api_funcs::email::send_email;

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
    fn test_email() {
        let settings_map = setup_test();
        let rt = tokio::runtime::Runtime::new();
        rt.unwrap().block_on(send_email(&settings_map, "test email", "test message"));
    }

}

pub mod discord {
    use serde::{Serialize, Deserialize};

    #[derive(Serialize, Deserialize)]
    pub struct Message {
        pub content: String,
    }

    #[derive(Deserialize)]
    pub struct User {
        pub username: String,
        pub discriminator: String,
        pub avatar: String,
        pub email: Option<String>,
        pub flags: Option<String>,
    }
}

pub mod client {
    use crate::discord::{Message, User};
    use reqwest::header::{AUTHORIZATION, HeaderValue};
    use reqwest::{Client, Error};

    pub struct HttpClient {
        client: Client,
        authorization_header: HeaderValue,
    }

    impl HttpClient {
        pub fn new() -> HttpClient {
            HttpClient {
                client: Client::new(),
                authorization_header: HeaderValue::from_static("Bot <bot token here>"),
            }
        }

        pub async fn get_user(&self, user_id: &str) -> Result<User, Error> {
            self.client.get(&*format!("https://discord.com/api/users/{}", user_id))
                .header(AUTHORIZATION, &self.authorization_header)
                .send()
                .await?
                .json::<User>()
                .await
        }

        pub async fn get_messages(&self, channel_id: &str) -> Result<Vec<Message>, Error> {
            self.client.get(&*format!("https://discord.com/api/channels/{}/messages", channel_id))
                .header(AUTHORIZATION, &self.authorization_header)
                .send()
                .await?
                .json::<Vec<Message>>()
                .await
        }

        pub async fn get_message(&self, channel_id: &str, message_id: &str) -> Result<Message, Error> {
            self.client.get(&*format!("https://discord.com/api/channels/{}/messages/{}", channel_id, message_id))
                .header(AUTHORIZATION, &self.authorization_header)
                .send()
                .await?
                .json::<Message>()
                .await
        }

        pub async fn send_message(&self, channel_id: &str, message: &str) -> Result<Message, Error> {
            let message = Message {
                content: message.parse().unwrap()
            };

            self.client.post(&*format!("https://discord.com/api/channels/{}/messages", channel_id))
                .header(AUTHORIZATION, &self.authorization_header)
                .json(&message)
                .send()
                .await?
                .json::<Message>()
                .await
        }
    }
}
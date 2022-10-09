use std::collections::HashMap;
use reqwest::header::{AUTHORIZATION, HeaderValue, InvalidHeaderValue};
use reqwest::{Client, Error, Response};
use serde::{Serialize, Deserialize};

const BASE_URL: &'static str = "https://discord.com/api/v10";

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub channel_id: String,
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

pub struct HttpClient {
    client: Client,
    authorization_header: HeaderValue,
}

impl HttpClient {
    pub fn new(bot_token: &str) -> Result<HttpClient, InvalidHeaderValue> {
        Ok(HttpClient {
            client: Client::new(),
            authorization_header: HeaderValue::from_str(&format!("Bot {}", bot_token))?,
        })
    }

    pub async fn get_user(&self, user_id: &str) -> Result<User, Error> {
        self.client.get(&format!("{}/users/{}", BASE_URL, user_id))
            .header(AUTHORIZATION, &self.authorization_header)
            .send()
            .await?
            .json::<User>()
            .await
    }

    pub async fn get_messages(&self, channel_id: &str) -> Result<Vec<Message>, Error> {
        self.client.get(&format!("{}/channels/{}/messages", BASE_URL, channel_id))
            .header(AUTHORIZATION, &self.authorization_header)
            .send()
            .await?
            .json::<Vec<Message>>()
            .await
    }

    pub async fn get_message(&self, channel_id: &str, message_id: &str) -> Result<Message, Error> {
        self.client.get(&format!("{}/channels/{}/messages/{}", BASE_URL, channel_id, message_id))
            .header(AUTHORIZATION, &self.authorization_header)
            .send()
            .await?
            .json::<Message>()
            .await
    }

    pub async fn create_message(&self, channel_id: &str, message: &str) -> Result<Message, Error> {
        let content: String = message.parse().unwrap();

        let mut map = HashMap::new();
        map.insert("content", content);

        self.client.post(&format!("{}/channels/{}/messages", BASE_URL, channel_id))
            .header(AUTHORIZATION, &self.authorization_header)
            .json(&map)
            .send()
            .await?
            .json::<Message>()
            .await
    }

    pub async fn edit_message(&self, channel_id: &str, message_id: &str, message: &str) -> Result<Message, Error> {
        let content: String = message.parse().unwrap();

        let mut map = HashMap::new();
        map.insert("content", content);

        self.client.patch(&format!("{}/channels/{}/messages/{}", BASE_URL, channel_id, message_id))
            .header(AUTHORIZATION, &self.authorization_header)
            .json(&map)
            .send()
            .await?
            .json::<Message>()
            .await
    }

    pub async fn delete_message(&self, channel_id: &str, message_id: &str) -> Result<Response,
        Error> {
        self.client.delete(&format!("{}/channels/{}/messages/{}", BASE_URL, channel_id, message_id))
            .header(AUTHORIZATION, &self.authorization_header)
            .send()
            .await
    }

    pub async fn get_reactions(&self, channel_id: &str, message_id: &str, emoji: &str) ->
                                                                                       Result<Vec<User>, Error> {
        self.client.get(&format!("{}/channels/{}/messages/{}/reactions/{}", BASE_URL, channel_id, message_id, emoji))
            .header(AUTHORIZATION, &self.authorization_header)
            .send()
            .await?
            .json::<Vec<User>>()
            .await
    }

    pub async fn create_reaction(&self, channel_id: &str, message_id: &str, emoji: &str) ->
                                                                                         Result<Response, Error> {
        self.client.put(&format!("{}/channels/{}/messages/{}/reactions/{}/@me", BASE_URL, channel_id, message_id, emoji))
            .header(AUTHORIZATION, &self.authorization_header)
            .send()
            .await
    }

    pub async fn delete_reaction(&self, channel_id: &str, message_id: &str, emoji: &str) ->
                                                                                         Result<Response, Error> {
        self.client.delete(&format!("{}/channels/{}/messages/{}/reactions/{}/@me", BASE_URL, channel_id, message_id, emoji))
            .header(AUTHORIZATION, &self.authorization_header)
            .send()
            .await
    }

    pub async fn delete_user_reaction(&self, channel_id: &str, message_id: &str, emoji: &str,
                                      user_id: &str) -> Result<Response, Error> {
        self.client.delete(&format!("{}/channels/{}/messages/{}/reactions/{}/{}", BASE_URL, channel_id, message_id, emoji, user_id))
            .header(AUTHORIZATION, &self.authorization_header)
            .send()
            .await
    }

    pub async fn get_gateway(&self) -> Result<String, Error> {
        self.client.get(&format!("{}/gateway", BASE_URL))
            .header(AUTHORIZATION, &self.authorization_header)
            .send()
            .await?
            .json()
            .await
    }
}
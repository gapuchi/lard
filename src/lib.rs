pub mod discord;

pub mod high {
    use crate::discord::HttpClient;
    use reqwest::{Client, Error, Response};

    pub struct Channel<'a> {
        client: &'a HttpClient,
        id: String,
    }

    pub struct Message<'a> {
        client: &'a HttpClient,
        id: String,
        channel_id: String,
        content: String,
    }

    impl Channel<'_> {
        async fn send(&self, msg: &str) -> Result<Message, Error> {
            let super::discord::Message { id, channel_id, content } = self.client
                .create_message(&self.id, msg)
                .await?;

            Ok(Message {
                client: self.client,
                id,
                channel_id,
                content,
            })
        }
    }

    impl Message<'_> {
        fn delete(&self) -> Result<(), Error> {

            &self.client.delete_message(&*self.channel_id, &*self.id);

            Ok(())
        }
    }
}
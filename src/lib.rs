use std::thread::sleep;
use std::time::Duration;
use futures_util::{SinkExt, StreamExt};
use native_tls::TlsConnector;
use reqwest::header::InvalidHeaderValue;
use serde_json::{from_str, to_string};
use tokio::select;
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::connect_async_tls_with_config;
use tokio_tungstenite::Connector::NativeTls;
use tokio_tungstenite::tungstenite::Message;
use crate::discord::{ConnectionProperties, DiscordClient, GatewayEvent, Hello, Identify};

pub mod discord;

pub mod high {
    use reqwest::Error;

    use crate::discord::DiscordClient;

    pub struct Channel<'a> {
        client: &'a DiscordClient,
        id: String,
    }

    pub struct Message<'a> {
        client: &'a DiscordClient,
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
        async fn edit(&self, msg: &str) -> Result<Message, Error> {
            let super::discord::Message { id, channel_id, content } = self.client
                .edit_message(&*self.channel_id, &*self.id, msg)
                .await?;

            Ok(Message {
                client: self.client,
                id,
                channel_id,
                content,
            })
        }
    }
}

pub struct Client {
    token: String,
    client: DiscordClient,
}

impl Client {
    pub fn new(token: &str) -> Result<Client, InvalidHeaderValue> {
        Ok(Client {
            token: token.to_string(),
            client: DiscordClient::new(token)?,
        })
    }

    pub async fn start(self) -> Result<(), Box<dyn std::error::Error>> {
        let (tx, rx) = oneshot::channel::<u64>();
        let (tx2, mut rx2) = mpsc::channel::<()>(1);

        let address = self.client.get_gateway().await.unwrap().url;

        let (stream, _) = connect_async_tls_with_config(address, None, Some(NativeTls(TlsConnector::new()?)))
            .await?;

        let (mut write, mut read) = stream.split();
        let handle = tokio::spawn(async move {
            match read.next().await {
                Some(Ok(Message::Text(x))) => {
                    println!("{}", x);
                    let result1 = from_str::<GatewayEvent<Hello>>(&x).unwrap();
                    tx.send(result1.d.heartbeat_interval).unwrap();
                }
                _ => {}
            }

            let foo = GatewayEvent::<Identify> {
                op: 2,
                d: Identify {
                    token: self.token.parse().unwrap(),
                    intents: (1 << 9) | (1 << 10),
                    properties: ConnectionProperties {
                        os: "".to_string(),
                        browser: "".to_string(),
                        device: "".to_string(),
                    },
                },
                s: None,
                t: None,
            };

            write.send(Message::Text(to_string(&foo).unwrap()))
                .await
                .unwrap();

            loop {
                select! {
                Some(val) = read.next() => {
                    match val {
                        Ok(Message::Text(x)) => {
                            println!("Got from socket {}", x);
                        }
                        Err(e) => {
                            println!("{}", e);
                        }
                        Ok(Message::Binary(x)) => {
                            println!("Binary {:?}", x);
                        }
                        Ok(Message::Ping(x)) => {
                            println!("Ping {:?}", x);
                        }
                        Ok(Message::Pong(x)) => {
                            println!("Pong {:?}", x);
                        }
                        Ok(Message::Close(x)) => {
                            println!("Close {:?}", x);
                        }
                        _ => {panic!("AHHH")}
                    }
                }
                _ = rx2.recv() => {
                    let heartbeat_event = GatewayEvent::<Option<i16>>{
                        op: 1,
                        d: None,
                        s: None,
                        t: None,
                    };
                    println!("Heart beat");
                    write.send(Message::Text(to_string(&heartbeat_event).unwrap()))
                    .await
                    .unwrap();
                }
            }
            }
        });


        let heartbeat_handle = tokio::spawn(async move {
            let heartbeat_interval = rx.await.unwrap();
            println!("Received hello event - {}", heartbeat_interval);
            loop {
                sleep(Duration::from_millis(1000));
                println!("Send it!");
                tx2.send(()).await.unwrap()
            }
        });

        handle.await?;
        heartbeat_handle.await?;

        Ok(())
    }
}
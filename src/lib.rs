use std::fmt::Debug;
use std::time::Duration;

use futures_util::{Sink, SinkExt, Stream, StreamExt};
use futures_util::stream::{SplitSink, SplitStream};
use native_tls::TlsConnector;
use rand::Rng;
use reqwest::header::InvalidHeaderValue;
use serde_json::{from_str, from_value, to_string, Value};
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
        pub async fn send(&self, msg: &str) -> Result<Message, Error> {
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
        pub async fn edit(&self, msg: &str) -> Result<Message, Error> {
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
    sequence_number: Option<i16>,
}

impl Client {
    pub fn new(token: &str) -> Result<Client, InvalidHeaderValue> {
        Ok(Client {
            token: token.to_string(),
            client: DiscordClient::new(token)?,
            sequence_number: None,
        })
    }

    pub async fn start(mut self) -> Result<(), Box<dyn std::error::Error>> {
        let (heartbeat_interval_sender, heartbeat_interval_receiver) = oneshot::channel::<u64>();
        let (heartbeat_sender, mut heartbeat_receiver) = mpsc::channel::<()>(1);

        let address = self.client.get_gateway().await.unwrap().url;
        let (stream, _) = connect_async_tls_with_config(address, None, Some(NativeTls(TlsConnector::new()?))).await?;
        let (mut websocket_writer, mut websocket_reader) = stream.split();

        let handle = tokio::spawn(async move {
            self.receive_hello_event(&mut websocket_reader, heartbeat_interval_sender).await;
            self.send_identify_event(&mut websocket_writer).await;

            loop {
                tokio::select! {
                    Some(val) = websocket_reader.next() => {
                        match val {
                            Ok(Message::Text(x)) => self.handle_text_websocket_message(x),
                            Ok(x) => println!("Received non text websocket message {:?}", x),
                            Err(e) =>  println!("Received error {}", e),
                        }
                    }
                    _ = heartbeat_receiver.recv() => Self::send_heartbeat(&mut websocket_writer).await
                }
            }
        });


        let heartbeat_handle = tokio::spawn(async move {
            let heartbeat_interval = heartbeat_interval_receiver.await.unwrap();
            println!("Got heartbeat interval {}. Starting heartbeat", heartbeat_interval);
            loop {
                let i = rand::thread_rng().gen_range(0..heartbeat_interval);
                //TODO Why the async version?
                tokio::time::sleep(Duration::from_millis(i)).await;
                heartbeat_sender.send(()).await.unwrap()
            }
        });

        handle.await?;
        heartbeat_handle.await?;

        Ok(())
    }

    pub fn handle_text_websocket_message(&mut self, x: String) {
        let gateway_event = from_str::<GatewayEvent<Value>>(&*x).unwrap();

        if gateway_event.op != 0 {
            println!("Received op code {} with data {:?}", gateway_event.op, gateway_event.d);
            return;
        }

        match gateway_event.t.unwrap().as_ref() {
            "MESSAGE_CREATE" => {
                let msg = from_value::<discord::Message>(gateway_event.d);
                println!("Received message create {:?}", msg);
            }
            z => {
                println!("Received other event {}", z);
            }
        }

        if gateway_event.s.is_some() {
            self.sequence_number = gateway_event.s;
        }
    }

    pub async fn receive_hello_event<T, U>(&self, reader: &mut SplitStream<T>, heartbeat_interval_sender: oneshot::Sender<u64>)
        where T: Stream<Item=Result<Message, U>> + Unpin {
        match reader.next().await {
            Some(Ok(Message::Text(x))) => {
                let hello_event = from_str::<GatewayEvent<Hello>>(&x).unwrap();
                heartbeat_interval_sender.send(hello_event.d.heartbeat_interval).unwrap();
            }
            _ => { panic!("Didn't even get a hello...") }
        }
    }

    pub async fn send_identify_event<T>(&self, writer: &mut SplitSink<T, Message>)
        where T: Sink<Message>,
              T::Error: Debug {
        let identify_event = GatewayEvent::<Identify> {
            op: 2,
            d: Identify {
                token: self.token.parse().unwrap(),
                intents: (1 << 9) | (1 << 10) | (1 << 15),
                properties: ConnectionProperties {
                    os: "".to_string(),
                    browser: "".to_string(),
                    device: "".to_string(),
                },
            },
            s: None,
            t: None,
        };

        writer.send(Message::Text(to_string(&identify_event).unwrap()))
            .await
            .unwrap();
    }

    pub async fn send_heartbeat<T>(writer: &mut SplitSink<T, Message>)
        where T: Sink<Message>,
              T::Error: Debug {
        let heartbeat_event = GatewayEvent::<Option<i16>> {
            op: 1,
            //TODO Sequence number here
            d: None,
            s: None,
            t: None,
        };
        println!("Heart beat with seq {:?}", heartbeat_event.d);
        writer.send(Message::Text(to_string(&heartbeat_event).unwrap()))
            .await
            .unwrap();
    }
}
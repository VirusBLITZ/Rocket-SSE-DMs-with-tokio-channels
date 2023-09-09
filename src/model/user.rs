use std::sync::Arc;

use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::RwLock;

use crate::model::message::Message;

#[derive(Clone)]
pub struct User {
    pub username: String,
    pub unread: Vec<Message>,
    pub send: Option<Sender<Message>>,
}

impl User {
    pub fn with_name(name: String) -> Self {
        User {
            username: name,
            unread: Vec::new(),
            send: None,
        }
    }
}

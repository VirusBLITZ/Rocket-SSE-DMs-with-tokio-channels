use std::collections::HashMap;

use model::message::Message;
use model::user::User;
use once_cell::sync::Lazy;
use rocket::form::Form;
use rocket::http::Status;
use rocket::response::stream::{Event, EventStream};
use tokio::sync::RwLock;

#[macro_use]
extern crate rocket;

mod cors;
mod model;

static USERS: Lazy<RwLock<HashMap<i32, RwLock<User>>>> = Lazy::new(|| RwLock::new(HashMap::new()));

#[get("/messages?<user_id>")]
async fn recv_msg(user_id: i32) -> EventStream![] {
    let users = crate::USERS.read().await;
    let user = &mut users.get(&user_id).unwrap().write().await;
    let (tx, mut rx) = tokio::sync::mpsc::channel(32);
    user.send = Some(tx);

    EventStream! {
        while let Some(msg) = rx.recv().await {
            yield Event::data(msg.content);
        }
    }
}

#[post("/msg/<receiver>/<content>")]
async fn direct_message(receiver: i32, content: String) -> Status {
    let users = &*USERS.read().await;
    let recipient = &*users.get(&receiver).unwrap().read().await;
    _ = recipient
        .send
        .clone()
        .unwrap()
        .send(Message {
            sender: User {
                username: "anonymous".to_owned(),
                unread: Vec::new(),
                send: None,
            },
            content,
        })
        .await;
    Status::Ok
}

#[derive(FromForm)]
pub struct SendMessage {
    pub sender: String,
    pub message: String,
    pub receiver: String,
}

#[post("/test", format = "application/json", data = "<data>")]
async fn test(data: Form<SendMessage>) -> &'static str {
    "Hello, world!"
}

#[post("/message", format = "application/json", data = "<send_message>")]
async fn send_msg(send_message: Form<SendMessage>) -> Status {
    let users = crate::USERS.read().await;
    let receiver = &mut *users
        .get(&send_message.receiver.parse::<i32>().unwrap())
        .unwrap()
        .write()
        .await;
    let sender = &mut *users
        .get(&send_message.sender.parse::<i32>().unwrap())
        .unwrap()
        .write()
        .await;
    receiver
        .send
        .as_mut()
        .unwrap()
        .send(Message {
            sender: sender.clone(),
            content: send_message.message.to_string(),
        })
        .await
        .unwrap();
    Status::Ok
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[launch]
async fn rocket() -> _ {
    let users = &mut *USERS.write().await;
    users.insert(0, RwLock::new(User::with_name("john".to_owned())));
    users.insert(1, RwLock::new(User::with_name("ben".to_owned())));

    rocket::build()
        .mount("/", routes![index, test, recv_msg, send_msg, direct_message])
        .attach(cors::CORS)
}

use crate::model::user::User;

#[derive(Clone)]
pub struct Message {
    pub sender: User,
    pub content: String,
}

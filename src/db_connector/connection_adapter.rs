use sqlx::Connection;


pub enum Adapter {
    Connected(),
    Disconnected,
}

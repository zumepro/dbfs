pub mod row;
pub use row::Row;




#[derive(Default, Debug)]
pub enum ColumnType {
    #[default]
    MysqlInteger,
    MysqlString,
}




mod types_encoder;
mod request_frames;
mod response_frames;




mod stream_handler;
pub struct SqlConnection {
    socket_handler: stream_handler::SocketHandler,
}

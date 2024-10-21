use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use crate::settings;




/// Socket handler IO error
///
/// # Connection pipeline
/// ```text
/// + ---------------- +     Fail     + ----------------------- +      Fail
/// | Try IO operation | -----------> | Create a new connection | --------------> Err
/// + ---------------- +              + ----------------------- +
///          ^                                     |
///          +------------------------------------ +
///                        Connected
/// ```
/// This cycle will continue until the limit is reached.
#[derive(Debug)]
pub enum SocketIOError {
    /// Socket handler could not connect to the server socket.
    /// This usually means that the server socket is not listening.
    Connecting,
    /// Error when reading from a socket - this means that the maximum retry count was hit when reading.
    Reading,
    /// Error when writing to a socket - this means that the maximum retry count was hit when writing.
    Writing,
}


impl std::fmt::Display for SocketIOError {


    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let was_doing = match self {
            Self::Connecting => "connecting to",
            Self::Reading => "reading from",
            Self::Writing => "writing to",
        };
        write!(f, "connection_handler: error while {} socket", was_doing)
    }


}




pub enum ConnectionState {
    Disconnected,
    Connected(TcpStream),
}


macro_rules! connstate_check_open {
    ($adapter: expr, $disconnected_err: expr) => {
        match &mut $adapter {
            ConnectionState::Connected(val) => val,
            ConnectionState::Disconnected => { return Err($disconnected_err); }
        }
    };
}




pub struct SocketHandler {
    adapter: ConnectionState,
    host: &'static str,
}




impl SocketHandler {


    /// Create a new Sql connection with settings from crate::settings.
    pub fn new() -> Self {
        Self {
            adapter: ConnectionState::Disconnected,
            host: settings::SQL_HOST,
        }
    }


    fn new_override(host: &'static str) -> Self {
        Self {
            adapter: ConnectionState::Disconnected,
            host,
        }
    }


    async fn connect(&mut self) -> Result<(), SocketIOError> {
        self.adapter = ConnectionState::Connected(TcpStream::connect(self.host).await.map_err(|_| SocketIOError::Connecting)?);
        Ok(())
    }
    
    async fn write_inner(&mut self, value: &Vec<u8>) -> Result<(), SocketIOError> {
        let stream = connstate_check_open!(self.adapter, SocketIOError::Writing);
        stream.write_all(&value).await.map_err(|_| {
            self.adapter = ConnectionState::Disconnected;
            SocketIOError::Writing
        })?;
        Ok(())
    }


    pub async fn write(&mut self, value: &Vec<u8>) -> Result<(), SocketIOError> {
        if let ConnectionState::Disconnected = self.adapter {
            self.connect().await?;
        }
        for _ in 0..settings::INET_MAX_IO_RETRIES {
            if let Ok(val) = self.write_inner(value).await { return Ok(val); };
        }
        Err(SocketIOError::Reading)
    }


    async fn read_inner(&mut self, buf: &mut [u8]) -> Result<usize, SocketIOError> {
        let stream = connstate_check_open!(self.adapter, SocketIOError::Reading);
        stream.read(buf).await.map_err(|_| SocketIOError::Reading)
    }


    /// Read
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize, SocketIOError> {
        if let ConnectionState::Disconnected = self.adapter {
            self.connect().await?;
        }
        for _ in 0..settings::INET_MAX_IO_RETRIES {
            if let Ok(val) = self.read_inner(buf).await { return Ok(val); }
        }
        Err(SocketIOError::Writing)
    }


}




#[cfg(feature = "inet_testing")]
#[cfg(test)]
mod test {

    use super::*;

    #[tokio::test]
    async fn ping_pong_hello() {
        let mut test = SocketHandler::new_override("[::1]:42069");
        test.write(&vec!['H' as u8 , 'e' as u8 , 'l' as u8 , 'l' as u8 , 'o' as u8 , ',' as u8 , ' ' as u8 , 'w' as u8 , 'o' as u8 , 'r' as u8 , 'l' as u8 , 'd' as u8 , '!' as u8]).await.unwrap();
        let mut buffer: [u8; 13] = [0; 13];
        let read_bytes: usize = test.read(&mut buffer).await.unwrap();
        assert_eq!(read_bytes, 13);
        assert_eq!(String::from_utf8_lossy(&buffer), "Hello, world!");
    }

}

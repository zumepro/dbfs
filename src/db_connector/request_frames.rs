use crate::settings;

enum Capabilities {
    ClientMysql = 1,
    FoundRows = 2,
    ConnectWithDb = 8,
    Compress = 32,
    LocalFiles = 128,
    IgnoreSpace = 256,
    ClientProtocol41 = 1 << 9,
    ClientInteractive = 1 << 10,
    Ssl = 1 << 11,
    Transactions = 1 << 13,
    SecureConnection = 1 << 15,
    MultiStatements = 1 << 16,
    MultiResults = 1 << 17,
    PsMultiResults = 1 << 18,
    PluginAuth = 1 << 19,
    ConnectAttrs = 1 << 20,
    PluginAuthLenencClientData = 1 << 21,
    ClientCanHandleExpiredPasswords = 1 << 22,
    ClientSessionTrack = 1 << 23,
    ClientDeprecateEof = 1 << 24,
}


#[derive(Default)]
enum Collations {
    LatinSwedishCi = 8,
    #[default]
    Utf8Mb3GeneralCi = 33,
    Binary = 63,
}




const DEFAULT_CAPABILITES: u32 = Capabilities::ConnectWithDb as u32;
const DEFAULT_MAX_PACKET_SIZE: u32 = 0xffffffff;




trait FrameToBytes {
    fn to_bytes(&self) -> Vec<u8>;
}




struct HandshakeResponse {
    client_capabilities: u32,
    max_packet_size: u32,
    collation: u8,
    username: &'static str,
}


impl Default for HandshakeResponse {
    fn default() -> Self {
        Self {
            client_capabilities: DEFAULT_CAPABILITES,
            max_packet_size: DEFAULT_MAX_PACKET_SIZE,
            collation: Collations::default() as u8,
            username: settings::SQL_USER,
        }
    }
}


impl FrameToBytes for HandshakeResponse {
    fn to_bytes(&self) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();
        result.extend_from_slice(&self.client_capabilities.to_le_bytes());
        result.extend_from_slice(&self.max_packet_size.to_le_bytes());
        result.push(self.collation);
        result.extend_from_slice(&[0; 19]);
        result
    }
}


#[cfg(test)]
mod test {


    use super::*;


    #[test]
    fn handshake_frame_to_bytes() {
        let frame = HandshakeResponse::default();
        assert_eq!(frame.to_bytes(), vec![0x08, 0x0, 0x0, 0x0, 0xff, 0xff, 0xff, 0xff, 33, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0]);
    }
}

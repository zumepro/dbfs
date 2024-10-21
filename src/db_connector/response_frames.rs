enum ResponseFrameType {
    OK,
    ERR,
    ResultsetRow,
}


pub struct SqlResponseFrame {
    packet: Vec<u8>,
    frame_type: ResponseFrameType,
}


impl SqlResponseFrame {
    fn hint_type(&self) -> Result<ResponseFrameType, ()> {
        if self.packet[0] == 0x00 || self.packet[0] == 0xfe { // Ok packet

        }

        Err(()) // Frame type not supported (or invalid)
    }
}

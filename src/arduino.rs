use std::{time::Duration, usize};

use tokio_serial::SerialPortBuilderExt;

#[derive(Debug)]
#[repr(C)]
pub struct Arduino {
    pub port: Box<dyn tokio_serial::SerialPort>,
    pub baud_rate: u32,
    pub data_collection: Vec<Vec<()>>,
    serial_buffer: Vec<u8>,
}

#[derive(Debug)]
pub enum PacketKind {
    String,
    Integer,
    Binary,
    Unknown,
}

impl Into<PacketKind> for u8 {
    fn into(self) -> PacketKind {
        match self {
            1 => PacketKind::String,
            2 => PacketKind::Integer,
            3 => PacketKind::Binary,
            _ => PacketKind::Unknown,
        }
    }
}

#[derive(Debug)]
pub struct Packet {
    packet_type: PacketKind,
}

impl Packet {
    pub fn new(packet_type: PacketKind) -> Self {
        Self { packet_type }
    }
}

impl Arduino {
    pub fn new(port_path: String, baud_rate: u32) -> Self {
        let mut port = tokio_serial::new(port_path.clone(), baud_rate)
            .timeout(Duration::from_millis(100))
            .open_native_async()
            .expect("Failed to open port");
        #[cfg(unix)]
        port.set_exclusive(false).expect(&format!(
            "Failed to set port '{:?}' exclusivity to false",
            port_path.clone()
        ));
        Self {
            port: Box::new(port),
            baud_rate,
            data_collection: Vec::new(),
            serial_buffer: vec![0; 64],
        }
    }

    pub async fn flush_buffer(&mut self) {
        self.serial_buffer.clear();
    }

    /// Reads from the serial data, determines the type from the first byte and then calls the
    /// appropriate read function
    pub async fn read_from_serial_packet(&mut self) {
        let packet: Packet;
        match self.port.read(self.serial_buffer.as_mut_slice()) {
            Ok(t) => {
                let packet_kind: PacketKind = self.serial_buffer[0].into();
                packet = Packet::new(packet_kind);
                self.flush_buffer().await; // Clear buffer after reading
                println!("{:?}", packet);
            }
            Err(_e) => (), // xd
        }
    }

    /// Read serial and convert the data to a utf-8 ASCII string
    pub async fn read_string_from_serial(&mut self) {}

    /// Reads serial and converts the data to an integer
    pub async fn read_integer_from_serial(&mut self) {}

    /// Reads the raw binary from serial
    pub async fn read_binary_from_serial(&mut self) {}
}

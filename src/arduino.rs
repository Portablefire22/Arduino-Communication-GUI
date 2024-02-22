use std::{io, time::Duration, usize};

use tokio_serial::SerialPortBuilderExt;

#[derive(Debug)]
#[repr(C)]
pub struct Arduino {
    pub port: Option<Box<dyn tokio_serial::SerialPort>>,
    pub baud_rate: Option<u32>,
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
    packet_id: u8, // The arduino will probably send data relating to multiple things, this will
    // allow for the packet to be assigned to something
    raw_data: Vec<u8>,
}

impl Packet {
    pub fn new(packet_type: PacketKind, packet_id: u8, raw_data: Vec<u8>) -> Self {
        Self {
            packet_type,
            packet_id,
            raw_data,
        }
    }
}

impl Arduino {
    /// Returns a completely empty Arduino class, ready for manipulation
    pub fn new() -> Self {
        Self {
            port: None,
            baud_rate: None,
            data_collection: Vec::new(),
            serial_buffer: vec![0; 32],
        }
    }

    /// Connects to the specified port with the given baud rate
    /// Panics if the port cannot be opened
    pub fn connect(&mut self, port_path: String, baud_rate: u32) {
        let mut port = tokio_serial::new(port_path.clone(), baud_rate)
            .timeout(Duration::from_millis(100))
            .open_native_async()
            .expect("Failed to open port");
        #[cfg(unix)]
        port.set_exclusive(false).expect(&format!(
            "Failed to set port '{:?}' exclusivity to false",
            port_path.clone()
        ));
        self.port = Some(Box::new(port));
        self.baud_rate = Some(baud_rate);
    }

    /// Wipes all data in the buffer and then resizes the buffer
    pub fn modify_buffer_size(&mut self, size: usize) {
        self.serial_buffer.clear();
        self.serial_buffer.resize(size, 0);
    }

    pub async fn read_loop(&mut self) {
        match self.port {
            Some(_) => {
                loop {
                    match self
                        .port
                        .as_mut()
                        .unwrap()
                        .read(self.serial_buffer.as_mut_slice())
                    {
                        Ok(t) => {
                            /*let recieved =
                                String::from_utf8_lossy(&serial_buffer[..t]);
                            println!("{}", recieved);*/
                            println!("{:?}", &self.serial_buffer[..t]);
                            println!("----------------");
                        }
                        Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                        Err(_e) => (),
                    }
                }
            }
            _ => {
                eprintln!("Arduino is not connected!");
            }
        }
    }

    pub async fn flush_buffer(&mut self) {
        self.serial_buffer.clear();
    }

    /// Reads from the serial data, determines the type from the first byte and then calls the
    /// appropriate read function
    pub async fn read_from_serial_packet(&mut self) {
        let packet: Packet;
        match self
            .port
            .as_mut()
            .unwrap()
            .read(self.serial_buffer.as_mut_slice())
        {
            Ok(t) => {
                let packet_kind: PacketKind = self.serial_buffer[0].into();
                let packet_id: u8 = self.serial_buffer[1];
                packet = Packet::new(packet_kind, packet_id, self.serial_buffer[2..].to_vec());
                self.flush_buffer().await; // Clear buffer after reading
                                           //println!("{:?}", packet);
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

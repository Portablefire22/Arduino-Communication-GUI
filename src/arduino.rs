use std::{
    time::{Duration, Instant, SystemTime},
    usize,
};

use colored::Colorize;
use tokio::sync::mpsc;
use tokio_serial::SerialPortBuilderExt;

#[derive(Debug)]
#[repr(C)]
pub struct Arduino {
    pub port: Option<Box<dyn tokio_serial::SerialPort>>,
    pub baud_rate: Option<u32>,
    serial_buffer: Vec<u8>,
}

#[derive(Debug, Clone)]
pub enum PacketData {
    Integer(isize, u8, Instant),
    String(String, u8, Instant),
    None(),
}

impl PacketData {
    pub fn display_variant(&self) -> &str {
        match self {
            Self::Integer(_, _, _) => "Integer",
            Self::String(_, _, _) => "String",
            _ => "None",
        }
    }
}

#[derive(Debug, Clone)]
pub enum ThreadMSG {
    Start((String, usize)), // Port path & baud rate
    Data(PacketData),       // Data ID & Data
    Disconnect(),
}

#[derive(Debug, PartialEq, Clone)]
pub enum PacketKind {
    String,
    PosInteger,
    NegInteger,
    Binary,
    Unknown,
}

impl Into<PacketKind> for u8 {
    fn into(self) -> PacketKind {
        match self {
            1 => PacketKind::String,
            2 => PacketKind::PosInteger,
            3 => PacketKind::NegInteger,
            4 => PacketKind::Binary,
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
    constructed_data: PacketData,
}

impl Packet {
    pub fn new(packet_type: PacketKind, packet_id: u8, raw_data: Vec<u8>) -> Self {
        Self {
            packet_type,
            packet_id,
            raw_data,
            constructed_data: PacketData::None(),
        }
    }
}

impl Arduino {
    /// Returns a completely empty Arduino class, ready for manipulation
    pub fn new() -> Self {
        Self {
            port: None,
            baud_rate: None,
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

    /// Disconnects from the current port
    pub fn disconnect(&mut self) {
        match self.port {
            Some(_) => {
                self.port = None;
                self.baud_rate = None;
            }
            _ => {
                eprintln!("Cannot disconnect: Arduino is not connected!");
            }
        }
    }

    /// Wipes all data in the buffer and then resizes the buffer
    pub fn modify_buffer_size(&mut self, size: usize) {
        self.serial_buffer.clear();
        self.serial_buffer.resize(size, 0);
    }

    pub fn read_loop(&mut self, rx: &mut mpsc::Receiver<ThreadMSG>, tx: mpsc::Sender<ThreadMSG>) {
        match self.port {
            Some(_) => {
                loop {
                    self.read_from_serial_packet(tx.clone());

                    // Break if a disconnect message is sent
                    match rx.try_recv() {
                        Err(mpsc::error::TryRecvError::Empty) => {}
                        Err(mpsc::error::TryRecvError::Disconnected) => break,
                        Ok(t) => match t {
                            ThreadMSG::Disconnect() => break,
                            _ => {}
                        },
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
    pub fn read_from_serial_packet(&mut self, tx: mpsc::Sender<ThreadMSG>) {
        let mut packet: Packet;
        match self
            .port
            .as_mut()
            .unwrap()
            .read(self.serial_buffer.as_mut_slice())
        {
            Ok(_t) => {
                let packet_kind: PacketKind = self.serial_buffer[0].into();
                if packet_kind == PacketKind::Unknown {
                    // Unknown packets are likely corrupt and
                    // will cause a panic, so let's just return
                    eprintln!("Packet Errror: Recevied packet with unknown type!");
                    return;
                };
                let packet_id: u8 = self.serial_buffer[1];
                let mut tmp_vec: Vec<u8> = vec![0; self.serial_buffer.len() - 3];
                let mut j = 0;
                for i in self.serial_buffer[2..].into_iter() {
                    if *i == 0x0D || j == tmp_vec.len() {
                        break;
                    }
                    tmp_vec[j] = *i;
                    j += 1;
                }
                tmp_vec.resize(j, 0);
                packet = Packet::new(packet_kind, packet_id, tmp_vec);
                match packet.packet_type {
                    PacketKind::String => self.read_string_from_serial(&mut packet),
                    PacketKind::PosInteger => self.read_integer_from_serial(false, &mut packet),
                    PacketKind::NegInteger => self.read_integer_from_serial(true, &mut packet),
                    PacketKind::Binary => (), // Not implemented, not sure if this is needed
                    _ => unreachable!(),
                }
                crate::app::send_thread_msg(tx, ThreadMSG::Data(packet.constructed_data));
            }
            Err(_e) => (), // xd
        }
    }

    /// Read serial and convert the data to a utf-8 ASCII string
    pub fn read_string_from_serial(&mut self, packet: &mut Packet) {
        let mut tmp_string: String = "".to_owned();
        for byte in (&packet.raw_data).into_iter() {
            if *byte != 0 {
                tmp_string.push(*byte as char);
            }
        }
        packet.constructed_data = PacketData::String(tmp_string, packet.packet_id, Instant::now());
    }

    /// Reads serial and converts the data to an integer, boolean determines
    /// if the integer is positive or negative
    pub fn read_integer_from_serial(&mut self, is_negative: bool, packet: &mut Packet) {
        let mut tmp = 0;
        let max_bytes = isize::BITS / 8;
        for (i, byte) in (&packet.raw_data).into_iter().enumerate() {
            let mut tmp_byte = byte.clone() as i16;
            if is_negative {
                if tmp_byte == 0 {
                    continue;
                }
                tmp_byte -= 0xFF;
                if i == 0 {
                    // Two's compliment ?
                    tmp_byte -= 1;
                }
            }
            if i == max_bytes
                .try_into()
                .expect("Packet integer exceeds the integer limit!")
            {
                tmp += (tmp_byte as isize) << ((i * 8) - 1);
            } else if i > max_bytes
                .try_into()
                .expect("Packet integer exceeds the integer limit!")
            {
                eprintln!("Integer exceeds the integer limit, stopping!");
                break;
            } else {
                tmp += (tmp_byte as isize) << i * 8;
            }
        }
        packet.constructed_data = PacketData::Integer(tmp, packet.packet_id, Instant::now());
    }

    /// Reads the raw binary from serial
    pub async fn read_binary_from_serial(&mut self) {}
}

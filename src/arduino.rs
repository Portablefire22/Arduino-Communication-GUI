use std::time::Duration;

pub struct Arduino {
    port: Box<dyn serialport::SerialPort>,
    baud_rate: u32,
    data_collection: Vec<Vec<()>>,
}

impl Arduino {
    pub fn new(port_path: String, baud_rate: u32) -> Self {
        let port = serialport::new(port_path, baud_rate)
            .timeout(Duration::from_millis(100))
            .open()
            .expect("Failed to open port");
        Self {
            port,
            baud_rate,
            data_collection: Vec::new(),
        }
    }

    pub fn read_serial(&mut self) {}
}

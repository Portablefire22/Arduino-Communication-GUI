use std::time::Duration;

use tokio_serial::SerialPortBuilderExt;

#[derive(Debug)]
pub struct Arduino {
    pub port: Box<dyn tokio_serial::SerialPort>,
    pub baud_rate: u32,
    pub data_collection: Vec<Vec<()>>,
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
        }
    }

    pub fn read_serial(&mut self) {}
}

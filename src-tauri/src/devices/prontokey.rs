use super::BaseDevice;

use std::thread;
use std::time::Duration;

use serde_json::Value;

/// A representation of a ProntoKey device.
pub struct ProntoKeyDevice {
	address: String
}

impl BaseDevice for ProntoKeyDevice {
	fn num_dials(&self) -> u8 { 2 }
	fn num_rows(&self) -> u8 { 3 }
	fn num_columns(&self) -> u8 { 3 }

	fn id(&self) -> String {
		format!("pk-{}", self.address)
	}
	fn name(&self) -> String {
		String::from("ProntoKey")
	}
	fn r#type(&self) -> u8 { 7 }

	fn key_down(&self, key: u8) {
		println!("{} down {}", &self.address, key);
	}
	fn key_up(&self, key: u8) {
		println!("{} up {}", &self.address, key);
	}
	fn dial_rotate(&self, dial: u8, ticks: i16) {
		println!("{} rotate {} by {}", &self.address, dial, ticks);
	}
}

impl ProntoKeyDevice {
	/// Attempt to open a serial connection with the device and handle incoming data.
	pub async fn init(port: String) {
		let mut device: Option<ProntoKeyDevice> = Option::None;
		let mut last_key: u8 = 0;
		let mut last_sliders: Vec<i16> = vec![0; 2];

		let mut port = match
			serialport::new(port, 57600)
			.timeout(Duration::from_millis(10))
			.open()
		{
			Ok(p) => p,
			Err(e) => panic!("Failed to open serial port: {}", e.description)
		};
		let _ = port.write("register".as_bytes()); 

		let mut serial_buf: Vec<u8> = vec![0; 1024];
		let mut holding_string = String::from("");

		loop {
			match port.read(serial_buf.as_mut_slice()) {
				Ok(t) => {
					match std::str::from_utf8(&serial_buf[..t]) {
						Ok(data) => holding_string += data,
						Err(_) => break
					}
					// Iterate through JSON objects received from device that are being held in the buffer.
					while holding_string.contains("}") {
						let index = holding_string.find("}").unwrap_or_default();
						let chunk = holding_string[..=index].trim();
						let j: Value = match serde_json::from_str(chunk) {
							Ok(data) => data,
							Err(_) => continue
						};
						holding_string = holding_string[(index + 1)..].to_owned();
						
						// If the device is uninitialised, attempt to read its MAC address and initialise.
						if device.is_none() {
							match &j["address"] {
								Value::String(address) => {
									device = Some(ProntoKeyDevice { address: address.clone() });
									match &device {
										Some(device) => super::DEVICES.lock().unwrap().push(super::DeviceInfo::new(device)),
										_ => {}
									}
								},
								_ => {}
							}
							continue;
						}

						let device = device.as_ref().unwrap();

						// Handle key presses and releases.
						match &j["key"] {
							Value::Number(num) => {
								match num.as_u64().unwrap_or_default() as u8 {
									0 => device.key_up(last_key),
									val => {
										device.key_down(val);
										last_key = val;
									}
								}
							},
							_ => {}
						}

						// Handle slider value changes.
						match &j["slider0"] {
							Value::Number(val) => {
								let val: i16 = match val.as_i64() {
									Some(v) => v as i16,
									_ => last_sliders[0]
								};
								device.dial_rotate(0, val - last_sliders[0]);
								last_sliders[0] = val;
							},
							_ => ()
						}
						match &j["slider1"] {
							Value::Number(val) => {
								let val: i16 = match val.as_i64() {
									Some(v) => v as i16,
									_ => last_sliders[1]
								};
								device.dial_rotate(1, val - last_sliders[1]);
								last_sliders[1] = val;
							},
							_ => {}
						}
					}
				},
				Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => (),
				Err(e) => eprintln!("{:?}", e)
			}
			thread::sleep(Duration::from_millis(10));
		}
	}
}
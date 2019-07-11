use usbmux::Client;

fn main() {
    let mut client = Client::new().unwrap();
    let devices = client.devices().unwrap();
    let device = devices.first().unwrap();

    let mut stream = client.connection(device.device_id, 2345).unwrap();

    // copied this from the python's peertalk.py. able to get messages to show up on ipad by sending
    // this.
    stream.send_raw(b"\x00\x00\x00\x01\x00\x00\x00e\x00\x00\x00\x00\x00\x00\x00\n".to_vec()).unwrap();
    stream.send_raw(b"\x00\x00\x00\x06powpow".to_vec()).unwrap();
    std::thread::sleep(std::time::Duration::from_secs(999));
}
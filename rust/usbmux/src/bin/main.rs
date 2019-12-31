use usbmux::Client;

use bmp::Pixel;
use std::fs::File;
use std::io::Write;
use structure::{structure, structure_impl};
use xcb::Setup;
use xcb_util::image::shm;

// code for connecting to ipad, we'll need it later
fn main() {
    let mut client = Client::new().unwrap();
    let devices = client.devices().unwrap();
    let device = devices.first().unwrap();

    let mut stream = client.connection(device.device_id, 2345).unwrap();

    let image = grab_image();
    let image_data = image.data();

    //let image_data = "ajwiefjaoiwef".as_bytes();

    // copied this from the python's peertalk.py. able to get messages to show up on ipad by sending
    // this.
    send(image_data, &mut (stream.inner)).unwrap();
    //    stream
    //        .send_raw(b"\x00\x00\x00\x01\x00\x00\x00e\x00\x00\x00\x00\x00\x00\x00\n".to_vec())
    //        .unwrap();
    //    stream.send_raw(b"\x00\x00\x00\x06powpow".to_vec()).unwrap();
}

// got this from peertalk.py, copy of the python structure
fn send(data: &[u8], writer: &mut impl Write) -> Result<(), Box<dyn std::error::Error>> {
    // send the header
    let header_structure = structure!("!IIII");
    let message_type = 101;
    header_structure.pack_into(writer, 1, message_type, 0, data.len() as u32 + 4)?;

    let payload_structure = structure!("!I22380544s");
    //    let payload_structure = structure!("!I13S");
    payload_structure.pack_into(writer, data.len() as u32, data)?;

    Ok(())
}

fn grab_image() -> xcb_util::image::shm::Image {
    let (conn, preferred_display_no) = xcb::Connection::connect(None).unwrap();
    let setup = conn.get_setup();

    // first screen is the external display, 0 is the laptop screen
    let screen = setup.roots().nth(0).unwrap();

    // maybe grab these programatically and select the highest one automatically? or this will be
    // user configurable?
    let depth = 32;

    // dimensions and position of VIRTUAL1 (from xrandr): 2048x2732+0+0
    let mut image = shm::create(&conn, depth, 2048, 2732).unwrap();

    let drawable = screen.root();
    // we should get these values from X somehow... for the display that we want to share
    let offset = (0, 0);

    // for the plane mask, we're just doing what OBS does. see
    // https://github.com/obsproject/obs-studio/blob/c938ea712bce0e9d8e0cf348fd8f77725122b9a5/plugins/linux-capture/xshm-input.c#L424
    let plane_mask = !0;

    shm::get(&conn, drawable, &mut image, offset.0, offset.1, plane_mask).unwrap();
    println!("xcb image format {:?}", image.format());
    println!("xcb image bpp: {:?}", image.bpp());

    image
}

fn _view_color_depths(setup: &Setup) {
    for format in setup.pixmap_formats() {
        println!("{:?}", format.depth());
    }
}

// these are more raw bindings to shm
fn _xcb_raw_main() {
    use xcb::ffi::xcb_get_image_unchecked;
    let _image = unsafe {
        xcb_get_image_unchecked(
            unimplemented!(),
            unimplemented!(),
            unimplemented!(),
            unimplemented!(),
            unimplemented!(),
            unimplemented!(),
            unimplemented!(),
            unimplemented!(),
        )
    };
}

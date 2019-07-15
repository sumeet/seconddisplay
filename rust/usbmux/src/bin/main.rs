use usbmux::Client;

use bmp::Pixel;
use xcb::Setup;
use xcb_util::image::shm;

fn main() {
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

    let image = shm::get(&conn, drawable, &mut image, offset.0, offset.1, plane_mask).unwrap();
    println!("xcb image format {:?}", image.format());

    // TODO: convert pixmap to jpg in a fast way... right now even converting to bitmap is incredibly
    // slow the way we're doing it here. but at least we can see output on the screen!

    // we found out that there are 4 u8s in image.data() for each pixel
    // length of data is: 22380544
    // width * height = 5595136

    let mut bmp = bmp::Image::new(image.width() as u32, image.height() as u32);
    println!("{:?} {:?}", image.width(), image.height());
    for x in 0..image.width() as u32 {
        for y in 0..image.height() as u32 {
            let pixel = image.get(x, y);
            let blue = pixel & 255;
            let green = (pixel >> 8) & 255;
            let red = (pixel >> 16) & 255;
            let bmp_pixel = Pixel::new(red as u8, green as u8, blue as u8);
            bmp.set_pixel(x, y, bmp_pixel);
        }
    }
    bmp.save("screen.bmp").unwrap();
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

// code for connecting to ipad, we'll need it later
fn _oldmain() {
    let mut client = Client::new().unwrap();
    let devices = client.devices().unwrap();
    let device = devices.first().unwrap();

    let mut stream = client.connection(device.device_id, 2345).unwrap();

    // copied this from the python's peertalk.py. able to get messages to show up on ipad by sending
    // this.
    stream
        .send_raw(b"\x00\x00\x00\x01\x00\x00\x00e\x00\x00\x00\x00\x00\x00\x00\n".to_vec())
        .unwrap();
    stream.send_raw(b"\x00\x00\x00\x06powpow".to_vec()).unwrap();
    std::thread::sleep(std::time::Duration::from_secs(999));
}

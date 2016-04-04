extern crate rscam;

use std::io::Write;

fn main() {
    let mut camera = rscam::new("/dev/video0").unwrap();

    camera.start(&rscam::Config {
        interval: (1, 30),      // 30 fps.
        resolution: (1280, 720),
        format: b"RGB3",
        ..Default::default()
    }).unwrap();

    loop {
        match camera.capture() {
            Ok(_) => println!("frame"),
            Err(_) => println!("no frame"),
        }
    }
}

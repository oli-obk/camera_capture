extern crate camera_capture;
extern crate image;

use std::fs::File;
use std::path::Path;

fn main() {
    let cam = camera_capture::create(0).unwrap();

    let mut cam_iter = cam.fps(5.0).unwrap().start().unwrap();
    let img = cam_iter.next().unwrap();

    let file_name = "test.png";
    let path = Path::new(&file_name);
    let _ = &mut File::create(&path).unwrap();
    img.save(&path).unwrap();

    println!("img saved to {}", file_name);

}

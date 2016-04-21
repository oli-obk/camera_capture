extern crate camera_capture;

fn main() {
    let cam = camera_capture::create(0).unwrap();
    let cam = cam.fps(5.0).unwrap().start().unwrap();
    for _image in cam {
        println!("frame");
    }
    println!("done");
}

extern crate video_input;

fn main() {
    let cam = video_input::create("/dev/video0").unwrap();
    for _image in cam.start().unwrap() {
        println!("frame");
    }
    println!("done");
}

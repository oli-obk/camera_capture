extern crate video_input;

fn main() {
    let cam = video_input::create("/dev/video0").unwrap();
    let cam = cam.fps(5.0).unwrap().start().unwrap();
    for _image in cam {
        println!("frame");
    }
    println!("done");
}

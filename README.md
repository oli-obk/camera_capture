# Webcam capturing in Rust

## Description

Captures webcam images and offers access to them through an iterator. Works with
v4l2 on Linux. OSX is not supported.

## TODO

* [ ] threaded access through channel `Receiver`
* [ ] automatic webcam detection and selection

## Documentation
You can create the documentation (locally) by executing `cargo doc --no-deps --open`.

## Example

```rust
extern crate camera_capture;

fn main() {
    let cam = camera_capture::create(0).unwrap();
    let cam = cam.fps(5.0).unwrap().start().unwrap();
    for _image in cam {
        println!("frame");
    }
    println!("done");
}
```

## Piston Example

* run via `cargo run --example piston`
* [source](https://github.com/oli-obk/camera_capture/blob/master/examples/piston.rs)

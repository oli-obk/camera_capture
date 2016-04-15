# Webcam capturing in Rust

## Description

Captures webcam images and offers access to them through an iterator. Works with
v4l2 on linux.

## TODO

* [ ] Windows Support (WIP)
* [ ] threaded access through channel `Receiver`
* [ ] automatic webcam detection and selection

## Example

```rust
extern crate video_input;

fn main() {
    let cam = video_input::create("/dev/video0").unwrap();
    let cam = cam.fps(5.0).unwrap().start().unwrap();
    for _image in cam {
        println!("frame");
    }
    println!("done");
}
```

## Piston Example

```rust
extern crate video_input;
extern crate piston_window;
extern crate image;
extern crate texture;

use piston_window::{PistonWindow, Texture, WindowSettings, TextureSettings, clear};
use image::ConvertBuffer;

fn main() {
    let window: PistonWindow =
        WindowSettings::new("piston: image", [300, 300])
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut tex: Option<Texture<_>> = None;
    let (sender, receiver) = std::sync::mpsc::channel();
    let imgthread = std::thread::spawn(move || {
        let cam = video_input::create("/dev/video0").unwrap()
                                                    .fps(5.0)
                                                    .unwrap()
                                                    .start()
                                                    .unwrap();
        for frame in cam {
            if let Err(_) = sender.send(frame.convert()) {
                break;
            }
        }
    });

    for e in window {
        if let Ok(frame) = receiver.try_recv() {
            if let Some(mut t) = tex {
                t.update(&mut *e.encoder.borrow_mut(), &frame).unwrap();
                tex = Some(t);
            } else {
                tex = Texture::from_image(&mut *e.factory.borrow_mut(), &frame, &TextureSettings::new()).ok();
            }
        }
        e.draw_2d(|c, g| {
            clear([1.0; 4], g);
            if let Some(ref t) = tex {
                piston_window::image(t, c.transform, g);
            }
        });
    }
    drop(receiver);
    imgthread.join().unwrap();
}
```
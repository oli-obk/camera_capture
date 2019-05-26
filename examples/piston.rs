extern crate camera_capture;
extern crate image;
extern crate piston_window;

use image::ConvertBuffer;
use piston_window::{clear, PistonWindow, Texture, TextureSettings, WindowSettings};

fn main() {
    let mut window: PistonWindow = WindowSettings::new("piston: image", [300, 300])
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut tex: Option<Texture<_>> = None;
    let (sender, receiver) = std::sync::mpsc::channel();
    let imgthread = std::thread::spawn(move || {
        let res = camera_capture::create(0);
        if let Err(e) = res {
            eprintln!("could not open camera: {}", e);
            std::process::exit(1);
        }
        let cam = res.unwrap().fps(5.0).unwrap().start().unwrap();
        for frame in cam {
            if sender.send(frame.convert()).is_err() {
                break;
            }
        }
    });

    while let Some(e) = window.next() {
        if let Ok(frame) = receiver.try_recv() {
            if let Some(mut t) = tex {
                t.update(&mut window.encoder, &frame).unwrap();
                tex = Some(t);
            } else {
                tex =
                    Texture::from_image(&mut window.factory, &frame, &TextureSettings::new()).ok();
            }
        }
        window.draw_2d(&e, |c, g| {
            clear([1.0; 4], g);
            if let Some(ref t) = tex {
                piston_window::image(t, c.transform, g);
            }
        });
    }
    drop(receiver);
    imgthread.join().unwrap();
}

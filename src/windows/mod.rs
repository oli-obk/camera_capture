use lazy_static::lazy_static;

use crate::error::Error;

pub type Frame = Vec<u8>;

pub struct ImageIterator {
    camera: escapi::Device<'static>,
}

pub struct Builder {
    resolution: (u32, u32),
    camera_id: u32,
}

pub fn create(i: u32) -> std::io::Result<Builder> {
    Ok(Builder {
        camera_id: i,
        resolution: (640, 480),
    })
}

impl Iterator for ImageIterator {
    type Item = image::ImageBuffer<image::Rgb<u8>, Frame>;
    fn next(&mut self) -> Option<Self::Item> {
        let wdt = self.camera.width();
        let hgt = self.camera.height();
        match self.camera.capture(50) {
            Ok(frame) => {
                let len = (wdt * hgt) as usize;
                let mut buf = vec![0; len * 3];
                for i in 0..len {
                    buf[i * 3 + 2] = frame[i * 4];
                    buf[i * 3 + 1] = frame[i * 4 + 1];
                    buf[i * 3] = frame[i * 4 + 2];
                }
                image::ImageBuffer::from_raw(wdt, hgt, buf)
            }
            Err(_) => None,
        }
    }
}

impl Builder {
    pub fn fps(self, _fps: f64) -> Result<Self, Error> {
        Ok(self)
    }

    pub fn resolution(mut self, wdt: u32, hgt: u32) -> Result<Self, Error> {
        self.resolution = (wdt, hgt);
        Ok(self)
    }

    pub fn start(self) -> std::io::Result<ImageIterator> {
        lazy_static! {
            static ref CAMERAS: escapi::Cameras = escapi::init().unwrap();
        }
        match CAMERAS.init(self.camera_id, self.resolution.0, self.resolution.1, 10) {
            Ok(cam) => Ok(ImageIterator {
                camera: cam,
            }),
            Err(err) => Err(std::io::Error::new(std::io::ErrorKind::Other, err)),
        }
    }
}

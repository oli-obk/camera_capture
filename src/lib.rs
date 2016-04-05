extern crate rscam;
extern crate image;

use std::path::Path;
use std::default::Default;

pub struct ImageIterator {
    camera: rscam::Camera,
}

pub struct Builder {
    fps: (u32, u32),
    resolution: (u32, u32),
    camera: rscam::Camera,
}

pub fn create<P: AsRef<Path>>(p: P) -> std::io::Result<Builder> {
    let camera = try!(rscam::Camera::new(p.as_ref().to_str().expect("unicode path")));
    Ok(Builder {
        camera: camera,
        resolution: (640, 480),
        fps: (1, 10),
    })
}

#[derive(Debug)]
pub enum FpsError {
    Negative,
    ResolutionTooHigh,
}

#[derive(Debug)]
pub enum ResolutionError {
    ZeroSized,
}

impl Builder {
    pub fn fps(mut self, fps: f64) -> Result<Self, FpsError> {
        if fps < 0.0 {
            return Err(FpsError::Negative);
        }
        self.fps = (10, (fps * 10.0) as u32);
        if self.fps.0 == 0 || self.fps.1 == 0 {
            Err(FpsError::ResolutionTooHigh)
        } else {
            Ok(self)
        }
    }
    pub fn resolution(mut self, wdt: u32, hgt: u32) -> Result<Self, ResolutionError> {
        self.resolution = (wdt, hgt);
        if wdt == 0 || hgt == 0 {
            Err(ResolutionError::ZeroSized)
        } else {
            Ok(self)
        }
    }
    pub fn start(mut self) -> std::io::Result<ImageIterator> {
        match self.camera.start(&rscam::Config {
            interval: self.fps,      // 30 fps.
            resolution: self.resolution,
            format: b"RGB3",
            ..Default::default()
        }) {
            Ok(()) => Ok(ImageIterator { camera: self.camera }),
            Err(rscam::Error::Io(io)) => Err(io),
            _ => unreachable!(),
        }
    }
}

impl Iterator for ImageIterator {
    type Item = image::RgbImage;
    fn next(&mut self) -> Option<Self::Item> {
        match self.camera.capture() {
            Ok(frame) => image::ImageBuffer::from_raw(frame.resolution.0, frame.resolution.1, frame.to_owned()),
            Err(_) => None,
        }
    }
}

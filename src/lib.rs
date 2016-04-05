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
    InvalidFps(rscam::IntervalInfo),
    Io(std::io::Error),
}

#[derive(Debug)]
pub enum ResolutionError {
    InvalidResolution(rscam::ResolutionInfo),
    Io(std::io::Error),
}

impl Builder {
    pub fn fps(mut self, fps: f64) -> Result<Self, FpsError> {
        if fps < 5.0 {
            self.fps = (1000, (fps * 1000.0) as u32);
        } else {
            self.fps = (1, fps as u32);
        }
        let intervals = match self.camera.intervals(b"RGB3", self.resolution) {
            Ok(intervals) => intervals,
            Err(rscam::Error::Io(io)) => return Err(FpsError::Io(io)),
            _ => unreachable!(),
        };
        match intervals {
            rscam::IntervalInfo::Discretes(ref v) => for &(a, b) in v {
                if a == self.fps.0 && b == self.fps.1 {
                    return Ok(self);
                }
            },
            rscam::IntervalInfo::Stepwise { min, max, step } => {
                if ((self.fps.0 - min.0) / step.0) * step.0 + min.0 == self.fps.0
                && ((self.fps.1 - min.1) / step.1) * step.1 + min.1 == self.fps.1
                && max.0 >= self.fps.0
                && max.1 >= self.fps.1 {
                    return Ok(self);
                }
            }
        }
        Err(FpsError::InvalidFps(intervals))
    }
    pub fn resolution(mut self, wdt: u32, hgt: u32) -> Result<Self, ResolutionError> {
        self.resolution = (wdt, hgt);
        let res = match self.camera.resolutions(b"RGB3") {
            Ok(res) => res,
            Err(rscam::Error::Io(io)) => return Err(ResolutionError::Io(io)),
            _ => unreachable!(),
        };
        match res {
            rscam::ResolutionInfo::Discretes(ref v) => for &(w, h) in v {
                if w == wdt && h == hgt {
                    return Ok(self);
                }
            },
            rscam::ResolutionInfo::Stepwise { min, max, step } => {
                if ((wdt - min.0) / step.0) * step.0 + min.0 == wdt
                && ((hgt - min.1) / step.1) * step.1 + min.1 == hgt
                && max.0 >= wdt
                && max.1 >= hgt {
                    return Ok(self);
                }
            }
        }
        Err(ResolutionError::InvalidResolution(res))
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
    type Item = image::ImageBuffer<image::Rgb<u8>, rscam::Frame>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.camera.capture() {
            Ok(frame) => image::ImageBuffer::from_raw(frame.resolution.0, frame.resolution.1, frame),
            Err(_) => None,
        }
    }
}

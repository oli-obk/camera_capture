use std::default::Default;

use crate::error::Error;

pub type Frame = rscam::Frame;

pub struct ImageIterator {
    camera: rscam::Camera,
}

pub struct Builder {
    fps: (u32, u32),
    resolution: (u32, u32),
    camera: rscam::Camera,
}

pub fn create(i: u32) -> std::io::Result<Builder> {
    Ok(Builder {
        camera: rscam::Camera::new(&format!("/dev/video{}", i))?,
        resolution: (640, 480),
        fps: (1, 10),
    })
}

impl Iterator for ImageIterator {
    type Item = image::ImageBuffer<image::Rgb<u8>, Frame>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.camera.capture() {
            Ok(frame) => image::ImageBuffer::from_raw(frame.resolution.0, frame.resolution.1, frame),
            Err(_) => None,
        }
    }
}

impl Builder {
    pub fn fps(mut self, fps: f64) -> Result<Self, Error> {
        if fps < 5.0 {
            self.fps = (1000, (fps * 1000.0) as u32);
        } else {
            self.fps = (1, fps as u32);
        }
        let intervals = match self.camera.intervals(b"RGB3", self.resolution) {
            Ok(intervals) => intervals,
            Err(rscam::Error::Io(io)) => return Err(Error::Io(io)),
            _ => unreachable!(),
        };
        match intervals {
            rscam::IntervalInfo::Discretes(ref v) => {
                for &(a, b) in v {
                    if a == self.fps.0 && b == self.fps.1 {
                        return Ok(self);
                    }
                }
                Err(Error::InvalidFps(v.iter().map(|&(a, b)| a as f64 / b as f64).collect()))
            }
            rscam::IntervalInfo::Stepwise { min, max, step } => {
                if ((self.fps.0 - min.0) / step.0) * step.0 + min.0 == self.fps.0
                    && ((self.fps.1 - min.1) / step.1) * step.1 + min.1 == self.fps.1
                    && max.0 >= self.fps.0
                    && max.1 >= self.fps.1 {
                    Ok(self)
                } else {
                    Err(Error::InvalidFps([min, max].iter().map(|&(a, b)| a as f64 / b as f64).collect()))
                }
            }
        }
    }

    pub fn resolution(mut self, wdt: u32, hgt: u32) -> Result<Self, Error> {
        self.resolution = (wdt, hgt);
        let res = match self.camera.resolutions(b"RGB3") {
            Ok(res) => res,
            Err(rscam::Error::Io(io)) => return Err(Error::Io(io)),
            _ => unreachable!(),
        };
        match res {
            rscam::ResolutionInfo::Discretes(ref v) => {
                for &(w, h) in v {
                    if w == wdt && h == hgt {
                        return Ok(self);
                    }
                }
                Err(Error::InvalidResolution(v.clone()))
            }
            rscam::ResolutionInfo::Stepwise { min, max, step } => {
                if ((wdt - min.0) / step.0) * step.0 + min.0 == wdt
                    && ((hgt - min.1) / step.1) * step.1 + min.1 == hgt
                    && max.0 >= wdt
                    && max.1 >= hgt {
                    Ok(self)
                } else {
                    Err(Error::InvalidResolution(vec![min, max]))
                }
            }
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

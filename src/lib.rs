#[cfg(windows)]
extern crate escapi;
#[cfg(linux)]
extern crate rscam;
#[cfg(windows)]
#[macro_use]
extern crate lazy_static;
extern crate image;

#[cfg(linux)]
use std::path::Path;
#[cfg(linux)]
use std::default::Default;

#[cfg(linux)]
pub use rscam::Frame;
#[cfg(windows)]
pub type Frame = Vec<u8>;

pub struct ImageIterator {
    #[cfg(linux)]
    camera: rscam::Camera,
    #[cfg(windows)]
    camera: escapi::Device<'static>,
}

pub struct Builder {
    #[cfg(linux)]
    fps: (u32, u32),
    resolution: (u32, u32),
    #[cfg(linux)]
    camera: rscam::Camera,
    #[cfg(windows)]
    camera_id: u32,
}

#[cfg(linux)]
pub fn create(i: u32) -> std::io::Result<Builder> {
    Ok(Builder {
        camera: try!(rscam::Camera::new(format!("/dev/video{}", i))),
        resolution: (640, 480),
        fps: (1, 10),
    })
}

#[cfg(windows)]
pub fn create(i: u32) -> std::io::Result<Builder> {
    Ok(Builder {
        camera_id: i,
        resolution: (640, 480),
    })
}

#[derive(Debug)]
pub enum Error {
    InvalidFps(Vec<f64>),
    InvalidResolution(Vec<(u32, u32)>),
    Io(std::io::Error),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::Io(err)
    }
}

impl Builder {
    #[cfg(linux)]
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
        Err(Error::InvalidFps(intervals))
    }
    #[cfg(windows)]
    pub fn fps(self, _fps: f64) -> Result<Self, Error> {
        Ok(self)
    }
    #[cfg(linux)]
    pub fn resolution(mut self, wdt: u32, hgt: u32) -> Result<Self, Error> {
        self.resolution = (wdt, hgt);
        let res = match self.camera.resolutions(b"RGB3") {
            Ok(res) => res,
            Err(rscam::Error::Io(io)) => return Err(Error::Io(io)),
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
        Err(Error::InvalidResolution(res))
    }
    #[cfg(windows)]
    pub fn resolution(mut self, wdt: u32, hgt: u32) -> Result<Self, Error> {
        self.resolution = (wdt, hgt);
        Ok(self)
    }
    #[cfg(linux)]
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
    #[cfg(windows)]
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

#[cfg(linux)]
impl Iterator for ImageIterator {
    type Item = image::ImageBuffer<image::Rgb<u8>, Frame>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.camera.capture() {
            Ok(frame) => image::ImageBuffer::from_raw(frame.resolution.0, frame.resolution.1, frame),
            Err(_) => None,
        }
    }
}

#[cfg(windows)]
impl Iterator for ImageIterator {
    type Item = image::ImageBuffer<image::Rgb<u8>, Frame>;
    fn next(&mut self) -> Option<Self::Item> {
        let wdt = self.camera.width();
        let hgt = self.camera.height();
        match self.camera.capture(50) {
            Ok(frame) => {
                let len = (wdt*hgt) as usize;
                let mut buf = vec![0; len*3];
                for i in 0..len {
                    buf[i*3 + 2] = frame[i*4];
                    buf[i*3 + 1] = frame[i*4 + 1];
                    buf[i*3] = frame[i*4 + 2];
                }
                image::ImageBuffer::from_raw(wdt, hgt, buf)
            },
            Err(_) => None,
        }
    }
}

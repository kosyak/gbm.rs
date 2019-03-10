extern crate drm;
extern crate gbm;

use drm::control::{crtc, framebuffer, Device as ControlDevice};
use gbm::{Device, Format, BufferObjectFlags};

use std::fs::{File, OpenOptions};
use std::os::unix::io::{AsRawFd, RawFd};

#[derive(Debug)]
// This is our customized struct that implements the traits in drm.
struct Card(File);

// Need to implement AsRawFd before we can implement drm::Device
impl AsRawFd for Card {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

impl drm::Device for Card {}
impl ControlDevice for Card {}

impl Card {
    fn open(path: &str) -> Self {
        let mut options = OpenOptions::new();
        options.read(true);
        options.write(true);
        Card(options.open(path).unwrap())
    }

    fn open_global() -> Self {
        Self::open("/dev/dri/card0")
    }

    // fn open_control() -> Self {
    //     Self::open("/dev/dri/controlD64")
    // }
}

fn main() {
    let card = Card::open_global();
    let gbm = Device::new(card).unwrap();

    // create a buffer
    let mut bo = gbm.create_buffer_object::<()>(
        1920, 1080,
        Format::ARGB8888,
        BufferObjectFlags::SCANOUT | BufferObjectFlags::WRITE
    ).unwrap();

    // write something to it (usually use import or egl rendering instead)
    let buffer = {
        let mut buffer = Vec::new();
        for i in 0..1920 {
            for _ in 0..1080 {
                buffer.push(if i % 2 == 0 { 0 } else { 255 });
            }
        }
        buffer
    };
    let _ = bo.write(&buffer).unwrap();

    // create a framebuffer from our buffer
    let fb_info = framebuffer::create(&gbm, &bo).unwrap();

    let res_handles = gbm.resource_handles().unwrap();
    let con = *res_handles.connectors().iter().next().unwrap();
    let crtc_handle = *res_handles.crtcs().iter().next().unwrap();
    let mode_list = gbm.get_modes(con).unwrap();
    let mode = mode_list.as_slice().iter().next().unwrap();

    // display it (and get a crtc, mode and connector before)
    crtc::set(&gbm, crtc_handle, fb_info.handle(), &[con], (0, 0), Some(*mode)).unwrap();
}



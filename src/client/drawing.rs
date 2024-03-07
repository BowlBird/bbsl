use std::{ffi::c_void, os::fd::BorrowedFd, ptr};

use nix::libc::{close, ftruncate, mmap, munmap, shm_open, shm_unlink, MAP_FAILED, MAP_SHARED, O_CREAT, O_EXCL, O_RDWR, PROT_READ, PROT_WRITE};
use rand::{distributions::Alphanumeric, Rng};
use wayland_client::{protocol::{wl_buffer::WlBuffer, wl_shm::{self, WlShm}, wl_surface::WlSurface}, QueueHandle};

use super::{AppState, Rect};

/**
 *  pool_data is a writable u32 array 
 *
 * Example:
 * *(pool_data.offset(((y * width + x) * 4) as isize) as *mut u32) = color
 */
#[derive(Clone)]
pub struct FrameBuffer {
    wl_buffer: WlBuffer,
    pub pool_data: *mut c_void,
    pub dimension: Rect,
}

pub trait Release {
    fn release(&self);
}

impl Release for &FrameBuffer {
    fn release(&self) {
        self.wl_buffer.destroy();
        
        unsafe {
            munmap(
                self.pool_data, 
                (self.dimension.height * self.dimension.width * 4) as usize
            );
        }
    }
}

pub fn generate_frame_buffer(dimension: &Rect, wl_shm: &WlShm, qh: &QueueHandle<AppState>) -> Result<FrameBuffer, ()> {

    if dimension.width == 0 || dimension.height == 0 {
        return Err(());
    }

    let stride = dimension.width * 4;
    let shm_pool_size = dimension.height * stride;

    let fd = unsafe {
        let random: String = rand::thread_rng()
            .sample_iter(Alphanumeric)
            .take(128)
            .map(char::from)
            .collect();
        let name = format!("/buffer-{}\0", random);
        let fd = shm_open(name.as_ptr() as *const i8, O_RDWR | O_CREAT | O_EXCL, 0600);
        shm_unlink(name.as_ptr() as *const i8);
        fd
    };

    if fd == -1 {
        return Err(())
    }

    unsafe {
        let result = ftruncate(fd, shm_pool_size as i64);
        if result == -1 {
            return Err(())
        }
    }

    let pool_data = unsafe {
        mmap(ptr::null_mut() as *mut c_void, shm_pool_size as usize, PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0)
    };

    if pool_data == MAP_FAILED {
        return Err(());
    }

    let pool = wl_shm
        .create_pool(
            unsafe {BorrowedFd::borrow_raw(fd) }, 
            shm_pool_size as i32, 
            qh, 
            ()
        );

    let index = 0;
    let offset = dimension.height * stride * index;

    let buffer = pool.create_buffer(
        offset as i32, 
        dimension.width as i32, 
        dimension.height as i32, 
        stride as i32, 
        wl_shm::Format::Xrgb8888, 
        qh, 
        ()
    );

    pool.destroy();
    unsafe {close(fd)};

    return Ok(FrameBuffer {wl_buffer: buffer, pool_data: pool_data, dimension: Rect {width: dimension.width, height: dimension.height}});
}


pub fn attach_to_surface(frame_buffer: Option<&FrameBuffer>, wl_surface: &WlSurface) {
    let buffer = match frame_buffer {
        Some(frame_buffer) => {
            Some(&frame_buffer.wl_buffer)
        }
        None => None
    };

    wl_surface.attach(buffer, 0, 0);
    wl_surface.damage_buffer(0, 0, i32::MAX, i32::MAX);
    wl_surface.commit();
}
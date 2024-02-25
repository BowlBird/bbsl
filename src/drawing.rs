use std::{ffi::c_void, os::fd::BorrowedFd, ptr};

use nix::libc::{close, ftruncate, mmap, munmap, shm_open, MAP_FAILED, MAP_SHARED, O_CREAT, O_EXCL, O_RDWR, PROT_READ, PROT_WRITE};
use rand::{distributions::Alphanumeric, Rng};
use wayland_client::{protocol::{wl_buffer::WlBuffer, wl_shm}, QueueHandle};

use crate::AppState;

pub fn draw_frame(state: &AppState, qh: &QueueHandle<AppState>) -> Result<WlBuffer, ()> {

    let dimension = state
        .dimension.as_ref()
        .expect("could not connect to dimension");
    
    let width = if dimension.width != 0 {
        dimension.width
    }
    else { 1 };
    let height = if dimension.height != 0 {
        dimension.height
    }
    else { 1 };

    let stride = width * 4;
    let shm_pool_size = height * stride;

    let fd = unsafe {
        let random: String = rand::thread_rng()
            .sample_iter(Alphanumeric)
            .take(128)
            .map(char::from)
            .collect();
        let name = format!("/buffer-{}\0", random);
        shm_open(name.as_ptr() as *const i8, O_RDWR | O_CREAT | O_EXCL, 0600)
        // syscall(SYS_memfd_create, "buffer", 0) as RawFd
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


    let _pool_data = unsafe {
        mmap(ptr::null_mut() as *mut c_void, shm_pool_size as usize, PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0)
    };

    if _pool_data == MAP_FAILED {
        return Err(());
    }

    let pool = state.shm.as_ref().unwrap().create_pool(unsafe {BorrowedFd::borrow_raw(fd) }, shm_pool_size as i32, &qh, ());

    let index = 0;
    let offset = height * stride * index;

    let buffer = pool.create_buffer(
        offset as i32, 
        width as i32, 
        height as i32, 
        stride as i32, 
        wl_shm::Format::Xrgb8888, 
        qh, 
        ()
    );
    pool.destroy();
    unsafe {
        close(fd);
        munmap(_pool_data, shm_pool_size as usize);
    };

    return Ok(buffer);
}
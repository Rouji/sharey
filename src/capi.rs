use core::{
    ffi::{c_char, CStr},
    ptr,
};

use crate::sharex::{
    config::Config,
    uploader::{Input, SharexUploader},
};

#[repr(C)]
pub struct ShareyConfig {
    _private: [u8; 0],
}

#[repr(C)]
pub struct ShareySharexUploader {
    _private: [u8; 0],
}

#[no_mangle]
pub unsafe extern "C" fn sharey_config_from_file(path: *const c_char) -> *mut ShareyConfig {
    if path.is_null() {
        return ptr::null_mut();
    }
    let path = match unsafe { CStr::from_ptr(path) }.to_str() {
        Ok(p) => p,
        _ => return ptr::null_mut(),
    };
    match Config::from_file(path) {
        Ok(c) => Box::into_raw(Box::new(c)) as *mut ShareyConfig,
        _ => ptr::null_mut(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn sharey_config_free(config: *mut ShareyConfig) {
    if !config.is_null() {
        drop(unsafe { Box::from_raw(config as *mut Config) });
    }
}

#[no_mangle]
pub unsafe extern "C" fn sharey_uploader_new() -> *mut ShareySharexUploader {
    Box::into_raw(Box::new(SharexUploader::new())) as *mut ShareySharexUploader
}

#[no_mangle]
pub unsafe extern "C" fn sharey_uploader_free(uploader: *mut ShareySharexUploader) {
    if !uploader.is_null() {
        drop(unsafe { Box::from_raw(uploader as *mut SharexUploader) });
    }
}

#[no_mangle]
pub unsafe extern "C" fn sharey_upload(config: *const ShareyConfig, path: *const c_char) -> bool {
    let Some(config) = (unsafe { config.cast::<Config>().as_ref() }) else {
        return false;
    };
    let path = match unsafe { CStr::from_ptr(path) }.to_str() {
        Ok(p) => p,
        _ => return false,
    };
    let input = match Input::from_file(path) {
        Ok(i) => i,
        _ => return false,
    };
    SharexUploader::new().upload(config, input).is_ok()
}

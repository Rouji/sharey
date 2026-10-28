use core::{
    ffi::{c_char, CStr},
    ptr,
};

use crate::sharex::{
    config::Config,
    uploader::{Input, SharexUploader},
};

macro_rules! opaque_type {
    ($name:ident) => {
        #[repr(C)]
        pub struct $name {
            _private: [u8; 0],
        }
    };
}

macro_rules! free_func {
    ($func_name:ident, $type:ident) => {
        #[no_mangle]
        pub unsafe extern "C" fn $func_name(obj: *mut $type) {
            if !obj.is_null() {
                drop(unsafe { Box::from_raw(obj as *mut $type) });
            }
        }
    };
}

opaque_type!(ShareyConfig);
opaque_type!(ShareySharexUploader);
opaque_type!(ShareyInput);

free_func!(sharey_uploader_free, ShareySharexUploader);
free_func!(sharey_config_free, ShareyConfig);
free_func!(sharey_input_free, ShareyInput);

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
pub unsafe extern "C" fn sharey_input_from_file(path: *const c_char) -> *mut ShareyInput {
    if path.is_null() {
        return ptr::null_mut();
    }
    let path = match unsafe { CStr::from_ptr(path) }.to_str() {
        Ok(p) => p,
        _ => return ptr::null_mut(),
    };
    match Input::from_file(path) {
        Ok(i) => Box::into_raw(Box::new(i)) as *mut ShareyInput,
        _ => ptr::null_mut(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn sharey_uploader_new() -> *mut ShareySharexUploader {
    Box::into_raw(Box::new(SharexUploader::new())) as *mut ShareySharexUploader
}

#[no_mangle]
pub unsafe extern "C" fn sharey_upload(
    uploader: *const ShareySharexUploader,
    config: *const ShareyConfig,
    input: *const ShareyInput,
) -> bool {
    let Some(uploader) = (unsafe { uploader.cast::<SharexUploader>().as_ref() }) else {
        return false;
    };
    let Some(config) = (unsafe { config.cast::<Config>().as_ref() }) else {
        return false;
    };
    let input = unsafe { Box::from_raw(input as *mut Input) };
    uploader.upload(config, *input).is_ok()
}

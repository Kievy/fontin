use super::{Activator, unique_paths};
use crate::error::Result;
use crate::model::Face;
use core_foundation::base::{CFRelease, TCFType};
use core_foundation::url::{CFURL, CFURLRef};
use std::ffi::c_void;

const SCOPE_SESSION: u32 = 3; // kCTFontManagerScopeSession

#[link(name = "CoreText", kind = "framework")]
unsafe extern "C" {
    fn CTFontManagerRegisterFontsForURL(
        url: CFURLRef,
        scope: u32,
        error: *mut *const c_void,
    ) -> bool;
    fn CTFontManagerUnregisterFontsForURL(
        url: CFURLRef,
        scope: u32,
        error: *mut *const c_void,
    ) -> bool;
}

#[derive(Default)]
pub struct CoreText;

impl Activator for CoreText {
    fn activate(&self, faces: &[&Face]) -> Result<()> {
        for p in unique_paths(faces) {
            if let Some(url) = CFURL::from_path(p, false) {
                let mut err = std::ptr::null();
                // Already-registered fonts report an error we can ignore.
                unsafe {
                    CTFontManagerRegisterFontsForURL(
                        url.as_concrete_TypeRef(),
                        SCOPE_SESSION,
                        &mut err,
                    );
                    if !err.is_null() {
                        CFRelease(err);
                    }
                }
            }
        }
        Ok(())
    }

    fn deactivate(&self, faces: &[&Face]) -> Result<()> {
        for p in unique_paths(faces) {
            if let Some(url) = CFURL::from_path(p, false) {
                let mut err = std::ptr::null();
                unsafe {
                    CTFontManagerUnregisterFontsForURL(
                        url.as_concrete_TypeRef(),
                        SCOPE_SESSION,
                        &mut err,
                    );
                    if !err.is_null() {
                        CFRelease(err);
                    }
                }
            }
        }
        Ok(())
    }

    fn sync(&self, faces: &[&Face]) -> Result<()> {
        self.activate(faces)
    }
}

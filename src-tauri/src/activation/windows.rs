use super::{Activator, unique_paths};
use crate::error::Result;
use crate::model::Face;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use windows_sys::Win32::Graphics::Gdi::{AddFontResourceW, RemoveFontResourceW};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    HWND_BROADCAST, SendNotifyMessageW, WM_FONTCHANGE,
};

#[derive(Default)]
pub struct Gdi;

fn wide(p: &Path) -> Vec<u16> {
    p.as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

fn broadcast() {
    // Non-blocking: a hung window must not freeze Fontin.
    unsafe { SendNotifyMessageW(HWND_BROADCAST, WM_FONTCHANGE, 0, 0) };
}

impl Activator for Gdi {
    fn activate(&self, faces: &[&Face]) -> Result<()> {
        for p in unique_paths(faces) {
            if unsafe { AddFontResourceW(wide(p).as_ptr()) } == 0 {
                return Err(crate::i18n::Msg::WindowsRejected(p.display().to_string()).into());
            }
        }
        broadcast();
        Ok(())
    }

    fn deactivate(&self, faces: &[&Face]) -> Result<()> {
        for p in unique_paths(faces) {
            unsafe { RemoveFontResourceW(wide(p).as_ptr()) };
        }
        broadcast();
        Ok(())
    }

    fn sync(&self, faces: &[&Face]) -> Result<()> {
        // GDI registrations die with the session, so startup just re-adds.
        self.activate(faces)
    }
}

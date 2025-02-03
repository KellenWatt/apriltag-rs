use crate::native::*;

#[allow(dead_code)]
pub struct ImageU8 {
    raw: *mut image_u8_t,
}

pub type Image = ImageU8;

impl Drop for ImageU8 {
    fn drop(&mut self) {
    unsafe {
        image_u8_destroy(self.raw);
    }
    }
}

#[allow(dead_code)]
impl ImageU8 {
    fn new<T: AsRef<[u8]>>(width: u32, height: u32, data: T) -> Option<ImageU8> {
        unsafe {
            if data.as_ref().len() != (width * height) as usize {
                return None;
            }
            let raw = image_u8_create(width, height);
            if raw.is_null() {
                return None;
            }
            let data_ptr = data.as_ref().as_ptr();
            std::ptr::copy_nonoverlapping(data_ptr, (*raw).buf, (height * width) as usize);

            Some(ImageU8 {
                raw
            })
        }
    }

    pub unsafe fn to_raw(&self) -> *mut image_u8_t {
        self.raw
    }
}

impl Clone for ImageU8 {
    fn clone(&self) -> ImageU8 {
        unsafe {
        ImageU8 {
            raw: image_u8_copy(self.raw),
        }
        }
    }
}


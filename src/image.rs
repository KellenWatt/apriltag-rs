use crate::native::*;

#[allow(dead_code)]
pub struct ImageU8<T: AsRef<[u8]>> {
    width: u32,
    height: u32,
    data: T,
}

pub type Image<T> = ImageU8<T>;

// impl Drop for ImageU8 {
//     fn drop(&mut self) {
//     unsafe {
//         image_u8_destroy(self.raw);
//     }
//     }
// }

#[allow(dead_code)]
impl<T: AsRef<[u8]>> ImageU8<T> {
    pub fn new(width: u32, height: u32, data: T) -> ImageU8<T> {
        ImageU8 {
            width,
            height,
            data,
        }
        // unsafe {
        //     let bytes = data.as_ref();
        //     if bytes.len() != (width * height) as usize {
        //         return None;
        //     }
        //     let raw = image_u8_create(width, height);
        //     if raw.is_null() {
        //         return None;
        //     }
        //     let data_ptr = bytes.as_ptr();
        //     std::ptr::copy_nonoverlapping(data_ptr, (*raw).buf, (height * width) as usize);
        // 
        //     Some(ImageU8 {
        //         raw
        //     })
        // }
    }

    pub unsafe fn as_image_u8(&self) -> image_u8_t {
        image_u8_t {
            width: self.width as i32,
            height: self.height as i32,
            stride: self.width as i32,
            buf: self.data.as_ref().as_ptr() as *mut u8,
        }
    }

    // pub unsafe fn to_raw(&self) -> *mut image_u8_t {
    //     self.raw
    // }
}

// impl Clone for ImageU8 {
//     fn clone(&self) -> ImageU8 {
//         unsafe {
//         ImageU8 {
//             raw: image_u8_copy(self.raw),
//         }
//         }
//     }
// }


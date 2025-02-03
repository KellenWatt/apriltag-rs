use crate::native::*;

use std::sync::OnceLock;

#[allow(dead_code)]
pub struct Family {
    tag_family: TagFamily,
    raw: *mut apriltag_family_t,
}

impl Family {
    #[allow(dead_code)]
    pub unsafe fn into_raw(&self) -> *mut apriltag_family_t {
        self.raw
    }
}

macro_rules! destroy {
    ($fam:tt, $ptr:expr) => {
        unsafe{
            paste!{[<tag $fam:camel _destroy>]($ptr)}
        }
    }
}

// This will never be used in normal code (statics are not dropped), but it may be useful at some point.
// At minimum, it prevents a memory leak in safe code.
impl Drop for Family {
    fn drop(&mut self) {
        match self.tag_family {
            TagFamily::Tag16h5 => destroy!(16h5, self.raw),
            TagFamily::Tag25h9 => destroy!(25h9, self.raw),
            TagFamily::Tag36h10 => destroy!(36h10, self.raw),
            TagFamily::Tag36h11 => destroy!(36h11, self.raw),
            TagFamily::TagCircle21h7 => destroy!(circle21h7, self.raw),
            TagFamily::TagCircle49h12 => destroy!(circle49h12, self.raw),
            TagFamily::TagCustom48h12 => destroy!(custom48h12, self.raw),
            TagFamily::TagStandard41h12 => destroy!(standard41h12, self.raw),
            TagFamily::TagStandard52h13 => destroy!(standard52h13, self.raw),
        }
    }
}

unsafe impl Send for Family {}
unsafe impl Sync for Family {}

// Each of these allow for any family to exist without actually instantiating it if it isn't used.
// The only access to this will be TagFamily::family, so these effectively behave as non-global
// singletons.
#[allow(dead_code)]
static TAG_16H5: OnceLock<Family> = OnceLock::new();
#[allow(dead_code)]
static TAG_25H9: OnceLock<Family> = OnceLock::new();
#[allow(dead_code)]
static TAG_36H10: OnceLock<Family> = OnceLock::new();
#[allow(dead_code)]
static TAG_36H11: OnceLock<Family> = OnceLock::new();
#[allow(dead_code)]
static TAG_CIRCLE21H7: OnceLock<Family> = OnceLock::new();
#[allow(dead_code)]
static TAG_CIRCLE49H12: OnceLock<Family> = OnceLock::new();
#[allow(dead_code)]
static TAG_CUSTOM48H12: OnceLock<Family> = OnceLock::new();
#[allow(dead_code)]
static TAG_STANDARD41H12: OnceLock<Family> = OnceLock::new();
#[allow(dead_code)]
static TAG_STANDARD52H13: OnceLock<Family> = OnceLock::new();

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum TagFamily {
    Tag16h5,
    Tag25h9,
    Tag36h10,
    Tag36h11,
    TagCircle21h7,
    TagCircle49h12,
    TagCustom48h12,
    TagStandard41h12,
    TagStandard52h13,
}

use paste::paste;
macro_rules! get_family {
    ($fam:tt) => {
        paste! {
            [<TAG_ $fam:upper>].get_or_init(|| {
                Family {
                    tag_family: TagFamily::[<Tag $fam:camel>],
                    raw: unsafe {[<tag $fam:camel _create>]()},
                }
            })
        }
    };
}

impl TagFamily {
    #[allow(dead_code)]
    pub fn family(&self) -> &Family {
        match self {
            TagFamily::Tag16h5 => get_family!(16h5),
            TagFamily::Tag25h9 => get_family!(25h9),
            TagFamily::Tag36h10 => get_family!(36h10),
            TagFamily::Tag36h11 => get_family!(36h11),
            TagFamily::TagCircle21h7 => get_family!(circle21h7),
            TagFamily::TagCircle49h12 => get_family!(circle49h12),
            TagFamily::TagCustom48h12 => get_family!(custom48h12),
            TagFamily::TagStandard41h12 => get_family!(standard41h12),
            TagFamily::TagStandard52h13 => get_family!(standard52h13),
        }
    }
}

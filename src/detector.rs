use crate::native::*;
use crate::family::TagFamily;
// use crate::array::Array;
use crate::image::ImageU8;
use std::mem::MaybeUninit;
#[cfg(feature = "3d")]
use nalgebra::Matrix3;
#[cfg(feature = "3d")]
use nalgebra::linalg::{QR};

// These are here as a result of libapriltag using `static inline` on all of its useful zarray
// functions. That's great for efficiency (kind of), but not so much for porting.
extern "C" {
    fn zarray_size__extern(za: *mut zarray_t) -> i32;
    fn zarray_get__extern(za: *mut zarray_t, idx: i32, p: *mut ::std::os::raw::c_void);
    fn zarray_destroy__extern(za: *mut zarray_t);
}


#[cfg(feature = "3d")]
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct CameraIntrinsics {
    pub fx: f64,
    pub fy: f64,
    pub cx: f64,
    pub cy: f64,
}

#[cfg(feature = "3d")]
#[allow(dead_code)]
#[derive(Clone, Copy)]
pub struct Rotation {
    quat: [f64; 4],
}

#[cfg(feature = "3d")]
#[allow(dead_code)]
impl Rotation {
    unsafe fn from_matd(mat: *mut matd_t) -> Rotation {
        let m = Matrix3::from_fn(|i, j| {
            matd_get(mat, i as u32, j as u32)
        });
        matd_destroy(mat);

        //orthogonalize (source: WPILib source https://github.com/wpilibsuite/allwpilib/blob/main/apriltag/src/main/native/cpp/AprilTagPoseEstimator.cpp#L24)
        let qr = QR::new(m);
        let mut q = qr.q();
        let r = qr.r();

        for i in 0..3 {
            if r[(i, i)] < 0.0 {
                for j in 0..3 {
                    q[(j, i)] = -q[(j, i)];
                }
            }
        }

        let m = q;

        // translate to quaternion (source: WPILib's wpimath/algorithms.md)
        if (m * m.transpose() - Matrix3::identity()).norm() > 1e-9 {
            panic!("rotation matrix isn't orthogonal");
        }

        if (m.determinant() - 1.0).abs() > 1e-9 {
            panic!("rotation matrix is orthogonal, but not special orthogonal")
        }

        let trace = m.trace();
        let m00 = m[(0,0)];
        let m11 = m[(1,1)];
        let m22 = m[(2,2)];
        if trace > 0.0 {
            let s = 0.5 / (trace + 1.0).sqrt();
            let w = 0.25 / s;
            let x = (m[(2,1)] - m[(1,2)]) * s;
            let y = (m[(0,2)] - m[(2,0)]) * s;
            let z = (m[(1,0)] - m[(0,1)]) * s;
            Rotation{quat: [w,x,y,z]}
        } else {
            if m00 > m11 && m00 > m22 {
                let s = 2.0 * (1.0 + m00 - m11 - m22).sqrt();
                let w = (m[(2,1)] - m[(1,2)]) / s;
                let x = 0.25 * s;
                let y = (m[(0,1)] + m[(1,0)]) / s;
                let z = (m[(0,2)] + m[(2,0)]) / s;
                Rotation{quat: [w,x,y,z]}
            } else if m11 > m22 {
                let s = 2.0 * (1.0 + m11 - m00 - m22);
                let w = (m[(0,2)] - m[(2,0)]) / s;
                let x = (m[(0,1)] + m[(1,0)]) / s;
                let y = 0.25 * s;
                let z = (m[(1,2)] + m[(2,1)]) / s;
                Rotation{quat: [w,x,y,z]}
            } else {
                let s = 2.0 * (1.0 + m22 - m00 - m11).sqrt();
                let w = (m[(1, 0)] - m[(0, 1)]) / s;
                let x = (m[(0, 2)] + m[(2, 0)]) / s;
                let y = (m[(1, 2)] + m[(2, 1)]) / s;
                let z = 0.25 * s;
                Rotation{quat: [w,x,y,z]}
            }
        }
    }

    pub fn roll(&self) -> f64 {
        let w = self.quat[0];
        let x = self.quat[1];
        let y = self.quat[2];
        let z = self.quat[3];

        let cxcy = 1.0 - 2.0 * (x*x + y*y);
        let sxcy = 2.0 * (w*x + y*z);
        let cy_sq = cxcy*cxcy + sxcy*sxcy;
        if cy_sq > 1e-20 {
            sxcy.atan2(cxcy)
        } else {
            0.0
        }
    }

    pub fn roll_deg(&self) -> f64 {
        self.roll() / std::f64::consts::PI * 180.0
    }
    
    pub fn pitch(&self) -> f64 {
        let w = self.quat[0];
        let x = self.quat[1];
        let y = self.quat[2];
        let z = self.quat[3];

        let ratio = 2.0 * (w*y - z*x);
        if ratio.abs() >= 1.0 {
            (std::f64::consts::PI / 2.0).copysign(ratio)
        } else {
            ratio.asin()
        }
    }
    pub fn pitch_deg(&self) -> f64 {
        self.pitch() / std::f64::consts::PI * 180.0
    }

    pub fn yaw(&self) -> f64 {
        let w = self.quat[0];
        let x = self.quat[1];
        let y = self.quat[2];
        let z = self.quat[3];

        let cycz = 1.0 - 2.0 * (y*y + z*z);
        let cysz = 2.0 * (w*z + x*y);
        let cy_sq = cycz*cycz + cysz*cysz;
        if cy_sq > 1e-20 {
            cysz.atan2(cycz)
        } else {
            (2.0*w*z).atan2(w*w - z*z)
        }
    }
    pub fn yaw_deg(&self) -> f64 {
        self.yaw() / std::f64::consts::PI * 180.0
    }
}

#[cfg(feature = "3d")]
#[allow(dead_code)]
#[derive(Clone, Copy)]
pub struct Translation {
    pub x: f64,
    pub y: f64, 
    pub z: f64,
}

#[cfg(feature = "3d")]
#[allow(dead_code)]
impl Translation {
    unsafe fn from_matd(mat: *mut matd_t) -> Translation {
        let x = matd_get(mat, 0, 0);
        let y = matd_get(mat, 1, 0);
        let z = matd_get(mat, 2, 0);
        matd_destroy(mat);
        Translation{x, y, z}
    }
}

#[cfg(feature = "3d")]
#[allow(dead_code)]
pub struct Pose {
    pub rot: Rotation,
    pub pos: Translation,
}

pub type Point = [f64; 2];

#[allow(dead_code)]
pub struct Detection {
    raw: *mut apriltag_detection_t,
}

impl Drop for Detection {
    fn drop(&mut self) {
        unsafe {
            apriltag_detection_destroy(self.raw)
        }
    }
}

#[allow(dead_code)]
impl Detection {
    pub unsafe fn from_raw(ptr: *mut apriltag_detection_t) -> Detection {
        Detection {
            raw: ptr
        }
    }

    pub fn id(&self) -> u32 {
        unsafe {(*self.raw).id as u32}
    }

    pub fn hamming(&self) -> u32 {
        unsafe {(*self.raw).hamming as u32}
    }

    pub fn decision_margin(&self) -> f32 {
        unsafe {(*self.raw).decision_margin}
    }

    pub fn center(&self) -> Point {
        unsafe {(*self.raw).c}
    }

    pub fn corners(&self) -> [Point; 4] {
        unsafe {(*self.raw).p}
    }

    #[cfg(feature = "3d")]
    pub fn estimate_pose(&self, intrinsics: &CameraIntrinsics, tag_size: f64) -> Pose {
        unsafe {
            let mut info = apriltag_detection_info_t {
                det: self.raw,
                tagsize: tag_size,
                fx: intrinsics.fx,
                fy: intrinsics.fy,
                cx: intrinsics.cx,
                cy: intrinsics.cy,
            };
            let info_ptr = &mut info as *mut apriltag_detection_info_t;

            let mut pose = MaybeUninit::<apriltag_pose_t>::uninit();

            estimate_tag_pose(info_ptr, pose.as_mut_ptr());
            let pose = pose.assume_init();

            Pose {
                rot: Rotation::from_matd(pose.R),
                pos: Translation::from_matd(pose.t),
            }
        }
    }
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct DetectorConfig {
    pub threads: u32,
    pub quad_decimate: f32,
    pub quad_sigma: f32,
    pub refine_edges: bool,
    pub decode_sharpening: f64,
    pub debug: bool,
}

impl Default for DetectorConfig {
    fn default() -> DetectorConfig {
        DetectorConfig {
            threads: 1,
            quad_decimate: 2.0,
            quad_sigma: 0.0,
            refine_edges: false,
            decode_sharpening: 0.25,
            debug: false,
        }
    }
}

#[allow(dead_code)]
pub struct Detector {
    raw: *mut apriltag_detector_t,
}

unsafe impl Send for Detector {}

impl Drop for Detector {
    fn drop(&mut self) {
        unsafe {
            apriltag_detector_destroy(self.raw);
        }
    }
}

#[allow(dead_code)]
impl Detector {
    pub fn new() -> Detector {
        unsafe {
            let ptr = apriltag_detector_create();
            Detector {
                raw: ptr,
            }
        }
    }

    pub fn new_with_threads(n: i32) -> Detector {
        unsafe {
            let ptr = apriltag_detector_create();
            (*ptr).nthreads = n;
            Detector {
                raw: ptr,
            }
        }

    }

    pub fn from_config(cfg: DetectorConfig) -> Detector {
        unsafe {
            let ptr = apriltag_detector_create();
            (*ptr).nthreads = cfg.threads as i32;
            (*ptr).quad_decimate = cfg.quad_decimate;
            (*ptr).quad_sigma = cfg.quad_sigma;
            (*ptr).refine_edges = cfg.refine_edges;
            (*ptr).decode_sharpening = cfg.decode_sharpening;
            (*ptr).debug = cfg.debug;
            Detector {
                raw: ptr,
            }
        }
    }

    pub fn add_with_bits(&mut self, fam: TagFamily, bits: u8) {
        unsafe {
            let fam = fam.family().into_raw();
            // println!("family: {:p}", fam);
            apriltag_detector_add_family_bits(self.raw, fam, bits as i32);
        }
    }

    pub fn add(&mut self, fam: TagFamily) {
        self.add_with_bits(fam, 2);
    }

    pub fn clear(&mut self) {
        unsafe {
            apriltag_detector_clear_families(self.raw);
        }
    }

    pub fn detect<T: AsRef<[u8]>>(&mut self, image: ImageU8<T>) -> Vec<Detection> {
        unsafe {
            let mut img_u8 = image.as_image_u8();
            let img_ptr = (&mut img_u8) as *mut image_u8_t;
            let arr = apriltag_detector_detect(self.raw, img_ptr);

            let size = zarray_size__extern(arr);
            let mut out = Vec::with_capacity(size as usize);

            for i in 0..size {
                let mut det = MaybeUninit::<*mut apriltag_detection_t>::uninit();
                zarray_get__extern(arr, i, det.as_mut_ptr() as *mut ::std::os::raw::c_void);
                // let detection_p: *mut apriltag_detection_t = 
                //     (*arr).data.add(n * std::mem::size_of::<*mut apriltag_detection_t>()) as *mut apriltag_detection_t;
                let detection = Detection::from_raw(det.assume_init());
                out.push(detection);
            }
            zarray_destroy__extern(arr);
            out
        }
    }
}


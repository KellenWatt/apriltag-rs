use crate::native::*;
use crate::family::TagFamily;
use crate::array::Array;
use crate::image::ImageU8;
use std::mem::MaybeUninit;
use nalgebra::Matrix3;
use nalgebra::linalg::{QR};

// #[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct CameraIntrinsics {
    pub fx: f64,
    pub fy: f64,
    pub cx: f64,
    pub cy: f64,
}

#[allow(dead_code)]
pub struct Rotation {
    quat: [f64; 4],
}

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
            let x = m[(2,1)] - m[(1,2)] * s;
            let y = m[(0,2)] - m[(2,0)] * s;
            let z = m[(1,0)] - m[(0,1)] * s;
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

    pub fn x(&self) -> f64 {
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
    
    pub fn y(&self) -> f64 {
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

    pub fn z(&self) -> f64 {
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
}

#[allow(dead_code)]
pub struct Translation {
    pub x: f64,
    pub y: f64, 
    pub z: f64,
}

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

#[allow(dead_code)]
pub struct Pose {
    rot: Rotation,
    pos: Translation,
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


#[allow(dead_code)]
pub struct Detector {
    raw: *mut apriltag_detector_t,
}

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
        Detector {
            raw: unsafe{apriltag_detector_create()},
        }
    }

    pub fn add_with_bits(&mut self, fam: TagFamily, bits: u8) {
        unsafe {
            apriltag_detector_add_family_bits(self.raw, fam.family().into_raw(), bits as i32);
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

    pub fn detect(&mut self, image: &ImageU8) -> Vec<Detection> {
        unsafe {
            let arr = apriltag_detector_detect(self.raw, image.to_raw());
            let list = Array::from_raw(arr);
            let len = list.len();
            let mut out = Vec::with_capacity(len);

            for i in 0..len {
                out.push(list.get_value_unchecked(i));
            }
            out
        }
    }
}

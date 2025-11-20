use crate::mat4::Mat4;
use crate::vec3::Vec3;
use std::ops::{Add, Mul, Neg};

/// Quaternion para representar rotações em 3D
/// Formato: w + xi + yj + zk
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quat {
    pub const IDENTITY: Quat = Quat {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 1.0,
    };

    #[inline]
    pub const fn from_xyzw(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    #[inline]
    pub fn from_axis_angle(axis: Vec3, angle: f32) -> Self {
        let half_angle = angle * 0.5;
        let (sin, cos) = half_angle.sin_cos();
        let axis = axis.normalize();

        Self {
            x: axis.x * sin,
            y: axis.y * sin,
            z: axis.z * sin,
            w: cos,
        }
    }

    #[inline]
    pub fn from_rotation_x(angle: f32) -> Self {
        let half_angle = angle * 0.5;
        let (sin, cos) = half_angle.sin_cos();
        Self {
            x: sin,
            y: 0.0,
            z: 0.0,
            w: cos,
        }
    }

    #[inline]
    pub fn from_rotation_y(angle: f32) -> Self {
        let half_angle = angle * 0.5;
        let (sin, cos) = half_angle.sin_cos();
        Self {
            x: 0.0,
            y: sin,
            z: 0.0,
            w: cos,
        }
    }

    #[inline]
    pub fn from_rotation_z(angle: f32) -> Self {
        let half_angle = angle * 0.5;
        let (sin, cos) = half_angle.sin_cos();
        Self {
            x: 0.0,
            y: 0.0,
            z: sin,
            w: cos,
        }
    }

    #[inline]
    pub fn from_euler(roll: f32, pitch: f32, yaw: f32) -> Self {
        let (sr, cr) = (roll * 0.5).sin_cos();
        let (sp, cp) = (pitch * 0.5).sin_cos();
        let (sy, cy) = (yaw * 0.5).sin_cos();

        Self {
            x: sr * cp * cy - cr * sp * sy,
            y: cr * sp * cy + sr * cp * sy,
            z: cr * cp * sy - sr * sp * cy,
            w: cr * cp * cy + sr * sp * sy,
        }
    }

    #[inline]
    pub fn to_euler(self) -> (f32, f32, f32) {
        let sinr_cosp = 2.0 * (self.w * self.x + self.y * self.z);
        let cosr_cosp = 1.0 - 2.0 * (self.x * self.x + self.y * self.y);
        let roll = sinr_cosp.atan2(cosr_cosp);

        let sinp = 2.0 * (self.w * self.y - self.z * self.x);
        let pitch = if sinp.abs() >= 1.0 {
            std::f32::consts::FRAC_PI_2.copysign(sinp)
        } else {
            sinp.asin()
        };

        let siny_cosp = 2.0 * (self.w * self.z + self.x * self.y);
        let cosy_cosp = 1.0 - 2.0 * (self.y * self.y + self.z * self.z);
        let yaw = siny_cosp.atan2(cosy_cosp);

        (roll, pitch, yaw)
    }

    #[inline]
    pub fn length_squared(self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w
    }

    #[inline]
    pub fn length(self) -> f32 {
        self.length_squared().sqrt()
    }

    #[inline]
    pub fn normalize(self) -> Self {
        let len = self.length();
        if len != 0.0 {
            Self {
                x: self.x / len,
                y: self.y / len,
                z: self.z / len,
                w: self.w / len,
            }
        } else {
            Self::IDENTITY
        }
    }

    #[inline]
    pub fn conjugate(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: self.w,
        }
    }

    #[inline]
    pub fn inverse(self) -> Self {
        let len_sq = self.length_squared();
        if len_sq != 0.0 {
            let inv_len_sq = 1.0 / len_sq;
            Self {
                x: -self.x * inv_len_sq,
                y: -self.y * inv_len_sq,
                z: -self.z * inv_len_sq,
                w: self.w * inv_len_sq,
            }
        } else {
            Self::IDENTITY
        }
    }

    #[inline]
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
    }

    #[inline]
    pub fn lerp(self, other: Self, t: f32) -> Self {
        let start = self;
        let end = other;
        Self {
            x: start.x + (end.x - start.x) * t,
            y: start.y + (end.y - start.y) * t,
            z: start.z + (end.z - start.z) * t,
            w: start.w + (end.w - start.w) * t,
        }
        .normalize()
    }

    #[inline]
    pub fn slerp(self, other: Self, t: f32) -> Self {
        let mut dot = self.dot(other);
        let mut end = other;

        // Se o dot product é negativo, inverte um quaternion
        if dot < 0.0 {
            end = -end;
            dot = -dot;
        }

        // Se estão muito próximos, usa interpolação linear
        if dot > 0.9995 {
            return self.lerp(end, t);
        }

        let theta = dot.acos();
        let sin_theta = theta.sin();
        let a = ((1.0 - t) * theta).sin() / sin_theta;
        let b = (t * theta).sin() / sin_theta;

        Self {
            x: self.x * a + end.x * b,
            y: self.y * a + end.y * b,
            z: self.z * a + end.z * b,
            w: self.w * a + end.w * b,
        }
    }

    #[inline]
    pub fn rotate_vec3(self, v: Vec3) -> Vec3 {
        let qv = Vec3::new(self.x, self.y, self.z);
        let uv = qv.cross(v);
        let uuv = qv.cross(uv);
        v + (uv * self.w + uuv) * 2.0
    }

    #[inline]
    pub fn to_mat4(self) -> Mat4 {
        let q = self.normalize();
        let xx = q.x * q.x;
        let yy = q.y * q.y;
        let zz = q.z * q.z;
        let xy = q.x * q.y;
        let xz = q.x * q.z;
        let yz = q.y * q.z;
        let wx = q.w * q.x;
        let wy = q.w * q.y;
        let wz = q.w * q.z;

        Mat4::from_cols_array(&[
            1.0 - 2.0 * (yy + zz),
            2.0 * (xy + wz),
            2.0 * (xz - wy),
            0.0,
            2.0 * (xy - wz),
            1.0 - 2.0 * (xx + zz),
            2.0 * (yz + wx),
            0.0,
            2.0 * (xz + wy),
            2.0 * (yz - wx),
            1.0 - 2.0 * (xx + yy),
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
        ])
    }
}

impl Mul for Quat {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self {
        Self {
            w: self.w * rhs.w - self.x * rhs.x - self.y * rhs.y - self.z * rhs.z,
            x: self.w * rhs.x + self.x * rhs.w + self.y * rhs.z - self.z * rhs.y,
            y: self.w * rhs.y - self.x * rhs.z + self.y * rhs.w + self.z * rhs.x,
            z: self.w * rhs.z + self.x * rhs.y - self.y * rhs.x + self.z * rhs.w,
        }
    }
}

impl Mul<Vec3> for Quat {
    type Output = Vec3;

    #[inline]
    fn mul(self, rhs: Vec3) -> Vec3 {
        self.rotate_vec3(rhs)
    }
}

impl Add for Quat {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
            w: self.w + rhs.w,
        }
    }
}

impl Neg for Quat {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: -self.w,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity() {
        let q = Quat::IDENTITY;
        let v = Vec3::new(1.0, 2.0, 3.0);
        let rotated = q.rotate_vec3(v);
        assert!((rotated.x - v.x).abs() < 0.0001);
        assert!((rotated.y - v.y).abs() < 0.0001);
        assert!((rotated.z - v.z).abs() < 0.0001);
    }

    #[test]
    fn test_rotation_z_90() {
        let q = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2);
        let v = Vec3::new(1.0, 0.0, 0.0);
        let rotated = q.rotate_vec3(v);
        assert!((rotated.x - 0.0).abs() < 0.0001);
        assert!((rotated.y - 1.0).abs() < 0.0001);
        assert!((rotated.z - 0.0).abs() < 0.0001);
    }

    #[test]
    fn test_normalize() {
        let q = Quat::from_xyzw(1.0, 2.0, 3.0, 4.0);
        let normalized = q.normalize();
        let len = normalized.length();
        assert!((len - 1.0).abs() < 0.0001);
    }
}

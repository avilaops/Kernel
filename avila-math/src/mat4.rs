use crate::vec3::Vec3;
use crate::vec4::Vec4;
use std::ops::Mul;

/// Matriz 4x4 em column-major order (compatível com OpenGL/Vulkan)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat4 {
    pub cols: [Vec4; 4],
}

impl Mat4 {
    pub const ZERO: Mat4 = Mat4 {
        cols: [Vec4::ZERO, Vec4::ZERO, Vec4::ZERO, Vec4::ZERO],
    };

    pub const IDENTITY: Mat4 = Mat4 {
        cols: [Vec4::X, Vec4::Y, Vec4::Z, Vec4::W],
    };

    #[inline]
    pub const fn from_cols(col0: Vec4, col1: Vec4, col2: Vec4, col3: Vec4) -> Self {
        Self {
            cols: [col0, col1, col2, col3],
        }
    }

    #[inline]
    pub fn from_cols_array(m: &[f32; 16]) -> Self {
        Self {
            cols: [
                Vec4::new(m[0], m[1], m[2], m[3]),
                Vec4::new(m[4], m[5], m[6], m[7]),
                Vec4::new(m[8], m[9], m[10], m[11]),
                Vec4::new(m[12], m[13], m[14], m[15]),
            ],
        }
    }

    #[inline]
    pub fn to_cols_array(&self) -> [f32; 16] {
        [
            self.cols[0].x, self.cols[0].y, self.cols[0].z, self.cols[0].w,
            self.cols[1].x, self.cols[1].y, self.cols[1].z, self.cols[1].w,
            self.cols[2].x, self.cols[2].y, self.cols[2].z, self.cols[2].w,
            self.cols[3].x, self.cols[3].y, self.cols[3].z, self.cols[3].w,
        ]
    }

    #[inline]
    pub fn from_translation(translation: Vec3) -> Self {
        Self::from_cols(
            Vec4::X,
            Vec4::Y,
            Vec4::Z,
            Vec4::new(translation.x, translation.y, translation.z, 1.0),
        )
    }

    #[inline]
    pub fn from_scale(scale: Vec3) -> Self {
        Self::from_cols(
            Vec4::new(scale.x, 0.0, 0.0, 0.0),
            Vec4::new(0.0, scale.y, 0.0, 0.0),
            Vec4::new(0.0, 0.0, scale.z, 0.0),
            Vec4::W,
        )
    }

    #[inline]
    pub fn from_rotation_x(angle: f32) -> Self {
        let (sin, cos) = angle.sin_cos();
        Self::from_cols(
            Vec4::X,
            Vec4::new(0.0, cos, sin, 0.0),
            Vec4::new(0.0, -sin, cos, 0.0),
            Vec4::W,
        )
    }

    #[inline]
    pub fn from_rotation_y(angle: f32) -> Self {
        let (sin, cos) = angle.sin_cos();
        Self::from_cols(
            Vec4::new(cos, 0.0, -sin, 0.0),
            Vec4::Y,
            Vec4::new(sin, 0.0, cos, 0.0),
            Vec4::W,
        )
    }

    #[inline]
    pub fn from_rotation_z(angle: f32) -> Self {
        let (sin, cos) = angle.sin_cos();
        Self::from_cols(
            Vec4::new(cos, sin, 0.0, 0.0),
            Vec4::new(-sin, cos, 0.0, 0.0),
            Vec4::Z,
            Vec4::W,
        )
    }

    #[inline]
    pub fn from_axis_angle(axis: Vec3, angle: f32) -> Self {
        let (sin, cos) = angle.sin_cos();
        let one_minus_cos = 1.0 - cos;
        let axis = axis.normalize();

        let x = axis.x;
        let y = axis.y;
        let z = axis.z;

        Self::from_cols(
            Vec4::new(
                cos + x * x * one_minus_cos,
                x * y * one_minus_cos + z * sin,
                x * z * one_minus_cos - y * sin,
                0.0,
            ),
            Vec4::new(
                x * y * one_minus_cos - z * sin,
                cos + y * y * one_minus_cos,
                y * z * one_minus_cos + x * sin,
                0.0,
            ),
            Vec4::new(
                x * z * one_minus_cos + y * sin,
                y * z * one_minus_cos - x * sin,
                cos + z * z * one_minus_cos,
                0.0,
            ),
            Vec4::W,
        )
    }

    #[inline]
    pub fn look_at_rh(eye: Vec3, target: Vec3, up: Vec3) -> Self {
        let f = (target - eye).normalize();
        let s = f.cross(up).normalize();
        let u = s.cross(f);

        Self::from_cols(
            Vec4::new(s.x, u.x, -f.x, 0.0),
            Vec4::new(s.y, u.y, -f.y, 0.0),
            Vec4::new(s.z, u.z, -f.z, 0.0),
            Vec4::new(-s.dot(eye), -u.dot(eye), f.dot(eye), 1.0),
        )
    }

    #[inline]
    pub fn perspective_rh(fov_y_radians: f32, aspect_ratio: f32, z_near: f32, z_far: f32) -> Self {
        let tan_half_fov = (fov_y_radians / 2.0).tan();
        
        Self::from_cols(
            Vec4::new(1.0 / (aspect_ratio * tan_half_fov), 0.0, 0.0, 0.0),
            Vec4::new(0.0, 1.0 / tan_half_fov, 0.0, 0.0),
            Vec4::new(0.0, 0.0, -(z_far + z_near) / (z_far - z_near), -1.0),
            Vec4::new(0.0, 0.0, -(2.0 * z_far * z_near) / (z_far - z_near), 0.0),
        )
    }

    #[inline]
    pub fn orthographic_rh(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self {
        let rcp_width = 1.0 / (right - left);
        let rcp_height = 1.0 / (top - bottom);
        let rcp_depth = 1.0 / (far - near);

        Self::from_cols(
            Vec4::new(2.0 * rcp_width, 0.0, 0.0, 0.0),
            Vec4::new(0.0, 2.0 * rcp_height, 0.0, 0.0),
            Vec4::new(0.0, 0.0, -2.0 * rcp_depth, 0.0),
            Vec4::new(
                -(right + left) * rcp_width,
                -(top + bottom) * rcp_height,
                -(far + near) * rcp_depth,
                1.0,
            ),
        )
    }

    #[inline]
    pub fn transpose(&self) -> Self {
        Self::from_cols(
            Vec4::new(self.cols[0].x, self.cols[1].x, self.cols[2].x, self.cols[3].x),
            Vec4::new(self.cols[0].y, self.cols[1].y, self.cols[2].y, self.cols[3].y),
            Vec4::new(self.cols[0].z, self.cols[1].z, self.cols[2].z, self.cols[3].z),
            Vec4::new(self.cols[0].w, self.cols[1].w, self.cols[2].w, self.cols[3].w),
        )
    }

    #[inline]
    pub fn determinant(&self) -> f32 {
        let a = self.cols[0];
        let b = self.cols[1];
        let c = self.cols[2];
        let d = self.cols[3];

        let det_a = a.x * (b.y * c.z * d.w + b.z * c.w * d.y + b.w * c.y * d.z 
                          - b.w * c.z * d.y - b.z * c.y * d.w - b.y * c.w * d.z);
        let det_b = a.y * (b.x * c.z * d.w + b.z * c.w * d.x + b.w * c.x * d.z
                          - b.w * c.z * d.x - b.z * c.x * d.w - b.x * c.w * d.z);
        let det_c = a.z * (b.x * c.y * d.w + b.y * c.w * d.x + b.w * c.x * d.y
                          - b.w * c.y * d.x - b.y * c.x * d.w - b.x * c.w * d.y);
        let det_d = a.w * (b.x * c.y * d.z + b.y * c.z * d.x + b.z * c.x * d.y
                          - b.z * c.y * d.x - b.y * c.x * d.z - b.x * c.z * d.y);

        det_a - det_b + det_c - det_d
    }

    #[inline]
    pub fn transform_point3(&self, point: Vec3) -> Vec3 {
        let v = Vec4::new(point.x, point.y, point.z, 1.0);
        let result = *self * v;
        Vec3::new(result.x / result.w, result.y / result.w, result.z / result.w)
    }

    #[inline]
    pub fn transform_vector3(&self, vector: Vec3) -> Vec3 {
        let v = Vec4::new(vector.x, vector.y, vector.z, 0.0);
        let result = *self * v;
        Vec3::new(result.x, result.y, result.z)
    }
}

impl Mul for Mat4 {
    type Output = Self;
    
    #[inline]
    fn mul(self, rhs: Self) -> Self {
        let a = self;
        let b = rhs;
        
        Self::from_cols(
            a * b.cols[0],
            a * b.cols[1],
            a * b.cols[2],
            a * b.cols[3],
        )
    }
}

impl Mul<Vec4> for Mat4 {
    type Output = Vec4;
    
    #[inline]
    fn mul(self, rhs: Vec4) -> Vec4 {
        let x = self.cols[0] * rhs.x;
        let y = self.cols[1] * rhs.y;
        let z = self.cols[2] * rhs.z;
        let w = self.cols[3] * rhs.w;
        x + y + z + w
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity() {
        let id = Mat4::IDENTITY;
        let v = Vec4::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(id * v, v);
    }

    #[test]
    fn test_translation() {
        let trans = Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0));
        let point = Vec3::new(0.0, 0.0, 0.0);
        let result = trans.transform_point3(point);
        assert_eq!(result, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_scale() {
        let scale = Mat4::from_scale(Vec3::new(2.0, 3.0, 4.0));
        let point = Vec3::new(1.0, 1.0, 1.0);
        let result = scale.transform_point3(point);
        assert_eq!(result, Vec3::new(2.0, 3.0, 4.0));
    }
}

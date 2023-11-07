use bevy::{
    math::{vec3, Mat3A, Vec3A},
    prelude::Mat3,
};

pub trait MatrixExtras {
    fn decompose_scale_rotation(&self) -> (Mat3, Mat3);
}

impl MatrixExtras for Mat3A {
    fn decompose_scale_rotation(&self) -> (Mat3, Mat3) {
        let det = self.determinant();
        let scale = Vec3A::new(
            self.x_axis.length() * f32::signum(det),
            self.y_axis.length(),
            self.z_axis.length(),
        );

        let inv_scale = scale.recip();
        let rotation = Mat3::from_cols(
            (self.x_axis * inv_scale.x).into(),
            (self.y_axis * inv_scale.y).into(),
            (self.z_axis * inv_scale.z).into(),
        );

        let scale_mat = Mat3::from_cols(
            vec3(scale.x, 0., 0.),
            vec3(0., scale.y, 0.),
            vec3(0., 0., scale.z),
        );

        (scale_mat, rotation)
    }
}

impl MatrixExtras for Mat3 {
    #[inline]
    fn decompose_scale_rotation(&self) -> (Mat3, Mat3) {
        Mat3A::from(*self).decompose_scale_rotation()
    }
}

use cgmath::{
    Point3,
    Vector3,
    Quaternion,
    Matrix3,
    EuclideanSpace,
};

#[derive(Clone)]
pub struct Sphere {
    pub center: Point3<f32>,
    pub radius: f32,
}

impl Sphere {
    pub fn new(center: Point3<f32>, radius: f32) -> Self {
        Sphere { center, radius }
    }
}

#[derive(Clone)]
pub struct BoundingBox {
    pub min: Point3<f32>,
    pub max: Point3<f32>,
}

impl BoundingBox {
    pub fn new(min: Point3<f32>, max: Point3<f32>) -> Self {
        BoundingBox { min, max }
    }

    pub fn closest_point(&self, point: &Point3<f32>) -> Vector3<f32> {
        Vector3::new(
            point.x.max(self.min.x).min(self.max.x),
            point.y.max(self.min.y).min(self.max.y),
            point.z.max(self.min.z).min(self.max.z),
        )
    }

    pub fn center(&self) -> Point3<f32> {
        Point3::new(
            (self.min.x + self.max.x) / 2.0,
            (self.min.y + self.max.y) / 2.0,
            (self.min.z + self.max.z) / 2.0,
        )
    }

    pub fn size(&self) -> Vector3<f32> {
        self.max - self.min
    }

    pub fn rotated_points(&self, rotation: &Quaternion<f32>) -> Vec<Vector3<f32>> {
        let half_size = self.size() / 2.0;

        let corners = vec![
            Vector3::new(-half_size.x, -half_size.y, -half_size.z),
            Vector3::new(half_size.x, -half_size.y, -half_size.z),
            Vector3::new(-half_size.x, half_size.y, -half_size.z),
            Vector3::new(half_size.x, half_size.y, -half_size.z),
            Vector3::new(-half_size.x, -half_size.y, half_size.z),
            Vector3::new(half_size.x, -half_size.y, half_size.z),
            Vector3::new(-half_size.x, half_size.y, half_size.z),
            Vector3::new(half_size.x, half_size.y, half_size.z),
        ];

        let rotation_matrix = Matrix3::from(*rotation);

        corners
            .into_iter()
            .map(|corner| rotation_matrix * corner + self.center().to_vec())
            .collect()
    }
}

#[derive(Clone)]
pub enum CollisionBox {
    BoundingBox(BoundingBox),
    Sphere(Sphere),
}

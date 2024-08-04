use cgmath::{Point3, Vector3, InnerSpace};

#[derive(Clone)]
pub struct Sphere {
    pub center: Point3<f32>,
    pub radius: f32,
}

impl Sphere {
    pub fn new(center: Point3<f32>, radius: f32) -> Self {
        Sphere { center, radius }
    }

    pub fn intersects_sphere(&self, other: &Sphere) -> bool {
        let distance = (self.center - other.center).magnitude();
        distance < (self.radius + other.radius)
    }

    pub fn intersects_box(&self, bbox: &BoundingBox) -> bool {
        bbox.intersects_sphere(self)
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

    pub fn intersects_box(&self, other: &BoundingBox) -> bool {
        self.min.x <= other.max.x && self.max.x >= other.min.x &&
            self.min.y <= other.max.y && self.max.y >= other.min.y &&
            self.min.z <= other.max.z && self.max.z >= other.min.z
    }

    pub fn intersects_sphere(&self, sphere: &Sphere) -> bool {
        let closest_point = Point3 {
            x: sphere.center.x.max(self.min.x).min(self.max.x),
            y: sphere.center.y.max(self.min.y).min(self.max.y),
            z: sphere.center.z.max(self.min.z).min(self.max.z),
        };
        (closest_point - sphere.center).magnitude2() < sphere.radius * sphere.radius
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
}

#[derive(Clone)]
pub enum CollisionBox {
    BoundingBox(BoundingBox),
    Sphere(Sphere),
}

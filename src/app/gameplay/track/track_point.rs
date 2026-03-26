use bevy::math::{Quat, Vec3};
#[derive(Debug, Clone)]
pub struct TrackPoint {
    pub rotation: Quat,
    pub position: Vec3,
    pub forward: Vec3,
    pub right: Vec3,
    pub up: Vec3,
}

impl TrackPoint {
    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        Self {
            rotation: self.rotation.slerp(other.rotation, t),
            position: self.position.lerp(other.position, t),
            forward: self.forward.lerp(other.forward, t),
            right: self.right.lerp(other.right, t),
            up: self.up.lerp(other.up, t),
        }
    }

    pub fn catmull_rom(p0: &Self, p1: &Self, p2: &Self, p3: &Self, t: f32) -> Self {
        let t2 = t * t;
        let t3 = t2 * t;

        fn cr(v0: Vec3, v1: Vec3, v2: Vec3, v3: Vec3, t: f32, t2: f32, t3: f32) -> Vec3 {
            0.5 * (2.0 * v1
                + (-v0 + v2) * t
                + (2.0 * v0 - 5.0 * v1 + 4.0 * v2 - v3) * t2
                + (-v0 + 3.0 * v1 - 3.0 * v2 + v3) * t3)
        }

        Self {
            rotation: p1.rotation.slerp(p2.rotation, t),
            position: cr(
                p0.position,
                p1.position,
                p2.position,
                p3.position,
                t,
                t2,
                t3,
            ),
            forward: cr(p0.forward, p1.forward, p2.forward, p3.forward, t, t2, t3).normalize(),
            right: cr(p0.right, p1.right, p2.right, p3.right, t, t2, t3).normalize(),
            up: cr(p0.up, p1.up, p2.up, p3.up, t, t2, t3).normalize(),
        }
    }
}

pub fn smooth_positions(points: &mut [TrackPoint], strength: f32) {
    for i in 1..points.len() - 1 {
        let prev = points[i - 1].position;
        let next = points[i + 1].position;

        let avg = (prev + next) * 0.5;
        points[i].position = points[i].position.lerp(avg, strength);
    }
}

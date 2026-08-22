#[derive(Clone, Copy, Debug)]
pub struct AABB {
    pub min: (f32, f32),
    pub max: (f32, f32),
    pub size: f32,
    pub center: (f32, f32),
}

impl AABB {
    pub fn from_center_half_size(center: (f32, f32), half_size: f32) -> Self {
        Self {
            min: (center.0 - half_size, center.1 - half_size),
            max: (center.0 + half_size, center.1 + half_size),
            size: half_size * 2.0,
            center,
        }
    }

    pub fn from_center_size(center: (f32, f32), size: f32) -> Self {
        let half_size = size * 0.5;
        Self {
            min: (center.0 - half_size, center.1 - half_size),
            max: (center.0 + half_size, center.1 + half_size),
            size,
            center,
        }
    }

    pub fn contains(&self, other: &AABB) -> bool {
        other.min.0 >= self.min.0
            && other.min.1 >= self.min.1
            && other.max.0 <= self.max.0
            && other.max.1 <= self.max.1
    }

    pub fn overlaps(&self, other: &AABB) -> bool {
        self.min.0 <= other.max.0
            && self.max.0 >= other.min.0
            && self.min.1 <= other.max.1
            && self.max.1 >= other.min.1
    }

    pub fn split_into_quadrants(&self) -> [AABB; 4] {
        let half_size = self.size * 0.5;
        let quarter_size = self.size * 0.25;

        [
            AABB::from_center_size(
                (self.center.0 - quarter_size, self.center.1 - quarter_size),
                half_size,
            ),
            AABB::from_center_size(
                (self.center.0 + quarter_size, self.center.1 - quarter_size),
                half_size,
            ),
            AABB::from_center_size(
                (self.center.0 - quarter_size, self.center.1 + quarter_size),
                half_size,
            ),
            AABB::from_center_size(
                (self.center.0 + quarter_size, self.center.1 + quarter_size),
                half_size,
            ),
        ]
    }
}

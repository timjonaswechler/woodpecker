use super::*;
impl ShootingRange {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
    pub(crate) fn hit(
        &mut self,
        index: TargetId,
        radius: f32,
        distance: f32,
    ) -> Result<u32, HitRejected> {
        if self.targets_hit[index.index()] {
            return Err(HitRejected::AlreadyBroken);
        }
        if !radius.is_finite() || !(0.0..=1.0).contains(&radius) || !distance.is_finite() {
            return Err(HitRejected::InvalidSample);
        }
        if distance < MIN_TARGET_DISTANCE {
            return Err(HitRejected::TooClose(distance));
        }
        let points = if radius < 0.25 {
            50
        } else if radius < 0.55 {
            25
        } else {
            10
        };
        self.score += points;
        self.hits += 1;
        self.targets_hit[index.index()] = true;
        self.hit_flash = 0.18;
        Ok(points)
    }
}

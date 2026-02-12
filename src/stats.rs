#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RunStats {
    pub shots_fired: u32,
    pub shots_hit: u32,
    pub hits_large_asteroid: u32,
    pub hits_medium_asteroid: u32,
    pub hits_small_asteroid: u32,
    pub hits_large_alien: u32,
    pub hits_small_alien: u32,
}

impl RunStats {
    pub fn accuracy_percent(&self) -> f32 {
        if self.shots_fired == 0 {
            0.0
        } else {
            (self.shots_hit as f32 / self.shots_fired as f32) * 100.0
        }
    }

    pub fn serialize(&self) -> String {
        format!(
            "{},{},{},{},{},{},{}",
            self.shots_fired,
            self.shots_hit,
            self.hits_large_asteroid,
            self.hits_medium_asteroid,
            self.hits_small_asteroid,
            self.hits_large_alien,
            self.hits_small_alien
        )
    }

    pub fn parse(text: &str) -> Option<Self> {
        let mut parts = text.split(',');
        Some(Self {
            shots_fired: parts.next()?.parse().ok()?,
            shots_hit: parts.next()?.parse().ok()?,
            hits_large_asteroid: parts.next()?.parse().ok()?,
            hits_medium_asteroid: parts.next()?.parse().ok()?,
            hits_small_asteroid: parts.next()?.parse().ok()?,
            hits_large_alien: parts.next()?.parse().ok()?,
            hits_small_alien: parts.next()?.parse().ok()?,
        })
    }
}

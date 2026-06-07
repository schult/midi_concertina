#[derive(PartialEq)]
pub enum BellowsDirection {
    Push,
    Pull,
    None,
}

pub struct BellowsState {
    pub direction: BellowsDirection,
    pub magnitude: u16,
}

impl BellowsState {
    pub fn default() -> Self {
        Self {
            direction: BellowsDirection::None,
            magnitude: 0,
        }
    }

    pub fn new(pascals: f32) -> Self {
        const DEAD_ZONE: f32 = 70.0f32;

        // TODO: Apply mapping function.
        let ratio = (pascals / 1000.0f32).clamp(-1.0, 1.0);
        let magnitude = (16383.0f32 * ratio.abs()) as u16;

        let direction = if pascals > DEAD_ZONE {
            BellowsDirection::Push
        } else if pascals < -DEAD_ZONE {
            BellowsDirection::Pull
        } else {
            BellowsDirection::None
        };

        Self {
            direction,
            magnitude,
        }
    }
}

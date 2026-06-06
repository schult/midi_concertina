#[derive(PartialEq)]
pub enum BellowsDirection {
    Push,
    Pull,
    None,
}

pub struct BellowsState {
    pub direction: BellowsDirection,
    // TODO: pub magnitude: u8,
}

impl BellowsState {
    pub fn default() -> Self {
        Self {
            direction: BellowsDirection::None,
            // magnitude: 0,
        }
    }

    pub fn new(pascals: f32) -> Self {
        const DEAD_ZONE: f32 = 70.0f32;

        // TODO: Apply mapping function to ratio.
        // let magnitude = (127.0 * ratio.abs()) as u8;

        let direction = if pascals > DEAD_ZONE {
            BellowsDirection::Push
        } else if pascals < -DEAD_ZONE {
            BellowsDirection::Pull
        } else {
            BellowsDirection::None
        };

        Self {
            direction,
            // magnitude,
        }
    }
}

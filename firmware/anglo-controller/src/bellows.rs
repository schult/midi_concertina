#[derive(Clone, PartialEq)]
pub enum BellowsDirection {
    Push,
    Pull,
    None,
}

#[derive(Clone, PartialEq)]
pub struct BellowsState {
    pub direction: BellowsDirection,
    pub magnitude: u16,
}

impl BellowsState {
    pub const fn default() -> Self {
        Self {
            direction: BellowsDirection::None,
            magnitude: 0,
        }
    }

    pub const fn new(pascals: f32) -> Self {
        const MIN_PA: f32 = 20.0;
        const MAX_PA: f32 = 1000.0;

        let ratio = (pascals.abs() - MIN_PA) / (MAX_PA - MIN_PA);
        let ratio = ratio.clamp(0.0, 1.0);
        let index = round(ratio * (TABLE_POINTS - 1) as f32) as usize;
        let magnitude = BELLOWS_CURVE[index];

        let direction = if magnitude == 0 {
            BellowsDirection::None
        } else if pascals > 0.0 {
            BellowsDirection::Push
        } else {
            BellowsDirection::Pull
        };

        Self {
            direction,
            magnitude,
        }
    }
}

type TableEntry = u16;
const TABLE_POINTS: usize = 256;

const BELLOWS_CURVE: [TableEntry; TABLE_POINTS] = generate_bellows_curve();

const fn generate_bellows_curve() -> [TableEntry; TABLE_POINTS] {
    const VOLUME_MIN: f32 = 0.0;
    const VOLUME_BREAK: f32 = 3840.0;
    const VOLUME_MAX: f32 = 16383.0;

    const X1: [f32; 4] = [0.0, 0.125, 0.125, 1.0];
    const Y1: [f32; 4] = [VOLUME_MIN, VOLUME_MIN, VOLUME_BREAK, VOLUME_BREAK];
    const N1: usize = TABLE_POINTS / 10;
    const P1: usize = N1 * 10;

    const X2: [f32; 4] = [0.0, 0.1, 0.3, 1.0];
    const Y2: [f32; 4] = [VOLUME_BREAK, VOLUME_BREAK, VOLUME_MAX, VOLUME_MAX];
    const N2: usize = TABLE_POINTS - N1;
    const P2: usize = N2 * 10;

    let mut table = [0 as TableEntry; TABLE_POINTS];
    let (segment1, segment2) = table.split_at_mut(N1);
    segment1.copy_from_slice(bezier::<N1, P1>(&X1, &Y1).as_slice());
    segment2.copy_from_slice(bezier::<N2, P2>(&X2, &Y2).as_slice());

    table
}

const fn bezier<const N: usize, const CURVE_POINTS: usize>(
    x: &[f32; 4],
    y: &[f32; 4],
) -> [TableEntry; N] {
    let mut table = [0 as TableEntry; N];

    let mut curve_x = [0.0f32; CURVE_POINTS];
    let mut curve_y = [0.0f32; CURVE_POINTS];

    let mut i: usize = 0;
    while i < CURVE_POINTS {
        let t = i as f32 / (CURVE_POINTS - 1) as f32;
        let t_2 = t * t;
        let t_3 = t_2 * t;

        let nt = 1.0 - t;
        let nt_2 = nt * nt;
        let nt_3 = nt_2 * nt;

        curve_x[i] =
            (nt_3 * x[0]) + (3.0 * nt_2 * t * x[1]) + (3.0 * nt * t_2 * x[2]) + (t_3 * x[3]);

        curve_y[i] =
            (nt_3 * y[0]) + (3.0 * nt_2 * t * y[1]) + (3.0 * nt * t_2 * y[2]) + (t_3 * y[3]);

        i += 1;
    }

    let mut i: usize = 0;
    let mut j: usize = 0;
    while i < N {
        let x = i as f32 / (N - 1) as f32;
        while j < (CURVE_POINTS - 1) {
            let x1 = curve_x[j];
            let x2 = curve_x[j + 1];
            if x1 <= x && x <= x2 {
                let t = (x - x1) / (x2 - x1);
                let y1 = curve_y[j];
                let y2 = curve_y[j + 1];
                let y = y1 + t * (y2 - y1);
                table[i] = round(y) as TableEntry;
                break;
            }
            j += 1;
        }
        i += 1;
    }

    table
}

const fn round(x: f32) -> i32 {
    (x + (x.signum() / 2.0)) as i32
}

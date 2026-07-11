use crate::i2c;
use embassy_time::Timer;
use i2c_proto::ControllerIo;

#[derive(Clone, PartialEq)]
pub enum Direction {
    Push,
    Pull,
    None,
}

#[derive(Clone, PartialEq)]
pub struct State {
    value: i16,
}

impl State {
    pub const fn default() -> Self {
        Self { value: 0 }
    }

    pub const fn from_pascals(pascals: f32) -> Self {
        const MIN_PA: f32 = 20.0;
        const MAX_PA: f32 = 1000.0;

        let ratio = (pascals.abs() - MIN_PA) / (MAX_PA - MIN_PA);
        let ratio = ratio.clamp(0.0, 1.0);
        let index = round(ratio * (TABLE_POINTS - 1) as f32) as usize;
        let value = MAGNITUDE_TABLE[index] * pascals.signum() as i16;

        Self { value }
    }

    pub async fn read<'a>(i2c: &mut i2c::Wrapper, address: u8) -> Result<Self, i2c::Error> {
        const CONTROL_REG: u8 = 0x30;
        const DATA_REG: u8 = 0x06;

        i2c.write(address, &[CONTROL_REG, 0x0A]).await?;
        // Wait for flag indicating the data register has a new value. If we don't get a new value
        // within 1ms, we can accept the old value and catch the new value next time.
        for _ in 0..10 {
            let mut buffer = [0; 1];
            i2c.write_read(address, &[CONTROL_REG], &mut buffer).await?;
            if buffer[0] == 0x02 {
                break;
            }
            Timer::after_micros(100).await;
        }

        let mut buffer = [0; 3];
        i2c.write_read(address, &[DATA_REG], &mut buffer).await?;

        let mut reading = buffer[2] as i32;
        reading |= (buffer[1] as i32) << 8;
        reading |= (buffer[0] as i32) << 16;
        if reading & 0x00800000 != 0 {
            reading -= 16777216;
        }

        Ok(Self::from_pascals(
            1.02f32 * ((reading as f32) / 838.8608f32),
        ))
    }

    pub const fn direction(&self) -> Direction {
        if self.value > 0 {
            Direction::Push
        } else if self.value < 0 {
            Direction::Pull
        } else {
            Direction::None
        }
    }

    pub const fn magnitude_msb(&self) -> u8 {
        ((self.value.abs() >> 7) & 0x7F) as u8
    }

    pub const fn magnitude_lsb(&self) -> u8 {
        (self.value.abs() & 0x7F) as u8
    }
}

type TableEntry = i16;
const TABLE_POINTS: usize = 256;
const MAGNITUDE_TABLE: [TableEntry; TABLE_POINTS] = generate_volume_table();

const fn generate_volume_table() -> [TableEntry; TABLE_POINTS] {
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
    segment1.copy_from_slice(&bezier::<N1, P1>(&X1, &Y1));
    segment2.copy_from_slice(&bezier::<N2, P2>(&X2, &Y2));

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

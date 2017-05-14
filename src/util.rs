
/// Rounding modes for round()
#[derive(Copy,Clone,Debug,PartialEq)]
pub enum Rounding {
    Up(f32),
    Down(f32),
    Half(f32),
    None,
}

impl Rounding {
    fn seconds(&self) -> f32 {
        match *self {
            Rounding::Up(r) => r,
            Rounding::Down(r) => r,
            Rounding::Half(r) => r,
            Rounding::None => 0.0,
        }
    }
}

/// Round times with a Rounding mode
pub fn round(seconds: f64, rounding: Rounding) -> f64 {
    // Pre-round. Not ideal but this prevents times like 3.0001234 from displaying
    // surprisingly differently than the rounded versions. Eg displaying as 3.00
    // when unrounded, but 3.25 when rounded up.
    let seconds = {
        if rounding.seconds() >= 36.0 {
            (seconds / 36.0).round() * 36.0
        } else {
            seconds
        }
    };

    let res = match rounding {
        Rounding::Up(r) => (seconds / r as f64).ceil() * r as f64,
        Rounding::Down(r) => (seconds / r as f64).floor() * r as f64,
        Rounding::Half(r) => (seconds / r as f64).round() * r as f64,
        Rounding::None => seconds,
    };
    // don't return NaN
    if res.is_nan() { seconds } else { res }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TIME_ACTUAL: f64 = 38160.12345; // ~10:36am in seconds
    const TIME_UP: f64 = 38700.0; // 10:45am in seconds
    const TIME_DOWN: f64 = 37800.0; // 10:30am in seconds

    #[test]
    fn round_up_test() {
        // round up
        let res = round(TIME_ACTUAL, Rounding::Up(900.0));
        assert_eq!(res, TIME_UP);
        let res = round(TIME_UP, Rounding::Up(900.0));
        assert_eq!(res, TIME_UP);
        let res = round(TIME_UP - 1.0, Rounding::Up(900.0));
        assert_eq!(res, TIME_UP);
        assert_eq!(TIME_UP, TIME_UP);

        // test pre-rounding
        let res = round(37800.000045, Rounding::Up(900.0));
        assert_eq!(res, 37800.0);
    }

    #[test]
    fn round_down_test() {
        // round down
        let res = round(TIME_ACTUAL, Rounding::Down(900.0));
        assert_eq!(res, TIME_DOWN);
        let res = round(TIME_DOWN, Rounding::Down(900.0));
        assert_eq!(res, TIME_DOWN);
        let res = round(TIME_DOWN + 1.0, Rounding::Down(900.0));
        assert_eq!(res, TIME_DOWN);

        // test pre-rounding
        let res = round(37790.0, Rounding::Down(900.0));
        assert_eq!(res, 37800.0);
    }

    #[test]
    fn round_half_test() {
        // round half
        let res = round(TIME_ACTUAL, Rounding::Half(900.0));
        assert_eq!(res, TIME_DOWN);
        let res = round(TIME_DOWN, Rounding::Half(900.0));
        assert_eq!(res, TIME_DOWN);
    }

    #[test]
    fn round_quarter_test() {
        // round to 1/4 sec
        let res = round(TIME_ACTUAL, Rounding::Up(0.25));
        assert_eq!(res, 38160.25);
    }

    #[test]
    fn round_zero_test() {
        // round with zero
        let res = round(TIME_ACTUAL, Rounding::Up(0.0));
        assert_eq!(res, TIME_ACTUAL);
    }

    #[test]
    fn round_none_test() {
        // round None
        let res = round(TIME_ACTUAL, Rounding::None);
        assert_eq!(res, TIME_ACTUAL);

    }
}


/// Rounding modes for round()
#[derive(Copy,Clone,Debug,PartialEq)]
pub enum Rounding {
    Up(f32),
    Down(f32),
    Half(f32),
    None,
}

/// Round f64 with Rounding mode
pub fn round(seconds: f64, rounding: Rounding) -> f64 {
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

    #[test]
    fn round_up_test() {
        const TIME_ACTUAL: f64 = 38160.12345; // ~10:36am in seconds
        const TIME_UP: f64 = 38700.0; // 10:45am in seconds
        const TIME_DOWN: f64 = 37800.0; // 10:30am in seconds

        // round up
        let res = round(TIME_ACTUAL, Rounding::Up(900.0));
        assert_eq!(res, TIME_UP);
        let res = round(TIME_UP, Rounding::Up(900.0));
        assert_eq!(res, TIME_UP);
        let res = round(TIME_UP - 1.0, Rounding::Up(900.0));
        assert_eq!(res, TIME_UP);

        // round down
        let res = round(TIME_ACTUAL, Rounding::Down(900.0));
        assert_eq!(res, TIME_DOWN);
        let res = round(TIME_DOWN, Rounding::Down(900.0));
        assert_eq!(res, TIME_DOWN);
        let res = round(TIME_DOWN + 1.0, Rounding::Down(900.0));
        assert_eq!(res, TIME_DOWN);

        // round half
        let res = round(TIME_ACTUAL, Rounding::Half(900.0));
        assert_eq!(res, TIME_DOWN);
        let res = round(TIME_DOWN, Rounding::Half(900.0));
        assert_eq!(res, TIME_DOWN);

        // round to 1/4 sec
        let res = round(TIME_ACTUAL, Rounding::Up(0.25));
        assert_eq!(res, 38160.25);

        // round with zero
        let res = round(TIME_ACTUAL, Rounding::Up(0.0));
        assert_eq!(res, TIME_ACTUAL);

        // round None
        let res = round(TIME_ACTUAL, Rounding::None);
        assert_eq!(res, TIME_ACTUAL);

    }
}

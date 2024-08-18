pub trait ApproxEq {
    fn approx_eq(self, other: f64, dec: u8) -> bool;
}

impl ApproxEq for f64 {
    fn approx_eq(self, other: f64, dec: u8) -> bool {
        let factor = 10_f64.powi(dec as i32);

        let a = (self * factor).trunc();
        let b = (other * factor).trunc();

        return a == b;
    }
}

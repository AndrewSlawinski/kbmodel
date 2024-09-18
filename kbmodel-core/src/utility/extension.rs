pub trait ApproxEq
{
    fn approx_eq(self, other: f32, dec: u8) -> bool;
}

impl ApproxEq for f32
{
    fn approx_eq(self, other: f32, dec: u8) -> bool
    {
        let factor = 10_f32.powi(dec as i32);

        let a = (self * factor).trunc();
        let b = (other * factor).trunc();

        return a == b;
    }
}

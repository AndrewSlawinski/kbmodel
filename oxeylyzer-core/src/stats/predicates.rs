pub struct Predicates;
impl Predicates
{
    #[inline(always)]
    pub const fn all_equal(a: &[u8]) -> bool
    {
        let mut i = 1;

        while i < a.len()
        {
            if a[0] != a[i]
            {
                return false;
            }

            i += 1;
        }

        return true;
    }

    #[allow(unused)]
    #[inline(always)]
    const fn unique(a: &[u8]) -> bool
    {
        let mut x = 0;

        while x < a.len()
        {
            let mut j = 0;
            let mut y = 0;

            while y < a.len()
            {
                j += if a[x] == a[y] { 1 } else { 0 };
                y += 1;
            }

            if j > 1
            {
                return false;
            }

            x += 1;
        }

        return true;
    }

    #[inline]
    pub const fn is_sf(a: &[u8]) -> bool
    {
        let mut i = 1;

        match a[0] % 10
        {
            | 3 | 4 =>
            {
                while i < a.len()
                {
                    if !(a[i] % 10 == 3 || a[i] % 10 == 4) || a[i - 1] == a[i]
                    {
                        return false;
                    }

                    i += 1;
                }
            },
            | 5 | 6 =>
            {
                while i < a.len()
                {
                    if !(a[i] % 10 == 5 || a[i] % 10 == 6) || a[i - 1] == a[i]
                    {
                        return false;
                    }

                    i += 1;
                }
            },
            | _ =>
            {
                while i < a.len()
                {
                    if a[i] % 10 != a[0] % 10 || a[i - 1] == a[i]
                    {
                        return false;
                    }

                    i += 1;
                }
            },
        };

        return true;
    }

    #[inline]
    pub const fn is_scissor(a: &[u8]) -> bool
    {
        let mut i = 1;

        while i < a.len()
        {
            if a[i - 1] % 10 == a[i] % 10
            {
                return false;
            }

            let diff = a[i - 1].abs_diff(a[i]);

            if !(diff > 15 && diff < 25)
            {
                return false;
            }

            let sum = a[i - 1] + a[i];

            match a[i - 1]
            {
                | ..= 4 =>
                {
                    if !(a[i] <= 4 && sum != 7)
                    {
                        return false;
                    }
                },
                | 5 .. =>
                {
                    if !(a[i] >= 5 && sum != 11)
                    {
                        return false;
                    }
                },
            };

            i += 1;
        }

        return true;
    }

    #[inline]
    pub const fn is_ls(a: &[u8]) -> bool
    {
        let mut i = 1;

        while i < a.len()
        {
            if a[i - 1] % 10 == a[i] % 10
            {
                return false;
            }

            let min = if a[0] % 10 <= a[1] % 10
            {
                a[0] % 10
            }
            else
            {
                a[1] % 10
            };

            let max = if a[0] % 10 <= a[1] % 10
            {
                a[1] % 10
            }
            else
            {
                a[0] % 10
            };

            if !(max == 4 && (min <= 2)) && !(min == 5 && (max >= 7))
            {
                return false;
            }

            i += 1;
        }

        return true;
    }

    #[inline(always)]
    pub const fn is_inroll(a: &[u8]) -> bool
    {
        let mut i = 1;

        while i < a.len()
        {
            if a[i - 1] % 10 >= a[i] % 10
            {
                return false;
            }

            i += 1;
        }

        return true;
    }

    #[inline(always)]
    pub const fn is_outroll(a: &[u8]) -> bool
    {
        let mut i = 1;

        while i < a.len()
        {
            if a[i - 1] % 10 <= a[i] % 10
            {
                return false;
            }

            i += 1;
        }

        return true;
    }

    #[inline(always)]
    pub const fn is_lh_inroll(a: &[u8]) -> bool
    {
        let mut i = 1;

        while i < a.len()
        {
            if !Self::is_left_hand(&a[i])
            {
                return false;
            }

            if a[i - 1] % 10 >= a[i] % 10
            {
                return false;
            }

            i += 1;
        }

        return true;
    }

    #[inline(always)]
    pub const fn is_rh_inroll(a: &[u8]) -> bool
    {
        let mut i = 1;

        while i < a.len()
        {
            if Self::is_left_hand(&a[i])
            {
                return false;
            }

            if a[i - 1] % 10 >= a[i] % 10
            {
                return false;
            }

            i += 1;
        }

        return true;
    }

    #[inline(always)]
    pub const fn is_lh_outroll(a: &[u8]) -> bool
    {
        let mut i = 1;

        while i < a.len()
        {
            if !Self::is_left_hand(&a[i])
            {
                return false;
            }

            if a[i - 1] % 10 <= a[i] % 10
            {
                return false;
            }

            i += 1;
        }

        return true;
    }

    #[inline(always)]
    pub const fn is_rh_outroll(a: &[u8]) -> bool
    {
        let mut i = 1;

        while i < a.len()
        {
            if Self::is_left_hand(&a[i])
            {
                return false;
            }

            if a[i - 1] % 10 <= a[i] % 10
            {
                return false;
            }

            i += 1;
        }

        return true;
    }

    #[inline(always)]
    pub const fn is_uproll(a: &[u8]) -> bool
    {
        let mut i = 1;

        while i < a.len()
        {
            if a[i - 1] / 10 >= a[i] / 10
            {
                return false;
            }

            i += 1;
        }

        return true;
    }

    #[inline(always)]
    pub const fn is_downroll(a: &[u8]) -> bool
    {
        let mut i = 1;

        while i < a.len()
        {
            if a[i - 1] / 10 <= a[i] / 10
            {
                return false;
            }

            i += 1;
        }

        return true;
    }

    #[inline(always)]
    pub const fn is_lh_uproll(a: &[u8]) -> bool
    {
        let mut i = 1;

        while i < a.len()
        {
            if !Self::is_left_hand(&a[i])
            {
                return false;
            }

            if a[i - 1] / 10 >= a[i] / 10
            {
                return false;
            }

            i += 1;
        }

        return true;
    }

    #[inline(always)]
    pub const fn is_rh_uproll(a: &[u8]) -> bool
    {
        let mut i = 1;

        while i < a.len()
        {
            if Self::is_left_hand(&a[i])
            {
                return false;
            }

            if a[i - 1] / 10 >= a[i] / 10
            {
                return false;
            }

            i += 1;
        }

        return true;
    }

    #[inline(always)]
    pub const fn is_lh_downroll(a: &[u8]) -> bool
    {
        let mut i = 1;

        while i < a.len()
        {
            if !Self::is_left_hand(&a[i])
            {
                return false;
            }

            if a[i - 1] / 10 <= a[i] / 10
            {
                return false;
            }

            i += 1;
        }

        return true;
    }

    pub const fn is_rh_downroll(a: &[u8]) -> bool
    {
        let mut i = 1;

        while i < a.len()
        {
            if Self::is_left_hand(&a[i])
            {
                return false;
            }

            if a[i - 1] / 10 <= a[i] / 10
            {
                return false;
            }

            i += 1;
        }

        return true;
    }

    #[inline]
    pub const fn is_alternate(a: &[u8]) -> bool
    {
        let mut i = 1;

        while i < a.len()
        {
            if a[i - 1] % 10 <= 4
            {
                if a[i] % 10 < 5
                {
                    return false;
                };
            }
            else
            {
                if a[i] % 10 > 4
                {
                    return false;
                }
            }

            i += 1;
        }

        return true;
    }

    #[inline]
    pub const fn is_redirect(a: &[u8]) -> bool
    {
        let mut i = 1;
        let mut c = 0;

        match a[0] % 10
        {
            | ..= 4 =>
            {
                while i < a.len()
                {
                    if a[i] % 10 > 4
                    {
                        return false;
                    }

                    if a[i - 1] % 10 + a[i] % 10 == 7
                    {
                        return false;
                    }

                    if a[i - 1] % 10 > a[i] % 10
                    {
                        c += 1;
                    }
                    else if a[i - 1] % 10 < a[i] % 10
                    {
                        c -= 1;
                    }
                    else
                    {
                        return false;
                    }

                    i += 1;
                }
            },
            | _ =>
            {
                while i < a.len()
                {
                    if a[i] % 10 < 5
                    {
                        return false;
                    }

                    if a[i - 1] % 10 + a[i] % 10 == 11
                    {
                        return false;
                    }

                    if a[i - 1] % 10 > a[i] % 10
                    {
                        c += 1;
                    }
                    else if a[i - 1] % 10 < a[i] % 10
                    {
                        c -= 1;
                    }
                    else
                    {
                        return false;
                    }

                    i += 1;
                }
            },
        }

        return c == 0;
    }

    #[inline(always)]
    pub const fn is_left_hand(i: &u8) -> bool
    {
        return *i % 10 < 5;
    }
}

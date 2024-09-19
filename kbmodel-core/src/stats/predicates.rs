pub struct Predicates;

// LH
impl Predicates
{
    #[inline(always)]
    pub const fn is_lh_repeat(a: &[u8]) -> bool
    {
        if !Self::is_lh(&a[0])
        {
            return false;
        }

        return Self::all_equal(a);
    }

    #[inline]
    pub const fn is_lh_sf(a: &[u8]) -> bool
    {
        if !Self::is_all_lh(a)
        {
            return false;
        }

        match a[0] % 10
        {
            | 3 | 4 =>
            {
                let mut i = 1;
                while i < a.len()
                {
                    match a[i] % 10
                    {
                        | 3 | 4 =>
                        {
                            if a[i - 1] == a[i]
                            {
                                return false;
                            }
                        },
                        | _ => return false,
                    }

                    i += 1;
                }
            },
            | _ =>
            {
                let mut i = 1;
                while i < a.len()
                {
                    if !Self::is_same_column(&a[i], &a[0]) || a[i - 1] == a[i]
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
    pub const fn is_lh_scissor(a: &[u8]) -> bool
    {
        if !Self::is_all_lh(a)
        {
            return false;
        }

        let mut i = 1;
        while i < a.len()
        {
            let x = a[i - 1] % 10;
            let y = a[i] % 10;

            if x + y >= 7 || x == y
            {
                return false;
            }

            if !Self::is_row_skip(&a[i - 1], &a[i])
            {
                return false;
            }

            i += 1;
        }

        return true;
    }

    #[inline(always)]
    pub const fn is_lh_row_skip(a: &[u8]) -> bool
    {
        if !Self::is_all_lh(a)
        {
            return false;
        }

        return Self::all_row_skip(a);
    }

    #[inline]
    pub const fn is_lh_ls(a: &[u8]) -> bool
    {
        if !Self::is_all_lh(a)
        {
            return false;
        }

        let mut i = 1;
        while i < a.len()
        {
            if Self::is_same_column(&a[i - 1], &a[i])
            {
                return false;
            }

            let min = *Self::min(&(a[i - 1] % 10), &(a[i] % 10));
            let max = *Self::max(&(a[i - 1] % 10), &(a[i] % 10));

            if max != 4 || min > 2
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
        if !Self::is_all_lh(a)
        {
            return false;
        }

        return Self::is_rightward(a);
    }

    #[inline(always)]
    pub const fn is_lh_outroll(a: &[u8]) -> bool
    {
        if !Self::is_all_lh(a)
        {
            return false;
        }

        return Self::is_leftward(a);
    }

    #[inline(always)]
    pub const fn is_lh_uproll(a: &[u8]) -> bool
    {
        if !Self::is_all_lh(a)
        {
            return false;
        }

        return Self::is_upward(a);
    }

    #[inline(always)]
    pub const fn is_lh_downroll(a: &[u8]) -> bool
    {
        if !Self::is_all_lh(a)
        {
            return false;
        }

        return Self::is_downward(a);
    }

    #[inline(always)]
    pub const fn is_lh_alternate(a: &[u8]) -> bool
    {
        if !Self::is_lh(&a[0])
        {
            return false;
        }

        let mut i = 1;
        while i < a.len()
        {
            if !(Self::is_lh(&a[i - 1]) ^ Self::is_lh(&a[i]))
            {
                return false;
            }

            i += 1;
        }

        return true;
    }

    #[inline(always)]
    pub const fn is_lh_redirect(a: &[u8]) -> bool
    {
        if !Self::is_all_lh(a)
        {
            return false;
        }

        return Self::is_redirect(a);
    }
}

// RH
impl Predicates
{
    #[inline(always)]
    pub const fn is_rh_repeat(a: &[u8]) -> bool
    {
        if Self::is_lh(&a[0])
        {
            return false;
        }

        return Self::all_equal(a);
    }

    #[inline]
    pub const fn is_rh_sf(a: &[u8]) -> bool
    {
        match a[0] % 10
        {
            | .. 5 => return false,
            | 5 | 6 =>
            {
                let mut i = 1;
                while i < a.len()
                {
                    match a[i] % 10
                    {
                        | 5 | 6 =>
                        {
                            if a[i - 1] == a[i]
                            {
                                return false;
                            }
                        },
                        | _ => return false,
                    }

                    i += 1;
                }
            },
            | _ =>
            {
                let mut i = 1;
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
    pub const fn is_rh_scissor(a: &[u8]) -> bool
    {
        if !Self::is_all_rh(a)
        {
            return false;
        }

        let mut i = 1;
        while i < a.len()
        {
            let x = a[i - 1] % 10;
            let y = a[i] % 10;

            if x + y <= 11 || x == y
            {
                return false;
            }

            if !Self::is_row_skip(&a[i - 1], &a[i])
            {
                return false;
            }

            i += 1;
        }

        return true;
    }

    #[inline]
    pub const fn is_rh_row_skip(a: &[u8]) -> bool
    {
        if !Self::is_all_rh(a)
        {
            return false;
        }

        return Self::all_row_skip(a);
    }

    #[inline]
    pub const fn is_rh_ls(a: &[u8]) -> bool
    {
        if !Self::is_all_rh(a)
        {
            return false;
        }

        let mut i = 1;
        while i < a.len()
        {
            if Self::is_same_column(&a[i - 1], &a[i])
            {
                return false;
            }

            let min = *Self::min(&(a[i - 1] % 10), &(a[i] % 10));
            let max = *Self::max(&(a[i - 1] % 10), &(a[i] % 10));

            if min != 5 || max < 7
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
        if !Self::is_all_rh(a)
        {
            return false;
        }

        return Self::is_leftward(a);
    }

    #[inline(always)]
    pub const fn is_rh_outroll(a: &[u8]) -> bool
    {
        if !Self::is_all_rh(a)
        {
            return false;
        }

        return Self::is_rightward(a);
    }

    #[inline(always)]
    pub const fn is_rh_uproll(a: &[u8]) -> bool
    {
        if !Self::is_all_rh(a)
        {
            return false;
        }

        return Self::is_upward(a);
    }

    #[inline(always)]
    pub const fn is_rh_downroll(a: &[u8]) -> bool
    {
        if !Self::is_all_rh(a)
        {
            return false;
        }

        return Self::is_downward(a);
    }

    #[inline]
    pub const fn is_rh_alternate(a: &[u8]) -> bool
    {
        if Self::is_lh(&a[0])
        {
            return false;
        }

        let mut i = 1;
        while i < a.len()
        {
            if !(Self::is_lh(&a[i - 1]) ^ Self::is_lh(&a[i]))
            {
                return false;
            }

            i += 1;
        }

        return true;
    }

    #[inline(always)]
    pub const fn is_rh_redirect(a: &[u8]) -> bool
    {
        if !Self::is_all_rh(a)
        {
            return false;
        }

        return Self::is_redirect(a);
    }
}

impl Predicates
{
    #[inline(always)]
    pub const fn is_lh(i: &u8) -> bool
    {
        return *i % 10 < 5;
    }

    #[inline(always)]
    pub const fn is_lh_pair(i: &u8, j: &u8) -> bool
    {
        return Self::is_lh(i) && Self::is_lh(j);
    }

    #[inline(always)]
    pub const fn is_all_lh(a: &[u8]) -> bool
    {
        let mut i = 0;
        while i < a.len()
        {
            if a[i] % 10 > 4
            {
                return false;
            }

            i += 1;
        }

        return true;
    }

    #[inline(always)]
    pub const fn is_all_rh(a: &[u8]) -> bool
    {
        let mut i = 0;
        while i < a.len()
        {
            if a[i] % 10 < 5
            {
                return false;
            }

            i += 1;
        }

        return true;
    }

    #[inline(always)]
    pub const fn is_same_column(i: &u8, j: &u8) -> bool
    {
        return *i % 10 == *j % 10;
    }

    #[inline(always)]
    pub const fn all_same_column(a: &[u8]) -> bool
    {
        let mut i = 1;
        while i < a.len()
        {
            if !Self::is_same_column(&a[0], &a[i])
            {
                return false;
            }

            i += 1;
        }

        return true;
    }

    #[inline(always)]
    pub const fn is_row_skip(i: &u8, j: &u8) -> bool
    {
        return (*i / 10).abs_diff(*j / 10) > 1;
    }

    #[inline(always)]
    pub const fn all_row_skip(a: &[u8]) -> bool
    {
        let mut i = 1;
        while i < a.len()
        {
            if !Self::is_row_skip(&a[i - 1], &a[i])
            {
                return false;
            }

            i += 1;
        }

        return true;
    }

    #[inline(always)]
    pub const fn is_upward(a: &[u8]) -> bool
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
    pub const fn is_downward(a: &[u8]) -> bool
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
    pub const fn is_leftward(a: &[u8]) -> bool
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
    pub const fn is_rightward(a: &[u8]) -> bool
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

    #[inline]
    pub const fn is_redirect(a: &[u8]) -> bool
    {
        let mut c = 0;

        let mut i = 1;
        while i < a.len()
        {
            if a[i - 1] % 10 == a[i] % 10
                || a[i - 1] % 10 + a[i] % 10 == 7
                || a[i - 1] % 10 + a[i] % 10 == 11
            {
                return false;
            }

            c += if a[i - 1] % 10 > a[i] % 10 { 1 } else { -1 };
            i += 1;
        }

        return c == 0;
    }

    #[inline(always)]
    pub const fn min<'a>(i: &'a u8, j: &'a u8) -> &'a u8
    {
        return if *i <= *j { i } else { j };
    }

    #[inline(always)]
    pub const fn max<'a>(i: &'a u8, j: &'a u8) -> &'a u8
    {
        return if *i >= *j { i } else { j };
    }

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
}

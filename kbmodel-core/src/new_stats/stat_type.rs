use crate::new_stats::stat_type::StatType::*;
use crate::stats::predicates::Predicates;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum StatType
{
    SameFinger,
    LateralStretch,

    LInroll,
    LOutroll,

    RInroll,
    ROutroll,

    LUproll,
    LDownroll,

    RUproll,
    RDownroll,

    Alternate,
    Repeat,

    RowJump,

    LScissor,
    RScissor,

    LIndex,
    LMiddle,
    LRing,
    LPinky,

    RIndex,
    RMiddle,
    RRing,
    RPinky,
}

impl StatType
{
    #[inline]
    pub fn f(&self) -> fn(a: &[u8]) -> bool
    {
        return match self
        {
            | SameFinger => Predicates::is_sf,
            | LateralStretch => Predicates::is_ls,
            | Alternate => Predicates::is_alternate,
            | Repeat => Predicates::all_equal,

            | LScissor => Predicates::is_lh_scissor,
            | RScissor => Predicates::is_rh_scissor,
            | RowJump => Predicates::is_row_jump,

            | LInroll => Predicates::is_lh_inroll,
            | LOutroll => Predicates::is_lh_outroll,

            | RInroll => Predicates::is_rh_inroll,
            | ROutroll => Predicates::is_rh_outroll,

            | LUproll => Predicates::is_lh_uproll,
            | LDownroll => Predicates::is_lh_downroll,

            | RUproll => Predicates::is_rh_uproll,
            | RDownroll => Predicates::is_rh_downroll,

            | LPinky => |a| a[0] % 10 == 0,
            | LRing => |a| a[0] % 10 == 1,
            | LMiddle => |a| a[0] % 10 == 2,
            | LIndex => |a| a[0] % 10 == 3 || a[0] % 10 == 4,
            | RPinky => |a| a[0] % 10 == 9,
            | RRing => |a| a[0] % 10 == 8,
            | RMiddle => |a| a[0] % 10 == 7,
            | RIndex => |a| a[0] % 10 == 6 || a[0] % 10 == 5,
        };
    }

    pub const fn default() -> [StatType; 15]
    {
        return [
            SameFinger,
            LateralStretch,
            Alternate,
            Repeat,
            LScissor,
            RScissor,
            RowJump,
            LInroll,
            LOutroll,
            RInroll,
            ROutroll,
            LUproll,
            LDownroll,
            RUproll,
            RDownroll,
        ];
    }

    pub const fn columns() -> [StatType; 8]
    {
        return [
            LIndex, LMiddle, LRing, LPinky, RIndex, RMiddle, RRing, RPinky,
        ];
    }
}

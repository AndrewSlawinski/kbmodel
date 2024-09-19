use crate::stats::predicates::Predicates;
use crate::stats::stat_type::StatType::*;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum StatType
{
    LSameFinger,
    RSameFinger,

    LLateralStretch,
    RLateralStretch,

    LInward,
    RInward,

    LOutward,
    ROutward,

    LUpward,
    RUpward,

    LDownward,
    RDownward,

    LAlternate,
    RAlternate,

    LRepeat,
    RRepeat,

    LRowSkip,
    RRowSkip,

    LScissor,
    RScissor,

    LRedirect,
    RRedirect,

    LIndex,
    LMiddle,
    LRing,
    LPinky,

    RIndex,
    RMiddle,
    RRing,
    RPinky,

    LUpper,
    LCentre,
    LLower,

    RUpper,
    RCentre,
    RLower,
}

impl StatType
{
    #[inline]
    pub fn f(&self) -> fn(a: &[u8]) -> bool
    {
        return match self
        {
            | LSameFinger => Predicates::is_lh_sf,
            | RSameFinger => Predicates::is_rh_sf,

            | LLateralStretch => Predicates::is_lh_ls,
            | RLateralStretch => Predicates::is_rh_ls,

            | LAlternate => Predicates::is_lh_alternate,
            | RAlternate => Predicates::is_rh_alternate,

            | LRedirect => Predicates::is_lh_redirect,
            | RRedirect => Predicates::is_rh_redirect,

            | LRepeat => Predicates::is_lh_repeat,
            | RRepeat => Predicates::is_rh_repeat,

            | LScissor => Predicates::is_lh_scissor,
            | RScissor => Predicates::is_rh_scissor,

            | LRowSkip => Predicates::is_lh_row_skip,
            | RRowSkip => Predicates::is_rh_row_skip,

            | LInward => Predicates::is_lh_inroll,
            | RInward => Predicates::is_rh_inroll,

            | LOutward => Predicates::is_lh_outroll,
            | ROutward => Predicates::is_rh_outroll,

            | LUpward => Predicates::is_lh_uproll,
            | RUpward => Predicates::is_rh_uproll,

            | LDownward => Predicates::is_lh_downroll,
            | RDownward => Predicates::is_rh_downroll,

            | LPinky => |a| a[0] % 10 == 0,
            | LRing => |a| a[0] % 10 == 1,
            | LMiddle => |a| a[0] % 10 == 2,
            | LIndex => |a| a[0] % 10 == 3 || a[0] % 10 == 4,

            | RPinky => |a| a[0] % 10 == 9,
            | RRing => |a| a[0] % 10 == 8,
            | RMiddle => |a| a[0] % 10 == 7,
            | RIndex => |a| a[0] % 10 == 6 || a[0] % 10 == 5,

            | LUpper => |a| a[0] / 10 == 0 && a[0] % 10 < 5,
            | LCentre => |a| a[0] / 10 == 1 && a[0] % 10 < 5,
            | LLower => |a| a[0] / 10 == 2 && a[0] % 10 < 5,

            | RUpper => |a| a[0] / 10 == 0 && a[0] % 10 > 4,
            | RCentre => |a| a[0] / 10 == 1 && a[0] % 10 > 4,
            | RLower => |a| a[0] / 10 == 2 && a[0] % 10 > 4,
        };
    }

    pub const fn bigram() -> [StatType; 20]
    {
        return [
            LSameFinger,
            RSameFinger,
            LLateralStretch,
            RLateralStretch,
            LAlternate,
            RAlternate,
            LRepeat,
            RRepeat,
            LScissor,
            RScissor,
            LRowSkip,
            RRowSkip,
            LInward,
            RInward,
            LOutward,
            ROutward,
            LUpward,
            RUpward,
            LDownward,
            RDownward,
        ];
    }

    pub const fn trigram() -> [StatType; 22]
    {
        return [
            LSameFinger,
            RSameFinger,
            LLateralStretch,
            RLateralStretch,
            LAlternate,
            RAlternate,
            LRedirect,
            RRedirect,
            LRepeat,
            RRepeat,
            LScissor,
            RScissor,
            LRowSkip,
            RRowSkip,
            LInward,
            RInward,
            LOutward,
            ROutward,
            LUpward,
            RUpward,
            LDownward,
            RDownward,
        ];
    }

    pub const fn character() -> [StatType; 14]
    {
        return [
            LIndex, RIndex, LMiddle, RMiddle, LRing, RRing, LPinky, RPinky, LUpper, RUpper,
            LCentre, RCentre, LLower, RLower,
        ];
    }
}

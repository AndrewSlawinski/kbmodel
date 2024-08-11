#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum TrigramPattern {
    Alternate,
    AlternateSfs,
    Inroll,
    Outroll,
    Onehand,
    Redirect,
    RedirectSfs,
    BadRedirect,
    BadRedirectSfs,
    Sfb,
    BadSfb,
    Sft,
    Other,
    Invalid,
}

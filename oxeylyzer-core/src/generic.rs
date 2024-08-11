pub type CharToFinger = [usize; 60];
pub type Fixed<T> = [T; 30];
pub type Bigram<T> = [T; 2];
pub type NGram<T, const N: usize> = [T; N];

pub mod days {
    automod::dir!("src/days/");
}

pub mod delta {
    use std::{cmp::Ord, ops::Sub};

    #[derive(Copy, Clone, Debug)]
    pub enum Delta<T> {
        Add(T),
        Sub(T),
    }

    impl<T: Ord + Sub<Output = T>> Delta<T> {
        pub fn new(old: T, new: T) -> Delta<T> {
            if new > old {
                Delta::Add(new - old)
            } else {
                Delta::Sub(old - new)
            }
        }
    }
}

pub mod parsing {
    pub fn lines_without_endings(s: &str) -> impl Iterator<Item = &str> {
        s.lines().map(|l| {
            l.strip_suffix("\r\n")
                .or_else(|| l.strip_suffix("\n"))
                .unwrap_or(l)
        })
    }
}

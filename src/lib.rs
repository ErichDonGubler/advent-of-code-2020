pub mod days {
    automod::dir!("src/days/");
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

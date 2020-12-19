use {
    crate::parsing::lines_without_endings,
    anyhow::{anyhow, Context},
    std::cmp::Ordering,
};

#[derive(Debug)]
struct XmasEncryptedData {
    data: Vec<u64>,
    preamble_len: usize,
}

impl XmasEncryptedData {
    fn parse(s: &str, preamble_len: usize) -> anyhow::Result<Self> {
        Ok(Self {
            data: lines_without_endings(s)
                .map(|l| l.parse().context("failed to parse line"))
                .collect::<anyhow::Result<Vec<_>>>()?,
            preamble_len,
        })
    }

    fn day_input() -> Self {
        Self::parse(include_str!("d09.txt"), 25).expect("day 9 puzzle input should not be invalid")
    }

    fn sample() -> Self {
        Self::parse(
            "\
35
20
15
25
47
40
62
55
65
95
102
117
150
182
127
219
299
277
309
576
",
            5,
        )
        .expect("day 9 sample should not be invalid")
    }

    fn find_first_weakness(&self) -> Option<(usize, u64)> {
        let &Self {
            ref data,
            preamble_len,
        } = self;
        data.windows(preamble_len)
            .zip(data.iter().copied().enumerate().skip(preamble_len))
            .filter_map(|(previous_values, (next_check_idx, next_check_value))| {
                let previous_values = previous_values.iter().copied();

                let is_strong =
                    previous_values
                        .clone()
                        .zip(1..)
                        .any(|(augend, multiplier_start)| {
                            previous_values
                                .clone()
                                .skip(multiplier_start)
                                .find_map(|addend| {
                                    augend
                                        .checked_add(addend)
                                        .filter(|&sum| sum == next_check_value)
                                })
                                .is_some()
                        });

                if is_strong {
                    None
                } else {
                    Some((next_check_idx, next_check_value))
                }
            })
            .next()
    }
}

fn part_1(encrypted_data: &XmasEncryptedData) -> anyhow::Result<(usize, u64)> {
    encrypted_data
        .find_first_weakness()
        .context("no weak data found")
}

#[test]
fn p1_sample() {
    assert_eq!(part_1(&XmasEncryptedData::sample()).unwrap(), (14, 127));
}

#[test]
fn p1_answer() {
    assert_eq!(
        part_1(&XmasEncryptedData::day_input()).unwrap(),
        (555, 69316178),
    );
}

#[test]
fn p2_sample() {
    assert_eq!(part_2(&XmasEncryptedData::sample()).unwrap(), (15, 47, 62))
}

fn part_2(encrypted_data: &XmasEncryptedData) -> anyhow::Result<(u64, u64, u64)> {
    let (_weakness_idx, weakness_value) = part_1(encrypted_data)?;
    let sequence = encrypted_data
        .data
        .iter()
        .copied()
        .enumerate()
        .zip(1..)
        .filter(|&((_el_idx, el), _skip_idx)| el < weakness_value)
        .find_map(|((start_idx, start), skip_idx)| {
            let mut sum = start;
            for (end_idx, end) in encrypted_data
                .data
                .iter()
                .copied()
                .enumerate()
                .skip(skip_idx)
            {
                match sum.checked_add(end) {
                    Some(new_sum) => {
                        sum = new_sum;
                        match sum.cmp(&weakness_value) {
                            Ordering::Less => (),
                            Ordering::Equal => {
                                return Some(&encrypted_data.data[start_idx..=end_idx])
                            }
                            Ordering::Greater => break,
                        }
                    }
                    None => break,
                }
            }
            None
        })
        .with_context(|| {
            anyhow!(
                "no contiguous sequence adding up to first weakness ({}) found",
                weakness_value,
            )
        })?;
    let min = sequence.iter().copied().min().unwrap();
    let max = sequence.iter().copied().max().unwrap();
    Ok((min, max, min + max))
}

#[test]
fn p2_answer() {
    assert_eq!(
        part_2(&XmasEncryptedData::day_input()).unwrap(),
        (2834836, 6516690, 9351526),
    )
}

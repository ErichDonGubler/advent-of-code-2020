use {
    anyhow::{anyhow, Context},
    std::convert::TryFrom,
};

const SUM_TARGET: u32 = 2020;

#[derive(Debug)]
struct Answer {
    entries: Vec<(usize, u32)>,
    sum: u32,
    product: u32,
}

fn find_2020_sum_constituents(input: &str, num_entries: usize) -> anyhow::Result<Option<Answer>> {
    let expense_report_entries = input
        .lines()
        .enumerate()
        .filter_map(|(idx, l)| {
            let trimmed = l.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.parse::<u32>().with_context(|| {
                    anyhow!(
                        "failed to parse line {} as a number, which is: {:?}",
                        idx,
                        l
                    )
                }))
            }
        })
        .collect::<Result<Vec<_>, _>>()
        .context("failed to parse input")?;
    if num_entries > expense_report_entries.len() || num_entries == 0 {
        return Ok(None);
    }

    let mut entries_stack: Vec<(usize, u32)> = {
        let mut entries = Vec::with_capacity(num_entries);
        entries.extend(
            expense_report_entries
                .iter()
                .copied()
                .take(num_entries - 1)
                .enumerate(),
        );
        entries
    };

    loop {
        let checked_add = |sum: u32, entry_idx, entry| {
            let new_sum = sum.checked_add(entry);
            if new_sum.is_none() {
                eprintln!(
                    "warning: addition overflowed for {:?} ({}) + {:?}",
                    entries_stack,
                    sum,
                    (entry_idx, entry)
                )
            }
            new_sum.filter(|&s| s <= SUM_TARGET)
        };

        if let Some(last_entry) = entries_stack
            .iter()
            .copied()
            .try_fold((0usize, 0u32), |(_idx, sum), (idx, entry)| {
                checked_add(sum, idx, entry).map(|sum| (idx, sum))
            })
            .and_then(|(idx, semifinal_sum)| {
                expense_report_entries
                    .iter()
                    .copied()
                    .enumerate()
                    .skip(idx)
                    .find_map(|(idx, entry)| {
                        checked_add(semifinal_sum, idx, entry)
                            .filter(|&sum| sum == SUM_TARGET)
                            .map(|_sum| (idx, entry))
                    })
            })
        {
            entries_stack.push(last_entry);
            break Ok(Some(Answer {
                product: entries_stack
                    .iter()
                    .copied()
                    .fold(1, |product: u32, (_idx, entry)| -> u32 {
                        product.checked_mul(entry).unwrap()
                    }),
                entries: entries_stack,
                sum: SUM_TARGET,
            }));
        }

        match entries_stack
            .iter()
            .copied()
            .map(|(idx, _entry)| idx)
            .enumerate()
            .rev()
            .zip(1..)
            .find_map(|((stack_idx, entry_idx), num_digits_carried)| {
                if num_digits_carried + entry_idx < expense_report_entries.len() {
                    Some((stack_idx, entry_idx))
                } else {
                    None
                }
            }) {
            None => break Ok(None),
            Some((stack_idx, entry_idx)) => {
                entries_stack.iter_mut().skip(stack_idx).zip(1..).for_each(
                    |(stack_entry, offset)| {
                        let new_entry_idx = entry_idx + offset;
                        *stack_entry = (new_entry_idx, expense_report_entries[new_entry_idx]);
                    },
                );
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
struct Part1Answer {
    e1: (usize, u32),
    e2: (usize, u32),
    sum: u32,
    product: u32,
}

fn part_1(input: &str) -> anyhow::Result<Part1Answer> {
    find_2020_sum_constituents(input, 2)
        .and_then(|ans| {
            ans.with_context(|| anyhow!("failed to find entry pair that sums to {}", SUM_TARGET))
        })
        .map(
            |Answer {
                 entries,
                 sum,
                 product,
             }| {
                let [e1, e2] = <[(usize, u32); 2]>::try_from(entries).unwrap();
                Part1Answer {
                    e1,
                    e2,
                    sum,
                    product,
                }
            },
        )
}

#[derive(Debug, Eq, PartialEq)]
struct Part2Answer {
    e1: (usize, u32),
    e2: (usize, u32),
    e3: (usize, u32),
    sum: u32,
    product: u32,
}

fn part_2(input: &str) -> anyhow::Result<Part2Answer> {
    find_2020_sum_constituents(input, 3)
        .and_then(|ans| {
            ans.with_context(|| anyhow!("failed to find entry triplet that sums to {}", SUM_TARGET))
        })
        .map(
            |Answer {
                 entries,
                 sum,
                 product,
             }| {
                let [e1, e2, e3] = <[(usize, u32); 3]>::try_from(entries).unwrap();
                Part2Answer {
                    e1,
                    e2,
                    e3,
                    sum,
                    product,
                }
            },
        )
}

const EXAMPLE: &str = "
        1721
        979
        366
        299
        675
        1456
        ";
const INPUT: &str = include_str!("d01.txt");

#[test]
fn d01p1_sample() {
    assert_eq!(
        part_1(EXAMPLE).unwrap(),
        Part1Answer {
            e1: (0, 1721),
            e2: (3, 299),
            sum: 2020,
            product: 514579,
        },
    );
}

#[test]
fn d01p1_answer() {
    assert_eq!(
        part_1(INPUT).unwrap(),
        Part1Answer {
            e1: (68, 1751),
            e2: (140, 269),
            sum: 2020,
            product: 471019,
        },
    );
}

#[test]
fn d01p2_sample() {
    assert_eq!(
        part_2(EXAMPLE).unwrap(),
        Part2Answer {
            e1: (1, 979),
            e2: (2, 366),
            e3: (4, 675),
            sum: 2020,
            product: 241861950,
        },
    );
}

#[test]
fn d01p2_answer() {
    assert_eq!(
        part_2(INPUT).unwrap(),
        Part2Answer {
            e1: (62, 1442),
            e2: (105, 396),
            e3: (150, 182),
            sum: 2020,
            product: 103927824,
        },
    );
}

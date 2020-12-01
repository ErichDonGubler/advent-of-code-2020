use {
    anyhow::{anyhow, Context},
    std::{
        convert::TryInto,
        io::{stdin, Read},
    },
    structopt::StructOpt,
};

#[derive(Debug, StructOpt)]
enum CliArgs {
    Part1,
    Part2,
}

fn main() -> anyhow::Result<()> {
    let args = CliArgs::from_args();
    let stdin = {
        let mut buf = String::new();
        stdin()
            .read_to_string(&mut buf)
            .context("failed to read stdin as UTF-8")?;
        buf
    };

    match args {
        CliArgs::Part1 => part_1(&stdin).map(
            |Part1Answer {
                e1,
                e2,
                sum,
                product,
            }| {
                println!("{} + {} = {}", e1, e2, sum);
                println!("{} x {} = {:?}", e1, e2, product);
            },
        ),
        CliArgs::Part2 => todo!("I haven't solved part 1 yet!"),
    }
}

const SUM_TARGET: u32 = 2020;

#[derive(Debug)]
struct Answer {
    entries: Vec<u32>,
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

    let mut entries_indices = (0..num_entries)
        .map(|idx| (idx, expense_report_entries[idx]))
        .collect::<Vec<_>>();

    loop {
        if entries_indices
            .iter()
                .copied()
                .map(|(idx, entry)| entry)
                .try_fold(0u32, |sum, entry| {
                    sum.checked_add(entry).filter(|&s| s <= SUM_TARGET)
                })
        .is_some()
        {

        }

        let carry_idx = entries_indices.iter().copied().enumerate().rev().find(|&(idx, _entry)| idx < expense_report_entries.len());
        entries_indices[carry_idx] += 1;
        entries_indices.iter_mut().skip(carry_idx + 1).
    }

    expense_report_entries
        .iter()
        .copied()
        .enumerate()
        .find_map(|(idx, e1)| {
            expense_report_entries
                .iter()
                .copied()
                .skip(idx + 1)
                .find_map(|e2| {
                    e1.checked_add(e2)
                        .filter(|&sum| sum == SUM_TARGET)
                        .map(|sum| (e1, e2, sum))
                })
        })
    .and_then(|(e1, e2, sum)| {
        let product = e1.checked_mul(e2).with_context(|| anyhow!("failed "))?;
        Ok(Part1Answer {
            e1,
            e2,
            sum,
            product,
        })
    })
}

#[derive(Debug, Eq, PartialEq)]
struct Part1Answer {
    e1: u32,
    e2: u32,
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
            let [e1, e2] = entries.try_into().unwrap();
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
    e1: u32,
    e2: u32,
    e3: u32,
    sum: u32,
    product: u32,
}

fn part_2(input: &str) -> anyhow::Result<Part2Answer> {
    find_2020_sum_constituents(input, 3).map(
        |Answer {
            entries,
            sum,
            product,
        }| {
            let [e1, e2, e3] = entries.try_into().unwrap();
            Part1Answer {
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

#[test]
            fn d01p1_sample() {
                assert_eq!(
                    part_1(EXAMPLE).unwrap(),
                    Part1Answer {
                        e1: 1721,
                        e2: 299,
                        sum: 2020,
                        product: 514579,
                    }
                )
            }

#[test]
fn d01p1_answer() {
    assert_eq!(
        part_1(include_str!("d01.txt")).unwrap(),
        Part1Answer {
            e1: 1751,
            e2: 269,
            sum: 2020,
            product: 471019,
        }
    );
}

#[test]
fn d01p2_sample() {
    assert_eq!(
        part_2(EXAMPLE).unwrap(),
        Part2Answer {
            e1: 979,
            e2: 366,
            e3: 675,
            sum: 2020,
            product: 241861950,
        }
    )
}

#[test]
fn d01p2_answer() {
    todo!();
}

use {
    anyhow::{anyhow, Context},
    std::io::{stdin, Read},
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

#[derive(Debug, Eq, PartialEq)]
struct Part1Answer {
    e1: u32,
    e2: u32,
    sum: u32,
    product: u32,
}

fn part_1(input: &str) -> anyhow::Result<Part1Answer> {
    const SUM_TARGET: u32 = 2020;

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
        .with_context(|| anyhow!("failed to find entry pair that sums to {}", SUM_TARGET))
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

#[test]
fn d01p1_sample() {
    assert_eq!(
        part_1(
            "
            1721
            979
            366
            299
            675
            1456
            "
        )
        .unwrap(),
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

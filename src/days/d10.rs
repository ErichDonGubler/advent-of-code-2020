use {
    crate::parsing::lines_without_endings,
    anyhow::{anyhow, ensure, Context},
    std::{
        convert::{TryFrom, TryInto},
        ops::Deref,
        str::FromStr,
    },
};

#[derive(Debug)]
pub struct JoltageAdapterSet(
    /// Adapters in the set, sorted in ascending order of joltage rating.
    Vec<u16>,
);

impl FromStr for JoltageAdapterSet {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut adapters = lines_without_endings(s)
            .enumerate()
            .map(|(line_idx, l)| -> anyhow::Result<u16> {
                l.parse::<u16>()
                    .with_context(|| anyhow!("failed to parse line {}", line_idx))
            })
            .collect::<anyhow::Result<Vec<_>>>()?;
        ensure!(!adapters.is_empty(), "no adapters specified");
        adapters.push(0);
        adapters.sort();

        Ok(Self(adapters))
    }
}

impl Deref for JoltageAdapterSet {
    type Target = [u16];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl JoltageAdapterSet {
    #[track_caller]
    fn joltage_flows_between_adapters(source: u16, target: u16) -> bool {
        (1..=3).contains(&(target.checked_sub(source).unwrap()))
    }

    pub fn connectable(&self) -> ConnectableJoltageAdapterSet<'_> {
        let end_idx = self
            .windows(2)
            .zip(1..)
            .take_while(|&(window, _end_idx)| matches!(window, &[left, right] if Self::joltage_flows_between_adapters(left, right)))
            .map(|(_window, end_idx)| end_idx)
            .last().unwrap();

        ConnectableJoltageAdapterSet(&self[1..=end_idx]) // we use `1` here because `0` will always be the first element
    }

    /// Calculates the sum of the number of members of each power set of elements in runs of
    /// optional adapter elements of this sequence.
    pub fn num_valid_variants(&self) -> anyhow::Result<usize> {
        // Alright, I had to look this one up. I still don't feel like I completely grok the theory
        // behind it -- I understand the logic for generating cases for sequence possibility
        // multiplication, but not _why_ that logic is valid.

        pub struct PossibilityAccumulator {
            last_skippable: u16,
            num_consecutive_single_steps: usize,
            num_possible_sequences: usize,
        }

        impl PossibilityAccumulator {
            fn new() -> Self {
                Self {
                    last_skippable: 0,
                    num_consecutive_single_steps: 0,
                    num_possible_sequences: 1,
                }
            }

            fn on_break_single_step_skippable_streak(&mut self) -> anyhow::Result<()> {
                let Self {
                    last_skippable: _,
                    num_consecutive_single_steps,
                    num_possible_sequences,
                } = self;

                let naive_new_possibilities = (dbg!(*num_consecutive_single_steps))
                    .try_into()
                    .ok()
                    .and_then(|steps| 2usize.checked_pow(steps))
                    .context(
                        "naive number of new possible sequences not representable with `usize`",
                    )?;

                *num_possible_sequences = (*num_possible_sequences)
                    .checked_mul(dbg!(
                        naive_new_possibilities - (naive_new_possibilities * 3 / 16)
                    ))
                    .context("accumulated possible sequences no representable with `usize`")?;
                *num_consecutive_single_steps = 0;

                Ok(())
            }

            pub fn accumulate(&mut self, skippable: u16) -> anyhow::Result<()> {
                if dbg!(self.last_skippable + 1) == dbg!(skippable) {
                    self.num_consecutive_single_steps += 1;
                } else {
                    self.on_break_single_step_skippable_streak()?;
                    self.num_consecutive_single_steps = 1;
                };

                self.last_skippable = skippable;

                Ok(())
            }

            pub fn finished(mut self) -> anyhow::Result<usize> {
                self.on_break_single_step_skippable_streak()
                    .map(|()| self.num_possible_sequences)
            }
        }

        let mut acc = PossibilityAccumulator::new();
        self.windows(3)
            .filter_map(|window| {
                let [left, mid, right] = <[_; 3]>::try_from(window).unwrap();
                if Self::joltage_flows_between_adapters(left, right) {
                    Some(mid)
                } else {
                    None
                }
            })
            .try_for_each(|skippable| acc.accumulate(skippable))?;
        acc.finished()
    }
}

#[derive(Debug)]
pub struct ConnectableJoltageAdapterSet<'a>(&'a [u16]);

impl Deref for ConnectableJoltageAdapterSet<'_> {
    type Target = [u16];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ConnectableJoltageAdapterSet<'_> {
    fn diff_counts(&self) -> JoltDiffCounts {
        let mut diff_counts = JoltDiffCounts {
            single: 0,
            triple: 1, // including the one at the end (AKA the laptop adapter)
        };

        let mut accumulate_diff = |diff| match diff {
            1 => diff_counts.single += 1,
            2 => (),
            3 => diff_counts.triple += 1,
            _ => unreachable!(),
        };
        accumulate_diff(*self.first().unwrap());
        self.windows(2).for_each(|window| {
            let [source, target] = <[_; 2]>::try_from(window).unwrap();
            accumulate_diff(target - source)
        });

        diff_counts
    }
}

#[derive(Debug, Eq, PartialEq)]
struct JoltDiffCounts {
    single: usize,
    triple: usize,
}

const FIRST_SAMPLE: &str = "\
16
10
15
5
1
11
7
19
6
12
4
";

const SECOND_SAMPLE: &str = "\
28
33
18
42
31
14
46
20
48
47
24
23
49
45
19
38
39
11
1
32
25
35
8
17
7
9
4
2
34
10
3
";

#[test]
fn p1_samples() {
    #[track_caller]
    fn test_sample(s: &str, expected_max_joltage: u16, expected_jolt_diff_counts: JoltDiffCounts) {
        let adapters = s.parse::<JoltageAdapterSet>().unwrap();

        let connectable_adapters = adapters.connectable();

        let max_joltage = connectable_adapters
            .last()
            .unwrap()
            .checked_add(3)
            .context("max joltage from adapters in bag not representable with u16")
            .unwrap();

        assert_eq!(max_joltage, expected_max_joltage);

        assert_eq!(
            connectable_adapters.diff_counts(),
            expected_jolt_diff_counts,
        );
    }
    test_sample(
        FIRST_SAMPLE,
        22,
        JoltDiffCounts {
            single: 7,
            triple: 5,
        },
    );

    test_sample(
        SECOND_SAMPLE,
        52,
        JoltDiffCounts {
            single: 22,
            triple: 10,
        },
    );
}

const INPUT: &str = include_str!("d10.txt");

#[test]
fn p1_answer() {
    let diff_counts = INPUT
        .parse::<JoltageAdapterSet>()
        .unwrap()
        .connectable()
        .diff_counts();
    assert_eq!(
        diff_counts,
        JoltDiffCounts {
            single: 72,
            triple: 36,
        }
    );
    assert_eq!(
        diff_counts
            .single
            .checked_mul(diff_counts.triple)
            .context("diff count stat multiplication not representable")
            .unwrap(),
        2592 // 72 * 36
    );
}

#[test]
fn p2_sample() {
    assert_eq!(part_2(FIRST_SAMPLE).unwrap(), 8);
    assert_eq!(part_2(SECOND_SAMPLE).unwrap(), 19208);
}

#[test]
fn p2_my_research() {
    assert_eq!(part_2("1\n2\n3\n4\n5").unwrap(), 13);
}

fn part_2(s: &str) -> anyhow::Result<usize> {
    Ok(s.parse::<JoltageAdapterSet>()?.num_valid_variants()?)
}

#[test]
fn p2_answer() {
    assert_eq!(part_2(INPUT).unwrap(), 198428693313536);
}

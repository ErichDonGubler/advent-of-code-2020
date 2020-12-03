use {
    anyhow::{anyhow, Context},
    itertools::Itertools,
    re_parse::ReParse,
    regex::Regex, // FIXME: file an upstream PR to get rid of the need for this
    serde::Deserialize,
    std::{borrow::Cow, convert::TryInto, num::NonZeroUsize, ops::RangeInclusive},
};

trait PasswordPolicy
where
    Self: Sized,
{
    fn from_raw(lower: u8, upper: u8, character: char) -> anyhow::Result<Self>;
    fn validate(&self, password: &str) -> bool;
}

/// Parses a policy-password pair of the form:
///
/// ```txt
/// <lower>-<upper> <char>: <password>
/// ```
fn parse_policy_password_pair<T>(s: &str) -> anyhow::Result<(T, Cow<'_, str>)>
where
    T: PasswordPolicy,
{
    #[derive(Debug, Deserialize, ReParse)]
    #[re_parse(regex = "^(?P<lower>[0-9]+)-(?P<upper>[0-9]+) (?P<character>.): (?P<password>.*)$")]
    struct RawPasswordPolicy<'a> {
        lower: u8,
        upper: u8,
        character: char,
        password: Cow<'a, str>,
    }

    let RawPasswordPolicy {
        character,
        lower,
        password,
        upper,
    } = s
        .parse()
        .context("failed to parse raw policy password pair")?;

    let concrete_policy = T::from_raw(lower, upper, character)
        .context("parse succeeded, but conversion to concrete policy failed")?;

    Ok((concrete_policy, password))
}

#[derive(Debug, Eq, PartialEq)]
struct MisrememberedPasswordPolicy {
    range: RangeInclusive<u8>,
    character: char,
}

impl PasswordPolicy for MisrememberedPasswordPolicy {
    fn from_raw(lower: u8, upper: u8, character: char) -> anyhow::Result<Self> {
        // TODO: There's some error surface not getting modeled here yet. Ew.
        Ok(Self {
            range: RangeInclusive::new(lower, upper),
            character,
        })
    }

    fn validate(&self, password: &str) -> bool {
        let Self { character, range } = self;
        let count = match password
            .chars()
            .filter(|c| c == character)
            .count()
            .try_into()
        {
            Ok(c) => c,
            Err(_) => {
                panic!(
                    "number of matching password policy characters is {} \
                    (greater than representable by `u8`), which is unexpected"
                )
            }
        };
        range.contains(&count)
    }
}

fn parse_password_policy_lines<T>(
    s: &str,
) -> impl Iterator<Item = anyhow::Result<(T, Cow<'_, str>)>>
where
    T: PasswordPolicy,
{
    s.lines()
        .map(|l| {
            l.strip_suffix("\r\n")
                .or_else(|| l.strip_suffix("\n"))
                .unwrap_or(l)
        })
        .filter(|l| !l.is_empty())
        .map(parse_policy_password_pair)
}

fn part_1(s: &str) -> usize {
    parse_password_policy_lines::<MisrememberedPasswordPolicy>(s)
        .filter_map(|res| res.ok())
        .filter(|(pol, pw)| pol.validate(&pw))
        .count()
}

#[derive(Debug, Eq, PartialEq)]
struct ActualPasswordPolicy {
    positions: [NonZeroUsize; 2],
    character: char,
}

impl PasswordPolicy for ActualPasswordPolicy {
    fn from_raw(lower: u8, upper: u8, character: char) -> anyhow::Result<Self> {
        Ok(Self {
            character,
            positions: {
                let parse_one_based_idx = |idx_name, idx| {
                    NonZeroUsize::new(usize::from(idx)).ok_or_else(|| {
                        anyhow!(
                            "received position 0 for 1-based character {} index",
                            idx_name
                        )
                    })
                };
                [
                    parse_one_based_idx("first", lower)?,
                    parse_one_based_idx("second", upper)?,
                ]
            },
        })
    }

    fn validate(&self, password: &str) -> bool {
        let Self {
            character,
            positions,
        } = self;

        password
            .chars()
            .zip((1..).map(|x| NonZeroUsize::new(x).unwrap()))
            .filter(|(c, idx)| positions.contains(idx) && c == character)
            .count()
            == 1
    }
}

fn part_2(s: &str) -> usize {
    parse_password_policy_lines::<ActualPasswordPolicy>(s)
        .filter_map(|res| res.ok())
        .filter(|(pol, pw)| pol.validate(&pw))
        .count()
}

const SAMPLE: &str = "\
1-3 a: abcde
1-3 b: cdefg
2-9 c: ccccccccc
";

const INPUT: &str = include_str!("d02.txt");

#[test]
fn d02_p1_sample() {
    assert_eq!(
        parse_password_policy_lines::<MisrememberedPasswordPolicy>(SAMPLE)
            .filter_map(|res| res.ok())
            .filter(|(pol, pw)| !pol.validate(&pw))
            .collect_tuple::<(_,)>(),
        Some(((
            MisrememberedPasswordPolicy {
                range: RangeInclusive::new(1, 3),
                character: 'b',
            },
            "cdefg".into(),
        ),)),
    );
    assert_eq!(part_1(SAMPLE), 2);
}

#[test]
fn d02_p1_answer() {
    assert_eq!(part_1(INPUT), 603);
}

#[test]
fn d02_p2_sample() {
    assert_eq!(
        parse_password_policy_lines::<ActualPasswordPolicy>(SAMPLE)
            .filter_map(|res| res.ok())
            .filter(|(pol, pw)| !pol.validate(&pw))
            .collect_tuple::<(_, _)>(),
        Some((
            (
                ActualPasswordPolicy {
                    positions: [NonZeroUsize::new(1).unwrap(), NonZeroUsize::new(3).unwrap()],
                    character: 'b',
                },
                "cdefg".into(),
            ),
            (
                ActualPasswordPolicy {
                    positions: [NonZeroUsize::new(2).unwrap(), NonZeroUsize::new(9).unwrap()],
                    character: 'c',
                },
                "ccccccccc".into(),
            )
        )),
    );
    assert_eq!(part_2(SAMPLE), 1);
}

#[test]
fn d02_p2_answer() {
    assert_eq!(part_2(INPUT), 404);
}

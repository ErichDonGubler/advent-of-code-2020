use {
    anyhow::Context,
    itertools::Itertools,
    re_parse::ReParse,
    regex::Regex, // FIXME: file an upstream PR to get rid of the need for this
    serde::Deserialize,
    std::{borrow::Cow, convert::TryInto, ops::RangeInclusive},
};

#[derive(Debug, Eq, PartialEq)]
struct PasswordPolicy {
    range: RangeInclusive<u8>,
    character: char,
}

impl PasswordPolicy {
    fn validate(&self, password: &str) -> bool {
        let Self { character, range } = self;
        println!(
            "checking if there are {:?} {:?} characters in {:?}",
            range, character, password,
        );
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

/// Parses a policy-password pair of the form:
///
/// ```txt
/// <lower>-<upper> <char>: <password>
/// ```
fn parse_policy_password_pair(s: &str) -> anyhow::Result<(PasswordPolicy, Cow<'_, str>)> {
    #[derive(Debug, Deserialize, ReParse)]
    #[re_parse(regex = "^(?P<lower>[0-9]+)-(?P<upper>[0-9]+) (?P<character>.): (?P<password>.*)$")]
    struct RawPolicyPasswordPair<'a> {
        lower: u8,
        upper: u8,
        character: char,
        password: Cow<'a, str>,
    }

    let RawPolicyPasswordPair {
        character,
        lower,
        password,
        upper,
    } = s.parse().context("failed to parse policy password pair")?;

    // RawPolicyPasswordPair::new;
    Ok((
        PasswordPolicy {
            range: RangeInclusive::new(lower, upper),
            character,
        },
        password,
    ))
}

fn parse_policy_password_pairs(
    s: &str,
) -> impl Iterator<Item = anyhow::Result<(PasswordPolicy, Cow<'_, str>)>> {
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
    parse_policy_password_pairs(s)
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
        parse_policy_password_pairs(SAMPLE)
            .filter_map(|res| res.ok())
            .filter(|(pol, pw)| !pol.validate(&pw))
            .collect_tuple::<(_,)>(),
        Some(((
            PasswordPolicy {
                range: RangeInclusive::new(1, 3),
                character: 'b',
            },
            "cdefg".into(),
        ),))
    );
    assert_eq!(part_1(SAMPLE), 2);
}

#[test]
fn d02_p1_answer() {
    assert_eq!(part_1(INPUT), 603);
}

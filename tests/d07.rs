use {
    advent_of_code_2020::parsing::lines_without_endings,
    anyhow::{anyhow, bail, ensure, Context},
    itertools::Itertools,
    std::{
        collections::{hash_map::HashMap, HashSet},
        num::NonZeroU8,
        ops::Deref,
    },
};

const SAMPLE: &str = "\
light red bags contain 1 bright white bag, 2 muted yellow bags.
dark orange bags contain 3 bright white bags, 4 muted yellow bags.
bright white bags contain 1 shiny gold bag.
muted yellow bags contain 2 shiny gold bags, 9 faded blue bags.
shiny gold bags contain 1 dark olive bag, 2 vibrant plum bags.
dark olive bags contain 3 faded blue bags, 4 dotted black bags.
vibrant plum bags contain 5 faded blue bags, 6 dotted black bags.
faded blue bags contain no other bags.
dotted black bags contain no other bags.
";

const INPUT: &str = include_str!("d07.txt");

#[test]
fn d07_p1_sample() {
    assert_eq!(part_1(SAMPLE).unwrap(), 4);
}

#[derive(Debug)]
struct LuggageRules<'a>(HashMap<&'a str, LuggageRule<'a>>);

impl<'a> Deref for LuggageRules<'a> {
    type Target = HashMap<&'a str, LuggageRule<'a>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
struct LuggageRule<'a>(HashMap<&'a str, NonZeroU8>);

impl<'a> Deref for LuggageRule<'a> {
    type Target = HashMap<&'a str, NonZeroU8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn parse_luggage_rules(s: &str) -> anyhow::Result<LuggageRules<'_>> {
    let mut rules = HashMap::new();
    let mut rules_lines = HashMap::<_, u64>::new();
    let mut unverified = HashSet::new();
    lines_without_endings(s)
        .zip(1u64..)
        .map(|(l, line_num)| {
            (|| -> anyhow::Result<()> {
                let l = {
                    const PERIOD: &str = ".";
                    match l.strip_suffix(PERIOD) {
                        Some(l) => l,
                        None => bail!("rule didn't end in {:?}", PERIOD),
                    }
                };
                const BAGS_CONTAIN: &str = " bags contain ";
                let (color, raw_bags_inside) = l
                    .splitn(2, BAGS_CONTAIN)
                    .collect_tuple()
                    .with_context(|| anyhow!("unable to find {:?}", BAGS_CONTAIN))?;
                match rules.get(color) {
                    None => {
                        rules_lines.insert(color, line_num);
                        unverified.remove(color);
                    }
                    Some(entry) => bail!(
                        "duplicate rule for {:?} {:?}; previously specified on line {}",
                        color,
                        entry,
                        rules_lines.get(color).unwrap(),
                    ),
                };
                let bags_inside = {
                    if raw_bags_inside == "no other bags" {
                        LuggageRule(HashMap::new())
                    } else {
                        raw_bags_inside.split(", ").map(|raw_bag| -> anyhow::Result<_> {
                            let mut count_word_split = raw_bag.splitn(2, ' ');

                            let count = {
                                let raw_count = count_word_split.next().unwrap();
                                raw_count.parse::<NonZeroU8>().with_context(|| anyhow!("expected non-zero positive integer for contained bag count, got {:?}", raw_count))?
                            };

                            let contained_color = {
                                let raw_bag_desc = count_word_split.next().context("expected bag description after count")?;

                                let proper_bag_keyword = if count == NonZeroU8::new(1).unwrap() {
                                    " bag"
                                } else {
                                    " bags"
                                };

                                raw_bag_desc.strip_suffix(proper_bag_keyword).ok_or_else(|| {
                                    anyhow!(
                                        "expected {:?} at the end of the bag description, got {:?}",
                                        proper_bag_keyword,
                                        raw_bag_desc,
                                    )
                                })?
                            };

                            if rules.get(contained_color).is_none() {
                                unverified.insert(contained_color);
                            }

                            Ok((contained_color, count))
                        }).collect::<Result<HashMap<_, _>, _>>().map(LuggageRule)?
                    }
                };
                rules.insert(color, bags_inside);
                Ok(())
            })()
            .with_context(|| anyhow!("failed to parse line {}", line_num))
        })
    .collect::<anyhow::Result<()>>()?;
    ensure!(
        unverified.is_empty(),
        "the follows bag colors were referred to as being contained by other bag colors, \
        but are unspecified: {:?}",
        unverified,
    );
    Ok(LuggageRules(rules))
}

fn part_1(s: &str) -> anyhow::Result<usize> {
    fn does_color_contain_color<'a>(
        memo: &mut HashMap<&'a str, bool>,
        luggage_rules: &LuggageRules<'a>,
        container: &'a str,
        containee: &'a str,
    ) -> bool {
        if let Some(&memoized) = memo.get(container) {
            return memoized;
        }
        let answer = luggage_rules
            .get(container)
            .unwrap()
            .keys()
            .any(|&contained| {
                contained == containee
                    || does_color_contain_color(memo, luggage_rules, contained, containee)
            });
        memo.insert(container, answer);
        answer
    }
    let luggage_rules = parse_luggage_rules(s)?;
    let mut memoized_query = HashMap::new();
    Ok(luggage_rules
        .keys()
        .filter(|color| {
            does_color_contain_color(&mut memoized_query, &luggage_rules, color, "shiny gold")
        })
        .count())
}

#[test]
fn d07_p1_answer() {
    assert_eq!(part_1(INPUT).unwrap(), 151);
}

#[test]
fn d07_p2_sample_1() {
    assert_eq!(part_2(SAMPLE).unwrap(), 32)
}

#[test]
fn d07_p2_sample_2() {
    assert_eq!(
        part_2(
            "\
shiny gold bags contain 2 dark red bags.
dark red bags contain 2 dark orange bags.
dark orange bags contain 2 dark yellow bags.
dark yellow bags contain 2 dark green bags.
dark green bags contain 2 dark blue bags.
dark blue bags contain 2 dark violet bags.
dark violet bags contain no other bags.
"
        )
        .unwrap(),
        126
    );
}

fn part_2(s: &str) -> anyhow::Result<u32> {
    fn num_bags_for_color<'a>(
        memo: &mut HashMap<&'a str, u32>,
        luggage_rules: &LuggageRules<'a>,
        container: &'a str,
    ) -> u32 {
        if let Some(&memoized) = memo.get(container) {
            return memoized;
        }
        let answer = luggage_rules
            .get(container)
            .unwrap()
            .iter()
            .map(|(&contained, count)| {
                num_bags_for_color(memo, luggage_rules, contained)
                    .checked_mul(count.get().into())
                    .unwrap()
            })
            .fold(1u32, |sum, count| sum.checked_add(count).unwrap());
        memo.insert(container, answer);
        answer
    }
    Ok(
        num_bags_for_color(&mut HashMap::new(), &parse_luggage_rules(s)?, "shiny gold") - 1, /* because we don't include the outermost bag (???) */
    )
}

#[test]
fn d07_p2_answer() {
    assert_eq!(part_2(INPUT).unwrap(), 41559);
}

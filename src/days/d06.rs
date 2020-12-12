use {crate::parsing::lines_without_endings, std::collections::HashSet};

const SAMPLE: &str = "\
abc

a
b
c

ab
ac

a
a
a
a

b
";

#[test]
fn p1_sample() {
    assert_eq!(sum_of_unique_question_answer_counts(SAMPLE), 11);
}

fn sum_of_unique_question_answer_counts(s: &str) -> usize {
    s.split("\n\n")
        .map(|group| {
            group
                .split_whitespace()
                .flat_map(|c| c.chars())
                .collect::<HashSet<_>>()
                .len()
        })
        .sum()
}

const INPUT: &str = include_str!("d06.txt");

#[test]
fn p1_answer() {
    assert_eq!(sum_of_unique_question_answer_counts(INPUT), 7128);
}

#[test]
fn p2_sample() {
    assert_eq!(
        sum_of_group_individuals_who_answered_yes_in_each_group(SAMPLE),
        6
    );
}

fn sum_of_group_individuals_who_answered_yes_in_each_group(s: &str) -> usize {
    s.split("\n\n")
        .map(|group| {
            let mut questions = lines_without_endings(group);
            let mut individuals_responding_yes_to_everything =
                questions.next().unwrap().chars().collect::<HashSet<_>>();
            questions.for_each(|question| {
                individuals_responding_yes_to_everything = question
                    .chars()
                    .filter(|individual_responding_yes| {
                        individuals_responding_yes_to_everything.contains(individual_responding_yes)
                    })
                    .collect::<HashSet<_>>();
            });
            individuals_responding_yes_to_everything.len()
        })
        .sum()
}

#[test]
fn p2_answer() {
    assert_eq!(
        sum_of_group_individuals_who_answered_yes_in_each_group(INPUT),
        3640
    );
}

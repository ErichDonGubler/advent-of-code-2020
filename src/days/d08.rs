use {
    crate::parsing::lines_without_endings,
    anyhow::{anyhow, bail, Context},
    itertools::Itertools,
    std::{collections::HashSet, convert::TryInto},
};

const SAMPLE: &str = "\
nop +0
acc +1
jmp +4
acc +3
jmp -3
acc -99
acc +1
jmp -4
acc +6
";

const INPUT: &str = include_str!("d08.txt");

#[test]
fn d08_p1_sample() {
    assert_eq!(part_1(SAMPLE).unwrap(), 5);
}

#[derive(Clone, Debug)]
struct BootCodeInstruction {
    operation: BootCodeOperation,
    argument: i16,
}

#[derive(Clone, Copy, Debug)]
enum BootCodeOperation {
    Accumulate,
    Jump,
    NoOp,
}

#[derive(Debug)]
struct BootCodeEmulator {
    instruction_counter: usize,
    accumulator: i32,
}

impl BootCodeEmulator {
    fn zeroed() -> Self {
        Self {
            instruction_counter: 0,
            accumulator: 0,
        }
    }

    fn execute_single_instruction(
        &mut self,
        instructions: &[BootCodeInstruction],
    ) -> anyhow::Result<()> {
        (|| {
            let Self {
                instruction_counter,
                accumulator,
            } = self;

            let instruction = instructions
                .get(*instruction_counter)
                .context("instruction counter out-of-bounds")?;
            let increment_inst_counter = |counter: &mut usize| {
                counter
                    .checked_add(1)
                    .map(|new_counter| *counter = new_counter)
                    .context("next instruction counter increment overflows")
            };
            match instruction.clone() {
                BootCodeInstruction {
                    operation: BootCodeOperation::NoOp,
                    argument: _,
                } => increment_inst_counter(instruction_counter),
                BootCodeInstruction {
                    operation: BootCodeOperation::Jump,
                    argument,
                } => if argument.is_positive() {
                    instruction_counter
                        .checked_add(argument.try_into().unwrap())
                        .context("jump instruction overflowed")
                } else {
                    instruction_counter
                        .checked_sub(argument.checked_neg().unwrap().try_into().unwrap())
                        .context("jump instruction underflowed")
                }
                .map(|new_counter| *instruction_counter = new_counter),
                BootCodeInstruction {
                    operation: BootCodeOperation::Accumulate,
                    argument,
                } => accumulator
                    .checked_add(argument.into())
                    .context("accumulator went out-of-range")
                    .map(|new_acc| *accumulator = new_acc)
                    .and_then(|()| increment_inst_counter(instruction_counter)),
            }
            .with_context(move || anyhow!("failed to execute instruction {:?}", instruction))
        })()
        .with_context(|| {
            anyhow!(
                "failed to execute next instruction; current state: {:?}",
                self
            )
        })
    }
}

fn parse_instructions(s: &str) -> anyhow::Result<Vec<BootCodeInstruction>> {
    lines_without_endings(s)
        .zip(1..)
        .map(|(line, line_idx)| {
            (|| -> anyhow::Result<_> {
                let (raw_operation, raw_argument) = line
                    .splitn(2, ' ')
                    .collect_tuple()
                    .context("expected a space dividing ")?;
                Ok(BootCodeInstruction {
                    operation: match raw_operation {
                        "acc" => BootCodeOperation::Accumulate,
                        "jmp" => BootCodeOperation::Jump,
                        "nop" => BootCodeOperation::NoOp,
                        _ => bail!("invalid operation {:?}", raw_operation),
                    },
                    argument: {
                        raw_argument
                            .strip_prefix("+")
                            .unwrap_or(raw_argument)
                            .parse::<i16>()
                            .context("argument is outside i16 range")?
                    },
                })
            })()
            .with_context(|| anyhow!("failed to parse line {}", line_idx))
        })
        .collect::<Result<Vec<_>, _>>()
}

fn part_1(s: &str) -> anyhow::Result<i32> {
    let instructions = parse_instructions(s)?;
    let mut emulator = BootCodeEmulator::zeroed();
    let mut previously_seen_inst_counters = HashSet::new();
    while previously_seen_inst_counters.insert(emulator.instruction_counter) {
        emulator.execute_single_instruction(&instructions)?;
    }
    Ok(emulator.accumulator)
}

#[test]
fn d08_p1_answer() {
    assert_eq!(part_1(INPUT).unwrap(), 1801);
}

fn part_2(s: &str) -> anyhow::Result<i32> {
    let mut instructions = parse_instructions(s)?;
    let changes_with_interesting_results = (0..instructions.len())
        .filter_map(|change_idx| {
            let original = instructions[change_idx].operation;
            let changed = match original {
                BootCodeOperation::Accumulate => return None,
                BootCodeOperation::NoOp => BootCodeOperation::Jump,
                BootCodeOperation::Jump => BootCodeOperation::NoOp,
            };

            instructions[change_idx].operation = changed;
            let mut previously_seen_inst_counters = HashSet::new();
            let mut emulator = BootCodeEmulator::zeroed();
            let filtered = loop {
                let instruction_counter = emulator.instruction_counter;
                if instruction_counter == instructions.len() {
                    break Some(Ok((change_idx, emulator.accumulator)));
                }
                if !previously_seen_inst_counters.insert(instruction_counter) {
                    break None;
                }
                if let Err(e) = emulator
                    .execute_single_instruction(&instructions)
                    .with_context(|| {
                        anyhow!("replacing instruction {} yielded an error", change_idx)
                    })
                {
                    break Some(Err(e));
                }
            };
            instructions[change_idx].operation = original;
            filtered
        })
        .collect::<Vec<anyhow::Result<_>>>();

    Ok(changes_with_interesting_results
        .first()
        .unwrap()
        .as_ref()
        .unwrap()
        .1)
}

#[test]
fn d08_p2_sample() {
    assert_eq!(part_2(SAMPLE).unwrap(), 8);
}

#[test]
fn d08_p2_answer() {
    assert_eq!(part_2(INPUT).unwrap(), 2060);
}

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

fn part_1(s: &str) -> anyhow::Result<i32> {
    let instructions = lines_without_endings(s)
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
        .collect::<Result<Vec<_>, _>>()?;
    let mut emulator_inst_counter = 0;
    let mut emulator_accumulator = 0i32;
    let mut previously_seen_inst_counters = HashSet::new();

    while !previously_seen_inst_counters.contains(&emulator_inst_counter) {
        previously_seen_inst_counters.insert(emulator_inst_counter);
        (|| {
            let instruction = instructions
                .get(emulator_inst_counter)
                .context("instruction counter out-of-bounds")?;
            let increment_inst_counter = |counter: usize| {
                counter
                    .checked_add(1)
                    .context("next instruction counter increment overflows")
            };
            match instruction.clone() {
                BootCodeInstruction {
                    operation: BootCodeOperation::NoOp,
                    argument: _,
                } => increment_inst_counter(emulator_inst_counter)
                    .map(|new_counter| emulator_inst_counter = new_counter),
                BootCodeInstruction {
                    operation: BootCodeOperation::Jump,
                    argument,
                } => if argument.is_positive() {
                    emulator_inst_counter
                        .checked_add(argument.try_into().unwrap())
                        .context("jump instruction overflowed")
                } else {
                    emulator_inst_counter
                        .checked_sub(argument.checked_neg().unwrap().try_into().unwrap())
                        .context("jump instruction underflowed")
                }
                .map(|new_counter| emulator_inst_counter = new_counter),
                BootCodeInstruction {
                    operation: BootCodeOperation::Accumulate,
                    argument,
                } => emulator_accumulator
                    .checked_add(argument.into())
                    .context("accumulator went out-of-range")
                    .map(|new_acc| emulator_accumulator = new_acc)
                    .and_then(|()| increment_inst_counter(emulator_inst_counter))
                    .map(|new_counter| emulator_inst_counter = new_counter),
            }
            .with_context(move || anyhow!("failed to execute instruction {:?}", instruction))
        })()
        .with_context(|| {
            anyhow!(
                "failed to execute next instruction; current state: {{ instruction_counter: {}, accumulator: {} }}",
                emulator_inst_counter,
                emulator_accumulator,
            )
        })?;
    }

    Ok(emulator_accumulator)
}

#[test]
fn d08_p1_answer() {
    assert_eq!(part_1(include_str!("d08.txt")).unwrap(), 1801);
}

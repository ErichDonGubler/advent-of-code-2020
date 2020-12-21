use {
    crate::parsing::lines_without_endings,
    anyhow::{anyhow, ensure, Context},
    itertools::Itertools,
    std::str::FromStr,
};

#[test]
fn p1_sample() -> anyhow::Result<()> {
    let sample = "\
939
7,13,x,x,59,x,31,19
";
    let calc = Part1Calculation::new(
        &sample
            .parse::<Part1Data>()
            .context("failed to parse sample data")?,
    );
    assert_eq!(
        calc,
        Part1Calculation {
            soonest_bus: 59,
            wait_after_initial: 5,
        }
    );
    assert_eq!(calc.answer().unwrap(), 295);
    Ok(())
}

#[test]
fn p1_answer() -> anyhow::Result<()> {
    let calc = Part1Calculation::new(
        &include_str!("d13.txt")
            .parse::<Part1Data>()
            .context("failed to parse input data")?,
    );
    assert_eq!(
        calc,
        Part1Calculation {
            soonest_bus: 607,
            wait_after_initial: 5,
        }
    );
    assert_eq!(calc.answer().unwrap(), 3035);
    Ok(())
}

#[derive(Debug, Eq, PartialEq)]
pub struct Part1Calculation {
    soonest_bus: u32,
    wait_after_initial: u32,
}

impl Part1Calculation {
    pub fn new(data: &Part1Data) -> Self {
        let Part1Data {
            initial_wait,
            bus_ids,
        } = data;
        let (soonest_bus, wait_after_initial) = bus_ids
            .iter()
            .copied()
            .map(|bus_id| (bus_id, bus_id - (initial_wait % bus_id)))
            .min_by_key(|(_bus_id, wait)| *wait)
            .unwrap();

        Self {
            soonest_bus,
            wait_after_initial,
        }
    }

    pub fn answer(&self) -> anyhow::Result<u32> {
        let &Self {
            soonest_bus,
            wait_after_initial,
        } = self;
        soonest_bus
            .checked_mul(wait_after_initial)
            .with_context(|| anyhow!("answer is unrepresentable with {:?}", self))
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Part1Data {
    initial_wait: u32,
    bus_ids: Vec<u32>,
}

impl FromStr for Part1Data {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (initial_wait, bus_ids) = {
            let (raw_initial_wait, raw_bus_ids) = lines_without_endings(s)
                .collect_tuple()
                .context("expected two lines of input")?;
            (
                raw_initial_wait
                    .parse::<u32>()
                    .with_context(|| anyhow!("failed to parse {:?} initial wait"))?,
                raw_bus_ids
                    .split(',')
                    .filter(|&s| s != "x")
                    .enumerate()
                    .map(|(raw_id_idx, raw_id)| {
                        raw_id.parse::<u32>().with_context(|| {
                            anyhow!("failed to parse raw bus ID {} ({:?})", raw_id_idx, raw_id)
                        })
                    })
                    .collect::<anyhow::Result<Vec<_>>>()?,
            )
        };

        ensure!(!bus_ids.is_empty(), "no bus IDs specified");

        Ok(Self {
            initial_wait,
            bus_ids,
        })
    }
}

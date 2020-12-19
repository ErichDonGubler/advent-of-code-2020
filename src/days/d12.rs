use {
    crate::parsing::lines_without_endings,
    anyhow::{anyhow, bail, Context},
    std::{fmt::Debug, str::FromStr},
    ux::u62,
};

#[derive(Clone, Debug)]
pub enum NavigationInstruction {
    Move {
        direction: MoveDirection,
        units: u62,
    },
    Turn(TurnInstruction),
}

impl FromStr for NavigationInstruction {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();

        let parse_unit = |s: &str| {
            s.parse::<u64>()
                .map_err(anyhow::Error::from)
                .and_then(|u| {
                    if (!(u64::MAX >> 2) & u) > 0 {
                        bail!("outside of constrained value range");
                    }
                    Ok(u62::new(u))
                })
                .with_context(|| anyhow!("unable to parse {:?} as unit for movement", s))
        };

        let parse_degrees = |s| {
            Ok(match s {
                "90" => Degrees::Ninety,
                "180" => Degrees::OneEighty,
                "270" => Degrees::TwoSeventy,
                _ => bail!("{:?} is not recognized as a valid turn degrees value"),
            })
        };

        let action_char = chars.next().context("string is empty")?;

        Ok(match action_char {
            'N' | 'E' | 'S' | 'W' | 'F' | 'B' => NavigationInstruction::Move {
                direction: match action_char {
                    'N' => MoveDirection::Cardinal(CardinalDirection::North),
                    'E' => MoveDirection::Cardinal(CardinalDirection::East),
                    'S' => MoveDirection::Cardinal(CardinalDirection::South),
                    'W' => MoveDirection::Cardinal(CardinalDirection::West),
                    'F' => MoveDirection::Forward,
                    'B' => MoveDirection::Backward,
                    _ => unreachable!(),
                },
                units: parse_unit(chars.as_str())?,
            },
            'L' | 'R' => NavigationInstruction::Turn(TurnInstruction {
                direction: match action_char {
                    'L' => TurnDirection::Left,
                    'R' => TurnDirection::Right,
                    _ => unreachable!(),
                },
                degrees: parse_degrees(chars.as_str())?,
            }),
            c => bail!("{:?} does not correspond to an instruction action", c),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CardinalDirection {
    North,
    East,
    South,
    West,
}

#[derive(Clone, Copy, Debug)]
pub enum MoveDirection {
    Cardinal(CardinalDirection),
    Forward,
    Backward,
}

#[derive(Clone, Debug)]
pub struct TurnInstruction {
    direction: TurnDirection,
    degrees: Degrees,
}

#[derive(Clone, Copy, Debug)]
pub enum TurnDirection {
    Right,
    Left,
}

#[derive(Clone, Copy, Debug)]
pub enum Degrees {
    Ninety,
    OneEighty,
    TwoSeventy,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Ship {
    position: (i64, i64),
    orientation: CardinalDirection,
}

impl Ship {
    fn new() -> Self {
        Self {
            position: (0, 0),
            orientation: CardinalDirection::East,
        }
    }

    pub fn navigate(&mut self, instruction: NavigationInstruction) -> anyhow::Result<()> {
        let Self {
            position,
            orientation,
        } = self;

        match instruction {
            NavigationInstruction::Turn(inst) => *orientation = inst.turn(*orientation).unwrap(),
            NavigationInstruction::Move { units, direction } => {
                *position = translate_pos(
                    *position,
                    units,
                    match direction {
                        MoveDirection::Forward => *orientation,
                        MoveDirection::Backward => orientation.reverse().unwrap(),
                        MoveDirection::Cardinal(dir) => dir,
                    },
                )?
            }
        };
        Ok(())
    }

    pub fn manhattan_distance_from_origin(&self) -> u64 {
        manhattan_distance(self.position)
    }

    pub fn position(&self) -> ((EastWest, u64), (NorthSouth, u64)) {
        convert_position(self.position)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NorthSouth {
    North,
    South,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EastWest {
    East,
    West,
}

#[test]
fn p1_sample() -> anyhow::Result<()> {
    let ship = navigate(
        Ship::new(),
        parse_navigation_instructions(SAMPLE)?.into_iter(),
        Some(&[
            Ship {
                position: (10, 0),
                orientation: CardinalDirection::East,
            },
            Ship {
                position: (10, 3),
                orientation: CardinalDirection::East,
            },
            Ship {
                position: (17, 3),
                orientation: CardinalDirection::East,
            },
            Ship {
                position: (17, 3),
                orientation: CardinalDirection::South,
            },
            Ship {
                position: (17, -8),
                orientation: CardinalDirection::South,
            },
        ]),
    )?;

    assert_eq!(
        ship.position(),
        ((EastWest::East, 17), (NorthSouth::South, 8))
    );
    assert_eq!(ship.manhattan_distance_from_origin(), 25);
    Ok(())
}

fn parse_navigation_instructions(s: &str) -> anyhow::Result<Vec<NavigationInstruction>> {
    lines_without_endings(s)
        .enumerate()
        .map(|(line_idx, line)| {
            line.parse()
                .with_context(|| anyhow!("failed to parse line {}", line_idx))
        })
        .collect()
}

fn abs_unsigned(x: i64) -> u64 {
    x.checked_abs().map(|i| i as u64).unwrap_or(x as u64)
}

#[test]
fn p1_answer() -> anyhow::Result<()> {
    let ship = navigate(
        Ship::new(),
        parse_navigation_instructions(INPUT)?.into_iter(),
        None,
    )?;

    assert_eq!(
        ship.position(),
        ((EastWest::East, 1253), (NorthSouth::South, 1044))
    );
    assert_eq!(ship.manhattan_distance_from_origin(), 2297);
    Ok(())
}

#[track_caller]
fn navigate<T>(
    mut navigatable: T,
    instructions: impl IntoIterator<Item = NavigationInstruction>,
    expected_steps_states: Option<&[T]>,
) -> anyhow::Result<T>
where
    T: Debug + Eq + Navigate,
{
    instructions
        .into_iter()
        .enumerate()
        .try_for_each(|(inst_idx, inst)| -> anyhow::Result<()> {
            navigatable
                .navigate(inst)
                .with_context(|| anyhow!("failed to execute navigation instruction {}", inst_idx))?;
            if let Some(expected_state) = expected_steps_states.map(|ss| {
                ss.get(inst_idx).with_context(|| anyhow!(
                        "test error: navigation instruction {} does not have a corresponding expected state",
                        inst_idx,
                ))
            }).transpose()? {
                assert_eq!(&navigatable, expected_state);
            }
            Ok(())
        })?;
    Ok(navigatable)
}

#[test]
fn p2_sample() -> anyhow::Result<()> {
    let navigation_system = navigate(
        NavigationSystem::new(),
        parse_navigation_instructions(SAMPLE)?,
        Some(&[
            NavigationSystem {
                ship_position: (100, 10),
                waypoint: (10, 1),
            },
            NavigationSystem {
                ship_position: (100, 10),
                waypoint: (10, 4),
            },
            NavigationSystem {
                ship_position: (170, 38),
                waypoint: (10, 4),
            },
            NavigationSystem {
                ship_position: (170, 38),
                waypoint: (4, -10),
            },
            NavigationSystem {
                ship_position: (214, -72),
                waypoint: (4, -10),
            },
        ]),
    )?;
    assert_eq!(
        navigation_system.position(),
        ((EastWest::East, 214), (NorthSouth::South, 72)),
    );
    assert_eq!(
        navigation_system.waypoint(),
        ((EastWest::East, 4), (NorthSouth::South, 10)),
    );
    assert_eq!(navigation_system.manhattan_distance_from_origin(), 286);
    Ok(())
}

#[derive(Debug, Eq, PartialEq)]
pub struct NavigationSystem {
    ship_position: (i64, i64),
    waypoint: (i64, i64),
}

impl NavigationSystem {
    fn navigate(&mut self, instruction: NavigationInstruction) -> anyhow::Result<()> {
        let Self {
            ship_position,
            waypoint,
        } = self;

        match instruction {
            NavigationInstruction::Move { units, direction } => match direction {
                MoveDirection::Cardinal(dir) => *waypoint = translate_pos(*waypoint, units, dir)?,
                MoveDirection::Forward | MoveDirection::Backward => {
                    let (waypoint_x, waypoint_y) = if matches!(direction, MoveDirection::Backward) {
                        let (x, y) = *waypoint;
                        (|| {
                            Some((x.checked_neg()?, y.checked_neg()?))
                        })()
                        .with_context(|| anyhow!("inverted waypoint ({}, {}) is unrepresentable with `i64` dimensions", x, y))?
                    } else {
                        *waypoint
                    };
                    let &mut (x, y) = ship_position;
                    let add_dim = |dim: i64, add: i64| {
                        dim.checked_add(add.checked_mul(u64::from(units) as i64)?)
                    };
                    *ship_position = (|| { Some((add_dim(x, waypoint_x)?, add_dim(y, waypoint_y)?)) })()
                        .with_context(
                            || anyhow!(
                                "moving {} times {:?} with waypoint {:?} is unrepresentable with `i64` dimensions",
                                units,
                                direction,
                                waypoint,
                            )
                        )?
                }
            },
            NavigationInstruction::Turn(inst) => {
                *waypoint = inst.turn(*waypoint).with_context(|| {
                    let (x, y) = waypoint;
                    let TurnInstruction { direction, degrees } = inst;
                    anyhow!(
                        "waypoint ({}, {}) turned {:?} by {:?} degrees",
                        x,
                        y,
                        direction,
                        degrees,
                    )
                })?
            }
        };
        Ok(())
    }

    fn new() -> Self {
        Self {
            ship_position: (0, 0),
            waypoint: (10, 1),
        }
    }

    pub fn position(&self) -> ((EastWest, u64), (NorthSouth, u64)) {
        convert_position(self.ship_position)
    }

    pub fn waypoint(&self) -> ((EastWest, u64), (NorthSouth, u64)) {
        convert_position(self.waypoint)
    }

    pub fn manhattan_distance_from_origin(&self) -> u64 {
        manhattan_distance(self.ship_position)
    }
}

fn translate_pos(
    position: (i64, i64),
    units: u62,
    direction: CardinalDirection,
) -> anyhow::Result<(i64, i64)> {
    let units = u64::from(units) as i64;

    // positive first number is east, positive second number is north
    let (x, y) = position;
    (|| {
        Some(match direction {
            CardinalDirection::North => (x, y.checked_add(units)?),
            CardinalDirection::East => (x.checked_add(units)?, y),
            CardinalDirection::South => (x, y.checked_sub(units)?),
            CardinalDirection::West => (x.checked_sub(units)?, y),
        })
    })().with_context(
    || anyhow!(
        "cannot move {} units {:?} with position {:?}; new position is not representable with i64 coordinates",
        units,
        direction,
        position,
    )
    )
}

pub trait Navigate {
    fn navigate(&mut self, instruction: NavigationInstruction) -> anyhow::Result<()>;
}

impl<T> Navigate for &'_ mut T
where
    T: Navigate,
{
    fn navigate(&mut self, instruction: NavigationInstruction) -> anyhow::Result<()> {
        T::navigate(self, instruction)
    }
}

impl Navigate for Ship {
    fn navigate(&mut self, instruction: NavigationInstruction) -> anyhow::Result<()> {
        self.navigate(instruction)
    }
}

impl Navigate for NavigationSystem {
    fn navigate(&mut self, instruction: NavigationInstruction) -> anyhow::Result<()> {
        self.navigate(instruction)
    }
}

const SAMPLE: &str = "\
F10
N3
F7
R90
F11
";

pub trait Turn
where
    Self: Sized,
{
    fn single_turn_left(self) -> Option<Self>;
    fn single_turn_right(self) -> Option<Self>;
    fn reverse(self) -> Option<Self>;
}

impl Turn for CardinalDirection {
    fn single_turn_left(self) -> Option<Self> {
        Some(match self {
            Self::North => Self::West,
            Self::East => Self::North,
            Self::South => Self::East,
            Self::West => Self::South,
        })
    }

    fn single_turn_right(self) -> Option<Self> {
        Some(match self {
            Self::North => Self::East,
            Self::East => Self::South,
            Self::South => Self::West,
            Self::West => Self::North,
        })
    }

    fn reverse(self) -> Option<Self> {
        Some(match self {
            Self::North => Self::South,
            Self::East => Self::West,
            Self::South => Self::North,
            Self::West => Self::East,
        })
    }
}

impl TurnInstruction {
    fn turn<T>(&self, t: T) -> Option<T>
    where
        T: Turn,
    {
        let TurnInstruction { direction, degrees } = self;

        match (degrees, direction) {
            (Degrees::Ninety, TurnDirection::Left)
            | (Degrees::TwoSeventy, TurnDirection::Right) => t.single_turn_left(),
            (Degrees::Ninety, TurnDirection::Right)
            | (Degrees::TwoSeventy, TurnDirection::Left) => t.single_turn_right(),
            (Degrees::OneEighty, _) => t.reverse(),
        }
    }
}

impl Turn for (i64, i64) {
    fn single_turn_left(self) -> Option<Self> {
        let (x, y) = self;
        Some((y.checked_neg()?, x))
    }

    fn single_turn_right(self) -> Option<Self> {
        let (x, y) = self;
        Some((y, x.checked_neg()?))
    }

    fn reverse(self) -> Option<Self> {
        let (x, y) = self;
        Some((x.checked_neg()?, y.checked_neg()?))
    }
}

#[test]
fn p2_answer() -> anyhow::Result<()> {
    let navigation_system = navigate(
        NavigationSystem::new(),
        parse_navigation_instructions(INPUT)?,
        None,
    )?;
    assert_eq!(
        navigation_system.position(),
        ((EastWest::West, 3847), (NorthSouth::South, 86137)),
    );
    assert_eq!(
        navigation_system.waypoint(),
        ((EastWest::West, 39), (NorthSouth::North, 39)),
    );
    assert_eq!(navigation_system.manhattan_distance_from_origin(), 89984);
    Ok(())
}

const INPUT: &str = include_str!("d12.txt");

fn convert_position(coords: (i64, i64)) -> ((EastWest, u64), (NorthSouth, u64)) {
    let (x, y) = coords;
    (
        (
            if x.is_negative() {
                EastWest::West
            } else {
                EastWest::East
            },
            abs_unsigned(x),
        ),
        (
            if y.is_negative() {
                NorthSouth::South
            } else {
                NorthSouth::North
            },
            abs_unsigned(y),
        ),
    )
}

fn manhattan_distance((x, y): (i64, i64)) -> u64 {
    abs_unsigned(x) + abs_unsigned(y)
}

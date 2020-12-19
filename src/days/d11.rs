use {
    crate::parsing::lines_without_endings,
    anyhow::{anyhow, ensure, Context},
    array_iterator::ArrayIterator,
    arrayvec::ArrayVec,
    std::{
        cmp::min,
        fmt::{self, Display, Formatter},
        iter::successors,
        str::FromStr,
    },
};

#[test]
fn p1_sample() {
    let mut simulation =
        WaitingAreaSeatingSimulation::new(SAMPLE.parse::<WaitingAreaMap>().unwrap());

    check_simulation_steps_and_exhaustion(
        &mut simulation,
        &mut Part1OccupantBehavior,
        &[
            "\
#.##.##.##
#######.##
#.#.#..#..
####.##.##
#.##.##.##
#.#####.##
..#.#.....
##########
#.######.#
#.#####.##
",
            "\
#.LL.L#.##
#LLLLLL.L#
L.L.L..L..
#LLL.LL.L#
#.LL.LL.LL
#.LLLL#.##
..L.L.....
#LLLLLLLL#
#.LLLLLL.L
#.#LLLL.##
",
            "\
#.##.L#.##
#L###LL.L#
L.#.#..#..
#L##.##.L#
#.##.LL.LL
#.###L#.##
..#.#.....
#L######L#
#.LL###L.L
#.#L###.##
",
            "\
#.#L.L#.##
#LLL#LL.L#
L.L.L..#..
#LLL.##.L#
#.LL.LL.LL
#.LL#L#.##
..L.L.....
#L#LLLL#L#
#.LLLLLL.L
#.#L#L#.##
",
            "\
#.#L.L#.##
#LLL#LL.L#
L.#.L..#..
#L##.##.L#
#.#L.LL.LL
#.#L#L#.##
..L.L.....
#L#L##L#L#
#.LLLLLL.L
#.#L#L#.##
",
        ],
    )
    .unwrap();

    assert_eq!(
        simulation
            .current_state()
            .tiles
            .iter()
            .filter(|tile| matches!(tile, WaitingAreaMapTile::Seat { occupied: true }))
            .count(),
        37,
    );
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WaitingAreaMapTile {
    Seat { occupied: bool },
    Floor,
}

impl Display for WaitingAreaMapTile {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_char())
    }
}

impl WaitingAreaMapTile {
    const OCCUPIED_SEAT: char = '#';
    const UNOCCUPIED_SEAT: char = 'L';
    const FLOOR: char = '.';

    pub fn from_char(c: char) -> Option<Self> {
        Some(match c {
            Self::UNOCCUPIED_SEAT => WaitingAreaMapTile::Seat { occupied: false },
            Self::OCCUPIED_SEAT => WaitingAreaMapTile::Seat { occupied: true },
            Self::FLOOR => WaitingAreaMapTile::Floor,
            _ => return None,
        })
    }

    pub fn as_char(self) -> char {
        match self {
            Self::Seat { occupied } => {
                if occupied {
                    Self::OCCUPIED_SEAT
                } else {
                    Self::UNOCCUPIED_SEAT
                }
            }
            Self::Floor => Self::FLOOR,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WaitingAreaMap {
    tiles: Vec<WaitingAreaMapTile>,
    map_width: usize,
}

impl WaitingAreaMap {
    fn tiles(&self) -> &[WaitingAreaMapTile] {
        &self.tiles
    }

    fn get_adjacent_tiles(&self, offset: usize) -> impl Iterator<Item = WaitingAreaMapTile> + '_ {
        let mut areas = ArrayVec::<[WaitingAreaMapTile; 9]>::new();

        let &Self {
            map_width: width,
            ref tiles,
        } = self;

        let area = tiles.len();

        let gather_window_with_center_at = move |offset| {
            let WaitingAreaMapCoords { x, y } =
                Self::translate_offset_into_human_coords(offset, width);
            let offset_from_new_x = |x| {
                Self::translate_human_coords_into_offset(WaitingAreaMapCoords { x, y }, width, area)
            };
            let start = x.saturating_sub(1);
            let end = min(x.saturating_add(1), width - 1);
            offset_from_new_x(start)..=offset_from_new_x(end)
        };

        if let Some(top_adjacent_area) = offset
            .checked_sub(width)
            .map(|o| gather_window_with_center_at(o))
        {
            areas.extend(tiles[top_adjacent_area].iter().copied());
        }

        gather_window_with_center_at(offset)
            .filter(|&o| o != offset)
            .for_each(|o| {
                areas.push(tiles[o]);
            });

        if let Some(bottom_adjacent_area) = offset
            .checked_add(width)
            .filter(|&o| o < tiles.len())
            .map(|o| gather_window_with_center_at(o))
        {
            areas.extend(tiles[bottom_adjacent_area].iter().copied());
        }

        areas.into_iter()
    }

    fn get_visible_seats(&self, offset: usize) -> impl Iterator<Item = bool> + '_ {
        let &Self {
            map_width,
            ref tiles,
        } = self;

        let area = tiles.len();
        assert!(offset < area);

        #[derive(Clone, Copy, Debug)]
        enum OffsetOp {
            PlusOne,
            NegOne,
        }

        impl OffsetOp {
            fn apply(self, pos: usize) -> Option<usize> {
                match self {
                    Self::PlusOne => pos.checked_add(1),
                    Self::NegOne => pos.checked_sub(1),
                }
            }
        }

        let WaitingAreaMapCoords { x, y } =
            Self::translate_offset_into_human_coords(offset, map_width);
        let map_height = area / map_width;

        ArrayIterator::new([
            (None, Some(OffsetOp::PlusOne)),                    // up
            (None, Some(OffsetOp::NegOne)),                     // down
            (Some(OffsetOp::PlusOne), None),                    // right
            (Some(OffsetOp::NegOne), None),                     // left
            (Some(OffsetOp::PlusOne), Some(OffsetOp::PlusOne)), // up-right
            (Some(OffsetOp::PlusOne), Some(OffsetOp::NegOne)),  // down-right
            (Some(OffsetOp::NegOne), Some(OffsetOp::PlusOne)),  // up-left
            (Some(OffsetOp::NegOne), Some(OffsetOp::NegOne)),   // down-left
        ])
        .filter_map(move |(ox, oy)| {
            successors(Some((x, y)), |&(x, y)| {
                Some((
                    ox.map(|ox| ox.apply(x).filter(|&x| x < map_width))
                        .unwrap_or(Some(x))?,
                    oy.map(|oy| oy.apply(y).filter(|&y| y < map_height))
                        .unwrap_or(Some(y))?,
                ))
            })
            .skip(1)
            .find_map(|(x, y)| {
                let offset = Self::translate_human_coords_into_offset(
                    WaitingAreaMapCoords { x, y },
                    map_width,
                    area,
                );
                match tiles[offset] {
                    WaitingAreaMapTile::Seat { occupied } => Some(occupied),
                    _ => None,
                }
            })
        })
    }

    fn translate_offset_into_human_coords(offset: usize, width: usize) -> WaitingAreaMapCoords {
        WaitingAreaMapCoords {
            x: offset % width,
            y: offset / width,
        }
    }

    fn translate_human_coords_into_offset(
        coords: WaitingAreaMapCoords,
        width: usize,
        area: usize,
    ) -> usize {
        let WaitingAreaMapCoords { x, y } = coords;
        assert!(x < width);
        let offset = y
            .checked_mul(width)
            .and_then(|offset_y| offset_y.checked_add(x))
            .filter(|&offset| offset < area);
        assert!(offset.is_some());
        offset.unwrap()
    }
}

impl Display for WaitingAreaMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let &Self {
            ref tiles,
            map_width,
        } = self;

        tiles.chunks(map_width).try_for_each(|chunk| {
            chunk
                .iter()
                .copied()
                .try_for_each(|tile| write!(f, "{}", tile))?;
            writeln!(f)
        })
    }
}

impl FromStr for WaitingAreaMap {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut map_string_lines = lines_without_endings(s).peekable();
        let expected_row_width = map_string_lines
            .peek()
            .context("first line is empty")?
            .chars()
            .count();
        let tiles = map_string_lines
            .zip(1..)
            .flat_map(|(line, line_idx)| {
                line.char_indices()
                    .enumerate()
                    .map(|(count, (idx, c))| {
                        ensure!(
                            count < expected_row_width,
                            "line's character count exceeds the one found on first line ({}), \
                            producing incomplete map dimensions",
                            expected_row_width,
                        );

                        Ok(WaitingAreaMapTile::from_char(c).with_context(|| {
                            anyhow!(
                                "unrecognized value {:?} for character {} (byte index {})",
                                c,
                                count,
                                idx,
                            )
                        })?)
                    })
                    .map(move |res| {
                        res.with_context(move || anyhow!("failed to parse line {}", line_idx))
                    })
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        Ok(Self {
            tiles,
            map_width: expected_row_width,
        })
    }
}

#[derive(Clone, Debug)]
struct WaitingAreaSeatingSimulation {
    map_copies: [WaitingAreaMap; 2],
    curr_map_idx: usize,
}

#[derive(Clone, Debug)]
struct WaitingAreaMapCoords {
    x: usize,
    y: usize,
}

trait WaitingAreaOccupantBehavior {
    fn would_enter_seat(&mut self, prev_map: &WaitingAreaMap, tile_idx: usize) -> bool;
    fn would_leave_seat(&mut self, prev_map: &WaitingAreaMap, tile_idx: usize) -> bool;
}

impl<'a, F> WaitingAreaOccupantBehavior for &'a mut F
where
    F: WaitingAreaOccupantBehavior,
{
    fn would_enter_seat(&mut self, prev_map: &WaitingAreaMap, tile_idx: usize) -> bool {
        F::would_enter_seat(self, prev_map, tile_idx)
    }

    fn would_leave_seat(&mut self, prev_map: &WaitingAreaMap, tile_idx: usize) -> bool {
        F::would_leave_seat(self, prev_map, tile_idx)
    }
}

impl WaitingAreaSeatingSimulation {
    pub fn new(starting_map: WaitingAreaMap) -> Self {
        Self {
            map_copies: [starting_map.clone(), starting_map],
            curr_map_idx: 0,
        }
    }

    fn next_step<B>(&mut self, mut occupant_behavior: B) -> Option<&WaitingAreaMap>
    where
        B: WaitingAreaOccupantBehavior,
    {
        let &mut Self {
            curr_map_idx,
            map_copies: [ref mut first_map, ref mut second_map],
        } = self;

        let (prev_map, (next_map_idx, next_map)) = match curr_map_idx {
            0 => (&first_map, (1, second_map)),
            1 => (&second_map, (0, first_map)),
            _ => unreachable!(),
        };

        let mut changed = false;
        prev_map
            .tiles
            .iter()
            .zip(next_map.tiles.iter_mut())
            .enumerate()
            .for_each(|(idx, (&prev_tile, next_tile))| {
                *next_tile = match prev_tile {
                    WaitingAreaMapTile::Seat { occupied: false }
                        if occupant_behavior.would_enter_seat(prev_map, idx) =>
                    {
                        changed = true;
                        WaitingAreaMapTile::Seat { occupied: true }
                    }
                    WaitingAreaMapTile::Seat { occupied: true }
                        if occupant_behavior.would_leave_seat(prev_map, idx) =>
                    {
                        changed = true;
                        WaitingAreaMapTile::Seat { occupied: false }
                    }
                    _ => prev_tile,
                };
            });

        if changed {
            self.curr_map_idx = next_map_idx;
            Some(self.current_state())
        } else {
            None
        }
    }

    pub fn current_state(&self) -> &WaitingAreaMap {
        let &Self {
            curr_map_idx,
            ref map_copies,
        } = self;
        &map_copies[curr_map_idx]
    }
}

fn num_seats_with_behavior<B>(mut b: B) -> anyhow::Result<usize>
where
    B: WaitingAreaOccupantBehavior,
{
    let mut simulation = WaitingAreaSeatingSimulation::new(include_str!("d11.txt").parse()?);
    while simulation.next_step(&mut b).is_some() {}
    Ok(simulation
        .current_state()
        .tiles()
        .iter()
        .filter(|tile| matches!(tile, WaitingAreaMapTile::Seat { occupied: true }))
        .count())
}

#[derive(Clone, Debug)]
struct Part1OccupantBehavior;

impl WaitingAreaOccupantBehavior for Part1OccupantBehavior {
    fn would_enter_seat(&mut self, prev_map: &WaitingAreaMap, tile_idx: usize) -> bool {
        prev_map
            .get_adjacent_tiles(tile_idx)
            .all(|tile| !matches!(tile, WaitingAreaMapTile::Seat { occupied: true }))
    }

    fn would_leave_seat(&mut self, prev_map: &WaitingAreaMap, tile_idx: usize) -> bool {
        prev_map
            .get_adjacent_tiles(tile_idx)
            .find({
                let mut count = 0;
                move |tile| {
                    if matches!(tile, WaitingAreaMapTile::Seat { occupied: true }) {
                        count += 1;
                    }
                    count == 4
                }
            })
            .is_some()
    }
}

#[test]
fn p1_answer() {
    assert_eq!(
        num_seats_with_behavior(Part1OccupantBehavior).unwrap(),
        2386
    );
}

#[derive(Clone, Debug)]
struct Part2OccupantBehavior;

impl WaitingAreaOccupantBehavior for Part2OccupantBehavior {
    fn would_enter_seat(&mut self, prev_map: &WaitingAreaMap, tile_idx: usize) -> bool {
        prev_map
            .get_visible_seats(tile_idx)
            .all(|occupied| !occupied)
    }

    fn would_leave_seat(&mut self, prev_map: &WaitingAreaMap, tile_idx: usize) -> bool {
        prev_map
            .get_visible_seats(tile_idx)
            .filter(|&occupied| occupied)
            .count()
            >= 5
    }
}

const SAMPLE: &str = "\
L.LL.LL.LL
LLLLLLL.LL
L.L.L..L..
LLLL.LL.LL
L.LL.LL.LL
L.LLLLL.LL
..L.L.....
LLLLLLLLLL
L.LLLLLL.L
L.LLLLL.LL
";

#[test]
fn p2_sample() {
    let find_top_left_empty_seat = |map: &WaitingAreaMap| {
        map.tiles()
            .iter()
            .enumerate()
            .find_map(|(idx, tile)| match tile {
                WaitingAreaMapTile::Seat { occupied: false } => Some(idx),
                _ => None,
            })
    };
    {
        let map = "\
.......#.
...#.....
.#.......
.........
..#L....#
....#....
.........
#........
...#.....
"
        .parse::<WaitingAreaMap>()
        .unwrap();

        assert_eq!(
            map.get_visible_seats(find_top_left_empty_seat(&map).unwrap())
                .filter(|&occupied| occupied)
                .count(),
            8,
        );
    }

    {
        let map = "\
.............
.L.L.#.#.#.#.
.............
"
        .parse::<WaitingAreaMap>()
        .unwrap();

        assert_eq!(
            map.get_visible_seats(find_top_left_empty_seat(&map).unwrap())
                .map(|occupied| occupied)
                .collect::<Vec<_>>(),
            &[false],
        );
    }

    {
        let map = "\
.##.##.
#.#.#.#
##...##
...L...
##...##
#.#.#.#
.##.##.
"
        .parse::<WaitingAreaMap>()
        .unwrap();

        assert_eq!(
            map.get_visible_seats(find_top_left_empty_seat(&map).unwrap())
                .count(),
            0
        );
    }

    {
        let mut simulation = WaitingAreaSeatingSimulation::new(SAMPLE.parse().unwrap());
        check_simulation_steps_and_exhaustion(
            &mut simulation,
            Part2OccupantBehavior,
            &[
                "\
#.##.##.##
#######.##
#.#.#..#..
####.##.##
#.##.##.##
#.#####.##
..#.#.....
##########
#.######.#
#.#####.##
",
                "\
#.LL.LL.L#
#LLLLLL.LL
L.L.L..L..
LLLL.LL.LL
L.LL.LL.LL
L.LLLLL.LL
..L.L.....
LLLLLLLLL#
#.LLLLLL.L
#.LLLLL.L#
",
                "\
#.L#.##.L#
#L#####.LL
L.#.#..#..
##L#.##.##
#.##.#L.##
#.#####.#L
..#.#.....
LLL####LL#
#.L#####.L
#.L####.L#
",
                "\
#.L#.L#.L#
#LLLLLL.LL
L.L.L..#..
##LL.LL.L#
L.LL.LL.L#
#.LLLLL.LL
..L.L.....
LLLLLLLLL#
#.LLLLL#.L
#.L#LL#.L#
",
                "\
#.L#.L#.L#
#LLLLLL.LL
L.L.L..#..
##L#.#L.L#
L.L#.#L.L#
#.L####.LL
..#.#.....
LLL###LLL#
#.LLLLL#.L
#.L#LL#.L#
",
                "\
#.L#.L#.L#
#LLLLLL.LL
L.L.L..#..
##L#.#L.L#
L.L#.LL.L#
#.LLLL#.LL
..#.L.....
LLL###LLL#
#.LLLLL#.L
#.L#LL#.L#
",
            ],
        )
        .unwrap();
    }
}

fn check_simulation_steps_and_exhaustion<'a, B>(
    simulation: &'a mut WaitingAreaSeatingSimulation,
    mut occupant_behavior: B,
    steps: &[&str],
) -> anyhow::Result<&'a WaitingAreaMap>
where
    B: WaitingAreaOccupantBehavior,
{
    steps.iter().enumerate().try_for_each(|(step_idx, step)| {
        (|| -> anyhow::Result<_> {
            let expected_next_step_map = step
                .parse::<WaitingAreaMap>()
                .context("failed to parse expected map of step")?;
            let actual_next_step_map = match simulation.next_step(&mut occupant_behavior) {
                Some(map) => map,
                None => simulation.current_state(),
            };
            ensure!(
                &expected_next_step_map == actual_next_step_map,
                "comparison of step map failed:\n  expected:\n{}\n  actual:\n{}",
                expected_next_step_map,
                actual_next_step_map,
            );
            Ok(())
        })()
        .with_context(|| anyhow!("step {} (0-based) of checked simulation failed", step_idx))
    })?;
    ensure!(
        simulation.next_step(occupant_behavior).is_none(),
        "waiting area simulation activity was not exhausted"
    );
    Ok(simulation.current_state())
}

#[test]
fn p2_answer() {
    assert_eq!(
        num_seats_with_behavior(Part2OccupantBehavior).unwrap(),
        2091,
    );
}

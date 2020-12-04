use {
    advent_of_code_2020::parsing::lines_without_endings,
    anyhow::{anyhow, ensure, Context},
    itertools::Itertools,
    std::iter::once,
};

const SAMPLE: &str = "\
..##.......
#...#...#..
.#....#..#.
..#.#...#.#
.#...##..#.
..#.##.....
.#.#.#....#
.#........#
#.##...#...
#...##....#
.#..#...#.#
";

const INPUT: &str = include_str!("d03.txt");

#[derive(Debug, Clone, Copy)]
enum TobogganAreaTile {
    OpenSquare,
    Tree,
}

#[derive(Debug, Clone)]
struct TobogganArea {
    definition_width: usize,
    tiles: Vec<TobogganAreaTile>,
}

#[derive(Debug, Clone)]
struct TobogganSlope {
    horiz_step: usize,
}

impl TobogganArea {
    fn new(s: &str) -> anyhow::Result<Self> {
        let mut lines = lines_without_endings(s);
        let (first_line,) = lines.by_ref().take(1).collect_tuple().unwrap();

        ensure!(
            !first_line.is_empty(),
            "first line was empty; \
            need at least one character per line for a toboggan area definition",
        );
        let expected_line_len = first_line.len();

        let tiles = once(first_line)
            .chain(lines)
            .zip(1..)
            .flat_map(|(l, one_based_line_idx)| {
                let line_err_ctx = move || anyhow!("failed to parse line {}", one_based_line_idx);
                if l.len() != expected_line_len {
                    Some(
                        Err(anyhow!(
                            "expected line to be of len {}, but it was of len {}",
                            expected_line_len,
                            l.len(),
                        ))
                        .with_context(line_err_ctx),
                    )
                } else {
                    None
                }
                .into_iter()
                .chain(l.char_indices().zip(1..).take(expected_line_len).map(
                    move |((zero_based_char_byte_idx, c), one_based_col)| {
                        const OPEN_SQUARE: char = '.';
                        const TREE: char = '#';
                        match c {
                            OPEN_SQUARE => Ok(TobogganAreaTile::OpenSquare),
                            TREE => Ok(TobogganAreaTile::Tree),
                            c => Err(anyhow!(
                                "expected one of {:?}, got {:?} at column {} (byte {})",
                                [OPEN_SQUARE, TREE],
                                c,
                                one_based_col,
                                zero_based_char_byte_idx,
                            ))
                            .with_context(line_err_ctx),
                        }
                    },
                ))
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            definition_width: expected_line_len,
            tiles,
        })
    }

    fn iter_slope_tiles(
        &self,
        slope: TobogganSlope,
    ) -> anyhow::Result<impl Iterator<Item = TobogganAreaTile> + '_> {
        let &Self {
            ref tiles,
            definition_width,
        } = self;
        let TobogganSlope { horiz_step } = slope;
        let logical_vert_step = 1;

        ensure!(
            horiz_step < definition_width,
            "toboggan area width ({}) is not greater than horizontal step ({})",
            definition_width,
            horiz_step,
        );
        {
            let height = tiles.len() / definition_width;
            ensure!(
                logical_vert_step < height,
                "toboggan area height ({}) is not greater than vertical step ({})",
                height,
                logical_vert_step,
            );
        }

        let mut current_pos = 0;
        let mut current_logical_vert_pos = 0;
        Ok(std::iter::from_fn(move || {
            // NOTE(erichdongubler): I'm actually not sure if it'd be easier/faster to just
            // manipulate logical coordinates that then get translated into a single new offset,
            // instead of trying to fancily recalculate the offset like we are doing here.
            let new_logical_vert_pos = current_logical_vert_pos + logical_vert_step;
            let new_pos = {
                let horiz_adjusted_pos = horiz_step.checked_add(current_pos)?;
                let already_moved_one_logical_vert_step =
                    horiz_adjusted_pos / definition_width != current_logical_vert_pos;
                let actual_vert_step = definition_width
                    * (logical_vert_step
                        - if already_moved_one_logical_vert_step {
                            1
                        } else {
                            0
                        });
                horiz_adjusted_pos.checked_add(actual_vert_step)?
            };
            let tile = *tiles.get(new_pos)?;

            current_pos = new_pos;
            current_logical_vert_pos = new_logical_vert_pos;

            Some(tile)
        }))
    }
}

fn part_1(s: &str) -> anyhow::Result<usize> {
    let area = TobogganArea::new(s).context("failed to parse toboggan area")?;
    let tiles = area.iter_slope_tiles(TobogganSlope { horiz_step: 3 })?;
    let trees_touched = tiles
        .filter(|t| matches!(t, TobogganAreaTile::Tree))
        .count();
    Ok(trees_touched)
}

#[test]
fn d03_p1_sample() {
    // TODO: Could make this more robust with a visualizastion like in the exercise spec.
    assert_eq!(part_1(SAMPLE).unwrap(), 7);
}

#[test]
fn d03_p1_answer() {
    assert_eq!(part_1(INPUT).unwrap(), 184);
}

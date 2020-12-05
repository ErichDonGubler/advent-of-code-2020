use {
    advent_of_code_2020::parsing::lines_without_endings,
    anyhow::{bail, ensure, Context},
    std::{ops::Sub, str::FromStr},
    ux::{i11, u10, u3, u7},
};

const INPUT: &str = include_str!("d05.txt");

#[test]
fn d05_p1_sample() {
    #[track_caller]
    fn test_seat_id(s: &str, (row, seat): (u8, u8), expected_seat_id: u16) {
        let seat_id = s.parse().unwrap();
        assert_eq!(SeatId(u10::new(expected_seat_id)), seat_id);
        assert_eq!(seat_id.row_and_seat(), (u7::new(row), u3::new(seat)));
    }
    test_seat_id("FBFBBFFRLR", (44, 5), 357);
    test_seat_id("BFFFBBFRRR", (70, 7), 567);
    test_seat_id("FFFBBBFRRR", (14, 7), 119);
    test_seat_id("BBFFBBFRLL", (102, 4), 820);
}

#[test]
fn d05_p1_answer() {
    assert_eq!(
        lines_without_endings(INPUT)
            .map(|l| l.parse::<SeatId>().unwrap())
            .max()
            .unwrap(),
        SeatId(u10::new(806))
    );
}

#[test]
fn d05_p2_answer() {
    let mut seats = lines_without_endings(INPUT)
        .map(|l| l.parse::<SeatId>())
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    seats.sort();

    let available_seat = seats[..]
        .windows(2)
        .find_map(|window| match window {
            &[before, after] => {
                if after - before == i11::new(2) {
                    Some(before.checked_add(1).unwrap())
                } else {
                    None
                }
            }
            _ => unreachable!(),
        })
        .context("did not find a lonely empty space")
        .unwrap();

    assert_eq!(available_seat, SeatId(u10::new(562)));
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SeatId(pub u10);

impl SeatId {
    const TEN_RIGHTMOST_BITS: u16 = 0x03FF;

    pub fn row_and_seat(self) -> (u7, u3) {
        const THREE_RIGHTMOST_BITS: u16 = 0b111;

        let Self(s) = self;
        (
            u7::new(((u16::from(s) & !THREE_RIGHTMOST_BITS) >> 3) as u8),
            u3::new((u16::from(s) & THREE_RIGHTMOST_BITS) as u8),
        )
    }

    pub fn checked_add(self, addend: u16) -> Option<Self> {
        let is_within_range = |x| x & !Self::TEN_RIGHTMOST_BITS == 0;
        u16::from(self.0)
            .checked_add(addend)
            .filter(|&sum| is_within_range(sum))
            .map(|sum| SeatId(u10::new(sum)))
    }
}

impl FromStr for SeatId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ensure!(s.len() == 10, "expected 10 bytes of input, got {}", s.len(),);

        let mut seat = 0;
        const TENTH_BIT_SET: u16 = 0b10_0000_0000;
        let mut char_indices = s.chars().enumerate();
        for (count, c) in char_indices.by_ref().take(7) {
            seat |= match c {
                'F' => 0,
                'B' => TENTH_BIT_SET >> count,
                _ => bail!(
                    "expected 'F' or 'B' for character {}, but got {:?}",
                    count,
                    c,
                ),
            };
        }
        for (count, c) in char_indices.take(3) {
            seat |= match c {
                'L' => 0,
                'R' => TENTH_BIT_SET >> count,
                _ => bail!(
                    "expected 'L' or 'R' for character {}, but got {:?}",
                    count,
                    c,
                ),
            };
        }
        Ok(Self(u10::new(seat)))
    }
}

impl Sub for SeatId {
    type Output = i11;

    fn sub(self, other: Self) -> Self::Output {
        i11::new(u16::from(self.0) as i16 - u16::from(other.0) as i16)
    }
}

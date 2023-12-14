use eyre::{eyre, Result};
use itertools::Itertools;
use std::{iter::once, ops::Sub, str::FromStr};
use strum::{AsRefStr, EnumString, IntoStaticStr};
use tap::prelude::*;

const INPUT: &str = include_str!("./day14.txt");

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, EnumString, AsRefStr, IntoStaticStr,
)]
enum Rock {
    #[strum(serialize = "O")]
    Round,
    #[strum(serialize = "#")]
    Cube,
}

impl std::fmt::Display for Rock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_ref().pipe(|v: &str| v.fmt(f))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Input(Vec<Vec<Option<Rock>>>);

impl std::fmt::Display for Input {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.iter().try_for_each(|line| {
            line.iter()
                .map(|c| c.map(<&'static str>::from).unwrap_or("."))
                .chain(once("\n"))
                .try_for_each(|c| c.fmt(f))
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
enum Direction {
    North,
    West,
    South,
    East,
}

impl Direction {
    const fn all() -> [Self; 4] {
        [Self::North, Self::West, Self::South, Self::East]
    }
}

#[derive(Clone, PartialEq, Eq, Copy, PartialOrd, Ord, Hash)]
struct Position {
    column: u16,
    row: u16,
}

impl Position {
    fn neighbour(self, direction: Direction) -> Self {
        let Self { column, row } = self;
        match direction {
            Direction::North => Self {
                column,
                row: row - 1,
            },
            Direction::South => Self {
                column,
                row: row + 1,
            },
            Direction::East => Self {
                column: column + 1,
                row,
            },
            Direction::West => Self {
                column: column - 1,
                row,
            },
        }
    }
}

struct Positioned<T> {
    inner: T,
    position: Position,
}

impl Input {
    fn rows(&self) -> impl Iterator<Item = (u16, &[Option<Rock>])> + '_ {
        self.0
            .iter()
            .enumerate()
            .map(|(row, v)| (row as _, v.as_slice()))
    }
    fn all(&self) -> impl Iterator<Item = Positioned<Option<Rock>>> + '_ {
        self.rows().flat_map(move |(row_idx, row)| {
            row.iter()
                .copied()
                .enumerate()
                .map(move |(column, rock)| Positioned::<_> {
                    inner: rock,
                    position: Position {
                        row: row_idx,
                        column: column as _,
                    },
                })
        })
    }

    fn get(&self, Position { column, row }: Position) -> Result<Option<Rock>> {
        self.0
            .get(row as usize)
            .ok_or_else(|| eyre!("invalid row: {row}"))
            .and_then(|row| {
                row.get(column as usize)
                    .ok_or_else(|| eyre!("invalid column: {column}"))
                    .map(|v| v.as_ref().cloned())
            })
    }
    fn neighbour(
        &self,
        position: Position,
        direction: Direction,
    ) -> Result<Positioned<Option<Rock>>> {
        position.neighbour(direction).pipe(|position| {
            self.get(position).map(|rock| Positioned {
                inner: rock,
                position,
            })
        })
    }
    fn get_mut(&mut self, Position { column, row }: Position) -> Result<&mut Option<Rock>> {
        self.0
            .get_mut(row as usize)
            .ok_or_else(|| eyre!("invalid row: {row}"))
            .and_then(|row| {
                row.get_mut(column as usize)
                    .ok_or_else(|| eyre!("invalid column: {column}"))
            })
    }
    fn set(&mut self, position: Position, value: Option<Rock>) -> Result<Option<Rock>> {
        self.get_mut(position).map(|v| std::mem::replace(v, value))
    }

    fn swap(&mut self, first: Position, other: Position) -> Result<()> {
        self.get(first)
            .map(|value| (first, value))
            .and_then(|first| {
                self.get(other)
                    .map(|value| (other, value))
                    .map(|other| (first, other))
            })
            .and_then(|(first, other)| {
                self.set(first.0, other.1)
                    .and_then(|_| self.set(other.0, first.1))
                    .map(|_| ())
            })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Simulation {
    state: Input,
}

#[derive(Debug, Clone, Copy)]
struct Change;

impl Simulation {
    fn tick(&mut self, tilt: Direction) -> usize {
        self.pipe(|Simulation { state }| {
            state
                .all()
                .collect_vec()
                .into_iter()
                .filter_map(|tile| match tile {
                    Positioned {
                        inner: Some(Rock::Round),
                        position,
                    } => match state.neighbour(position, tilt) {
                        Ok(Positioned {
                            inner: None,
                            position: neighbour,
                        }) => state
                            .swap(position, neighbour)
                            .expect("failed to swap")
                            .pipe(|_| Some(Change)),
                        _ => None,
                    },
                    _other => None,
                })
                .count()
        })
    }
}

impl std::fmt::Display for Simulation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.state.fmt(f)
    }
}

fn main() {
    INPUT
        .lines()
        .filter(|l| !l.is_empty())
        .map(|line| {
            line.split("")
                .filter(|v| !v.is_empty())
                .map(Rock::from_str)
                .map(Result::ok)
                .collect_vec()
        })
        .collect_vec()
        .pipe(Input)
        .tap(|input| {
            let total_load = |simulation: &Simulation| -> usize {
                simulation
                    .state
                    .all()
                    .filter_map(|s| match s {
                        Positioned {
                            inner: Some(Rock::Round),
                            position,
                        } => simulation
                            .state
                            .0
                            .len()
                            .sub(position.row.conv::<usize>())
                            .pipe(Some),
                        _ => None,
                    })
                    .sum::<usize>()
            };
            input
                .clone()
                .pipe(|input| Simulation { state: input })
                .tap(|simulation| {
                    simulation.pipe(Clone::clone).pipe(|mut simulation| {
                        std::iter::once(())
                            .cycle()
                            .map(|_| simulation.tick(Direction::North))
                            .take_while(|&changes| changes != 0)
                            .last()
                            .map(|_| {
                                simulation
                                    .pipe_ref(total_load)
                                    .pipe(|part_1| println!("part 1: {part_1}"))
                            });
                    });
                })
                .tap(|simulation| {
                    simulation.pipe(Clone::clone).pipe(|mut simulation| {
                        (0..1000)
                            .map(|_| {
                                let previous = simulation.clone();
                                Direction::all()
                                    .map(|d| {
                                        std::iter::once(())
                                            .cycle()
                                            .map(|_| simulation.tick(d))
                                            .take_while(|&changes| changes != 0)
                                            .sum()
                                    })
                                    .iter()
                                    .sum::<usize>()
                                    .pipe(|cycle_changes| {
                                        (previous, simulation.clone(), cycle_changes)
                                    })
                                // .map(|(changes)| (previous, simulation, changes))
                            })
                            .take_while(|(previous, next, _)| previous != next)
                            .last()
                            .map(|_| {
                                total_load(&simulation).pipe(|part_2| println!("part 2: {part_2}"))
                            })
                        // std::iter::once(())
                        //     .cycle()
                        //     .map(|_| simulation.tick())
                        //     .take_while(|&changes| changes != 0)
                        //     .last()
                        //     .map(|_| {
                        //         println!("{simulation}");
                        //         simulation.pipe_ref(total_load).pipe(|part_1| println!("part 1: {part_1}"))
                        //     });
                    });
                });
        });
}

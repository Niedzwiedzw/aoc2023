use eyre::ContextCompat;
use itertools::Itertools;
use std::{
    fmt::Debug,
    ops::{Div, Mul},
};
use tap::prelude::*;

const INPUT: &str = include_str!("./day13.txt");

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Position {
    row: i32,
    column: i32,
}

impl std::fmt::Display for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.flipped_by(|position| position.row).fmt(f)
    }
}

type Key = fn(Position) -> i32;

struct FlippedBy<'pattern> {
    pattern: &'pattern Pattern,
    key: Key,
}

impl Pattern {
    fn flipped_by(&self, key: Key) -> FlippedBy<'_> {
        FlippedBy { pattern: self, key }
    }
}

impl<'pattern> FlippedBy<'pattern> {
    fn rows(&self) -> Vec<(i32, Vec<char>)> {
        self.pattern
            .positions()
            .sorted()
            .sorted_by_key(|&position| (self.key)(position))
            .group_by(|&position| (self.key)(position))
            .into_iter()
            .map(|(idx, line)| {
                line.into_iter()
                    .map(|position| self.pattern.get(position).expect("bad position"))
                    .collect::<Vec<char>>()
                    .pipe(|line| (idx, line))
            })
            .collect_vec()
    }
}
impl<'pattern> std::fmt::Display for FlippedBy<'pattern> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.rows().into_iter().try_for_each(|(idx, line)| {
            line.into_iter()
                .collect::<String>()
                .pipe(|line| writeln!(f, "{idx}\t{line}"))
        })
    }
}

#[derive(Clone)]
struct Pattern(Vec<Vec<char>>);

impl Pattern {
    fn get_mut(&mut self, position: Position) -> Option<&mut char> {
        self.0
            .get_mut(position.row.try_conv::<usize>().expect("bad row 2"))
            .and_then(|line| {
                line.get_mut(position.column.try_conv::<usize>().expect("bad column 2"))
            })
    }
    fn smudge_fixes(&self) -> impl Iterator<Item = Self> + '_ {
        self.positions().map(|position| {
            self.clone().tap_mut(|pattern| {
                pattern.get_mut(position).unwrap().pipe(|current| {
                    match *current {
                        '.' => '#',
                        '#' => '.',
                        other => panic!("should not be {other}"),
                    }
                    .pipe(|new| *current = new)
                })
            })
        })
    }
}

impl Pattern {
    fn positions(&self) -> impl Iterator<Item = Position> + '_ {
        (0..self.0.len()).flat_map(|row| {
            self.0
                .first()
                .map(|first| first.len())
                .unwrap_or_default()
                .pipe(move |len| {
                    (0..len).map(move |column| Position {
                        column: column.try_conv().expect("bad column"),
                        row: row.try_conv().expect("bad row"),
                    })
                })
        })
    }
    fn get(&self, position: Position) -> Option<char> {
        self.0
            .get(position.row.try_conv::<usize>().expect("bad row 2"))
            .and_then(|line| line.get(position.column.try_conv::<usize>().expect("bad column 2")))
            .copied()
    }
}
fn is_reflection_by<U, T, F>(slice: &[U], by: F) -> bool
where
    T: PartialEq + Eq,
    F: Fn(&U) -> &T,
{
    slice
        .iter()
        .zip(slice.iter().rev())
        .all(|(one, other)| by(one) == by(other))
}

fn part_1(pattern: &Pattern) -> Vec<u32> {
    [
        [
            ((|Position { column: _, row }| row) as fn(_) -> _, false),
            ((|Position { column: _, row }| -row), true),
        ],
        [
            ((|Position { column, row: _ }| column), false),
            ((|Position { column, row: _ }| -column), true),
        ],
    ]
    .map(|keys| {
        keys.iter()
            .map(|&(key, flip)| {
                pattern.flipped_by(key).pipe(|flipped| {
                    flipped.rows().pipe(|rows| {
                        (2..(rows.len()))
                            .rev()
                            .filter(|r| r % 2 == 0)
                            .find_map(move |offset| {
                                (&rows[..offset])
                                    .pipe(|slice| {
                                        is_reflection_by(slice, |(_idx, slice)| slice)
                                            .then_some(slice.len().div(2))
                                            .map(|slice_part| {
                                                slice_part
                                                    + match flip {
                                                        false => 0,
                                                        true => rows.len() - slice.len(),
                                                    }
                                            })
                                    })
                                    .map(|v| v.try_conv::<u32>().unwrap())
                            })
                    })
                    // .tap(|result| println!("{flipped}\nresult: {result:?}\n\n"))
                })
            })
            .collect_vec()
    })
    .pipe(|[row, column]| {
        row.into_iter()
            .zip(column)
            .flat_map(|(row, column)| match (row, column) {
                (None, None) => vec![],
                (None, Some(column)) => vec![column],
                (Some(row), None) => vec![row.mul(100)],
                (Some(row), Some(column)) => vec![row.mul(100), column],
            })
    })
    .collect_vec()
}

fn main() {
    INPUT
        .split("\n\n")
        .map(|pattern| {
            pattern
                .lines()
                .map(|line| line.chars().collect_vec())
                .collect_vec()
                .pipe(Pattern)
        })
        .collect_vec()
        .tap(|patterns| {
            patterns
                .iter()
                .map(|pattern| {
                    part_1(pattern)
                        .tap(|value| {
                            println!("pattern:\n{pattern}\nvalue: {value:?}\n\n");
                        })
                        .first()
                        .copied()
                        .expect("must be here")
                })
                .sum::<u32>()
                .pipe(|part_1| println!("part 1: {part_1}"))
        })
        .tap(|patterns| {
            patterns
                .iter()
                .map(|original| {
                    let original_res = part_1(original)
                        .first()
                        .copied()
                        .expect("already checked above");
                    original
                        .smudge_fixes()
                        .flat_map(|pattern| part_1(&pattern))
                        .find(|p| p != &original_res)
                        .context(format!("should be at least one for \n{original}"))
                        .unwrap()
                        .tap(|v| {
                            println!("pattern:\n{original}\npart 2 value: {v}");
                        })
                })
                .sum::<u32>()
                .pipe(|part_2| println!("part 2: {part_2}"))
        });
}

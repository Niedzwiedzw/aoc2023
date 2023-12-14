use std::collections::BTreeMap;

use eyre::ContextCompat;
use itertools::Itertools;
use tap::prelude::*;

const INPUT: &str = include_str!("./day11.txt");

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Point {
    row: i64,
    column: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Galaxy {
    id: usize,
    position: Point,
}

impl Point {
    fn distance(self, other: &Self) -> u64 {
        self.row.abs_diff(other.row) + self.column.abs_diff(other.column)
    }
    fn rows(&self, other: &Self) -> impl Iterator<Item = u64> {
        (self.row.min(other.row)..(self.row.max(other.row))).map(|v| v.try_conv::<_>().unwrap())
    }
    fn columns(&self, other: &Self) -> impl Iterator<Item = u64> {
        (self.column.min(other.column)..(self.column.max(other.column)))
            .map(|v| v.try_conv::<_>().unwrap())
    }
}

fn columns<T: Copy>(input: &[Vec<T>]) -> impl Iterator<Item = Vec<T>> + '_ {
    input.first().expect("cannot be empty").len().pipe(|len| {
        (0..len).map(|idx| {
            input
                .iter()
                .map(|line| line.get(idx).expect("bad column"))
                .copied()
                .collect_vec()
        })
    })
}

fn main() {
    INPUT
        .lines()
        .filter(|l| !l.is_empty())
        .map(|line| line.chars().collect_vec())
        .collect_vec()
        .tap(|input| {
            println!("input:");
            input.iter().for_each(|line| {
                line.iter()
                    .copied()
                    .join("")
                    .pipe(|line| println!("{line}"))
            })
        })
        .pipe(|lines| {
            lines
                .iter()
                .enumerate()
                .filter_map(|(row, line)| (!line.contains(&'#')).then_some(row))
                .collect_vec()
                .pipe(|expanded_rows| {
                    columns(&lines)
                        .enumerate()
                        .filter_map(|(row, line)| (!line.contains(&'#')).then_some(row))
                        .collect_vec()
                        .pipe(|expanded_columns| {
                            lines
                                .into_iter()
                                .enumerate()
                                .flat_map(|(row, line)| {
                                    line.into_iter().enumerate().filter_map(move |(column, c)| {
                                        c.eq(&'#').then_some(Point {
                                            row: row.try_conv().unwrap(),
                                            column: column.try_conv().unwrap(),
                                        })
                                    })
                                })
                                .enumerate()
                                .map(|(id, position)| Galaxy {
                                    id: id + 1,
                                    position,
                                })
                                .collect_vec()
                                .pipe(|galaxies| {
                                    println!("{galaxies:#?}");
                                    galaxies
                                        .clone()
                                        .into_iter()
                                        .flat_map(move |galaxy| {
                                            galaxies
                                                .clone()
                                                .into_iter()
                                                .filter(move |other| &galaxy != other)
                                                .map(|other| {
                                                    galaxy
                                                        .position
                                                        .distance(&other.position)
                                                        .pipe(|distance| {
                                                            galaxy
                                                                .position
                                                                .rows(&other.position)
                                                                .filter(|&row| {
                                                                    expanded_rows.contains(
                                                                        &row.try_conv::<usize>()
                                                                            .unwrap(),
                                                                    )
                                                                })
                                                                .chain(
                                                                    galaxy
                                                                        .position
                                                                        .columns(&other.position)
                                                                        .filter(|&column| {
                                                                            expanded_columns
                                                                                .contains(
                                                                                &column
                                                                                    .try_conv::<usize>()
                                                                                    .unwrap(),
                                                                            )
                                                                        }),
                                                                )
                                                                // PART 1
                                                                // .count()
                                                                // .try_conv::<u64>()
                                                                // .expect("bad distance")

                                                                // PART 2
                                                                .map(|_| 1000000 - 1).sum::<u64>()
                                                                + distance
                                                        })
                                                        .pipe(|distance| {
                                                            (
                                                                [galaxy.id, other.id]
                                                                    .tap_mut(|a| a.sort()),
                                                                distance,
                                                            )
                                                        })
                                                })
                                                .collect_vec()
                                        })
                                        .fold(BTreeMap::default(), |acc, ([a, b], distance)| {
                                            acc.tap_mut(|acc| {
                                                acc.entry(a.min(b))
                                                    .or_insert(BTreeMap::new())
                                                    .insert(a.max(b), distance);
                                            })
                                        })
                                })
                                .pipe(|distances| {
                                    distances.tap(|distances| {
                                        println!("{distances:#?}");
                                        distances
                                            .values()
                                            .flat_map(|v| v.values())
                                            .count()
                                            .pipe(|pairs| println!("pairs: {pairs}"));

                                        distances
                                            .values()
                                            .flat_map(|v| v.values())
                                            .sum::<u64>()
                                            .pipe(|part_1| {
                                                println!("part 1: {part_1}");
                                            })
                                    })
                                });
                        })
                })
        })
}

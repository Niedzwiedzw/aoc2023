use std::ops::Mul;

use eyre::{eyre, Result, WrapErr};
use itertools::Itertools;
use tap::prelude::*;

const INPUT: &str = include_str!("./day6.txt");

trait BoxedIterator<Item> {
    fn boxed(self) -> Box<dyn Iterator<Item = Item>>;
}

fn main() -> Result<()> {
    let ways_to_win = |time: usize, distance: usize| {
        (0..time)
            .filter_map(move |held| {
                time.checked_sub(held)
                    .map(|remaining_time| remaining_time.mul(held.pipe(|speed| speed)))
                    .map(|distance| (held, distance))
            })
            .filter(move |(_held, moved)| moved.gt(&distance))
            .count()
    };
    INPUT
        .lines()
        .filter(|line| !line.is_empty())
        .collect_vec()
        .pipe(|v| {
            v.try_conv::<[&str; 2]>()
                .map_err(|v| eyre!("bad lines: {v:?}"))
                .unwrap()
        })
        .tap(|[time, distance]| {
            (
                time.split_whitespace()
                    .filter(|v| !v.is_empty())
                    .skip(1)
                    .map(|v| {
                        v.parse::<u32>()
                            .wrap_err_with(|| format!("bad numbers: {v:?}"))
                    })
                    .collect::<Result<Vec<_>>>()
                    .unwrap(),
                distance
                    .split_whitespace()
                    .filter(|v| !v.is_empty())
                    .map(|v| {
                        v.parse::<u32>()
                            .wrap_err_with(|| format!("bad numbers: {v:?}"))
                    })
                    .skip(1)
                    .collect::<Result<Vec<_>>>()
                    .unwrap(),
            )
                .pipe(|(time, distance)| time.into_iter().zip(distance).collect_vec())
                .tap(|entries| {
                    entries
                        .iter()
                        .copied()
                        .map(|(time, distance)| ways_to_win(time as _, distance as _))
                        .product::<usize>()
                        .pipe(|part_1| {
                            println!("part 1: {part_1}");
                        })
                });
        })
        .tap(|[time, distance]| {
            let parse = |input: &str| {
                input.split_once(": ").unwrap().pipe(|(_, entries)| {
                    entries
                        .split_whitespace()
                        .join("")
                        .pipe(|v| v.parse::<usize>().unwrap())
                })
            };
            (time.pipe_deref(parse), distance.pipe_deref(parse)).pipe(|(time, distance)| {
                ways_to_win(time, distance).pipe(|part_2| {
                    println!("part 2: {part_2}");
                })
            })
        })
        .pipe(|_| Ok(()))
}

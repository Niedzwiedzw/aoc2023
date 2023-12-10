use std::{collections::HashSet, iter::once};

use eyre::Result;
use itertools::Itertools;
use tap::prelude::*;

const INPUT: &str = include_str!("./day4.txt");

type Scored = (usize, usize);

fn copies(current: Scored, values: &[Scored]) -> Vec<Scored> {
    once(current)
        // empty()
        .chain(current.pipe(|(id, count)| {
            (0..count)
                .map(move |count| id + count)
                .filter_map(|id| values.get(id))
                .flat_map(|current| copies(*current, values))
        }))
        .collect()
}

fn main() -> Result<()> {
    INPUT
        .lines()
        .filter_map(|line| {
            line.split_once(':').and_then(|(card, scores)| {
                card.split_once(' ').and_then(|(_, id)| id.trim().parse::<usize>().ok()).map(|id| {
                    scores
                        .split_once('|')
                        .and_then(|(winning, actual)| {
                            winning
                                .split_ascii_whitespace()
                                .map(|v| v.parse::<usize>())
                                .collect::<Result<HashSet<_>, _>>()
                                .ok()
                                .zip(
                                    actual
                                        .split_ascii_whitespace()
                                        .map(|v| v.parse::<usize>())
                                        .collect::<Result<Vec<_>, _>>()
                                        .ok(),
                                )
                                .map(|(winning, actual)| (id, (winning, actual)))
                        })
                        .expect("bad input")
                })
            })
        })
        .collect::<Vec<_>>()
        .pipe_ref(|values| {
            values
                .iter()
                .map(|(id, (winning, actual))| {
                    actual
                        .iter()
                        .filter(|actual| winning.contains(actual))
                        .count()
                        .pipe(|count| (id, (winning, actual), count))
                })
                .collect::<Vec<_>>()
                .tap(|scored| {
                    scored
                        .iter()
                        .map(|(_id, _, count)| match count {
                            0 => 0,
                            &count => 2usize.pow((count as u32) - 1),
                        })
                        .sum::<usize>()
                        .tap(|part_1| {
                            println!("part 1: {part_1}");
                        });
                })
                .tap(|scored| {
                    scored.pipe(|scored| {
                        scored.iter().map(|(id, _, score)| (**id, *score)).collect_vec().tap(|scored| {
                            scored.iter().flat_map(|current| copies(*current, &scored[..])).count().tap(|part_2| {
                                println!("part 2: {}", part_2);
                            });
                        });
                    })
                })
        });

    Ok(())
}

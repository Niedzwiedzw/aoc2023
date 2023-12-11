use itertools::Itertools;
use tap::prelude::*;

const INPUT: &str = include_str!("./day9.txt");

fn differences(line: &[i32]) -> Option<Vec<i32>> {
    line.iter().any(|v| v != &0).then(|| {
        line.iter()
            .zip(line.iter().skip(1))
            .map(|(prev, next)| next - prev)
            .collect()
    })
}

fn main() {
    INPUT
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| {
            l.split_whitespace()
                .map(|v| v.parse::<i32>().expect("bad input"))
                .collect_vec()
        })
        .collect_vec()
        .tap(|input| {
            input
                .iter()
                .cloned()
                .map(|line| vec![line])
                .map(|line| {
                    line.tap_mut(|line| {
                        while let Some(differences) = differences(line.last().unwrap()) {
                            line.push(differences);
                        }
                    })
                })
                .collect_vec()
                .tap(|results| {
                    results
                        .iter()
                        .cloned()
                        .map(|line| {
                            (0..line.len()).rev().pipe(|range| {
                                range
                                    .clone()
                                    .zip(range.skip(1))
                                    .fold(line, |line, (prev, next)| {
                                        line.tap_mut(|line| {
                                            line.get(next)
                                                .zip(line.get(prev))
                                                .and_then(|(next, prev)| {
                                                    next.last().zip(prev.last()).map(
                                                        |(value, prediciton)| value + prediciton,
                                                    )
                                                })
                                                .unwrap_or_default()
                                                .pipe(|prediction| {
                                                    line.get_mut(next).unwrap().push(prediction);
                                                })
                                        })
                                    })
                            })
                        })
                        .collect_vec()
                        .tap(|with_predictions| {
                            println!("{with_predictions:#?}");
                            with_predictions
                                .iter()
                                .flat_map(|line| {
                                    line.first()
                                        .iter()
                                        .flat_map(|subsequence| subsequence.last())
                                        .collect_vec()
                                })
                                .sum::<i32>()
                                .pipe(|part_1| println!("part 1: {part_1}"))
                        });
                })
                .tap(|results| {
                    results
                        .iter()
                        .cloned()
                        .map(|line| {
                            (0..line.len()).rev().pipe(|range| {
                                range
                                    .clone()
                                    .zip(range.skip(1))
                                    .fold(line, |line, (prev, next)| {
                                        line.tap_mut(|line| {
                                            line.get(next)
                                                .zip(line.get(prev))
                                                .and_then(|(next, prev)| {
                                                    next.first().zip(prev.first()).map(
                                                        |(value, prediciton)| value - prediciton,
                                                    )
                                                })
                                                .unwrap_or_default()
                                                .pipe(|prediction| {
                                                    line.get_mut(next)
                                                        .unwrap()
                                                        .insert(0, prediction);
                                                })
                                        })
                                    })
                            })
                        })
                        .collect_vec()
                        .tap(|with_predictions| {
                            println!("{with_predictions:#?}");
                            with_predictions
                                .iter()
                                .flat_map(|line| {
                                    line.first()
                                        .iter()
                                        .flat_map(|subsequence| subsequence.first())
                                        .collect_vec()
                                })
                                .sum::<i32>()
                                .pipe(|part_2| println!("part 2: {part_2}"))
                        });
                });
        });
}

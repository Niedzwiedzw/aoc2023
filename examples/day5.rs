use std::collections::BTreeMap;

use eyre::{eyre, ContextCompat, Result, WrapErr};
use itertools::Itertools;
use tap::prelude::*;

const INPUT: &str = include_str!("./day5.txt");

#[derive(Clone, Copy, Debug)]
struct MappingPart {
    source: usize,
    destination: usize,
    length: usize,
}

#[derive(Clone, Debug)]
struct Mapping(Vec<MappingPart>);

impl Mapping {
    pub fn get(&self, key: usize) -> Option<usize> {
        self.0.iter().rev().find_map(|mapping| mapping.get(key))
    }
}

impl MappingPart {
    pub fn get(&self, key: usize) -> Option<usize> {
        let Self {
            source,
            destination,
            length,
        } = *self;
        match source <= key && key <= source + length {
            true => Some(key + (destination - source)),
            false => None,
        }
    }
}

fn main() -> Result<()> {
    let parse_head = |head: &str| {
        head.split_once(": ")
            .with_context(|| format!("header title: {head:?}"))
            .map(|(_, seeds)| {
                seeds
                    .split_whitespace()
                    .map(|v| v.parse::<usize>().wrap_err("not a number"))
                    .collect::<Result<Vec<_>>>()
                    .expect("bad input")
            })
    };
    INPUT
        .split_once("\n\n")
        .context("header")
        .and_then(|(head, maps)| {
            parse_head(head).and_then(|seeds| {
                maps.split("\n\n")
                    .filter(|l| !l.trim().is_empty())
                    .map(|map| {
                        map.split_once(":\n")
                            .with_context(|| format!("map header: {map:?}"))
                            .and_then(|(head, mappings)| {
                                head.split_once(' ')
                                    .context("map header name")
                                    .and_then(|(name, _)| {
                                        name.split_once("-to-").context("mapping kind")
                                    })
                                    .and_then(|(source_name, destination_name)| {
                                        mappings
                                            .split('\n')
                                            .filter(|line| !line.is_empty())
                                            .map(|line| {
                                                line.split_whitespace()
                                                    .map(|value| {
                                                        value.parse::<usize>().with_context(|| {
                                                            format!("range number: {value:?}")
                                                        })
                                                    })
                                                    .collect::<Result<Vec<_>>>()
                                                    .and_then(|v| {
                                                        v.try_conv::<[usize; 3]>()
                                                            .map_err(|_| eyre!("bad length"))
                                                    })
                                                    .map(|[destination, source, length]| {
                                                        (destination, source, length)
                                                    })
                                                    .with_context(|| {
                                                        format!("parsing line: {line:?}")
                                                    })
                                            })
                                            .collect::<Result<Vec<_>>>()
                                            .map(|mappings| {
                                                ((source_name, destination_name), mappings)
                                            })
                                            .context("numbers")
                                            .with_context(|| {
                                                format!("parsing mappings: {mappings:?}")
                                            })
                                    })
                            })
                    })
                    .collect::<Result<Vec<_>>>()
                    .context("mappings")
                    .map(|mappings| (seeds, mappings))
                    .map(|(seeds, mappings)| {
                        mappings
                            .into_iter()
                            .map(|((source_name, destination_name), mappings)| {
                                mappings
                                    .iter()
                                    .cloned()
                                    .map(|(destination, source, length)| MappingPart {
                                        source,
                                        length,
                                        destination,
                                    })
                                    .collect_vec()
                                    .pipe(|mappings| {
                                        ((source_name, destination_name), Mapping(mappings))
                                    })
                            })
                            .collect::<BTreeMap<_, _>>()
                            .pipe(|seed_map| {
                                let min_out_of =
                                    |seeds: Vec<usize>| {
                                        seeds
                                            .iter()
                                            .map(|&seed| {
                                                [
                                                    ("seed", "soil"),
                                                    ("soil", "fertilizer"),
                                                    ("fertilizer", "water"),
                                                    ("water", "light"),
                                                    ("light", "temperature"),
                                                    ("temperature", "humidity"),
                                                    ("humidity", "location"),
                                                ]
                                                .into_iter()
                                                .fold(seed, |seed, mapping| {
                                                    seed_map
                                                        .get(&mapping)
                                                        .expect("bad mapping")
                                                        .get(seed)
                                                        .unwrap_or(seed)
                                                })
                                            })
                                            .min()
                                    };
                                min_out_of(seeds.clone()).pipe(|day_1| {
                                    println!("day 1: {day_1:?}");
                                });
                                min_out_of(
                                    seeds
                                        .windows(2)
                                        .step_by(2)
                                        .map(|window| {
                                            window.to_vec().try_conv::<[usize; 2]>().unwrap()
                                        })
                                        .flat_map(|[start, length]| (start..(start + length)))
                                        .collect(),
                                )
                                .pipe(|day_2| {
                                    println!("day 2: {day_2:?}");
                                });
                            })
                    })
            })
        })
}

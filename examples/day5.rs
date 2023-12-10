use std::{
    collections::BTreeMap,
    iter::{empty, once},
};

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

// impl MappingPart {
//     pub fn merge(self, other: MappingPart) -> impl Iterator<Item = Self> + '_ {}
// }

trait BoxedIterator<Item> {
    fn boxed(self) -> Box<dyn Iterator<Item = Item>>;
}

impl<T, Item> BoxedIterator<Item> for T
where
    T: Iterator<Item = Item> + 'static,
{
    fn boxed(self) -> Box<dyn Iterator<Item = Item>> {
        Box::new(self)
    }
}

impl MappingPart {
    pub fn map_range(&self, SeedRange { start, end }: SeedRange) -> impl Iterator<Item = SeedRange> + '_ {
        let Self { source, destination, length } = *self;
        let a = start;
        let b = end;
        let c = source;
        let d = source + length;
        // A -- B
        //        C -- D
        if b < c {
            empty().chain(once((a, b))).boxed()
        // A -- B
        //   C -- D
        } else if a < c && c < b {
            empty().chain(once((a, c - 1))).chain(once((destination, destination + length))).boxed()
        //    A -- B
        // C -- D
        } else if c < a && a < d {
            empty().chain(once((destination, destination + length))).chain(once((d + 1, b))).boxed()
        //         A -- B
        // C -- D
        } else if a < d {
            empty().chain(once((a, b))).boxed()
        //         A -- B
        // C --------------- D
        } else if c < a && b < d {
            empty().chain(once((a + destination - source, b + destination - source))).boxed()
        //         A ------------- B
        //              C --- D
        } else {
            empty()
                .chain(once((a, c - 1)))
                .chain(once((destination, destination + length)))
                .chain(once((d + 1, b)))
                .boxed()
        }
        .map(|(start, end)| SeedRange { start, end })
    }
}

#[derive(Clone, Debug)]
struct Mapping(Vec<MappingPart>);

impl Mapping {
    pub fn get(&self, key: usize) -> Option<usize> {
        self.0.iter().rev().find_map(|mapping| mapping.get(key))
    }
    pub fn map_range(&self, seed_range: SeedRange) -> impl Iterator<Item = SeedRange> + '_ {
        self.0.iter().flat_map(move |part| part.map_range(seed_range).unique()).unique()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct SeedRange {
    start: usize,
    end: usize,
}

impl MappingPart {
    pub fn get(&self, key: usize) -> Option<usize> {
        let Self { source, destination, length } = *self;
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
    const MAPPING_ORDER: &[(&str, &str)] = &[
        ("seed", "soil"),
        ("soil", "fertilizer"),
        ("fertilizer", "water"),
        ("water", "light"),
        ("light", "temperature"),
        ("temperature", "humidity"),
        ("humidity", "location"),
    ];
    INPUT.split_once("\n\n").context("header").and_then(|(head, maps)| {
        parse_head(head).and_then(|seeds| {
            maps.split("\n\n")
                .filter(|l| !l.trim().is_empty())
                .map(|map| {
                    map.split_once(":\n")
                        .with_context(|| format!("map header: {map:?}"))
                        .and_then(|(head, mappings)| {
                            head.split_once(' ')
                                .context("map header name")
                                .and_then(|(name, _)| name.split_once("-to-").context("mapping kind"))
                                .and_then(|(source_name, destination_name)| {
                                    mappings
                                        .split('\n')
                                        .filter(|line| !line.is_empty())
                                        .map(|line| {
                                            line.split_whitespace()
                                                .map(|value| value.parse::<usize>().with_context(|| format!("range number: {value:?}")))
                                                .collect::<Result<Vec<_>>>()
                                                .and_then(|v| v.try_conv::<[usize; 3]>().map_err(|_| eyre!("bad length")))
                                                .map(|[destination, source, length]| (destination, source, length))
                                                .with_context(|| format!("parsing line: {line:?}"))
                                        })
                                        .collect::<Result<Vec<_>>>()
                                        .map(|mappings| ((source_name, destination_name), mappings))
                                        .context("numbers")
                                        .with_context(|| format!("parsing mappings: {mappings:?}"))
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
                                .map(|(destination, source, length)| MappingPart { source, length, destination })
                                .collect_vec()
                                .pipe(|mappings| ((source_name, destination_name), Mapping(mappings)))
                        })
                        .collect::<BTreeMap<_, _>>()
                        .pipe(|seed_map| {
                            let min_out_of = |seeds: Vec<usize>| {
                                seeds
                                    .iter()
                                    .map(|&seed| {
                                        MAPPING_ORDER.iter().cloned().fold(seed, |seed, mapping| {
                                            seed_map.get(&mapping).expect("bad mapping").get(seed).unwrap_or(seed)
                                        })
                                    })
                                    .min()
                            };
                            min_out_of(seeds.clone()).pipe(|day_1| {
                                println!("day 1: {day_1:?}");
                            });

                            seeds
                                .windows(2)
                                .step_by(2)
                                .map(|window| window.to_vec().try_conv::<[usize; 2]>().unwrap())
                                .map(|[start, length]| (start, (start + length)))
                                .map(|(start, end)| SeedRange { start, end })
                                .flat_map(|seed| {
                                    MAPPING_ORDER.iter().cloned().fold(vec![seed], |seeds, mapping| {
                                        seeds
                                            .iter()
                                            .cloned()
                                            .flat_map(|seed| seed_map.get(&mapping).expect("bad mapping").map_range(seed))
                                            .collect::<Vec<_>>()
                                            .tap(|new_seeds| {
                                                println!("{seeds:?} -> {new_seeds:?}");
                                            })
                                    })
                                })
                                .map(|SeedRange { start, .. }| start)
                                .min()
                                .pipe(|day_2| {
                                    println!("day 2: {day_2:?}");
                                })
                        })
                })
        })
    })
}

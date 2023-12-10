use std::{collections::BTreeMap, iter::empty};

use eyre::{Context, ContextCompat, Result};
use tap::prelude::*;

const INPUT: &str = include_str!("./day8.txt");

fn main() {
    INPUT.split_once("\n\n").map(|(directions, maps)| {
        maps.split('\n')
            .filter(|l| !l.is_empty())
            .map(|line| {
                line.split_once(" = ")
                    .context("no = ")
                    .and_then(|(key, leftright)| {
                        leftright
                            .trim_start_matches('(')
                            .trim_end_matches(')')
                            .split_once(", ")
                            .context("no comma")
                            .map(|(left, right)| (key, (left, right)))
                    })
                    .wrap_err_with(|| format!("parsing line: {line}"))
            })
            .collect::<Result<Vec<_>>>()
            .wrap_err("parsing maps")
            .map(|lines| {
                let first = lines.first().expect("empty input").0;
                lines
                    .into_iter()
                    .map(|(key, (left, right))| {
                        (
                            key,
                            BTreeMap::new()
                                .tap_mut(|map| {
                                    map.insert('L', left);
                                })
                                .tap_mut(|map| {
                                    map.insert('R', right);
                                }),
                        )
                    })
                    .collect::<BTreeMap<_, _>>()
                    .tap(|lookup| {
                        let mut current = (0usize, if lookup.contains_key("AAA") { "AAA" } else { first });
                        directions
                            .chars()
                            .cycle()
                            .enumerate()
                            .map(|(step, direction)| (step + 1, direction))
                            .map(|(step, direction)| {
                                lookup
                                    .get(current.1)
                                    .with_context(|| format!("in unknown location: {:?}", current))
                                    .unwrap()
                                    .pipe(|lookup| lookup.get(&direction).context("invalid direction").unwrap())
                                    .pipe(|new_location| {
                                        current = (step, *new_location);
                                        current
                                    })
                            })
                            .find(|(_step, curr)| curr == &"ZZZ")
                            .pipe(|part_1| {
                                println!("part 1: {part_1:?}");
                            });
                    })
                    .tap(|lookup| {
                        lookup
                            .keys()
                            .filter(|key| key.ends_with('A'))
                            .flat_map(|key| all_descendants(lookup, key).inspect(move |next| println!("{key} -> {next}")))
                            .count()
                            .pipe(|part_2| {
                                println!("part 2: {part_2}");
                            })
                    })
            })
    });
}

fn all_descendants<'iterator, 'input>(
    lookup: &'input BTreeMap<&'input str, BTreeMap<char, &'input str>>,
    parent: &'input str,
) -> Box<dyn Iterator<Item = &'input str> + 'iterator>
where
    'input: 'iterator,
{
    lookup
        .get(parent)
        .expect("bad parent")
        .values()
        .filter(move |&value| value != &parent)
        .flat_map(move |&parent| match parent.ends_with('Z') {
            false => all_descendants(lookup, parent),
            true => empty().pipe(Box::new),
        })
        .pipe(Box::new)
}

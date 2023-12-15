use std::{
    collections::BTreeMap,
    ops::{Add, Mul},
};

use itertools::Itertools;
use tap::prelude::*;

const INPUT: &str = include_str!("./day15.txt");

#[derive(Debug, Default, PartialEq, Eq)]
struct HashState(usize);

fn ascii_code(c: char) -> u8 {
    c.try_conv().expect("bad char")
}

impl HashState {
    fn feed(&mut self, c: char) {
        self.0 = self
            .0
            .add(ascii_code(c).conv::<usize>())
            .pipe(|new| new.mul(17).pipe(|new| new % 256))
    }
}

fn hash(sequence: &str) -> usize {
    sequence
        .chars()
        .fold(HashState::default(), |acc, next| {
            acc.tap_mut(|acc| acc.feed(next))
        })
        .pipe(|HashState(state)| state)
        .tap(|output| println!("{sequence} -> {output}"))
}

#[derive(Debug)]
enum Operation<'input> {
    Remove(&'input str),
    Replace(LensEntry<'input>),
}

#[derive(Clone, Copy)]
struct LensEntry<'input> {
    label: &'input str,
    length: u8,
}

impl<'a> std::fmt::Debug for LensEntry<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.pipe(|Self { label, length }| write!(f, "[{label} {length}]"))
    }
}

fn remove_by<T, F>(collection: &mut Vec<T>, key: F) -> Option<T>
where
    F: (Fn(&T) -> bool),
{
    match collection.iter().find_position(|e| key(e)) {
        Some((index, _)) => collection.remove(index).pipe(Some),
        None => None,
    }
}

fn main() {
    assert_eq!(72, ascii_code('H'));
    assert_eq!(52, hash("HASH"));
    INPUT
        .trim()
        .split(',')
        .map(|operation| {
            match operation.ends_with('-') {
                true => (&operation[..(operation.len() - 1)])
                    .pipe(|label| Operation::Remove(label).pipe(|parsed| (hash(label), parsed))),
                false => operation
                    .split_once('=')
                    .expect("no =")
                    .pipe(|(label, focal)| {
                        {
                            focal
                                .parse::<u8>()
                                .expect("bad number")
                                .pipe(|length| Operation::Replace(LensEntry { label, length }))
                                .pipe(|parsed| (hash(label), parsed))
                        }
                    }),
            }
            .pipe(|parsed| (operation, parsed))
        })
        .collect_vec()
        .tap(|part_1| {
            part_1
                .iter()
                .map(|(operation, (_, _))| hash(operation))
                .sum::<usize>()
                .pipe(|v| println!("part 1: {v}"))
        })
        .tap(|part_2| {
            println!();
            part_2
                .iter()
                .fold(
                    BTreeMap::<usize, Vec<LensEntry>>::default(),
                    |acc, (raw, (hash, operation))| {
                        acc.tap_mut(|acc| {
                            acc.entry(*hash)
                                .or_default()
                                .tap_mut(|entry| match operation {
                                    Operation::Remove(label) => {
                                        remove_by(entry, |k| k.label.eq(*label));
                                    }
                                    Operation::Replace(to @ LensEntry { label, length }) => entry
                                        .iter_mut()
                                        .find(|l| l.label.eq(*label))
                                        .map(|found| found.length = *length)
                                        .unwrap_or_else(|| {
                                            entry.push(*to);
                                        }),
                                });
                        })
                        .tap(|boxes_state| {
                            println!("After \"{raw}\":");
                            boxes_state.iter().for_each(|(idx, value)| {
                                println!(
                                    "Box {idx}: {value}",
                                    value = value.iter().map(|f| format!("{f:?}")).join(" ")
                                );
                            });
                            println!();
                        })
                    },
                )
                .pipe(|boxes| {
                    boxes
                        .iter()
                        .flat_map(|(box_idx, b)| {
                            b.iter().copied().enumerate().map(|(i, v)| (i + 1, v)).map(
                                move |(slot_idx, lens_entry @ LensEntry { label: _, length })| {
                                    box_idx
                                        .add(1)
                                        .mul(slot_idx.mul((length).conv::<usize>()))
                                        .tap(|focusing_power| {
                                            println!(" - {lens_entry:?} (box {box_idx}) - {focusing_power}");
                                        })
                                },
                            )
                        })
                        .sum::<usize>()
                        .pipe(|part_2| {
                            println!("part 2: {part_2}");
                        });
                });
        });
}

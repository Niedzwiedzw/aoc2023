use std::collections::BTreeMap;

use eyre::Result;
use itertools::Itertools;
use tap::prelude::*;

const INPUT: &str = include_str!("./day3.txt");

#[derive(Debug)]
struct Entry<'input> {
    val: EntryKind<'input>,
    coords: Coords,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Coords {
    x: i32,
    y: i32,
}

#[allow(dead_code)]
fn color(input: &str, substring: &str, color: u16) -> String {
    input.replace(substring, &format!("\x1b[{color}m{substring}\x1b[0m"))
}

impl std::fmt::Debug for Coords {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { x, y } = self;
        write!(f, "[{x}, {y}]")
    }
}

impl Coords {
    pub fn neighbours(self) -> impl Iterator<Item = Self> {
        let Self { x, y } = self;
        [
            Coords { x: x + 1, y: y + 1 },
            Coords { x: x + 1, y: y - 1 },
            Coords { x: x - 1, y: y + 1 },
            Coords { x: x - 1, y: y - 1 },
            Coords { x, y: y + 1 },
            Coords { x, y: y - 1 },
            Coords { x: x + 1, y },
            Coords { x: x - 1, y },
        ]
        .into_iter()
    }
}

#[derive(Debug)]
enum EntryKind<'input> {
    Number(&'input str),
    Symbol(&'input str),
    Dot,
}

impl<'a> EntryKind<'a> {
    pub fn len(&self) -> i32 {
        match self {
            EntryKind::Number(v) => v.len() as _,
            EntryKind::Symbol(v) => v.len() as _,
            EntryKind::Dot => 1,
        }
    }
}

fn take_while<F>(input: &str, take_while: F) -> Option<(&str, &str)>
where
    F: Fn(&char) -> bool,
{
    input
        .chars()
        .take_while(|v| take_while(v))
        .collect::<Vec<_>>()
        .pipe(|matches| matches.len())
        .pipe(|offset| input.split_at(offset))
        .pipe(|(matches, rest)| (!matches.is_empty()).then_some((matches, rest)))
}

fn next_number(input: &str, offset: i32, row: i32) -> Option<(&str, Entry)> {
    take_while(input, |c| c.is_ascii_digit()).map(|(val, new_input)| {
        (
            new_input,
            Entry {
                val: EntryKind::Number(val),
                coords: Coords { x: offset, y: row },
            },
        )
    })
}
fn next_symbol(input: &str, offset: i32, row: i32) -> Option<(&str, Entry)> {
    take_while(input, |c| c != &'.' && !c.is_ascii_digit()).map(|(val, new_input)| {
        (
            new_input,
            Entry {
                val: EntryKind::Symbol(val),
                coords: Coords { x: offset, y: row },
            },
        )
    })
}

fn next_dot(input: &str, offset: i32, row: i32) -> Option<(&str, Entry)> {
    input.strip_prefix('.').map(|new_input| {
        (
            new_input,
            Entry {
                val: EntryKind::Dot,
                coords: Coords { x: offset, y: row },
            },
        )
    })
}

fn next_entry(input: &str, offset: i32, row: i32) -> Option<(&str, Entry)> {
    [next_number, next_dot, next_symbol]
        .into_iter()
        .find_map(|matches| matches(input, offset, row))
}

fn main() -> Result<()> {
    INPUT
        .lines()
        .enumerate()
        .filter(|(_row, line)| !line.is_empty())
        .flat_map(|(row, mut line)| {
            let mut entries = vec![];
            while let Some((rest, entry)) = next_entry(line, entries.iter().map(|v: &Entry| v.val.len()).sum(), row as _) {
                line = rest;
                entries.push(entry);
            }
            entries
        })
        .collect_vec()
        .tap(|entries| {
            entries
                .iter()
                .filter_map(|Entry { val, coords }| match val {
                    EntryKind::Number(number) if number.chars().all(|c| c.is_ascii_digit()) => Some((number, *coords)),
                    _ => None,
                })
                .map(|(engine_number, coords)| {
                    {
                        entries
                            .iter()
                            .find_map(|entry| match entry {
                                Entry {
                                    val: EntryKind::Symbol(_symbol),
                                    coords: check,
                                } => (0..engine_number.len())
                                    .flat_map(|offset| coords.tap_mut(|coords| coords.x += offset as i32).pipe(Coords::neighbours))
                                    .any(|coords| &coords == check)
                                    .then_some(()),
                                _ => None,
                            })
                            .map(|_| ((engine_number, coords), true))
                            .unwrap_or(((engine_number, coords), false))
                    }
                })
                .filter_map(|(engine_number, matches)| matches.then_some(engine_number))
                .map(|(engine_number, _)| engine_number.parse::<i32>().expect("not a number"))
                .sum::<i32>()
                .tap(|v| {
                    println!("day 1: {v:?}");
                });
        })
        .tap(|entries| {
            let lookup = entries
                .iter()
                .flat_map(|Entry { val, coords }| {
                    (0..val.len()).map(move |offset| {
                        (
                            (*coords).tap_mut(|coords| {
                                coords.x += offset;
                            }),
                            val,
                        )
                    })
                })
                .collect::<BTreeMap<_, _>>();
            entries
                .iter()
                .filter(|e| matches!(e.val, EntryKind::Symbol("*")))
                .map(|e| {
                    e.coords
                        .neighbours()
                        .filter_map(|coord| {
                            lookup.get(&coord).and_then(|entry_kind| match entry_kind {
                                EntryKind::Number(number) => Some(number.parse::<i32>().expect("bad number")),
                                _ => None,
                            })
                        })
                        .unique()
                        .collect_vec()
                })
                .filter_map(|values| values.try_conv::<[i32; 2]>().ok().map(|[one, two]| one * two))
                .sum::<i32>()
                .tap(|value| println!("day 2: {value}"));
        });

    Ok(())
}

use std::{cell::RefCell, collections::BTreeMap, iter::once_with};

use itertools::Itertools;
use tap::prelude::*;

const INPUT: &str = include_str!("./day12.txt");

// fn boxed<'a, T>(i: impl Iterator<Item = T> + 'a) -> Box<dyn Iterator<Item = T> + 'a> {
//     Box::new(i)
// }

// fn variants(character: char) -> impl Iterator<Item = char> {
//     match character {
//         '?' => once('#').chain(once('.')).pipe(boxed),
//         other => once(other).pipe(boxed),
//     }
// }

// fn possible_combinations(input: &[char]) -> impl Iterator<Item = Vec<char>> + '_ {
//     match input {
//         [] => once_with(Vec::new).pipe(boxed),
//         [first, rest @ ..] => variants(*first)
//             .flat_map(|variant| {
//                 possible_combinations(rest)
//                     .map(move |rest| rest.tap_mut(|rest| rest.insert(0, variant)))
//             })
//             .pipe(boxed),
//     }
// }

// fn matches_mask(mask: &[usize], input: &[char]) -> bool {
//     input
//         .iter()
//         .group_by(|&&c| c)
//         .into_iter()
//         .filter_map(|(c, group)| c.eq(&'#').then_some(group.count()))
//         .collect_vec()
//         .as_slice()
//         .eq(mask)
// }

type Lookup = BTreeMap<(Vec<char>, Vec<usize>), u128>;

thread_local! {
    static CACHE: RefCell<Lookup> = Default::default();
}

fn count_arrangements(sizes: &[usize], string: &[char]) -> u128 {
    let cached = |sizes: &[usize], string: &[char]| {
        let key = (string.to_vec(), sizes.to_vec());
        CACHE
            .with_borrow(|cache| cache.get(&key).cloned())
            .unwrap_or_else(|| {
                count_arrangements(sizes, string).pipe(|count| {
                    CACHE.with_borrow_mut(|cache| {
                        cache
                            .insert(key.clone(), count)
                            .pipe(|_| *cache.get(&key).unwrap())
                    })
                })
            })
    };

    if sizes.is_empty() && !string.contains(&'#') {
        return 1;
    }
    if sizes.is_empty() || string.is_empty() {
        return 0;
    }

    let mut tot = 0;
    let group = sizes[0];
    if !(string.get(..group).unwrap_or_default()).contains(&'.')
        && string.len() >= group
        && string.get(group..(group + 1)).unwrap_or_default() != ['#']
    {
        tot += cached(
            sizes.get(1..).unwrap_or_default(),
            string.get((group + 1)..).unwrap_or_default(),
        )
    }
    if string.first().copied().unwrap_or('~') != '#' {
        tot += cached(sizes, string.get(1..).unwrap_or_default())
    }
    tot
}

fn answer_line(input: &[char], mask: &[usize]) -> u128 {
    count_arrangements(mask, input)
}

fn answer(input: &[(Vec<char>, Vec<usize>)]) -> u128 {
    input
        .iter()
        .map(|(input, mask)| answer_line(input, mask))
        .sum::<_>()
}

fn part_2_input(input: Vec<char>) -> Vec<char> {
    once_with(|| input.clone())
        .cycle()
        .map(|v| [v, vec!['?']])
        .take(5)
        .flatten()
        .flatten()
        .collect_vec()
        .pipe(|v| v.iter().copied().take(v.len() - 1).collect_vec())
}

fn main() {
    assert_eq!(
        ".#".chars()
            .collect_vec()
            .pipe(part_2_input)
            .into_iter()
            .collect::<String>(),
        ".#?.#?.#?.#?.#"
    );
    assert_eq!(
        "???.###"
            .chars()
            .collect_vec()
            .pipe(part_2_input)
            .into_iter()
            .collect::<String>(),
        "???.###????.###????.###????.###????.###"
    );
    INPUT
        .lines()
        .filter(|line| !line.is_empty())
        .flat_map(|line| {
            line.split_once(' ').map(|(input, counts)| {
                counts
                    .split(',')
                    .map(|c| c.parse::<usize>().expect("bad number"))
                    .collect_vec()
                    .pipe(|counts| (input.chars().collect_vec(), counts))
            })
        })
        .collect_vec()
        .tap(|input| {
            answer(input).pipe(|part_1| {
                println!("part 1: {part_1}\n");
            })
        })
        .tap(|input| {
            input
                .iter()
                .map(|(line, mask)| {
                    (
                        part_2_input(line.clone()),
                        once_with(|| mask.clone())
                            .cycle()
                            .take(5)
                            .flatten()
                            .collect_vec(),
                    )
                })
                .collect_vec()
                .pipe_ref(|input| {
                    input
                        .iter()
                        .map(|(input, sizes)| {
                            count_arrangements(sizes, input).tap(|arrangements| {
                                println!("{input:?}: {arrangements} arrangements");
                            })
                        })
                        .sum::<u128>()
                        .pipe(|part_2| {
                            println!("part 2: {part_2}");
                        })
                })
        });
}

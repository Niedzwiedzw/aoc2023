use eyre::{bail, Result};
use itertools::Itertools;
use std::{collections::BTreeMap, iter::once};
use tap::prelude::*;

const INPUT: &str = include_str!("./day7.txt");

trait BoxedIterator<Item> {
    fn boxed(self) -> Box<dyn Iterator<Item = Item>>;
}

pub const STRENGTHS: &[char] = &[
    'A', 'K', 'Q', 'J', 'T', '9', '8', '7', '6', '5', '4', '3', '2',
];

pub const STRENGTHS_PART_2: &[char] = &[
    'A', 'K', 'Q', 'T', '9', '8', '7', '6', '5', '4', '3', '2', 'J',
];

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
enum HandKind {
    HighCard,
    OnePair,
    TwoPair,
    ThreeOfAKind,
    FullHouse,
    FourOfAKind,
    FiveOfAKind,
}

fn kind(hand: &str) -> Result<HandKind> {
    use HandKind::*;
    hand.chars()
        .sorted()
        .group_by(|&v| v)
        .into_iter()
        .map(|(value, occurrences)| (value, occurrences.into_iter().count()))
        .collect_vec()
        .pipe(|occurences| {
            match occurences
                .into_iter()
                .map(|(_char, count)| count)
                .sorted()
                .collect_vec()
                .as_slice()
            {
                [5] => Ok(FiveOfAKind),
                [1, 4] => Ok(FourOfAKind),
                [2, 3] => Ok(FullHouse),
                [1, 1, 3] => Ok(ThreeOfAKind),
                [1, 2, 2] => Ok(TwoPair),
                [1, 1, 1, 2] => Ok(OnePair),
                [1, 1, 1, 1, 1] => Ok(HighCard),
                other => bail!("'{hand}' produced an unhandled variant: {other:?}"),
            }
        })
}

fn card_strength(card: char) -> usize {
    card.pipe(|value| {
        STRENGTHS
            .iter()
            .find_position(|&&e| e == value)
            .map(|(value, _)| STRENGTHS.len() - value)
            .expect("invalid hand")
    })
}

fn card_strength_part_2(card: char) -> usize {
    card.pipe(|value| {
        STRENGTHS_PART_2
            .iter()
            .find_position(|&&e| e == value)
            .map(|(value, _)| STRENGTHS_PART_2.len() - value)
            .expect("invalid hand")
    })
}

fn boxed<'a, T: Iterator<Item = I> + 'a, I: 'static>(
    iterator: T,
) -> Box<dyn (Iterator<Item = I>) + 'a> {
    Box::new(iterator)
}

fn combinations<'input>(
    possible_cards: &'input [&'input [char]],
) -> impl Iterator<Item = Vec<char>> + 'input {
    match possible_cards {
        [] => once(vec![]).pipe(boxed),
        [head, tail @ ..] => head
            .iter()
            .copied()
            .flat_map(|head| {
                combinations(tail).map(move |tail| tail.tap_mut(|tail| tail.insert(0, head)))
            })
            .pipe(boxed),
    }
}

fn main() -> Result<()> {
    #[allow(clippy::unit_arg)]
    INPUT
        .lines()
        .filter_map(|line| {
            line.split_once(' ')
                .map(|(hand, bet)| (hand, bet.parse::<usize>().expect("Bad bet")))
        })
        .collect::<Vec<_>>()
        .pipe(|lines| {
            lines
                .tap(|lines| {
                    lines
                        .iter()
                        .map(|(hand, bet)| {
                            (
                                (hand, bet),
                                (
                                    kind(hand).expect("bad hand"),
                                    hand.chars().map(card_strength).collect_vec(),
                                ),
                            )
                        })
                        .sorted_unstable_by_key(|(_, (kind, strengths))| {
                            (kind.to_owned(), strengths.clone())
                        })
                        .enumerate()
                        .map(|(i, v)| (i + 1, v))
                        .map(|(rank, ((_hand, bet), (_kind, _strongest)))| rank * bet)
                        .sum::<usize>()
                        .tap(|part_1| {
                            println!("part 1: {part_1:?}");
                        });
                })
                .tap(|lines| {
                    let variant_mapping = STRENGTHS_PART_2
                        .iter()
                        .map(|&c| match c {
                            'J' => (c, STRENGTHS_PART_2.to_vec()),
                            other => (c, [other].to_vec()),
                        })
                        .collect::<BTreeMap<_, _>>();
                    lines
                        .iter()
                        .map(|(hand, bet)| {
                            hand.chars()
                                .map(|c| variant_mapping.get(&c).unwrap().pipe(|v| v.as_slice()))
                                .collect_vec()
                                .pipe(|v| {
                                    combinations(&v)
                                        .map(move |new_hand| {
                                            (
                                                (hand, new_hand.clone(), bet),
                                                (
                                                    kind(&new_hand.into_iter().join(""))
                                                        .expect("bad hand"),
                                                    hand.chars()
                                                        .map(card_strength_part_2)
                                                        .collect_vec(),
                                                ),
                                            )
                                        })
                                        .max_by_key(|(_, (kind, strengths))| {
                                            (*kind, strengths.clone())
                                        })
                                })
                                .expect("no variants")
                        })
                        .sorted_unstable_by_key(|(_, (kind, strengths))| {
                            (kind.to_owned(), strengths.clone())
                        })
                        .enumerate()
                        .map(|(i, v)| (i + 1, v))
                        .inspect(|e| println!("{e:?}"))
                        .map(|(rank, ((_hand, _new_hand, &bet), (_kind, _strongest)))| rank * bet)
                        .sum::<usize>()
                        .tap(|part_2| {
                            println!("part 2: {part_2:?}");
                        });
                })
        });
    Ok(())
}

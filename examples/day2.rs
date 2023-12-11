use eyre::{Context, ContextCompat, Result};
use std::collections::BTreeMap;
use tap::prelude::*;

const INPUT: &str = include_str!("./day2.txt");

#[derive(Debug, PartialEq, Eq, PartialOrd, Clone, Ord)]
struct Color(String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
struct Subgame(BTreeMap<Color, usize>);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
struct Entry {
    color: Color,
    count: usize,
}

impl Subgame {
    pub fn entries(self) -> Vec<Entry> {
        self.0
            .into_iter()
            .map(|(color, count)| Entry { color, count })
            .collect()
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Clone)]
struct Game {
    id: usize,
    subgames: Vec<Subgame>,
}

fn filter_by_subgame_count<'game, 'color: 'game>(
    game: &'game Game,
    lookup_color: &'color Color,
    max_count: usize,
) -> Option<&'game Game> {
    match game
        .subgames
        .iter()
        .flat_map(|subgame| {
            subgame
                .0
                .iter()
                .filter_map(|(color, &count)| color.eq(lookup_color).then_some(count))
        })
        .find(|count| count.gt(&max_count))
    {
        Some(_count) => None,
        None => Some(game),
    }
}

fn fewest_cubes<'game, 'color: 'game>(game: &'game Game, lookup_color: &'color Color) -> usize {
    game.subgames
        .iter()
        .flat_map(|subgame| subgame.clone().entries())
        .filter_map(|e| (&e.color == lookup_color).then_some(e.count))
        .max()
        .unwrap_or_default()
}

fn main() -> Result<()> {
    INPUT
        .lines()
        .filter(|l| !l.is_empty())
        .map(|line| {
            line.split_once(": ")
                .context("no :")
                .and_then(|(header, subgames)| {
                    header
                        .split_once(' ')
                        .context("no game id")
                        .and_then(|(_game, id)| -> Result<usize> { id.parse().context("bad id") })
                        .and_then(|id| {
                            subgames
                                .split("; ")
                                .map(|colors| {
                                    colors
                                        .split(", ")
                                        .map(|color| {
                                            color
                                                .split_once(' ')
                                                .context("bad color")
                                                .and_then(|(count, color)| {
                                                    count.parse().context("Bad count").map(
                                                        |count| Entry {
                                                            color: Color(color.to_owned()),
                                                            count,
                                                        },
                                                    )
                                                })
                                                .with_context(|| {
                                                    format!("parsing color: '{color}'")
                                                })
                                        })
                                        .collect::<Result<Vec<_>>>()
                                        .context("reading entries")
                                        .map(|entries| {
                                            Subgame(
                                                entries
                                                    .into_iter()
                                                    .map(|Entry { color, count }| (color, count))
                                                    .collect(),
                                            )
                                        })
                                        .with_context(|| format!("parsing colors: '{colors}'"))
                                })
                                .collect::<Result<Vec<_>>>()
                                .context("bad entries")
                                .map(|subgames| Game { id, subgames })
                        })
                })
                .with_context(|| format!("parsing line: '{line}'"))
        })
        .collect::<Result<Vec<_>>>()?
        .tap(|games| {
            let red = Color("red".to_owned());
            let blue = Color("blue".to_owned());
            let green = Color("green".to_owned());

            games
                .iter()
                .filter_map(|game| filter_by_subgame_count(game, &red, 12))
                .filter_map(|game| filter_by_subgame_count(game, &green, 13))
                .filter_map(|game| filter_by_subgame_count(game, &blue, 14))
                .map(|game| game.id)
                .sum::<usize>()
                .tap(|sum| {
                    println!("day 1: {sum}");
                });

            games
                .iter()
                .map(|game| {
                    [&red, &blue, &green]
                        .into_iter()
                        .map(|color| fewest_cubes(game, color))
                        .product::<usize>()
                        .tap(|power| println!("power: {power}, game: {game:?}"))
                })
                .sum::<usize>()
                .tap(|day2| println!("day 2: {day2}"));
        });
    Ok(())
}

use std::{
    cell::RefCell,
    collections::{BTreeSet, HashSet},
    fmt::Display,
    iter::{once, successors},
    ops::Div,
    rc::Rc,
};

use eyre::ContextCompat;
use itertools::Itertools;
use tap::{Pipe as _, Tap as _};

const INPUT: &str = include_str!("./day10.txt");

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Tile {
    ///  | is a vertical pipe connecting north and south.
    ///  - is a horizontal pipe connecting east and west.
    ///  L is a 90-degree bend connecting north and east.
    ///  J is a 90-degree bend connecting north and west.
    ///  7 is a 90-degree bend connecting south and west.
    ///  F is a 90-degree bend connecting south and east.
    Pipe([Direction; 2]),
    ///  . is ground; there is no pipe in this tile.
    Ground,
    ///  S is the starting position of the animal;
    /// there is a pipe on this tile, but your sketch doesn't show what shape the pipe has.
    Start,
}
impl std::fmt::Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pipe([Direction::North, Direction::South]) => '┃',
            Self::Pipe([Direction::West, Direction::East]) => '━',
            Self::Pipe([Direction::North, Direction::East]) => '┗',
            Self::Pipe([Direction::North, Direction::West]) => '┛',
            Self::Pipe([Direction::South, Direction::West]) => '┓',
            Self::Pipe([Direction::South, Direction::East]) => '┏',
            Self::Pipe(_) => unreachable!(),
            Tile::Ground => '.',
            Tile::Start => 'S',
        }
        .pipe(|c| write!(f, "{c}"))
    }
}

impl std::fmt::Debug for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pipe([Direction::North, Direction::South]) => '|',
            Self::Pipe([Direction::West, Direction::East]) => '-',
            Self::Pipe([Direction::North, Direction::East]) => 'L',
            Self::Pipe([Direction::North, Direction::West]) => 'J',
            Self::Pipe([Direction::South, Direction::West]) => '7',
            Self::Pipe([Direction::South, Direction::East]) => 'F',
            Self::Pipe(_) => unreachable!(),
            Tile::Ground => '.',
            Tile::Start => 'S',
        }
        .pipe(|c| write!(f, "{c}"))
    }
}

impl From<char> for Tile {
    fn from(value: char) -> Self {
        use Direction::*;
        match value {
            '|' => Self::Pipe([North, South]),
            '-' => Self::Pipe([West, East]),
            'L' => Self::Pipe([North, East]),
            'J' => Self::Pipe([North, West]),
            '7' => Self::Pipe([South, West]),
            'F' => Self::Pipe([South, East]),
            '.' => Self::Ground,
            'S' => Self::Start,
            other => panic!("unexpected tile: '{other}'"),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Copy, PartialOrd, Ord, Hash)]
enum Direction {
    North,
    East,
    West,
    South,
}

impl Direction {
    const fn all() -> [Self; 4] {
        [Self::North, Self::South, Self::East, Self::West]
    }
}

impl std::fmt::Debug for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::North => "/\\",
            Direction::East => "->",
            Direction::West => "<-",
            Direction::South => "\\/",
        }
        .pipe(|v| write!(f, "[{v}]"))
    }
}

impl Direction {
    pub fn opposite(self) -> Self {
        match self {
            Direction::North => Self::South,
            Direction::East => Self::West,
            Direction::West => Self::East,
            Direction::South => Self::North,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Copy, PartialOrd, Ord, Hash)]
struct Position {
    column: u16,
    row: u16,
}

impl std::fmt::Debug for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { column, row } = *self;
        write!(f, "[{row}, {column}]")
    }
}

impl Position {
    fn direction(self, other: Self) -> Option<Direction> {
        self.neighbours()
            .iter()
            .find_map(|(position, direction)| (position == &other).then_some(*direction))
    }

    fn neighbour(self, direction: Direction) -> Self {
        let Self { column, row } = self;
        match direction {
            Direction::North => Self {
                column,
                row: row - 1,
            },
            Direction::South => Self {
                column,
                row: row + 1,
            },
            Direction::East => Self {
                column: column + 1,
                row,
            },
            Direction::West => Self {
                column: column - 1,
                row,
            },
        }
    }
    fn neighbours(self) -> [(Self, Direction); 4] {
        Direction::all().map(|direction| (self.neighbour(direction), direction))
    }
    fn neighbours_with_diagonals(self) -> [Self; 8] {
        let Self { column, row } = self;
        [
            Self {
                column,
                row: row - 1,
            },
            Self {
                column,
                row: row + 1,
            },
            Self {
                column: column + 1,
                row,
            },
            Self {
                column: column + 1,
                row: row - 1,
            },
            Self {
                column: column + 1,
                row: row + 1,
            },
            Self {
                column: column - 1,
                row,
            },
            Self {
                column: column - 1,
                row: row - 1,
            },
            Self {
                column: column - 1,
                row: row + 1,
            },
        ]
    }
}

struct Input(Vec<Vec<Tile>>);

impl std::fmt::Debug for Input {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.iter().try_for_each(|line| writeln!(f, "{line:?}"))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct PositionedTile {
    pub tile: Tile,
    pub position: Position,
}

impl Input {
    fn get(&self, position @ Position { column, row }: Position) -> Option<PositionedTile> {
        self.0
            .get(row as usize)
            .and_then(|row| row.get(column as usize))
            .map(Clone::clone)
            .map(|tile| PositionedTile { tile, position })
    }

    fn get_neighbour(&self, position: Position, direction: Direction) -> Option<PositionedTile> {
        position
            .neighbour(direction)
            .pipe(|position| self.get(position))
    }
    fn rows(&self) -> impl Iterator<Item = (u16, &[Tile])> + '_ {
        self.0
            .iter()
            .enumerate()
            .map(|(row, v)| (row as _, v.as_slice()))
    }
    fn all(&self) -> impl Iterator<Item = PositionedTile> + '_ {
        self.rows().flat_map(move |(row, tiles)| {
            tiles
                .iter()
                .copied()
                .enumerate()
                .map(move |(column, tile)| PositionedTile {
                    tile,
                    position: Position {
                        row,
                        column: column as _,
                    },
                })
        })
    }

    fn starts(&self) -> impl Iterator<Item = PositionedTile> + '_ {
        self.all()
            .filter(|PositionedTile { tile, .. }| matches!(tile, Tile::Start))
    }

    fn neighbours(
        &self,
        position: Position,
    ) -> impl Iterator<Item = (PositionedTile, Direction)> + '_ {
        position
            .neighbours()
            .into_iter()
            .flat_map(|(position, direction)| {
                self.get(position)
                    .map(|positioned_tile| (positioned_tile, direction))
            })
    }
    fn neighbours_with_diagonals(
        &self,
        position: Position,
    ) -> impl Iterator<Item = PositionedTile> + '_ {
        position
            .neighbours_with_diagonals()
            .into_iter()
            .flat_map(|position| self.get(position))
    }
}

fn popped_array<T: PartialEq>(array: [T; 2], element: &T) -> Option<T> {
    array
        .contains(element)
        .then_some(())
        .and_then(|_| array.into_iter().find(|needle| needle != element))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BoundingBox {
    min: Position,
    max: Position,
}

#[derive(Default, Clone, Copy)]
enum Color {
    Red,
    Yellow,
    #[default]
    Purple,
    Cyan,
}

impl Color {
    fn code(self) -> u16 {
        match self {
            Self::Red => 91,
            Self::Yellow => 93,
            Self::Purple => 95,
            Self::Cyan => 96,
        }
    }
    fn start(self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\x1b[{}m", self.code())
    }
    fn end(self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\x1b[0m")
    }
}

struct Colored<T> {
    inner: T,
    color: Color,
}

impl<T> std::fmt::Display for Colored<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.color.start(f)?;
        self.inner.fmt(f)?;
        self.color.end(f)?;
        Ok(())
    }
}

trait ColoredExt: Sized {
    fn colored(self, color: Color) -> Colored<Self>;
}

impl<T: Sized> ColoredExt for T {
    fn colored(self, color: Color) -> Colored<Self> {
        Colored { inner: self, color }
    }
}

impl<T> std::fmt::Debug for Colored<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.color.start(f)?;
        self.inner.fmt(f)?;
        self.color.end(f)?;
        Ok(())
    }
}

impl Tile {
    fn entered_from(self, direction: Direction) -> Option<(Self, Option<Direction>)> {
        match self {
            Tile::Pipe(connections) => popped_array(connections, &direction.opposite())
                .map(|remaining| (self, Some(remaining))),
            Tile::Ground => None,
            Tile::Start => Some((self, None)),
        }
    }
}

impl Direction {
    pub fn moved(self, in_direction: Turn) -> Self {
        match self {
            Direction::North => match in_direction {
                Turn::Straight => Direction::North,
                Turn::Left => Direction::West,
                Turn::Right => Direction::East,
            },
            Direction::East => match in_direction {
                Turn::Straight => Direction::East,
                Turn::Left => Direction::North,
                Turn::Right => Direction::South,
            },
            Direction::West => match in_direction {
                Turn::Straight => Direction::West,
                Turn::Left => Direction::South,
                Turn::Right => Direction::North,
            },
            Direction::South => match in_direction {
                Turn::Straight => Direction::West,
                Turn::Left => Direction::East,
                Turn::Right => Direction::West,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum Turn {
    Straight,
    Left,
    Right,
}

impl PositionedTile {
    fn entered_from(self, direction: Direction) -> Option<(Self, Option<Direction>)> {
        self.tile
            .entered_from(direction)
            .map(|(_, direction)| (self, direction))
    }
}

fn main() {
    INPUT
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| line.chars().map(Tile::from).collect_vec())
        .collect_vec()
        .pipe(Input)
        .tap(|input| {
            input
                .starts()
                .next()
                .unwrap()
                .pipe(|start| {
                    successors(Some((start, Option::<Direction>::None)), |&(current, direction)| {
                        input
                            .neighbours(current.position)
                            .find_map(|(neighbour, in_direction)| match (direction, current.tile) {
                                (None, Tile::Start) => input
                                    .neighbours(current.position)
                                    .find_map(|(neighbour, direction)| neighbour.entered_from(direction)),
                                (None, _) => unreachable!(),
                                (Some(direction), Tile::Pipe(_)) => input
                                    .get_neighbour(current.position, direction)
                                    .and_then(|positioned_tile| positioned_tile.entered_from(direction)),
                                (Some(_), Tile::Start) => Some((neighbour, Some(in_direction))),
                                (Some(_), Tile::Ground) => None,
                            })
                            .expect("this must be a closed loop")
                            .pipe(Some)
                    })
                    .inspect(|e| println!("jumped to {e:?}"))
                    .enumerate()
                    .take_while(|&state| !matches!(state, (1.., (PositionedTile { tile: Tile::Start, .. }, _,),)))
                })
                .collect_vec()
                .tap(|the_loop| {
                    println!("part 1: {}", the_loop.len().div(2));
                })
                .tap(|the_loop| {
                    the_loop
                        .iter()
                        .map(|(_, (PositionedTile { position, .. }, _))| *position)
                        .collect::<HashSet<_>>()
                        .tap(|borders| {
                            let tiles = || the_loop.iter().map(|(_, (p, _))| p);
                            let turns = || {
                                tiles().zip(tiles().cycle().skip(1)).map(|(prev, next)| {
                                    prev.position
                                        .direction(next.position)
                                        .with_context(|| format!("checking position between next: {next:?} and prev: {prev:?}"))
                                        .expect("invalid position checked")
                                        .pipe(|direction| (prev, direction))
                                })
                            };

                            Lookup::default().pipe(|lookup| {
                                turns()
                                    .filter_map(|(positioned, direction)| {
                                        input.get_neighbour(positioned.position, direction.moved(Turn::Right))
                                    })
                                    .filter(|v| !borders.contains(&v.position))
                                    .map(|start| {
                                        all_touching(input, start, lookup.clone(), |tile| {
                                            (!borders.contains(&tile.position)).then_some(tile)
                                        })
                                        .collect::<BTreeSet<_>>()
                                    })
                                    .unique()
                                    .flatten()
                                    .collect::<BTreeSet<_>>()
                                    .pipe(|contained| {
                                        contained.tap(|contained| {
                                            println!("contained: {contained:#?}");
                                            input
                                                .all()
                                                .group_by(
                                                    |PositionedTile {
                                                         position: Position { row, .. },
                                                         ..
                                                     }| *row,
                                                )
                                                .into_iter()
                                                .map(|(_, row)| {
                                                    row.into_iter()
                                                        .map(|positioned @ PositionedTile { tile, position }| {
                                                            (positioned.tile == Tile::Start)
                                                                .then_some(Color::Cyan)
                                                                .or_else(|| {
                                                                    borders
                                                                        .contains(&position)
                                                                        .then_some(Color::Yellow)
                                                                        .or_else(|| {
                                                                            contained
                                                                                .contains(&positioned)
                                                                                .then_some(Color::Red)
                                                                        })
                                                                })
                                                                .unwrap_or_default()
                                                                .pipe(|color| tile.colored(color).to_string())
                                                        })
                                                        .join("")
                                                })
                                                .join("\n")
                                                .pipe(|debugger| {
                                                    println!("{debugger}");
                                                });
                                            println!("part 2: {}", contained.len());
                                        });
                                    })
                            })
                        });
                });
        });
}

type Lookup = Rc<RefCell<HashSet<PositionedTile>>>;

fn all_touching<'input, C>(
    input: &'input Input,
    tile: PositionedTile,
    visited: Lookup,
    condition: C,
) -> impl Iterator<Item = PositionedTile> + 'input
where
    C: Fn(PositionedTile) -> Option<PositionedTile> + Clone + Copy + 'input,
{
    let already_checked = {
        let visited = visited.clone();
        move |p: PositionedTile| visited.borrow().contains(&p)
    };
    let set_checked = {
        {
            let visited = visited.clone();
            move |p: PositionedTile| {
                visited.borrow_mut().insert(p);
            }
        }
    };
    input
        .neighbours_with_diagonals(tile.position)
        .chain(once(tile))
        .filter(move |c| !already_checked(*c))
        .inspect(|n| println!("testing neighbours of {n:?}"))
        .filter_map(condition)
        .flat_map(move |tile| {
            // let set_checked = set_checked.clone();
            once(tile).chain(all_touching(input, tile, visited.clone(), condition))
        })
        .inspect(move |v| set_checked(*v))
        .pipe(boxed)
}

fn boxed<'a, T: Iterator<Item = I> + 'a, I: 'static>(
    iterator: T,
) -> Box<dyn (Iterator<Item = I>) + 'a> {
    Box::new(iterator)
}

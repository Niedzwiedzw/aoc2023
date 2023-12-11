use tap::prelude::*;

const INPUT: &str = include_str!("./day1.txt");

const DIGITS: &[(u32, &str)] = &[
    (0, "0"),
    (1, "1"),
    (2, "2"),
    (3, "3"),
    (4, "4"),
    (5, "5"),
    (6, "6"),
    (7, "7"),
    (8, "8"),
    (9, "9"),
    (0, "zero"),
    (1, "one"),
    (2, "two"),
    (3, "three"),
    (4, "four"),
    (5, "five"),
    (6, "six"),
    (7, "seven"),
    (8, "eight"),
    (9, "nine"),
];

fn next_digit(input: &str) -> Option<u32> {
    DIGITS
        .iter()
        .copied()
        .find_map(|(i, digit)| input.strip_prefix(digit).map(|_| (i)))
}

fn main() {
    INPUT
        .lines()
        .filter(|l| !l.is_empty())
        .filter_map(|line| {
            line.chars()
                .filter_map(|c| c.to_digit(10))
                .collect::<Vec<_>>()
                .pipe(|digits| digits.first().copied().zip(digits.last().copied()))
        })
        .map(|(first, last)| first * 10 + last)
        .sum::<u32>()
        .tap(|v| println!("part 1: {v}"));

    INPUT
        .lines()
        .filter(|l| !l.is_empty())
        .filter_map(|line| {
            (0..line.len())
                .filter_map(|start| line.split_at(start).1.pipe(next_digit))
                .collect::<Vec<_>>()
                .pipe(|v| v.first().copied().zip(v.last().copied()))
        })
        .map(|(first, last)| first * 10 + last)
        .sum::<u32>()
        .tap(|v| println!("\npart 2: {v}"));
}

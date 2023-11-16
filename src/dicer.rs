//! Implments dicer function, and parsing based on [Nom](https://docs.rs/nom).
//!
//! ## Expression Syntax
//!
//! An token enclosed in `=` specifies parsing a primitive type of Rust.
//!
//!
//! ```
//! expr -> term "=>" =u32=;
//!
//! term -> factor ( ("+" | "-") factor )*;
//! factor -> dice_or_int ( ("+" | "-") dice_or_int )*;
//!
//! dice_or_int => ( ( =u32= ('d' | 'D') =u32= ) | =i32= );
//! ```

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::character::complete::multispace0;
use nom::character::complete::{i32, u32};
use nom::combinator::cut;
use nom::combinator::{eof, map, opt};
use nom::multi::fold_many0;
use nom::sequence::{delimited, pair, preceded, separated_pair, terminated};
use nom::{bytes::complete::tag_no_case, IResult};
use std::str::FromStr;

use crate::FuukaBotError;

/// A dice candicate.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct DiceCandidate {
    /// The expression of the dice roll.
    pub expr: Expr,
    /// An optional target for the dice roll.
    pub target: Option<u32>,
}

impl FromStr for DiceCandidate {
    type Err = nom::error::Error<String>;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        use nom::error::Error;
        use nom::Finish;
        match terminated(expr, eof)(input).finish() {
            Ok((_, result)) => Ok(result),
            Err(Error { input, code }) => Err(Error {
                input: input.to_string(),
                code,
            }),
        }
    }
}

/// A dice roll.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Dice {
    /// How many times to roll.
    pub count: u32,
    /// How many sides the dice roll has.
    pub sides: u32,
}

/// A unit in expression that represents either a dice roll or an interger.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum DiceOrInt {
    /// A dice roll.
    Dice(Dice),
    /// An oridary interger.
    Int(i32),
}

/// An expression.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Expr {
    /// A unit in expression that represents either a dice roll or an interger.
    DiceOrInt(DiceOrInt),
    /// A binary operator.
    BinOp {
        /// Left hand expression.
        lhs: Box<Expr>,
        /// The operator.
        op: Op,
        /// Right hand expression.
        rhs: Box<Expr>,
    },
}

impl Expr {
    /// Evaluate the expression.
    pub fn eval(self) -> anyhow::Result<i32> {
        match self {
            Self::DiceOrInt(result) => match result {
                DiceOrInt::Dice(dice) => {
                    let Dice { count, sides } = dice;
                    Ok((fastrand::u32(1..=sides) * count) as i32)
                }
                DiceOrInt::Int(num) => Ok(num),
            },
            Self::BinOp { lhs, op, rhs } => {
                match op {
                    Op::Add => Ok(i32::checked_add(lhs.eval()?, rhs.eval()?)
                        .ok_or(FuukaBotError::MathOverflow)?),
                    Op::Sub => Ok(i32::checked_sub(lhs.eval()?, rhs.eval()?)
                        .ok_or(FuukaBotError::MathOverflow)?),
                    Op::Mul => Ok(i32::checked_mul(lhs.eval()?, rhs.eval()?)
                        .ok_or(FuukaBotError::MathOverflow)?),
                    Op::Div => Ok(i32::checked_div(lhs.eval()?, rhs.eval()?)
                        .ok_or(FuukaBotError::DivByZero)?),
                }
            }
        }
    }
}

/// The supported operator.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Op {
    /// `+`.
    Add,
    /// `-`.
    Sub,
    /// `*`.
    Mul,
    /// `/`.
    Div,
}

fn expr(input: &str) -> IResult<&str, DiceCandidate> {
    map(
        pair(
            term,
            opt(preceded(
                tag("=>"),
                cut(delimited(multispace0, u32, multispace0)),
            )),
        ),
        |(expr, target)| DiceCandidate { expr, target },
    )(input)
}

// XXX: Allow this
// Returning directly gives this error:
// error[E0597]: `result` does not live long enough
//    --> src/dicer.rs:103:12
//     |
// 92  | /     fold_many0(
// 93  | |         alt((
// 94  | |             |input| {
// 95  | |                 let (remaining, mul) = preceded(char('*'), dice_or_int)(input)?;
// ...   |
// 103 | |         || result.clone(),
//     | |         -- ^^^^^^ borrowed value does not live long enough
//     | |         |
//     | |         value captured here
// ...   |
// 111 | |         },
// 112 | |     )(remaining)
//     | |_____- a temporary with access to the borrow is created here ...
// 113 |   }
//     |   -
//     |   |
//     |   `result` dropped here while still borrowed
//     |   ... and the borrow might be used here, when that temporary is dropped and runs the destructor for type `impl FnMut(&str) -> Result<(&str, Expr), nom::Err<nom::error::Error<&str>>>`
//     |
//     = note: the temporary is part of an expression at the end of a block;
//             consider forcing this temporary to be dropped sooner, before the block's local variables are dropped
// help: for example, you could save the expression's value in a new local variable `x` and then make `x` be the expression at the end of the block
//     |
// 92  ~     let x = fold_many0(
// 93  |         alt((
//   ...
// 111 |         },
// 112 ~     )(remaining); x
//     |
#[allow(clippy::let_and_return)]
fn factor(input: &str) -> IResult<&str, Expr> {
    let (remaining, result) = dice_or_int(input)?;
    let result = fold_many0(
        alt((
            |input| {
                let (remaining, mul) = preceded(char('*'), cut(dice_or_int))(input)?;
                Ok((remaining, (Op::Mul, mul)))
            },
            |input| {
                let (remaining, div) = preceded(char('/'), cut(dice_or_int))(input)?;
                Ok((remaining, (Op::Div, div)))
            },
        )),
        || result.clone(),
        |acc, pair| {
            let (op, expr) = pair;
            Expr::BinOp {
                lhs: Box::new(acc),
                op,
                rhs: Box::new(expr),
            }
        },
    )(remaining);
    result
}

// XXX: Allow this
#[allow(clippy::let_and_return)]
fn term(input: &str) -> IResult<&str, Expr> {
    let (remaining, result) = factor(input)?;
    let result = fold_many0(
        alt((
            |input| {
                let (remaining, expr) = preceded(char('+'), cut(factor))(input)?;
                Ok((remaining, (Op::Add, expr)))
            },
            |input| {
                let (remaining, expr) = preceded(char('-'), cut(factor))(input)?;
                Ok((remaining, (Op::Sub, expr)))
            },
        )),
        || result.clone(),
        |acc, pair| {
            let (op, expr) = pair;
            Expr::BinOp {
                lhs: Box::new(acc),
                op,
                rhs: Box::new(expr),
            }
        },
    )(remaining);
    result
}

fn dice_or_int(input: &str) -> IResult<&str, Expr> {
    map(
        delimited(
            multispace0,
            alt((
                map(
                    separated_pair(opt(u32), tag_no_case("d"), cut(u32)),
                    |(count, sides)| {
                        DiceOrInt::Dice(Dice {
                            count: count.unwrap_or(1),
                            sides,
                        })
                    },
                ),
                map(i32, DiceOrInt::Int),
            )),
            multispace0,
        ),
        Expr::DiceOrInt,
    )(input)
}

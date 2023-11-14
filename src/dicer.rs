use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::character::complete::multispace0;
use nom::character::complete::{i32, u32};
use nom::combinator::{eof, map, opt};
use nom::multi::fold_many0;
use nom::sequence::{delimited, pair, preceded, separated_pair, terminated};
use nom::{bytes::complete::tag_no_case, IResult};
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct DiceCandidate {
    pub expr: Expr,
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

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Dice {
    pub count: u32,
    pub sides: u32,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum DiceOrInt {
    Dice(Dice),
    Int(i32),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Expr {
    DiceOrInt(DiceOrInt),
    BinOp {
        lhs: Box<Expr>,
        op: Op,
        rhs: Box<Expr>,
    },
}

impl Expr {
    pub fn eval(self) -> i32 {
        match self {
            Self::DiceOrInt(result) => match result {
                DiceOrInt::Dice(dice) => {
                    let Dice { count, sides } = dice;
                    (fastrand::u32(1..=sides) * count) as i32
                }
                DiceOrInt::Int(num) => num,
            },
            Self::BinOp { lhs, op, rhs } => match op {
                Op::Add => lhs.eval() + rhs.eval(),
                Op::Sub => lhs.eval() - rhs.eval(),
                Op::Mul => lhs.eval() * rhs.eval(),
                Op::Div => lhs.eval() / rhs.eval(),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

fn expr(input: &str) -> IResult<&str, DiceCandidate> {
    map(
        pair(
            term,
            opt(preceded(
                tag("=>"),
                delimited(multispace0, u32, multispace0),
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
                let (remaining, mul) = preceded(char('*'), dice_or_int)(input)?;
                Ok((remaining, (Op::Mul, mul)))
            },
            |input| {
                let (remaining, div) = preceded(char('/'), dice_or_int)(input)?;
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
    let (remaining, result) = dice_or_int(input)?;
    let result = fold_many0(
        alt((
            |input| {
                let (remaining, expr) = preceded(char('+'), factor)(input)?;
                Ok((remaining, (Op::Add, expr)))
            },
            |input| {
                let (remaining, expr) = preceded(char('-'), factor)(input)?;
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
                    separated_pair(opt(u32), tag_no_case("d"), u32),
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

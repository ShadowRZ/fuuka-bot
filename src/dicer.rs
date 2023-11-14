use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::character::complete::multispace0;
use nom::character::complete::{i32, u32};
use nom::combinator::{eof, map, opt};
use nom::multi::fold_many0;
use nom::sequence::{delimited, pair, preceded, separated_pair, terminated};
use nom::{bytes::complete::tag_no_case, IResult};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct DiceCandidate {
    pub expr: Expr,
    pub target: Option<u32>,
}

impl<'input> TryFrom<&'input str> for DiceCandidate {
    type Error = nom::Err<nom::error::Error<&'input str>>;

    fn try_from(input: &'input str) -> Result<Self, <Self as TryFrom<&'input str>>::Error> {
        let (_, result) = terminated(expr, eof)(input)?;
        Ok(result)
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
    pub fn eval(self, rng: &mut fastrand::Rng) -> i32 {
        match self {
            Self::DiceOrInt(result) => match result {
                DiceOrInt::Dice(dice) => {
                    let Dice { count, sides } = dice;
                    (rng.u32(1..=sides) * count) as i32
                }
                DiceOrInt::Int(num) => num,
            },
            Self::BinOp { lhs, op, rhs } => match op {
                Op::Add => lhs.eval(rng) + rhs.eval(rng),
                Op::Sub => lhs.eval(rng) - rhs.eval(rng),
                Op::Mul => lhs.eval(rng) * rhs.eval(rng),
                Op::Div => lhs.eval(rng) / rhs.eval(rng),
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

use super::{BinaryOperator, Expression, Program, Statement};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    character::complete::{alpha1, alphanumeric0, digit1, multispace0, space1},
    combinator::{map, map_res, opt, recognize},
    multi::separated_list1,
    sequence::{delimited, preceded, tuple},
    IResult,
};
use std::str::FromStr;

pub fn parse(input: &str) -> IResult<&str, Program> {
    let (input, program) = parse_program(input)?;

    Ok((input, program))
}

fn parse_line_number(input: &str) -> IResult<&str, u32> {
    map_res(digit1, u32::from_str)(input)
}

fn parse_number(input: &str) -> IResult<&str, i32> {
    map_res(digit1, i32::from_str)(input)
}

fn parse_string_literal(input: &str) -> IResult<&str, String> {
    let (input, content) = delimited(tag("\""), take_while(|c: char| c != '"'), tag("\""))(input)?;

    Ok((input, content.to_string()))
}

// variables are sequences of alphabetic characters, optionally followed by a dollar sign, to indicate a string variable
fn parse_variable(input: &str) -> IResult<&str, String> {
    let (input, name) = recognize(tuple((alpha1, alphanumeric0, opt(tag("$")))))(input)?;
    Ok((input, name.to_string()))
}

fn parse_factor(input: &str) -> IResult<&str, Expression> {
    let (input, expr) = alt((
        map(parse_number, Expression::NumberLiteral),
        map(move |i| parse_variable(i), Expression::Variable),
        map(move |i| parse_string_literal(i), Expression::StringLiteral),
        move |i| parse_parens_expression(i),
    ))(input)?;

    Ok((input, expr))
}

fn parse_parens_expression(input: &str) -> IResult<&str, Expression> {
    let (input, expr) = delimited(
        tag("("),
        preceded(multispace0, move |i| {
            let (i, expr) = parse_expression(i)?;
            Ok((i, expr))
        }),
        tag(")"),
    )(input)?;

    Ok((input, expr))
}

fn parse_mul_div(input: &str) -> IResult<&str, Expression> {
    fn parse_mul_div_sign(input: &str) -> IResult<&str, BinaryOperator> {
        alt((
            map(tag("*"), |_| BinaryOperator::Mul),
            map(tag("/"), |_| BinaryOperator::Div),
        ))(input)
    }

    let (input, left) = parse_factor(input)?;

    // try to parse a multiplication or division operator
    let (input, right) = opt(preceded(multispace0, move |i| {
        let (i, op) = parse_mul_div_sign(i)?;
        let (i, _) = multispace0(i)?;
        let (i, right) = parse_mul_div(i)?;

        Ok((i, (op, right)))
    }))(input)?;

    // if we didn't find an operator, return the left expression
    if let Some((op, right)) = right {
        let left = Box::new(left);
        let right = Box::new(right);
        Ok((input, Expression::Binary { left, op, right }))
    } else {
        Ok((input, left))
    }
}

fn parse_add_sub(input: &str) -> IResult<&str, Expression> {
    fn parse_add_sub_sign(input: &str) -> IResult<&str, BinaryOperator> {
        alt((
            map(tag("+"), |_| BinaryOperator::Add),
            map(tag("-"), |_| BinaryOperator::Sub),
        ))(input)
    }

    let (input, left) = parse_mul_div(input)?;

    // try to parse an addition or subtraction operator
    let (input, right) = opt(preceded(multispace0, move |i| {
        let (i, op) = parse_add_sub_sign(i)?;
        let (i, _) = multispace0(i)?;
        let (i, right) = parse_add_sub(i)?;

        Ok((i, (op, right)))
    }))(input)?;

    // if we didn't find an operator, return the left expression
    if let Some((op, right)) = right {
        let left = Box::new(left);
        let right = Box::new(right);
        Ok((input, Expression::Binary { left, op, right }))
    } else {
        Ok((input, left))
    }
}

fn parse_comparison(input: &str) -> IResult<&str, Expression> {
    fn parse_comparison_sign(input: &str) -> IResult<&str, BinaryOperator> {
        alt((
            map(tag("="), |_| BinaryOperator::Eq),
            map(tag("<>"), |_| BinaryOperator::Ne),
            map(tag("<="), |_| BinaryOperator::Le),
            map(tag(">="), |_| BinaryOperator::Ge),
            map(tag("<"), |_| BinaryOperator::Lt),
            map(tag(">"), |_| BinaryOperator::Gt),
        ))(input)
    }

    let (input, left) = parse_add_sub(input)?;

    // try to parse a comparison operator
    let (input, right) = opt(preceded(multispace0, move |i| {
        let (i, op) = parse_comparison_sign(i)?;
        let (i, _) = multispace0(i)?;
        let (i, right) = parse_add_sub(i)?;

        Ok((i, (op, right)))
    }))(input)?;

    // if we didn't find an operator, return the left expression
    if let Some((op, right)) = right {
        let left = Box::new(left);
        let right = Box::new(right);
        Ok((input, Expression::Binary { left, op, right }))
    } else {
        Ok((input, left))
    }
}

fn parse_and_or(input: &str) -> IResult<&str, Expression> {
    fn parse_and_or_sign(input: &str) -> IResult<&str, BinaryOperator> {
        alt((
            map(tag("AND"), |_| BinaryOperator::And),
            map(tag("OR"), |_| BinaryOperator::Or),
        ))(input)
    }

    let (input, left) = parse_comparison(input)?;

    // try to parse an AND or OR operator
    let (input, right) = opt(preceded(multispace0, move |i| {
        let (i, op) = parse_and_or_sign(i)?;
        let (i, _) = multispace0(i)?;
        let (i, right) = parse_comparison(i)?;

        Ok((i, (op, right)))
    }))(input)?;

    // if we didn't find an operator, return the left expression
    if let Some((op, right)) = right {
        let left = Box::new(left);
        let right = Box::new(right);
        Ok((input, Expression::Binary { left, op, right }))
    } else {
        Ok((input, left))
    }
}

fn parse_expression(input: &str) -> IResult<&str, Expression> {
    parse_and_or(input)
}

fn parse_let(input: &str) -> IResult<&str, Statement> {
    // LET keyoword is optional
    let (input, _) = opt(tag("LET"))(input)?;
    let (input, _) = multispace0(input)?;
    let (input, variable) = parse_variable(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("=")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, expression) = parse_expression(input)?;

    Ok((
        input,
        Statement::Let {
            variable,
            expression,
        },
    ))
}

fn parse_print(input: &str) -> IResult<&str, Statement> {
    let (input, _) = tag("PRINT")(input)?;
    let (input, _) = space1(input)?;

    // PRINT can be followed by multiple expressions or string literals separated by semicolons
    let (input, content) = separated_list1(
        // semi-colon followed by optional whitespace
        delimited(tag(";"), multispace0, multispace0),
        move |i| parse_expression(i),
    )(input)?;

    Ok((input, Statement::Print { content }))
}

// INPUT "name"; NAME$
fn parse_input(input: &str) -> IResult<&str, Statement> {
    let (input, _) = tag("INPUT")(input)?;
    let (input, _) = space1(input)?;

    // get optional prompt
    let (input, prompt) = opt(move |i| parse_expression(i))(input)?;

    // if a promtp was found, skip the semicolon and optional whitespace
    let (input, _) = opt(delimited(tag(";"), multispace0, multispace0))(input)?;
    let (input, variable) = parse_variable(input)?;

    Ok((input, Statement::Input { prompt, variable }))
}

fn parse_goto(input: &str) -> IResult<&str, Statement> {
    let (input, _) = tag("GOTO")(input)?;
    let (input, _) = space1(input)?;
    let (input, line_number) = parse_line_number(input)?;

    Ok((input, Statement::Goto { line_number }))
}

fn parse_gosub(input: &str) -> IResult<&str, Statement> {
    let (input, _) = tag("GOSUB")(input)?;
    let (input, _) = space1(input)?;
    let (input, line_number) = parse_line_number(input)?;

    Ok((input, Statement::GoSub { line_number }))
}

fn parse_return(input: &str) -> IResult<&str, Statement> {
    let (input, _) = tag("RETURN")(input)?;

    Ok((input, Statement::Return))
}

fn parse_if(input: &str) -> IResult<&str, Statement> {
    let (input, _) = tag("IF")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, condition) = parse_expression(input)?;
    let (input, _) = opt(preceded(space1, tag("THEN")))(input)?;
    let (input, _) = space1(input)?;

    let (input, then) = parse_statement(input)?;
    let then = Box::new(then);

    let (input, else_) = opt(preceded(space1, move |i| {
        let (i, _) = tag("ELSE")(i)?;
        let (i, _) = space1(i)?;
        let (i, else_) = parse_statement(i)?;
        let else_ = Box::new(else_);
        Ok((i, else_))
    }))(input)?;

    Ok((
        input,
        Statement::If {
            condition,
            then,
            else_,
        },
    ))
}

fn parse_for(input: &str) -> IResult<&str, Statement> {
    let (input, _) = tag("FOR")(input)?;
    let (input, _) = space1(input)?;
    let (input, variable) = parse_variable(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("=")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, from) = parse_expression(input)?;
    let (input, _) = space1(input)?;
    let (input, _) = tag("TO")(input)?;
    let (input, _) = space1(input)?;
    let (input, to) = parse_expression(input)?;
    let (input, step) = opt(preceded(space1, move |i| {
        let (i, _) = tag("STEP")(i)?;
        let (i, _) = space1(i)?;
        let (i, step) = parse_expression(i)?;
        Ok((i, step))
    }))(input)?;

    Ok((
        input,
        Statement::For {
            variable,
            from,
            to,
            step,
        },
    ))
}

fn parse_next(input: &str) -> IResult<&str, Statement> {
    let (input, _) = tag("NEXT")(input)?;
    let (input, _) = space1(input)?;
    let (input, variable) = parse_variable(input)?;

    Ok((input, Statement::Next { variable }))
}

fn parse_end(input: &str) -> IResult<&str, Statement> {
    let (input, _) = tag("END")(input)?;

    Ok((input, Statement::End))
}

fn parse_atomic_statement(input: &str) -> IResult<&str, Statement> {
    alt((
        move |i| parse_let(i),
        move |i| parse_print(i),
        move |i| parse_input(i),
        move |i| parse_goto(i),
        move |i| parse_for(i),
        move |i| parse_next(i),
        move |i| parse_end(i),
        move |i| parse_gosub(i),
        move |i| parse_if(i),
        move |i| parse_return(i),
    ))(input)
}

fn parse_statement(input: &str) -> IResult<&str, Statement> {
    let (input, statements) = separated_list1(
        preceded(multispace0, tag(":")),
        preceded(multispace0, move |i| parse_atomic_statement(i)),
    )(input)?;

    if statements.len() == 1 {
        let statement = statements.into_iter().next().unwrap();
        Ok((input, statement))
    } else {
        Ok((input, Statement::Seq { statements }))
    }
}

// Comment lines start with REM
fn parse_comment(input: &str) -> IResult<&str, ()> {
    let (input, _) = tag("REM")(input)?;
    let (input, _) = take_while(|c: char| c != '\n')(input)?;

    Ok((input, ()))
}

fn parse_comment_line(input: &str) -> IResult<&str, ()> {
    let (input, _) = tuple((
        move |i| parse_line_number(i),
        space1,
        move |i| parse_comment(i),
    ))(input)?;

    Ok((input, ()))
}

fn parse_line(input: &str) -> IResult<&str, (u32, Statement)> {
    let (input, (number, _, statement)) = tuple((
        move |i| parse_line_number(i),
        space1,
        move |i| parse_statement(i),
    ))(input)?;

    Ok((input, (number, statement)))
}

fn parse_program(input: &str) -> IResult<&str, Program> {
    let mut program = Program::new();
    let mut input = input;

    // TODO: improve this loop
    while !input.is_empty() {
        let (new_input, _) = multispace0(input)?;
        if new_input.is_empty() {
            break;
        }

        let (new_input, _) = opt(move |i| parse_comment_line(i))(new_input)?;
        let (new_input, _) = multispace0(new_input)?;
        if new_input.is_empty() {
            break;
        }

        let (new_input, line) = parse_line(new_input)?;
        program.add_line(line.0, line.1);
        input = new_input;
    }

    Ok((input, program))
}

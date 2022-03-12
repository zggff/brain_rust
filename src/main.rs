use derive_more::{Display, Error};
use std::{env::args, fs, str::FromStr};

#[derive(Debug)]
pub enum Token {
    PointerShift(isize),
    ValueShift(i16),
    ValueOutput,
    ValueInput,
    Loop(Program),
}

#[derive(Debug)]
pub struct Program(Vec<Token>);

impl Program {
    pub fn parse(code: &str) -> Result<Program, ParseError> {
        // filter only acceptable brainfuck tokens
        let mut code = code
            .chars()
            .filter(|char| matches!(char, '+' | '-' | '<' | '>' | '.' | ',' | '[' | ']'));
        Program::parse_segment(&mut code, &mut 0)
    }
    fn parse_segment(
        iter: &mut impl Iterator<Item = char>,
        bracket_count: &mut usize,
    ) -> Result<Program, ParseError> {
        let mut program = Vec::new();
        while let Some(token) = iter.next() {
            let token = match token {
                '>' => {
                    // NOTE:To me this seems as the easiest solution to joining together multiple operands
                    let shift = if let Some(Token::PointerShift(shift)) = program.last() {
                        let shift = *shift;
                        program.pop();
                        shift
                    } else {
                        0
                    };
                    Token::PointerShift(shift + 1)
                }
                '<' => {
                    let shift = if let Some(Token::PointerShift(shift)) = program.last() {
                        let shift = *shift;
                        program.pop();
                        shift
                    } else {
                        0
                    };
                    Token::PointerShift(shift - 1)
                }
                '+' => {
                    let shift = if let Some(Token::ValueShift(shift)) = program.last() {
                        let shift = *shift;
                        program.pop();
                        shift
                    } else {
                        0
                    };
                    Token::ValueShift(shift + 1)
                }
                '-' => {
                    let shift = if let Some(Token::ValueShift(shift)) = program.last() {
                        let shift = *shift;
                        program.pop();
                        shift
                    } else {
                        0
                    };
                    Token::ValueShift(shift - 1)
                }
                '.' => Token::ValueOutput,
                ',' => Token::ValueInput,
                '[' => {
                    *bracket_count += 1;
                    Token::Loop(Program::parse_segment(iter, bracket_count)?)
                }
                ']' => {
                    return if *bracket_count > 0 {
                        *bracket_count -= 1;
                        Ok(Program(program))
                    } else {
                        Err(ParseError::MissingOpeningBracket)
                    }
                }
                _ => {
                    continue;
                }
            };
            program.push(token);
        }
        if *bracket_count == 0 {
            Ok(Program(program))
        } else {
            Err(ParseError::MissingClosingBracket)
        }
    }
}

impl FromStr for Program {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut code = s
            .chars()
            .filter(|char| matches!(char, '+' | '-' | '<' | '>' | '.' | ',' | '[' | ']'));
        Program::parse_segment(&mut code, &mut 0)
    }
}

#[derive(Error, Debug, Display)]
pub enum ParseError {
    MissingClosingBracket,
    MissingOpeningBracket,
    // UnsupportedSymbol,
}

pub fn execute(program: &Program, memory: &mut Vec<u8>, pointer: &mut usize) {
    for token in &program.0 {
        match token {
            &Token::PointerShift(shift) => {
                if shift.is_negative() {
                    let shift = -shift as usize;
                    assert!(*pointer >= shift);
                    *pointer -= shift
                } else {
                    *pointer += shift as usize
                }
            }
            &Token::ValueShift(shift) => {
                if shift.is_negative() {
                    let shift = -shift as u8;
                    memory[*pointer] = memory[*pointer].overflowing_sub(shift).0
                } else {
                    memory[*pointer] = memory[*pointer].overflowing_add(shift as u8).0
                }
            }
            Token::ValueOutput => {
                print!("{}", char::from_u32(memory[*pointer] as u32).unwrap())
            }
            Token::ValueInput => {
                // TODO:implement input
                let mut string = String::with_capacity(5);
                std::io::stdin().read_line(&mut string).unwrap();
                //NOTE:for now defaulting to 0 is ok
                memory[*pointer] = string.chars().next().unwrap_or_default() as u8;
            }
            Token::Loop(sub_program) => {
                while memory[*pointer] > 0 {
                    execute(sub_program, memory, pointer)
                }
            }
        }
    }
}

fn main() {
    let mut args = args().skip(1);
    let filename = args.next().expect("filename required");
    let file_content = fs::read_to_string(filename).unwrap();
    let program = &file_content.parse().unwrap();
    execute(program, &mut vec![0; 3000], &mut 0);
}

use derive_more::{Display, Error};
use indoc::{formatdoc, indoc};
use std::{io::{Write, Read}, str::FromStr};

#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    PointerShift(isize),
    ValueShift(i16),
    ValueOutput,
    ValueInput,
    Loop(Program),
}

impl Token {
    pub fn to_assembly(&self, depth: u8) -> String {
        match self {
            Self::PointerShift(shift) => formatdoc!(
                r#"
                    ; POINTER SHIFT
                    add X6, X6, #{}  

                "#,
                shift
            ),

            Self::ValueShift(shift) => formatdoc!(
                r#"
                    ; VALUE SHIFT
                    ldrb    W7, [X6]
                    add     W7, W7, #{}
                    strb    W7, [X6] 

                "#,
                shift
            ),
            Self::ValueOutput => indoc!(
                r#"
                    ; VALUE OUTPUT
                    mov      X1, X6
                    syscall3 SYS_write, STDOUT, X1, 1

                "#,
            )
            .to_string(),
            Self::ValueInput => indoc!(
                r#"
                    ; VALUE INPUT
                    mov     X1, X6
                    syscall3 SYS_read,  STDIN,  X1, 1
                    
                "#,
            )
            .to_string(),
            Self::Loop(sub_program) => formatdoc!(
                r#"
                    ; LOOP
                    {}:
                    ldrb W7, [X6]
                    cmp  W7, #0
                    b.eq {}f

                    {}
                    b {}b
                    {}:

                "#,
                depth * 2 + 1,
                depth * 2 + 2,
                sub_program
                    .0
                    .iter()
                    .map(|token| token.to_assembly(depth + 1))
                    .collect::<String>(),
                depth * 2 + 1,
                depth * 2 + 2
            ),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Program(Vec<Token>);

impl Program {
    // NOTE:the only purpose of this function is to create Program by hand for tests
    #[cfg(test)]
    pub fn new(tokens: Vec<Token>) -> Self {
        Self(tokens)
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

    pub fn interpret_with_custom_io<I, O>(
        &self,
        memory: &mut Vec<u8>,
        pointer: &mut usize,
        input: &I,
        output: &O,
    ) where
        I: Fn() -> u8,
        O: Fn(u8),
    {
        for token in &self.0 {
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
                Token::ValueOutput => output(memory[*pointer]),
                Token::ValueInput => {
                    memory[*pointer] = input();
                }
                Token::Loop(sub_program) => {
                    while memory[*pointer] > 0 {
                        sub_program.interpret_with_custom_io(memory, pointer, input, output)
                    }
                }
            }
        }
    }

    pub fn interpret(&self, memory: &mut Vec<u8>, pointer: &mut usize) {
        let input = || {
            std::io::stdin().bytes().next().and_then(|r| r.ok()).unwrap()
        };
        let output = |value: u8| {
            print!("{}", char::from_u32(value as u32).unwrap());
            std::io::stdout().flush().unwrap();
        };
        self.interpret_with_custom_io(memory, pointer, &input, &output)
    }

    pub fn compile_to_assembly(&self, memory_size: usize) -> String {
        let code = formatdoc!(
            r#"
            .macro syscall1 syscall X0 
                mov 	X0, \X0
                mov 	X16, \syscall
                svc 	#0x80
            .endm

            .macro syscall3 syscall X0 X1 X2
                mov 	X0, \X0
                mov 	X1, \X1
                mov 	X2, \X2
                mov 	X16, \syscall
                svc 	#0x80 
            .endm

            .set SYS_return, 	1
            .set SYS_read,  	3
            .set SYS_write, 	4

            .set STDIN,			0
            .set STDOUT,		0
            
            .bss
                .lcomm memory, {}
            
            .text
            .align 2
            .global _start
            _start:
            adrp    X6, memory@PAGE
            add     X6,	X6,	memory@PAGEOFF
            
            ; Program:

            {}
            syscall1 SYS_return, #0
            "#,
            memory_size,
            self.0
                .iter()
                .map(|token| token.to_assembly(0))
                .collect::<String>()
        );
        code
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

#[derive(Error, Debug, Display, PartialEq, Eq)]
pub enum ParseError {
    MissingClosingBracket,
    MissingOpeningBracket,
}

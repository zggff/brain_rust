use std::{cell::RefCell, rc::Rc};

use crate::program::{ParseError, Program, Token};

macro_rules! test_interpreter_result {
    ($code: expr, $input: expr, $output: expr) => {{
        let input = $input;
        let program = $code.parse::<Program>().unwrap();
        let input_vec = Rc::new(RefCell::new(input.iter().copied()));
        let output_vec = Rc::new(RefCell::new(Vec::new()));
        let input = || input_vec.borrow_mut().next().unwrap();
        let output = |value| {
            output_vec.borrow_mut().push(value);
        };
        program.interpret_with_custom_io(&mut vec![0; 3000], &mut 0, &input, &output);

        assert_eq!(Rc::try_unwrap(output_vec).unwrap().into_inner(), $output);
        assert_eq!(
            Rc::try_unwrap(input_vec)
                .unwrap()
                .into_inner()
                .collect::<Vec<u8>>(),
            vec![]
        );
    }};
}

#[test]
pub fn test_parser() {
    let code = "-+.--- <<>.>>[-],.";
    assert_eq!(
        code.parse::<Program>(),
        Ok(Program::new(vec![
            Token::ValueShift(0),
            Token::ValueOutput,
            Token::ValueShift(-3),
            Token::PointerShift(-1),
            Token::ValueOutput,
            Token::PointerShift(2),
            Token::Loop(Program::new(vec![Token::ValueShift(-1)])),
            Token::ValueInput,
            Token::ValueOutput,
        ]))
    );
    let code = "[[]";
    assert_eq!(
        code.parse::<Program>(),
        Err(ParseError::MissingClosingBracket)
    );

    let code = "[]]";
    assert_eq!(
        code.parse::<Program>(),
        Err(ParseError::MissingOpeningBracket)
    );
    let code = "[]][";
    assert_eq!(
        code.parse::<Program>(),
        Err(ParseError::MissingOpeningBracket)
    );
}

#[test]
pub fn test_interpreter() {
    test_interpreter_result!(r#",+.."#, vec![1], vec![2, 2]);
    test_interpreter_result!(
        r#"+++++++++++
    >+>>>>++++++++++++++++++++++++++++++++++++++++++++
    >++++++++++++++++++++++++++++++++<<<<<<[>[>>>>>>+>
    +<<<<<<<-]>>>>>>>[<<<<<<<+>>>>>>>-]<[>++++++++++[-
    <-[>>+>+<<<-]>>>[<<<+>>>-]+<[>[-]<[-]]>[<<[>>>+<<<
    -]>>[-]]<<]>>>[>>+>+<<<-]>>>[<<<+>>>-]+<[>[-]<[-]]
    >[<<+>>[-]]<<<<<<<]>>>>>[+++++++++++++++++++++++++
    +++++++++++++++++++++++.[-]]++++++++++<[->-<]>++++
    ++++++++++++++++++++++++++++++++++++++++++++.[-]<<
    <<<<<<<<<<[>>>+>+<<<<-]>>>>[<<<<+>>>>-]<-[>>.>.<<<
    [-]]<<[>>+>+<<<-]>>>[<<<+>>>-]<<[<+>-]>[<+>-]<<<-]"#,
        b"",
        b"1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89"
    );
    test_interpreter_result!(
        r#"++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.
    "#,
        b"",
        b"Hello World!\n"
    );
}

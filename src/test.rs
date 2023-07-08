use std::{cell::RefCell, rc::Rc};

use crate::program::{ParseError, Program, Token};

macro_rules! test_interpreter_result {
    ($code: expr, $input: expr, $output: expr) => {{
        let input = $input;
        let program = $code.parse::<Program>().unwrap();
        let input_vec = Rc::new(RefCell::new(input.iter().copied()));
        let output_vec = Rc::new(RefCell::new(Vec::new()));
        let input = || input_vec.borrow_mut().next();
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
    test_interpreter_result!(
        r#"-,+[                         Read first character and start outer character reading loop
    -[                       Skip forward if character is 0
        >>++++[>++++++++<-]  Set up divisor (32) for division loop
                               (MEMORY LAYOUT: dividend copy remainder divisor quotient zero zero)
        <+<-[                Set up dividend (x minus 1) and enter division loop
            >+>+>-[>>>]      Increase copy and remainder / reduce divisor / Normal case: skip forward
            <[[>+<-]>>+>]    Special case: move remainder back to divisor and increase quotient
            <<<<<-           Decrement dividend
        ]                    End division loop
    ]>>>[-]+                 End skip loop; zero former divisor and reuse space for a flag
    >--[-[<->+++[-]]]<[         Zero that flag unless quotient was 2 or 3; zero quotient; check flag
        ++++++++++++<[       If flag then set up divisor (13) for second division loop
                               (MEMORY LAYOUT: zero copy dividend divisor remainder quotient zero zero)
            >-[>+>>]         Reduce divisor; Normal case: increase remainder
            >[+[<+>-]>+>>]   Special case: increase remainder / move it back to divisor / increase quotient
            <<<<<-           Decrease dividend
        ]                    End division loop
        >>[<+>-]             Add remainder back to divisor to get a useful 13
        >[                   Skip forward if quotient was 0
            -[               Decrement quotient and skip forward if quotient was 1
                -<<[-]>>     Zero quotient and divisor if quotient was 2
            ]<<[<<->>-]>>    Zero divisor and subtract 13 from copy if quotient was 1
        ]<<[<<+>>-]          Zero divisor and add 13 to copy if quotient was 0
    ]                        End outer skip loop (jump to here if ((character minus 1)/32) was not 2 or 3)
    <[-]                     Clear remainder from first division if second division was skipped
    <.[-]                    Output ROT13ed character from copy and clear it
    <-,+                     Read next character
]                            End character reading loop"#,
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz",
        b"NOPQRSTUVWXYZABCDEFGHIJKLMnopqrstuvwxyzabcdefghijklm"
)
}

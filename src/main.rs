use program::Program;
use std::env::args;
mod program;

#[cfg(test)]
mod test;

enum ExecType {
    Interpret,
    CompileToAssembly(String),
}

fn main() {
    let mut args = args().skip(1);
    let mut exec_type = ExecType::Interpret;
    let mut memory_size = None;
    let mut input_file = None;
    while let Some(arg) = args.next() {
        if arg == "-c" {
            exec_type = ExecType::CompileToAssembly(args.next().unwrap())
        } else if arg == "-m" {
            memory_size = Some(args.next().unwrap().parse::<usize>().unwrap())
        } else {
            input_file = Some(arg)
        }
    }
    let memory_size = memory_size.unwrap_or(3000);
    let input_file = input_file.expect("filename required");
    let input = std::fs::read_to_string(input_file).unwrap();
    let program = &input.parse::<Program>().unwrap();
    match exec_type {
        ExecType::Interpret => program.interpret(&mut vec![0; memory_size], &mut 0),
        ExecType::CompileToAssembly(output_file) => {
            let assembly = program.compile_to_assembly(memory_size);
            std::fs::write(output_file, assembly).unwrap()
        }
    }
}

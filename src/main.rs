use bs::executor::Executor;
use bs::lexer::tokenize;
use bs::parser::{self, Ast};
use std::time::Instant;
use std::{fs, process};

fn main() {
    let input = fs::read_to_string("./example.bs").unwrap();
    let tokens = tokenize(&input);
    let ast = match parser::parse(tokens) {
        Ok(e) => Ast::new(e),
        Err(e) => {
            println!("{}", e.format_with(&input, "parse error"));
            process::exit(1);
        }
    };
    let mut executor = match Executor::build(ast) {
        Ok(e) => e,
        Err(e) => {
            println!("at build: {}", e.format_with(&input, "build error"));
            process::exit(1);
        }
    };

    let time = Instant::now();
    let result = executor.execute("main", vec![&["hello", "world"]]);
    match result {
        Ok(r) => println!("result: {:?}", r),
        Err(e) => println!("{}", e.format_with(&input, "execution error")),
    }
    println!("execution took: {:?}", time.elapsed());
}

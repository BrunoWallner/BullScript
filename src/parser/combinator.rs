use crate::{error::ParseError, lexer::TokenStream};

pub fn any<O>(
    function: &[fn(&mut TokenStream, depth: u32) -> Result<O, ParseError>],
    input: &mut TokenStream,
    depth: u32,
) -> Result<O, ParseError> {
    let mut errors = Vec::new();
    for f in function.into_iter() {
        let pointer = input.pointer();
        match f(input, depth) {
            Ok(o) => return Ok(o),
            Err(e) => {
                // reset
                input.set_pointer(pointer);
                errors.push(e);
            }
        }
    }
    errors.sort_unstable_by_key(|e| e.depth);
    errors.reverse();
    Err(errors.remove(0))
}

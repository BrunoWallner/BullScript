// use super::Token;
// use nom::InputLength;

// use std::cmp::PartialEq;

// #[derive(Clone, Debug, PartialEq)]
// pub enum TokenKind {
//     Return,
//     Identifier,
// }
// impl InputLength for TokenKind {
//     fn input_len(&self) -> usize {
//         1
//     }
// }
// impl PartialEq<Token> for TokenKind {
//     fn eq(&self, other: &Token) -> bool {
//         match self {
//             // Self::Return => {other == &Token::Identifier{..}},
//             Self::Return => {
//                 match other {
//     &Token::Identifier {
//         ..
//     } => true,
//     _ => false

//     }
//             }
//             _ => false,
//         }
//     }
// }

use std::ops::Index;
use super::{Token, TokenKind};
// use nom::{InputLength, InputTake};

#[derive(Clone, Debug)]
pub struct TokenStream {
    token: Vec<Token>,
    pointer: usize,
}

impl TokenStream {
    pub fn new(token: &[Token]) -> Self {
        Self {
            token: Vec::from(token),
            pointer: 0,
        }
    }

    pub fn from(token: Vec<Token>) -> Self {
        Self { token, pointer: 0 }
    }

    pub fn is_empty(&self) -> bool {
        self.token.is_empty() || self.token.len() <= self.pointer
    }

    pub fn pointer(&self) -> usize {
        self.pointer
    }

    pub fn set_pointer(&mut self, pointer: usize) {
        self.pointer = pointer
    }

    pub fn peek(&self, index: usize) -> Option<&Token> {
        let i = self.pointer + index;
        self.token.get(i)
    }

    pub fn get_current_index(&self) -> Option<usize> {
        Some(self.peek(0)?.index)
    }

    pub fn next(&mut self) -> Option<&Token> {
        let tmp = self
            .token
            .get(self.pointer);
        self.pointer += 1;
        tmp
    }

    pub fn skip_if(&mut self, token: &TokenKind) -> Option<()> {
        (&self.peek(0)?.kind == token).then(|| self.advance(1))
    }

    pub fn advance(&mut self, advance: usize) {
        self.pointer += advance;
    }

    /// expects the first left token to be stripped
    /// does not ommit the last right token
    pub fn peek_pair_counting_stripped_inclusive(
        &mut self,
        left: &TokenKind,
        right: &TokenKind,
    ) -> Option<&[Token]> {
        let mut index = 0;
        let mut end = None;
        let mut count = 1;
        while let Some(t) = self.peek(index) {
            if &t.kind == left {
                count += 1;
            }
            if &t.kind == right {
                end = Some(index);
                count -= 1;
                if count <= 0 {
                    break;
                }
            }
            index += 1;
        }
        match end {
            Some(end) => Some(&self.token[self.pointer..=self.pointer + end]),
            _ => None,
        }
    }
}

impl Index<usize> for TokenStream {
    type Output = Token;

    fn index(&self, index: usize) -> &Self::Output {
        &self.token[index]
    }
}

impl Iterator for TokenStream {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.token.len() > 0 {
            Some(self.token.remove(0))
        } else {
            None
        }
    }
}

impl FromIterator<Token> for TokenStream {
    fn from_iter<T: IntoIterator<Item = Token>>(iter: T) -> Self {
        let token: Vec<Token> = iter.into_iter().collect();
        TokenStream::from(token)
    }
}

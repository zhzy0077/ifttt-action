use crate::{Indexable, Mapper};
use anyhow::Result;
use serde::export::Formatter;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{self, Display};
use std::iter::FromIterator;

// use {name} to express.
pub struct TextMapper {
    text: String,
}

impl TextMapper {
    pub fn new(text: impl Into<String>) -> Self {
        TextMapper { text: text.into() }
    }
}

#[derive(Debug)]
enum TextMapperError {
    NoMatchBracket,
}

impl Error for TextMapperError {
    fn description(&self) -> &str {
        unimplemented!()
    }
}

impl Display for TextMapperError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Mapper for TextMapper {
    fn map(&self, input: &dyn Indexable) -> Result<String> {
        let output: Vec<_> = self.text.chars().collect();
        let mut replacement = HashMap::new();

        for mut idx in 0..output.len() {
            if output.get(idx) != Some(&'{') || (idx != 0 && output.get(idx - 1) == Some(&'\\')) {
                continue;
            }

            let start = idx;
            let mut end = None;
            while idx < output.len() {
                idx += 1;
                if output.get(idx) == Some(&'}') && output.get(idx - 1) != Some(&'\\') {
                    end = Some(idx);
                    break;
                }
            }
            let end = end.ok_or_else(|| TextMapperError::NoMatchBracket)?;
            let expr = String::from_iter(&output[start..end + 1]);
            let value = input.index(&expr[1..expr.len() - 1]);

            replacement.insert(expr, value);
        }

        let mut output = String::from_iter(output);
        for (from, to) in replacement {
            output = output.replace(&from, &to[..]);
        }

        Ok(output)
    }
}

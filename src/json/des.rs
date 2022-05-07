use crate::token::*;
use peekread::{PeekRead, SeekPeekReader};
use std::io::{Read, Seek};
use anyhow::{Result, bail};
use byteorder::{BigEndian, ReadBytesExt};

pub struct JsonDeserializer {
    pub stream: TDFTokenStream,
    input: String,
    cursor: usize,
}


#[derive(Debug)]
pub enum JsonDesError {
    
}

impl JsonDeserializer {
    fn new(input: String) -> Self {
        Self {
            stream: TDFTokenStream::new(),
            input,
            cursor: 0,
        }
    }

    fn get_char(&self) -> Option<&str> {
        self.input.get(self.cursor.clone()..1)
    }

    fn validate_symbol(&mut self) -> Result<&str> {
        let c = match self.get_char() {
            Some(c) => c,
            None => bail!("Expected char, got None!"),
        };
        Ok(c)
    }

    fn read_string(&mut self) -> Result<String> {
        let a = self.validate_symbol()?;
        if a != "\"" {
            bail!("Expected string start, found {}!", a)
        }
        self.cursor += 1;
        let mut s = String::new();

        loop {
            match self.validate_symbol()? {
                "\\" => {
                    self.cursor += 2;
                    continue;
                },
                "\"" => {
                    self.cursor += 1;
                    break;
                },
                k => {
                    s.push_str(k);
                    self.cursor += 1;
                },
            }
        }
        Ok(s)
    }

    fn deserialize_string(&mut self) -> Result<()> {
        let s = self.read_string()?;
        self.stream.push(TDFToken::String(s.as_bytes().to_vec()));
        Ok(())
    }

    fn deserialize_map(&mut self) -> Result<()> {

        let key = self.read_string()?;

        loop {
            let a = self.validate_symbol()?;
            match a {
                "}" => {
                    self.cursor += 1;
                    self.stream.push(TDFToken::MapEnd);
                    break;
                },
                "\"" => {
                    self.stream.push(TDFToken::StringType);
                    self.deserialize_string()?;
                },
                a => {
                    bail!("Unexpected symbol {}!", a)
                }
            }
        }

        Ok(())
    }

    fn deserialize(&mut self) -> Result<()> {

        match self.validate_symbol()? {
            "{" => {
                self.cursor += 1;
                self.stream.push(TDFToken::MapStart);
                self.deserialize_map()?;
            },
            "}" => {
                self.cursor += 1;
                self.stream.push(TDFToken::MapEnd);
            },
            "\"" => {
                self.cursor += 1;
                self.stream.push(TDFToken::StringType);
                let mut s = String::new();
                while let Some(a) = self.get_char() {
                    if a == "\"" {
                        break;
                    }
                    s.push_str(a);
                }
                
                self.deserialize()?;
            },
            a => {
                bail!("Unexpected symbol {}!", a)
            }
        }
        Ok(())
    }
}

impl<'a> TDFDeserializer<String> for JsonDeserializer {
    fn deserialize(reader: &mut String) -> Result<TDFTokenStream> {

        let mut des = Self::new(reader.clone());
        des.deserialize()?;
        Ok(des.stream)
    }
}
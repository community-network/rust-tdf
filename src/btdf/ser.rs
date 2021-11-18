
use crate::token::*;
use std::io::{Write};
use anyhow::{Result, bail};
use byteorder::{BigEndian, WriteBytesExt};

use super::des::{VARSIZE_NEGATIVE, VARSIZE_MORE};

pub struct BTDFSerializer {
    stream: TDFTokenStream,
}

impl BTDFSerializer {

    pub fn new(stream: TDFTokenStream) -> Self {
        Self {
            stream
        }
    }

    pub fn ser_token(&mut self, writer: &mut dyn Write, token_type: TDFToken, is_root: bool) -> Result<()> {
        match token_type {
            TDFToken::IntType        => self.ser_int(writer),
            TDFToken::StringType     => self.ser_string(writer),
            TDFToken::BlobType       => self.ser_blob(writer),
            TDFToken::MapType        => self.ser_map(writer, is_root),
            TDFToken::ListType       => self.ser_list(writer),
            TDFToken::PairListType   => self.ser_pair_list(writer),
            TDFToken::UnionType      => self.ser_union(writer),
            TDFToken::IntListType    => self.ser_int_list(writer),
            TDFToken::ObjectTypeType => self.ser_object_type(writer),
            TDFToken::ObjectIdType   => self.ser_object_id(writer),
            TDFToken::FloatType      => self.ser_float(writer),
            TDFToken::GenericType    => self.ser_generic(writer),
            _ => bail!("Trying to parse type token, but found {:?}", token_type)
        }
    }

    pub fn ser_int(&mut self, writer: &mut dyn Write) -> Result<()> {
        let token =  self.stream.next()?;
        match token {
            TDFToken::Int(number) => self.write_number(writer, number),
            _=> bail!("Expected Integer, found {:?}", token),
        }
    }

    pub fn ser_string(&mut self, writer: &mut dyn Write) -> Result<()> {
        let token =  self.stream.next()?;
        match token {
            TDFToken::String(string) => self.write_string(writer, string),
            _=> bail!("Expected String, found {:?}", token),
        }
    }

    pub fn ser_blob(&mut self, writer: &mut dyn Write) -> Result<()> {
        let token =  self.stream.next()?;
        match token {
            TDFToken::Blob(blob) => self.write_blob(writer, blob),
            _=> bail!("Expected Blob, found {:?}", token),
        }
    }

    pub fn ser_map(&mut self, writer: &mut dyn Write, is_root: bool) -> Result<()> {

        let token = self.stream.next()?;
        if token != TDFToken::MapStart {
            bail!("Expected Map, found {:?}", token);
        }

        loop {

            let mut label = self.stream.next()?;

            if label == TDFToken::MapEnd {
                if !is_root {
                    writer.write_u8(0)?;
                }
                return Ok(());
            } else if label == TDFToken::MapUnion {
                writer.write_u8(2)?;
                label = self.stream.next()?;
            }

            match label {
                TDFToken::Label(label_string) => self.write_label(writer, &label_string)?,
                _ => bail!("Expected Label in Map, found {:?}", label),
            }
            
            let value = self.stream.next()?;
            writer.write_u8(value.get_tag()?)?;

            self.ser_token(writer, value, false)?;
        }

    }

    pub fn ser_list(&mut self, writer: &mut dyn Write) -> Result<()> {

        let token = self.stream.next()?;
        let size = match token {
            TDFToken::ListStart(s) => s,
            _ => bail!("Expected List, found {:?}", token),
        };
        
        let inner_type = self.stream.next()?;
        writer.write_u8(inner_type.get_tag()?)?;

        self.write_number(writer, size as i64)?;

        for _ in 0..size {
            self.ser_token(writer, inner_type.clone(), false)?;
        }

        let end_token = self.stream.next()?;
        if end_token != TDFToken::ListEnd {
            bail!("Expected End of list, found {:?}", end_token)
        }

        Ok(())
    }

    pub fn ser_pair_list(&mut self, writer: &mut dyn Write) -> Result<()> {

        let token = self.stream.next()?;
        let size = match token {
            TDFToken::PairListStart(s) => s,
            _ => bail!("Expected Map, found {:?}", token),
        };
        
        let k_type = self.stream.next()?;
        writer.write_u8(k_type.get_tag()?)?;

        let v_type = self.stream.next()?;
        writer.write_u8(v_type.get_tag()?)?;

        self.write_number(writer, size as i64)?;

        for _ in 0..size {
            self.ser_token(writer, k_type.clone(), false)?;
            self.ser_token(writer, v_type.clone(), false)?;
        }

        let end_token = self.stream.next()?;
        if end_token != TDFToken::PairListEnd {
            bail!("Expected End of Pair list, found {:?}", end_token)
        }

        Ok(())
    }

    pub fn ser_union(&mut self, writer: &mut dyn Write) -> Result<()> {

        let token = self.stream.next()?;
        let union_type = match token {
            TDFToken::UnionStart(t) => t,
            _ => bail!("Expected Union start, found {:?}", token),
        };
        

        writer.write_u8(union_type as u8)?;

        if union_type == UnionType::Unset {

            let end_token = self.stream.next()?;
            if end_token != TDFToken::UnionEnd {
                bail!("Expected End of Union, found {:?}", end_token)
            }

            return Ok(());

        }

        let union_label = self.stream.next()?;

        match union_label {
            TDFToken::Label(label_string) => self.write_label(writer, &label_string)?,
            _ => bail!("Expected Label in Union, found {:?}", union_label),
        }
        
        let value = self.stream.next()?;
        writer.write_u8(value.get_tag()?)?;

        self.ser_token(writer, value, false)?;

        let end_token = self.stream.next()?;
        if end_token != TDFToken::UnionEnd {
            bail!("Expected End of Union, found {:?}", end_token)
        }

        Ok(())
    }

    pub fn ser_generic(&mut self, writer: &mut dyn Write) -> Result<()> {

        let token = self.stream.next()?;
        let exist = match token {
            TDFToken::GenericStart(t) => t,
            _ => bail!("Expected Generic start, found {:?}", token),
        };
        

        writer.write_u8(if exist {1} else {0})?;

        if !exist {

            let end_token = self.stream.next()?;
            if end_token != TDFToken::GenericEnd {
                bail!("Expected End of Generic, found {:?}", end_token)
            }

            return Ok(());

        }

        let generic_label = self.stream.next()?;

        match generic_label {
            TDFToken::Label(label_string) => self.write_label(writer, &label_string)?,
            _ => bail!("Expected Label in Generic, found {:?}", generic_label),
        }
        
        let value = self.stream.next()?;
        writer.write_u8(value.get_tag()?)?;

        self.ser_token(writer, value, false)?;

        let end_token = self.stream.next()?;
        if end_token != TDFToken::GenericEnd {
            bail!("Expected End of Generic, found {:?}", end_token)
        }
        
        // Terminator, but only if not empty
        writer.write_u8(0)?;

        Ok(())
    }

    pub fn ser_int_list(&mut self, writer: &mut dyn Write) -> Result<()> {

        let token = self.stream.next()?;
        let size = match token {
            TDFToken::IntListStart(s) => s,
            _ => bail!("Expected Int List start, found {:?}", token),
        };

        self.write_number(writer, size as i64)?;

        for _ in 0..size {
           self.ser_int(writer)?;
        }

        let end_token = self.stream.next()?;
        if end_token != TDFToken::IntListEnd {
            bail!("Expected End of Int List, found {:?}", end_token)
        }
        
        Ok(())
    }

    
    pub fn ser_object_type(&mut self, writer: &mut dyn Write) -> Result<()> {

        for _ in 0..2 {
            self.ser_int(writer)?;
        }

        Ok(())
    }

    pub fn ser_object_id(&mut self, writer: &mut dyn Write) -> Result<()> {

        for _ in 0..3 {
            self.ser_int(writer)?;
        }

        Ok(())
    }

    pub fn ser_float(&mut self, writer: &mut dyn Write) -> Result<()> {

        let token = self.stream.next()?;
        let number = match token {
            TDFToken::Float(f) => f,
            _ => bail!("Expected Int List start, found {:?}", token),
        };

        writer.write_f32::<BigEndian>(number)?;

        Ok(())
    }

    pub fn write_number(&self, writer: &mut dyn Write, mut number: i64) -> Result<()> {

        if number == 0 {
            writer.write_u8(0)?;
            return Ok(());
        }

        let mut extra = vec![];

        if number < 0 {
            number = -number;
            extra.push(number as u8 | (VARSIZE_MORE | VARSIZE_NEGATIVE));
        } else {
            extra.push((number as u8 & (VARSIZE_NEGATIVE - 1)) | VARSIZE_MORE);
        }

        number >>= 6;

        while number > 0 {
            extra.push(number as u8 | VARSIZE_MORE);
            number >>= 7;
        }

        let last = extra.last_mut().unwrap();
        *last &= !VARSIZE_MORE;
        
        writer.write(&extra)?;

        Ok(())

    }

    pub fn write_string(&self, writer: &mut dyn Write, string: Vec<u8>) -> Result<()> {
        self.write_number(writer, (string.len() + 1) as i64)?;
        writer.write(&string)?;
        writer.write_u8(0)?;
        Ok(())
    }

    pub fn write_blob(&self, writer: &mut dyn Write, blob: Vec<u8>) -> Result<()> {
        self.write_number(writer, blob.len() as i64)?;
        writer.write(&blob)?;
        Ok(())
    }

    pub fn write_label(&self, writer: &mut dyn Write, label: &String) -> Result<()> {

        let mut label_string = label.to_uppercase().replace("_", " ");

        if label_string.len() < 4 {
            for _ in label_string.len()..4 {   
                label_string.push_str(" ");
            }
        } else if label_string.len() > 4 {
            label_string = label_string.drain(..4).collect();
        }

        let c = label_string.as_bytes();

        let a: Vec<u8> = Vec::from(c).iter().map(|v| { 
            if *v == 3_u8 {
                return 0_u8
            }
            *v as u8
        }).collect();

        writer.write(&[
            (a[0] & 0x40) << 1 | (a[0] & 0x1F) << 2 | (a[1] & 0x40) >> 5 | (a[1] & 0x10) >> 4, 
            (a[1] & 0xF) << 4 | (a[2] & 0x40) >> 3 | (a[2] & 0x10) >> 2 | ((a[2] & 0xC) >> 2), 
            (a[2] & 3) << 6 | (a[3] & 0x40) >> 1 | a[3] & 0x1F,
        ])?;

        Ok(())
    }
}

impl<W: Write> TDFSerializer<W> for BTDFSerializer {
    fn serialize(stream: TDFTokenStream, mut writer: &mut W) -> Result<()> {
        let mut des = Self {
            stream
        };
        let token = des.stream.next()?;
        des.ser_token(&mut writer, token, true)
    }
}

use crate::token::*;
use peekread::{PeekRead, SeekPeekReader};
use std::io::{Read, Seek};
use anyhow::{Result, bail};
use byteorder::{BigEndian, ReadBytesExt};

// use itertools::Itertools;
use num_traits::FromPrimitive;

pub const VARSIZE_NEGATIVE: u8 = 0x40;
pub const VARSIZE_MORE: u8 = 0x80;

pub struct BTDFDeserializer {
    pub stream: TDFTokenStream,
}


impl BTDFDeserializer {

    pub fn new() -> Self {
        Self {
            stream: TDFTokenStream::new(),
        }
    }

    pub fn des_token(&mut self, reader: &mut impl PeekRead, tdf_type: TDFToken, is_root: bool) -> Result<()> {

        log::trace!("Token: {:?}", tdf_type);

        let result = match tdf_type {
            TDFToken::IntType        => self.des_int(reader),
            TDFToken::StringType     => self.des_string(reader),
            TDFToken::BlobType       => self.des_blob(reader),
            TDFToken::MapType        => self.des_map(reader, is_root),
            TDFToken::ListType       => self.des_list(reader),
            TDFToken::PairListType   => self.des_pair_list(reader, false),
            TDFToken::UnionType      => self.des_union(reader),
            TDFToken::IntListType    => self.des_int_list(reader),
            TDFToken::ObjectTypeType => self.des_object_type(reader),
            TDFToken::ObjectIdType   => self.des_object_id(reader),
            TDFToken::FloatType      => self.des_float(reader),
            TDFToken::GenericType    => self.des_generic(reader),
            _ => bail!("Expected token, found {:?}!", tdf_type)
        };

        result

    }

    fn des_bugged_token(&mut self, reader: &mut impl PeekRead, tdf_type: TDFToken, is_root: bool) -> Result<()> {
        log::trace!("Bugged Token: {:?}", tdf_type);

        let result = match tdf_type {
            TDFToken::IntType        => self.des_int(reader),
            TDFToken::StringType     => self.des_string(reader),
            TDFToken::BlobType       => self.des_blob(reader),
            TDFToken::MapType        => self.des_map(reader, is_root),
            TDFToken::ListType       => self.des_list(reader),
            TDFToken::PairListType   => self.des_pair_list(reader, true),
            TDFToken::UnionType      => self.des_union(reader),
            TDFToken::IntListType    => self.des_int_list(reader),
            TDFToken::ObjectTypeType => self.des_object_type(reader),
            TDFToken::ObjectIdType   => self.des_object_id(reader),
            TDFToken::FloatType      => self.des_float(reader),
            TDFToken::GenericType    => self.des_generic(reader),
            _ => bail!("Expected token, found {:?}!", tdf_type)
        };

        result
    }

    pub fn des_label(&mut self, reader: &mut impl PeekRead) -> Result<bool> {

        // Apparently EA tdf has a bug
        // Where if u use pairlist of pairlist it encodes second pairlist type as a map
        // Idk what to do with it, so added this check for later replacement
        // In one specific field that I have this problem
        let mut has_heat1_bug = false;

        let mut label_tag_bytes = [0; 3];
        reader.read(&mut label_tag_bytes)?;

        let tag_bytes = Vec::from(label_tag_bytes);

        if vec![0x9E, 0x2C, 0xA1] == tag_bytes {
            has_heat1_bug = true;
        }

        let mut label_bytes = String::new(); 

        fn converter(m: u8, c: u8) -> char {
            if m | c == 0x00 {
                // Space
                return char::from(32);
            } else if m & 0x40 == 0 {
                return char::from(0x30 | c)
            } else {
                return char::from(m | c)
            }
        }

        label_bytes.push(converter(
            (tag_bytes[0] & 0x80) >> 1,
            (tag_bytes[0] & 0x7C) >> 2
        ));

        label_bytes.push(converter(
            (tag_bytes[0] & 2) << 5,
            ((tag_bytes[0] & 1) << 4) | ((tag_bytes[1] & 0xF0) >> 4)
        ));
    
        label_bytes.push(converter(
            (tag_bytes[1] & 8) << 3, 
            ((tag_bytes[1] & 7) << 2) | ((tag_bytes[2] & 0xC0) >> 6)
        ));
    
        label_bytes.push(converter(
            (tag_bytes[2] & 0x20) << 1,
            tag_bytes[2] & 0x1F
        ));

        

        self.stream.push(TDFToken::Label(label_bytes));

        Ok(has_heat1_bug)
    }
    
    pub fn des_map(&mut self, reader: &mut impl PeekRead, _is_root: bool) -> Result<()> {

        self.stream.push(TDFToken::MapStart);

        loop {

            let terminator_result = reader.peek().read_u8();

            match terminator_result {   
                Ok(terminator) => {
                    if terminator == 0_u8 {
                        reader.read_u8()?;
                        self.stream.push(TDFToken::MapEnd);
                        return Ok(());
                    } else if terminator <= 2 { 
                        self.stream.push(TDFToken::MapUnion);
                        reader.read_u8()?;
                    }
                },
                Err(_) => {
                    self.stream.push(TDFToken::MapEnd);
                    return Ok(());
                }
            }

            let has_bug = self.des_label(reader)?;
            
            let type_tag = reader.read_u8()?;
            let tdf_type = TDFToken::from_tag(type_tag)?;

            self.stream.push(tdf_type.clone());

            if has_bug {
                self.des_bugged_token(reader, tdf_type, false)?;
            } else {
                self.des_token(reader, tdf_type, false)?;
            }
        }

    }

    pub fn des_int(&mut self, reader: &mut impl PeekRead) -> Result<()> {
        self.stream.push(TDFToken::Int(self.read_number(reader)?));
        Ok(())
    }

    pub fn read_number(&self, reader: &mut impl PeekRead) -> Result<i64> {

        let mut b = reader.read_u8()?;

        let is_negative = (b & VARSIZE_NEGATIVE) != 0;
        let mut value =  (b as u64) & (VARSIZE_NEGATIVE - 1) as u64;

        let mut shift = 6;
        let mut more = (b & VARSIZE_MORE) != 0;

        while more {
            b = reader.read_u8()?;
            value |= ((b as u64) & ((VARSIZE_MORE - 1) as u64)) << shift;
            more = (b & VARSIZE_MORE) != 0;
            shift += 7;
        }

        let mut value = value as i64;

        if is_negative {
            if value != 0 {
                value = -value;
            } else {
                value = i64::MIN;
            }
        }

        Ok(value)
    }

    pub fn des_string(&mut self, reader: &mut impl PeekRead) -> Result<()> {

        let size = self.read_number(reader)?;

        log::trace!("String size = {}", size);

        /*
            In some situation len might me 0
            In that case (0-1) as usize will give usize::MAX
            Which will result in buffer overflow
        */
        if size == 0 {
            self.stream.push(TDFToken::String(vec![]));
            return Ok(());
        }

        /*
            It appears that in new versions -1 is allowed
            That means String can be as long as it is possible
            Read till we hit terminator than
            This is generally slower
        */
        if size < 0 {
            let mut res = vec![];
            let mut b = reader.read_u8()?;
            while b != 0 {
                res.push(b);
                b = reader.read_u8()?;
            }
            self.stream.push(TDFToken::String(res));
            return Ok(());
        }

        let mut res = vec![0; (size - 1) as usize];
        reader.read(&mut res)?;

        // Null terminated string
        reader.read_u8()?;

        self.stream.push(TDFToken::String(res));

        Ok(())
    }

    pub fn des_blob(&mut self, reader: &mut impl PeekRead) -> Result<()> {

        let size = self.read_number(reader)?;

        let mut res = vec![0; size as usize];
        reader.read(&mut res)?;

        self.stream.push(TDFToken::Blob(res));

        Ok(())

    }

    pub fn des_list(&mut self, reader: &mut impl PeekRead) -> Result<()> {

        let type_tag = reader.read_u8()?;
        let tdf_type = TDFToken::from_tag(type_tag)?;

        let size = self.read_number(reader)? as usize;

        self.stream.push(TDFToken::ListStart(size));
        self.stream.push(tdf_type.clone());

        for _ in 0..size {
            self.des_token(reader, tdf_type.clone(), false)?;
        }
        
        self.stream.push(TDFToken::ListEnd);

        Ok(())
    }

    pub fn des_pair_list(&mut self, reader: &mut impl PeekRead, has_bug: bool) -> Result<()> {

        let key_tag = reader.read_u8()?;
        let tdf_key = TDFToken::from_tag(key_tag)?;

        let value_tag = reader.read_u8()?;
        let mut tdf_value = TDFToken::from_tag(value_tag)?;

        if has_bug {
            tdf_value = TDFToken::PairListType;
        }

        let size = self.read_number(reader)? as usize;

        log::trace!("Pairlist size: {}", size);

        self.stream.push(TDFToken::PairListStart(size));
        self.stream.push(tdf_key.clone());
        self.stream.push(tdf_value.clone());

        for _ in 0..size {
            self.des_token(reader, tdf_key.clone(), false)?;
            self.des_token(reader, tdf_value.clone(), false)?;
        }
        
        self.stream.push(TDFToken::PairListEnd);

        Ok(())
    }

    pub fn des_int_list(&mut self, reader: &mut impl PeekRead) -> Result<()> {

        let size = self.read_number(reader)? as usize;

        self.stream.push(TDFToken::IntListStart(size));

        for _ in 0..size {
            self.des_int(reader)?;
        }
        
        self.stream.push(TDFToken::IntListEnd);

        Ok(())
    }

    pub fn des_union(&mut self, reader: &mut impl PeekRead) -> Result<()> {

        let union_type = FromPrimitive::from_u8(
            reader.read_u8()?
        ).unwrap_or(UnionType::Unset);

        self.stream.push(TDFToken::UnionStart(union_type.clone()));

        if union_type != UnionType::Unset {

            self.des_label(reader)?;
            
            let type_tag = reader.read_u8()?;
            let tdf_type = TDFToken::from_tag(type_tag)?;

            self.stream.push(tdf_type.clone());

            self.des_token(reader, tdf_type, false)?;

        }
        
        self.stream.push(TDFToken::UnionEnd);

        Ok(())
    }

    pub fn des_generic(&mut self, reader: &mut impl PeekRead) -> Result<()> {

        let generic_exists = reader.read_u8()? != 0;

        self.stream.push(TDFToken::GenericStart(generic_exists));

        if generic_exists {

            self.des_int(reader)?;
            self.des_label(reader)?;
            
            let type_tag = reader.read_u8()?;
            let tdf_type = TDFToken::from_tag(type_tag)?;

            self.stream.push(tdf_type.clone());

            self.des_token(reader, tdf_type, false)?;
            let null = reader.read_u8()?;

            if null != 0 {
                log::trace!("Generic terminator is not null!!1 Found [{}].", {null});
            }
        }
        
        self.stream.push(TDFToken::GenericEnd);

        Ok(())
    }

    pub fn des_object_type(&mut self, reader: &mut impl PeekRead) -> Result<()> {
        for _ in 0..2 {
            self.des_int(reader)?;
        }
        Ok(())
    }

    pub fn des_object_id(&mut self, reader: &mut impl PeekRead) -> Result<()> {
        for _ in 0..3 {
            self.des_int(reader)?;
        }
        Ok(())
    }

    pub fn des_float(&mut self, reader: &mut impl PeekRead) -> Result<()> {
        let float = reader.read_f32::<BigEndian>()?;
        self.stream.push(TDFToken::Float(float));
        Ok(())
    }
}

impl<R: Read + Seek> TDFDeserializer<R> for BTDFDeserializer {
    fn deserialize(reader: &mut R) -> Result<TDFTokenStream> {

        // Make seek reader
        let mut reader = SeekPeekReader::new(reader);

        let mut des = Self::new();

        // Des self as map
        des.stream.push(TDFToken::MapType);
        des.des_map(&mut reader, true)?;
        
        Ok(des.stream)
    }
}

use crate::token::*;
use std::io::{Write};
use anyhow::{Result, bail};
use byteorder::{BigEndian, WriteBytesExt};


pub struct JsonSerializer {
    stream: TDFTokenStream,
}

impl JsonSerializer {

    pub fn new(stream: TDFTokenStream) -> Self {
        Self {
            stream
        }
    }

    pub fn ser_token(&mut self, token_type: TDFToken, level: u32) -> Result<String> {
        return match token_type {
            TDFToken::IntType        => self.ser_int(),
            TDFToken::StringType     => self.ser_string(),
            TDFToken::BlobType       => self.ser_blob(),
            TDFToken::MapType        => self.ser_map(level),
            TDFToken::ListType       => self.ser_list(level),
            TDFToken::PairListType   => self.ser_pair_list(level),
            TDFToken::UnionType      => self.ser_union(level),
            TDFToken::IntListType    => self.ser_int_list(),
            TDFToken::ObjectTypeType => self.ser_object_type(),
            TDFToken::ObjectIdType   => self.ser_object_id(),
            TDFToken::FloatType      => self.ser_float(),
            TDFToken::GenericType    => self.ser_generic(level),
            _ => bail!("Trying to parse type token, but found {:?}", token_type)
        }
    }

    pub fn ser_int(&mut self) -> Result<String> {
        let token = self.stream.next()?;
        match token {
            TDFToken::Int(number) => self.write_number(number),
            _=> bail!("Expected Integer, found {:?}", token),
        }
    }

    pub fn ser_string(&mut self) -> Result<String> {
        let token =  self.stream.next()?;
        match token {
            TDFToken::String(string) => self.write_string(string),
            _=> bail!("Expected String, found {:?}", token),
        }
    }

    pub fn ser_blob(&mut self) -> Result<String> {
        let token =  self.stream.next()?;
        match token {
            TDFToken::Blob(blob) => self.write_blob(blob),
            _=> bail!("Expected Blob, found {:?}", token),
        }
    }

    pub fn ser_map(&mut self, level: u32) -> Result<String> {

        let token = self.stream.next()?;
        if token != TDFToken::MapStart {
            bail!("Expected Map, found {:?}", token);
        }

        let mut output = String::new();
        let mut iter = 0;
        output.push_str("{");

        loop {

            let mut label = self.stream.next()?;

            if label == TDFToken::MapEnd {
                if iter != 0 {
                    output.push_str("\n");
                } else {
                    for _ in 0..level {
                        output.push_str("\t");
                    }
                }
                output.push_str("}");
                return Ok(output);
            } else if label == TDFToken::MapUnion {
                label = self.stream.next()?;
            }

            if iter != 0 {
                output.push_str(",\n");
            } else {
                output.push_str("\n");
            }

            for _ in 0..level+1 {
                output.push_str("\t");
            }

            match label {
                TDFToken::Label(label_string) => {
                    output.push_str(&self.write_label(&label_string)?);
                },
                _ => bail!("Expected Label in Map, found {:?}", label),
            }
            output.push_str(": ");
            
            let value = self.stream.next()?;
            output.push_str(&self.ser_token(value, level+1)?);

            iter += 1;
        }

    }

    pub fn ser_list(&mut self, level: u32) -> Result<String> {

        let token = self.stream.next()?;
        let size = match token {
            TDFToken::ListStart(s) => s,
            _ => bail!("Expected List, found {:?}", token),
        };

        let mut output = String::new();
        
        let inner_type = self.stream.next()?;

        output.push_str("[");
        for i in 0..size {
            output.push_str(&self.ser_token( inner_type.clone(), level)?);
            if i != size-1 {
                output.push_str(",");
            }
        }
        output.push_str("]");

        let end_token = self.stream.next()?;
        if end_token != TDFToken::ListEnd {
            bail!("Expected End of list, found {:?}", end_token)
        }

        Ok(output)
    }

    pub fn ser_pair_list(&mut self, level: u32) -> Result<String> {

        let token = self.stream.next()?;
        let size = match token {
            TDFToken::PairListStart(s) => s,
            _ => bail!("Expected Map, found {:?}", token),
        };

        let mut output = String::new();
        
        let k_type = self.stream.next()?;
        let v_type = self.stream.next()?;

        output.push_str("[");
        for i in 0..size {
            output.push_str("[");
            output.push_str(&self.ser_token( k_type.clone(), level)?);
            output.push_str(",");
            output.push_str(&self.ser_token( v_type.clone(), level)?);
            output.push_str("]");
            if i != size-1 {
                output.push_str(",\n");
            }
        }
        output.push_str("]");

        let end_token = self.stream.next()?;
        if end_token != TDFToken::PairListEnd {
            bail!("Expected End of Pair list, found {:?}", end_token)
        }

        Ok(output)
    }

    pub fn ser_union(&mut self, level: u32) -> Result<String> {

        let token = self.stream.next()?;
        let union_type = match token {
            TDFToken::UnionStart(t) => t,
            _ => bail!("Expected Union start, found {:?}", token),
        };

        let mut output = String::new();
        output.push_str("{");
        output.push_str(&format!("\"type\": \"union\",\n\"union\": {}", union_type as u8));

        if union_type == UnionType::Unset {

            let end_token = self.stream.next()?;
            if end_token != TDFToken::UnionEnd {
                bail!("Expected End of Union, found {:?}", end_token)
            }
            output.push_str("}");

            return Ok(output);

        }

        let union_label = self.stream.next()?;

        match union_label {
            TDFToken::Label(label_string) => {
                output.push_str(",\n");
                output.push_str(&self.write_label(&label_string)?);
            },
            _ => bail!("Expected Label in Union, found {:?}", union_label),
        }
        
        let value = self.stream.next()?;
        output.push_str(",\n\"tag\": ");
        output.push_str(&format!("{}", value.get_tag()?));
        output.push_str(",\n\"valie\": ");
        output.push_str(&self.ser_token(value, level)?);
        output.push_str("}");
        

        let end_token = self.stream.next()?;
        if end_token != TDFToken::UnionEnd {
            bail!("Expected End of Union, found {:?}", end_token)
        }

        Ok(output)
    }

    pub fn ser_generic(&mut self, level: u32) -> Result<String> {

        let token = self.stream.next()?;
        let exist = match token {
            TDFToken::GenericStart(t) => t,
            _ => bail!("Expected Generic start, found {:?}", token),
        };
        let mut output = String::new();
        
        if !exist {
            output.push_str("{ \"type\": \"generic\"}");

            let end_token = self.stream.next()?;
            if end_token != TDFToken::GenericEnd {
                bail!("Expected End of Generic, found {:?}", end_token)
            }

            return Ok(output);

        }

        let tdf_id = self.stream.next()?;
        output.push_str("{ \"type\": \"generic\",");

        match tdf_id {
            TDFToken::Int(id) => {
                output.push_str("\n\"id\": ");
                output.push_str(&self.write_number(id)?);
            },
            _ => bail!("Expected TDFID in Generic, found {:?}", tdf_id),
        }

        let generic_label = self.stream.next()?;

        match generic_label {
            TDFToken::Label(label_string) => {
                output.push_str(",\n");
                output.push_str(&self.write_label(&label_string)?);
            },
            TDFToken::GenericEnd => {
                output.push_str("}");
                return Ok(output);
            }
            _ => bail!("Expected Label in Generic, found {:?}", generic_label),
        };
        
        let value = self.stream.next()?;

        output.push_str("{\n");
        output.push_str(&format!("\"tag\": {},\n", value.get_tag()?));
        output.push_str(&format!("\"value\": "));
        output.push_str(&self.ser_token(value, level)?);
        output.push_str("}");

        let end_token = self.stream.next()?;
        if end_token != TDFToken::GenericEnd {
            bail!("Expected End of Generic, found {:?}", end_token)
        }

        output.push_str("}");

        Ok(output)
    }

    pub fn ser_int_list(&mut self) -> Result<String> {

        let token = self.stream.next()?;
        let size = match token {
            TDFToken::IntListStart(s) => s,
            _ => bail!("Expected Int List start, found {:?}", token),
        };
        
        let mut output = String::new();
        output.push_str("[");

        for i in 0..size {
            output.push_str(&self.ser_int()?);
           if i != size-1 {
                output.push_str(",");
           }
        }
        output.push_str("]");

        let end_token = self.stream.next()?;
        if end_token != TDFToken::IntListEnd {
            bail!("Expected End of Int List, found {:?}", end_token)
        }
        
        Ok(output)
    }

    
    pub fn ser_object_type(&mut self) -> Result<String> {

        let mut output = String::new();
        output.push_str("[");
        output.push_str(&self.ser_int()?);
        output.push_str(",");
        output.push_str(&self.ser_int()?);
        output.push_str("]");

        Ok(output)
    }

    pub fn ser_object_id(&mut self) -> Result<String> {

        let mut output = String::new();
        output.push_str("[");
        output.push_str(&self.ser_int()?);
        output.push_str(",");
        output.push_str(&self.ser_int()?);
        output.push_str(",");
        output.push_str(&self.ser_int()?);
        output.push_str("]");

        Ok(output)
    }

    pub fn ser_float(&mut self) -> Result<String> {

        let token = self.stream.next()?;
        let number = match token {
            TDFToken::Float(f) => f,
            _ => bail!("Expected Int List start, found {:?}", token),
        };

        Ok(format!("{}", number))
    }

    pub fn write_number(&self, mut number: i64) -> Result<String> {
        Ok(format!("{}", number))
    }

    pub fn write_string(&self, string: Vec<u8>) -> Result<String> {
        Ok(format!("\"{}\"", String::from_utf8_lossy(&string).to_string()))
    }

    pub fn write_blob(&self, blob: Vec<u8>) -> Result<String> {
        Ok(format!("\"Blob[{:x?}]\"", blob))
    }

    pub fn write_label(&self, label: &String) -> Result<String> {
        Ok(format!("\"{}\"", label))
    }
}

impl TDFSerializer<String> for JsonSerializer {
    fn serialize(stream: TDFTokenStream, writer: &mut String) -> Result<()> {
        let mut des = Self {
            stream
        };
        let token = des.stream.next()?;
        let str_res = des.ser_token(token, 0)?;
        writer.insert_str(writer.len(), &str_res);
        Ok(())
    }
}
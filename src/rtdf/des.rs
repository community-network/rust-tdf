
use crate::token::*;
use crate::rtdf::{ObjectId, ObjectType, IntList, Union, IpAddress, Localization};

use anyhow::{Result, bail};
use std::fmt;
use std::convert::TryInto;

pub struct RTDFDeserializer {
    pub stream: TDFTokenStream,
}


#[derive(Debug)]
pub enum RTDFDesError {

}

impl std::error::Error for RTDFDesError {}

impl std::fmt::Display for RTDFDesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // match &self {
        //     Self::NotExpectedToken(expected, got) => write!(f, "Expected {:?}, found {:?}", expected, got),
        //     Self::NotEnoughFields => write!(f, "Attempt to read field, but Map ended!"),
        // }
        write!(f, "")
    }
}

impl RTDFDeserializer {
    pub fn new() -> Self {
        Self {
            stream: TDFTokenStream::new()
        }
    }
    pub fn des_field<S: AsRef<str>, D: Deserialize>(&mut self, label: S, value: &mut D) -> Result<()> {
        self.stream.push(TDFToken::Label(label.as_ref().to_owned()));
        self.des_type::<D>()?;
        value.deserialize(self)?;
        Ok(())
    }
    pub fn des_type<D: Deserialize>(&mut self) -> Result<()> {
        self.stream.push(D::TYPE);
        Ok(())
    }
}

/// Des rust struct or primitive
pub trait Deserialize {
    const TYPE: TDFToken;
    fn deserialize(&mut self, des: &mut RTDFDeserializer) -> Result<()>;
}

impl<D: Deserialize> TDFDeserializer<D> for RTDFDeserializer {
    fn deserialize(to_des: &mut D) -> Result<TDFTokenStream> {

        let mut des = Self::new();

        des.des_type::<D>()?;

        to_des.deserialize(&mut des)?;
        
        Ok(des.stream)
    }
}

impl Deserialize for i64 {
    const TYPE: TDFToken = TDFToken::IntType;
    fn deserialize(&mut self, des: &mut RTDFDeserializer) -> Result<()> {
        des.stream.push(TDFToken::Int(*self));
        Ok(())
    }
}

impl Deserialize for u64 {
    const TYPE: TDFToken = TDFToken::IntType;
    fn deserialize(&mut self, des: &mut RTDFDeserializer) -> Result<()> {
        des.stream.push(TDFToken::Int(*self as i64));
        Ok(())
    }
}

impl Deserialize for i32 {
    const TYPE: TDFToken = TDFToken::IntType;
    fn deserialize(&mut self, des: &mut RTDFDeserializer) -> Result<()> {
        des.stream.push(TDFToken::Int(*self as i64));
        Ok(())
    }
}
    
impl Deserialize for u32 {
    const TYPE: TDFToken = TDFToken::IntType;
    fn deserialize(&mut self, des: &mut RTDFDeserializer) -> Result<()> {
        des.stream.push(TDFToken::Int(*self as i64));
        Ok(())
    }
}

impl Deserialize for bool {
    const TYPE: TDFToken = TDFToken::IntType;
    fn deserialize(&mut self, des: &mut RTDFDeserializer) -> Result<()> {
        des.stream.push(TDFToken::Int(if *self { 1 } else { 0 }));
        Ok(())
    }
}

impl Deserialize for String {
    const TYPE: TDFToken = TDFToken::StringType;
    fn deserialize(&mut self, des: &mut RTDFDeserializer) -> Result<()> {
        des.stream.push(TDFToken::String(Vec::from(self.as_bytes())));
        Ok(())
    }
}

impl<D: Deserialize> Deserialize for Vec<D> {
    const TYPE: TDFToken = TDFToken::ListType;
    fn deserialize(&mut self, des: &mut RTDFDeserializer) -> Result<()> {

        des.stream.push(TDFToken::ListStart(self.len()));

        des.des_type::<D>()?;

        for item in self {
            item.deserialize(des)?;
        }

        des.stream.push(TDFToken::ListEnd);

        Ok(())

    }
}

impl Deserialize for IntList {
    const TYPE: TDFToken = TDFToken::IntListType;
    fn deserialize(&mut self, des: &mut RTDFDeserializer) -> Result<()> {

        let list = &mut self.0;

        des.stream.push(TDFToken::IntListStart(list.len()));

        for item in list {
            item.deserialize(des)?;
        }

        des.stream.push(TDFToken::IntListEnd);

        Ok(())

    }
}


impl Deserialize for ObjectId {
    const TYPE: TDFToken = TDFToken::ObjectIdType;
    fn deserialize(&mut self, des: &mut RTDFDeserializer) -> Result<()> {

        self.0.deserialize(des)?;

        self.1.deserialize(des)?;

        self.2.deserialize(des)?;

        Ok(())
    }
}


impl Deserialize for ObjectType {
    const TYPE: TDFToken = TDFToken::ObjectTypeType;
    fn deserialize(&mut self, des: &mut RTDFDeserializer) -> Result<()> {

        self.0.deserialize(des)?;

        self.1.deserialize(des)?;

        Ok(())
    }
}


impl<K: Deserialize, V: Deserialize> Deserialize for Vec<(K, V)> {

    const TYPE: TDFToken = TDFToken::PairListType;

    fn deserialize(&mut self, des: &mut RTDFDeserializer) -> Result<()> {

        des.stream.push(TDFToken::PairListStart(self.len()));

        des.des_type::<K>()?;
        des.des_type::<V>()?;

        for (k, v) in self {

            k.deserialize(des)?;
            v.deserialize(des)?;

        }

        des.stream.push(TDFToken::PairListEnd);

        Ok(())

    }
}

impl Deserialize for f32 {

    const TYPE: TDFToken = TDFToken::FloatType;

    fn deserialize(&mut self, des: &mut RTDFDeserializer) -> Result<()> {
        
        des.stream.push(TDFToken::Float(*self));

        Ok(())
    }
}

impl Deserialize for f64 {

    const TYPE: TDFToken = TDFToken::FloatType;

    fn deserialize(&mut self, des: &mut RTDFDeserializer) -> Result<()> {

        des.stream.push(TDFToken::Float(*self as f32));

        Ok(())
    }
}

impl Deserialize for Vec<u8> {

    const TYPE: TDFToken = TDFToken::BlobType;

    fn deserialize(&mut self, des: &mut RTDFDeserializer) -> Result<()> {

        des.stream.push(TDFToken::Blob(self.clone()));

        Ok(())

    }
}


impl Deserialize for Union {

    const TYPE: TDFToken = TDFToken::UnionType;

    fn deserialize(&mut self, des: &mut RTDFDeserializer) -> Result<()> {

        match self {
            Self::XboxClientAddr { dctx } => {
                des.stream.push(TDFToken::UnionStart(UnionType::XboxClientAddr));
                des.stream.push(TDFToken::Label("DLSC".into()));
                des.stream.push(TDFToken::MapType);
                des.stream.push(TDFToken::MapStart);
                des.des_field("dctx", dctx)?;
                des.stream.push(TDFToken::MapEnd);
                des.stream.push(TDFToken::UnionEnd);
            },
            Self::IpPairAddr { internal, external, mac_addr } => {
                des.stream.push(TDFToken::UnionStart(UnionType::IpPairAddr));
                des.stream.push(TDFToken::Label("VALU".into()));
                des.stream.push(TDFToken::MapType);
                des.stream.push(TDFToken::MapStart);
                des.des_field("INIP", internal)?;
                des.des_field("EXIP", external)?;
                des.des_field("MACI", mac_addr)?;
                des.stream.push(TDFToken::MapEnd);
                des.stream.push(TDFToken::UnionEnd);
            },
            Self::Unset => {
                des.stream.push(TDFToken::UnionStart(UnionType::Unset));
                des.stream.push(TDFToken::UnionEnd);
            },
            _ => bail!("Union Unsupported yet!"),
        }

        Ok(())
    }
}

impl Deserialize for IpAddress {

    const TYPE: TDFToken = TDFToken::MapType;

    fn deserialize(&mut self, des: &mut RTDFDeserializer) -> Result<()> {

        des.stream.push(TDFToken::MapStart);

        
        des.des_field("IP", &mut self.ip)?;
        des.des_field("MACI", &mut self.maci)?;
        des.des_field("PORT", &mut self.port)?;

        des.stream.push(TDFToken::MapEnd);

        Ok(())
    }

}

impl Deserialize for Localization {

    const TYPE: TDFToken = TDFToken::IntType;

    fn deserialize(&mut self, des: &mut RTDFDeserializer) -> Result<()> {

        des.stream.push(TDFToken::Int(u32::from_be_bytes(self.0.as_bytes().try_into()?) as i64));

        Ok(())

    }
}

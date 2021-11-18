
use crate::token::*;
use crate::rtdf::{ObjectId, ObjectType, IntList, Union, IpAddress, Localization};

use anyhow::{Result, bail};
use std::collections::HashMap;
use std::fmt;
use super::des::Deserialize;
use super::Generic;

#[derive(Debug)]
pub enum RTDFSerError {
    NotExpectedToken(TDFToken, TDFToken),
    NotEnoughFields,
}

impl std::error::Error for RTDFSerError {}

impl std::fmt::Display for RTDFSerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::NotExpectedToken(expected, got) => write!(f, "Expected {:?}, found {:?}", expected, got),
            Self::NotEnoughFields => write!(f, "Attempt to read field, but Map ended!"),
        }
    }
}

pub struct RTDFSerializer {
    stream: TDFTokenStream,
}

impl RTDFSerializer {

    pub fn new(stream: TDFTokenStream) -> Self {
        Self {
            stream
        }
    }

    pub fn ser_root(&mut self, prop: &mut dyn RustSerialize) -> Result<()> {
        let token = self.stream.next()?;
        if token != TDFToken::MapType {
            bail!("Unable to serialize tdf root, expected MapType, found {:?}", token);
        }

        prop.serialize(self)?;

        Ok(())
    }



    pub fn ser_field<T: Serialize>(&mut self) -> Result<(String, T)> {

        let mut label = self.stream.next()?;

        if label == TDFToken::MapEnd {

            // Map ended for some reason
            bail!(RTDFSerError::NotEnoughFields);

        } else if label == TDFToken::MapUnion {

            // Skip Union map field declaration
            label = self.stream.next()?;

        }

        let label_string = match label {
            TDFToken::Label(label_string) => label_string,
            _ => bail!(RTDFSerError::NotExpectedToken(TDFToken::Label(String::new()), label)),
        };
        
        let value_type = self.stream.next()?;
        let expected_type = match T::serialize( self) {
            Ok(t) => t,
            Err(e) => bail!("Error serializing field ({}, {:?}): {}", label_string, value_type, e),
        };

        Ok((label_string, expected_type))
    }

    pub fn ser_field_optional<T: Serialize>(&mut self, match_label: TDFToken) -> Result<Option<T>> {
        let label = self.stream.next()?;

        // Move cursor back
        self.stream.1 -= 1;

        if label != match_label {
            return Ok(None)
        }

        let (_, value) = self.ser_field::<T>()?;

        Ok(Some(value))

    }

    /// Get map start token
    pub fn map_start(&mut self) -> Result<()> {
        self.check_token(TDFToken::MapStart)?;
        Ok(())
    }

    /// Get map end token
    pub fn map_end(&mut self) -> Result<()> {
        self.check_token(TDFToken::MapEnd)?;
        Ok(())
    }

    /// Get next token from the stream and check if it is equal to expected one
    pub fn check_token(&mut self, expected_token: TDFToken) -> Result<TDFToken> {
        let token = self.stream.next()?;
        if token != expected_token {
            bail!(RTDFSerError::NotExpectedToken(expected_token, token));
        }
        Ok(token)
    }

}

impl Serialize for i64 {
    fn serialize(ser: &mut RTDFSerializer) -> Result<Self> {
        
        let value = ser.stream.next()?;
        match value {
            TDFToken::Int(v) => {
                return Ok(v);
            },
            _ => {
                bail!("Expected Int, found {:?}", value);
            }
        }
    }
}

impl Serialize for u64 {
    fn serialize(ser: &mut RTDFSerializer) -> Result<Self> {
        let num = i64::serialize(ser)?;
        Ok(num as u64)
    }
}

impl Serialize for i32 {
    fn serialize(ser: &mut RTDFSerializer) -> Result<Self> {
        let num = i64::serialize(ser)?;
        Ok(num as i32)
    }
}

impl Serialize for u32 {
    fn serialize(ser: &mut RTDFSerializer) -> Result<Self> {
        let num = i64::serialize(ser)?;
        Ok(num as u32)
    }
}

impl Serialize for bool {
    fn serialize(ser: &mut RTDFSerializer) -> Result<Self> {
        let num = i64::serialize(ser)?;
        Ok(num == 0)
    }
}

impl<T: Serialize> Serialize for Vec<T> {
    fn serialize(ser: &mut RTDFSerializer) -> Result<Self> {

        let value = ser.stream.next()?;
        let size = match value {
            TDFToken::ListStart(s) => s,
            _ => bail!("Expected List, found {:?}", value),
        };

        let value_type = ser.stream.next()?;


        let mut out = Vec::with_capacity(size);

        for _ in 0..size {
            let expected_type = match T::serialize(ser) {
                Ok(t) => t,
                Err(e) => bail!("Error serializing list item ({:?}): {}", value_type, e),
            };
            out.push(expected_type);
        }

        let end_token = ser.stream.next()?;
        if end_token != TDFToken::ListEnd {
            bail!("Expected ListEnd, found type {:?}", end_token);
        }

        Ok(out)

    }
}

impl<T: Serialize + Default + Copy, const N: usize> Serialize for [T; N] {
    fn serialize(ser: &mut RTDFSerializer) -> Result<Self> {

        let value = ser.stream.next()?;
        let size = match value {
            TDFToken::ListStart(s) => s,
            _ => bail!("Expected List, found {:?}", value),
        };

        if size != N {
            bail!("Expected Array of fixed length {}, but got length {:?}", N, value);
        }

        let value_type = ser.stream.next()?;

        let default_allocation = T::default();
        let mut out: [T; N] = [default_allocation; N];

        for i in 0..size {
            let expected_type = match T::serialize(ser) {
                Ok(t) => t,
                Err(e) => bail!("Error serializing list item ({:?}): {}", value_type, e),
            };
            out[i] = expected_type;
        }

        let end_token = ser.stream.next()?;
        if end_token != TDFToken::ListEnd {
            bail!("Expected ListEnd, found type {:?}", end_token);
        }

        Ok(out)

    }
}

impl<T1: Serialize + core::hash::Hash + Eq, T2: Serialize> Serialize for HashMap<T1, T2> {
    fn serialize(ser: &mut RTDFSerializer) -> Result<Self> {

        let value = ser.stream.next()?;
        let size = match value {
            TDFToken::PairListStart(s) => s,
            _ => bail!("Expected Pair List, found {:?}", value),
        };

        let key_type = ser.stream.next()?;
        let value_type = ser.stream.next()?;


        let mut out = HashMap::with_capacity(size);

        for _ in 0..size {
            let expected_key = match T1::serialize(ser) {
                Ok(t) => t,
                Err(e) => bail!("Error Pair list key ({:?}): {}", key_type, e),
            };
            let expected_value = match T2::serialize(ser) {
                Ok(t) => t,
                Err(e) => bail!("Error Pair list value ({:?}): {}", value_type, e),
            };
            out.insert(expected_key, expected_value);
        }

        let end_token = ser.stream.next()?;
        if end_token != TDFToken::PairListEnd {
            bail!("Expected PairListEnd, found type {:?}", end_token);
        }

        Ok(out)

    }
}

impl Serialize for String {
    fn serialize(ser: &mut RTDFSerializer) -> Result<Self> {
        let value = ser.stream.next()?;
        match value {
            TDFToken::String(v) => {
                return match String::from_utf8(v.clone()) {
                    Ok(s) => Ok(s),
                    Err(_) => Ok(format!("{:?}", v)),
                }
            },
            _ => {
                bail!("Expected String, found {:?}", value);
            }
        }
    }
}

impl Serialize for IntList {
    fn serialize(ser: &mut RTDFSerializer) -> Result<Self> {

        let value = ser.stream.next()?;
        let size = match value {
            TDFToken::IntListStart(s) => s,
            _ => bail!("Expected Int List Start, found {:?}", value),
        };


        let mut out = Vec::with_capacity(size);

        for _ in 0..size {
            let expected_type = match i64::serialize(ser) {
                Ok(t) => t,
                Err(e) => bail!("Error serializing int list item: {}", e),
            };
            out.push(expected_type);
        }

        let end_token = ser.stream.next()?;
        if end_token != TDFToken::IntListEnd {
            bail!("Expected IntListEnd, found type {:?}", end_token);
        }

        Ok(IntList(out))

    }
}



impl Serialize for ObjectId {
    fn serialize(ser: &mut RTDFSerializer) -> Result<Self> {
        let value = ser.stream.next()?;
        let c1 = match value {
            TDFToken::Int(v) => v,
            _ => bail!("Expected Int, found {:?}", value),
        };

        let value = ser.stream.next()?;
        let c2 = match value {
            TDFToken::Int(v) => v,
            _ => bail!("Expected Int, found {:?}", value),
        };

        let value = ser.stream.next()?;
        let c3 = match value {
            TDFToken::Int(v) => v,
            _ => bail!("Expected Int, found {:?}", value),
        };

        return Ok(ObjectId(c1, c2, c3))
    }
}



impl Serialize for ObjectType {
    fn serialize(ser: &mut RTDFSerializer) -> Result<Self> {
        let value = ser.stream.next()?;
        let c1 = match value {
            TDFToken::Int(v) => v,
            _ => bail!("Expected Int, found {:?}", value),
        };

        let value = ser.stream.next()?;
        let c2 = match value {
            TDFToken::Int(v) => v,
            _ => bail!("Expected Int, found {:?}", value),
        };

        return Ok(ObjectType(c1, c2))
    }
}

impl<T1: Serialize, T2: Serialize> Serialize for Vec<(T1, T2)> {
    fn serialize(ser: &mut RTDFSerializer) -> Result<Self> {

        let value = ser.stream.next()?;
        let size = match value {
            TDFToken::PairListStart(s) => s,
            _ => bail!("Expected Pair List, found {:?}", value),
        };

        let key_type = ser.stream.next()?;
        let value_type = ser.stream.next()?;


        let mut out = Vec::with_capacity(size);

        for _ in 0..size {
            let expected_key = match T1::serialize(ser) {
                Ok(t) => t,
                Err(e) => bail!("Error Pair list key ({:?}): {}", key_type, e),
            };
            let expected_value = match T2::serialize(ser) {
                Ok(t) => t,
                Err(e) => bail!("Error Pair list value ({:?}): {}", value_type, e),
            };
            out.push((expected_key, expected_value));
        }

        let end_token = ser.stream.next()?;
        if end_token != TDFToken::PairListEnd {
            bail!("Expected PairListEnd, found type {:?}", end_token);
        }

        Ok(out)

    }
}


impl Serialize for f32 {
    fn serialize(ser: &mut RTDFSerializer) -> Result<Self> {
        
        let value = ser.stream.next()?;
        match value {
            TDFToken::Float(v) => {
                return Ok(v);
            },
            _ => {
                bail!("Expected Float, found {:?}", value);
            }
        }
    }
}

impl Serialize for f64 {
    fn serialize(ser: &mut RTDFSerializer) -> Result<Self> {
        let num = f32::serialize(ser)?;
        Ok(num as f64)
    }
}

impl Serialize for Vec<u8> {
    fn serialize(ser: &mut RTDFSerializer) -> Result<Self> {
        let value = ser.stream.next()?;
        match value {
            TDFToken::Blob(v) => {
                return Ok(v)
            },
            _ => {
                bail!("Expected Blob, found {:?}", value);
            }
        }
    }
}

impl<T: Deserialize + Serialize> Serialize for Generic<T> {

    fn serialize(ser: &mut RTDFSerializer) -> Result<Self> {
        let value = ser.stream.next()?;
        let is_valid = match value {
            TDFToken::GenericStart(t) => t,
            _ => bail!("Expected Generic, found {:?}", value),
        };

        let this = if is_valid {
            
            let id = ser.stream.next()?;
            let tdf_id = match id {
                TDFToken::Int(tdf_id) => {tdf_id},
                _ => {
                    bail!("Unable to serialize union, expected Label, found {:?}", id);
                }
            };

            let label = ser.stream.next()?;
            let label_string = match label {
                TDFToken::Label(label_string) => {label_string},
                _ => {
                    bail!("Unable to serialize union, expected Label, found {:?}", label);
                }
            };

            let value_type = ser.stream.next()?;
            let expected_type = match T::serialize( ser) {
                Ok(t) => t,
                Err(e) => bail!("Error serializing field ({}, {:?}): {}", label_string, value_type, e),
            };
            
            Self(Some((label_string, tdf_id, expected_type)))
        } else {
            Self(None)
        };

        ser.check_token(TDFToken::GenericEnd)?;

        Ok(this)
    }
}

impl Serialize for Union {
    fn serialize(ser: &mut RTDFSerializer) -> Result<Self> {
        let value = ser.stream.next()?;
        let union_type = match value {
            TDFToken::UnionStart(t) => t,
            _ => bail!("Expected Union, found {:?}", value),
        };
        Ok(
            match union_type {
                UnionType::Unset => {
                    ser.check_token(TDFToken::UnionEnd)?;
                    Union::Unset
                },
                UnionType::XboxClientAddr => {

                    let label = ser.stream.next()?;
                    match label {
                        TDFToken::Label(_) => {},
                        _ => {
                            bail!("Unable to serialize union, expected Label, found {:?}", label);
                        }
                    }

                    ser.check_token(TDFToken::MapType)?;
                    ser.map_start()?;

                    let (_, dctx) = ser.ser_field::<i64>()?;

                    ser.map_end()?;
                    ser.check_token(TDFToken::UnionEnd)?;

                    Union::XboxClientAddr {
                        dctx
                    }
                },
                UnionType::IpPairAddr => {

                    let label = ser.stream.next()?;
                    match label {
                        TDFToken::Label(_) => {},
                        _ => {
                            bail!("Unable to serialize union, expected Label, found {:?}", label);
                        }
                    }

                    ser.check_token(TDFToken::MapType)?;
                    ser.map_start()?;

                    let (_, internal) = ser.ser_field::<IpAddress>()?;
                    let (_, external) = ser.ser_field::<IpAddress>()?;
                    let (_, mac_addr) = ser.ser_field::<i64>()?;

                    ser.map_end()?;
                    ser.check_token(TDFToken::UnionEnd)?;

                    Union::IpPairAddr {
                        internal,
                        external,
                        mac_addr,
                    }
                },

                UnionType::IpAddr => {

                    let label = ser.stream.next()?;
                    match label {
                        TDFToken::Label(_) => {},
                        _ => {
                            bail!("Unable to serialize union, expected Label, found {:?}", label);
                        }
                    }

                    ser.check_token(TDFToken::MapType)?;
                    ser.map_start()?;

                    let (_, addr) = ser.ser_field::<IpAddress>()?;

                    ser.map_end()?;
                    ser.check_token(TDFToken::UnionEnd)?;

                    Union::IpAddr {
                        addr,
                    }
                },

                _ => {
                    bail!("This union type {:?} not supported yet!", union_type)
                }
            }
        )

    }
}

impl Serialize for IpAddress {
    fn serialize(ser: &mut RTDFSerializer) -> Result<Self> {

        ser.map_start()?;
        
        let (_, ip)   = ser.ser_field::<u64>()?;
        let (_, maci) = ser.ser_field::<u64>()?;
        let (_, port) = ser.ser_field::<u64>()?;

        ser.map_end()?;

        Ok(
            Self {
                port,
                ip,
                maci
            }
        )
    }
}

impl Serialize for Localization {
    fn serialize(ser: &mut RTDFSerializer) -> Result<Self> {
        let num = u32::serialize(ser)?;
        let loc = String::from_utf8(num.to_be_bytes().to_vec())?;
        Ok(Localization(loc))
    }
}


/// Any Rust type constructor
pub trait RustSerialize {
    fn serialize(&mut self, ser: &mut RTDFSerializer) -> Result<()>;
}

/// Trait to implement on Rust structs and primitives
/// Provides possibility to ser this struct
pub trait Serialize: Sized {
    fn serialize(ser: &mut RTDFSerializer) -> Result<Self>;
}

/// Constructor for Rust structs
pub struct StructConstructor<T: Serialize> {
    inner: Option<T>,
}

impl<T: Serialize> StructConstructor<T> {
    pub fn new() -> Self {
        Self {
            inner: None
        }
    }
    pub fn build(self) -> Result<T> {
        match self.inner {
            Some(inner) => {
                return Ok(inner)
            }
            None => {
                bail!("Attempt to build unserialized rust struct!")
            }
        }
    }
}

impl<T: Serialize> RustSerialize for StructConstructor<T> {
    fn serialize(&mut self, ser: &mut RTDFSerializer) -> Result<()> {
        self.inner = Some(T::serialize(ser)?);
        Ok(())
    }
}

impl<S: RustSerialize> TDFSerializer<S> for RTDFSerializer {
    fn serialize(stream: TDFTokenStream, prop: &mut S) -> Result<()> {
        let mut ser = Self::new(stream);
        ser.ser_root(prop)
    }
}
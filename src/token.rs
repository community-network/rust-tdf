/*
    Rust TDF Tokenization

    While TDF is a binary format, 
    We want to ser/des it into rust primitives and structs

    Tokens Are in the middle of ser/des process
    Binary data is being converted to tokens, and tokens can be 
    converted to rust types or formats like json, xml, etc
*/

use anyhow::{Result, bail};
use num_derive::FromPrimitive;

#[derive(Debug, Clone)]
pub struct TDFTokenStream(pub Vec<TDFToken>, pub usize);

impl TDFTokenStream {
    pub fn new() -> Self {
        Self(Vec::new(), 0)
    }
    pub fn next(&mut self) -> Result<TDFToken> {
        let token = self.get(self.1)?;
        self.1 += 1;
        Ok(token)
    }
    pub fn get(&self, index: usize) -> Result<TDFToken> {
        if index >= self.0.len() {
            bail!("Attemt to read Token at position {} outside of stream bounds!", index);
        }
        Ok(self.0[index].clone())
    }
    pub fn push(&mut self, token: TDFToken) {
        self.0.push(token);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TDFToken {
    /// Indicates Integer type
    IntType,
    /// Integer
    Int(i64),
    /// Indicates String type
    StringType,
    /// Bytes representing a string, 
    /// but not always valid utf-8
    String(Vec<u8>),
    /// Indicates Blob type
    BlobType,
    /// Any sort of raw bytes
    Blob(Vec<u8>),
    /// Indicates Map type
    MapType,
    /// Map starting point
    MapStart,
    /// Label primitive
    Label(String),
    /// Map end marker
    MapEnd,
    /// Special map Marker
    MapUnion,
    /// Indicates List type
    ListType,
    /// Start list with size
    ListStart(usize),
    /// End List
    ListEnd,
    /// Indicates Pair list type
    PairListType,
    /// Pair list with length
    PairListStart(usize),
    /// End the pair list
    PairListEnd,
    /// Indicates Union type
    UnionType,
    /// Some Typed Union start
    UnionStart(UnionType),
    /// End of the Union
    UnionEnd,
    /// Indicates Int List type
    IntListType,
    /// Start of Int list with size
    IntListStart(usize),
    /// End of the int list
    IntListEnd,
    /// Indicates Object type type
    ObjectTypeType,
    /// Object Id type
    ObjectIdType,
    /// Indicates Float type
    FloatType,
    /// Float number
    Float(f32),
    /// Indicates Generic type
    GenericType,
    /// Indicates if Generic exists
    GenericStart(bool),
    /// End of Generic
    GenericEnd,
}

/// Type of Union token
/// Used for indicating network topology
#[derive(Debug, Clone, PartialEq, FromPrimitive, Copy)]
pub enum UnionType {
    /// Client address specific for Xbox
    XboxClientAddr = 0x0,
    /// Server address specific for Xbox
    XboxServerAddr = 0x1,
    /// Pair of IPs
    IpPairAddr     = 0x2,
    /// Info about IP address
    IpAddr         = 0x3,
    /// Address of game server
    HostnameAddr   = 0x4,
    /// None is specified in this Union
    Unset          = 0x7F,
}

/// Serializer writes into stream or data given TDFToken
pub trait TDFSerializer<W> {
    fn serialize(stream: TDFTokenStream, writer: &mut W) -> Result<()>;
}

/// Deserializer produces TDFToken from some stream or data
pub trait TDFDeserializer<R> {
    fn deserialize(reader: &mut R) -> Result<TDFTokenStream>;
}

impl TDFToken {

    /// Get the corresponding tag for the typed token
    pub fn get_tag(&self) -> Result<u8> {
        Ok(match self {
            Self::IntType        => 0,
            Self::StringType     => 1,
            Self::BlobType       => 2,
            Self::MapType        => 3,
            Self::ListType       => 4,
            Self::PairListType   => 5,
            Self::UnionType      => 6,
            Self::IntListType    => 7,
            Self::ObjectTypeType => 8,
            Self::ObjectIdType   => 9,
            Self::FloatType      => 10,
            // time here also
            Self::GenericType    => 12,
            _ => bail!("Attempt to get tag of non-type token!")
        })
    }

    /// Get type token from tag
    pub fn from_tag(tag: u8) -> Result<Self> {
        Ok(match tag {
            0 => Self::IntType,
            1 => Self::StringType,
            2 => Self::BlobType,
            3 => Self::MapType,
            4 => Self::ListType,
            5 => Self::PairListType,
            6 => Self::UnionType,
            7 => Self::IntListType,
            8 => Self::ObjectTypeType,
            9 => Self::ObjectIdType,
            10 => Self::FloatType,
            12 => Self::GenericType,
            _ => bail!("Tag {} doesn't match any known type!", tag)
        })
    }

}
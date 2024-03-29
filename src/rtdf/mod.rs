/*
    Ser/des TDF into and from Rust types
*/

mod ser;
pub use ser::*;

mod des;
pub use des::*;



/// TDF Object type
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ObjectType(pub i64, pub i64);

/// TDF Object id
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ObjectId(pub i64, pub i64, pub i64);

/// TDF Integer list
#[derive(Debug, PartialEq, Clone)]
pub struct IntList(pub Vec<i64>);


/// Network Union
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Union {
    /// Client address specific for Xbox
    XboxClientAddr {
        dctx: i64,
    },
    /// Server address specific for Xbox
    XboxServerAddr {
        // Unknown
    },
    /// Pair of IPs
    IpPairAddr {
        internal: IpAddress,
        external: IpAddress,
        mac_addr: i64,
    },
    /// Info about IP address
    IpAddr {
        addr: IpAddress
    },
    /// Address of game server
    HostnameAddr {
        // Unknown
    },
    /// None is specified in this Union
    Unset,
}


#[derive(Debug, PartialEq, Clone)]
pub struct Localization(pub String);


/// Network IP address
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct IpAddress {
    pub ip: u64,
    pub maci: u64,
    pub port: u64,
}


// pub struct Generic<T: Deserialize + Serialize>(pub Option<(String, i64, T)>);



pub type GenericTdfId = i64;
pub type Label = String;


#[derive(Debug, PartialEq, Clone)]
pub enum Generic {
    Valid(GenericTdfId, GenericContent),
    Invalid
}

#[derive(Debug, PartialEq, Clone)]
pub enum GenericContent {
    Labeled(Label, GenericType),
    Empty,
}

#[derive(Debug, PartialEq, Clone)]
pub enum GenericType {
    Int(i64),
    String(String)
}
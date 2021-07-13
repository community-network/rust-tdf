/*
    Ser/des TDF into and from Rust types
*/

mod ser;
pub use ser::*;

mod des;
pub use des::*;


/// TDF Object type
#[derive(Debug, PartialEq)]
pub struct ObjectType(pub i64, pub i64);

/// TDF Object id
#[derive(Debug, PartialEq)]
pub struct ObjectId(pub i64, pub i64, pub i64);

/// TDF Integer list
#[derive(Debug, PartialEq)]
pub struct IntList(pub Vec<i64>);


/// Network Union
#[derive(Debug, PartialEq)]
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


#[derive(Debug, PartialEq)]
pub struct Localization(pub String);


/// Network IP address
#[derive(Debug, PartialEq)]
pub struct IpAddress {
    pub port: i64,
    pub ip: i64,
    pub maci: i64
}
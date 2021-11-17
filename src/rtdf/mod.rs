/*
    Ser/des TDF into and from Rust types
*/

mod ser;
pub use ser::*;

mod des;
pub use des::*;



/// TDF Object type:
/// Type for some Object in blaze hub
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ObjectType(pub i64, pub i64);


/// TDF Object id:
/// Tripple value used to identify objects in EA blaze hub
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ObjectId(pub i64, pub i64, pub i64);


/// TDF Integer list:
/// Specially defined to not interf with list type
#[derive(Debug, PartialEq, Clone)]
pub struct IntList(pub Vec<i64>);


/**
Defines some localization, like "enUS"

Check more at https://www.oracle.com/java/technologies/javase/jdk8-jre8-suported-locales.html

This localization is strictly 4 symbols
*/
#[derive(Debug, PartialEq, Clone)]
pub struct Localization(pub String);


/// Tells that this union is Invalid and need to be skipped
pub const UNION_INVALID: u32 = 127;


/**
    Union type, simmilar to rust Enum

    **IMPORTAN:** Rust has `union` C-like type, but it is different! (And unsafe.)

    **NOTE:** You can define it as Union<T> and use like Union {...}

    But it prefered to use it with derive macro #[derive(Union)]
    It will be mapped by ID ("valu" tag than auto-used), 
    current name (if not longer than 4 symbols) or #[rename("TAG")]

 */
pub struct TDFUnion<T: Deserialize + Serialize> {
    // value ID or UNION_INVALID ID
    id: u32,
    // K, V
    value: Option<(String, T)>,
}

/// Use this in Macro to allow Enum <-> Union convertation on Ser/des
pub trait AsUnion {
    fn union_id(&self) -> u32;
    fn union_tag(&self) -> String;
    fn union_match(id: u32, value: (String, T)) -> Self;
}

impl<T: AsUnion> From<Option<T>> for TDFUnion<T> {
    fn from(into: Option<T>) -> Self {
        match into {
            None => Self {
                id: UNION_INVALID,
                value: None
            },
            Some(value) => Self {
                id: value.union_id(),
                value: Some((value.union_tag(), value)),
            }
        }
    }
}

impl<T: AsUnion> From<TDFUnion<T>> for Option<T> {
    fn from(into: TDFUnion<T>) -> Self {
        match into.id {
            UNION_INVALID => None,
            _ => Some(T::union_match(into.id, into.value.unwrap()))
        }
    }
}

pub struct One {

}

pub struct Two {
    
}


enum A {
    Beta(One),
    Omega(Two),
}


impl AsUnion for A {
    fn union_id(&self) -> u32 {
        match self {
            Self::Beta => 0,
            Self::Omega => 1,
        }
    }
    fn union_tag(&self) -> String {
        return "".into();
    }
    fn union_match(id: u32, value: (String, Self)) -> Self {
        value.1
    }
}


// /// Network Union
// #[derive(Debug, PartialEq, Clone, Copy)]
// pub enum Union {
//     /// Client address specific for Xbox
//     XboxClientAddr {
//         dctx: i64,
//     },
//     /// Server address specific for Xbox
//     XboxServerAddr {
//         // Unknown
//     },
//     /// Pair of IPs
//     IpPairAddr {
//         internal: IpAddress,
//         external: IpAddress,
//         mac_addr: i64,
//     },
//     /// Info about IP address
//     IpAddr {
//         addr: IpAddress
//     },
//     /// Address of game server
//     HostnameAddr {
//         // Unknown
//     },
//     /// None is specified in this Union
//     Unset,
// }





// /// Network IP address
// #[derive(Debug, PartialEq, Clone, Copy)]
// pub struct IpAddress {
//     pub ip: u64,
//     pub maci: u64,
//     pub port: u64,
// }
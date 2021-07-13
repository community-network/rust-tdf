
pub mod token;
pub mod btdf;
pub mod rtdf;

extern crate macro_tdf;


/// Basic imports to use the library as rust <-> bin tdf ser/des
pub mod prelude {

    // Macro
    pub use macro_tdf::*;

    // Ser/des rust tdf
    pub use crate::rtdf::{RTDFDeserializer, RTDFSerializer, Deserialize, Serialize, StructConstructor, ObjectType, ObjectId, IntList, Union, Localization, IpAddress};

    // Ser/des defenitions
    pub use crate::token::{TDFSerializer, TDFDeserializer, TDFTokenStream, TDFToken};

    // Important for results in des/ser
    pub use anyhow::Result;

}


use btdf::peekreader::{PeekRead};
use btdf::{BTDFDeserializer, BTDFSerializer};
use rtdf::{Deserialize, RTDFSerializer, Serialize, StructConstructor, RTDFDeserializer};
use token::{TDFSerializer, TDFDeserializer};
use anyhow::Result;
use std::io::Write;


/// Performs TDF binary to rust strcut conversion
pub fn bin_to_struct<T: Serialize, R: PeekRead + Sized>(reader: &mut R) -> Result<T>  {
    // Conver bin into token stream
    let stream = BTDFDeserializer::deserialize(reader)?;
    // Init Struct builder
    let mut sc = StructConstructor::<T>::new();
    // Ser given struct
    RTDFSerializer::serialize(stream, &mut sc)?;
    // Buid it
    sc.build()
}

/// Performs rust struct to tdf bin stream conversion
pub fn struct_to_bin<D: Deserialize, W: Write>(structure: &mut D, writer: &mut W) -> Result<()>  {
    let stream = RTDFDeserializer::deserialize(structure)?;
    BTDFSerializer::serialize(stream, writer)?;
    Ok(())
}



#[cfg(test)]
mod tests {

    use crate::btdf::peekreader::PeekReader;
    use crate::prelude::*;
    use crate::{struct_to_bin, bin_to_struct};
    use std::io::Cursor;
    use std::fmt::Debug;

    #[derive(Pack, Debug, PartialEq)]
    struct TestNumbers {
        a: i64,
        b: i32,
        c: u64,
        d: u32,
    }

    #[derive(Pack, Debug, PartialEq)]
    struct TestBasic {
        a: String,
        b: Vec<u8>,
        c: Vec<i32>,
        d: IntList,
    }

    #[derive(Pack, Debug, PartialEq)]
    struct TestCustom {
        a: Localization,
        b: ObjectType,
        c: ObjectId,
    }

    #[derive(Pack, Debug, PartialEq)]
    struct TestUnions {
        a: Union,
        b: Union,
        c: Union,
    }


    pub fn test_bi_direct<T: Deserialize + Serialize + PartialEq + Debug>(mut input: T) -> Result<()> {
        let test_vector: Vec<u8> = vec![];
        let mut rw_cursor = Cursor::new(test_vector);
        struct_to_bin(&mut input, &mut rw_cursor)?;
        rw_cursor.set_position(0);
        let tested_struct = bin_to_struct::<T, PeekReader<Cursor<Vec<u8>>>>(&mut PeekReader::new(rw_cursor))?;
        assert_eq!(tested_struct, input);
        Ok(())
    }

    impl TestNumbers {
        fn new() -> Self {
            Self {
                a: -78,
                b: 543654,
                c: 0,
                d: 456,
            }
        }
    }

    impl TestBasic {
        fn new() -> Self {
            Self {
                a: "frostbite-test-string".into(),
                b: vec![0, 45, 255, 6, 0],
                c: vec![7894, 45, -6543, 56],
                d: IntList(vec![675, 5, 6, -1]),
            }
        }
    }

    impl TestCustom {
        fn new() -> Self {
            Self {
                a: Localization("enUS".to_string()),
                b: ObjectType(34, 56),
                c: ObjectId(1, 2, 3),
            }
        }
    }

    impl TestUnions {
        fn new() -> Self {
            Self {
                a: Union::Unset,
                b: Union::XboxClientAddr { dctx: 3 },
                c: Union::IpPairAddr { internal: IpAddress { ip: 34, port: 0, maci: 0 }, external: IpAddress { ip: 34, port: 0, maci: 0 }, mac_addr: 80 },
            }
        }
    }

    #[test]
    fn numbers_test() {
        test_bi_direct(TestNumbers::new()).unwrap();
    }

    #[test]
    fn customs_test() {
        test_bi_direct(TestCustom::new()).unwrap();
    }

    #[test]
    fn basic_test() {
        test_bi_direct(TestBasic::new()).unwrap();
    }

    #[test]
    fn unions_test() {
        test_bi_direct(TestUnions::new()).unwrap();
    }

}
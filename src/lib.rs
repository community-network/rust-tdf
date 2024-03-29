
pub mod token;
pub mod btdf;
pub mod rtdf;
pub mod json;
//pub mod auto;

extern crate macro_tdf;


/// Basic imports to use the library as rust <-> bin tdf ser/des
pub mod prelude {

    // Macro
    pub use macro_tdf::*;

    // Ser/des rust tdf
    pub use crate::rtdf::{Generic, GenericContent, GenericType, RTDFDeserializer, RTDFSerializer, Deserialize, Serialize, StructConstructor, ObjectType, ObjectId, IntList, Union, Localization, IpAddress};

    // Ser/des defenitions
    pub use crate::token::{TDFSerializer, TDFDeserializer, TDFTokenStream, TDFToken};

    // Important for results in des/ser
    pub use anyhow::Result;

}


use btdf::{BTDFDeserializer, BTDFSerializer};
use json::JsonSerializer;
use rtdf::{Deserialize, RTDFSerializer, Serialize, StructConstructor, RTDFDeserializer};
use token::{TDFSerializer, TDFDeserializer};
use anyhow::Result;
use std::io::{Write, Read, Seek};
//use auto::HelpSerializer;

/// Performs TDF binary to rust strcut conversion
pub fn bin_to_struct<T: Serialize, R: Read + Seek+ Sized>(reader: &mut R) -> Result<T>  {
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

/// Performs TDF binary to json strcut conversion
pub fn bin_to_json< R: Read + Seek+ Sized>(reader: &mut R) -> Result<String>  {
    // Conver bin into token stream
    let stream = BTDFDeserializer::deserialize(reader)?;
    let mut sc = String::new();
    // Ser given struct
    JsonSerializer::serialize(stream, &mut sc)?;
    Ok(sc)
}

// /// Auto generates Rust pseudo code for given binary stream
// pub fn auto_gen_from_bin<R: Read + Seek+ Sized>(reader: &mut R) -> Result<String>  {
//     // Conver bin into token stream
//     let stream = BTDFDeserializer::deserialize(reader)?;

//     let mut sc = String::new();

//     HelpSerializer::serialize(stream, &mut sc)?;

//     Ok(sc)
// } 

#[cfg(test)]
mod tests {

    use peekread::{SeekPeekReader};
    use crate::{prelude::*, bin_to_json};
    use crate::{struct_to_bin, bin_to_struct};
    use std::collections::HashMap;
    use std::io::Cursor;
    use std::fmt::Debug;


    
    #[derive(Pack, Debug, PartialEq)]
    struct TestOptional {
        a: Option<i32>,
    }

    #[derive(Pack, Debug, PartialEq)]
    struct TestNumbers {
        a: i64,
        c: u64,
        b: i32,
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

    // #[derive(Pack, Debug, PartialEq)]
    // enum TestEnum {
    //     Alpha,
    //     Beta = 0x34
    // }

    // #[derive(Pack, Debug, PartialEq)]
    // struct TestUnumsInStruct {
    //     a: TestEnum,
    //}

    pub fn test_bi_direct<T: Deserialize + Serialize + PartialEq + Debug>(mut input: T) -> Result<()> {
        let test_vector: Vec<u8> = vec![];
        let mut rw_cursor = Cursor::new(test_vector);
        struct_to_bin(&mut input, &mut rw_cursor)?;
        rw_cursor.set_position(0);
        let tested_struct = bin_to_struct::<T, SeekPeekReader<Cursor<Vec<u8>>>>(&mut SeekPeekReader::new(rw_cursor))?;
        assert_eq!(tested_struct, input);
        Ok(())
    }


    pub fn test_json<T: Deserialize + Serialize + PartialEq + Debug>(mut input: T) -> Result<()> {
        let test_vector: Vec<u8> = vec![];
        let mut rw_cursor = Cursor::new(test_vector);
        struct_to_bin(&mut input, &mut rw_cursor)?;
        rw_cursor.set_position(0);
        let json_string = bin_to_json::<SeekPeekReader<Cursor<Vec<u8>>>>(&mut SeekPeekReader::new(rw_cursor))?;
        println!("{}", json_string);
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
                a: "crossplayGames".into(),
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

    #[test]
    fn hash_map_test() {

        #[derive(Pack, Debug, PartialEq)]
        struct Test {
            map: HashMap<u32, String>,
        }

        let mut test_case = HashMap::new();

        test_case.insert(1, "seven".to_string());
        test_case.insert(2, "five".to_string());

        test_bi_direct(Test { map: test_case }).unwrap();
    }

    #[test]
    fn array_test() {

        #[derive(Pack, Debug, PartialEq)]
        struct Test {
            array: [u32; 3],
        }

        test_bi_direct(Test { array: [3, 3, 5] }).unwrap();
    }

    
    #[test]
    fn generic_test() -> Result<()> {

        
        use simple_logger::SimpleLogger;
        SimpleLogger::new().init().unwrap();
    
        // Use Info, Debug, Trace to see more
        log::set_max_level(log::LevelFilter::Trace);

        #[derive(Pack, Debug, PartialEq)]
        struct Test {
            alph: Generic,
            beta: Generic,
            gama: Generic,
        }

        test_bi_direct(Test { 
            alph: Generic::Valid(0x06, GenericContent::Empty),
            beta: Generic::Invalid,
            gama: Generic::Valid(0x05, GenericContent::Labeled("GAMA".into(), GenericType::Int(34))),
        })
    }

    #[test]
    fn ea_bug_fix_test() {

        // use simple_logger::SimpleLogger;
        // SimpleLogger::new().init().unwrap();
    
        // // Use Info, Debug, Trace to see more
        // log::set_max_level(log::LevelFilter::Trace);

        #[derive(Pack, Debug, PartialEq)]
        struct Test {
            gbra: Vec<(u32, Vec<(u32, String)>)>,
            msid: Vec<(u32, Vec<(u32, String)>)>,
            pelm: Vec<(String, Vec<String>)>,
        }

        test_bi_direct(Test { 
            gbra: vec![(1, vec![(2, "test".into())])],
            msid: vec![(1, vec![(2, "test".into())])],
            pelm: vec![("a".into(), vec!["test".into()])],
        }).unwrap();
    }


    #[test]
    fn array_test_json() {

        #[derive(Pack, Debug, PartialEq)]
        struct TestCustomJson {
            a: String,
            b: Vec<u8>,
            c: Vec<i32>,
            d: IntList,
            e: Localization,
            f: ObjectType,
            g: ObjectId,
            t: TestBasic,
            unio: TestUnions,
            gama: Generic,
            alph: Generic,
            beta: Generic,
        }

        impl TestCustomJson {
            pub fn new() -> Self {
                Self {
                    a: "test-frostbite string".into(),
                    b: vec![0xf, 0xf, 0xc],
                    c: vec![12, 34325, 0],
                    d: IntList(vec![675, 5, 6, -1]),
                    e: Localization("enUS".to_string()),
                    f: ObjectType(34, 56),
                    g: ObjectId(1, 2, 3),
                    t: TestBasic::new(),
                    unio: TestUnions::new(),
                    gama: Generic::Valid(0x05, GenericContent::Labeled("GAMA".into(), GenericType::Int(34))),
                    alph: Generic::Valid(0x06, GenericContent::Empty),
                    beta: Generic::Invalid,
                }
            }
        }

        test_json(TestCustomJson::new()).unwrap();
    }


}
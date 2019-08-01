use std::cmp::Ordering;
use std::fs::File;
use std::io::{BufReader, Cursor, Read, Seek, SeekFrom};

use byteorder::{BigEndian, ReadBytesExt, ByteOrder};
use flate2::bufread::ZlibDecoder;

#[derive(Eq)]
struct ChunkLocation {
    offset: u32,
    sector_num: u8
}

impl ChunkLocation {
    fn get_offset_byte(&self) -> u32 {
        self.offset * 4096
    }
}

impl PartialOrd for ChunkLocation {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.offset.partial_cmp(&other.offset)
    }
}

impl Ord for ChunkLocation {
    fn cmp(&self, other: &Self) -> Ordering {
        self.offset.cmp(&other.offset)
    }
}

impl PartialEq for ChunkLocation {
    fn eq(&self, other: &Self) -> bool {
        self.offset == other.offset && self.sector_num == other.sector_num
    }
}

struct Region {

}

impl Region {
    fn load_mca() {
        let mut reader = BufReader::new(File::open("resources/mca/r.0.0.mca").unwrap());
        let mut chunk_location_vec = Vec::<ChunkLocation>::new();

        for _ in 0..1024 {
            let mut buffer: [u8; 3] = [0; 3];
            reader.read(&mut buffer).unwrap();
            let current = [&[0], &buffer[..]].concat();
            let mut rdr = Cursor::new(current);
            let result = rdr.read_u32::<BigEndian>().unwrap();

            let mut buffer: [u8; 1] = [0];
            reader.read(&mut buffer).unwrap();

            chunk_location_vec.push(ChunkLocation {offset: result, sector_num: buffer[0]});

        }

        chunk_location_vec.sort();

        let offset = chunk_location_vec[0].get_offset_byte();

        let _ = reader.seek(SeekFrom::Start(offset as u64));

        let mut buffer: [u8; 4] = [0; 4];
        let _ = reader.read_exact(&mut buffer);
        let length = (&buffer[..]).read_u32::<BigEndian>().unwrap();

        let mut buffer: [u8; 1] = [0];
        let _ = reader.read_exact(&mut buffer).unwrap();
        let compress_type = buffer[0];

        match compress_type {
            1 => {
                unimplemented!()
            },
            2 => {
                println!("{}", length);
                let vec = reader.bytes().take((length - 1) as usize).map(|e| e.unwrap()).collect::<Vec<u8>>();
                let mut d = ZlibDecoder::new(&vec[..]);

                let mut flate_vec = Vec::new();
                loop {
                    let mut buffer: [u8; 1024] = [0; 1024];
                    match d.read(&mut buffer).unwrap() {
                        0 => break,
                        n => {
                            println!("{}", n);
                            flate_vec.append(&mut buffer[0..n].to_vec());
                        }
                    }
                }

                println!("{}:{:?}", flate_vec.len(), flate_vec);
            },
            _ => {
                panic!("unknown type");
            }
        }


    }
}

struct NbtParser<'a> {
    nbt_slice: &'a[u8]
}

impl <'a> NbtParser<'a> {
    fn new(nbt_slice: &[u8]) -> NbtParser {
        NbtParser {nbt_slice}
    }

    fn parse(&self) -> NbtTag {
        let mut cur = Cursor::new(self.nbt_slice);
        let mut t = [0u8; 1];
        let _ = cur.read(&mut t);
        match t[0] {
            0 => {
                NbtTag::End
            },
            1 => {
                let mut name_byte_num_buffer = [0u8; 2];
                let _ = cur.read(&mut name_byte_num_buffer);

                let name_byte_num = (&name_byte_num_buffer[..]).read_u16::<BigEndian>().unwrap();
                let mut vec = cur.bytes().take((name_byte_num + 1) as usize).map(|e| e.unwrap()).collect::<Vec<u8>>();
                let value = vec.split_off(name_byte_num as usize);
                let name = String::from_utf8(vec).unwrap();

                NbtTag::Byte(name, value[0] as i8)
            },
            2 => {
                let mut name_byte_num_buffer = [0u8; 2];
                let _ = cur.read(&mut name_byte_num_buffer);

                let name_byte_num = (&name_byte_num_buffer[..]).read_u16::<BigEndian>().unwrap();
                let mut vec = cur.bytes().take((name_byte_num + 2) as usize).map(|e| e.unwrap()).collect::<Vec<u8>>();
                let value = vec.split_off(name_byte_num as usize);
                let name = String::from_utf8(vec).unwrap();

                NbtTag::Short(name, (&value[..]).read_u16::<BigEndian>().unwrap() as i16)
            },
            3 => {
                let mut name_byte_num_buffer = [0u8; 2];
                let _ = cur.read(&mut name_byte_num_buffer);

                let name_byte_num = (&name_byte_num_buffer[..]).read_u16::<BigEndian>().unwrap();
                let mut vec = cur.bytes().take((name_byte_num + 4) as usize).map(|e| e.unwrap()).collect::<Vec<u8>>();
                let value = vec.split_off(name_byte_num as usize);
                let name = String::from_utf8(vec).unwrap();

                NbtTag::Int(name, (&value[..]).read_u32::<BigEndian>().unwrap() as i32)
            },
            4 => {
                let mut name_byte_num_buffer = [0u8; 2];
                let _ = cur.read(&mut name_byte_num_buffer);

                let name_byte_num = (&name_byte_num_buffer[..]).read_u16::<BigEndian>().unwrap();
                let mut vec = cur.bytes().take((name_byte_num + 8) as usize).map(|e| e.unwrap()).collect::<Vec<u8>>();
                let value = vec.split_off(name_byte_num as usize);
                let name = String::from_utf8(vec).unwrap();

                NbtTag::Long(name, (&value[..]).read_u64::<BigEndian>().unwrap() as i64)
            },
            5 => {
                let mut name_byte_num_buffer = [0u8; 2];
                let _ = cur.read(&mut name_byte_num_buffer);

                let name_byte_num = (&name_byte_num_buffer[..]).read_u16::<BigEndian>().unwrap();
                let mut vec = cur.bytes().take((name_byte_num + 4) as usize).map(|e| e.unwrap()).collect::<Vec<u8>>();
                let value = vec.split_off(name_byte_num as usize);
                let name = String::from_utf8(vec).unwrap();

                NbtTag::Float(name, (&value[..]).read_f32::<BigEndian>().unwrap())
            },
            6 => {
                let mut name_byte_num_buffer = [0u8; 2];
                let _ = cur.read(&mut name_byte_num_buffer);

                let name_byte_num = (&name_byte_num_buffer[..]).read_u16::<BigEndian>().unwrap();
                let mut vec = cur.bytes().take((name_byte_num + 8) as usize).map(|e| e.unwrap()).collect::<Vec<u8>>();
                let value = vec.split_off(name_byte_num as usize);
                let name = String::from_utf8(vec).unwrap();

                NbtTag::Double(name, (&value[..]).read_f64::<BigEndian>().unwrap())
            },
            7 => {
                let mut name_byte_num_buffer = [0u8; 2];
                let before_position = cur.position();
                let _ = cur.read(&mut name_byte_num_buffer);

                let name_byte_num = (&name_byte_num_buffer[..]).read_u16::<BigEndian>().unwrap();
                let mut vec = cur.bytes().take((name_byte_num + 4) as usize).map(|e| e.unwrap()).collect::<Vec<u8>>();
                let array_num = vec.split_off(name_byte_num as usize);
                let array_num = (&array_num[..]).read_u32::<BigEndian>().unwrap();
                let name = String::from_utf8(vec).unwrap();

                cur = Cursor::new(self.nbt_slice);
                cur.seek(SeekFrom::Start(before_position + 2 + name_byte_num as u64 + 4));

                let array = cur.bytes().take((array_num) as usize).map(|e| e.unwrap()).collect::<Vec<u8>>();

                NbtTag::ByteArray(name, array)

            },
            8 => {
                let mut name_byte_num_buffer = [0u8; 2];
                let before_position = cur.position();
                let _ = cur.read(&mut name_byte_num_buffer);

                let name_byte_num = (&name_byte_num_buffer[..]).read_u16::<BigEndian>().unwrap();
                let mut vec = cur.bytes().take((name_byte_num + 2) as usize).map(|e| e.unwrap()).collect::<Vec<u8>>();
                let value_num = vec.split_off(name_byte_num as usize);
                let value_num = (&value_num[..]).read_u16::<BigEndian>().unwrap();
                let name = String::from_utf8(vec).unwrap();

                cur = Cursor::new(self.nbt_slice);
                cur.seek(SeekFrom::Start(before_position + 2 + name_byte_num as u64 + 2));

                let array = cur.bytes().take((value_num) as usize).map(|e| e.unwrap()).collect::<Vec<u8>>();
                let value = String::from_utf8(array).unwrap();

                NbtTag::String(name, value)
            },
            9 => {
                let mut name_byte_num_buffer = [0u8; 2];
                let before_position = cur.position();
                let _ = cur.read(&mut name_byte_num_buffer);

                let name_byte_num = (&name_byte_num_buffer[..]).read_u16::<BigEndian>().unwrap();
                let mut vec = cur.bytes().take((name_byte_num) as usize).map(|e| e.unwrap()).collect::<Vec<u8>>();
                let name = String::from_utf8(vec).unwrap();

                cur = Cursor::new(self.nbt_slice);
                cur.seek(SeekFrom::Start(before_position + 2 + name_byte_num as u64));

                let tag_type = cur.read_u8().unwrap();
                let item_num = cur.read_u32::<BigEndian>().unwrap();
                match tag_type {
                    1 => {
                        let mut item_vec = Vec::new();
                        for i in 0..item_num {
                            let item = cur.read_u8().unwrap();
                            item_vec.push(NbtTag::Byte("".to_string(), item as i8));
                        }
                        NbtTag::List(name, item_vec)
                    },
                    2 => {
                        let mut item_vec = Vec::new();
                        for i in 0..item_num {
                            let item = cur.read_u16::<BigEndian>().unwrap();
                            item_vec.push(NbtTag::Short("".to_string(), item as i16));
                        }
                        NbtTag::List(name, item_vec)
                    },
                    3 => {
                        let mut item_vec = Vec::new();
                        for i in 0..item_num {
                            let item = cur.read_u32::<BigEndian>().unwrap();
                            item_vec.push(NbtTag::Int("".to_string(), item as i32));
                        }
                        NbtTag::List(name, item_vec)
                    },
                    4 => {
                        let mut item_vec = Vec::new();
                        for i in 0..item_num {
                            let item = cur.read_u64::<BigEndian>().unwrap();
                            item_vec.push(NbtTag::Long("".to_string(), item as i64));
                        }
                        NbtTag::List(name, item_vec)
                    },
                    5 => {
                        let mut item_vec = Vec::new();
                        for i in 0..item_num {
                            let item = cur.read_f32::<BigEndian>().unwrap();
                            item_vec.push(NbtTag::Float("".to_string(), item));
                        }
                        NbtTag::List(name, item_vec)
                    }
                    _ => {
                        unimplemented!()
                    }
                }
            }
            _ => {
                unimplemented!();
            }
        }
    }
}

#[derive(PartialEq, Debug)]
enum NbtTag {
    End,
    Byte(String, i8),
    Short(String, i16),
    Int(String, i32),
    Long(String, i64),
    Float(String, f32),
    Double(String, f64),
    ByteArray(String, Vec<u8>),
    String(String, String),
    List(String, Vec<NbtTag>)
}

#[cfg(test)]
mod tests {
    use super::{NbtParser, Region, NbtTag};

    #[test]
    fn test() {
        Region::load_mca();
    }

    #[test]
    fn test_nbt_parser() {
        NbtParser::new(&[0]);
    }

    #[test]
    fn test_nbt_end() {
        let parser = NbtParser::new(&[0]);
        assert_eq!(parser.parse(), NbtTag::End);
    }

    #[test]
    fn test2() {
        print!("{:?}", "foo".as_bytes());
    }

    #[test]
    fn test_nbt_byte() {
        let parser = NbtParser::new(&[1, 0, 3, 102, 111, 111, 3]);

        assert_eq!(parser.parse(), NbtTag::Byte("foo".to_string(), 3));
    }

    #[test]
    fn test_nbt_short() {
        let parser = NbtParser::new(&[2, 0, 3, 102, 111, 111, 1, 1]);

        assert_eq!(parser.parse(), NbtTag::Short("foo".to_string(), 257));
    }

    #[test]
    fn test_nbt_int() {
        let parser = NbtParser::new(&[3, 0, 3, 102, 111, 111, 1, 1, 1, 1]);

        assert_eq!(parser.parse(), NbtTag::Int("foo".to_string(), 16843009));
    }

    #[test]
    fn test_nbt_long() {
        let parser = NbtParser::new(&[4, 0, 3, 102, 111, 111, 1, 1, 1, 1, 1, 1, 1, 1]);

        assert_eq!(parser.parse(), NbtTag::Long("foo".to_string(), 72340172838076673));
    }

    #[test]
    fn test_float() {
        println!("{}", 1.0f32.to_bits());
    }

    #[test]
    fn test_nbt_float() {
        let parser = NbtParser::new(&[5, 0, 3, 102, 111, 111, 63, 128, 0, 0]);

        assert_eq!(parser.parse(), NbtTag::Float("foo".to_string(), 1f32));
    }

    #[test]
    fn test_double() {
        println!("{}", 1.0f64.to_bits());
    }

    #[test]
    fn test_nbt_double() {
        let parser = NbtParser::new(&[6, 0, 3, 102, 111, 111, 63, 240, 0, 0, 0, 0, 0, 0]);

        assert_eq!(parser.parse(), NbtTag::Double("foo".to_string(), 1f64));
    }

    #[test]
    fn test_nbt_byte_array() {
        let parser = NbtParser::new(&[7, 0, 3, 102, 111, 111, 0, 0, 0, 3, 1, 2, 3]);

        assert_eq!(parser.parse(), NbtTag::ByteArray("foo".to_string(), vec![1, 2, 3]));
    }

    #[test]
    fn test_nbt_string() {
        let parser = NbtParser::new(&[8, 0, 3, 102, 111, 111, 0, 6, 227, 129, 187, 227, 129, 146]);

        assert_eq!(parser.parse(), NbtTag::String("foo".to_string(), "ほげ".to_string()));
    }


    mod nbt_tag_list {
        use super::super::{NbtParser, NbtTag};

        #[test]
        fn test_nbt_list_byte() {
            let parser = NbtParser::new(&[9, 0, 3, 102, 111, 111, 1, 0, 0, 0, 2, 1, 2]);

            assert_eq!(parser.parse(), NbtTag::List("foo".to_string(), vec![NbtTag::Byte("".to_string(), 1), NbtTag::Byte("".to_string(), 2)]));
        }

        #[test]
        fn test_nbt_list_short() {
            let parser = NbtParser::new(&[9, 0, 3, 102, 111, 111, 2, 0, 0, 0, 2, 0, 1, 0, 2]);

            assert_eq!(parser.parse(), NbtTag::List("foo".to_string(), vec![NbtTag::Short("".to_string(), 1), NbtTag::Short("".to_string(), 2)]));
        }

        #[test]
        fn test_nbt_list_int() {
            let parser = NbtParser::new(&[9, 0, 3, 102, 111, 111, 3, 0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0, 2]);

            assert_eq!(parser.parse(), NbtTag::List("foo".to_string(), vec![NbtTag::Int("".to_string(), 1), NbtTag::Int("".to_string(), 2)]));
        }

        #[test]
        fn test_nbt_list_long() {
            let parser = NbtParser::new(&[9, 0, 3, 102, 111, 111, 4, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2]);

            assert_eq!(parser.parse(), NbtTag::List("foo".to_string(), vec![NbtTag::Long("".to_string(), 1), NbtTag::Long("".to_string(), 2)]));
        }

        #[test]
        fn test_nbt_list_float() {
            let parser = NbtParser::new(&[9, 0, 3, 102, 111, 111, 5, 0, 0, 0, 2, 63, 128, 0, 0, 63, 128, 0, 0]);

            assert_eq!(parser.parse(), NbtTag::List("foo".to_string(), vec![NbtTag::Float("".to_string(), 1f32), NbtTag::Float("".to_string(), 1f32)]));
        }

        #[test]
        fn test_nbt_list_double() {
            let parser = NbtParser::new(&[9, 0, 3, 102, 111, 111, 6, 0, 0, 0, 2, 63, 240, 0, 0, 0, 0, 0, 0, 63, 240, 0, 0, 0, 0, 0, 0]);

            assert_eq!(parser.parse(), NbtTag::List("foo".to_string(), vec![NbtTag::Double("".to_string(), 1f64), NbtTag::Double("".to_string(), 1f64)]));
        }

    }

}
use std::cmp::Ordering;
use std::fs::File;
use std::io::{BufReader, Cursor, Read, Seek, SeekFrom};

use byteorder::{BigEndian, ReadBytesExt};
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
                let parser = NbtParser::new(&flate_vec[..]);
                let nbt_tag = parser.parse();

                println!("{:?}", nbt_tag);
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

    fn parse(&self) -> (NbtTag, usize) {
        let mut cur = Cursor::new(self.nbt_slice);
        let tag_type = cur.read_u8().unwrap();
        match tag_type {
            0 => {
                (NbtTag::End, 1)
            }
            n @ 1...11 => {
                let name_byte_num = cur.read_u16::<BigEndian>().unwrap();

                let position = cur.position();
                let vec = cur.bytes().take((name_byte_num) as usize).map(|e| e.unwrap()).collect::<Vec<u8>>();
                let name = String::from_utf8(vec).unwrap();

                println!("{}: {}", n, name);
                let (tag, value_size) = self.parse_value(name, n, position + name_byte_num as u64);

                (tag, 1 + 2 + name_byte_num as usize + value_size)
            }
            _ => {
                panic!()
            }
        }
    }

    fn parse_value(&self, name: String, tag_type: u8, position: u64) -> (NbtTag, usize) {
        let mut cur = Cursor::new(self.nbt_slice);
        let _ = cur.seek(SeekFrom::Start(position));
        match tag_type {
            0 => {
                (NbtTag::End, 0)
            }
            1 => {
                let value = cur.read_u8().unwrap();
                (NbtTag::Byte(name, value as i8), 1)
            }
            2 => {
                let value = cur.read_u16::<BigEndian>().unwrap();
                (NbtTag::Short(name, value as i16), 2)
            }
            3 => {
                let value = cur.read_u32::<BigEndian>().unwrap();
                (NbtTag::Int(name, value as i32), 4)
            }
            4 => {
                let value = cur.read_u64::<BigEndian>().unwrap();
                (NbtTag::Long(name, value as i64), 8)
            }
            5 => {
                let value = cur.read_f32::<BigEndian>().unwrap();
                (NbtTag::Float(name, value), 4)
            }
            6 => {
                let value = cur.read_f64::<BigEndian>().unwrap();
                (NbtTag::Double(name, value), 8)
            }
            7 => {

                let array_num = cur.read_u32::<BigEndian>().unwrap();

                let array = cur.bytes().take((array_num) as usize).map(|e| e.unwrap()).collect::<Vec<u8>>();

                (NbtTag::ByteArray(name, array), 4 + array_num as usize)

            }
            8 => {
                let value_num = cur.read_u16::<BigEndian>().unwrap();
                let array = cur.bytes().take((value_num) as usize).map(|e| e.unwrap()).collect::<Vec<u8>>();
                let value = String::from_utf8(array).unwrap();

                (NbtTag::String(name, value), 2 + value_num as usize)

            }
            9 => {
                let tag_type = cur.read_u8().unwrap();
                let item_num = cur.read_u32::<BigEndian>().unwrap();
                match tag_type {
                    0 => {
                        let mut item_vec = Vec::new();
                        for _ in 0..item_num {
                            let tag = NbtTag::End;
                            item_vec.push(tag);

                        }
                        (NbtTag::List(name, item_vec), 1 + 4 + item_num as usize)
                    }
                    n @ 1...11 => {
                        let mut item_vec = Vec::new();
                        let mut item_length = 0;
                        for _ in 0..item_num {
                            let position = cur.position();

                            let (tag, size) = self.parse_value("".to_string(), n, position);
                            item_vec.push(tag);

                            let _ = cur.seek(SeekFrom::Start(position + size as u64));
                            item_length += size;
                        }
                        (NbtTag::List(name, item_vec), 1 + 4 + item_length as usize)
                    },

                    n => {
                        println!("{}", n);
                        panic!()
                    }
                }
            }
            10 => {
                let position = cur.position();
                let mut item_vec = Vec::new();
                let mut item_length = 0;
                loop {
                    let current_position = cur.position();
                    let tag_type = cur.read_u8().unwrap();
                    if tag_type == 0 {
                        item_length += 1;
                        break
                    }

                    let name_byte_num = cur.read_u16::<BigEndian>().unwrap();
                    let name_vec = cur.bytes().take(name_byte_num as usize).map(|e| e.unwrap()).collect();
                    let name = String::from_utf8(name_vec).unwrap();
                    item_length += 1 + 2 + name_byte_num;


                    let (tag, size) = self.parse_value(name, tag_type, current_position + 1 + 2 + name_byte_num as u64);
                    item_vec.push(tag);

                    cur = Cursor::new(self.nbt_slice);
                    item_length += size as u16;
                    let _ = cur.seek(SeekFrom::Start(position + item_length as u64));
                }

                (NbtTag::Compound(name, item_vec), item_length as usize)
            }
            11 => {
                let item_num = cur.read_u32::<BigEndian>().unwrap();
                let mut item_vec = Vec::new();

                for _ in 0..item_num {
                    let item = cur.read_u32::<BigEndian>().unwrap();

                    item_vec.push(item as i32);
                }

                (NbtTag::IntArray(name, item_vec), 4 + (item_num * 4) as usize)
            }
            _ => {
                panic!()
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
    List(String, Vec<NbtTag>),
    Compound(String, Vec<NbtTag>),
    IntArray(String, Vec<i32>)
}

#[cfg(test)]
mod tests {
    use super::{NbtParser, NbtTag, Region};

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

        assert_eq!(parser.parse(), (NbtTag::End, 1));
    }

    #[test]
    fn test2() {
        print!("{:?}", "foo".as_bytes());
    }

    #[test]
    fn test_nbt_byte() {
        let parser = NbtParser::new(&[1, 0, 3, 102, 111, 111, 3]);

        assert_eq!(parser.parse(), (NbtTag::Byte("foo".to_string(), 3), 7));
    }

    #[test]
    fn test_nbt_short() {
        let parser = NbtParser::new(&[2, 0, 3, 102, 111, 111, 1, 1]);

        assert_eq!(parser.parse(), (NbtTag::Short("foo".to_string(), 257), 8));
    }

    #[test]
    fn test_nbt_int() {
        let parser = NbtParser::new(&[3, 0, 3, 102, 111, 111, 1, 1, 1, 1]);

        assert_eq!(parser.parse(), (NbtTag::Int("foo".to_string(), 16843009), 10));
    }

    #[test]
    fn test_nbt_long() {
        let parser = NbtParser::new(&[4, 0, 3, 102, 111, 111, 1, 1, 1, 1, 1, 1, 1, 1]);

        assert_eq!(parser.parse(), (NbtTag::Long("foo".to_string(), 72340172838076673), 14));
    }

    #[test]
    fn test_float() {
        println!("{}", 1.0f32.to_bits());
    }

    #[test]
    fn test_nbt_float() {
        let parser = NbtParser::new(&[5, 0, 3, 102, 111, 111, 63, 128, 0, 0]);

        assert_eq!(parser.parse(), (NbtTag::Float("foo".to_string(), 1f32), 10));
    }

    #[test]
    fn test_double() {
        println!("{}", 1.0f64.to_bits());
    }

    #[test]
    fn test_nbt_double() {
        let parser = NbtParser::new(&[6, 0, 3, 102, 111, 111, 63, 240, 0, 0, 0, 0, 0, 0]);

        assert_eq!(parser.parse(), (NbtTag::Double("foo".to_string(), 1f64), 14));
    }

    #[test]
    fn test_nbt_byte_array() {
        let parser = NbtParser::new(&[7, 0, 3, 102, 111, 111, 0, 0, 0, 3, 1, 2, 3]);

        assert_eq!(parser.parse(), (NbtTag::ByteArray("foo".to_string(), vec![1, 2, 3]), 13));
    }

    #[test]
    fn test_nbt_string() {
        let parser = NbtParser::new(&[8, 0, 3, 102, 111, 111, 0, 6, 227, 129, 187, 227, 129, 146]);

        assert_eq!(parser.parse(), (NbtTag::String("foo".to_string(), "ほげ".to_string()), 14));
    }


    mod nbt_tag_list {
        use super::super::{NbtParser, NbtTag};

        #[test]
        fn test_nbt_list_end() {
            let parser = NbtParser::new(&[9, 0, 3, 102, 111, 111, 0, 0, 0, 0, 2, 0, 0]);

            assert_eq!(parser.parse(), (NbtTag::List("foo".to_string(), vec![NbtTag::End, NbtTag::End]), 13));
        }

        #[test]
        fn test_nbt_list_byte() {
            let parser = NbtParser::new(&[9, 0, 3, 102, 111, 111, 1, 0, 0, 0, 2, 1, 2]);

            assert_eq!(parser.parse(), (NbtTag::List("foo".to_string(), vec![NbtTag::Byte("".to_string(), 1), NbtTag::Byte("".to_string(), 2)]), 13));
        }

        #[test]
        fn test_nbt_list_short() {
            let parser = NbtParser::new(&[9, 0, 3, 102, 111, 111, 2, 0, 0, 0, 2, 0, 1, 0, 2]);

            assert_eq!(parser.parse(), (NbtTag::List("foo".to_string(), vec![NbtTag::Short("".to_string(), 1), NbtTag::Short("".to_string(), 2)]), 15));
        }

        #[test]
        fn test_nbt_list_int() {
            let parser = NbtParser::new(&[9, 0, 3, 102, 111, 111, 3, 0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0, 2]);

            assert_eq!(parser.parse(), (NbtTag::List("foo".to_string(), vec![NbtTag::Int("".to_string(), 1), NbtTag::Int("".to_string(), 2)]), 19));
        }

        #[test]
        fn test_nbt_list_long() {
            let parser = NbtParser::new(&[9, 0, 3, 102, 111, 111, 4, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2]);

            assert_eq!(parser.parse(), (NbtTag::List("foo".to_string(), vec![NbtTag::Long("".to_string(), 1), NbtTag::Long("".to_string(), 2)]), 27));
        }

        #[test]
        fn test_nbt_list_float() {
            let parser = NbtParser::new(&[9, 0, 3, 102, 111, 111, 5, 0, 0, 0, 2, 63, 128, 0, 0, 63, 128, 0, 0]);

            assert_eq!(parser.parse(), (NbtTag::List("foo".to_string(), vec![NbtTag::Float("".to_string(), 1f32), NbtTag::Float("".to_string(), 1f32)]), 19));
        }

        #[test]
        fn test_nbt_list_double() {
            let parser = NbtParser::new(&[9, 0, 3, 102, 111, 111, 6, 0, 0, 0, 2, 63, 240, 0, 0, 0, 0, 0, 0, 63, 240, 0, 0, 0, 0, 0, 0]);

            assert_eq!(parser.parse(), (NbtTag::List("foo".to_string(), vec![NbtTag::Double("".to_string(), 1f64), NbtTag::Double("".to_string(), 1f64)]), 27));
        }

        #[test]
        fn test_nbt_list_byte_array() {
            let parser = NbtParser::new(&[9, 0, 3, 102, 111, 111, 7, 0, 0, 0, 2, 0, 0, 0, 2, 1, 2, 0, 0, 0, 3, 1, 2, 3]);

            assert_eq!(parser.parse(),
                       (NbtTag::List("foo".to_string(),
                                     vec![NbtTag::ByteArray("".to_string(), vec![1, 2]), NbtTag::ByteArray("".to_string(), vec![1, 2, 3])])
                           , 24
                       )
            );
        }

        #[test]
        fn test_nbt_list_string() {
            let parser = NbtParser::new(&[9, 0, 3, 102, 111, 111, 8, 0, 0, 0, 2, 0, 6, 227, 129, 187, 227, 129, 146, 0, 6, 227, 129, 187, 227, 129, 146]);

            assert_eq!(parser.parse(), (
                NbtTag::List("foo".to_string(),
                             vec![NbtTag::String("".to_string(), "ほげ".to_string()), NbtTag::String("".to_string(), "ほげ".to_string())])
                , 27)
            );
        }

        #[test]
        fn test_nbt_list_list_byte() {
            let parser = NbtParser::new(&[9, 0, 3, 102, 111, 111, 9, 0, 0, 0, 2, 1, 0, 0, 0, 2, 1, 2, 1, 0, 0, 0, 2, 1, 2]);

            assert_eq!(parser.parse(), (
                NbtTag::List("foo".to_string(),
                vec![NbtTag::List("".to_string(), vec![NbtTag::Byte("".to_string(), 1), NbtTag::Byte("".to_string(), 2)]),
                     NbtTag::List("".to_string(), vec![NbtTag::Byte("".to_string(), 1), NbtTag::Byte("".to_string(), 2)])]
                ), 25))
        }

    }

    mod nbt_tag_compound {
        use super::super::{NbtParser, NbtTag};
        #[test]
        fn test_nbt_compound() {
            let parser = NbtParser::new(&[10, 0, 3, 102, 111, 111, 0]);

            assert_eq!(parser.parse(), (NbtTag::Compound("foo".to_string(), vec![]), 7))
        }

        #[test]
        fn test_nbt_byte_in_compound() {
            let parser = NbtParser::new(&[10, 0, 3, 102, 111, 111, 1, 0, 3, 102, 111, 111, 3, 0]);

            assert_eq!(parser.parse(), (NbtTag::Compound("foo".to_string(), vec![NbtTag::Byte("foo".to_string(), 3)]), 14))
        }

        #[test]
        fn test_two_nbt_byte_in_compound() {
            let parser = NbtParser::new(&[10, 0, 3, 102, 111, 111, 1, 0, 3, 102, 111, 111, 3, 1, 0, 3, 102, 111, 111, 3, 0]);

            assert_eq!(parser.parse(), (NbtTag::Compound("foo".to_string(), vec![NbtTag::Byte("foo".to_string(), 3), NbtTag::Byte("foo".to_string(), 3)]), 21));
        }

        #[test]
        fn test_nbt_short_in_compound() {
            let parser = NbtParser::new(&[10, 0, 3, 102, 111, 111, 2, 0, 3, 102, 111, 111, 0, 3, 0]);

            assert_eq!(parser.parse(), (NbtTag::Compound("foo".to_string(), vec![NbtTag::Short("foo".to_string(), 3)]), 15));
        }

        #[test]
        fn test_two_nbt_short_in_compound() {
            let parser = NbtParser::new(&[10, 0, 3, 102, 111, 111, 2, 0, 3, 102, 111, 111, 0, 3, 2, 0, 3, 102, 111, 111, 0, 3, 0]);

            assert_eq!(parser.parse(),
                       (NbtTag::Compound("foo".to_string(), vec![NbtTag::Short("foo".to_string(), 3),
                                                                 NbtTag::Short("foo".to_string(), 3)]), 23));
        }

        #[test]
        fn test_nbt_int_in_compound() {
            let parser = NbtParser::new(&[10, 0, 3, 102, 111, 111, 3, 0, 3, 102, 111, 111, 0, 0, 0, 3, 0]);

            assert_eq!(parser.parse(),
                       (NbtTag::Compound("foo".to_string(), vec![NbtTag::Int("foo".to_string(), 3)]), 17));
        }

        #[test]
        fn test_nbt_two_int_in_compound() {
            let parser = NbtParser::new(&[10, 0, 3, 102, 111, 111, 3, 0, 3, 102, 111, 111, 0, 0, 0, 3, 3, 0, 3, 102, 111, 111, 0, 0, 0, 3, 0]);

            assert_eq!(parser.parse(),
                       (NbtTag::Compound("foo".to_string(), vec![NbtTag::Int("foo".to_string(), 3), NbtTag::Int("foo".to_string(), 3)]), 27));
        }

        // TODO 他パターンのテストケースの作成
    }


    #[test]
    fn test_nbt_int_array() {
        let parser = NbtParser::new(&[11, 0, 3, 102, 111, 111, 0, 0, 0, 2, 0, 0 ,0, 1, 0, 0, 0, 1]);

        assert_eq!(parser.parse(),
                   (NbtTag::IntArray("foo".to_string(), vec![1, 1]), 18));
    }
}
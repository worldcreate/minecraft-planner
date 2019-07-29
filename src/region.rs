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

        reader.seek(SeekFrom::Start(offset as u64));

        let mut buffer: [u8; 4] = [0; 4];
        reader.read_exact(&mut buffer);
        let length = (&buffer[..]).read_u32::<BigEndian>().unwrap();

        let mut buffer: [u8; 1] = [0];
        reader.read_exact(&mut buffer).unwrap();
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
        match self.nbt_slice[0] {
            0 => {
                return NbtTag::End;
            },
            _ => {
                unimplemented!();
            }
        }
    }
}

#[derive(PartialEq, Debug)]
enum NbtTag {
    End
}

#[cfg(test)]
mod tests {
    use crate::region::{NbtParser, Region, NbtTag};

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
}
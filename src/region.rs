use std::io::{BufReader, Read, Cursor, SeekFrom, Seek};
use std::fs::File;
use byteorder::{ReadBytesExt, BigEndian};
use std::cmp::Ordering;
use flate2::bufread::{GzDecoder, ZlibDecoder};

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
                println!("{:?}", vec);
                let mut d = ZlibDecoder::new(&vec[..]);
                let mut s = String::new();
                d.read_to_string(&mut s).unwrap();
                println!("{}", s);
            },
            _ => {
                panic!("unknown type");
            }
        }


    }
}

#[cfg(test)]
mod tests {
    use crate::region::Region;

    #[test]
    fn test() {
        Region::load_mca();
    }
}
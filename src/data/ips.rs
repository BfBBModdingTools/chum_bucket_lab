use byteorder::{BigEndian, ByteOrder};
use std::io::{Cursor, Read};

const EOF: &'static [u8] = &[b'E', b'O', b'F'];

pub struct Ips {
    file: Cursor<Vec<u8>>,
}

impl Ips {
    pub fn new(bytes: Vec<u8>) -> Self {
        // TODO: file type checking
        Ips {
            file: Cursor::new(bytes),
        }
    }

    // FIXME: mutable reference to self is coneceptually odd,
    // but required for seeking on file
    pub fn apply_to(&mut self, rom: &mut Vec<u8>) -> Result<(), std::io::Error> {
        let f = &mut self.file;

        // Skip 'PATCH' HEADER
        f.set_position(5);

        // NOTE: 0th index is padding and should always be ZERO
        let rom_start_size = rom.len();
        let mut buf = vec![0u8; 6];
        loop {
            f.read_exact(&mut buf[1..4])?;

            let address = &buf[1..4];
            if address == EOF {
                break;
            }

            // Read length separately to avoid reading past the EOF
            f.read_exact(&mut buf[4..6])?;

            let address = BigEndian::read_u32(&buf[0..4]) as usize;
            let length = BigEndian::read_u16(&buf[4..6]) as usize;

            if length != 0 {
                // Create a buffer to hold the patch data
                let mut data_buf = vec![0u8; length];
                f.read_exact(&mut data_buf[0..length])?;

                // Replace bytes at address with patch data
                for (buf_index, rom_address) in (address..address + length).enumerate() {
                    if rom_address < rom_start_size {
                        rom[rom_address] = data_buf[buf_index];
                    } else {
                        push_at(rom, rom_address, data_buf[buf_index]);
                    }
                }
            } else {
                // RLE Encoding
                // Need to read two more bytes for the actual size
                // One byte for the repeated byte

                f.read_exact(&mut buf[1..4])?;
                let length = BigEndian::read_u16(&buf[1..3]) as usize;
                let byte = buf[3];

                for rom_address in address..address + length {
                    if rom_address < rom_start_size {
                        rom[rom_address] = byte;
                    } else {
                        push_at(rom, rom_address, byte);
                    }
                }
            }
        }

        fn push_at(v: &mut Vec<u8>, i: usize, b: u8) {
            for j in v.len()..i {
                v.push(0u8);
            }
            v.push(b);
        }

        Ok(())
    }
}

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
        let mut buf = vec![0u8; 6];
        loop {
            f.read_exact(&mut buf[1..4])?;

            let address = &buf[1..4];
            if address == EOF {
                break;
            }

            f.read_exact(&mut buf[4..6])?;

            let address = BigEndian::read_u32(&buf[0..4]) as usize;
            let length = BigEndian::read_u16(&buf[4..6]) as usize;

            // Create a buffer to hold the patch data
            let mut data_buf = vec![0u8; length];
            f.read_exact(&mut data_buf[0..length])?;

            // Replace bytes at address with patch data
            for (buf_index, rom_address) in (address..address + 1).enumerate() {
                rom[rom_address] = data_buf[buf_index];
            }

            // TODO: RLE Encoding
        }

        Ok(())
    }
}

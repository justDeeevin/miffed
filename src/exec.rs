use byte_unit::Byte;
use std::collections::HashMap;
use thiserror::Error;

pub struct Machine {
    pub memory: Memory,
    pub registers: [i32; 32],
    pub pc: u32,
    pub hi: i32,
    pub lo: i32,
}

const PAGE_SIZE: Byte = Byte::KIBIBYTE.multiply(4).unwrap();

pub struct Memory {
    pages: HashMap<u32, [u8; PAGE_SIZE.as_u64() as usize]>,
    pub last_executable_address: u32,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Segmentation violation")]
    SegFault,
}

impl Memory {
    pub fn read(&self, mut address: u32, mut buf: &mut [u8]) -> Result<()> {
        while !buf.is_empty() {
            let page_number = address >> PAGE_SIZE.as_u64().trailing_zeros();
            let page_offset = (address as u64 & (PAGE_SIZE.as_u64() - 1)) as usize;
            // Attempt to read uninitialized memory
            let page = self.pages.get(&page_number).ok_or(Error::SegFault)?;

            let bytes = (PAGE_SIZE.as_u64() as usize - page_offset).min(buf.len());
            buf[..bytes].copy_from_slice(&page[page_offset..page_offset + bytes]);
            address += bytes as u32;
            buf = &mut buf[bytes..];
        }

        Ok(())
    }

    pub fn write(&mut self, address: u32, buf: &[u8]) -> Result<()> {
        if address <= self.last_executable_address {
            // Attempt to write to executable memory
            return Err(Error::SegFault);
        }
        self.write_unchecked(address, buf);

        Ok(())
    }

    fn write_unchecked(&mut self, mut address: u32, mut buf: &[u8]) {
        while !buf.is_empty() {
            let page_number = address >> PAGE_SIZE.as_u64().trailing_zeros();
            let page_offset = (address as u64 & (PAGE_SIZE.as_u64() - 1)) as usize;
            let page = self
                .pages
                .entry(page_number)
                .or_insert_with(|| [0; PAGE_SIZE.as_u64() as usize]);
            let bytes = (PAGE_SIZE.as_u64() as usize - page_offset).min(buf.len());
            page[page_offset..page_offset + bytes].copy_from_slice(&buf[..bytes]);
            address += bytes as u32;
            buf = &buf[bytes..];
        }
    }

    pub fn new(text: Vec<i32>, data: Vec<u8>) -> Self {
        let mut out = Self {
            pages: HashMap::from_iter([(0, [0; PAGE_SIZE.as_u64() as usize])]),
            last_executable_address: (text.len() as u32).saturating_sub(1),
        };

        out.write_unchecked(
            0,
            &text
                .into_iter()
                .flat_map(|n| n.to_ne_bytes())
                .collect::<Vec<_>>(),
        );
        out.write(out.last_executable_address + 1, &data).unwrap();

        out
    }
}

pub enum Syscall {
    PrintInt = 1,
    PrintFloat,
    PrintDouble,
    PrintString,
    ReadInt,
    ReadFloat,
    ReadDouble,
    ReadString,
    Sbrk,
    Exit,
    PrintChar,
    ReadChar,
    Open,
    Read,
    Write,
    Close,
    Exit2,
}

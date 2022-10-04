use std::vec::Vec;

pub const PAGE_SIZE: usize = 4096;

pub type Page = [u8; PAGE_SIZE];

pub trait Memory {
    /// Returns new zeroed slice of memory
    fn new() -> Self;

    /// Zeroes out the memory region
    fn zero(&mut self);

    /// Access
    fn double(&self, offset: u32) -> &[u8; 8];
    fn word(&self, offset: u32) -> &[u8; 4];
    fn half(&self, offset: u32) -> &[u8; 2];
    fn byte(&self, offset: u32) -> &u8;

    fn double_mut(&mut self, offset: u32) -> &mut [u8; 8];
    fn word_mut(&mut self, offset: u32) -> &mut [u8; 4];
    fn half_mut(&mut self, offset: u32) -> &mut [u8; 2];
    fn byte_mut(&mut self, offset: u32) -> &mut u8;
}

impl Memory for Page {
    fn new() -> Self {
        [0; PAGE_SIZE]
    }

    fn zero(&mut self) {
        *self = [0; PAGE_SIZE]
    }

    fn double(&self, offset: u32) -> &[u8; 8] {
        array_ref![self, offset as usize, 8]
    }

    fn word(&self, offset: u32) -> &[u8; 4] {
        array_ref![self, offset as usize, 4]
    }

    fn half(&self, offset: u32) -> &[u8; 2] {
        array_ref![self, offset as usize, 2]
    }

    fn byte(&self, offset: u32) -> &u8 {
        &self[offset as usize]
    }

    fn double_mut(&mut self, offset: u32) -> &mut [u8; 8] {
        array_mut_ref![self, offset as usize, 8]
    }

    fn word_mut(&mut self, offset: u32) -> &mut [u8; 4] {
        array_mut_ref![self, offset as usize, 4]
    }

    fn half_mut(&mut self, offset: u32) -> &mut [u8; 2] {
        array_mut_ref![self, offset as usize, 2]
    }

    fn byte_mut(&mut self, offset: u32) -> &mut u8 {
        &mut self[offset as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn byte_test() {
        let page = &mut Page::new();
        *page.byte_mut(0) = 1u8;

        assert_eq!(*page.byte(0), 1);
        assert_eq!(*page.byte(1), 0);
        
    }

    #[test]
    fn half_test() {
        let page = &mut Page::new();
        *page.half_mut(4094) = (0xfffe as u16).to_le_bytes();

        assert_eq!(*page.byte(4093), 0x00);
        assert_eq!(*page.byte(4094), 0xfe);
        assert_eq!(*page.byte(4095), 0xff);
        assert_eq!(*page.half(4094), (0xfffe as u16).to_le_bytes());
    }

    #[test]
    fn word_test() {
        let page = &mut Page::new();
        *page.word_mut(0) = (0xfffefdfc as u32).to_le_bytes();

        assert_eq!(*page.byte(0), 0xfc);
        assert_eq!(*page.byte(1), 0xfd);
        assert_eq!(*page.byte(2), 0xfe);
        assert_eq!(*page.byte(3), 0xff);
        assert_eq!(*page.byte(4), 0x00);
        assert_eq!(*page.word(0), (0xfffefdfc as u32).to_le_bytes());
    }

    #[test]
    fn word_alignment_test() {
        let page = &mut Page::new();
        *page.word_mut(1) = (0xfffefdfc as u32).to_le_bytes();
        
        assert_eq!(*page.byte(0), 0x00);
        assert_eq!(*page.byte(1), 0xfc);
        assert_eq!(*page.byte(2), 0xfd);
        assert_eq!(*page.byte(3), 0xfe);
        assert_eq!(*page.byte(4), 0xff);
        assert_eq!(*page.byte(5), 0x00);
        assert_eq!(*page.word(1), (0xfffefdfc as u32).to_le_bytes())
    }

    #[test]
    fn double_test() {
        let page = &mut Page::new();
        *page.double_mut(0) = (1242357348375.53245412343432 as f64).to_le_bytes();

        assert_eq!(*page.byte(0), 0b10000101);
        assert_eq!(*page.byte(1), 0b01111000);
        assert_eq!(*page.byte(2), 0b10000001);
        assert_eq!(*page.byte(3), 0b01001011);
        assert_eq!(*page.byte(4), 0b00100100);
        assert_eq!(*page.byte(5), 0b00010100);
        assert_eq!(*page.byte(6), 0b01110010);
        assert_eq!(*page.byte(7), 0b01000010);
        assert_eq!(*page.byte(8), 0);
            
        assert_eq!(*page.double(0), (1242357348375.53245412343432 as f64).to_le_bytes());
    }

    #[test]
    fn zero_test() {
        let page = &mut Page::new();
        *page.double_mut(0) = (1242357348375.53245412343432 as f64).to_le_bytes();

        assert_eq!(*page.byte(0), 0b10000101);
        assert_eq!(*page.byte(1), 0b01111000);
        assert_eq!(*page.byte(2), 0b10000001);
        assert_eq!(*page.byte(3), 0b01001011);
        assert_eq!(*page.byte(4), 0b00100100);
        assert_eq!(*page.byte(5), 0b00010100);
        assert_eq!(*page.byte(6), 0b01110010);
        assert_eq!(*page.byte(7), 0b01000010);
        
        page.zero();

        assert_eq!(*page.byte(0), 0);
        assert_eq!(*page.byte(1), 0);
        assert_eq!(*page.byte(2), 0);
        assert_eq!(*page.byte(3), 0);
        assert_eq!(*page.byte(4), 0);
        assert_eq!(*page.byte(5), 0);
        assert_eq!(*page.byte(6), 0);
        assert_eq!(*page.byte(7), 0);
    }
}
use std::fmt::Display;
use std::result::Result;
use std::vec::Vec;

pub type MemAddr = u32;


#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
enum ControlFlags {
    NoPermissions,
    R,
    X,
    W,
    RX,
    RW,
}

impl ControlFlags {
    pub fn is_readable(&self) -> bool {
        match self {
            ControlFlags::R | ControlFlags::RX | ControlFlags::RW => true,
            _ => false,
        }
    }

    pub fn is_writeable(&self) -> bool {
        match self {
            ControlFlags::W | ControlFlags::RW => true,
            _ => false,
        }
    }

    pub fn is_executable(&self) -> bool {
        match self {
            ControlFlags::X | ControlFlags::RX => true,
            _ => false,
        }
    }

    fn make_readable(&mut self) {
        *self = match self {
            ControlFlags::NoPermissions => ControlFlags::R,
            ControlFlags::X => ControlFlags::RX,
            ControlFlags::W => ControlFlags::RW,
            ControlFlags::R | ControlFlags::RX | ControlFlags::RW => *self,
        };
    }

    fn make_writeable(&mut self) {
        *self = match self {
            ControlFlags::NoPermissions => ControlFlags::W,
            ControlFlags::R => ControlFlags::RW,
            ControlFlags::W | ControlFlags::RW => *self,
            ControlFlags::X | ControlFlags::RX => {
                panic!("Cannot make executable segment writeable!")
            }
        }
    }

    fn make_executable(&mut self) {
        *self = match self {
            ControlFlags::NoPermissions => ControlFlags::X,
            ControlFlags::R => ControlFlags::RX,
            ControlFlags::X | ControlFlags::RX => *self,
            ControlFlags::W | ControlFlags::RW => {
                panic!("Cannot make writable segment executable!")
            }
        }
    }
}

impl Display for ControlFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match *self {
            ControlFlags::NoPermissions => "---",
            ControlFlags::R => "R--",
            ControlFlags::W => "-W-",
            ControlFlags::X => "--X",
            ControlFlags::RX => "R-X",
            ControlFlags::RW => "RW-",
        })
    }
}

// Note that almost all macros can be replaced with const generics when they land

macro_rules! read_mem_decl {
    ($name: ident , $type: ty) => {
        fn $name(&self, address: MemAddr) -> Result<$type, PageError>;
    };
}

macro_rules! write_mem_decl {
    ($name: ident , $type: ty) => {
        fn $name(&mut self, address: MemAddr, data: $type) -> Result<(), PageError>;
    };
}

// Standard Implementation
macro_rules! read_mem_impl {
    ($name: ident , $type: ty) => {
        fn $name(&self, address: MemAddr) -> Result<$type, PageError> {
            let offset = address - self.bottom();
            let bytes = array_ref![self.pages, offset as usize, std::mem::size_of::<$type>()];

            if cfg!(target_endian = "little") {
                Ok(<$type>::from_le_bytes(*bytes))
            } else {
                Ok(<$type>::from_be_bytes(*bytes))
            }
        }
    };
}

macro_rules! write_mem_impl {
    ($name: ident , $type: ty) => {
        fn $name(&mut self, address: MemAddr, data: $type) -> Result<(), PageError> {
            let offset = address - self.bottom();
            let bytes = array_mut_ref![self.pages, offset as usize, std::mem::size_of::<$type>()];

            if cfg!(target_endian = "little") {
                *bytes = data.to_le_bytes();
            } else {
                *bytes = data.to_be_bytes();
            }

            Ok(())
        }
    };
}


// TODO Macros
macro_rules! read_mem_todo {
    ($name: ident , $type: ty) => {
        fn $name(&self, address: MemAddr) -> Result<$type, PageError> {
            todo!()
        }
    };
}

macro_rules! write_mem_todo {
    ($name: ident , $type: ty) => {
        fn $name(&mut self, address: MemAddr, data: $type) -> Result<(), PageError> {
            todo!()
        }
    };
}

// Repeat
macro_rules! read_primitives_proto {
    ($name: ident) => {
        $name!(read_u8, u8);
        $name!(read_u16, u16);
        $name!(read_u32, u32);
        $name!(read_u64, u64);
        $name!(read_u128, u128);

        $name!(read_i8, i8);
        $name!(read_i16, i16);
        $name!(read_i32, i32);
        $name!(read_i64, i64);
        $name!(read_i128, i128);

        $name!(read_f32, f32);
        $name!(read_f64, f64);
    }
}

macro_rules! write_primitives_proto {
    ($name: ident) => {
        $name!(write_u8, u8);
        $name!(write_u16, u16);
        $name!(write_u32, u32);
        $name!(write_u64, u64);
        $name!(write_u128, u128);

        $name!(write_i8, i8);
        $name!(write_i16, i16);
        $name!(write_i32, i32);
        $name!(write_i64, i64);
        $name!(write_i128, i128);

        $name!(write_f32, f32);
        $name!(write_f64, f64);
    }
}


// Page macros
macro_rules! page_read_impl {
    ($name: ident, $type: ty, $fn: path) => {
        #[doc="Allows for reading a copy of the data from the page"]
        pub fn $name(&self, address: MemAddr) -> Result<$type, PageError> {
            let segment = self.segment(address)?;
            $fn(segment, address)
        }
    };
}

macro_rules! page_write_impl {
    ($name: ident, $type: ty, $fn: path) => {
        #[doc="Allows for writing data to the page"]
        pub fn $name(&mut self, address: MemAddr, data: $type) -> Result<(), PageError> {
            let segment = self.segment_mut(address)?;
            $fn(segment, address, data)
        }
    };
}

// Repeat
macro_rules! page_read_proto {
    ($name: ident) => {
        $name!(read_u8, u8, Segment::read_u8);
        $name!(read_u16, u16, Segment::read_u16);
        $name!(read_u32, u32, Segment::read_u32);
        $name!(read_u64, u64, Segment::read_u64);
        $name!(read_u128, u128, Segment::read_u128);

        $name!(read_i8, i8, Segment::read_i8);
        $name!(read_i16, i16, Segment::read_i16);
        $name!(read_i32, i32, Segment::read_i32);
        $name!(read_i64, i64, Segment::read_i64);
        $name!(read_i128, i128, Segment::read_i128);

        $name!(read_f32, f32, Segment::read_f32);
        $name!(read_f64, f64, Segment::read_f64);
    }
}

macro_rules! page_write_proto {
    ($name: ident) => {
        $name!(write_u8, u8, Segment::write_u8);
        $name!(write_u16, u16, Segment::write_u16);
        $name!(write_u32, u32, Segment::write_u32);
        $name!(write_u64, u64, Segment::write_u64);
        $name!(write_u128, u128, Segment::write_u128);

        $name!(write_i8, i8, Segment::write_i8);
        $name!(write_i16, i16, Segment::write_i16);
        $name!(write_i32, i32, Segment::write_i32);
        $name!(write_i64, i64, Segment::write_i64);
        $name!(write_i128, i128, Segment::write_i128);

        $name!(write_f32, f32, Segment::write_f32);
        $name!(write_f64, f64, Segment::write_f64);
    }
}

trait Segment {
    fn bottom(&self) -> MemAddr;
    fn top(&self) -> MemAddr;

    read_primitives_proto!(read_mem_decl);
    write_primitives_proto!(write_mem_decl);
}

/// Segment for data. Grow increases the high address.
struct DataSegment {
    // Meta information
    low_address: MemAddr,
    high_address: MemAddr,

    // Bounds
    max_size: u32,

    // Control bits
    control: ControlFlags,

    // Data
    pages: Vec<u8>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum PageError {
    MemoryOutOfBounds(u32),
}

impl DataSegment {
    // Private helper for aligned addresses

    // Public API for ALIGNED addresses. (Note should also work for unaligned, but not across page boundaries)

    // fn read_u8(&self, address: MemAddr) -> Result<u8, PageError> {
    //     let offset = address - self.low_address;
    //     Ok(self.pages[offset as usize])
    // }

    fn sized_region(&mut self, address: MemAddr, size: u32) -> Result<&[u8], PageError> {
        Ok(&self.pages
            [(address - self.low_address) as usize..(address + size - self.low_address) as usize])
    }

    fn sized_region_mut(&mut self, address: MemAddr, size: u32) -> Result<&mut [u8], PageError> {
        Ok(&mut self.pages
            [(address - self.low_address) as usize..(address + size - self.low_address) as usize])
    }

    fn address_region(&mut self, low_addr: MemAddr, high_addr: u32) -> Result<&[u8], PageError> {
        Ok(&self.pages
            [(low_addr - self.low_address) as usize..(high_addr - self.low_address) as usize])
    }

    fn address_region_mut(
        &mut self,
        low_addr: MemAddr,
        high_addr: u32,
    ) -> Result<&mut [u8], PageError> {
        Ok(&mut self.pages
            [(low_addr - self.low_address) as usize..(high_addr - self.low_address) as usize])
    }

    /// Increases the data segment by `amount`. This is typically used for growing the data segment or heap
    fn grow_by(&mut self, amount: u32) {
        // TODO: size check
        // TODO: Increase size
        // TODO: Update high_address

        self.pages.resize(self.pages.len() + amount as usize, 0)
    }

    fn grow_to(&mut self, address: u32) {
        // TODO: size check
        // TODO: Increase size
        // TODO: Update high_address

        self.pages.resize((address - self.low_address) as usize, 0)
    }
}

impl Segment for DataSegment {
    fn bottom(&self) -> MemAddr {
        self.low_address
    }

    fn top(&self) -> MemAddr {
        self.high_address
    }

    read_primitives_proto!(read_mem_impl);
    write_primitives_proto!(write_mem_impl);
}

/// Segment for data. Grow increases the low address.
struct StackSegment {
    // Meta information
    low_address: MemAddr,
    high_address: MemAddr,

    // Bounds
    max_size: u32,

    // Control bits
    control: ControlFlags,

    // Data
    pages: Vec<u8>,
}

impl StackSegment {
    // Private helper for aligned addresses

    // Public API for ALIGNED addresses. (Note should also work for unaligned, but not across page boundaries)

    /// Not possible until const generics are implemented, at which time we can remove all of the macros and allow for more generics
    // fn read<T: Sized>(&self, address: MemAddr) -> Result<T, PageError> {
    //     let offset = address - self.low_address;
    //     let bytes = array_ref![self.pages, offset as usize, std::mem::size_of::<T>()];
    //     if cfg!(target_endian = "little") {
    //         Ok(<T>::from_le_bytes(*bytes))
    //     } else {
    //         Ok(<T>::from_be_bytes(*bytes))
    //     }
    // }

    fn sized_region(&mut self, address: MemAddr, size: u32) -> Result<&[u8], PageError> {
        Ok(&self.pages
            [(address - self.low_address) as usize..(address + size - self.low_address) as usize])
    }

    fn sized_region_mut(&mut self, address: MemAddr, size: u32) -> Result<&mut [u8], PageError> {
        Ok(&mut self.pages
            [(address - self.low_address) as usize..(address + size - self.low_address) as usize])
    }

    fn address_region(&mut self, low_addr: MemAddr, high_addr: u32) -> Result<&[u8], PageError> {
        Ok(&self.pages
            [(low_addr - self.low_address) as usize..(high_addr - self.low_address) as usize])
    }

    fn address_region_mut(
        &mut self,
        low_addr: MemAddr,
        high_addr: u32,
    ) -> Result<&mut [u8], PageError> {
        Ok(&mut self.pages
            [(low_addr - self.low_address) as usize..(high_addr - self.low_address) as usize])
    }

    /// Decreases the stack by amount. It is recommended to ensure amount is large to amortize the cost of growing the stack.
    fn grow_by(&mut self, amount: u32) {
        // TODO: size check
        // TODO: Increase size
        // TODO: Update low_address

        let mut new_page = vec![0; amount as usize];
        new_page.append(&mut self.pages);
        self.pages = new_page;
    }

    /// Grows to new bottom address
    fn grow_to(&mut self, address: u32) {
        // TODO: size check
        // TODO: Increase size
        // TODO: Update low_address

        let mut new_page = vec![0; (address - self.low_address) as usize];
        new_page.append(&mut self.pages);
        self.pages = new_page;
    }
}

impl Segment for StackSegment {
    read_primitives_proto!(read_mem_impl);
    write_primitives_proto!(write_mem_impl);

    fn bottom(&self) -> MemAddr {
        self.low_address
    }

    fn top(&self) -> MemAddr {
        self.high_address
    }
}

struct TextSegment;

impl Segment for TextSegment {
    read_primitives_proto!(read_mem_todo);
    write_primitives_proto!(write_mem_todo);

    fn bottom(&self) -> MemAddr {
        todo!()
    }

    fn top(&self) -> MemAddr {
        todo!()
    }
}

struct EmptySegment;

impl Segment for EmptySegment {
    read_primitives_proto!(read_mem_todo);
    write_primitives_proto!(write_mem_todo);

    fn bottom(&self) -> MemAddr {
        todo!()
    }

    fn top(&self) -> MemAddr {
        todo!()
    }
}

pub struct PageTable {
    data_seg: DataSegment,
    text_seg: TextSegment,

    heap_seg: DataSegment,
    stack_seg: StackSegment,

    kdata_seg: DataSegment,
    ktext_seg: TextSegment,

    mmio_seg: DataSegment,
    mmap_seg: EmptySegment, // Unimplemented

    gp_midpoint: MemAddr,
}

impl PageTable {
    page_read_proto!(page_read_impl);
    page_write_proto!(page_write_impl);

    fn segment(&self, address: MemAddr) -> Result<&dyn Segment, PageError> {
        // TODO: Add MMIO
        if address >= self.text_seg.bottom() && address < self.text_seg.top() {
            Ok(&self.text_seg)
        } else if address >= self.data_seg.bottom() && address < self.data_seg.top() {
            Ok(&self.data_seg)
        } else if address >= self.stack_seg.bottom() && address < self.stack_seg.top() {
            Ok(&self.stack_seg)
        } else if address >= self.ktext_seg.bottom() && address < self.ktext_seg.top() {
            Ok(&self.ktext_seg)
        } else if address >= self.kdata_seg.bottom() && address < self.kdata_seg.top() {
            Ok(&self.kdata_seg)
        } else {
            // This should generate a bad memory read for the receiver
            Err(PageError::MemoryOutOfBounds(address))
        }
    }

    fn segment_mut(&mut self, address: MemAddr) -> Result<&mut dyn Segment, PageError> {
        // TODO: Add MMIO
        if address >= self.text_seg.bottom() && address < self.text_seg.top() {
            Ok(&mut self.text_seg)
        } else if address >= self.data_seg.bottom() && address < self.data_seg.top() {
            Ok(&mut self.data_seg)
        } else if address >= self.stack_seg.bottom() && address < self.stack_seg.top() {
            Ok(&mut self.stack_seg)
        } else if address >= self.ktext_seg.bottom() && address < self.ktext_seg.top() {
            Ok(&mut self.ktext_seg)
        } else if address >= self.kdata_seg.bottom() && address < self.kdata_seg.top() {
            Ok(&mut self.kdata_seg)
        } else {
            // This should generate a bad memory read for the receiver
            Err(PageError::MemoryOutOfBounds(address))
        }
    }

    pub fn segments(&self) -> [&dyn Segment; 8] {
        [&self.data_seg, &self.text_seg, &self.heap_seg, &self.stack_seg, &self.kdata_seg, &self.ktext_seg, &self.mmio_seg, &self.mmap_seg]
    }

    // print_mem


    // mem_dump_profile

    // make_memory

    // new
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_endian = "big")]
    fn read_data_segment() {
        let mut a = vec![];

        for i in 0x00..=0xff {
            a.push(255u8 - i);
        }

        let seg = DataSegment {
            low_address: 0,
            high_address: 256,
            max_size: 1024,
            control: ControlFlags::RW,
            pages: a,
        };

        assert_eq!(seg.read_u8(0), Ok(0xffu8));
        assert_eq!(seg.read_i8(0), Ok(i8::from_be_bytes(0xffu8.to_be_bytes())));
        assert_eq!(seg.read_u16(0), Ok(0xfffeu16));
        assert_eq!(
            seg.read_i16(0),
            Ok(i16::from_be_bytes(0xfffeu16.to_be_bytes()))
        );
        assert_eq!(seg.read_u32(0), Ok(0xfffefdfcu32));
        assert_eq!(
            seg.read_i32(0),
            Ok(i32::from_be_bytes(0xfffefdfcu32.to_be_bytes()))
        );
        assert_eq!(
            seg.read_f32(0),
            Ok(f32::from_be_bytes(0xfffefdfcu32.to_be_bytes()))
        );

        assert_eq!(seg.read_u64(0), Ok(0xfffefdfcfbfaf9f8u64));
        assert_eq!(
            seg.read_i64(0),
            Ok(i64::from_be_bytes(0xfffefdfcfbfaf9f8u64.to_be_bytes()))
        );
        assert_eq!(
            seg.read_f64(0),
            Ok(f64::from_be_bytes(0xfffefdfcfbfaf9f8u64.to_be_bytes()))
        );

        assert_eq!(seg.read_u128(0), Ok(0xfffefdfcfbfaf9f8f7f6f5f4f3f2f1f0u128));
        assert_eq!(
            seg.read_i128(0),
            Ok(i128::from_be_bytes(
                0xfffefdfcfbfaf9f8f7f6f5f4f3f2f1f0u128.to_be_bytes()
            ))
        );
    }

    #[test]
    #[cfg(target_endian = "little")]
    fn read_data_segment() {
        let mut a = vec![];

        for i in 0x00..=0xff {
            a.push(255u8 - i);
        }

        let seg = DataSegment {
            low_address: 0xffff0000,
            high_address: 0xffff00ff,
            max_size: 1024,
            control: ControlFlags::RW,
            pages: a,
        };

        assert_eq!(seg.read_u8(0xffff0000), Ok(0xffu8));
        assert_eq!(
            seg.read_i8(0xffff0000),
            Ok(i8::from_le_bytes(0xffu8.to_le_bytes()))
        );
        assert_eq!(seg.read_u16(0xffff0000), Ok(0xfeffu16));
        assert_eq!(
            seg.read_i16(0xffff0000),
            Ok(i16::from_le_bytes(0xfeffu16.to_le_bytes()))
        );
        assert_eq!(seg.read_u32(0xffff0000), Ok(0xfcfdfeff));
        assert_eq!(
            seg.read_i32(0xffff0000),
            Ok(i32::from_le_bytes(0xfcfdfeffu32.to_le_bytes()))
        );
        assert_eq!(
            seg.read_f32(0xffff0000),
            Ok(f32::from_le_bytes(0xfcfdfeffu32.to_le_bytes()))
        );

        assert_eq!(seg.read_u64(0xffff0000), Ok(0xf8f9fafbfcfdfeffu64));
        assert_eq!(
            seg.read_i64(0xffff0000),
            Ok(i64::from_le_bytes(0xf8f9fafbfcfdfeffu64.to_le_bytes()))
        );
        assert_eq!(
            seg.read_f64(0xffff0000),
            Ok(f64::from_le_bytes(0xf8f9fafbfcfdfeffu64.to_le_bytes()))
        );

        assert_eq!(
            seg.read_u128(0xffff0000),
            Ok(0xf0f1f2f3f4f5f6f7f8f9fafbfcfdfeffu128)
        );
        assert_eq!(
            seg.read_i128(0xffff0000),
            Ok(i128::from_le_bytes(
                0xf0f1f2f3f4f5f6f7f8f9fafbfcfdfeffu128.to_le_bytes()
            ))
        );
    }

    #[test]
    #[cfg(target_endian = "little")]
    fn write_data_segment() {
        let mut a = vec![0; 256];

        let mut seg = DataSegment {
            low_address: 0xffff0000,
            high_address: 0xffff00ff,
            max_size: 1024,
            control: ControlFlags::RW,
            pages: a,
        };

        assert_eq!(Ok(()), seg.write_u8(0xffff0000, 0xff));
        assert_eq!(seg.read_u8(0xffff0000), Ok(0xffu8));

        assert_eq!(Ok(()), seg.write_i8(0xffff0000, -2i8));
        assert_eq!(seg.read_i8(0xffff0000), Ok(-2i8));

        assert_eq!(
            Ok(()),
            seg.write_i128(0xffff0001, -0x70f1f2f3f4f5f6f7f8f9fafbfcfdfeffi128)
        );
        assert_eq!(
            seg.read_i128(0xffff0001),
            Ok(-0x70f1f2f3f4f5f6f7f8f9fafbfcfdfeffi128)
        );
    }

    #[test]
    fn control_readable() {
        assert!(
            !ControlFlags::NoPermissions.is_readable(),
            "{} is not readable",
            ControlFlags::NoPermissions
        );
        assert!(
            ControlFlags::R.is_readable(),
            "{} should be readable",
            ControlFlags::R
        );
        assert!(
            !ControlFlags::W.is_readable(),
            "{} permissions is not readable",
            ControlFlags::W
        );
        assert!(
            !ControlFlags::X.is_readable(),
            "{} is not readable",
            ControlFlags::X
        );
        assert!(
            ControlFlags::RW.is_readable(),
            "{} should be readable",
            ControlFlags::RW
        );
        assert!(
            ControlFlags::RX.is_readable(),
            "{} should be readable",
            ControlFlags::RX
        );
    }

    #[test]
    fn control_writeable() {
        assert!(
            !ControlFlags::NoPermissions.is_writeable(),
            "{} is not writeable",
            ControlFlags::NoPermissions
        );
        assert!(
            !ControlFlags::R.is_writeable(),
            "{} permissions is not writeable",
            ControlFlags::R
        );
        assert!(
            ControlFlags::W.is_writeable(),
            "{} should be writeable",
            ControlFlags::W
        );
        assert!(
            !ControlFlags::X.is_writeable(),
            "{} is not writeable",
            ControlFlags::X
        );
        assert!(
            ControlFlags::RW.is_writeable(),
            "{} should be writeable",
            ControlFlags::RW
        );
        assert!(
            !ControlFlags::RX.is_writeable(),
            "{} should be writeable",
            ControlFlags::RX
        );
    }

    #[test]
    fn control_executable() {
        assert!(
            !ControlFlags::NoPermissions.is_executable(),
            "{} is not executable",
            ControlFlags::NoPermissions
        );
        assert!(
            !ControlFlags::R.is_executable(),
            "{} permissions is not executable",
            ControlFlags::R
        );
        assert!(
            !ControlFlags::W.is_executable(),
            "{} is not executable",
            ControlFlags::W
        );
        assert!(
            ControlFlags::X.is_executable(),
            "{} should be executable",
            ControlFlags::X
        );
        assert!(
            !ControlFlags::RW.is_executable(),
            "{} is not executable",
            ControlFlags::RW
        );
        assert!(
            ControlFlags::RX.is_executable(),
            "{} should be executable",
            ControlFlags::RX
        );
    }

    fn map_and_return(mut c: ControlFlags, f: &dyn Fn(&mut ControlFlags) -> ()) -> ControlFlags {
        f(&mut c);
        c
    }

    #[test]
    fn test_make_readable() {
        assert_eq!(
            ControlFlags::R,
            map_and_return(ControlFlags::NoPermissions, &ControlFlags::make_readable)
        );
        assert_eq!(
            ControlFlags::R,
            map_and_return(ControlFlags::R, &ControlFlags::make_readable)
        );
        assert_eq!(
            ControlFlags::RW,
            map_and_return(ControlFlags::W, &ControlFlags::make_readable)
        );
        assert_eq!(
            ControlFlags::RX,
            map_and_return(ControlFlags::X, &ControlFlags::make_readable)
        );
        assert_eq!(
            ControlFlags::RW,
            map_and_return(ControlFlags::RW, &ControlFlags::make_readable)
        );
        assert_eq!(
            ControlFlags::RX,
            map_and_return(ControlFlags::RX, &ControlFlags::make_readable)
        );
    }

    #[test]
    fn test_make_writeable() {
        assert_eq!(
            ControlFlags::W,
            map_and_return(ControlFlags::NoPermissions, &ControlFlags::make_writeable)
        );
        assert_eq!(
            ControlFlags::RW,
            map_and_return(ControlFlags::R, &ControlFlags::make_writeable)
        );
        assert_eq!(
            ControlFlags::W,
            map_and_return(ControlFlags::W, &ControlFlags::make_writeable)
        );
        assert_eq!(
            ControlFlags::RW,
            map_and_return(ControlFlags::RW, &ControlFlags::make_writeable)
        );
    }

    #[test]
    #[should_panic(expected = "Cannot make executable segment writeable!")]
    fn fail_make_writeable_x() {
        ControlFlags::X.make_writeable();
    }

    #[test]
    #[should_panic(expected = "Cannot make executable segment writeable!")]
    fn fail_make_writeable_rx() {
        ControlFlags::RX.make_writeable();
    }

    #[test]
    fn test_make_executable() {
        assert_eq!(
            ControlFlags::X,
            map_and_return(ControlFlags::NoPermissions, &ControlFlags::make_executable)
        );
        assert_eq!(
            ControlFlags::RX,
            map_and_return(ControlFlags::R, &ControlFlags::make_executable)
        );
        assert_eq!(
            ControlFlags::X,
            map_and_return(ControlFlags::X, &ControlFlags::make_executable)
        );
        assert_eq!(
            ControlFlags::RX,
            map_and_return(ControlFlags::RX, &ControlFlags::make_executable)
        );
    }

    #[test]
    #[should_panic(expected = "Cannot make writable segment executable!")]
    fn fail_make_executable_w() {
        ControlFlags::W.make_executable();
    }

    #[test]
    #[should_panic(expected = "Cannot make writable segment executable!")]
    fn fail_make_executable_rw() {
        ControlFlags::RW.make_executable();
    }
}

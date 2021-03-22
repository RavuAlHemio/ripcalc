use std::convert::{TryFrom, TryInto};
use std::error::Error;
use std::fmt;
use std::hash::Hash;
use std::num::ParseIntError;
use std::ops::{BitAnd, BitOr, BitXor};
use std::str::FromStr;

/// This trait is to be implemented by structures that represent an IP address or a similar network
/// address.
pub trait IpAddress: BitAnd<Output = Self> + BitOr<Output = Self> + BitXor<Output = Self> + Copy + fmt::Display + Hash + Ord + Sized {
    /// Returns the number of bytes required to encode this IP address in full.
    fn byte_count(&self) -> usize;

    /// Returns the number of bits within this IP address that have the value 1.
    fn count_ones(&self) -> u32;

    /// Returns the number of bits within this IP address that have the value 0.
    fn count_zeros(&self) -> u32;

    /// Serializes this IP address into its canonical byte-sequence representation.
    fn to_bytes(&self) -> Vec<u8>;

    /// Attempts to deserialize an IP address from its canonical byte-sequence representation.
    ///
    /// Returns `None` if this fails, e.g. because the byte sequence has the wrong length.
    fn from_bytes(bytes: &[u8]) -> Option<Self>;

    /// Returns this IP address with each bit negated.
    fn bitwise_negate(&self) -> Self;

    /// Returns the sum (with carry) of this and another IP address. Returns `None` if the addition
    /// overflows beyond the range of the IP address.
    fn add_addr(&self, other: &Self) -> Option<Self>;

    /// Returns the sum of this IP address and an offset. Returns `None` if the addition overflows
    /// beyond the range of the IP address.
    fn add_offset(&self, offset: i32) -> Option<Self>;

    /// Returns the difference (with borrow) between this and another IP address. Returns `None` if
    /// the subtraction overflows beyond the range of the IP address.
    fn subtract_addr(&self, other: &Self) -> Option<Self>;

    /// Returns the difference (with borrow) between this IP address and an offset. Returns `None`
    /// if the subtraction overflows beyond the range of the IP address.
    fn subtract_offset(&self, offset: i32) -> Option<Self>;
}

/// An IPv4 address.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Ipv4Address {
    addr_value: u32,
}

pub const IPV4_ZERO: Ipv4Address = Ipv4Address { addr_value: 0 };

impl Ipv4Address {
    /// Constructs a new IPv4 address from its 32-bit representation, where the leftmost byte in the
    /// canonical string representation is the most significant byte (i.e. `"1.2.3.4"` ->
    /// `0x01020304`).
    pub fn new(
        addr_value: u32,
    ) -> Ipv4Address {
        Ipv4Address {
            addr_value,
        }
    }

    fn add_internal(addr64: i64, offset64: i64) -> Option<Ipv4Address> {
        let sum = addr64 + offset64;
        if sum < 0 {
            None
        } else if sum > 0xFFFFFFFF {
            None
        } else {
            Some(Ipv4Address::new(sum.try_into().unwrap()))
        }
    }
}

impl IpAddress for Ipv4Address {
    fn byte_count(&self) -> usize { 4 }

    fn count_ones(&self) -> u32 { self.addr_value.count_ones() }
    fn count_zeros(&self) -> u32 { self.addr_value.count_zeros() }

    fn to_bytes(&self) -> Vec<u8> {
        let mut ret: Vec<u8> = Vec::with_capacity(4);
        ret.push(((self.addr_value >> 24) & 0xFF).try_into().unwrap());
        ret.push(((self.addr_value >> 16) & 0xFF).try_into().unwrap());
        ret.push(((self.addr_value >>  8) & 0xFF).try_into().unwrap());
        ret.push(((self.addr_value >>  0) & 0xFF).try_into().unwrap());
        ret
    }

    fn from_bytes(bytes: &[u8]) -> Option<Ipv4Address> {
        if bytes.len() != 4 {
            None
        } else {
            let addr_value: u32 =
                (u32::try_from(bytes[0]).unwrap() << 24) |
                (u32::try_from(bytes[1]).unwrap() << 16) |
                (u32::try_from(bytes[2]).unwrap() <<  8) |
                (u32::try_from(bytes[3]).unwrap() <<  0)
            ;
            Some(Ipv4Address::new(addr_value))
        }
    }

    fn bitwise_negate(&self) -> Ipv4Address {
        Ipv4Address::new(self.addr_value ^ 0xFFFFFFFFu32)
    }

    fn add_addr(&self, other: &Ipv4Address) -> Option<Ipv4Address> {
        Ipv4Address::add_internal(self.addr_value.into(), other.addr_value.into())
    }

    fn add_offset(&self, offset: i32) -> Option<Ipv4Address> {
        Ipv4Address::add_internal(self.addr_value.into(), offset.into())
    }

    fn subtract_addr(&self, other: &Ipv4Address) -> Option<Ipv4Address> {
        let other64: i64 = other.addr_value.into();
        Ipv4Address::add_internal(self.addr_value.into(), -other64)
    }

    fn subtract_offset(&self, offset: i32) -> Option<Ipv4Address> {
        let offset64: i64 = offset.into();
        Ipv4Address::add_internal(self.addr_value.into(), -offset64)
    }
}

impl FromStr for Ipv4Address {
    type Err = IpAddressParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let chunks: Vec<&str> = s.split('.').collect();
        if chunks.len() != 4 {
            return Err(IpAddressParseError::IncorrectChunkCount(chunks.len(), 4));
        }

        let mut addr_val: u32 = 0;
        for i in 0..4 {
            let shift_count = 24 - (i*8);

            if chunks[i].len() == 0 {
                return Err(IpAddressParseError::EmptyChunk(i));
            }

            let chunk_val: u32 = chunks[i].parse()
                .map_err(|e| IpAddressParseError::ChunkParseError(i, String::from(chunks[i]), e))?;
            if chunk_val > 255 {
                return Err(IpAddressParseError::ChunkOutOfRange(i, chunk_val, 0, 255));
            }

            addr_val |= chunk_val << shift_count;
        }

        Ok(Ipv4Address::new(addr_val))
    }
}

impl fmt::Display for Ipv4Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bytes = self.to_bytes();
        write!(f, "{}.{}.{}.{}", bytes[0], bytes[1], bytes[2], bytes[3])
    }
}

impl BitAnd for Ipv4Address {
    type Output = Ipv4Address;

    fn bitand(self, rhs: Self) -> Self::Output {
        Ipv4Address::new(self.addr_value & rhs.addr_value)
    }
}

impl BitOr for Ipv4Address {
    type Output = Ipv4Address;

    fn bitor(self, rhs: Self) -> Self::Output {
        Ipv4Address::new(self.addr_value | rhs.addr_value)
    }
}

impl BitXor for Ipv4Address {
    type Output = Ipv4Address;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Ipv4Address::new(self.addr_value ^ rhs.addr_value)
    }
}

/// An IPv6 address.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Ipv6Address {
    top_half: u64,
    bottom_half: u64,
}

pub const IPV6_ZERO: Ipv6Address = Ipv6Address { top_half: 0, bottom_half: 0 };

impl Ipv6Address {
    /// Constructs a new IPv6 address from its representation as a pair of 64-bit integers, where
    /// the leftmost byte in the canonical string representation is the most significant byte of
    /// the top half (i.e. `"0102:0304:0506:0708:090a:0b0c:0d0e:0f00"` ->
    /// `Ipv6Address::new(0x0102030405060708, 0x090a0b0c0d0e0f00)`).
    pub fn new(
        top_half: u64,
        bottom_half: u64,
    ) -> Ipv6Address {
        Ipv6Address {
            top_half,
            bottom_half,
        }
    }

    /// Outputs the IPv6 address in its full string representation with all leading zeroes and no
    /// omissions of consecutive zero fields.
    pub fn to_full_string(&self) -> String {
        let chunks = self.to_chunks();
        let mut chunk_strings = Vec::with_capacity(chunks.len());
        for i in 0..chunk_strings.len() {
            chunk_strings.push(format!("{:04x}", chunks[i]));
        }
        chunk_strings.join(":")
    }

    /// Returns this address represented as 16-bit chunks.
    pub fn to_chunks(&self) -> Vec<u16> {
        let mut ret: Vec<u16> = Vec::with_capacity(8);
        ret.push(((self.top_half >> 48) & 0xFFFF).try_into().unwrap());
        ret.push(((self.top_half >> 32) & 0xFFFF).try_into().unwrap());
        ret.push(((self.top_half >> 16) & 0xFFFF).try_into().unwrap());
        ret.push(((self.top_half >>  0) & 0xFFFF).try_into().unwrap());
        ret.push(((self.bottom_half >> 48) & 0xFFFF).try_into().unwrap());
        ret.push(((self.bottom_half >> 32) & 0xFFFF).try_into().unwrap());
        ret.push(((self.bottom_half >> 16) & 0xFFFF).try_into().unwrap());
        ret.push(((self.bottom_half >>  0) & 0xFFFF).try_into().unwrap());
        ret
    }

    /// Attempts to create an IPv6 address from its 16-bit chunk representation. Returns `None` if
    /// the number of chunks is incorrect.
    pub fn from_chunks(chunks: &[u16]) -> Option<Ipv6Address> {
        if chunks.len() != 8 {
            None
        } else {
            let top_half: u64 =
                (u64::try_from(chunks[0]).unwrap() << 48) |
                (u64::try_from(chunks[1]).unwrap() << 32) |
                (u64::try_from(chunks[2]).unwrap() << 16) |
                (u64::try_from(chunks[3]).unwrap() <<  0)
            ;
            let bottom_half: u64 =
                (u64::try_from(chunks[4]).unwrap() << 48) |
                (u64::try_from(chunks[5]).unwrap() << 32) |
                (u64::try_from(chunks[6]).unwrap() << 16) |
                (u64::try_from(chunks[7]).unwrap() <<  0)
            ;
            Some(Ipv6Address::new(top_half, bottom_half))
        }
    }

    fn add_internal(addrtop64: u64, addrbot64: u64, offtop64: u64, offbot64: u64) -> Option<Ipv6Address> {
        let bot_sum = addrbot64.wrapping_add(offbot64);
        let is_carry = bot_sum < addrbot64 || bot_sum < offbot64;

        let mut top_sum = addrtop64.checked_add(offtop64)?;
        if is_carry {
            top_sum = top_sum.checked_add(1)?;
        }
        Some(Ipv6Address::new(top_sum, bot_sum))
    }

    fn sub_internal(addrtop64: u64, addrbot64: u64, offtop64: u64, offbot64: u64) -> Option<Ipv6Address> {
        let bot_diff = addrbot64.wrapping_sub(offbot64);
        let is_borrow = bot_diff > addrbot64 || bot_diff > offbot64;

        let mut top_diff = addrtop64.checked_sub(offtop64)?;
        if is_borrow {
            top_diff = top_diff.checked_sub(1)?;
        }
        Some(Ipv6Address::new(top_diff, bot_diff))
    }
}

impl IpAddress for Ipv6Address {
    fn byte_count(&self) -> usize { 16 }

    fn count_ones(&self) -> u32 { self.top_half.count_ones() + self.bottom_half.count_ones() }
    fn count_zeros(&self) -> u32 { self.top_half.count_zeros() + self.bottom_half.count_zeros() }

    fn to_bytes(&self) -> Vec<u8> {
        let mut ret: Vec<u8> = Vec::with_capacity(16);
        ret.push(((self.top_half >> 56) & 0xFF).try_into().unwrap());
        ret.push(((self.top_half >> 48) & 0xFF).try_into().unwrap());
        ret.push(((self.top_half >> 40) & 0xFF).try_into().unwrap());
        ret.push(((self.top_half >> 32) & 0xFF).try_into().unwrap());
        ret.push(((self.top_half >> 24) & 0xFF).try_into().unwrap());
        ret.push(((self.top_half >> 16) & 0xFF).try_into().unwrap());
        ret.push(((self.top_half >>  8) & 0xFF).try_into().unwrap());
        ret.push(((self.top_half >>  0) & 0xFF).try_into().unwrap());
        ret.push(((self.bottom_half >> 56) & 0xFF).try_into().unwrap());
        ret.push(((self.bottom_half >> 48) & 0xFF).try_into().unwrap());
        ret.push(((self.bottom_half >> 40) & 0xFF).try_into().unwrap());
        ret.push(((self.bottom_half >> 32) & 0xFF).try_into().unwrap());
        ret.push(((self.bottom_half >> 24) & 0xFF).try_into().unwrap());
        ret.push(((self.bottom_half >> 16) & 0xFF).try_into().unwrap());
        ret.push(((self.bottom_half >>  8) & 0xFF).try_into().unwrap());
        ret.push(((self.bottom_half >>  0) & 0xFF).try_into().unwrap());
        ret
    }

    fn from_bytes(bytes: &[u8]) -> Option<Ipv6Address> {
        if bytes.len() != 16 {
            None
        } else {
            let top_half: u64 =
                (u64::try_from(bytes[ 0]).unwrap() << 56) |
                (u64::try_from(bytes[ 1]).unwrap() << 48) |
                (u64::try_from(bytes[ 2]).unwrap() << 40) |
                (u64::try_from(bytes[ 3]).unwrap() << 32) |
                (u64::try_from(bytes[ 4]).unwrap() << 24) |
                (u64::try_from(bytes[ 5]).unwrap() << 16) |
                (u64::try_from(bytes[ 6]).unwrap() <<  8) |
                (u64::try_from(bytes[ 7]).unwrap() <<  0)
            ;
            let bottom_half: u64 =
                (u64::try_from(bytes[ 8]).unwrap() << 56) |
                (u64::try_from(bytes[ 9]).unwrap() << 48) |
                (u64::try_from(bytes[10]).unwrap() << 40) |
                (u64::try_from(bytes[11]).unwrap() << 32) |
                (u64::try_from(bytes[12]).unwrap() << 24) |
                (u64::try_from(bytes[13]).unwrap() << 16) |
                (u64::try_from(bytes[14]).unwrap() <<  8) |
                (u64::try_from(bytes[15]).unwrap() <<  0)
            ;
            Some(Ipv6Address::new(top_half, bottom_half))
        }
    }

    fn bitwise_negate(&self) -> Ipv6Address {
        Ipv6Address::new(
            self.top_half ^ 0xFFFF_FFFF_FFFF_FFFFu64,
            self.bottom_half ^ 0xFFFF_FFFF_FFFF_FFFFu64,
        )
    }

    fn add_addr(&self, other: &Ipv6Address) -> Option<Ipv6Address> {
        Ipv6Address::add_internal(
            self.top_half, self.bottom_half,
            other.top_half, other.bottom_half,
        )
    }

    fn add_offset(&self, offset: i32) -> Option<Ipv6Address> {
        if offset < 0 {
            Ipv6Address::subtract_offset(&self, -offset)
        } else {
            Ipv6Address::add_internal(
                self.top_half, self.bottom_half,
                0, offset.try_into().unwrap(),
            )
        }
    }

    fn subtract_addr(&self, other: &Ipv6Address) -> Option<Ipv6Address> {
        Ipv6Address::sub_internal(
            self.top_half, self.bottom_half,
            other.top_half, other.bottom_half,
        )
    }

    fn subtract_offset(&self, offset: i32) -> Option<Ipv6Address> {
        if offset < 0 {
            Ipv6Address::add_offset(&self, -offset)
        } else {
            Ipv6Address::sub_internal(
                self.top_half, self.bottom_half,
                0, offset.try_into().unwrap(),
            )
        }
    }
}

impl FromStr for Ipv6Address {
    type Err = IpAddressParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut addr_str = String::from(s);
        if addr_str.starts_with(':') {
            addr_str.insert(0, '0');
        }
        if addr_str.ends_with(':') {
            addr_str.push('0');
        }

        let chunks: Vec<&str> = addr_str.split(":").collect();
        if chunks.len() > 8 {
            return Err(IpAddressParseError::IncorrectChunkCount(chunks.len(), 8));
        }

        // how many shortening elements do we have?
        let shortening_count = chunks.iter()
            .filter(|x| x.len() == 0)
            .count();
        if shortening_count > 1 {
            // "1234::5678::9abc" is invalid
            return Err(IpAddressParseError::TooManyShorteningElements(shortening_count, 1));
        }

        let mut actual_chunks = Vec::new();
        if shortening_count == 0 {
            // full address "123:45:678:9:ab:cd:ef:21"
            if chunks.len() != 8 {
                // too few chunks
                return Err(IpAddressParseError::IncorrectChunkCount(chunks.len(), 8));
            }

            for chunk in chunks.iter() {
                actual_chunks.push(String::from(*chunk));
            }
        } else {
            // shortened address "123::456a"
            for _ in 0..8 {
                actual_chunks.push(String::from("0"));
            }

            // copy from front
            for i in 0..chunks.len() {
                if chunks[i].len() == 0 {
                    break;
                }
                actual_chunks[i] = String::from(chunks[i]);
            }

            // copy from back
            for i in 0..chunks.len() {
                if chunks[chunks.len() - i - 1].len() == 0 {
                    break;
                }
                let j = actual_chunks.len() - i - 1;
                actual_chunks[j] = String::from(chunks[chunks.len() - i - 1]);
            }

            // leave remaining chunks as zero
        }

        let mut top_half: u64 = 0;
        let mut bottom_half: u64 = 0;
        for i in 0..8 {
            let shift_count = 112 - (i * 16);
            let shift_count_within_half = shift_count % 64;
            let into_top_half = i < 4;

            let chunk_value = match u16::from_str_radix(&actual_chunks[i], 16) {
                Ok(v) => v,
                Err(e)
                    => return Err(IpAddressParseError::ChunkParseError(i, actual_chunks[i].clone(), e)),
            };

            if into_top_half {
                top_half |= u64::from(chunk_value) << shift_count_within_half;
            } else {
                bottom_half |= u64::from(chunk_value) << shift_count_within_half;
            }
        }

        Ok(Ipv6Address::new(top_half, bottom_half))
    }
}

impl fmt::Display for Ipv6Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.top_half == 0 && self.bottom_half == 0 {
            return write!(f, "::");
        }

        let chunks = self.to_chunks();

        // attempt to shorten
        let mut i = 0;
        let mut zero_index: Option<usize> = None;
        let mut zero_length = 0;
        while i < 8 {
            if chunks[i] != 0 {
                i += 1;
                continue;
            }

            // zero chunk!
            let mut j = i + 1;
            while j < 8 {
                if chunks[j] != 0 {
                    break;
                }
                j += 1;
            }

            if zero_length < j - i {
                // new longest zero chunk found!
                zero_index = Some(i);
                zero_length = j - i;
            }

            // continue at j
            i = j;
        }

        let mut chunk_strings = Vec::new();
        let mut i = 0;
        while i < 8 {
            if zero_index == Some(i) {
                if i == 0 {
                    // the initial part of the address is zero
                    chunk_strings.push(String::from(""));
                }

                // an empty chunk causes two adjacent colons
                chunk_strings.push(String::from(""));

                // jump past the length
                i += zero_length;

                if i == 8 {
                    // the final part of the address is zero
                    chunk_strings.push(String::from(""));
                }
            } else {
                chunk_strings.push(format!("{:x}", chunks[i]));
                i += 1;
            }
        }

        write!(f, "{}", chunk_strings.join(":"))
    }
}

impl BitAnd for Ipv6Address {
    type Output = Ipv6Address;

    fn bitand(self, rhs: Self) -> Self::Output {
        Ipv6Address::new(
            self.top_half & rhs.top_half,
            self.bottom_half & rhs.bottom_half,
        )
    }
}

impl BitOr for Ipv6Address {
    type Output = Ipv6Address;

    fn bitor(self, rhs: Self) -> Self::Output {
        Ipv6Address::new(
            self.top_half | rhs.top_half,
            self.bottom_half | rhs.bottom_half,
        )
    }
}

impl BitXor for Ipv6Address {
    type Output = Ipv6Address;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Ipv6Address::new(
            self.top_half ^ rhs.top_half,
            self.bottom_half ^ rhs.bottom_half,
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IpAddressParseError {
    UnknownAddressType,
    IncorrectChunkCount(usize, usize),
    EmptyChunk(usize),
    ChunkParseError(usize, String, ParseIntError),
    ChunkOutOfRange(usize, u32, u32, u32),
    TooManyShorteningElements(usize, usize),
}
impl fmt::Display for IpAddressParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IpAddressParseError::UnknownAddressType
                => write!(f, "unknown IP address type"),
            IpAddressParseError::IncorrectChunkCount(got, expected)
                => write!(f, "IP address has {} chunk(s); expected {}", got, expected),
            IpAddressParseError::EmptyChunk(chunk_idx)
                => write!(f, "IP address chunk with index {} is empty", chunk_idx),
            IpAddressParseError::ChunkParseError(chunk_idx, chunk, error)
                => write!(f, "failed to parse IP address chunk with index {} ({:?}): {}", chunk_idx, chunk, error),
            IpAddressParseError::ChunkOutOfRange(chunk_idx, got, min, max)
                => write!(f, "IP address chunk with index {} ({}) is out of range {} <= n <= {} chunk", chunk_idx, got, min, max),
            IpAddressParseError::TooManyShorteningElements(got, expected_max)
                => write!(f, "IP address has {} shortening elements; expected maximum {}", got, expected_max),
        }
    }
}
impl Error for IpAddressParseError {
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_ipv4_format() {
        assert_eq!("0.0.0.0", Ipv4Address::new(0x00000000).to_string());
        assert_eq!("255.255.255.255", Ipv4Address::new(0xFFFFFFFF).to_string());
        assert_eq!("18.52.86.120", Ipv4Address::new(0x12345678).to_string());
        assert_eq!("127.0.0.1", Ipv4Address::new(0x7F000001).to_string());
    }

    fn parse_ipv4(s: &str) -> Result<Ipv4Address, IpAddressParseError> {
        s.parse()
    }

    #[test]
    fn test_ipv4_parse() {
        assert_eq!(Ok(Ipv4Address::new(0x00000000)), parse_ipv4("0.0.0.0"));
        assert_eq!(Ok(Ipv4Address::new(0x00000000)), parse_ipv4("00.000.00000.0"));
        assert_eq!(Ok(Ipv4Address::new(0x01020304)), parse_ipv4("1.2.3.4"));
        assert_eq!(Ok(Ipv4Address::new(0x01020304)), parse_ipv4("01.002.00003.4"));
        assert_eq!(Ok(Ipv4Address::new(0xFFFFFFFF)), parse_ipv4("255.255.255.255"));
        assert_eq!(Ok(Ipv4Address::new(0x12345678)), parse_ipv4("18.52.86.120"));
        assert_eq!(Ok(Ipv4Address::new(0x7F000001)), parse_ipv4("127.0.0.1"));

        assert_eq!(Err(IpAddressParseError::IncorrectChunkCount(2, 4)), parse_ipv4("."));
        assert_eq!(Err(IpAddressParseError::IncorrectChunkCount(3, 4)), parse_ipv4("1.2.3"));
        assert_eq!(Err(IpAddressParseError::IncorrectChunkCount(5, 4)), parse_ipv4("1.2.3.4.5"));
        if let Err(IpAddressParseError::ChunkParseError(idx, s, _)) = parse_ipv4("1.2.-3.4") {
            assert_eq!(2, idx);
            assert_eq!("-3", s);
        } else {
            panic!();
        }
        assert_eq!(Err(IpAddressParseError::ChunkOutOfRange(1, 256, 0, 255)), parse_ipv4("255.256.255.255"));
        if let Err(IpAddressParseError::ChunkParseError(idx, s, _)) = parse_ipv4("0xFF.256.255.255") {
            assert_eq!(0, idx);
            assert_eq!("0xFF", s);
        }
    }

    #[test]
    fn test_ipv4_bytes() {
        assert_eq!(vec![0, 0, 0, 0], Ipv4Address::new(0x00000000).to_bytes());
        assert_eq!(vec![1, 2, 3, 4], Ipv4Address::new(0x01020304).to_bytes());
        assert_eq!(vec![255, 255, 255, 255], Ipv4Address::new(0xFFFFFFFF).to_bytes());
        assert_eq!(vec![18, 52, 86, 120], Ipv4Address::new(0x12345678).to_bytes());
        assert_eq!(vec![127, 0, 0, 1], Ipv4Address::new(0x7F000001).to_bytes());
    }

    #[test]
    fn test_from_ipv4_bytes() {
        assert_eq!(Some(Ipv4Address::new(0x00000000)), Ipv4Address::from_bytes(&vec![0, 0, 0, 0]));
        assert_eq!(Some(Ipv4Address::new(0x01020304)), Ipv4Address::from_bytes(&vec![1, 2, 3, 4]));
        assert_eq!(Some(Ipv4Address::new(0xFFFFFFFF)), Ipv4Address::from_bytes(&vec![255, 255, 255, 255]));
        assert_eq!(Some(Ipv4Address::new(0x12345678)), Ipv4Address::from_bytes(&vec![18, 52, 86, 120]));
        assert_eq!(Some(Ipv4Address::new(0x7F000001)), Ipv4Address::from_bytes(&vec![127, 0, 0, 1]));

        assert_eq!(None, Ipv4Address::from_bytes(&vec![1, 2, 3]));
        assert_eq!(None, Ipv4Address::from_bytes(&vec![1, 2, 3, 4, 5]));
    }

    #[test]
    fn test_ipv4_eq() {
        fn teq(val: u32) {
            let left = Ipv4Address::new(val);
            let right = Ipv4Address::new(val);

            assert_eq!(left, right);
        }

        teq(0x00000000);
        teq(0xFFFFFFFF);
        teq(0x7F000001);
        teq(0x7F1234AB);
    }

    #[test]
    fn test_ipv4_and() {
        fn tand(expected: u32, left: u32, right: u32) {
            let expected_addr = Ipv4Address::new(expected);
            let left_addr = Ipv4Address::new(left);
            let right_addr = Ipv4Address::new(right);

            assert_eq!(expected_addr, left_addr & right_addr);
        }

        tand(0x7F000000, 0x7F000001, 0xFF000000);
        tand(0xC0A8A900, 0xC0A8A917, 0xFFFFFF00);
    }

    #[test]
    fn test_ipv6_format() {
        assert_eq!("::", Ipv6Address::new(0x0, 0x0).to_string());
        assert_eq!("::1", Ipv6Address::new(0x0, 0x1).to_string());
        assert_eq!("::123:4567", Ipv6Address::new(0x0000000000000000, 0x0000000001234567).to_string());
        assert_eq!("12:34::", Ipv6Address::new(0x0012003400000000, 0x0000000000000000).to_string());
        assert_eq!("abcd:123::256", Ipv6Address::new(0xABCD012300000000, 0x0000000000000256).to_string());
        assert_eq!("abcd::123:256", Ipv6Address::new(0xABCD000000000000, 0x0000000001230256).to_string());
        assert_eq!("ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff", Ipv6Address::new(0xFFFFFFFFFFFFFFFF, 0xFFFFFFFFFFFFFFFF).to_string());
        assert_eq!("fec0:abcd:1234:defa:1337:8008:1224:2323", Ipv6Address::new(0xFEC0ABCD1234DEFA, 0x1337800812242323).to_string());
    }

    #[test]
    fn test_ipv6_parse() {
        fn tp(top_half: u64, bottom_half: u64, input: &str) {
            let parsed: Ipv6Address = input.parse().unwrap();
            assert_eq!(Ipv6Address::new(top_half, bottom_half), parsed);
        }

        tp(0x0000000000000000, 0x0000000000000000, "0000:0000:0000:0000:0000:0000:0000:0000");
        tp(0x0000000000000000, 0x0000000000000000, "0:0:0:0:0:0:0:0");
        tp(0x0000000000000000, 0x0000000000000000, "::");
        tp(0x0000000000000000, 0x0000000000000000, "0:00:000:0000:000:0:0000:00");
        tp(0x0000000000000000, 0x0000000000000000, "0:00:000::0:0000:00");

        tp(0x0000000000000000, 0x0000000000000001, "0000:0000:0000:0000:0000:0000:0000:0001");
        tp(0x0000000000000000, 0x0000000000000001, "0:0:0:0:0:0:0:1");
        tp(0x0000000000000000, 0x0000000000000001, "::1");
        tp(0x0000000000000000, 0x0000000000000001, "0:00:000:0000:000:0:0000:01");
        tp(0x0000000000000000, 0x0000000000000001, "0:00:000::0:0000:01");

        tp(0xFE80000000000000, 0xA55E55ED0B501E7E, "fe80:0000:0000:0000:a55e:55ed:0b50:1e7e");
        tp(0xFE80000000000000, 0xA55E55ED0B501E7E, "fe80:0:0:0:a55e:55ed:b50:1e7e");
        tp(0xFE80000000000000, 0xA55E55ED0B501E7E, "fe80::a55e:55ed:0b50:1e7e");
        tp(0xFE80000000000000, 0xA55E55ED0B501E7E, "fe80::a55e:55ed:b50:1e7e");

        fn p6(input: &str) -> Result<Ipv6Address, IpAddressParseError> {
            input.parse()
        }

        assert_eq!(Err(IpAddressParseError::IncorrectChunkCount(2, 8)), p6(":"));
        assert_eq!(Err(IpAddressParseError::IncorrectChunkCount(2, 8)), p6("a:"));
        assert_eq!(Err(IpAddressParseError::IncorrectChunkCount(2, 8)), p6(":a"));
        assert_eq!(Err(IpAddressParseError::TooManyShorteningElements(2, 1)), p6(":::"));
        assert_eq!(Err(IpAddressParseError::TooManyShorteningElements(2, 1)), p6("fe80::a55e:55ed::0b50:1e7e"));
        if let Err(IpAddressParseError::ChunkParseError(idx, s, _)) = p6("fe80::a55e:55ed:0b50:1ete") {
            assert_eq!(7, idx);
            assert_eq!("1ete", s);
        } else {
            panic!();
        }
    }

    #[test]
    fn test_ipv6_bytes() {
        fn tb(bs: Vec<u8>, t: u64, b: u64) {
            let ip = Ipv6Address::new(t, b);
            assert_eq!(ip.to_bytes(), bs);
        }
        tb(vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], 0x0000000000000000, 0x0000000000000000);
        tb(vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01], 0x0000000000000000, 0x0000000000000001);
        tb(vec![0xFE, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xA5, 0x5E, 0x55, 0xED, 0x0B, 0x50, 0x1E, 0x7E], 0xFE80000000000000, 0xA55E55ED0B501E7E);
        tb(vec![0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0xFE, 0xDC, 0xBA, 0x98, 0x76, 0x54, 0x32, 0x10], 0x123456789ABCDEF0, 0xFEDCBA9876543210);
    }

    #[test]
    fn test_from_ipv6_bytes() {
        fn tfb(t: u64, b: u64, bs: Vec<u8>) {
            let ip = Ipv6Address::from_bytes(&bs).unwrap();
            assert_eq!(Ipv6Address::new(t, b), ip);
        }

        tfb(0x0000000000000000, 0x0000000000000000, vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        tfb(0x0000000000000000, 0x0000000000000001, vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01]);
        tfb(0xFE80000000000000, 0xA55E55ED0B501E7E, vec![0xFE, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xA5, 0x5E, 0x55, 0xED, 0x0B, 0x50, 0x1E, 0x7E]);
        tfb(0x123456789ABCDEF0, 0xFEDCBA9876543210, vec![0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0xFE, 0xDC, 0xBA, 0x98, 0x76, 0x54, 0x32, 0x10]);

        assert_eq!(None, Ipv6Address::from_bytes(&vec![0xFE, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xA5, 0x5E, 0x55, 0xED, 0x0B, 0x50, 0x1E]));
        assert_eq!(None, Ipv6Address::from_bytes(&vec![0xFE, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xA5, 0x5E, 0x55, 0xED, 0x0B, 0x50, 0x1E, 0x7E, 0x99]));
    }

    #[test]
    fn test_ipv6_eq() {
        fn teq(top_half: u64, bottom_half: u64) {
            let left = Ipv6Address::new(top_half, bottom_half);
            let right = Ipv6Address::new(top_half, bottom_half);

            assert_eq!(left, right);
        }

        teq(0x0000000000000000, 0x0000000000000000);
        teq(0x0000000000000000, 0x0000000000000001);
        teq(0xFE80000000000000, 0xA55E55ED0B501E7E);
        teq(0x123456789ABCDEF0, 0xFEDCBA9876543210);
    }

    #[test]
    fn test_ipv6_and() {
        fn tand(exp_top: u64, exp_bot: u64, left_top: u64, left_bot: u64, right_top: u64, right_bot: u64) {
            let exp = Ipv6Address::new(exp_top, exp_bot);
            let left = Ipv6Address::new(left_top, left_bot);
            let right = Ipv6Address::new(right_top, right_bot);

            assert_eq!(exp, left & right);
        }

        tand(0x1214121812141210, 0x1214121812141210, 0x123456789ABCDEF0, 0xFEDCBA9876543210, 0xFEDCBA9876543210, 0x123456789ABCDEF0);
    }
}

use std::convert::TryFrom;

use crate::addr::IpAddress;
use crate::cidr::prefix_from_subnet_mask_bytes;


/// Converts a slice of bytes into its constituent bits (most significant bit first).
pub fn bytes_to_bits(bytes: &[u8]) -> Vec<bool> {
    let mut ret = Vec::with_capacity(bytes.len() * 8);
    for byte in bytes {
        for bit_idx in 0..8 {
            ret.push(
                byte & (1 << (7 - bit_idx)) != 0
            );
        }
    }
    ret
}

/// Converts a slice of bytes into a string of '1' and '0' characters representing the bit values
/// (most significant bit first).
pub fn bytes_to_binary(bytes: &[u8]) -> String {
    let mut ret = String::with_capacity(bytes.len() * 8);
    for bit in bytes_to_bits(bytes) {
        ret.push(if bit { '1' } else { '0' });
    }
    ret
}

/// Converts a slice of bit values into bytes. Assumes that bits are ordered most significant bit
/// first. If the number of bits does not fit into a whole number of bytes, the bit slice is assumed
/// to be padded with zeroes at the end up to a byte boundary.
pub fn bits_to_bytes(bits: &[bool]) -> Vec<u8> {
    let mut byte_count = bits.len() / 8;
    if bits.len() % 8 != 0 {
        byte_count += 1;
    }

    let mut bytes = Vec::with_capacity(byte_count);
    let mut cur_byte = 0u8;
    for i in 0..bits.len() {
        if bits[i] {
            let shift_count = 7 - (i % 8);
            cur_byte |= 1 << shift_count;
        }

        if i % 8 == 7 {
            bytes.push(cur_byte);
            cur_byte = 0;
        }
    }

    if bits.len() % 8 != 0 {
        // incomplete byte; append it too
        bytes.push(cur_byte);
    }

    bytes
}

/// Converts the given address from its (potentially mixed) subnet mask to the equally-sized CIDR
/// subnet mask. This can be reversed using `weave_address`.
pub fn unravel_address<A: IpAddress>(addr: A, subnet_mask: A) -> A {
    let mask_bytes = subnet_mask.to_bytes();
    if prefix_from_subnet_mask_bytes(&mask_bytes).is_some() {
        // nothing to unravel :)
        return addr;
    }

    // given an address ABCDEFGH with subnet mask 11001001, turn it into ABEHCDFG (i.e. with subnet mask 11110000)
    let addr_bytes = addr.to_bytes();
    let addr_bits = bytes_to_bits(&addr_bytes);
    let mask_bits = bytes_to_bits(&mask_bytes);

    let mut net_bits = Vec::with_capacity(addr_bits.len());
    let mut host_bits = Vec::with_capacity(addr_bits.len());
    for (addr_bit, is_net) in addr_bits.iter().zip(mask_bits.iter()) {
        if *is_net {
            net_bits.push(*addr_bit)
        } else {
            host_bits.push(*addr_bit);
        }
    }

    let mut unraveled_bits = Vec::new();
    unraveled_bits.append(&mut net_bits);
    unraveled_bits.append(&mut host_bits);
    let ret_bytes = bits_to_bytes(&unraveled_bits);

    A::from_bytes(&ret_bytes).expect("address from bytes")
}

/// Converts the given address from the equally-sized CIDR subnet mask to the given (potentially
/// mixed) subnet mask. This is the reverse operation of `unravel_address`.
pub fn weave_address<A: IpAddress>(addr: A, subnet_mask: A) -> A {
    let mask_bytes = subnet_mask.to_bytes();
    if prefix_from_subnet_mask_bytes(&mask_bytes).is_some() {
        // nothing to weave :)
        return addr;
    }

    // given an address ABCDEFGH with subnet mask 11001001, convert from subnet mask 11110000 turning it into ABEFCGHD
    let addr_bytes = addr.to_bytes();
    let addr_bits = bytes_to_bits(&addr_bytes);
    let cidr_prefix: usize = mask_bytes.iter()
        .map(|b| usize::try_from(b.count_ones()).unwrap())
        .sum();
    let mask_bits = bytes_to_bits(&mask_bytes);

    let mut net_bits = Vec::with_capacity(addr_bits.len());
    let mut host_bits = Vec::with_capacity(addr_bits.len());

    // split up the bits
    for i in 0..addr_bits.len() {
        let addr_bit = addr_bits[i];
        let is_net = i < cidr_prefix;

        if is_net {
            net_bits.push(addr_bit)
        } else {
            host_bits.push(addr_bit);
        }
    }

    // weave the bits
    let mut ret_bits = Vec::with_capacity(addr_bits.len());
    let mut net_index = 0;
    let mut host_index = 0;
    for is_net in mask_bits {
        let should_set_bit = if is_net {
            let ssb = net_bits[net_index];
            net_index += 1;
            ssb
        } else {
            let ssb = host_bits[host_index];
            host_index += 1;
            ssb
        };
        ret_bits.push(should_set_bit);
    }

    let ret_bytes = bits_to_bytes(&ret_bits);
    A::from_bytes(&ret_bytes).expect("address from bytes")
}

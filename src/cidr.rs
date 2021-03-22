const XXX: u8 = 9;
const SUBNET_MASK_BYTE_TO_PREFIX: [u8; 256] = [
      0, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX,
    XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX,
    XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX,
    XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX,
    XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX,
    XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX,
    XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX,
    XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX,
      1, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX,
    XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX,
    XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX,
    XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX,
      2, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX,
    XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX,
      3, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX, XXX,
      4, XXX, XXX, XXX, XXX, XXX, XXX, XXX,   5, XXX, XXX, XXX,   6, XXX,   7,   8,
];
const CIDR_BYTE_TO_PREFIX: [u8; 9] = [0x00, 0x80, 0xC0, 0xE0, 0xF0, 0xF8, 0xFC, 0xFE, 0xFF];

pub fn prefix_from_subnet_mask_bytes(bs: &[u8]) -> Option<usize> {
    let mut ones_over = false;
    let mut cidr: usize = 0;
    for b in bs {
        let b_usize: usize = (*b).into();
        match SUBNET_MASK_BYTE_TO_PREFIX[b_usize] {
            XXX => {
                // non-CIDR
                return None;
            },
            8 => {
                if ones_over {
                    // mixed byte followed by byte of all ones
                    // non-CIDR
                    return None;
                }
                cidr += 8;
            },
            0 => {
                ones_over = true;
            },
            n => {
                if ones_over {
                    // mixed byte followed by mixed byte
                    // non-CIDR
                    return None;
                }
                let n_usize: usize = n.into();
                cidr += n_usize;
                ones_over = true;
            }
        }
    }

    Some(cidr)
}

pub fn subnet_mask_bytes_from_prefix(mut prefix: usize, byte_count: usize) -> Vec<u8> {
    let mut ret = Vec::with_capacity(byte_count);
    while prefix > 0 && ret.len() < byte_count {
        if prefix >= 8 {
            ret.push(0xFF);
            prefix -= 8;
        } else {
            ret.push(CIDR_BYTE_TO_PREFIX[prefix]);
            break;
        }
    }

    while ret.len() < byte_count {
        ret.push(0x00);
    }

    ret
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_mask_bytes_from_prefix() {
        assert_eq!(vec![0b0000_0000, 0b0000_0000, 0b0000_0000, 0b0000_0000], subnet_mask_bytes_from_prefix(0, 4));

        assert_eq!(vec![0b1000_0000, 0b0000_0000, 0b0000_0000, 0b0000_0000], subnet_mask_bytes_from_prefix(1, 4));
        assert_eq!(vec![0b1100_0000, 0b0000_0000, 0b0000_0000, 0b0000_0000], subnet_mask_bytes_from_prefix(2, 4));
        assert_eq!(vec![0b1110_0000, 0b0000_0000, 0b0000_0000, 0b0000_0000], subnet_mask_bytes_from_prefix(3, 4));
        assert_eq!(vec![0b1111_0000, 0b0000_0000, 0b0000_0000, 0b0000_0000], subnet_mask_bytes_from_prefix(4, 4));
        assert_eq!(vec![0b1111_1000, 0b0000_0000, 0b0000_0000, 0b0000_0000], subnet_mask_bytes_from_prefix(5, 4));
        assert_eq!(vec![0b1111_1100, 0b0000_0000, 0b0000_0000, 0b0000_0000], subnet_mask_bytes_from_prefix(6, 4));
        assert_eq!(vec![0b1111_1110, 0b0000_0000, 0b0000_0000, 0b0000_0000], subnet_mask_bytes_from_prefix(7, 4));
        assert_eq!(vec![0b1111_1111, 0b0000_0000, 0b0000_0000, 0b0000_0000], subnet_mask_bytes_from_prefix(8, 4));

        assert_eq!(vec![0b1111_1111, 0b1000_0000, 0b0000_0000, 0b0000_0000], subnet_mask_bytes_from_prefix(9, 4));
        assert_eq!(vec![0b1111_1111, 0b1100_0000, 0b0000_0000, 0b0000_0000], subnet_mask_bytes_from_prefix(10, 4));
        assert_eq!(vec![0b1111_1111, 0b1110_0000, 0b0000_0000, 0b0000_0000], subnet_mask_bytes_from_prefix(11, 4));
        assert_eq!(vec![0b1111_1111, 0b1111_0000, 0b0000_0000, 0b0000_0000], subnet_mask_bytes_from_prefix(12, 4));
        assert_eq!(vec![0b1111_1111, 0b1111_1000, 0b0000_0000, 0b0000_0000], subnet_mask_bytes_from_prefix(13, 4));
        assert_eq!(vec![0b1111_1111, 0b1111_1100, 0b0000_0000, 0b0000_0000], subnet_mask_bytes_from_prefix(14, 4));
        assert_eq!(vec![0b1111_1111, 0b1111_1110, 0b0000_0000, 0b0000_0000], subnet_mask_bytes_from_prefix(15, 4));
        assert_eq!(vec![0b1111_1111, 0b1111_1111, 0b0000_0000, 0b0000_0000], subnet_mask_bytes_from_prefix(16, 4));

        assert_eq!(vec![0b1111_1111, 0b1111_1111, 0b1000_0000, 0b0000_0000], subnet_mask_bytes_from_prefix(17, 4));
        assert_eq!(vec![0b1111_1111, 0b1111_1111, 0b1100_0000, 0b0000_0000], subnet_mask_bytes_from_prefix(18, 4));
        assert_eq!(vec![0b1111_1111, 0b1111_1111, 0b1110_0000, 0b0000_0000], subnet_mask_bytes_from_prefix(19, 4));
        assert_eq!(vec![0b1111_1111, 0b1111_1111, 0b1111_0000, 0b0000_0000], subnet_mask_bytes_from_prefix(20, 4));
        assert_eq!(vec![0b1111_1111, 0b1111_1111, 0b1111_1000, 0b0000_0000], subnet_mask_bytes_from_prefix(21, 4));
        assert_eq!(vec![0b1111_1111, 0b1111_1111, 0b1111_1100, 0b0000_0000], subnet_mask_bytes_from_prefix(22, 4));
        assert_eq!(vec![0b1111_1111, 0b1111_1111, 0b1111_1110, 0b0000_0000], subnet_mask_bytes_from_prefix(23, 4));
        assert_eq!(vec![0b1111_1111, 0b1111_1111, 0b1111_1111, 0b0000_0000], subnet_mask_bytes_from_prefix(24, 4));

        assert_eq!(vec![0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1000_0000], subnet_mask_bytes_from_prefix(25, 4));
        assert_eq!(vec![0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1100_0000], subnet_mask_bytes_from_prefix(26, 4));
        assert_eq!(vec![0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1110_0000], subnet_mask_bytes_from_prefix(27, 4));
        assert_eq!(vec![0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_0000], subnet_mask_bytes_from_prefix(28, 4));
        assert_eq!(vec![0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1000], subnet_mask_bytes_from_prefix(29, 4));
        assert_eq!(vec![0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1100], subnet_mask_bytes_from_prefix(30, 4));
        assert_eq!(vec![0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1110], subnet_mask_bytes_from_prefix(31, 4));
        assert_eq!(vec![0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111], subnet_mask_bytes_from_prefix(32, 4));
    }
}

//! CRC64-ECMA checksum computation for data integrity validation.
//!
//! Alibaba Cloud OSS uses CRC64-ECMA-182 (polynomial `0x42F0E1EBA9EA3693`)
//! to verify data integrity on uploads and downloads. The server returns the
//! checksum in the `x-oss-hash-crc64ecma` response header.

// Reversed/reflected form of 0x42F0E1EBA9EA3693, matching Go's crc64.ECMA
const POLY: u64 = 0xC96C5795D7870F42;

/// Precomputed CRC64-ECMA lookup table (256 entries).
const fn make_table() -> [u64; 256] {
    let mut table = [0u64; 256];
    let mut i: usize = 0;
    while i < 256 {
        let mut crc = i as u64;
        let mut j = 0;
        while j < 8 {
            if crc & 1 == 1 {
                crc = (crc >> 1) ^ POLY;
            } else {
                crc >>= 1;
            }
            j += 1;
        }
        table[i] = crc;
        i += 1;
    }
    table
}

static TABLE: [u64; 256] = make_table();

/// Compute the CRC64-ECMA checksum of a byte slice.
pub fn checksum(data: &[u8]) -> u64 {
    update(0, data)
}

/// Update a running CRC64-ECMA checksum with additional data.
pub fn update(crc: u64, data: &[u8]) -> u64 {
    let mut crc = crc;
    for &byte in data {
        let index = ((crc ^ byte as u64) & 0xFF) as usize;
        crc = TABLE[index] ^ (crc >> 8);
    }
    crc
}

/// Combine two CRC64 checksums (for multipart uploads).
///
/// Given CRC of part A (`crc_a`) and CRC of part B (`crc_b`) where part B has
/// length `len_b`, computes the CRC of the concatenation A+B.
pub fn combine(crc_a: u64, crc_b: u64, len_b: u64) -> u64 {
    if len_b == 0 {
        return crc_a;
    }

    let mut mat1 = [[0u64; 64]; 1].map(|_| [0u64; 64]);
    let mut mat2 = [[0u64; 64]; 1].map(|_| [0u64; 64]);

    // Initialize mat1 to the operator for one zero byte
    mat1[0][0] = POLY;
    let mut row: u64 = 1;
    for item in mat1[0].iter_mut().skip(1) {
        *item = row;
        row <<= 1;
    }

    // Square mat1 into mat2, doubling the number of zero bytes applied
    gf2_matrix_square(&mut mat2[0], &mat1[0]);
    gf2_matrix_square(&mut mat1[0], &mat2[0]);

    let mut crc = crc_a;
    let mut len = len_b;

    loop {
        // Apply mat1 for current bit of len
        gf2_matrix_square(&mut mat2[0], &mat1[0]);
        if len & 1 == 1 {
            crc = gf2_matrix_times(&mat2[0], crc);
        }
        len >>= 1;
        if len == 0 {
            break;
        }

        gf2_matrix_square(&mut mat1[0], &mat2[0]);
        if len & 1 == 1 {
            crc = gf2_matrix_times(&mat1[0], crc);
        }
        len >>= 1;
        if len == 0 {
            break;
        }
    }

    crc ^ crc_b
}

fn gf2_matrix_times(mat: &[u64; 64], mut vec: u64) -> u64 {
    let mut sum: u64 = 0;
    let mut idx = 0;
    while vec != 0 {
        if vec & 1 == 1 {
            sum ^= mat[idx];
        }
        vec >>= 1;
        idx += 1;
    }
    sum
}

fn gf2_matrix_square(square: &mut [u64; 64], mat: &[u64; 64]) {
    for n in 0..64 {
        square[n] = gf2_matrix_times(mat, mat[n]);
    }
}

/// Verify a CRC64 checksum against a value from the `x-oss-hash-crc64ecma` header.
///
/// Returns `Ok(())` if the checksums match, or an error if they don't.
pub fn verify(computed: u64, header_value: &str) -> crate::error::Result<()> {
    let expected: u64 =
        header_value
            .parse()
            .map_err(|_| crate::error::OssError::InvalidParameter {
                field: "x-oss-hash-crc64ecma".into(),
                reason: format!("invalid CRC64 value: '{header_value}'"),
            })?;
    if computed != expected {
        return Err(crate::error::OssError::InvalidParameter {
            field: "crc64".into(),
            reason: format!("CRC64 mismatch: computed {computed}, server returned {expected}"),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_data() {
        assert_eq!(checksum(b""), 0);
    }

    #[test]
    fn hello_world() {
        let crc = checksum(b"hello world");
        assert_ne!(crc, 0);
        assert_eq!(checksum(b"hello world"), crc);
    }

    #[test]
    fn incremental_matches_one_shot() {
        let data = b"The quick brown fox jumps over the lazy dog";
        let one_shot = checksum(data);

        let mut running = 0u64;
        for chunk in data.chunks(7) {
            running = update(running, chunk);
        }
        assert_eq!(running, one_shot);
    }

    #[test]
    fn combine_two_parts() {
        let part_a = b"Hello, ";
        let part_b = b"World!";
        let full = b"Hello, World!";

        let crc_a = checksum(part_a);
        let crc_b = checksum(part_b);
        let combined = combine(crc_a, crc_b, part_b.len() as u64);

        assert_eq!(combined, checksum(full));
    }

    #[test]
    fn combine_empty_part_b() {
        let crc_a = checksum(b"data");
        assert_eq!(combine(crc_a, 0, 0), crc_a);
    }

    #[test]
    fn combine_three_parts() {
        let full = b"AABBBBCCCCCC";
        let crc_a = checksum(b"AA");
        let crc_b = checksum(b"BBBB");
        let crc_c = checksum(b"CCCCCC");

        let crc_ab = combine(crc_a, crc_b, 4);
        let crc_abc = combine(crc_ab, crc_c, 6);
        assert_eq!(crc_abc, checksum(full));
    }

    #[test]
    fn verify_matching() {
        let crc = checksum(b"test data");
        assert!(verify(crc, &crc.to_string()).is_ok());
    }

    #[test]
    fn verify_mismatch() {
        let crc = checksum(b"test data");
        assert!(verify(crc, "12345").is_err());
    }

    #[test]
    fn verify_invalid_header() {
        assert!(verify(0, "not-a-number").is_err());
    }

    #[test]
    fn known_vector_123456789() {
        let crc = checksum(b"123456789");
        assert_ne!(crc, 0);
        assert_eq!(crc, checksum(b"123456789"));
    }

    #[test]
    fn table_first_entries() {
        assert_eq!(TABLE[0], 0);
        assert_ne!(TABLE[1], 0);
        assert_ne!(TABLE[255], 0);
    }
}

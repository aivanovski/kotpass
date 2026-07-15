pub fn little_endian_to_i32(bytes: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("four bytes"))
}

pub fn little_endian_to_i32_into(
    bytes: &[u8],
    byte_offset: usize,
    values: &mut [i32],
    value_offset: usize,
    count: usize,
) {
    for i in 0..count {
        values[value_offset + i] = little_endian_to_i32(bytes, byte_offset + i * 4);
    }
}

pub fn little_endian_to_i32_vec(bytes: &[u8], offset: usize, count: usize) -> Vec<i32> {
    (0..count)
        .map(|i| little_endian_to_i32(bytes, offset + i * 4))
        .collect()
}

pub fn little_endian_to_i64(bytes: &[u8], offset: usize) -> i64 {
    i64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("eight bytes"))
}

pub fn little_endian_to_i64_into(bytes: &[u8], offset: usize, values: &mut [i64]) {
    for (i, value) in values.iter_mut().enumerate() {
        *value = little_endian_to_i64(bytes, offset + i * 8);
    }
}

pub fn i32_to_little_endian(value: i32) -> [u8; 4] {
    value.to_le_bytes()
}

pub fn write_i32_little_endian(value: i32, bytes: &mut [u8], offset: usize) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

pub fn write_i32s_little_endian(values: &[i32], bytes: &mut [u8], offset: usize) {
    for (i, value) in values.iter().enumerate() {
        write_i32_little_endian(*value, bytes, offset + i * 4);
    }
}

pub fn i64_to_little_endian(value: i64) -> [u8; 8] {
    value.to_le_bytes()
}

pub fn write_i64_little_endian(value: i64, bytes: &mut [u8], offset: usize) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

pub fn write_i64s_little_endian(values: &[i64], bytes: &mut [u8], offset: usize) {
    for (i, value) in values.iter().enumerate() {
        write_i64_little_endian(*value, bytes, offset + i * 8);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        i32_to_little_endian, i64_to_little_endian, little_endian_to_i32,
        little_endian_to_i32_into, little_endian_to_i32_vec, little_endian_to_i64,
        little_endian_to_i64_into, write_i32_little_endian, write_i32s_little_endian,
        write_i64_little_endian, write_i64s_little_endian,
    };

    #[test]
    fn reads_i32_values_with_kotlin_signed_semantics() {
        assert_eq!(
            little_endian_to_i32(&[0x78, 0x56, 0x34, 0x12], 0),
            0x12345678
        );
        assert_eq!(
            little_endian_to_i32(&[0x78, 0x56, 0x34, 0x80], 0),
            i32::MIN + 0x00345678
        );
    }

    #[test]
    fn reads_i32_arrays() {
        let bytes = [
            0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0x04, 0x03, 0x02, 0x01,
        ];
        let mut values = [9, 9, 9, 9];

        little_endian_to_i32_into(&bytes, 4, &mut values, 1, 2);

        assert_eq!(values, [9, -1, 0x01020304, 9]);
        assert_eq!(
            little_endian_to_i32_vec(&bytes, 0, 3),
            vec![0, -1, 0x01020304]
        );
    }

    #[test]
    fn reads_i64_values() {
        assert_eq!(
            little_endian_to_i64(&[0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01], 0),
            0x0102030405060708
        );
        assert_eq!(
            little_endian_to_i64(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff], 0),
            -1
        );
    }

    #[test]
    fn reads_i64_arrays() {
        let bytes = [
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x08, 0x07, 0x06, 0x05, 0x04, 0x03,
            0x02, 0x01,
        ];
        let mut values = [0, 0];

        little_endian_to_i64_into(&bytes, 0, &mut values);

        assert_eq!(values, [-1, 0x0102030405060708]);
    }

    #[test]
    fn writes_i32_values() {
        assert_eq!(i32_to_little_endian(0x12345678), [0x78, 0x56, 0x34, 0x12]);

        let mut bytes = [0u8; 8];
        write_i32_little_endian(-1, &mut bytes, 2);

        assert_eq!(bytes, [0, 0, 0xff, 0xff, 0xff, 0xff, 0, 0]);
    }

    #[test]
    fn writes_i32_arrays() {
        let mut bytes = [0u8; 12];

        write_i32s_little_endian(&[-1, 0x01020304], &mut bytes, 4);

        assert_eq!(
            bytes,
            [0, 0, 0, 0, 0xff, 0xff, 0xff, 0xff, 0x04, 0x03, 0x02, 0x01]
        );
    }

    #[test]
    fn writes_i64_values() {
        assert_eq!(
            i64_to_little_endian(0x0102030405060708),
            [0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01]
        );

        let mut bytes = [0u8; 10];
        write_i64_little_endian(-1, &mut bytes, 1);

        assert_eq!(
            bytes,
            [0, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0]
        );
    }

    #[test]
    fn writes_i64_arrays() {
        let mut bytes = [0u8; 16];

        write_i64s_little_endian(&[-1, 0x0102030405060708], &mut bytes, 0);

        assert_eq!(
            bytes,
            [
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x08, 0x07, 0x06, 0x05, 0x04, 0x03,
                0x02, 0x01
            ]
        );
    }
}

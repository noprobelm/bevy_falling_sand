use std::io::Read;

pub(super) fn read_i32(reader: &mut impl Read) -> std::io::Result<i32> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    Ok(i32::from_le_bytes(buf))
}

pub(super) fn read_u16(reader: &mut impl Read) -> std::io::Result<u16> {
    let mut buf = [0u8; 2];
    reader.read_exact(&mut buf)?;
    Ok(u16::from_le_bytes(buf))
}

pub(super) fn read_f32(reader: &mut impl Read) -> std::io::Result<f32> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    Ok(f32::from_le_bytes(buf))
}

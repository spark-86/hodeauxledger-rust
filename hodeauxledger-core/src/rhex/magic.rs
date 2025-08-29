pub fn explode_magic(magic: &[u8; 32]) -> Result<([u8; 4], u8, u8), anyhow::Error> {
    let name: [u8; 4] = magic[0..4].try_into().unwrap();
    let flags = magic[4];
    let version = magic[5];
    Ok((name, flags, version))
}

pub const MAGIC_MORE: u8 = 1 << 0;
pub const MAGIC_RESUME: u8 = 1 << 1;
pub const MAGIC_METADATA: u8 = 1 << 2;

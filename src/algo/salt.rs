use nutype::nutype;

pub const SALT_LEN: usize = 32;

#[nutype(derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash))]
pub struct Salt([u8; SALT_LEN]);

pub fn salt() -> color_eyre::Result<Salt> {
    let mut buf = [0; SALT_LEN];
    getrandom::fill(&mut buf)?;
    Ok(Salt::new(buf))
}

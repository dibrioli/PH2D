//! O registo do mapa de um cel.

use super::{CodecError, Fields, put_field, put_uint, u32_of, uint, unzigzag, zigzag};
use crate::tiles::{TileGrid, TileRef};

pub(super) fn grid_record(g: &TileGrid) -> Vec<u8> {
    let mut b = Vec::with_capacity(g.tiles.len() * 4 + 16);
    put_uint(&mut b, 1, zigzag(g.x));
    put_uint(&mut b, 2, zigzag(g.y));
    put_uint(&mut b, 3, u64::from(g.w));
    put_uint(&mut b, 4, u64::from(g.h));
    let raw: Vec<u8> = g.tiles.iter().flat_map(|t| t.0.to_le_bytes()).collect();
    put_field(&mut b, 5, &raw);
    if (g.col, g.row) != (0, 0) {
        put_uint(&mut b, 6, zigzag(g.col));
        put_uint(&mut b, 7, zigzag(g.row));
    }
    b.push(0);
    b
}

pub(super) fn read_grid(bytes: &[u8]) -> Result<TileGrid, CodecError> {
    let mut g = TileGrid::default();
    for (t, v) in Fields::new(bytes)? {
        match t {
            1 => g.x = unzigzag(uint(v)?)?,
            2 => g.y = unzigzag(uint(v)?)?,
            3 => g.w = u32_of(v, "grid.w")?,
            4 => g.h = u32_of(v, "grid.h")?,
            5 => {
                let (cells, resto) = v.as_chunks::<4>();
                if !resto.is_empty() {
                    return Err(CodecError::Invalid("grid.tiles"));
                }
                g.tiles = cells
                    .iter()
                    .map(|c| TileRef(u32::from_le_bytes(*c)))
                    .collect();
            }
            6 => g.col = unzigzag(uint(v)?)?,
            7 => g.row = unzigzag(uint(v)?)?,
            _ => {}
        }
    }
    Ok(g)
}

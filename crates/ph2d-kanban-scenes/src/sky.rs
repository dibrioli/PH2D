//! A capa de imagem da cena 1, gerada aqui (sem ficheiro de fora, sem licença): um céu de fim de
//! tarde com o sol e um monte, num PNG de 256 × 144 com o deflate «guardado» (sem compressão).

#[must_use]
pub fn sky_png() -> ([u32; 2], Vec<u8>) {
    const W: u32 = 256;
    const H: u32 = 144;
    let mut raw = Vec::with_capacity(((W * 3 + 1) * H) as usize);
    for y in 0..H {
        raw.push(0);
        let t = y as f32 / H as f32;
        for x in 0..W {
            let (dx, dy) = (x as f32 - 180.0, y as f32 - 70.0);
            let sun = (dx * dx + dy * dy).sqrt() < 22.0;
            let hill = (y as f32) > 110.0 - 18.0 * ((x as f32) / 40.0).sin();
            let px = if hill {
                [40, 70 + (t * 30.0) as u8, 50]
            } else if sun {
                [255, 214, 120]
            } else {
                [
                    (70.0 + 180.0 * t) as u8,
                    (90.0 + 60.0 * t) as u8,
                    (170.0 - 60.0 * t) as u8,
                ]
            };
            raw.extend_from_slice(&px);
        }
    }
    let mut z = vec![0x78, 0x01];
    let mut chunks = raw.chunks(65_535).peekable();
    while let Some(c) = chunks.next() {
        let len = c.len() as u16;
        z.push(u8::from(chunks.peek().is_none()));
        z.extend_from_slice(&len.to_le_bytes());
        z.extend_from_slice(&(!len).to_le_bytes());
        z.extend_from_slice(c);
    }
    let (mut a, mut b) = (1u32, 0u32);
    for &v in &raw {
        a = (a + u32::from(v)) % 65_521;
        b = (b + a) % 65_521;
    }
    z.extend_from_slice(&((b << 16) | a).to_be_bytes());
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&W.to_be_bytes());
    ihdr.extend_from_slice(&H.to_be_bytes());
    ihdr.extend_from_slice(&[8, 2, 0, 0, 0]);
    let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
    for (kind, data) in [(*b"IHDR", ihdr), (*b"IDAT", z), (*b"IEND", Vec::new())] {
        png.extend_from_slice(&(data.len() as u32).to_be_bytes());
        let mut crc = !0u32;
        for &v in kind.iter().chain(&data) {
            crc ^= u32::from(v);
            for _ in 0..8 {
                crc = if crc & 1 == 1 {
                    (crc >> 1) ^ 0xEDB8_8320
                } else {
                    crc >> 1
                };
            }
        }
        png.extend_from_slice(&kind);
        png.extend_from_slice(&data);
        png.extend_from_slice(&(!crc).to_be_bytes());
    }
    ([W, H], png)
}

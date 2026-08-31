//! **Renderizar e OLHAR** — o aparelho que faltava aos efeitos de caminho (2026-07-18).
//!
//! Os efeitos foram construídos com 238 gates verdes e mutações a sangrar, e **três deles
//! saíram maus na mesma leva**: o Twist torcia um lowpoly, o Pucker & Bloat era uma escala
//! uniforme e o Repeater dividia os dois eixos pela média das dimensões. Nenhum desses defeitos
//! precisava de olho clínico — precisava de UM olhar, e não havia como olhar.
//!
//! Todos os oráculos perguntavam *"o buffer diz o que eu disse que dizia"*. Nenhum perguntava
//! *"isto parece a ferramenta cujo nome tem"*. É o padrão que a linha do Painter já pagou e
//! nomeou: **RENDERIZAR E OLHAR** (a sonda `push_look_probe`).
//!
//! # Sem dependência nenhuma, de propósito
//!
//! A `ph2d-vec-scene` é deliberadamente `serde` + `postcard` e mais nada (é o que obrigou o
//! `arclen.rs` a existir). Uma sonda que arrastasse um rasterizador para dentro dela pagaria
//! caro por uma cerca já decidida — então aqui está o mínimo: achatar cúbicas, preencher por
//! scanline, e escrever um PNG com blocos deflate **não comprimidos** (o formato admite-os, e
//! isso troca ~40 linhas de código por zero dependências).
//!
//! O ficheiro sai onde `PH2D_FX_LOOK_DIR` mandar, para o Enio abrir e olhar.

/// Um alvo de desenho em RGB, origem no canto superior esquerdo.
pub struct Canvas {
    pub w: usize,
    pub h: usize,
    px: Vec<[u8; 3]>,
}

impl Canvas {
    pub fn new(w: usize, h: usize, bg: [u8; 3]) -> Self {
        Self {
            w,
            h,
            px: vec![bg; w * h],
        }
    }

    pub fn set(&mut self, x: usize, y: usize, c: [u8; 3]) {
        if x < self.w && y < self.h {
            self.px[y * self.w + x] = c;
        }
    }

    /// Mistura `c` sobre o pixel com cobertura `a` em `0..=1` — é o que dá a borda suave, e sem
    /// ela uma sonda de aparência mostra escada onde o produto não tem.
    pub fn blend(&mut self, x: usize, y: usize, c: [u8; 3], a: f64) {
        if x >= self.w || y >= self.h || a <= 0.0 {
            return;
        }
        let a = a.min(1.0);
        let p = &mut self.px[y * self.w + x];
        for k in 0..3 {
            p[k] = (f64::from(p[k]) * (1.0 - a) + f64::from(c[k]) * a).round() as u8;
        }
    }
}

/// **Preenche um conjunto de contornos** (já em coordenadas de pixel), com anti-aliasing por
/// supersampling vertical.
///
/// ⚠️ `even_odd = false` (NON-ZERO) é o defeito, e a primeira versão desta sonda tinha-o ao
/// contrário. Com even-odd, uma forma que se auto-intersecta desenha a sobreposição como BURACO
/// — e o Twist a 240° auto-intersecta-se por natureza. A sonda mostrava um rasgo onde o produto
/// desenha tinta cheia, e eu quase fui perseguir um defeito do desenhador.
pub fn fill(canvas: &mut Canvas, contours: &[Vec<[f64; 2]>], colour: [u8; 3], even_odd: bool) {
    const SUB: usize = 4;
    for y in 0..canvas.h {
        for s in 0..SUB {
            let sy = y as f64 + (s as f64 + 0.5) / SUB as f64;
            // `(x, direção)` — a direção é o que distingue non-zero de even-odd.
            let mut xs: Vec<(f64, i32)> = Vec::new();
            for c in contours {
                let n = c.len();
                for i in 0..n {
                    let (a, b) = (c[i], c[(i + 1) % n]);
                    if (a[1] <= sy) != (b[1] <= sy) {
                        let t = (sy - a[1]) / (b[1] - a[1]);
                        xs.push((a[0] + t * (b[0] - a[0]), if b[1] > a[1] { 1 } else { -1 }));
                    }
                }
            }
            xs.sort_by(|p, q| p.0.total_cmp(&q.0));
            // Varre da esquerda para a direita acumulando o winding; um intervalo é tinta
            // enquanto a regra o disser.
            let mut spans: Vec<(f64, f64)> = Vec::new();
            let mut wind = 0i32;
            for i in 0..xs.len().saturating_sub(1) {
                wind += xs[i].1;
                let inside = if even_odd { (i % 2) == 0 } else { wind != 0 };
                if inside {
                    spans.push((xs[i].0, xs[i + 1].0));
                }
            }
            for (x0, x1) in spans {
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                for x in (x0.max(0.0) as usize)..=(x1.max(0.0) as usize).min(canvas.w - 1) {
                    // Cobertura parcial nas pontas do intervalo — o resto é cheio.
                    let l = (x as f64).max(x0);
                    let r = ((x + 1) as f64).min(x1);
                    let cov = (r - l).clamp(0.0, 1.0) / SUB as f64;
                    canvas.blend(x, y, colour, cov);
                }
            }
        }
    }
}

/// Desenha o contorno como linha fina (1 px, sem AA) — é o que mostra ONDE as âncoras caíram.
pub fn stroke(canvas: &mut Canvas, contours: &[Vec<[f64; 2]>], colour: [u8; 3]) {
    for c in contours {
        let n = c.len();
        for i in 0..n {
            let (a, b) = (c[i], c[(i + 1) % n]);
            let steps = ((b[0] - a[0]).abs().max((b[1] - a[1]).abs()).ceil() as usize).max(1);
            for k in 0..=steps {
                let t = k as f64 / steps as f64;
                let (x, y) = (a[0] + t * (b[0] - a[0]), a[1] + t * (b[1] - a[1]));
                if x >= 0.0 && y >= 0.0 {
                    canvas.set(x as usize, y as usize, colour);
                }
            }
        }
    }
}

/// Marca uma âncora com uma cruz — sem isto não se distingue "a curva está errada" de "as
/// âncoras estão no sítio errado", que foi exatamente a dúvida do Twist.
pub fn mark(canvas: &mut Canvas, p: [f64; 2], colour: [u8; 3]) {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let (cx, cy) = (p[0].max(0.0) as usize, p[1].max(0.0) as usize);
    for d in 0..3 {
        canvas.set(cx.saturating_sub(d), cy, colour);
        canvas.set(cx + d, cy, colour);
        canvas.set(cx, cy.saturating_sub(d), colour);
        canvas.set(cx, cy + d, colour);
    }
}

// ── PNG sem dependências ────────────────────────────────────────────────────────────────────

fn crc32(data: &[u8]) -> u32 {
    let mut table = [0u32; 256];
    for (i, e) in table.iter_mut().enumerate() {
        let mut c = i as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 {
                0xEDB8_8320 ^ (c >> 1)
            } else {
                c >> 1
            };
        }
        *e = c;
    }
    let mut c = 0xFFFF_FFFFu32;
    for b in data {
        c = table[((c ^ u32::from(*b)) & 0xFF) as usize] ^ (c >> 8);
    }
    c ^ 0xFFFF_FFFF
}

fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], body: &[u8]) {
    out.extend_from_slice(&(body.len() as u32).to_be_bytes());
    let mut full = kind.to_vec();
    full.extend_from_slice(body);
    out.extend_from_slice(&full);
    out.extend_from_slice(&crc32(&full).to_be_bytes());
}

/// Escreve o canvas como PNG RGB de 8 bits. O `zlib` usa **blocos armazenados** (deflate sem
/// compressão), que o formato admite: troca tamanho de ficheiro por zero dependências, e uma
/// sonda de diagnóstico não precisa de ser pequena.
pub fn write_png(path: &std::path::Path, c: &Canvas) -> std::io::Result<()> {
    let mut raw = Vec::with_capacity(c.h * (1 + c.w * 3));
    for y in 0..c.h {
        raw.push(0); // filtro None
        for x in 0..c.w {
            raw.extend_from_slice(&c.px[y * c.w + x]);
        }
    }
    // zlib: cabeçalho + blocos armazenados de até 65535 + adler32.
    let mut z = vec![0x78, 0x01];
    for (i, block) in raw.chunks(65_535).enumerate() {
        let last = u8::from((i + 1) * 65_535 >= raw.len());
        z.push(last);
        z.extend_from_slice(&(block.len() as u16).to_le_bytes());
        z.extend_from_slice(&(!(block.len() as u16)).to_le_bytes());
        z.extend_from_slice(block);
    }
    let (mut a, mut b) = (1u32, 0u32);
    for byte in &raw {
        a = (a + u32::from(*byte)) % 65_521;
        b = (b + a) % 65_521;
    }
    z.extend_from_slice(&((b << 16) | a).to_be_bytes());

    let mut png = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&(c.w as u32).to_be_bytes());
    ihdr.extend_from_slice(&(c.h as u32).to_be_bytes());
    ihdr.extend_from_slice(&[8, 2, 0, 0, 0]); // 8 bits, truecolor
    chunk(&mut png, b"IHDR", &ihdr);
    chunk(&mut png, b"IDAT", &z);
    chunk(&mut png, b"IEND", &[]);
    std::fs::write(path, png)
}

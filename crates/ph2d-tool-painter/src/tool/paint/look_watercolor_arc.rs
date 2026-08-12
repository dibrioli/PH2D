//! RENDER-AND-LOOK do arco palido da aquarela.
//!
//! # Por que uma sonda que DESENHA, e nao mais um numero
//!
//! Esta sessao produziu SEIS fixtures escalares seguidas cujo numero descrevia outra coisa que
//! nao o defeito reportado (o `docs/Painter/32` §6 as lista). A ultima delas — a janela da
//! concavidade — mede "clareamento acima da mediana local" num retangulo que **tambem contem a
//! transicao tinta/papel do vao da cruz**, entao ela reporta um contraste grande em Dilution 0,00
//! (onde a tinta e densa) e pequeno em 0,45 (onde e palida) — exatamente o INVERSO do que o Enio
//! ve. O numero nao esta errado; ele esta a responder outra pergunta.
//!
//! O repo ja tem o precedente para isto: `push_look_probe` (shell) e `fx_look` (vec-scene)
//! **desenham** e deixam o olho decidir. O escritor de PNG abaixo e o mesmo de
//! `ph2d-vec-scene/tests/look/mod.rs` — blocos deflate ARMAZENADOS, **zero dependencia**: uma
//! sonda de diagnostico nao precisa de ficheiro pequeno, e uma dep nova por causa dela seria o
//! preco errado.
//!
//! # Rodar
//!
//! ```text
//! env PH2D_WC_LOOK_DIR=/tmp/wc cargo test -p ph2d-tool-painter --release \
//!     probe_watercolor_arc -- --ignored --nocapture
//! ```
//!
//! Sem a variavel a sonda **nao escreve nada** e diz porque — um probe que escreve em sitio
//! escolhido por ele proprio e um probe que suja a arvore de quem so correu a suite.

use super::measure_watercolor_water_edge::wash_over_dry;

const SIDE: usize = 256;

/// Uma tela RGB de 8 bits. Nao ha alfa de proposito: o que se julga aqui e o que a tela MOSTRA.
struct Canvas {
    w: usize,
    h: usize,
    px: Vec<[u8; 3]>,
}

impl Canvas {
    fn new(w: usize, h: usize) -> Self {
        Self {
            w,
            h,
            px: vec![[0, 0, 0]; w * h],
        }
    }

    fn set(&mut self, x: usize, y: usize, c: [u8; 3]) {
        if x < self.w && y < self.h {
            self.px[y * self.w + x] = c;
        }
    }
}

/// O canvas do produto (RGBA premultiplicado sobre papel branco) como RGB opaco.
///
/// ⚠️ O `canvas_rgba` do Painter e **straight alpha sobre papel**: o papel ja esta la (o fixture
/// semeia 255). Compor de novo contra branco escureceria a tinta duas vezes, entao aqui e uma
/// copia de canal, nao um `over`.
fn as_rgb(px: &[u8], x0: usize, y0: usize, w: usize, h: usize, zoom: usize) -> Canvas {
    let mut c = Canvas::new(w * zoom, h * zoom);
    for row in 0..h {
        for col in 0..w {
            let i = ((y0 + row) * SIDE + (x0 + col)) * 4;
            let rgb = [px[i], px[i + 1], px[i + 2]];
            for dy in 0..zoom {
                for dx in 0..zoom {
                    c.set(col * zoom + dx, row * zoom + dy, rgb);
                }
            }
        }
    }
    c
}

// ── PNG sem dependencias (porte verbatim de ph2d-vec-scene/tests/look/mod.rs) ───────────────

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

fn write_png(path: &std::path::Path, c: &Canvas) -> std::io::Result<()> {
    let mut raw = Vec::with_capacity(c.h * (1 + c.w * 3));
    for y in 0..c.h {
        raw.push(0); // filtro None
        for x in 0..c.w {
            raw.extend_from_slice(&c.px[y * c.w + x]);
        }
    }
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

/// Desenha a cruz `wash_over_dry` e escreve o que a tela mostra.
///
/// ⚠️ **A cena e a MESMA das sondas escalares** (mesmo pincel real: `Falloff::Watercolor`,
/// raio 72, o que o modo aquarela de facto produz — o `Falloff::Constant` que as quatro
/// primeiras sondas usavam e um pincel que este modo **nao consegue** montar). O que muda e o
/// instrumento: aqui o oraculo e o olho, e o olho nao confunde "o vao da cruz e papel" com
/// "ha um arco palido acompanhando a concavidade".
#[test]
#[ignore = "sonda de diagnostico: escreve PNG, roda com PH2D_WC_LOOK_DIR"]
fn probe_watercolor_arc() {
    let Ok(dir) = std::env::var("PH2D_WC_LOOK_DIR") else {
        eprintln!(
            "probe_watercolor_arc: defina PH2D_WC_LOOK_DIR=<dir> para escrever os PNG.\n\
             Sem ela a sonda nao escreve nada — um probe nao escolhe sozinho onde sujar."
        );
        return;
    };
    std::fs::create_dir_all(&dir).expect("criar o diretorio de saida");

    for (tag, dilution) in [("d000", 0.00f32), ("d045", 0.45)] {
        for (edges, smooth) in [("smooth", true), ("hard", false)] {
            let px = wash_over_dry(dilution, smooth);

            // A cena inteira: e ela que diz se o arco e local ou se a lavagem toda mudou.
            let full = as_rgb(&px, 0, 0, SIDE, SIDE, 2);
            let p = format!("{dir}/{tag}_{edges}_full.png");
            write_png(std::path::Path::new(&p), &full).expect("escrever o PNG");

            // A concavidade INFERIOR-DIREITA da cruz, 4x. As quatro quinas sao equivalentes por
            // construcao (a faixa e horizontal, a vertical cruza no meio), entao uma basta.
            let crop = as_rgb(&px, 128, 100, 96, 96, 4);
            let p = format!("{dir}/{tag}_{edges}_quina.png");
            write_png(std::path::Path::new(&p), &crop).expect("escrever o PNG");
        }
        eprintln!("probe_watercolor_arc: Dilution {dilution:.2} escrito em {dir}");
    }
}

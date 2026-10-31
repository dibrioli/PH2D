//! A.1 — a prancha do Óleo Real na vista e o custo (doc `docs/Painter/49_realshader/A1_oleo_na_vista.md`).
//!
//! Sem argumentos grava `docs/Painter/49_realshader/capturas/a1_prancha.png`: 2×2 do mesmo recorte 64×64 da faca —
//! [hoje 100 % | nova 100 %] / [hoje 400 % (bilinear, a vista de hoje) | nova 400 %].
//! `--mede`: ms de CPU (UM fio) por megapíxel da vista nova a zoom 1, 2, 4 e do composite de hoje à resolução do
//! documento, intercalados, aquecidos, mínimo e mediana; e o quadro 1920×1080 / 3840×2160 a partir deles.

use ph2d_wet_paint::oleo_real::OleoReal;
use ph2d_wet_paint::oleo_vista::prova::{VISUAL, bilinear, camadas, campo, faca, hoje};
use ph2d_wet_paint::oleo_vista::{PreparoVista, Vista};
use std::time::Instant;

const RECORTE: (usize, usize, usize, usize) = (16, 16, 64, 64);

fn vista(s: f64) -> Vista {
    let (x0, y0, w, h) = RECORTE;
    Vista {
        x0: x0 as f64,
        y0: y0 as f64,
        w: w * s as usize,
        h: h * s as usize,
        s,
    }
}

fn crc32(d: &[u8]) -> u32 {
    let mut c = !0u32;
    for &b in d {
        c ^= u32::from(b);
        for _ in 0..8 {
            c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
        }
    }
    !c
}

/// PNG RGBA sem compressão (blocos «stored» do deflate).
fn png(rgba: &[u8], w: usize, h: usize) -> Vec<u8> {
    let mut raw = Vec::with_capacity((w * 4 + 1) * h);
    for y in 0..h {
        raw.push(0);
        raw.extend_from_slice(&rgba[y * w * 4..(y + 1) * w * 4]);
    }
    let mut z = vec![0x78, 0x01];
    let blocos: Vec<&[u8]> = raw.chunks(65_535).collect();
    for (k, b) in blocos.iter().enumerate() {
        z.push(u8::from(k + 1 == blocos.len()));
        let n = b.len() as u16;
        z.extend_from_slice(&n.to_le_bytes());
        z.extend_from_slice(&(!n).to_le_bytes());
        z.extend_from_slice(b);
    }
    let (mut a, mut bb) = (1u32, 0u32);
    for &x in &raw {
        a = (a + u32::from(x)) % 65_521;
        bb = (bb + a) % 65_521;
    }
    z.extend_from_slice(&((bb << 16) | a).to_be_bytes());
    let mut out = b"\x89PNG\r\n\x1a\n".to_vec();
    let mut chunk = |t: &[u8], d: &[u8]| {
        out.extend_from_slice(&(d.len() as u32).to_be_bytes());
        let mut td = t.to_vec();
        td.extend_from_slice(d);
        out.extend_from_slice(&td);
        out.extend_from_slice(&crc32(&td).to_be_bytes());
    };
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&(w as u32).to_be_bytes());
    ihdr.extend_from_slice(&(h as u32).to_be_bytes());
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]);
    chunk(b"IHDR", &ihdr);
    chunk(b"IDAT", &z);
    chunk(b"IEND", &[]);
    out
}

/// Sobre o branco (a tinta tem alfa).
fn sobre_branco(rgba: &[u8]) -> Vec<u8> {
    rgba.chunks(4)
        .flat_map(|p| {
            let a = f64::from(p[3]) / 255.0;
            let m = |c: u8| (f64::from(c) * a + 255.0 * (1.0 - a)).round() as u8;
            [m(p[0]), m(p[1]), m(p[2]), 255]
        })
        .collect()
}

fn nearest(src: &[u8], w: usize, h: usize, s: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(w * h * s * s * 4);
    for y in 0..h * s {
        for x in 0..w * s {
            let i = (x / s + (y / s) * w) * 4;
            out.extend_from_slice(&src[i..i + 4]);
        }
    }
    out
}

fn prancha() {
    let e = faca();
    let o = OleoReal::default();
    let c = campo(&e, &o);
    let (_, _, w, h) = RECORTE;
    let cam = camadas(&e);
    let hoje1 = hoje(&e, &o, &c, RECORTE);
    let nova1 = PreparoVista::novo(None, &cam, VISUAL, &o, &c, vista(1.0)).render();
    let hoje4 = bilinear(&hoje1, w, h, 4);
    let nova4 = PreparoVista::novo(None, &cam, VISUAL, &o, &c, vista(4.0)).render();
    // 100 % mostra-se ×4 em nearest (cada pixel do documento um bloco): a mesma área que os 400 %.
    let quadros = [
        nearest(&hoje1, w, h, 4),
        nearest(&nova1, w, h, 4),
        hoje4,
        nova4,
    ];
    let (q, gap) = (w * 4, 8);
    let lado = 2 * q + gap;
    let mut img = vec![40u8; lado * lado * 4];
    for (k, quadro) in quadros.iter().enumerate() {
        let quadro = sobre_branco(quadro);
        let (ox, oy) = ((k % 2) * (q + gap), (k / 2) * (q + gap));
        for y in 0..q {
            let d = ((oy + y) * lado + ox) * 4;
            img[d..d + q * 4].copy_from_slice(&quadro[y * q * 4..(y + 1) * q * 4]);
        }
    }
    let raiz = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/Painter/49_realshader/capturas");
    std::fs::create_dir_all(raiz).expect("pasta");
    let p = format!("{raiz}/a1_prancha.png");
    std::fs::write(&p, png(&img, lado, lado)).expect("png");
    println!("gravado {p} ({lado}×{lado})");
}

fn mede() {
    let e = faca();
    let o = OleoReal::default();
    let c = campo(&e, &o);
    let cam = camadas(&e);
    let (_, _, w, h) = RECORTE;
    // Variantes: 0 = hoje (doc), 1/2/3 = nova a zoom 1/2/4 (preparo + render). ns por píxel de SAÍDA.
    let corre = |k: usize| -> (f64, usize) {
        let t = Instant::now();
        let n = if k == 0 {
            std::hint::black_box(hoje(&e, &o, &c, RECORTE));
            w * h
        } else {
            let s = [1.0, 2.0, 4.0][k - 1];
            let v = vista(s);
            std::hint::black_box(PreparoVista::novo(None, &cam, VISUAL, &o, &c, v).render());
            v.w * v.h
        };
        (t.elapsed().as_secs_f64(), n)
    };
    for k in 0..4 {
        corre(k);
    }
    let mut amostras = vec![Vec::new(); 4];
    for bloco in 0..15 {
        for j in 0..4 {
            let k = (j + bloco) % 4;
            let (t, n) = corre(k);
            amostras[k].push(t * 1e9 / n as f64);
        }
    }
    let nomes = ["hoje (doc, 1×)", "nova s=1", "nova s=2", "nova s=4"];
    println!("um fio; ns/píxel de saída (mín · mediana) → ms por quadro 1920×1080 · 3840×2160");
    for (k, a) in amostras.iter_mut().enumerate() {
        a.sort_by(f64::total_cmp);
        let (mn, md) = (a[0], a[a.len() / 2]);
        let q = |px: f64| px * mn / 1e6;
        println!(
            "{:<16} {mn:8.1} · {md:8.1}   → {:8.1} ms · {:8.1} ms",
            nomes[k],
            q(1920.0 * 1080.0),
            q(3840.0 * 2160.0)
        );
    }
}

fn main() {
    if std::env::args().any(|a| a == "--mede") {
        mede();
    } else {
        prancha();
    }
}

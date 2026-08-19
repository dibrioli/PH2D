//! Rasterizador de software — a metade do entregável da W0 que o **Enio** julga.
//!
//! ⚠️ **Sombreamento PLANO (flat), de propósito.** A pergunta desta imagem é *"a quina saiu viva?"*,
//! e normal interpolada por vértice **esconde exatamente isso**: ela alisa a transição e faz um
//! canto arredondado parecer com um canto vivo. Com normal por triângulo, uma quina de verdade
//! aparece como duas faces de brilho distinto encostadas, e um canto arredondado aparece como uma
//! escadinha de facetas. A imagem tem de poder reprovar o motor.
//!
//! ⚠️ Cor literal aqui **não** fere o HR-15: aquilo governa a **UI** do app (tokens/i18n), e isto é
//! um PNG de diagnóstico de um spike que morre com ele — não há widget, tema nem usuário.

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

pub const BG: [u8; 4] = [24, 26, 30, 255];

/// Câmera ortográfica: olha da direção `from` para a origem.
pub struct View {
    pub width: usize,
    pub height: usize,
    /// Quantos pixels vale uma unidade de mundo, como fração da meia-altura.
    pub scale: f64,
    pub from: [f64; 3],
    /// O ponto do mundo que vai ao centro do quadro — é o que permite aproximar de uma aresta.
    pub target: [f64; 3],
}

impl Default for View {
    fn default() -> Self {
        Self {
            width: 560,
            height: 560,
            scale: 1.55,
            // Três-quartos, ligeiramente por cima: é o ângulo em que uma aresta viva e um filete
            // se distinguem sem ambiguidade.
            from: [1.0, 0.75, 1.1],
            target: [0.0, 0.0, 0.0],
        }
    }
}

fn norm(v: [f64; 3]) -> [f64; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    [v[0] / l, v[1] / l, v[2] / l]
}
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Rasteriza a malha e devolve RGBA8.
pub fn render(view: &View, verts: &[[f32; 3]], tris: &[[u32; 3]]) -> Vec<u8> {
    let (w, h) = (view.width, view.height);
    let mut color = vec![0u8; w * h * 4];
    for px in color.chunks_exact_mut(4) {
        px.copy_from_slice(&BG);
    }
    let mut depth = vec![f64::NEG_INFINITY; w * h];

    let z_axis = norm(view.from);
    let up = [0.0, 1.0, 0.0];
    let x_axis = norm(cross(up, z_axis));
    let y_axis = cross(z_axis, x_axis);

    let half = (h.min(w) as f64) * 0.5;
    let to_screen = |p: [f64; 3]| -> [f64; 3] {
        let p = [
            p[0] - view.target[0],
            p[1] - view.target[1],
            p[2] - view.target[2],
        ];
        let vx = dot(p, x_axis) * view.scale;
        let vy = dot(p, y_axis) * view.scale;
        let vz = dot(p, z_axis);
        [w as f64 * 0.5 + vx * half, h as f64 * 0.5 - vy * half, vz]
    };

    // Luz principal + preenchimento, ambas em espaço de mundo.
    let key = norm([0.6, 0.8, 0.5]);
    let fill = norm([-0.7, 0.2, 0.3]);

    for t in tris {
        let p: Vec<[f64; 3]> = t
            .iter()
            .map(|i| {
                let v = verts[*i as usize];
                [v[0] as f64, v[1] as f64, v[2] as f64]
            })
            .collect();

        let n = {
            let e1 = [p[1][0] - p[0][0], p[1][1] - p[0][1], p[1][2] - p[0][2]];
            let e2 = [p[2][0] - p[0][0], p[2][1] - p[0][1], p[2][2] - p[0][2]];
            let c = cross(e1, e2);
            let l = (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt();
            if l <= 0.0 {
                continue; // triângulo degenerado: não tem normal, não pinta
            }
            [c[0] / l, c[1] / l, c[2] / l]
        };

        // Frente ao observador? (a malha é estanque, então as costas não interessam)
        let facing = dot(n, z_axis);
        let n = if facing < 0.0 {
            [-n[0], -n[1], -n[2]]
        } else {
            n
        };

        let lambert = dot(n, key).max(0.0) * 0.78 + dot(n, fill).max(0.0) * 0.22;
        let ambient = 0.16 + 0.10 * (n[1] * 0.5 + 0.5);
        let lit = (lambert + ambient).clamp(0.0, 1.0);
        // Rampa levemente quente, para a faceta ler melhor que um cinza puro.
        let rgb = [
            (lit.powf(0.85) * 236.0) as u8,
            (lit.powf(0.92) * 226.0) as u8,
            (lit.powf(1.0) * 212.0) as u8,
        ];

        let s: Vec<[f64; 3]> = p.iter().map(|q| to_screen(*q)).collect();

        let min_x = s
            .iter()
            .map(|v| v[0])
            .fold(f64::INFINITY, f64::min)
            .floor()
            .max(0.0) as usize;
        let max_x = (s
            .iter()
            .map(|v| v[0])
            .fold(f64::NEG_INFINITY, f64::max)
            .ceil() as isize)
            .clamp(0, w as isize) as usize;
        let min_y = s
            .iter()
            .map(|v| v[1])
            .fold(f64::INFINITY, f64::min)
            .floor()
            .max(0.0) as usize;
        let max_y = (s
            .iter()
            .map(|v| v[1])
            .fold(f64::NEG_INFINITY, f64::max)
            .ceil() as isize)
            .clamp(0, h as isize) as usize;

        let area = edge(&s[0], &s[1], &s[2]);
        if area.abs() < 1e-12 {
            continue;
        }

        for y in min_y..max_y {
            for x in min_x..max_x {
                // Centro do pixel — não o canto. (A casa já pagou esse meio-pixel noutro módulo.)
                let pt = [x as f64 + 0.5, y as f64 + 0.5, 0.0];
                let w0 = edge(&s[1], &s[2], &pt) / area;
                let w1 = edge(&s[2], &s[0], &pt) / area;
                let w2 = edge(&s[0], &s[1], &pt) / area;
                if w0 < 0.0 || w1 < 0.0 || w2 < 0.0 {
                    continue;
                }
                let z = w0 * s[0][2] + w1 * s[1][2] + w2 * s[2][2];
                let idx = y * w + x;
                if z > depth[idx] {
                    depth[idx] = z;
                    let o = idx * 4;
                    color[o] = rgb[0];
                    color[o + 1] = rgb[1];
                    color[o + 2] = rgb[2];
                    color[o + 3] = 255;
                }
            }
        }
    }
    color
}

fn edge(a: &[f64; 3], b: &[f64; 3], c: &[f64; 3]) -> f64 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}

pub fn write_png(path: &Path, width: usize, height: usize, rgba: &[u8]) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let file = File::create(path)?;
    let mut enc = png::Encoder::new(BufWriter::new(file), width as u32, height as u32);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    let mut writer = enc
        .write_header()
        .map_err(|e| std::io::Error::other(e.to_string()))?;
    writer
        .write_image_data(rgba)
        .map_err(|e| std::io::Error::other(e.to_string()))?;
    Ok(())
}

/// Junta imagens lado a lado, com um filete escuro entre elas.
pub fn contact_sheet(panels: &[(usize, usize, Vec<u8>)], gap: usize) -> (usize, usize, Vec<u8>) {
    let h = panels.iter().map(|p| p.1).max().unwrap_or(0);
    let w: usize = panels.iter().map(|p| p.0).sum::<usize>() + gap * panels.len().saturating_sub(1);
    let mut out = vec![0u8; w * h * 4];
    for px in out.chunks_exact_mut(4) {
        px.copy_from_slice(&[12, 13, 15, 255]);
    }
    let mut x0 = 0usize;
    for (pw, ph, data) in panels {
        for y in 0..*ph {
            let src = y * pw * 4;
            let dst = (y * w + x0) * 4;
            out[dst..dst + pw * 4].copy_from_slice(&data[src..src + pw * 4]);
        }
        x0 += pw + gap;
    }
    (w, h, out)
}

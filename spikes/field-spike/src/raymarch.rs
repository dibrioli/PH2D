//! **O traçador de campo** — desenha a superfície **sem malha nenhuma no meio**.
//!
//! # Por que ele existe (achado do smoke do Enio, 2026-08-19)
//!
//! O veredito dele sobre a primeira imagem foi: *"quinas externas horríveis e completamente
//! inúteis para arte; quinas internas ruins mas promissoras"*. Isso levanta uma pergunta que a W0
//! ainda **não tinha separado**: o defeito está na **geometria** (o campo, os operadores) ou na
//! **malha** (a extração)? São causas diferentes e curas diferentes, e uma imagem tirada da malha
//! não consegue distingui-las — ela mostra as duas somadas.
//!
//! Este módulo responde: ele marcha raios contra o **campo**, ponto a ponto. O que sai aqui é o
//! **teto** — a forma que o modelo de facto tem, livre de qualquer erro de extração.
//!
//! - Se o traçado sair limpo e a malha não ⇒ a geometria está certa e **a malha é a culpada**.
//! - Se o traçado sair sujo ⇒ o problema é do campo, e trocar de extrator não salvaria nada.
//!
//! ⚠️ **Ele usa a MESMA câmera e a MESMA função de sombreamento** que o rasterizador de malha
//! ([`crate::render::shade`]). Duas luzes diferentes tornariam a comparação inútil.
//!
//! # O passo seguro, e por que não é `d`
//!
//! A marcha de esferas só pode avançar `d` quando o campo é uma distância honesta (‖∇f‖ = 1).
//! A §3 de `01_resultados_spike.md` **mediu** ‖∇f‖ chegando a **√2** no operador exato — logo um
//! passo de `d` **atravessaria** a superfície, e o furo apareceria como pixel de fundo no meio da
//! peça. O fator abaixo é `1/√2`, que é o recíproco da constante medida: não é um número escolhido
//! a olho, é o que a medição obriga.

use crate::Engine;
use crate::render::{BG, View, basis, shade};
use fidget::context::Tree;

/// `1/√2` — o recíproco do maior ‖∇f‖ medido (`01_resultados_spike.md` §3).
const SAFE_STEP: f32 = 0.7071;
const EPS: f32 = 2.0e-4;
const T_MAX: f32 = 8.0;
const MAX_STEPS: usize = 400;

pub struct TraceStats {
    pub hits: usize,
    pub steps: usize,
    pub evals: usize,
}

/// Marcha o campo e devolve RGBA8 + as estatísticas do custo.
pub fn trace(view: &View, tree: &Tree) -> (Vec<u8>, TraceStats) {
    let (w, h) = (view.width, view.height);
    let (x_axis, y_axis, z_axis) = basis(view.from);
    let half = (h.min(w) as f64) * 0.5;

    let shape = Engine::from(tree.clone());
    let tape = shape.float_slice_tape(Default::default());
    let mut eval = Engine::new_float_slice_eval();

    // Origem de cada raio: o plano ortográfico atrás da peça; direção comum `-z`.
    let n = w * h;
    let mut ox = vec![0f32; n];
    let mut oy = vec![0f32; n];
    let mut oz = vec![0f32; n];
    for py in 0..h {
        for px in 0..w {
            let sx = (px as f64 + 0.5 - w as f64 * 0.5) / half / view.scale;
            let sy = -(py as f64 + 0.5 - h as f64 * 0.5) / half / view.scale;
            let start = 4.0;
            let i = py * w + px;
            ox[i] = (view.target[0] + x_axis[0] * sx + y_axis[0] * sy + z_axis[0] * start) as f32;
            oy[i] = (view.target[1] + x_axis[1] * sx + y_axis[1] * sy + z_axis[1] * start) as f32;
            oz[i] = (view.target[2] + x_axis[2] * sx + y_axis[2] * sy + z_axis[2] * start) as f32;
        }
    }
    let d = [-z_axis[0] as f32, -z_axis[1] as f32, -z_axis[2] as f32];

    let mut t = vec![0f32; n];
    let mut hit = vec![false; n];
    let mut alive: Vec<u32> = (0..n as u32).collect();
    let (mut steps, mut evals) = (0usize, 0usize);

    let (mut xs, mut ys, mut zs) = (Vec::new(), Vec::new(), Vec::new());
    for _ in 0..MAX_STEPS {
        if alive.is_empty() {
            break;
        }
        steps += 1;
        xs.clear();
        ys.clear();
        zs.clear();
        for &i in &alive {
            let i = i as usize;
            xs.push(ox[i] + d[0] * t[i]);
            ys.push(oy[i] + d[1] * t[i]);
            zs.push(oz[i] + d[2] * t[i]);
        }
        let out = eval.eval(&tape, &xs, &ys, &zs).expect("avaliação em lote");
        evals += out.len();

        let mut next = Vec::with_capacity(alive.len());
        for (k, &i) in alive.iter().enumerate() {
            let iu = i as usize;
            let dist = out[k];
            if dist < EPS {
                hit[iu] = true;
                continue;
            }
            t[iu] += dist * SAFE_STEP;
            if t[iu] < T_MAX {
                next.push(i);
            }
        }
        alive = next;
    }

    // Normais por diferença central — 6 avaliações por pixel atingido, em lote.
    // ⚠️ A `fidget` sabe devolver o gradiente EXATO (`new_grad_slice_eval`); aqui a diferença
    // central basta e evita a semeadura de derivada, que é detalhe de API sem valor para a
    // pergunta desta imagem. No módulo de produção usa-se o exato.
    let hits: Vec<usize> = (0..n).filter(|i| hit[*i]).collect();
    const HE: f32 = 1.0e-4;
    let mut gx = Vec::with_capacity(hits.len() * 6);
    let mut gy = Vec::with_capacity(hits.len() * 6);
    let mut gz = Vec::with_capacity(hits.len() * 6);
    for &i in &hits {
        let (px, py, pz) = (
            ox[i] + d[0] * t[i],
            oy[i] + d[1] * t[i],
            oz[i] + d[2] * t[i],
        );
        for (ddx, ddy, ddz) in [
            (HE, 0.0, 0.0),
            (-HE, 0.0, 0.0),
            (0.0, HE, 0.0),
            (0.0, -HE, 0.0),
            (0.0, 0.0, HE),
            (0.0, 0.0, -HE),
        ] {
            gx.push(px + ddx);
            gy.push(py + ddy);
            gz.push(pz + ddz);
        }
    }
    let grads = if hits.is_empty() {
        Vec::new()
    } else {
        eval.eval(&tape, &gx, &gy, &gz)
            .expect("avaliação das normais")
            .to_vec()
    };
    evals += grads.len();

    let mut color = vec![0u8; n * 4];
    for px in color.chunks_exact_mut(4) {
        px.copy_from_slice(&BG);
    }
    for (k, &i) in hits.iter().enumerate() {
        let b = k * 6;
        let nx = (grads[b] - grads[b + 1]) as f64;
        let ny = (grads[b + 2] - grads[b + 3]) as f64;
        let nz = (grads[b + 4] - grads[b + 5]) as f64;
        let len = (nx * nx + ny * ny + nz * nz).sqrt();
        if len <= 0.0 {
            continue;
        }
        let rgb = shade([nx / len, ny / len, nz / len], z_axis);
        let o = i * 4;
        color[o] = rgb[0];
        color[o + 1] = rgb[1];
        color[o + 2] = rgb[2];
        color[o + 3] = 255;
    }

    (
        color,
        TraceStats {
            hits: hits.len(),
            steps,
            evals,
        },
    )
}

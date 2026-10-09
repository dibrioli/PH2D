//! ⭐⭐ **O que vem do MUNDO, no espaço do objecto Motion** — escolha 4.2 do dono (09/10): *«o rato,
//! os colisores, a câmara e as poses de objectos são convertidos sozinhos para o espaço do objecto —
//! nada a configurar»*. Doc ObjectsAndComponents 01 §14, item 6.
//!
//! O grafo pensa no espaço do OBJECTO e a saída leva a afim do dono ([`super::origem`]); a entrada
//! leva a INVERSA. Uma porta só: o editor publica em mundo, e aqui cada externo de mundo é
//! republicado, convertido, sob [`ph2d_nodegraph::external::in_space`] para cada dono fora da
//! origem; o cozido entrega a cópia ao nó desse dono (`Cook::set_spaces`).
//!
//! | externo | espécie | conversão |
//! |---|---|---|
//! | `$cursor`, `$at:<nome>`, `$curve:<nome>` | pontos | `P ← A⁻¹·P` |
//! | `$camera` | vista | `P ← A⁻¹·P`, `size ← size / s`, `zoom ← zoom · s` |
//! | `$pose:<nome>` | pose | `M = L⁻¹·R(rotation)·diag(size)` ⇒ `rotation`, `size` de `M` |
//!
//! `s = √|det L|` — o comprimento equivalente de uma escala não uniforme. A aparência (o nome cru),
//! os colisores que ela traz e as tabelas não dependem do espaço: não têm cópia, e o nó lê-os crus.
//!
//! ⭐ **Uma conversão serve os dois caminhos:** todo leitor de externo é um nó da CPU
//! (`ctx.external`); a placa só lê os `geometry_id` da aparência, e cai para a CPU quando o grafo
//! lê o mundo (`motion_bridge_gpu`).

use std::collections::BTreeMap;

use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::external::{self, CAMERA, CURSOR, RESERVED_PREFIX};

use super::origem::{linear, ponto};
use super::{Afim, IDENTIDADE};
use crate::motion_state::MotionState;

/// Como um externo de mundo muda de espaço.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Especie {
    Pontos,
    Vista,
    Pose,
}

/// A espécie do externo `nome`, ou `None` se ele não depende do espaço.
#[must_use]
pub fn especie(nome: &str) -> Option<Especie> {
    if nome == CURSOR {
        return Some(Especie::Pontos);
    }
    if nome == CAMERA {
        return Some(Especie::Vista);
    }
    let resto = nome.strip_prefix(RESERVED_PREFIX)?;
    if resto.starts_with("at:") || resto.starts_with("curve:") {
        Some(Especie::Pontos)
    } else if resto.starts_with("pose:") {
        Some(Especie::Pose)
    } else {
        None
    }
}

/// A inversa de `a`, ou `None` se ela não existe (escala zero: o dono não gera nada visível).
#[must_use]
pub fn inversa(a: &Afim) -> Option<Afim> {
    let det = a[0] * a[3] - a[2] * a[1];
    if !det.is_finite() || det.abs() <= f32::EPSILON {
        return None;
    }
    let (i0, i1, i2, i3) = (a[3] / det, -a[1] / det, -a[2] / det, a[0] / det);
    let t = [-(i0 * a[4] + i2 * a[5]), -(i1 * a[4] + i3 * a[5])];
    Some([i0, i1, i2, i3, t[0], t[1]])
}

/// O externo `s`, da espécie `e`, levado pela afim `a` (já a inversa do dono).
#[must_use]
pub fn converte(e: Especie, s: &Stream, a: &Afim) -> Stream {
    let escala = (a[0] * a[3] - a[2] * a[1]).abs().sqrt();
    let mut out = s.clone();
    match e {
        Especie::Pontos => mapeia_vec2(&mut out, s, "P", |p| ponto(a, p)),
        Especie::Vista => {
            mapeia_vec2(&mut out, s, "P", |p| ponto(a, p));
            mapeia_vec2(&mut out, s, "size", |v| [v[0] * escala, v[1] * escala]);
            if let Some(Column::Scalar(z)) = s.get(ph2d_node_source_camera::ZOOM_COLUMN) {
                let z = z.iter().map(|z| z / escala).collect();
                out.set(ph2d_node_source_camera::ZOOM_COLUMN, Column::Scalar(z));
            }
        }
        Especie::Pose => {
            let (Some(Column::Scalar(rot)), Some(Column::Vec2(size))) =
                (s.get("rotation"), s.get("size"))
            else {
                return out;
            };
            let (rots, sizes): (Vec<f32>, Vec<[f32; 2]>) = rot
                .iter()
                .zip(size)
                .map(|(&r, &sz)| pose_em(a, r, sz))
                .unzip();
            out.set("rotation", Column::Scalar(rots));
            out.set("size", Column::Vec2(sizes));
        }
    }
    out
}

/// `M = L·R(r)·diag(size)`, decomposta em rotação (a da 1.ª coluna) e escala (o comprimento dela, e
/// `det M / sx` — o sinal de um espelho fica na 2.ª).
fn pose_em(a: &Afim, r: f32, size: [f32; 2]) -> (f32, [f32; 2]) {
    let (s, c) = r.sin_cos();
    let c0 = linear(a, [c * size[0], s * size[0]]);
    let c1 = linear(a, [-s * size[1], c * size[1]]);
    let sx = c0[0].hypot(c0[1]);
    if sx <= f32::EPSILON {
        return (r, [0.0, 0.0]);
    }
    let det = c0[0] * c1[1] - c1[0] * c0[1];
    (c0[1].atan2(c0[0]), [sx, det / sx])
}

fn mapeia_vec2(out: &mut Stream, s: &Stream, coluna: &str, f: impl Fn([f32; 2]) -> [f32; 2]) {
    if let Some(Column::Vec2(v)) = s.get(coluna) {
        out.set(coluna, Column::Vec2(v.iter().map(|&p| f(p)).collect()));
    }
}

/// ⭐ **Publica as cópias por espaço** — no fim da porta das membranas
/// (`motion_externals::publish_all`), depois de todo externo do quadro estar publicado e antes do
/// cozido. Sem dono fora da origem: nenhuma cópia, mapa vazio, o caminho de sempre.
pub fn publica(motion: &mut MotionState) {
    let cook = &mut motion.pump.cook;
    cook.clear_spaced_externals();
    let deslocados: BTreeMap<u64, Afim> = motion
        .objeto
        .donos
        .iter()
        .filter(|(_, d)| d.afim != IDENTIDADE)
        .filter_map(|(&sid, d)| Some((sid, inversa(&d.afim)?)))
        .collect();
    let espacos: BTreeMap<u32, u64> = motion
        .doc
        .owners
        .iter()
        .filter(|(_, o)| deslocados.contains_key(o))
        .map(|(n, &o)| (n.0, o))
        .collect();
    cook.set_spaces(espacos);
    if deslocados.is_empty() {
        return;
    }
    let mundo: Vec<(String, Especie, Stream)> = cook
        .externals()
        .iter()
        .filter_map(|(k, e)| Some((k.clone(), especie(k)?, e.value.clone())))
        .collect();
    for (&sid, inv) in &deslocados {
        for (nome, e, s) in &mundo {
            cook.set_external(external::in_space(nome, sid), converte(*e, s, inv));
        }
    }
}

#[cfg(test)]
#[path = "motion_objeto_entradas_tests.rs"]
mod tests;

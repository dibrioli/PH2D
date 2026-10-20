//! **A CÓPIA de cada dab** — a lane do Wet é por (cópia, ponta) — filho de [`super::dab_route`] por LOC.
//!
//! Um ponto do caminho deixa as suas marcas JUNTAS e com o MESMO `arc_len` (`stroke/spray.rs::stamp_at`: o
//! dab, as passadas do Rough, as marcas do spray — cada uma seguida das suas cópias da simetria). O Jitter
//! espalha UMA ponta ([`ph2d_painter_brush::BrushSpec::limiar_da_lane`]); as cópias são pontas DISTINTAS. Só pela
//! distância, duas cópias do mesmo ponto caíam na mesma lane e a 2.ª, sem caminho andado desde a 1.ª, pousava
//! pigmento zero (o pigmento por comprimento) — o Count morria e a simetria perto do eixo perdia uma cópia.
//! Aqui: a k-ésima marca de um ponto é a cópia `k`, e a lane só casa dabs da MESMA cópia.

use ph2d_painter_brush::{BrushSpec, Dab};

/// A cópia de cada dab do lote: `0` para todo dab de um pincel sem cópias (o casamento de sempre); com spray,
/// Rough ou simetria, o ordinal do dab entre as marcas do seu ponto. `grupos` (o mapa do Tiling, vazio sem ele):
/// as cópias da costura são o MESMO dab e levam a cópia dele.
pub(super) fn copia_de_cada_dab(dabs: &[Dab], brush: &BrushSpec, grupos: &[u32]) -> Vec<u32> {
    let tem_copias =
        brush.symmetry.copies() > 1 || brush.spray_count > 1 || brush.rough_pass_count() > 1;
    if !tem_copias {
        return vec![0; dabs.len()];
    }
    let origem = |i: usize| grupos.get(i).map_or(i, |&g| g as usize);
    let mut out: Vec<u32> = Vec::with_capacity(dabs.len());
    for (i, d) in dabs.iter().enumerate() {
        let k = match i.checked_sub(1) {
            None => 0,
            Some(a) if origem(a) == origem(i) => out[a],
            Some(a) if dabs[a].arc_len.to_bits() == d.arc_len.to_bits() => out[a] + 1,
            Some(_) => 0,
        };
        out.push(k);
    }
    out
}

//! O campo PREGUIÇOSO [`Grid::material`] (Óleo Real W2 E3): o [`MaterialCelula`] da tinta de cada célula. VAZIO
//! (= [`MaterialCelula::NEUTRO`] em toda a parte) até o primeiro pincel com material ≠ NEUTRO pintar — 32 B por
//! célula para quem nunca o usa seria a maior grade do motor. Mistura-se pela massa como o [`Grid::relevo`]; a
//! ponta que pega tinta leva-o ([`crate::trail::Trail`]); o arrasto, o Smear e o Blend levam-no com a cor.
//! A advecção do fluxo NÃO o move (como o relevo e o glaze: o material é da tinta POUSADA).

use super::Grid;
use crate::material::MaterialCelula;
use crate::sim::Params;
use crate::tuning::Knob;

/// O campo, alocado a NEUTRO se ainda não existia.
pub fn garante(g: &mut Grid) -> &mut [MaterialCelula] {
    if g.material.is_empty() {
        g.material = vec![MaterialCelula::NEUTRO; g.cells];
    }
    &mut g.material
}

/// O material que o pincel pousa: os 9 knobs do grupo Material ([`Knob::Metallic`]…).
#[must_use]
pub fn do_pincel(p: &Params) -> MaterialCelula {
    let k = |kn: Knob| p.k(kn) as f32;
    MaterialCelula {
        metal: k(Knob::Metallic),
        rugosidade: k(Knob::Roughness),
        nivel_ior: k(Knob::IorLevel),
        verniz: k(Knob::Coat),
        verniz_rugosidade: k(Knob::CoatRoughness),
        veludo: k(Knob::Sheen),
        subsuperficie: k(Knob::Subsurface),
        raio_subsuperficie: k(Knob::SubsurfaceRadius),
        transparencia: k(Knob::Transparency),
    }
}

/// **Lock Reflectivity** ([`Knob::LockMaterial`]): o pincel não muda o material da tinta que já está na célula.
#[must_use]
pub fn travado(p: &Params) -> bool {
    p.k(Knob::LockMaterial) > 0.0
}

/// Pousa `v` de massa com o material `novo` sobre a célula `i`, que tinha `m_old` (susp + sett): a média pela massa
/// (a lei do relevo). `trava` preserva a célula com tinta; o papel limpo recebe o do pincel. Sem o campo, a tinta
/// NEUTRA não o aloca.
pub fn pousa(g: &mut Grid, i: usize, m_old: f64, novo: MaterialCelula, v: f64, trava: bool) {
    if trava && m_old > 0.0 {
        return;
    }
    if g.material.is_empty() && novo == MaterialCelula::NEUTRO {
        return;
    }
    let campo = garante(g);
    let a = campo[i];
    if a != novo {
        campo[i] = if m_old > 0.0 {
            MaterialCelula::mistura(a, novo, (v / (m_old + v)) as f32)
        } else {
            novo
        };
    }
}

/// A média de `ms` pesada por `ws` (o material é linear: a mesma `mistura` em dobra). `None` sem peso.
#[must_use]
pub fn media(ms: impl IntoIterator<Item = (MaterialCelula, f64)>) -> Option<MaterialCelula> {
    let mut acc: Option<(MaterialCelula, f64)> = None;
    for (m, w) in ms {
        if w <= 0.0 {
            continue;
        }
        acc = Some(match acc {
            None => (m, w),
            Some((a, wa)) => (MaterialCelula::mistura(a, m, (w / (wa + w)) as f32), wa + w),
        });
    }
    acc.map(|(m, _)| m)
}

/// O material da caixa `[x0..=x1] × [y0..=y1]`, linha a linha (a amostra do Smear); VAZIO sem o campo.
#[must_use]
pub fn recorte(g: &Grid, x0: i32, y0: i32, x1: i32, y1: i32) -> Vec<MaterialCelula> {
    if g.material.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(((x1 - x0 + 1) * (y1 - y0 + 1)) as usize);
    for y in y0..=y1 {
        let o = y as usize * g.s;
        out.extend_from_slice(&g.material[o + x0 as usize..=o + x1 as usize]);
    }
    out
}

/// O Smear borra o material da célula `i` para a amostra bilinear do [`recorte`] `amostra` (texels `ls`, pesos de
/// massa `ws`) na fracção `t` da cor. Sem o campo, nada.
pub fn borra(
    campo: &mut [MaterialCelula],
    amostra: &[MaterialCelula],
    i: usize,
    ls: [usize; 4],
    ws: [f64; 4],
    t: f64,
) {
    if amostra.is_empty() {
        return;
    }
    if let Some(m) = media(ls.into_iter().zip(ws).map(|(l, w)| (amostra[l], w))) {
        campo[i] = MaterialCelula::mistura(campo[i], m, t as f32);
    }
}

/// Devolve a janela de um recorte (`linhas`); vazio = a grade não tinha o campo ⇒ a janela volta a NEUTRO.
pub fn restaura_janela(g: &mut Grid, janela: (i32, i32, i32, i32), linhas: &[MaterialCelula]) {
    let (n, s) = (g.cells, g.s);
    super::altura::restaura_campo(
        &mut g.material,
        n,
        s,
        janela,
        linhas,
        MaterialCelula::NEUTRO,
    );
}

#[cfg(test)]
#[path = "material_tests.rs"]
mod tests;

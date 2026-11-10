//! O campo PREGUIÇOSO [`Grid::material`] (Óleo Real W2 E3): o [`MaterialCelula`] da tinta de cada célula. VAZIO
//! (= [`MaterialCelula::NEUTRO`] em toda a parte) até o primeiro pincel com material ≠ NEUTRO pintar — 32 B por
//! célula para quem nunca o usa seria a maior grade do motor. Mistura-se pela massa como o [`Grid::relevo`]; a
//! ponta que pega tinta leva-o ([`crate::trail::Trail`]); o arrasto, o Smear e o Blend levam-no com a cor.
//! A advecção do fluxo NÃO o move (como o relevo e o glaze: o material é da tinta POUSADA).

use super::Grid;
use super::campo_esparso::CampoEsparso;
use crate::material::MaterialCelula;
use crate::sim::Params;
use crate::tuning::Knob;

/// O campo da grade (B12c): pedaços esparsos, o ausente lê [`MaterialCelula::NEUTRO`].
pub type CampoMaterial = CampoEsparso<MaterialCelula>;

/// O campo que ainda não existe (o `Vec::new()` de antes).
#[must_use]
pub fn sem_campo() -> CampoMaterial {
    CampoEsparso::vazio(MaterialCelula::NEUTRO)
}

/// O denso `v` como campo (os testes, a tinta que volta).
#[must_use]
pub fn campo_de(v: &[MaterialCelula]) -> CampoMaterial {
    CampoEsparso::de_vec(MaterialCelula::NEUTRO, v, mesmo_bit)
}

/// Igualdade ao bit (o `==` de `f32` iguala `-0.0` e `0.0`).
#[must_use]
pub fn mesmo_bit(a: &MaterialCelula, b: &MaterialCelula) -> bool {
    let f = |c: &MaterialCelula| {
        [
            c.metal,
            c.rugosidade,
            c.nivel_ior,
            c.verniz,
            c.verniz_rugosidade,
            c.veludo,
            c.subsuperficie,
            c.raio_subsuperficie,
            c.transparencia,
            c.transmissao,
            c.relevo_papel,
            c.vidro_claro,
            c.amassa_papel,
        ]
        .map(f32::to_bits)
    };
    f(a) == f(b)
}

/// Os bytes que o campo ocupa (o gate do 1.º toque, B12c).
#[must_use]
pub fn bytes_alocados(g: &Grid) -> usize {
    g.material.bytes_vivos()
}

/// O campo, criado (sem pedaço vivo) se ainda não existia: a 1.ª escrita de cada célula aloca só o pedaço dela.
pub fn garante(g: &mut Grid) -> &mut CampoMaterial {
    if g.material.is_empty() {
        g.material = CampoEsparso::novo(MaterialCelula::NEUTRO, g.cells);
    }
    &mut g.material
}

/// Mexe todas as células (as sondas e os testes; o `iter_mut().enumerate()` do `Vec` de antes): aloca o campo inteiro.
pub fn para_cada(g: &mut Grid, mut f: impl FnMut(usize, &mut MaterialCelula)) {
    let c = garante(g);
    for i in 0..c.len() {
        f(i, &mut c[i]);
    }
}

/// O material que o pincel pousa: os 12 knobs do grupo Material ([`Knob::Metallic`]…).
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
        transmissao: k(Knob::Transmission),
        relevo_papel: k(Knob::PaperRelief),
        vidro_claro: k(Knob::TransmissionMode),
        amassa_papel: k(Knob::PaperPress),
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

/// O alcance, em células, da orla virgem que o material veste ([`veste_virgens`]). MEDIDO no gate
/// `o_material_veste_a_orla_virgem` (tool-painter): o pigmento que a água leva depois do rasto chega a 1–3 células
/// da caixa: alcance 2 (o da aparência) · 3 · 4 → 131 · 39 · 1 das 8 408 células com tinta sem o material (a de 4 com
/// massa 1e-23). Custo: o laço da caixa + 4 por janela, desprezável ao lado do depósito.
pub const ORLA_VIRGEM: i32 = 4;

/// A orla VIRGEM (sem tinta) à volta da caixa que o rasto acabou de pousar recebe o material `novo` — o par de
/// [`super::aparencia::veste_virgens`] (auditoria W3b C4): a advecção não move este campo, e o pigmento que a água
/// leva para fora do rasto chega a células virgens; sem isto elas mostravam o NEUTRO (um aro opaco à volta de um
/// traço de vidro). Uma célula virgem não se desenha: nunca muda um byte do que já está na tela.
pub fn veste_virgens(g: &mut Grid, (x0, y0, x1, y1): (i32, i32, i32, i32), novo: MaterialCelula) {
    if g.material.is_empty() && novo == MaterialCelula::NEUTRO {
        return;
    }
    let r = ORLA_VIRGEM;
    let (x0, y0) = ((x0 - r).max(1), (y0 - r).max(1));
    let (x1, y1) = ((x1 + r).min(g.w as i32), (y1 + r).min(g.h as i32));
    garante(g);
    for y in y0..=y1 {
        let o = y as usize * g.s;
        for i in o + x0 as usize..=o + x1 as usize {
            if g.susp[i] <= 0.0 && g.sett[i] <= 0.0 {
                g.material[i] = novo;
            }
        }
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
        g.material
            .copia_intervalo(o + x0 as usize, o + x1 as usize, &mut out);
    }
    out
}

/// O Smear borra o material da célula `i` para a amostra bilinear do [`recorte`] `amostra` (texels `ls`, pesos de
/// massa `ws`) na fracção `t` da cor. Sem o campo, nada.
pub fn borra(
    campo: &mut CampoMaterial,
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
    super::altura::restaura_campo_esparso(&mut g.material, n, s, janela, linhas, mesmo_bit);
}

#[cfg(test)]
#[path = "material_tests.rs"]
mod tests;

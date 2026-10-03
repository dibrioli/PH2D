//! ⭐⭐⭐ **AS FILEIRAS DO CÉU, no painel** — o molde do [`crate::brilho_painel`]: a tabela é DERIVADA
//! da arrumação do [`Ceu::pack`], e uma escrita é *desempacota, escreve a posição, empacota, saneia*.
//!
//! ⚠️ **Só no Render por MALHA**: o céu fotográfico vive no desenhista de jogo; o traçado (que sai
//! quando o dono aprovar) desenha o estúdio de sempre, e uma fileira que não muda nada lá seria um
//! controlo morto.

use ph2d_field::{Bound, Param};

use crate::ceu_foto::{CAIXA_MAX, Ceu, FORCA_MAX, GIRO_MAX};

/// A secção onde estas fileiras vivem.
const SECCAO: &str = "panel.model3d.section.sky";

/// ⭐ **Os nove céus, como CHAVES** (HR-15) — o `0` é o estúdio de sempre e os outros seguem o
/// [`ph2d_sky::Embarcado::TODOS`] (há gate: `os_nomes_seguem_os_embarcados`).
pub(crate) const CEUS: [&str; 9] = [
    "panel.model3d.sky.studio",
    "panel.model3d.sky.photo_studio",
    "panel.model3d.sky.interior",
    "panel.model3d.sky.city",
    "panel.model3d.sky.courtyard",
    "panel.model3d.sky.forest",
    "panel.model3d.sky.sunrise",
    "panel.model3d.sky.sunset",
    "panel.model3d.sky.night",
];

const LIGA: [&str; 2] = ["panel.model3d.sky.off", "panel.model3d.sky.on"];

struct Linha {
    slot: u8,
    key: &'static str,
    lo: f32,
    hi: f32,
    escolhas: &'static [&'static str],
}

/// ⭐⭐ **AS SEIS LINHAS.**
///
/// | linha | faixa | porquê |
/// |---|---|---|
/// | céu | `0..8` | os nove céus |
/// | giro | `0..360°` | uma volta |
/// | força | `±4` stops | `16×` para cada lado; `0` = a luz média do estúdio |
/// | luz-chave | `0..2` | `1` = a do estúdio; `0` = só o céu (sem sombra da caixa) |
/// | fundo | Off / On | o céu atrás da peça |
/// | desfoque | `0..1` | o `√α` do lóbulo com que o fundo é lido (`1` = só a cor média de cada lado) |
const LINHAS: [Linha; 6] = [
    Linha {
        slot: 0,
        key: "panel.model3d.sky.choice",
        lo: 0.0,
        hi: (CEUS.len() - 1) as f32,
        escolhas: &CEUS,
    },
    Linha {
        slot: 1,
        key: "panel.model3d.sky.rotation",
        lo: 0.0,
        hi: GIRO_MAX,
        escolhas: &[],
    },
    Linha {
        slot: 2,
        key: "panel.model3d.sky.strength",
        lo: -FORCA_MAX,
        hi: FORCA_MAX,
        escolhas: &[],
    },
    Linha {
        slot: 3,
        key: "panel.model3d.sky.key_light",
        lo: 0.0,
        hi: CAIXA_MAX,
        escolhas: &[],
    },
    Linha {
        slot: 4,
        key: "panel.model3d.sky.background",
        lo: 0.0,
        hi: 1.0,
        escolhas: &LIGA,
    },
    Linha {
        slot: 5,
        key: "panel.model3d.sky.blur",
        lo: 0.0,
        hi: 1.0,
        escolhas: &[],
    },
];

/// ⭐ **Porque é que esta fileira não faz nada agora** — o estúdio não tem foto para girar.
fn apagada(l: &Linha, c: &Ceu) -> Option<&'static str> {
    if l.slot != 0 && c.qual == 0 {
        return Some("field.inert.sky_is_studio");
    }
    if l.slot == 5 && !c.fundo {
        return Some("field.inert.sky_background_is_off");
    }
    None
}

/// ⭐⭐⭐ **AS FILEIRAS DO CÉU** — vazias fora do Render por malha.
#[must_use]
pub fn rows(ceu: Ceu, malha_render: bool) -> Vec<ph2d_panel_model3d::ParamRow> {
    if !malha_render {
        return Vec::new();
    }
    let v = ceu.pack();
    LINHAS
        .iter()
        .enumerate()
        .map(|(i, l)| ph2d_panel_model3d::ParamRow {
            entity: 0,
            param: Param::Sky(l.slot),
            key: l.key,
            value: v[l.slot as usize],
            lo: l.lo,
            // ⚠️ `Hard`: os tectos são da LEI (a cerca do `Ceu::sanitized`), não do gesto.
            bound: Bound::Hard(l.hi),
            inert: apagada(l, &ceu),
            integral: !l.escolhas.is_empty(),
            choices: l.escolhas,
            section: (i == 0).then_some(SECCAO),
            swatch: None,
            subject: None,
        })
        .collect()
}

/// ⭐⭐⭐ **A ESCRITA** — desempacota, escreve a posição, empacota, saneia. Fora do alcance devolve o
/// céu intacto.
#[must_use]
pub fn with_number(ceu: Ceu, slot: u8, value: f32) -> Ceu {
    let mut v = ceu.pack();
    let Some(alvo) = v.get_mut(slot as usize) else {
        return ceu;
    };
    *alvo = value;
    Ceu::unpack(&v)
}

#[cfg(test)]
#[path = "ceu_painel_tests.rs"]
mod tests;

//! **As rampas da paleta** (P13, `docs/PixelArt/P13_plano.md` §4): uma rampa é uma sequência
//! ordenada de índices da paleta, do escuro para o claro. Aqui só o DADO e as leis da estrutura;
//! a conta de cor (gerar em OKLCH, a métrica da quantização, a LUT da luz) mora na
//! `ph2d-pixel-rampa`, porque este núcleo não tem dependências (ADR-0196).

use crate::cycle::Cycle;
use crate::sprite::Sprite;

/// O id de uma rampa: vem do mesmo contador do documento (`next_id`) que camadas e frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RampId(pub u32);

/// Uma rampa: `indices[0]` é o degrau mais escuro.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ramp {
    pub id: RampId,
    pub name: String,
    pub indices: Vec<u8>,
}

/// As rampas e os ciclos de cor de uma paleta (o que a `Palette` ganha na P13).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PaletteRamps {
    pub ramps: Vec<Ramp>,
    pub cycles: Vec<Cycle>,
    /// «Light: Palette»: a luz 2D escolhe um degrau da rampa em vez de tingir a cor.
    pub luz_na_paleta: bool,
}

impl PaletteRamps {
    #[must_use]
    pub fn ramp(&self, id: RampId) -> Option<&Ramp> {
        self.ramps.iter().find(|r| r.id == id)
    }
}

/// *Light: Smooth · Palette* (P13 L) como `SetRamps`; `None` = o documento já está assim.
#[must_use]
pub fn set_luz_na_paleta(s: &Sprite, on: bool) -> Option<crate::Op> {
    let mut r = s.ramps().clone();
    (std::mem::replace(&mut r.luz_na_paleta, on) != on)
        .then(|| crate::Op::Doc(Box::new(crate::DocOp::SetRamps(r))))
}

/// A rampa que contém o índice `idx` e a posição dele nela (`0` = o degrau mais escuro).
/// Um índice pertence no máximo a uma rampa ([`check_ramps`]).
#[must_use]
pub fn rampa_de(pal: &PaletteRamps, idx: u8) -> Option<(RampId, usize)> {
    pal.ramps
        .iter()
        .find_map(|r| r.indices.iter().position(|&i| i == idx).map(|p| (r.id, p)))
}

/// As leis: ids não nulos, já emitidos e únicos; cada rampa não vazia; cada índice dentro da
/// paleta e em no máximo uma rampa; cada ciclo numa rampa que existe, um por rampa, período > 0.
pub(crate) fn check_ramps(
    pal: &PaletteRamps,
    palette_len: usize,
    next_id: u32,
) -> Result<(), &'static str> {
    let mut ids = std::collections::BTreeSet::new();
    let mut used = [false; 256];
    for r in &pal.ramps {
        if r.id.0 == 0 || r.id.0 >= next_id || !ids.insert(r.id) {
            return Err("ramp.id");
        }
        if r.indices.is_empty() {
            return Err("ramp.empty");
        }
        for &i in &r.indices {
            if usize::from(i) >= palette_len || std::mem::replace(&mut used[usize::from(i)], true) {
                return Err("ramp.index");
            }
        }
    }
    let mut cycled = std::collections::BTreeSet::new();
    for c in &pal.cycles {
        if !ids.contains(&c.ramp) || !cycled.insert(c.ramp) || c.periodo_ms == 0 {
            return Err("cycle");
        }
    }
    Ok(())
}

/// O `Remap` da paleta (`perm[novo] = antigo`): cada rampa segue as suas cores.
pub(crate) fn remap(pal: &mut PaletteRamps, perm: &[usize]) {
    let mut to_new = [0u8; 256];
    for (new, &old) in perm.iter().enumerate() {
        if let (Some(slot), Ok(n)) = (to_new.get_mut(old), u8::try_from(new)) {
            *slot = n;
        }
    }
    for r in &mut pal.ramps {
        for i in &mut r.indices {
            *i = to_new[usize::from(*i)];
        }
    }
}

impl Sprite {
    /// As rampas e os ciclos da paleta (P13).
    #[must_use]
    pub fn ramps(&self) -> &PaletteRamps {
        &self.ramps
    }
    /// As guias de perspectiva e isometria (P13).
    #[must_use]
    pub fn guias(&self) -> &crate::guia::Guias {
        &self.guias
    }
    /// As faixas de som sob os quadros (P13).
    #[must_use]
    pub fn audio(&self) -> &[crate::audio::FaixaDeAudio] {
        &self.audio
    }
}

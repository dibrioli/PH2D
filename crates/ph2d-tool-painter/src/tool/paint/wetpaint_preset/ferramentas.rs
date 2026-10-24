//! **A biblioteca de cada ferramenta** (dono 24/10, o padrão do Procreate): o Pincel, a Borracha, o Blur, o
//! Smear e o Clone têm cada um a sua lista de presets — a GAVETA é `(ferramenta, meio)`. O preset diz a sua no
//! campo `ferramenta` ([`Parametros::ferramenta`]); os de antes dele, pelo modo (Smear/Blur) ou pela lei Erase
//! Alpha. Os caminhos dos ficheiros não mudam: a pasta organiza, o campo decide.

use super::super::ferramenta_linhas::FerramentaDoPincel;
use super::super::{PaintMedia, PaintMode};
use super::meios::nome_do_meio;
use crate::PainterTool;
use FerramentaDoPincel as F;
use ph2d_brush_presets::Parametros;

/// As ferramentas com biblioteca própria: o nome que o ficheiro grava e o fio do trilho.
const NOMES: [(FerramentaDoPincel, &str, &str); 4] = [
    (F::Borracha, "Eraser", "eraser"),
    (F::Blur, "Blur", "blur"),
    (F::Smear, "Smear", "smear"),
    (F::Clone, "Clone", "clone"),
];

/// O nome que o preset grava para `f` (`None` = o Pincel, ou uma ferramenta sem biblioteca).
#[must_use]
pub fn nome_da_ferramenta(f: FerramentaDoPincel) -> Option<&'static str> {
    NOMES.iter().find(|(g, _, _)| *g == f).map(|(_, n, _)| *n)
}

/// O fio do trilho de `f` (`"brush"` para o Pincel).
fn fio_da_ferramenta(f: FerramentaDoPincel) -> &'static str {
    NOMES
        .iter()
        .find(|(g, _, _)| *g == f)
        .map_or("brush", |(_, _, fio)| *fio)
}

/// ⭐ **A ferramenta de um preset.** Um nome que esta versão não conhece fica no Pincel.
#[must_use]
pub fn ferramenta_do_preset(p: &Parametros) -> FerramentaDoPincel {
    if let Some(n) = &p.ferramenta {
        return NOMES
            .iter()
            .find(|(_, m, _)| m == n)
            .map_or(F::Pincel, |(f, _, _)| *f);
    }
    // Os de antes do campo: o Smear/Blur do Digital/Watercolor (o `Smear` do Wet é uma ferramenta DO FLUIDO) e
    // as borrachas de fábrica, que eram pincéis Erase Alpha.
    let rail = p.media == nome_do_meio(PaintMedia::Digital)
        || p.media == nome_do_meio(PaintMedia::Watercolor);
    match p.paint_mode.as_deref() {
        Some("Smear") if rail => F::Smear,
        Some("Blur") if rail => F::Blur,
        _ if p.media == nome_do_meio(PaintMedia::Digital)
            && p.brush.get("blend").and_then(|v| v.as_str()) == Some("EraseAlpha") =>
        {
            F::Borracha
        }
        _ => F::Pincel,
    }
}

/// A gaveta de presets: a ferramenta e o meio.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Gaveta {
    pub ferramenta: FerramentaDoPincel,
    pub meio: PaintMedia,
}

impl Gaveta {
    /// Uma chave pequena e única (o «aplicado» de cada gaveta).
    #[must_use]
    pub fn chave(self) -> u8 {
        let f = match self.ferramenta {
            F::Pincel => 0,
            F::Borracha => 1,
            F::Blur => 2,
            F::Smear => 3,
            _ => 4,
        };
        f * PaintMedia::COUNT + self.meio.to_u8()
    }

    /// O nome no `estado.json` (o lembrado de cada gaveta): o do meio no Pincel, como sempre.
    #[must_use]
    pub fn nome(self) -> String {
        match nome_da_ferramenta(self.ferramenta) {
            None => nome_do_meio(self.meio).to_owned(),
            Some(f) => format!("{f}/{}", nome_do_meio(self.meio)),
        }
    }
}

impl PainterTool {
    /// A ferramenta cuja biblioteca o navegador mostra (`None` = a ferramenta não tem presets: Máscara, Fill…).
    /// As do Impasto (Knife, Sculpt) são pincéis do Impasto.
    #[must_use]
    pub fn ferramenta_dos_presets(&self) -> Option<FerramentaDoPincel> {
        match self.paint.paint_mode {
            PaintMode::Paint | PaintMode::WetPaint if self.paint.eraser => Some(F::Borracha),
            PaintMode::Paint | PaintMode::WetPaint | PaintMode::Knife | PaintMode::Sculpt => {
                Some(F::Pincel)
            }
            PaintMode::Smear => Some(F::Smear),
            PaintMode::Blur => Some(F::Blur),
            PaintMode::Clone => Some(F::Clone),
            _ => None,
        }
    }

    /// A gaveta de agora.
    #[must_use]
    pub fn gaveta_de_presets(&self) -> Option<Gaveta> {
        self.ferramenta_dos_presets().map(|ferramenta| Gaveta {
            ferramenta,
            meio: self.paint_media(),
        })
    }

    /// Pega a ferramenta do preset ANTES do meio e do pincel (a lição de 22/07: o meio arma-se no slot onde o
    /// artista aterra).
    pub(super) fn pegar_a_ferramenta_do_preset(&mut self, p: &Parametros) -> FerramentaDoPincel {
        let f = ferramenta_do_preset(p);
        if self.ferramenta_dos_presets() != Some(f) {
            self.set_paint_tool_mode(fio_da_ferramenta(f));
        }
        f
    }
}

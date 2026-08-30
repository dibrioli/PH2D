//! **AS FIXTURAS partilhadas dos gates do pincel de contorno** — extraídas do ficheiro de gates
//! quando ele passou o teto de LOC (2026-08-30), e extraídas **por responsabilidade**: aqui está o
//! que se CONSTRÓI, ao lado está o que se AFIRMA.
//!
//! ⚠️ Elas são `pub(super)` porque os dois módulos de gate são irmãos sob o `brush_stroke.rs` — e
//! não `pub`, porque uma fixtura de teste que escapa da crate vira API por acidente.

use super::*;
use crate::{Rgba8, VecPathId, VecVertex};

/// Um quadrado de lado `l`, FECHADO — o contorno que o pincel percorre.
pub(super) fn quadrado(l: f64) -> VecPath {
    VecPath {
        verts: [[0.0, 0.0], [l, 0.0], [l, l], [0.0, l]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    }
}

/// A arte: um losango de `w × h`, centrado na origem.
pub(super) fn arte(w: f64, h: f64) -> VecPath {
    VecPath {
        verts: [
            [-w * 0.5, 0.0],
            [0.0, -h * 0.5],
            [w * 0.5, 0.0],
            [0.0, h * 0.5],
        ]
        .map(VecVertex::corner)
        .to_vec(),
        closed: true,
        ..VecPath::default()
    }
}

pub(super) fn pincel() -> BrushStroke {
    BrushStroke {
        art: Some(VecPathId::from(1u64)),
        fallback: Rgba8::new(1, 2, 3, 255),
        spacing: 1.0,
        offset: 0.0,
        flip: false,
        rotation_deg: 0.0,
        scale: 1.0,
    }
}

/// ⭐ **O TRAÇO que carrega o pincel** — a porta por onde o motor recebe tudo o que precisa: a
/// arte, a largura da faixa e o tracejado, os três do MESMO objecto.
pub(super) fn traco(b: &BrushStroke, width: f64, dash: Option<(f64, f64)>) -> crate::StrokeSpec {
    let mut s = crate::StrokeSpec::new(b.fallback, width);
    s.paint = crate::StrokePaint::Brush(Box::new(b.clone()));
    s.dash = dash;
    s
}

/// Uma reta ABERTA de `(0,0)` a `(l,0)` — a guia em que **a posição de arco é a coordenada `x`**,
/// e é por isso que ela existe: num quadrado não há como ler onde uma cópia caiu sem refazer a
/// travessia, e uma régua que refaz a conta que mede não mede nada.
pub(super) fn segmento(l: f64) -> VecPath {
    VecPath {
        verts: [[0.0, 0.0], [l, 0.0]].map(VecVertex::corner).to_vec(),
        closed: false,
        ..VecPath::default()
    }
}

/// O centro da caixa de uma cópia.
pub(super) fn centro(p: &VecPath) -> [f64; 2] {
    let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
    for v in &p.verts {
        for k in 0..2 {
            lo[k] = lo[k].min(v.anchor[k]);
            hi[k] = hi[k].max(v.anchor[k]);
        }
    }
    [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5]
}

/// A altura de cada cópia, medida na saída.
pub(super) fn altura(copias: &[VecPath]) -> f64 {
    let (mut lo, mut hi) = (f64::MAX, f64::MIN);
    for v in copias.first().map(|c| c.verts.clone()).unwrap_or_default() {
        lo = lo.min(v.anchor[1]);
        hi = hi.max(v.anchor[1]);
    }
    hi - lo
}

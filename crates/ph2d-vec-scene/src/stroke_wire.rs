//! **O `StrokeSpec` lê-se pela VERSÃO de quem o gravou** (F16c, v22 → v23).
//!
//! O postcard é posicional: ler bytes v22 com o tipo v23 não erra, lê lixo bem-formado. E o
//! `StrokeSpec` vive no FUNDO de três documentos — a `VecScene` (dentro do `ProjectFile`), os
//! `ObjectPose` dos estados de UI, e os blobs de `VecPath` que o Envelope e a pele dos ossos
//! guardam dentro de componentes. Congelar uma cópia de cada um custaria centenas de linhas por
//! um campo; ⇒ só o `StrokeSpec` tem forma antiga ([`StrokeSpecV22`]), e quem conhece a versão do
//! ficheiro abre um âmbito ([`with_stroke_layout_v22`]) em que todo `StrokeSpec` desserializado
//! nesta thread usa essa forma.
//!
//! ⚠️ **O âmbito é da THREAD e só vale durante a chamada.** Um blob guardado e descodificado mais
//! tarde (o `source` do Envelope e da pele) fica fora dele — por isso o load de um ficheiro antigo
//! REGRAVA esses blobs na forma nova, uma vez, em vez de os deixar à espera de um leitor que já não
//! sabe a versão.

use std::cell::Cell;

use serde::{Deserialize, Deserializer};

use crate::{DashPattern, LineCap, LineJoin, Marker, StrokeAlign, StrokePaint, StrokeSpec};

thread_local! {
    static V22: Cell<u32> = const { Cell::new(0) };
}

/// Corre `f` a ler todo [`StrokeSpec`] na forma do `VEC_SCENE_SCHEMA_VERSION` 22 (um par de
/// tracejado, um tamanho de ponta). Reentrante; o âmbito fecha mesmo num `panic`.
pub fn with_stroke_layout_v22<T>(f: impl FnOnce() -> T) -> T {
    let _v22 = StrokeLayoutV22::enter();
    f()
}

/// O mesmo âmbito como GUARDA — para um leitor cujo corpo tem `return`s pelo meio (o do
/// `.ph2dproj`): o âmbito dura até o guarda cair.
#[must_use = "o âmbito fecha quando o guarda cai"]
pub struct StrokeLayoutV22(());

impl StrokeLayoutV22 {
    pub fn enter() -> Self {
        V22.with(|c| c.set(c.get() + 1));
        Self(())
    }
}

impl Drop for StrokeLayoutV22 {
    fn drop(&mut self) {
        V22.with(|c| c.set(c.get() - 1));
    }
}

fn reading_v22() -> bool {
    V22.with(|c| c.get() > 0)
}

/// A forma v22, campo a campo — ⛔ congelada: mudá-la deixa de ler ficheiros reais.
#[derive(Deserialize)]
struct StrokeSpecV22 {
    paint: StrokePaint,
    width: f64,
    cap: LineCap,
    join: LineJoin,
    dash: Option<(f64, f64)>,
    marker_start: Marker,
    marker_end: Marker,
    marker_scale: f64,
    marker_round: f64,
    align: StrokeAlign,
}

/// O espelho da forma VIVA — a mesma ordem do `Serialize` derivado do [`StrokeSpec`] (o gate
/// `the_live_layout_round_trips` reprova uma divergência).
#[derive(Deserialize)]
struct StrokeSpecV23 {
    paint: StrokePaint,
    width: f64,
    cap: LineCap,
    join: LineJoin,
    dash: Option<DashPattern>,
    marker_start: Marker,
    marker_end: Marker,
    marker_scale_start: f64,
    marker_scale_end: f64,
    marker_round: f64,
    align: StrokeAlign,
}

impl From<StrokeSpecV22> for StrokeSpec {
    fn from(o: StrokeSpecV22) -> Self {
        Self {
            paint: o.paint,
            width: o.width,
            cap: o.cap,
            join: o.join,
            dash: o.dash.map(|(d, g)| DashPattern::single(d, g)),
            marker_start: o.marker_start,
            marker_end: o.marker_end,
            marker_scale_start: o.marker_scale,
            marker_scale_end: o.marker_scale,
            marker_round: o.marker_round,
            align: o.align,
        }
    }
}

impl From<StrokeSpecV23> for StrokeSpec {
    fn from(o: StrokeSpecV23) -> Self {
        Self {
            paint: o.paint,
            width: o.width,
            cap: o.cap,
            join: o.join,
            dash: o.dash,
            marker_start: o.marker_start,
            marker_end: o.marker_end,
            marker_scale_start: o.marker_scale_start,
            marker_scale_end: o.marker_scale_end,
            marker_round: o.marker_round,
            align: o.align,
        }
    }
}

impl<'de> Deserialize<'de> for StrokeSpec {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        if reading_v22() {
            StrokeSpecV22::deserialize(d).map(Into::into)
        } else {
            StrokeSpecV23::deserialize(d).map(Into::into)
        }
    }
}

#[cfg(test)]
#[path = "stroke_wire_tests.rs"]
mod tests;

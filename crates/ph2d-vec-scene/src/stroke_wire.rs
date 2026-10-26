//! **O fio lê-se pela VERSÃO de quem o gravou** (F16c v22 → v23 · W4.2 v23 → v24).
//!
//! O postcard é posicional: ler bytes antigos com o tipo vivo não erra, lê lixo bem-formado. O
//! `StrokeSpec` e o `VecVertex`/`VecPath` vivem no FUNDO de três documentos — a `VecScene` (dentro do
//! `ProjectFile`), os `ObjectPose` dos estados de UI, e os blobs de `VecPath` que o Envelope e a pele
//! dos ossos guardam dentro de componentes. ⇒ só esses tipos têm forma antiga, e quem conhece a versão
//! do ficheiro abre um âmbito ([`with_wire_layout`]) com a `VEC_SCENE_SCHEMA_VERSION` dos bytes: todo
//! tipo desserializado nesta thread lê a forma dessa versão (v22 = traço e geometria antigos · v23 =
//! geometria sem ids, [`crate::vert_id`]).
//!
//! ⚠️ **O âmbito é da THREAD e só vale durante a chamada.** Um blob guardado e descodificado mais
//! tarde (o `source` do Envelope e da pele) fica fora dele — por isso o load de um ficheiro antigo
//! REGRAVA esses blobs na forma nova, uma vez. Âmbitos aninhados valem pelo MAIS ANTIGO.

use std::cell::Cell;

use serde::{Deserialize, Deserializer};

use crate::{DashPattern, LineCap, LineJoin, Marker, StrokeAlign, StrokePaint, StrokeSpec};

thread_local! {
    /// A versão dos bytes em leitura; `0` = a forma viva.
    static LAYOUT: Cell<u32> = const { Cell::new(0) };
}

/// Corre `f` a ler na forma da `VEC_SCENE_SCHEMA_VERSION` `ver`. Reentrante; fecha mesmo num `panic`.
pub fn with_wire_layout<T>(ver: u32, f: impl FnOnce() -> T) -> T {
    let _g = WireLayout::enter(ver);
    f()
}

/// O mesmo âmbito como GUARDA — para um leitor cujo corpo tem `return`s pelo meio (o do
/// `.ph2dproj`): o âmbito dura até o guarda cair.
#[must_use = "o âmbito fecha quando o guarda cai"]
pub struct WireLayout {
    anterior: u32,
}

impl WireLayout {
    pub fn enter(ver: u32) -> Self {
        let anterior = LAYOUT.with(|c| {
            let a = c.get();
            c.set(if a == 0 { ver } else { a.min(ver) });
            a
        });
        Self { anterior }
    }
}

impl Drop for WireLayout {
    fn drop(&mut self) {
        LAYOUT.with(|c| c.set(self.anterior));
    }
}

fn reading_at_most(ver: u32) -> bool {
    LAYOUT.with(|c| (1..=ver).contains(&c.get()))
}

fn reading_v22() -> bool {
    reading_at_most(crate::schema::VEC_SCENE_SCHEMA_V22)
}

/// A geometria sem id por vértice (`<= 23`).
pub(crate) fn reading_geometry_v23() -> bool {
    reading_at_most(crate::schema::VEC_SCENE_SCHEMA_V23)
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

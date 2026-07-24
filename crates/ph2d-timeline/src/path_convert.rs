//! **A conversão entre os dois modos, e o que ela CUSTA** ([ADR-0141] §5).
//!
//! Position e o par TranslationX/Y são **modos**, não um modo mais um flag — a leitura
//! do After Effects, que diz com todas as letras que *Separate Dimensions precludes
//! having Spatial Keyframes*. Então há um gesto para trocar, e o gesto é **explícito e
//! nomeado**: nada aqui acontece por baixo do pano.
//!
//! # Porque a conversão PERDE coisa, nos dois sentidos
//!
//! **Separate → Path.** Os dois eixos têm keys em tempos **independentes** — é a razão
//! de o modo existir (o X chega antes do Y, e isso é *overlap*). Uma trajetória tem uma
//! âncora por instante, então a conversão toma a **união** dos tempos e amostra as duas
//! curvas em cada um. Onde só um eixo tinha key, o outro é amostrado no meio da curva
//! dele — o valor está certo, mas a **tangente** daquela curva não sobrevive: uma
//! Bézier em X e outra em Y não se compõem numa tangente espacial mais um ease
//! temporal. O que se perde são os *eases*, e a conversão os CONTA.
//!
//! **Path → Separate.** Exata nas keys e diferente entre elas: o caminho curva, e duas
//! tracks escalares só reproduziriam essa curva com beziers próprios que a conversão
//! não tem como inventar sem escolher por quem autora. As posições nas keys são
//! idênticas; o meio do caminho passa a ser interpolação de coordenadas — que é
//! exatamente a diferença que o modo Path existe para remover.
//!
//! Nenhuma delas é uma perda a esconder: [`ConversionReport`] existe para o shell a
//! dizer em voz alta, e um gate exige que os números sejam verdadeiros.
//!
//! [ADR-0141]: ../../../docs/architecture/decisions/0141-timeline-position-is-one-2d-channel-and-separate-axes-are-a-mode.md

use ph2d_anim::{AnimValue, AttributeEvaluator, Interp, RationalTime};

use crate::doc::TimelineDoc;
use crate::path::MotionPath;
use crate::prop::PropKind;

/// **How a NEW position key should be authored for one entity** — the question
/// `K` and auto-key ask before touching an object's position ([ADR-0141]).
///
/// The two modes are mutually exclusive per entity (a Position path OR separate
/// X/Y, never both — mixing them is the conflict that keys separate axes over a
/// visible motion path). So the answer is resolved, not chosen every time: an
/// entity already animating in one mode keeps it; only a *fresh* one falls to the
/// transport default.
///
/// [ADR-0141]: ../../../docs/architecture/decisions/0141-timeline-position-is-one-2d-channel-and-separate-axes-are-a-mode.md
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PositionKeyMode {
    /// One 2D channel following a trajectory: a key is a motion-path anchor.
    Path,
    /// Separate `TranslationX`/`TranslationY` scalar tracks.
    Separate,
}

/// O que uma conversão fez, e o que ela **deixou pelo caminho**.
///
/// Não é telemetria: é o que o shell mostra ao artista. Uma conversão que descarta
/// trabalho em silêncio é a diferença entre uma ferramenta e uma armadilha.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ConversionReport {
    /// Quantas keys o resultado tem.
    pub keys: usize,
    /// Quantos instantes existiam num eixo e **não** no outro. São eles que fazem os
    /// dois modos não serem intercambiáveis: o modo separado os expressa, o Path os
    /// junta num instante só.
    pub unmatched_times: usize,
    /// Quantas keys perderam o **ease** que carregavam (nem `Linear` nem `Hold`).
    /// Interpolação de curva não atravessa a fronteira dos modos.
    pub dropped_eases: usize,
}

impl TimelineDoc {
    /// **Separate → Path**: as duas tracks escalares de `entity` viram uma trajetória
    /// mais uma track de distância. Devolve `None` se não há nenhuma das duas.
    ///
    /// As tracks de origem são **removidas** — dois modos ao mesmo tempo seriam dois
    /// autores da mesma pose, e o de trás venceria em silêncio.
    pub fn separate_to_path(&mut self, entity: u64) -> Option<ConversionReport> {
        let (tx, ty) = (
            self.binding_for(entity, PropKind::TranslationX)?.target,
            self.binding_for(entity, PropKind::TranslationY)
                .map(|b| b.target),
        );
        let clip = self.active_clip();
        let track_x = clip.track(tx)?;
        let track_y = ty.and_then(|t| clip.track(t));

        // A UNIÃO dos instantes: um eixo pode ter key onde o outro não tem, e é
        // precisamente essa liberdade que a trajetória não expressa.
        let mut times: Vec<RationalTime> = track_x.keys().iter().map(|k| k.t).collect();
        let in_x = times.len();
        if let Some(ty) = track_y {
            times.extend(ty.keys().iter().map(|k| k.t));
        }
        times.sort_unstable();
        times.dedup();
        if times.is_empty() {
            return None;
        }
        let in_y = track_y.map_or(0, |t| t.keys().len());
        let unmatched_times = times.len() * 2 - in_x - in_y;

        // O ease que não atravessa: contado ANTES de as tracks morrerem.
        let eased = |tr: &ph2d_anim::Track| {
            tr.keys()
                .iter()
                .filter(|k| !matches!(k.interp, Interp::Linear | Interp::Hold))
                .count()
        };
        let dropped_eases = eased(track_x) + track_y.map_or(0, eased);

        let sample = |tr: Option<&ph2d_anim::Track>, t: f64| -> f32 {
            match tr.map(|tr| tr.sample(t)) {
                Some(AnimValue::Float(v)) => v,
                _ => 0.0,
            }
        };
        let pts: Vec<[f32; 2]> = times
            .iter()
            .map(|t| {
                let s = t.to_seconds();
                [sample(Some(track_x), s), sample(track_y, s)]
            })
            .collect();

        // Fora com as duas, e a trajetória entra pela porta única do K — uma âncora por
        // instante, re-suavizando à medida que cresce.
        self.unbind(entity, PropKind::TranslationX);
        self.unbind(entity, PropKind::TranslationY);
        let target = self.bind(entity, PropKind::Position);
        for (t, p) in times.iter().zip(pts) {
            self.add_path_key(target, *t, p);
        }
        Some(ConversionReport {
            keys: times.len(),
            unmatched_times,
            dropped_eases,
        })
    }

    /// **Path → Separate**: a trajetória vira duas tracks escalares, exatas nas keys.
    ///
    /// A trajetória e a track de distância são **removidas** — pela mesma razão do
    /// caminho inverso. Devolve `None` sem um binding Position com caminho e keys.
    pub fn path_to_separate(&mut self, entity: u64) -> Option<ConversionReport> {
        let b = self.binding_for(entity, PropKind::Position)?;
        let (target, path) = (b.target, b.path.clone()?);
        let track = self.active_clip().track(target)?;
        if track.keys().is_empty() {
            return None;
        }
        // Onde o objeto está em cada key — pela MESMA composição que o apply faz, senão
        // a conversão poria o objeto num sítio e o apply o desenharia noutro.
        let samples: Vec<(RationalTime, [f32; 2], Interp)> = track
            .keys()
            .iter()
            .filter_map(|k| {
                let AnimValue::Float(s) = k.value else {
                    return None;
                };
                path.at(f64::from(s)).map(|p| (k.t, p.point, k.interp))
            })
            .collect();
        let dropped_eases = samples
            .iter()
            .filter(|(_, _, i)| !matches!(i, Interp::Linear | Interp::Hold))
            .count();

        self.unbind(entity, PropKind::Position);
        for (t, p, interp) in &samples {
            for (prop, v) in [
                (PropKind::TranslationX, p[0]),
                (PropKind::TranslationY, p[1]),
            ] {
                // O ease É preservado aqui: um ease em DISTÂNCIA aplicado aos dois
                // eixos reproduz o timing, porque a distância é monótona. O que não
                // sobrevive é a FORMA entre as keys, e é isso que o relatório conta.
                self.insert_key(entity, prop, *t, AnimValue::Float(v), *interp);
            }
        }
        Some(ConversionReport {
            keys: samples.len(),
            // Os dois eixos nascem com os MESMOS instantes: a trajetória não sabe
            // expressar tempos independentes, então não há nada a desemparelhar.
            unmatched_times: 0,
            dropped_eases,
        })
    }

    /// A trajetória de `entity`, se ele estiver em modo Path — o que o painel pergunta
    /// para saber **qual** conversão oferecer.
    #[must_use]
    pub fn position_path(&self, entity: u64) -> Option<&MotionPath> {
        self.binding_for(entity, PropKind::Position)?.path.as_ref()
    }

    /// **Which mode a new position key for `entity` must use** ([ADR-0141]).
    ///
    /// The existing animation decides — a Position binding is Path, a
    /// `TranslationX`/`Y` binding is Separate — because the two modes cannot
    /// coexist, and keying the wrong one is precisely the *"even with points on the
    /// canvas the auto-key writes X/Y"* conflict. A fresh entity (no position
    /// animation yet) has no mode of its own, so it takes `default_path` (the
    /// transport toggle, [`crate::TimelineFlags::motion_path_keys`]).
    ///
    /// [ADR-0141]: ../../../docs/architecture/decisions/0141-timeline-position-is-one-2d-channel-and-separate-axes-are-a-mode.md
    #[must_use]
    pub fn position_key_mode(&self, entity: u64, default_path: bool) -> PositionKeyMode {
        if self.binding_for(entity, PropKind::Position).is_some() {
            PositionKeyMode::Path
        } else if self.binding_for(entity, PropKind::TranslationX).is_some()
            || self.binding_for(entity, PropKind::TranslationY).is_some()
        {
            PositionKeyMode::Separate
        } else if default_path {
            PositionKeyMode::Path
        } else {
            PositionKeyMode::Separate
        }
    }
}

#[cfg(test)]
#[path = "path_convert_tests.rs"]
mod tests;

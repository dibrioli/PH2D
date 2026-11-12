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
//! curvas em cada um. A **forma** atravessa (dono 12/11): cada segmento vira a cúbica
//! espacial pelas amostras em 0, ⅓, ⅔ e 1, exata para Linear e para a BezierW de terços que
//! o canvas escreve. O que se perde é o **ritmo** de cada eixo (nasce constante ao longo do
//! caminho), e a conversão CONTA os eases que o levavam.
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
use crate::path::{MotionPath, PathAnchor, TangentKind};
use crate::prop::PropKind;

/// A BezierW cujos tempos estão nos terços: o ritmo é linear e ela só guarda a FORMA do arco.
fn so_forma(i: Interp) -> bool {
    const TERCO: f64 = 1.0 / 3.0;
    const TOL: f64 = 1e-6;
    matches!(i, Interp::BezierW { x1, x2, .. } if (x1 - TERCO).abs() < TOL && (x2 - 2.0 * TERCO).abs() < TOL)
}

/// `Smooth` quando as duas alças existem e são colineares e opostas; senão `Corner`.
fn tipo_das_alcas(a: [f32; 2], b: [f32; 2]) -> TangentKind {
    let (la, lb) = (a[0].hypot(a[1]), b[0].hypot(b[1]));
    if la <= f32::EPSILON || lb <= f32::EPSILON {
        return TangentKind::Corner;
    }
    let cruz = (a[0] * b[1] - a[1] * b[0]) / (la * lb);
    let ponto = (a[0] * b[0] + a[1] * b[1]) / (la * lb);
    if cruz.abs() < 1e-3 && ponto < 0.0 {
        TangentKind::Smooth
    } else {
        TangentKind::Corner
    }
}

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

/// O plano Separate → Path de um clip: `None` se ele não tem chave de X nem de Y.
fn plano_separado(
    clip: &ph2d_anim::Clip,
    c: usize,
    alvos: [Option<ph2d_anim::AnimTarget>; 2],
    repouso: [f32; 2],
) -> Option<PlanoDoClip> {
    let [track_x, track_y] = alvos.map(|t| t.and_then(|t| clip.track(t)));
    // A UNIÃO dos instantes: um eixo pode ter key onde o outro não tem, e é
    // precisamente essa liberdade que a trajetória não expressa.
    let mut times: Vec<RationalTime> = [track_x, track_y]
        .into_iter()
        .flatten()
        .flat_map(|tr| tr.keys().iter().map(|k| k.t))
        .collect();
    let contadas = times.len();
    times.sort_unstable();
    times.dedup();
    if times.is_empty() {
        return None;
    }
    let unmatched_times = times.len() * 2 - contadas;

    // O ease que não atravessa: contado ANTES de as tracks morrerem. A BezierW de terços
    // (a que o canvas escreve) só guarda FORMA, que a trajetória carrega: não conta.
    let eased = |tr: &ph2d_anim::Track| {
        tr.keys()
            .iter()
            .filter(|k| !matches!(k.interp, Interp::Linear | Interp::Hold | Interp::Nearest))
            .filter(|k| !so_forma(k.interp))
            .count()
    };
    let dropped_eases = [track_x, track_y].into_iter().flatten().map(eased).sum();

    let sample = |tr: Option<&ph2d_anim::Track>, rest: f32, t: f64| -> f32 {
        match tr.map(|tr| tr.sample(t)) {
            Some(AnimValue::Float(v)) => v,
            _ => rest,
        }
    };
    let xy = |t: f64| {
        [
            sample(track_x, repouso[0], t),
            sample(track_y, repouso[1], t),
        ]
    };
    let parado = |tr: Option<&ph2d_anim::Track>, t: RationalTime| {
        tr.and_then(|tr| tr.keys().iter().rev().find(|k| k.t <= t))
            .is_some_and(|k| matches!(k.interp, Interp::Hold | Interp::Nearest))
    };
    // A FORMA atravessa (dono 12/11, item 13): cada segmento vira a cúbica espacial que passa
    // pelas amostras em 0, ⅓, ⅔ e 1 — exata quando os dois eixos são cúbicos no tempo (Linear e
    // a BezierW de terços do canvas). Um segmento parado (Hold) fica reto, sem alças.
    let mut anchors: Vec<PathAnchor> = times
        .iter()
        .map(|t| PathAnchor::corner(xy(t.to_seconds())))
        .collect();
    let mut held = vec![false; times.len()];
    for i in 0..times.len().saturating_sub(1) {
        if parado(track_x, times[i]) || parado(track_y, times[i]) {
            held[i] = true;
            continue;
        }
        let (a, b) = (times[i].to_seconds(), times[i + 1].to_seconds());
        let p = [0.0, 1.0, 2.0, 3.0].map(|k| xy(a + (b - a) * k / 3.0));
        let ctl = |w: [f32; 4]| [0, 1].map(|j| (0..4).map(|k| w[k] * p[k][j]).sum::<f32>() / 6.0);
        let (c1, c2) = (ctl([-5.0, 18.0, -9.0, 2.0]), ctl([2.0, -9.0, 18.0, -5.0]));
        anchors[i].out_handle = [c1[0] - p[0][0], c1[1] - p[0][1]];
        anchors[i + 1].in_handle = [c2[0] - p[3][0], c2[1] - p[3][1]];
    }
    for a in &mut anchors {
        a.kind = tipo_das_alcas(a.in_handle, a.out_handle);
    }
    Some(PlanoDoClip {
        clip: c,
        report: ConversionReport {
            keys: times.len(),
            unmatched_times,
            dropped_eases,
        },
        times,
        anchors,
        held,
    })
}

/// A conversão de UM clip, calculada antes de qualquer track morrer.
struct PlanoDoClip {
    clip: usize,
    times: Vec<RationalTime>,
    anchors: Vec<PathAnchor>,
    held: Vec<bool>,
    report: ConversionReport,
}

fn soma(a: ConversionReport, b: ConversionReport) -> ConversionReport {
    ConversionReport {
        keys: a.keys + b.keys,
        unmatched_times: a.unmatched_times + b.unmatched_times,
        dropped_eases: a.dropped_eases + b.dropped_eases,
    }
}

impl TimelineDoc {
    /// **Separate → Path**: as duas tracks escalares de `entity` viram uma trajetória
    /// mais uma track de distância, em **todo clip** que as anima (converter só o ativo
    /// deixaria órfãs as tracks dos outros, e o objeto pararia de andar neles). Devolve
    /// `None` (e não muda nada) se nenhum clip tem chave de X ou Y.
    ///
    /// As tracks de origem são **removidas** — dois modos ao mesmo tempo seriam dois
    /// autores da mesma pose, e o de trás venceria em silêncio. Um eixo sem track num clip
    /// vale o repouso do binding; sem repouso capturado, `repouso`.
    pub fn separate_to_path_with_rest(
        &mut self,
        entity: u64,
        repouso: [f32; 2],
    ) -> Option<ConversionReport> {
        let eixo = |doc: &Self, prop, fallback: f32| {
            doc.binding_for(entity, prop)
                .map(|b| (Some(b.target), b.rest.unwrap_or(fallback)))
                .unwrap_or((None, fallback))
        };
        let (tx, rx) = eixo(self, PropKind::TranslationX, repouso[0]);
        let (ty, ry) = eixo(self, PropKind::TranslationY, repouso[1]);
        let planos: Vec<PlanoDoClip> = (0..self.clips().len())
            .filter_map(|c| plano_separado(&self.clips()[c].clip, c, [tx, ty], [rx, ry]))
            .collect();
        if planos.is_empty() {
            return None;
        }
        let ativo = self.active_index();
        for c in 0..self.clips().len() {
            self.set_active(c);
            for t in [tx, ty].into_iter().flatten() {
                self.active_clip_mut().remove_track(t);
            }
        }
        self.set_active(ativo);
        self.unbind(entity, PropKind::TranslationX);
        self.unbind(entity, PropKind::TranslationY);
        let target = self.bind(entity, PropKind::Position);
        let mut report = ConversionReport::default();
        for plano in planos {
            self.set_active(plano.clip);
            for (t, h) in plano.times.iter().zip(&plano.held) {
                let interp = if *h { Interp::Hold } else { Interp::Linear };
                self.upsert_key(
                    entity,
                    PropKind::Position,
                    *t,
                    AnimValue::Float(0.0),
                    interp,
                );
            }
            self.set_active_path(target, MotionPath::new(plano.anchors));
            self.rewrite_path_key_values(target);
            report = soma(report, plano.report);
        }
        self.set_active(ativo);
        Some(report)
    }

    /// [`Self::separate_to_path_with_rest`] sem um repouso de fora (o do binding, ou zero).
    pub fn separate_to_path(&mut self, entity: u64) -> Option<ConversionReport> {
        self.separate_to_path_with_rest(entity, [0.0, 0.0])
    }

    /// **Path → Separate**: a trajetória vira duas tracks escalares, exatas nas keys, em
    /// **todo clip** que anima a Position de `entity`.
    ///
    /// A trajetória e a track de distância são **removidas** — pela mesma razão do
    /// caminho inverso. Devolve `None` sem um binding Position com caminho e keys.
    pub fn path_to_separate(&mut self, entity: u64) -> Option<ConversionReport> {
        let target = self.binding_for(entity, PropKind::Position)?.target;
        let ativo = self.active_index();
        let mut planos = Vec::new();
        for c in 0..self.clips().len() {
            self.set_active(c);
            // ⚠️ Pela porta de RESOLUÇÃO (`path_for`), não pela crua: converter é transformar
            // *o que o objeto faz na tela* em duas tracks escalares, e o que ele faz sai da
            // trajetória que o avaliador usa. Um clip que só cronometra sobre a jornada de
            // outro tem uma conversão legítima a oferecer.
            let Some(path) = self.path_for(target).cloned() else {
                continue;
            };
            let Some(track) = self.active_clip().track(target) else {
                continue;
            };
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
            if !samples.is_empty() {
                planos.push((c, samples));
            }
        }
        if planos.is_empty() {
            self.set_active(ativo);
            return None;
        }
        for c in 0..self.clips().len() {
            self.set_active(c);
            self.active_clip_mut().remove_track(target);
            self.take_active_path(target);
        }
        self.set_active(ativo);
        self.unbind(entity, PropKind::Position);
        let mut report = ConversionReport::default();
        for (c, samples) in planos {
            self.set_active(c);
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
            let dropped_eases = samples
                .iter()
                .filter(|(_, _, i)| !matches!(i, Interp::Linear | Interp::Hold | Interp::Nearest))
                .count();
            report = soma(
                report,
                ConversionReport {
                    keys: samples.len(),
                    // Os dois eixos nascem com os MESMOS instantes: a trajetória não sabe
                    // expressar tempos independentes, então não há nada a desemparelhar.
                    unmatched_times: 0,
                    dropped_eases,
                },
            );
        }
        self.set_active(ativo);
        Some(report)
    }

    /// A trajetória de `entity`, se ele estiver em modo Path — o que o painel pergunta
    /// para saber **qual** conversão oferecer.
    #[must_use]
    pub fn position_path(&self, entity: u64) -> Option<&MotionPath> {
        let target = self.binding_for(entity, PropKind::Position)?.target;
        // A mesma porta de resolução do `path_to_separate`: o painel pergunta *este objeto
        // está em modo Path?*, que é sobre o que o avaliador faz, não sobre qual clip
        // desenhou a curva.
        self.path_for(target)
    }

    /// **Which mode a new position key for `entity` must use** ([ADR-0141]).
    ///
    /// The existing animation decides — a Position binding is Path, a
    /// `TranslationX`/`Y` binding is Separate — because the two modes cannot
    /// coexist, and keying the wrong one is precisely the *"even with points on the
    /// canvas the auto-key writes X/Y"* conflict. A fresh entity (no position
    /// animation yet) has no mode of its own, so it takes `default_path` (the
    /// After Effects default is Path; the per-object toggle marks it otherwise via
    /// [`Self::set_position_mode`]).
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

    /// **Switch `entity` to Path or Separate** — the door the transport toggle and
    /// the menu (*Convert to Motion Path* / *Convert to Separate Axes*) both drive
    /// ([ADR-0141]). Per object: two objects on the same timeline can sit in
    /// different modes, and this only touches `entity`.
    ///
    /// An object with animation is **converted** ([`Self::separate_to_path`] /
    /// [`Self::path_to_separate`], which carry the poses across and report what the
    /// conversion drops). A fresh one is **marked** — an empty binding in the target
    /// mode — so the choice sticks per object and the next `K`/auto-key authors in
    /// it. Either way it ends in exactly ONE mode: the two are mutually exclusive,
    /// and a stray binding of the other mode is cleared here.
    ///
    /// [ADR-0141]: ../../../docs/architecture/decisions/0141-timeline-position-is-one-2d-channel-and-separate-axes-are-a-mode.md
    ///
    /// `repouso` = o valor LOCAL do objeto no mundo, para o eixo sem binding nem repouso capturado
    /// ([`Self::separate_to_path_with_rest`]); o intent com mundo o lê do `Transform`.
    pub fn set_position_mode(&mut self, entity: u64, to_path: bool, repouso: [f32; 2]) {
        if to_path {
            // Converts if X/Y carried keys; a no-op (returns `None`) if they were
            // empty markers or absent. Either way, clear any X/Y binding left and
            // mark Path with an (empty, if fresh) Position binding.
            self.separate_to_path_with_rest(entity, repouso);
            self.unbind(entity, PropKind::TranslationX);
            self.unbind(entity, PropKind::TranslationY);
            self.bind(entity, PropKind::Position);
        } else {
            self.path_to_separate(entity);
            self.unbind(entity, PropKind::Position);
            self.bind(entity, PropKind::TranslationX);
            self.bind(entity, PropKind::TranslationY);
        }
    }
}

#[cfg(test)]
#[path = "path_convert_tests.rs"]
mod tests;

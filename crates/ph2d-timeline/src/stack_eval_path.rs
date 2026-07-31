//! **O que uma DISTÂNCIA vira antes de entrar no blend** — a metade de trajetória do
//! [`super`] ([ADR-0141]).
//!
//! Uma track de [`crate::PropKind::Position`] guarda a distância percorrida, e distância só
//! significa algo sobre UMA curva. Enquanto existia uma trajetória por documento isso era
//! invisível; com a trajetória por CLIP, cruzar dois strips passou a misturar números de
//! réguas diferentes — o report do Enio (2026-07-30): *"o Fade gera Path de transição entre
//! um path de uma strip e outro path de outra strip … o Fade precisa ser similar ao modo
//! sem Path"*.
//!
//! Então **cada strip converte a própria distância na PRÓPRIA curva antes de o blend somar
//! qualquer coisa**, e o que cruza é uma COORDENADA. O maquinário do irmão não muda uma
//! linha, e a razão é aritmética: a média por lane, o `lerp` de Override e a soma de
//! Additive são todos AFINS nos valores de origem, e aplicar um blend afim componente a
//! componente **É** blendar pontos.
//!
//! [ADR-0141]: ../../../docs/architecture/decisions/0141-timeline-position-is-one-2d-channel-and-separate-axes-are-a-mode.md

use ph2d_anim::AnimTarget;

use super::{Query, eval_frame};
use crate::doc::TimelineDoc;
use crate::frame_solve::LinkFrame;
use crate::stack_frames::StackScratch;

/// **A distância vira o PONTO na curva DESTE clip**, e devolve a componente `axis`.
///
/// A porta que torna o fade de Position um fade de PONTOS: cada strip resolve o próprio
/// número na própria geometria ANTES de o blend somar qualquer coisa. Um clip sem
/// trajetória própria cai na do documento ([`TimelineDoc::path_for`]) — é o mesmo recuo que
/// mantém a composição viva quando o clip ativo nunca autorou uma.
///
/// `None` quando não há trajetória alguma, ou quando a distância cai fora dela: o strip
/// então não contribui, que é o mesmo silêncio de uma track vazia.
pub(super) fn on_axis(
    doc: &TimelineDoc,
    clip: usize,
    target: AnimTarget,
    axis: u8,
    d: f64,
) -> Option<f64> {
    let path = doc
        .clip_path(clip, target)
        .or_else(|| doc.path_for(target))?;
    let p = path.at(d)?.point;
    Some(f64::from(p[usize::from(axis)]))
}

/// **A pose composta de um binding de trajetória — o PONTO**, blendado componente a
/// componente ([`Query::axis`]).
///
/// É a porta que o apply usa para Position sob a composição, e é o que torna o fade igual
/// ao do modo sem Path: cada strip resolve a própria distância na PRÓPRIA curva e o que
/// cruza é a coordenada.
///
/// O `rest` de cada eixo sai do MESMO lugar que o escalar dizia — a distância de repouso
/// projetada na trajetória do documento —, só que como ponto: é literalmente *"a pose
/// autorada, na curva"*, que é o que o campo sempre significou.
pub(crate) fn sample_stack_point(
    doc: &TimelineDoc,
    scratch: &StackScratch,
    q: Query,
    links: &LinkFrame,
) -> Option<[f32; 2]> {
    let rest_pt = doc
        .path_for(q.target)
        .and_then(|p| p.at(f64::from(q.rest)))
        .map_or([0.0_f32, 0.0], |s| s.point);
    let mut out = [0.0_f32; 2];
    for (ax, slot) in out.iter_mut().enumerate() {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "the scene value is f32; f64 is the blend's working precision"
        )]
        let v = eval_frame(
            doc,
            scratch,
            0,
            Query {
                rest: rest_pt[ax],
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "0 e 1, e o eixo é u8 por caber num Copy pequeno"
                )]
                axis: Some(ax as u8),
                ..q
            },
            None,
            links,
        )? as f32;
        *slot = v;
    }
    Some(out)
}

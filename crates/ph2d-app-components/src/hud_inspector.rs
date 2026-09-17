//! **O instantâneo do HUD e o dreno das edições dele** (TOP-20 #20).
//!
//! Irmão do [`crate::particles_inspector`]: a shell publica o que o painel mostra, e o painel
//! devolve edições que voltam por aqui.
//!
//! ⚠️ **Um instantâneo por quadro, e ele traz DOIS factos que não são campos:** se há câmera de
//! jogo (senão o canvas não se cola a nada) e o que o rótulo mostra AGORA. *Sem eles, um HUD que
//! está exactamente como o artista pediu lê-se como partido, com todos os números certos.*

use ph2d_ecs::{Counter, CounterRuntime, Entity, LabelSource, SimWorld, UiButton, UiCanvas, UiLabel};
use ph2d_editor_core::hud_edits::{
    HudFieldEdit as E, HudNumber as N, HudText as T, InspectorHudInfo,
};
use ph2d_tags::TagTree;

/// O índice da fonte, para o segmentado do painel.
fn indice_da_fonte(s: &LabelSource) -> u8 {
    match s {
        LabelSource::Authored => 0,
        LabelSource::Counter(_) => 1,
        LabelSource::TimerLeft(_) => 2,
        LabelSource::TagCount(_) => 3,
    }
}

/// O nome que a fonte carrega — vazio na autorada, que não tem nenhum.
fn nome_da_fonte(s: &LabelSource) -> &str {
    match s {
        LabelSource::Authored => "",
        LabelSource::Counter(n) | LabelSource::TimerLeft(n) | LabelSource::TagCount(n) => n,
    }
}

/// A fonte que este índice escolhe, **conservando o nome** que já lá estava.
///
/// ⚠️ **Trocar a fonte não apaga o nome**, e é isso que torna o segmentado reversível: quem
/// carrega em `Timer` por engano e volta a `Counter` encontra o que tinha escrito. ⛔ Apagá-lo
/// faria o engano custar a digitação.
fn fonte_do_indice(i: u8, nome: &str) -> LabelSource {
    match i {
        1 => LabelSource::Counter(nome.to_owned()),
        2 => LabelSource::TimerLeft(nome.to_owned()),
        3 => LabelSource::TagCount(nome.to_owned()),
        _ => LabelSource::Authored,
    }
}

/// **O que o painel mostra do HUD deste objecto**, ou `None` se ele não tiver nenhum dos quatro.
#[must_use]
pub fn build_info(
    sim: &mut SimWorld,
    tree: &TagTree,
    bits: u64,
    tem_camera: bool,
) -> Option<InspectorHudInfo> {
    let e = Entity::from_bits(bits);
    let canvas = sim.world().get::<UiCanvas>(e).copied();
    let label = sim.world().get::<UiLabel>(e).cloned();
    let button = sim.world().get::<UiButton>(e).cloned();
    let counter = sim.world().get::<Counter>(e).cloned();
    if canvas.is_none() && label.is_none() && button.is_none() && counter.is_none() {
        return None;
    }
    // ⭐ O que o rótulo mostra AGORA — a MESMA porta que o desenho usa, nunca uma segunda conta.
    let vivo = label
        .as_ref()
        .and_then(|l| ph2d_ecs::hud::texto(sim.world_mut(), tree, l))
        .unwrap_or_default();
    let valor = sim
        .world()
        .get::<CounterRuntime>(e)
        .map_or_else(|| counter.as_ref().map_or(0, |c| c.start), |r| r.value);
    Some(InspectorHudInfo {
        entity_bits: bits,
        has_canvas: canvas.is_some(),
        ref_w: canvas.map_or(0.0, |c| c.ref_w),
        ref_h: canvas.map_or(0.0, |c| c.ref_h),
        fit: canvas.map_or(0, |c| u8::from(c.fit == ph2d_ecs::Fit::Stretch)),
        tem_camera,
        has_label: label.is_some(),
        source: label.as_ref().map_or(0, |l| indice_da_fonte(&l.source)),
        source_name: label
            .as_ref()
            .map(|l| nome_da_fonte(&l.source).to_owned())
            .unwrap_or_default(),
        prefix: label.as_ref().map(|l| l.prefix.clone()).unwrap_or_default(),
        suffix: label.as_ref().map(|l| l.suffix.clone()).unwrap_or_default(),
        vivo,
        has_button: button.is_some(),
        signal: button.as_ref().map(|b| b.signal.clone()).unwrap_or_default(),
        disabled: button.as_ref().is_some_and(|b| b.disabled),
        has_counter: counter.is_some(),
        counter_name: counter.as_ref().map(|c| c.name.clone()).unwrap_or_default(),
        #[expect(clippy::cast_precision_loss, reason = "o painel fala em f32")]
        counter_start: counter.as_ref().map_or(0.0, |c| c.start as f32),
        counter_value: valor,
    })
}

/// **Uma edição do painel volta ao componente.** `true` = alguma coisa mudou.
///
/// ⚠️ **Uma edição de um bloco ausente é INERTE, não um erro**: o painel só pinta o que existe,
/// mas um clique a meio de uma troca de selecção pode aterrar depois de o componente sair.
pub fn apply(sim: &mut SimWorld, bits: u64, edit: &E) -> bool {
    let e = Entity::from_bits(bits);
    match edit {
        E::Number(n, v) => match n {
            N::RefWidth | N::RefHeight => {
                let Some(mut c) = sim.world_mut().get_mut::<UiCanvas>(e) else {
                    return false;
                };
                // ⚠️ **O clamp é do LADO do componente e não do painel:** a lei do
                // `ph2d_hud::Canvas::new` recusa um lado `0`, e deixar o zero entrar aqui poria o
                // canvas a conduzir com escala infinita até alguém reparar.
                let v = v.max(f32::MIN_POSITIVE);
                if *n == N::RefWidth {
                    c.ref_w = v;
                } else {
                    c.ref_h = v;
                }
                true
            }
            N::CounterStart => {
                let Some(mut c) = sim.world_mut().get_mut::<Counter>(e) else {
                    return false;
                };
                #[expect(clippy::cast_possible_truncation, reason = "um contador é inteiro")]
                let novo = v.round() as i64;
                c.start = novo;
                true
            }
        },
        E::Text(t, s) => match t {
            T::SourceName => {
                let Some(mut l) = sim.world_mut().get_mut::<UiLabel>(e) else {
                    return false;
                };
                let i = indice_da_fonte(&l.source);
                l.source = fonte_do_indice(i, s);
                true
            }
            T::Prefix | T::Suffix => {
                let Some(mut l) = sim.world_mut().get_mut::<UiLabel>(e) else {
                    return false;
                };
                if *t == T::Prefix {
                    l.prefix = s.clone();
                } else {
                    l.suffix = s.clone();
                }
                true
            }
            T::Signal => {
                let Some(mut b) = sim.world_mut().get_mut::<UiButton>(e) else {
                    return false;
                };
                b.signal = s.clone();
                true
            }
            T::CounterName => {
                let Some(mut c) = sim.world_mut().get_mut::<Counter>(e) else {
                    return false;
                };
                c.name = s.clone();
                true
            }
        },
        E::Fit(i) => {
            let Some(mut c) = sim.world_mut().get_mut::<UiCanvas>(e) else {
                return false;
            };
            c.fit = if *i == 1 {
                ph2d_ecs::Fit::Stretch
            } else {
                ph2d_ecs::Fit::Keep
            };
            true
        }
        E::Source(i) => {
            let Some(mut l) = sim.world_mut().get_mut::<UiLabel>(e) else {
                return false;
            };
            let nome = nome_da_fonte(&l.source).to_owned();
            l.source = fonte_do_indice(*i, &nome);
            true
        }
        E::Disabled(on) => {
            let Some(mut b) = sim.world_mut().get_mut::<UiButton>(e) else {
                return false;
            };
            b.disabled = *on;
            true
        }
    }
}

/// Todas as edições de um quadro. `true` = alguma mudou.
pub fn apply_all(sim: &mut SimWorld, edits: &[(u64, E)]) -> bool {
    let mut mudou = false;
    for (bits, edit) in edits {
        mudou |= apply(sim, *bits, edit);
    }
    mudou
}

#[cfg(test)]
#[path = "hud_inspector_tests.rs"]
mod tests;

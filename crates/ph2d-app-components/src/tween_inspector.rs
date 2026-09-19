//! **A secção TWEEN: o instantâneo e o dreno** (suplente #22).
//!
//! ⛔⛔ **Ele mora na CRATE DA FAMÍLIA e não na shell**, pela mesma razão que o irmão do raio a
//! trouxe para cá em 19/09: a catraca `the_shell_only_shrinks` está apertada, e a cura que ela
//! prescreve por escrito é **MOVER para `crates/ph2d-app-<família>`, nunca subir o número**.
//!
//! ⚠️ **As duas metades vivem juntas de propósito:** quem lê o mundo para o painel e quem escreve a
//! edição de volta fazem a MESMA tradução (o `Canal` ↔ a tag, o `Easing` ↔ o par família/modo), e
//! separá-los seria a porta pela qual as duas divergem.
//!
//! # ⭐⭐ O instantâneo lê a CENA, e não só o componente
//!
//! Duas colunas não vêm do `Tweens`: **há um timer neste índice?** e **o objecto tem um `Sprite`?**
//! São elas que fazem a queixa do painel poder dizer *«este tween não tem relógio»* em vez de o
//! artista descobrir por o objecto não se mexer — e nenhuma delas é um campo.

use ph2d_ecs::{Entity, Timers, Tweens, World};
use ph2d_editor_core::tween_edits::{InspectorTweenInfo, InspectorTweenRow, TweenFieldEdit};
use ph2d_render::Sprite;
use ph2d_tween::{AoAcabar, Canal, Tween};

/// **O instantâneo.** `None` para quem não tem o componente (ADR-0166).
#[must_use]
pub fn build_tween_info(
    world: &World,
    entity_bits: u64,
    selected_count: usize,
) -> Option<InspectorTweenInfo> {
    let e = Entity::from_bits(entity_bits);
    let tweens = world.get::<Tweens>(e)?;
    // ⭐⭐⭐ **A DURAÇÃO de cada relógio** — a coluna que não vem do componente, e que responde à
    // pergunta do dono no smoke de 19/09: *«onde selecciono o tempo?»*. ⚠️ Ela é lida por ÍNDICE,
    // que é a lei do módulo: o tween `i` corre no timer `i`.
    let relogios = world.get::<Timers>(e);
    let tem_sprite = world.get::<Sprite>(e).is_some();
    Some(InspectorTweenInfo {
        entity_bits,
        rows: tweens
            .0
            .iter()
            .enumerate()
            .map(|(i, t)| InspectorTweenRow {
                canal: t.canal.tag(),
                de: t.de,
                para: t.para,
                familia: familia_tag(t.easing.family),
                modo: modo_tag(t.easing.mode),
                ao_acabar: t.ao_acabar.tag(),
                duracao_us: relogios
                    .as_ref()
                    .and_then(|ts| ts.0.get(i))
                    .map(|t| t.duration_us),
            })
            .collect(),
        tem_sprite,
        selected_count,
    })
}

/// ⚠️ **A tag de uma família é a POSIÇÃO no `ALL` dela**, e a tradução vive aqui e em mais lado
/// nenhum — o `ph2d-anim` não declara uma `tag()` (as curvas dele viajam num `Easing` serializado,
/// nunca num inteiro), logo esta é a única ponte entre o motor e um chip do painel.
#[allow(clippy::cast_possible_truncation)]
fn familia_tag(f: ph2d_anim::EasingFamily) -> u8 {
    ph2d_anim::EasingFamily::ALL
        .iter()
        .position(|&x| x == f)
        .unwrap_or(0) as u8
}

#[allow(clippy::cast_possible_truncation)]
fn modo_tag(m: ph2d_anim::EasingMode) -> u8 {
    ph2d_anim::EasingMode::ALL
        .iter()
        .position(|&x| x == m)
        .unwrap_or(0) as u8
}

/// **O dreno.** `true` = alguma coisa mudou.
///
/// ⚠️ **Um índice fora da lista é um no-op silencioso**, e não um pânico: a lista pode ter encolhido
/// entre o quadro que pintou e o quadro que despacha.
pub fn apply_tween_edit(world: &mut World, entity_bits: u64, edit: &TweenFieldEdit) -> bool {
    let e = Entity::from_bits(entity_bits);
    let Some(mut tweens) = world.get_mut::<Tweens>(e) else {
        return false;
    };
    match edit {
        TweenFieldEdit::Add => {
            // ⚠️ **O tecto é o do modelo** — e o painel já esconde o `+` no limite, logo isto é a
            // segunda cerca, na porta que ESCREVE. *Uma cerca só na UI é uma cerca que o próximo
            // chamador contorna.*
            if tweens.0.len() >= ph2d_ecs::TWEENS_MAX {
                return false;
            }
            tweens.0.push(Tween::default());
            true
        }
        TweenFieldEdit::Remove(i) => {
            let i = *i as usize;
            if i >= tweens.0.len() {
                return false;
            }
            tweens.0.remove(i);
            true
        }
        TweenFieldEdit::Canal(i, tag) => campo(&mut tweens, *i, |t| {
            let novo = Canal::from_tag(*tag);
            if t.canal == novo {
                return false;
            }
            t.canal = novo;
            true
        }),
        TweenFieldEdit::De(i, c, v) => campo(&mut tweens, *i, |t| componente(&mut t.de, *c, *v)),
        TweenFieldEdit::Para(i, c, v) => {
            campo(&mut tweens, *i, |t| componente(&mut t.para, *c, *v))
        }
        TweenFieldEdit::Familia(i, tag) => campo(&mut tweens, *i, |t| {
            let Some(&f) = ph2d_anim::EasingFamily::ALL.get(*tag as usize) else {
                return false;
            };
            if t.easing.family == f {
                return false;
            }
            t.easing.family = f;
            true
        }),
        TweenFieldEdit::Modo(i, tag) => campo(&mut tweens, *i, |t| {
            let Some(&m) = ph2d_anim::EasingMode::ALL.get(*tag as usize) else {
                return false;
            };
            if t.easing.mode == m {
                return false;
            }
            t.easing.mode = m;
            true
        }),
        // ⭐⭐⭐ **O PRESET escreve DUAS coisas, e a segunda é o que o faz um clique.**
        //
        // ⚠️ Sem a duração, o artista fica com um *flash* de **um segundo** (o valor de fábrica do
        // timer) — oito vezes mais lento do que a coisa que ele pediu —, e lê isso como *«o preset
        // não funcionou»*. ⛔ E ela vai para o `Timers` do MESMO ÍNDICE, que é o relógio dele.
        TweenFieldEdit::Preset(i, tag) => {
            let p = ph2d_tween::Preset::from_tag(*tag);
            let mexeu = campo(&mut tweens, *i, |t| {
                let novo = p.tween();
                if *t == novo {
                    return false;
                }
                *t = novo;
                true
            });
            // ⚠️ **O `Tweens` tem de estar LARGADO antes de pegar no `Timers`** — dois `get_mut`
            // vivos sobre o mesmo `World` não compilam, e é o compilador a dizer que são dois
            // componentes. ⛔ **E isso NÃO se escreve com um `drop`**: um `Option<Mut<_>>` não tem
            // destrutor, logo o empréstimo acaba na ÚLTIMA LEITURA (o `campo` acima) e o `drop`
            // explícito é só ruído que o clippy recusa (`drop_non_drop`). *A cerca é a ordem das
            // linhas, e ela é verificada pelo compilador — não por um comentário.*
            let relogio = world
                .get_mut::<ph2d_ecs::Timers>(e)
                .and_then(|mut ts| {
                    let t = ts.0.get_mut(*i as usize)?;
                    (t.duration_us != p.duracao_us()).then(|| {
                        t.duration_us = p.duracao_us();
                    })
                })
                .is_some();
            mexeu || relogio
        }
        TweenFieldEdit::AoAcabar(i, tag) => campo(&mut tweens, *i, |t| {
            let novo = AoAcabar::from_tag(*tag);
            if t.ao_acabar == novo {
                return false;
            }
            t.ao_acabar = novo;
            true
        }),
    }
}

/// Aplica `f` ao tween `i`, se ele existir. ⚠️ **O `&mut` do componente só é tomado quando alguma
/// coisa muda** — o `bevy` marca a alteração no `deref_mut`, e um componente tocado a cada clique
/// inerte é ruído para quem lê `Changed<…>`.
fn campo(tweens: &mut Tweens, i: u8, f: impl FnOnce(&mut Tween) -> bool) -> bool {
    tweens.0.get_mut(i as usize).is_some_and(f)
}

/// ⚠️ **Escreve só quando MUDA**, e é o que impede um arrasto parado de marcar o componente.
fn componente(alvo: &mut [f32; 4], c: u8, v: f32) -> bool {
    let Some(slot) = alvo.get_mut(c as usize) else {
        return false;
    };
    if *slot == v {
        return false;
    }
    *slot = v;
    true
}

/// **Aplica as edições que o painel emitiu neste quadro.** `true` = o documento mudou.
///
/// ⚠️ Irmã das `apply_all` das outras secções desta crate — a shell chama UMA função por secção, e
/// o laço vive com quem sabe o que cada edição significa.
pub fn apply_all(sim: &mut ph2d_ecs::SimWorld, edits: &[(u64, TweenFieldEdit)]) -> bool {
    let mut mexeu = false;
    for (bits, edit) in edits {
        if apply_tween_edit(sim.world_mut(), *bits, edit) {
            mexeu = true;
        }
    }
    mexeu
}

#[cfg(test)]
#[path = "tween_inspector_tests.rs"]
mod tests;

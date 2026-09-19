//! **O instantâneo e o dreno da secção PATH FOLLOW** (suplente #23) — irmão do
//! [`crate::tween_inspector`], e pela mesma lei.
//!
//! # ⚠️ Três colunas NÃO vêm do componente, e são elas que fazem a queixa
//!
//! *Existe um objecto com aquele nome?* · *Ele tem forma desenhada?* · *Há relógio naquele índice, e
//! quanto ele dura?* — sem as três, um seguidor sem pista parece-se com um seguidor partido, e o
//! artista não sabe qual das quatro coisas resolver.
//!
//! ⛔ **E nenhuma delas pede a cena vectorial:** *ter uma forma* é ter um `VecPathRef`, e isso o
//! mundo responde. Pedir a `VecScene` aqui poria uma dependência de documento numa função cujo
//! trabalho é ler o MUNDO.

use ph2d_ecs::{Entity, PathFollow, StableId, Timers, World};
use ph2d_editor_core::path_follow_edits::{InspectorPathFollowInfo, PathFollowFieldEdit};

/// O instantâneo da secção. `None` = a entidade não tem o componente (ADR-0166).
#[must_use]
pub fn build_path_follow_info(
    world: &mut World,
    entity_bits: u64,
    selected_count: usize,
    clock_playing: bool,
) -> Option<InspectorPathFollowInfo> {
    let e = Entity::from_bits(entity_bits);
    let pf = world.get::<PathFollow>(e)?.clone();
    let i = usize::from(pf.relogio);
    // ⚠️ **As três colunas do relógio são COPIADAS aqui**, e não lidas por referência mais abaixo: a
    // busca por nome pede `&mut World` (ver o `resolve`), e um empréstimo imutável vivo ao lado
    // dela não compila. *O compilador impõe a ordem que a leitura já queria.*
    let (duracao_us, repeat, autostart, n_relogios) =
        world
            .get::<Timers>(e)
            .map_or((None, false, false, 0), |ts| {
                let t = ts.0.get(i);
                (
                    t.map(|t| t.duration_us),
                    t.is_some_and(|t| t.repeat),
                    t.is_some_and(|t| t.autostart),
                    ts.0.len(),
                )
            });
    // ⚠️ **A busca por NOME é `&mut World`** — o `stable_id_for_name` regista o componente de
    // identidade se o mundo ainda não o viu, que é a armadilha que a `tagged` pagou.
    let (nome_existe, nome_tem_forma) = resolve(world, &pf.caminho);
    Some(InspectorPathFollowInfo {
        entity_bits,
        caminho: pf.caminho,
        nome_existe,
        nome_tem_forma,
        relogio: pf.relogio,
        relogios: n_relogios,
        duracao_us,
        repeat,
        autostart,
        ciclo: pf.ciclo.tag(),
        familia: familia_tag(pf.easing.family),
        modo: modo_tag(pf.easing.mode),
        ao_acabar: pf.ao_acabar.tag(),
        deslocamento: pf.deslocamento,
        alinha: pf.alinha,
        angulo: pf.angulo,
        lado: pf.lado,
        clock_playing,
        selected_count,
    })
}

/// *«Existe alguém com este nome, e ele tem forma desenhada?»* — as duas metades da queixa.
fn resolve(world: &mut World, nome: &str) -> (bool, bool) {
    let nome = nome.trim();
    if nome.is_empty() {
        return (false, false);
    }
    let id = ph2d_ecs::stable_id_for_name(world, nome);
    let Some(e) = ph2d_ecs::entity_of_stable_id(world, StableId(id)) else {
        return (false, false);
    };
    (true, world.get::<ph2d_ecs::VecPathRef>(e).is_some())
}

/// ⚠️ **A tag de uma família é a POSIÇÃO no `ALL` dela** — a mesma ponte que o
/// [`crate::tween_inspector`] declara, e pela mesma razão (o `ph2d-anim` não tem `tag()`).
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
/// ⚠️ **Uma tag fora do `ALL` é um no-op silencioso**, e não um pânico: o painel e o motor podem
/// estar um quadro dessincronizados.
pub fn apply_path_follow_edit(
    world: &mut World,
    entity_bits: u64,
    edit: &PathFollowFieldEdit,
) -> bool {
    let e = Entity::from_bits(entity_bits);
    let Some(mut pf) = world.get_mut::<PathFollow>(e) else {
        return false;
    };
    match edit {
        PathFollowFieldEdit::Caminho(n) => {
            if pf.caminho == *n {
                return false;
            }
            pf.caminho.clone_from(n);
        }
        PathFollowFieldEdit::Relogio(n) => {
            // ⚠️ **O tecto é o do MODELO** — o relógio de um seguidor é o `Timers[i]`, e a lista
            // dele nunca passa do `TIMERS_MAX`. *Uma cerca só na UI é uma cerca que o próximo
            // chamador contorna.*
            #[allow(clippy::cast_possible_truncation)]
            let n = (*n).min((ph2d_ecs::TIMERS_MAX - 1) as u8);
            if pf.relogio == n {
                return false;
            }
            pf.relogio = n;
        }
        PathFollowFieldEdit::Ciclo(t) => {
            let Some(&c) = ph2d_tween::Ciclo::ALL.get(usize::from(*t)) else {
                return false;
            };
            pf.ciclo = c;
        }
        PathFollowFieldEdit::AoAcabar(t) => {
            let Some(&a) = ph2d_tween::AoAcabar::ALL.get(usize::from(*t)) else {
                return false;
            };
            pf.ao_acabar = a;
        }
        PathFollowFieldEdit::Familia(t) => {
            let Some(&f) = ph2d_anim::EasingFamily::ALL.get(usize::from(*t)) else {
                return false;
            };
            pf.easing.family = f;
        }
        PathFollowFieldEdit::Modo(t) => {
            let Some(&m) = ph2d_anim::EasingMode::ALL.get(usize::from(*t)) else {
                return false;
            };
            pf.easing.mode = m;
        }
        // ⚠️ **A fracção satura em `0..1`** — fora dela o `rem_euclid` da lei devolveria o mesmo
        // ponto por dois valores do campo, e o artista teria duas posições do slider com a mesma
        // saída.
        PathFollowFieldEdit::Deslocamento(v) => pf.deslocamento = v.clamp(0.0, 1.0),
        PathFollowFieldEdit::Alinha(b) => pf.alinha = *b,
        PathFollowFieldEdit::Angulo(v) => pf.angulo = *v,
        PathFollowFieldEdit::Lado(v) => pf.lado = *v,
    }
    true
}

/// **Aplica as edições que o painel emitiu neste quadro.** `true` = o documento mudou.
///
/// ⚠️ Irmã das `apply_all` das outras secções desta crate — a shell chama UMA função por secção, e
/// o laço vive com quem sabe o que cada edição significa.
pub fn apply_all(sim: &mut ph2d_ecs::SimWorld, edits: &[(u64, PathFollowFieldEdit)]) -> bool {
    let mut mexeu = false;
    for (bits, edit) in edits {
        if apply_path_follow_edit(sim.world_mut(), *bits, edit) {
            mexeu = true;
        }
    }
    mexeu
}

#[cfg(test)]
#[path = "path_follow_inspector_tests.rs"]
mod tests;

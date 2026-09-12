//! ⭐⭐⭐ **OS GESTOS DE DUAS MÃOS da secção *Prefab*** — o conta-gotas do *Swap* (F4.6c wave 2).
//!
//! # Porque é um ficheiro próprio
//!
//! O irmão [`super::tests`] mede *«o que a secção OFERECE»* e *«o que um clique FAZ»* sobre o
//! objecto na mão. Aqui o sujeito é **outro**: o gesto arma, e o segundo clique escolhe um alvo que
//! não estava seleccionado. ⚠️ E o corte não é de gosto — juntos, os dois passavam o tecto de LOC
//! do shell (`630` contra `600`), e a lei da casa é **decompor por responsabilidade, nunca subir a
//! allowlist**.
//!
//! Os arneses (`scene`, `run`, `run_full`, `master_id`) vivem no irmão e são partilhados: duplicá-los
//! daria duas fixturas a divergir no dia em que uma delas mudasse.

use super::state_of;
use super::tests::{master_id, run, run_full, scene};
use crate::vec_component_edit::ComponentEdit;
use ph2d_ecs::{Entity, Name, Transform};

/// ⭐⭐ **O botão do *Swap* ARMA o conta-gotas — ele não age e não fala.**
///
/// ⚠️ **`changed == false` aqui NÃO é um clique comido:** o gesto é de duas mãos, e a resposta ao
/// primeiro clique é o painel trocar de rótulo para *Click a copy of the prefab* (o `swap_armed`
/// shell publica). *Um toast a dizer «agora clique noutra coisa» seria a terceira maneira de dizer
/// o que o botão já diz.*
///
/// (Mutação: `*arm_pick = true` a virar `false` ⇒ RED.)
#[test]
fn the_swap_button_arms_the_eyedropper_instead_of_acting() {
    let (mut sim, r, _map, _id, e) = scene();
    let mut toasts = ph2d_editor_core::ToastQueue::default();
    let ((changed, _), armed_pick) = run_full(ComponentEdit::Swap, &mut sim, &r, e, &mut toasts);
    assert!(!changed, "o Swap mudou o mundo no primeiro clique");
    assert!(
        armed_pick,
        "o botao do Swap nao armou o conta-gotas — o gesto de duas maos nao chega a abrir"
    );
}

/// ⭐⭐ **E a secção DIZ que está à espera** — é o `swap_armed` que troca o rótulo do botão.
#[test]
fn the_section_says_the_eyedropper_is_waiting() {
    let (mut sim, _r, map, id, _e) = scene();
    let idle = state_of(&mut sim, &map, &[id], None, false).expect("a seccao existe");
    let waiting = state_of(&mut sim, &map, &[id], None, true).expect("a seccao existe");
    assert!(!idle.swap_armed && waiting.swap_armed);
}

/// ⭐⭐⭐ **O SEGUNDO clique troca o prefab da cópia — e o alvo é uma CÓPIA do prefab que se quer.**
///
/// ⚠️ **É a lei que devolveu o `Instantiate`, aplicada ao gesto:** a receita está escondida no
/// canvas, então clicar nela é impossível; clicar numa cópia dela quer dizer *«esta também passa a
/// ser um destes»*. A fixtura monta a família como o artista a monta — um prefab, e uma **variante**
/// dele — porque sem parentesco o mapa determinístico não existe e a troca recusa (por desenho).
///
/// (Mutação: `master_subject` a virar `clicked` ⇒ RED, porque a cópia clicada não é um `MasterRoot`.)
#[test]
fn the_second_click_makes_the_copy_a_copy_of_the_clicked_prefab() {
    let (mut sim, r, _map, _id, e) = scene();
    let mut toasts = ph2d_editor_core::ToastQueue::default();
    // A é a receita, `copy_a` é a cópia que fica no lugar.
    let (_, out) = run(ComponentEdit::Create, &mut sim, &r, e, &mut toasts);
    let copy_a = out.map(Entity::from_bits).expect("a copia de A");
    // Promover uma cópia faz uma VARIANTE — é assim que a família nasce.
    let (_, out) = run(ComponentEdit::Create, &mut sim, &r, copy_a, &mut toasts);
    let copy_b = out.map(Entity::from_bits).expect("a copia da variante B");
    let b_id = master_id(&mut sim, copy_b).expect("o elo da copia de B");
    // E uma segunda cópia de A, que é quem vai trocar.
    let (_, out) = run(ComponentEdit::Place, &mut sim, &r, e, &mut toasts);
    let mine = out.map(Entity::from_bits).expect("a segunda copia de A");
    assert_ne!(master_id(&mut sim, mine), Some(b_id), "ja' nasceu como B");

    let mut echo = ph2d_app_components::instance_sync::MasterEcho::default();
    let did = super::swap_by_pick(&mut sim, &mut echo, &mut toasts, mine, copy_b);

    assert!(did, "o segundo clique nao trocou nada");
    assert_eq!(
        master_id(&mut sim, mine),
        Some(b_id),
        "a copia continua a seguir a receita antiga — o alvo nao foi resolvido pela copia clicada"
    );
}

/// ⛔⛔ **E clicar numa forma comum RECUSA em voz alta, sem desarmar.**
///
/// *Um gesto de duas mãos que se desarma no primeiro clique fora do alvo faz o artista pensar que a
/// troca aconteceu* — a mesma lei que o motor velho já escrevia no comentário dele.
#[test]
fn clicking_a_plain_shape_with_the_eyedropper_refuses_out_loud() {
    let (mut sim, r, _map, _id, e) = scene();
    let mut toasts = ph2d_editor_core::ToastQueue::default();
    let (_, out) = run(ComponentEdit::Create, &mut sim, &r, e, &mut toasts);
    let copy = out.map(Entity::from_bits).expect("a copia");
    let plain = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Loose")))
        .id();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let before = master_id(&mut sim, copy);

    let mut echo = ph2d_app_components::instance_sync::MasterEcho::default();
    let did = super::swap_by_pick(&mut sim, &mut echo, &mut toasts, copy, plain);

    assert!(!did, "trocou por uma forma que nao e' prefab nenhum");
    assert_eq!(master_id(&mut sim, copy), before, "a copia mexeu-se");
    assert!(
        toasts.iter().any(|t| t.message.contains("not a copy")),
        "a recusa nao diz o que clicar"
    );
}

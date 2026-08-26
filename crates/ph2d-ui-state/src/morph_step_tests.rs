//! ⭐⭐⭐ **O conjunto de Morph States dentro de uma animação de States** (plano 32 W11c).
//!
//! Enio, 2026-08-26: *"Assegure-se que esse sistema de states em morph seja integrado e
//! completamente compatível com o sistema de States previamente existente, ou seja, que eu possa
//! usar o state morph nas animações criadas em States."*
//!
//! ⚠️ Estes gates são o irmão exacto do [`super::bool_morph_tests`], e a semelhança é o argumento:
//! *esta crate sabe de que estado para que estado e a que altura, e entrega isso a quem coze.*

use crate::{Machine, ObjectPose, StateRole, Transition, UiState};
use ph2d_anim::{Easing, EasingFamily, EasingMode};

const SET: u64 = 7;
const WIDE: u64 = 10;
const TALL: u64 = 20;

/// Uma pose sobre o conjunto `SET`, a mostrar `shape`.
fn pose(shape: Option<u64>) -> ObjectPose {
    let mut p = ObjectPose::new(SET);
    p.morph_shape = shape;
    p
}

/// ⭐⭐⭐ **UMA TROCA DE FORMA VIRA UM PASSO, com as duas pontas e o `t`.**
///
/// **Mutação que deve sangrar:** o `morph_steps` filtrar por `from.morph_shape == to.morph_shape`
/// (o inverso) — nenhuma troca seria publicada, e um botão que muda de forma no `Hover` ficaria
/// **parado** sem que nada explicasse.
#[test]
fn a_shape_change_between_two_states_becomes_a_step() {
    let tr = Transition::new(&[pose(Some(WIDE))], &[pose(Some(TALL))]);
    let steps = tr.morph_steps(0.25);
    assert_eq!(steps.len(), 1, "a troca de forma tem de virar UM passo");
    let s = steps[0];
    assert_eq!((s.id, s.from, s.to), (SET, WIDE, TALL));
    assert!((s.t - 0.25).abs() < 1e-9, "e o t viaja com ele");
}

/// ⛔ **AS PONTAS devolvem VAZIO** — nelas o desenho já é uma das duas formas.
///
/// ⚠️ Não é economia: publicar um passo em `t = 0` ou `t = 1` faria o quadro de chegada pagar um
/// casamento de formas para desenhar o que já estava na tela.
#[test]
fn the_ends_publish_nothing() {
    let tr = Transition::new(&[pose(Some(WIDE))], &[pose(Some(TALL))]);
    assert!(tr.morph_steps(0.0).is_empty());
    assert!(tr.morph_steps(1.0).is_empty());
    // E fora do intervalo também — o clamp é o mesmo do irmão.
    assert!(tr.morph_steps(-0.5).is_empty());
    assert!(tr.morph_steps(9.0).is_empty());
}

/// ⛔ **DUAS POSES NA MESMA FORMA não publicam passo nenhum.**
///
/// ⚠️ *Não animar* e *animar de x para x* são coisas diferentes: a segunda custaria um casamento
/// por quadro para não mover nada. É a lei que o `is_same_as` já escreve para a pose inteira.
#[test]
fn the_same_shape_on_both_sides_is_not_a_step() {
    let tr = Transition::new(&[pose(Some(WIDE))], &[pose(Some(WIDE))]);
    assert!(tr.morph_steps(0.5).is_empty());
}

/// ⛔⛔ **UM LADO SEM FORMA (`None`) NÃO ENTRA** — `None` é *«não me pronuncio»*.
///
/// ⚠️ Interpolar a partir dele obrigaria a inventar uma ponta, e o objecto saltaria para a primeira
/// forma da lista no dia em que alguém gravasse uma pose **antes** de ele ser um conjunto.
///
/// **Mutação que deve sangrar:** trocar os dois `?` por `unwrap_or_default()` — o `0` é um
/// `VecPathId` **válido** (a primeira forma de toda cena), então a pose antiga passaria a mandar o
/// conjunto para lá.
#[test]
fn a_side_without_a_shape_never_becomes_a_step() {
    assert!(
        Transition::new(&[pose(None)], &[pose(Some(TALL))])
            .morph_steps(0.5)
            .is_empty(),
        "a partida sem forma nao pode inventar uma ponta"
    );
    assert!(
        Transition::new(&[pose(Some(WIDE))], &[pose(None)])
            .morph_steps(0.5)
            .is_empty(),
        "nem a chegada"
    );
    // O CONTROLE POSITIVO: com as duas pontas, ele publica.
    assert_eq!(
        Transition::new(&[pose(Some(WIDE))], &[pose(Some(TALL))])
            .morph_steps(0.5)
            .len(),
        1
    );
}

/// ⭐ **A pose interpolada SEGURA na forma de PARTIDA até chegar** — a lei do `bool_op`.
///
/// ⚠️ Não há meio caminho entre duas formas *nesta lista*: um `VecPathId` interpolado entre dois
/// ids seria o id de uma **terceira** forma, ou de nenhuma. Quem desenha o meio é o motor do Morph,
/// pelo passo acima.
///
/// **Mutação que deve sangrar:** `at` interpolar o campo, ou saltar para o destino a meio.
#[test]
fn the_pose_holds_the_start_shape_until_it_arrives() {
    let tr = Transition::new(&[pose(Some(WIDE))], &[pose(Some(TALL))]);
    for t in [0.0, 0.25, 0.5, 0.99] {
        assert_eq!(
            tr.at(t)[0].morph_shape,
            Some(WIDE),
            "a meio ({t}) a pose tem de SEGURAR a forma de partida"
        );
    }
    assert_eq!(
        tr.at(1.0)[0].morph_shape,
        Some(TALL),
        "e so' na chegada ela e' a de destino"
    );
}

/// ⭐⭐⭐ **A MÁQUINA VIVA PUBLICA OS PASSOS enquanto anda** — a costura que faltava.
///
/// ⛔⛔ **Este gate nasceu de uma mutação que SOBREVIVEU** (2026-08-26): apagar
/// `self.morph_steps = f.tr.morph_steps(t)` do `advance` deixava a suíte inteira verde. A
/// compatibilidade toda podia estar **morta** e nada dizia — os gates do `Transition` provam que a
/// crate *sabe* calcular o passo, e nenhum provava que a máquina o **entrega**.
///
/// *Uma afirmação que mutação nenhuma mata é uma afirmação sobre nada.*
///
/// **Mutação que deve sangrar:** apagar aquela linha do `advance`.
#[test]
fn a_running_machine_publishes_the_morph_steps() {
    let mut default_st = UiState::new(StateRole::Default);
    default_st.objects = vec![pose(Some(WIDE))];
    let mut hover = UiState::new(StateRole::Hover);
    hover.objects = vec![pose(Some(TALL))];

    let mut m = Machine::new(vec![default_st, hover]).expect("dois estados");
    assert!(
        m.morph_steps().is_empty(),
        "uma maquina PARADA nao pode publicar recado nenhum"
    );
    m.go_to(1, 1.0, Easing::new(EasingFamily::Linear, EasingMode::InOut));
    // Meio caminho: a maquina TEM de estar a publicar o passo.
    m.advance(0.5);
    let steps = m.morph_steps();
    assert_eq!(
        steps.len(),
        1,
        "⛔ a maquina nao publicou passo nenhum -- a compatibilidade com o States esta' MORTA"
    );
    assert_eq!((steps[0].id, steps[0].from, steps[0].to), (SET, WIDE, TALL));

    // ⭐ E na CHEGADA ela cala-se: ali o desenho ja' e' a forma de destino.
    m.advance(1.0);
    assert!(
        m.morph_steps().is_empty(),
        "chegou -- publicar aqui faria o quadro de chegada pagar um casamento por nada"
    );
}

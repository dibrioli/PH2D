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
///
/// ⛔⛔ **A 1.ª redacção deste gate era VÁCUA** (2026-08-26): ela passava duas poses **idênticas**,
/// que o `Transition::new` descarta antes de chegar a um `Step` — o balde ficava vazio e o
/// `is_empty()` lia como *«a lei funciona»*. *Um zero de «não medido» e um de «correcto» são o mesmo
/// byte.* As duas pontas têm de diferir **noutra coisa**, para que o objecto ENTRE na transição e a
/// afirmação seja sobre o filtro do `morph_steps`.
///
/// **Mutação que deve sangrar:** o `morph_steps` largar o `from.morph_shape != to.morph_shape`.
#[test]
fn the_same_shape_on_both_sides_is_not_a_step() {
    let mut moved = pose(Some(WIDE));
    moved.translation = [40.0, 0.0];
    let tr = Transition::new(&[pose(Some(WIDE))], &[moved]);
    assert_eq!(tr.len(), 1, "a fixtura tem de por o objecto EM movimento");
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

/// ⭐⭐⭐ **A POSE É O ESCRITOR DE BASE: ela nomeia uma forma em TODO instante.**
///
/// ⛔⛔ **Este gate substitui um meu que afirmava o CONTRÁRIO** (W11d, revertido na W11e). Ele
/// exigia que a pose se CALASSE (`None`) no meio de um voo, para o `install` e o `apply_ui_steps`
/// não escreverem o mesmo campo no mesmo quadro. Duas coisas estavam erradas nisso:
///
/// 1. **`None` já tem dono como significado** — *«esta pose não se pronuncia»*, o caso de uma pose
///    gravada antes de o objecto ser um conjunto ([`a_side_without_a_shape_never_becomes_a_step`]).
///    Dar-lhe um segundo sentido (*«estou a meio»*) põe dois factos no mesmo valor.
/// 2. **A escrita dupla não é um defeito, é uma CAMADA:** o `install` da pose escreve a forma de
///    base e o `apply_ui_steps` refina-a com o `t`. Calar a base deixa o mundo com o valor do
///    quadro ANTERIOR sempre que o passo não fala — e ele não fala nas pontas, nem quando uma das
///    pontas é `None`, que é exactamente o estado em que uma **interrupção** deixa a máquina.
///
/// ⚠️ *Uma pose interpolada não é só o que se desenha agora: ela é o `from` da próxima transição*
/// (`Machine::go_to` faz `Transition::new(&self.live, ..)`). Um canal que se cala nela apaga-se do
/// futuro.
///
/// ⚠️ A forma é DISCRETA e segura em `from` até chegar — a lei do `bool_op`, e a mesma razão: um
/// `VecPathId` interpolado entre dois ids seria o de uma **terceira** forma, ou de nenhuma. ⛔ A
/// consequência é **nomeada em vez de descoberta**: interromper um voo a meio faz a forma saltar
/// para a de partida em vez de desmorfar, porque o par vivo `(A, B, t)` não cabe numa pose — que
/// carrega **uma** forma. Curá-lo é modelo novo, não um ajuste aqui.
///
/// **Mutação que deve sangrar:** a pose devolver `None` a meio do voo (o defeito), interpolar o
/// campo, ou saltar para o destino antes da chegada.
#[test]
fn the_pose_names_a_shape_at_every_instant_of_a_flight() {
    let mut rest = UiState::new(StateRole::Default);
    rest.objects = vec![pose(Some(WIDE))];
    let mut hover = UiState::new(StateRole::Hover);
    hover.objects = vec![pose(Some(TALL))];
    let mut m = Machine::new(vec![rest, hover]).expect("dois estados");
    let linear = Easing::new(EasingFamily::Linear, EasingMode::InOut);

    m.go_to(1, 1.0, linear);
    for _ in 0..9 {
        m.advance(0.1);
        assert!(
            m.pose()[0].morph_shape.is_some(),
            "a pose calou-se a meio do voo -- o `install` fica sem forma de base para escrever"
        );
    }
    assert_eq!(
        m.pose()[0].morph_shape,
        Some(WIDE),
        "e a forma que ela nomeia e' a de PARTIDA (discreta), nunca uma terceira"
    );
    m.advance(1.0);
    assert_eq!(
        m.pose()[0].morph_shape,
        Some(TALL),
        "so' na chegada ela passa a nomear o destino"
    );

    // ⭐ E a INTERRUPÇÃO: a transição de volta nasce da pose viva, que tem de nomear uma forma —
    // senão `Transition::new` recebe um `None` e o `install` fica mudo o voo inteiro.
    m.go_to(1, 1.0, linear);
    m.advance(0.4);
    m.go_to(0, 1.0, linear);
    m.advance(0.3);
    assert!(
        m.pose()[0].morph_shape.is_some(),
        "⛔ a transicao de volta nasceu de uma pose MUDA: o mundo fica com o valor do quadro anterior"
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

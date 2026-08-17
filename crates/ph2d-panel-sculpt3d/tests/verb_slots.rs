//! **CADA FERRAMENTA LEMBRA A PRÓPRIA AFINAÇÃO** — a porta de troca de verbo, e
//! que nada atravessa a fronteira entre duas delas.
//!
//! ⚠️ **Este arquivo era o `arm_verb.rs`, e o assunto dele INVERTEU** (ordem do
//! Enio, 2026-08-17: *"as configurações dos parâmetros de cada tool não devem se
//! propagar para outra tool"*). A porta antiga (`arm_verb_defaults`) levava o
//! pincel VIVO para o verbo novo e re-armava campo a campo *"se o artista ainda
//! não mexeu"* — uma heurística que só podia acertar enquanto o artista não
//! mexesse em nada, e cujo preço estava medido no report: afinar a força do
//! Smooth e pegar o Clay levava a força do Smooth junto.
//!
//! ⚠️ **Os gates de então testavam a heurística, então eles não foram
//! recalibrados — eles foram SUBSTITUÍDOS.** O que sobrevive deles é a metade
//! que continua verdadeira: *um verbo que o artista nunca tocou veste a
//! referência dele*. O resto afirma a lei nova.

use ph2d_panel_sculpt3d::state::{BASE_RADIUS_PX, Sculpt3dUi, switch_verb};
use ph2d_sculpt3d::Verb;

/// **Um verbo NUNCA TOCADO veste a referência dele** — os quatro campos, num
/// gate só, porque eles são UMA decisão.
///
/// Os números são os do `Crease.js` do SculptGL: força `0,75`, raio `25` (a
/// METADE do resto — um vinco é fino por definição), accumulate desarmado, e a
/// quártica que toda tool de geometria dele compartilha.
///
/// ⚠️ **Isto sobrevive à lei nova por outro MECANISMO, e a distinção importa:**
/// antes o valor chegava porque a troca o *armava*; agora ele chega porque o
/// slot **nasce** com ele ([`ph2d_panel_sculpt3d::state::VerbSlot::for_verb`]) e
/// a troca apenas o carrega. O número é o mesmo e a razão não.
#[test]
fn a_verb_the_artist_never_touched_wears_its_reference() {
    let mut ui = Sculpt3dUi::default();
    assert_eq!(ui.brush.verb, Verb::Draw, "a fixture parte do de fábrica");

    switch_verb(&mut ui, Verb::Crease);

    assert_eq!(ui.brush.verb, Verb::Crease);
    assert!(
        (ui.brush.strength - 0.75).abs() < 1e-6,
        "força: Crease.js:10 diz 0,75, veio {}",
        ui.brush.strength
    );
    assert!(
        (ui.radius_px - BASE_RADIUS_PX * 0.5).abs() < 1e-6,
        "raio: Crease.js:9 diz 25 contra a base 50 — METADE; veio {}",
        ui.radius_px
    );
    assert!(!ui.brush.accumulate, "Crease.js não declara accumulate");
    assert_eq!(
        ui.brush.falloff,
        Verb::Crease.default_falloff(ui.brush.mode),
        "a CURVA é a do verbo"
    );
}

/// ⚠️ **O RAIO viaja com a ferramenta, e ele é o campo que mais fácil se perde**
/// — ele mora no [`Sculpt3dUi`] e não no `Brush` (é a régua da TELA, o outro é
/// derivado de mundo por dab), então uma tabela que guardasse só o pincel
/// faria a ferramenta lembrar a força e esquecer o próprio tamanho.
///
/// **O raio do Move é o TRIPLO**: um puxão é um gesto de REGIÃO, e um Move com
/// raio de pincel de detalhe é o que faz um artista concluir que a ferramenta
/// não funciona.
#[test]
fn the_grab_is_born_three_times_wider_and_the_radius_travels_with_it() {
    let mut ui = Sculpt3dUi::default();
    switch_verb(&mut ui, Verb::Move);
    assert!(
        (ui.radius_px - BASE_RADIUS_PX * 3.0).abs() < 1e-6,
        "Move.js:10 diz 150 contra a base 50; veio {}",
        ui.radius_px
    );

    // E ele volta: o tamanho é da FERRAMENTA.
    ui.radius_px = 88.0;
    switch_verb(&mut ui, Verb::Draw);
    switch_verb(&mut ui, Verb::Move);
    assert!(
        (ui.radius_px - 88.0).abs() < 1e-6,
        "o Move esqueceu o próprio tamanho; veio {}",
        ui.radius_px
    );
}

/// ⚠️ **A LEI DA WAVE, metade um: SAIR E VOLTAR DEVOLVE A AFINAÇÃO.**
///
/// A fixture mexe em tudo com valores que **não são o default de verbo nenhum**
/// — senão *"lembrou"* e *"re-armou de fábrica"* seriam indistinguíveis, que é
/// exatamente como a heurística antiga passava por gates que pareciam prová-la.
#[test]
fn leaving_a_verb_and_coming_back_restores_what_the_artist_tuned() {
    let mut ui = Sculpt3dUi::default();
    switch_verb(&mut ui, Verb::Smooth);

    ui.brush.strength = 0.37;
    ui.radius_px = 123.0;
    ui.brush.falloff = ph2d_sculpt3d::Falloff::Root;
    ui.brush.hardness = 0.61;
    ui.brush.auto_smooth = 0.29;
    // O accumulate é booleano, então *"mexido"* é ele estar no OPOSTO do que o
    // verbo declara — não há terceiro valor a escolher.
    let acc = !Verb::Smooth.default_accumulate();
    ui.brush.accumulate = acc;

    switch_verb(&mut ui, Verb::Clay);
    switch_verb(&mut ui, Verb::Smooth);

    assert!((ui.brush.strength - 0.37).abs() < 1e-6, "força");
    assert!((ui.radius_px - 123.0).abs() < 1e-6, "raio");
    assert_eq!(ui.brush.falloff, ph2d_sculpt3d::Falloff::Root, "curva");
    assert!((ui.brush.hardness - 0.61).abs() < 1e-6, "dureza");
    assert!((ui.brush.auto_smooth - 0.29).abs() < 1e-6, "alisamento");
    assert_eq!(ui.brush.accumulate, acc, "accumulate");
}

/// ⚠️ **A LEI DA WAVE, metade dois: NADA ATRAVESSA A FRONTEIRA.**
///
/// É o report do Enio ao pé da letra. O gate irmão acima **não o implica**: uma
/// tabela poderia lembrar corretamente e ainda assim ter carregado a força do
/// Smooth para dentro do Clay no caminho.
///
/// ⚠️ **A fixture mede contra o DEFAULT do verbo que entra**, e não contra um
/// literal: o número certo é *"o que o Clay teria se ninguém o tivesse tocado"*,
/// e escrevê-lo à mão o congelaria no dia em que a referência mudasse.
#[test]
fn tuning_one_verb_does_not_reach_another() {
    let mut ui = Sculpt3dUi::default();
    switch_verb(&mut ui, Verb::Smooth);
    ui.brush.strength = 0.37;
    ui.radius_px = 123.0;
    ui.brush.hardness = 0.61;

    switch_verb(&mut ui, Verb::Clay);

    assert!(
        (ui.brush.strength - Verb::Clay.default_strength()).abs() < 1e-6,
        "o Clay recebeu a força do Smooth: {} contra os {} dele",
        ui.brush.strength,
        Verb::Clay.default_strength()
    );
    assert!(
        (ui.radius_px - Verb::Clay.default_radius_px(BASE_RADIUS_PX)).abs() < 1e-6,
        "o Clay recebeu o raio do Smooth; veio {}",
        ui.radius_px
    );
    assert!(
        (ui.brush.hardness - Verb::Clay.default_hardness()).abs() < 1e-6,
        "o Clay recebeu a dureza do Smooth; veio {}",
        ui.brush.hardness
    );
}

/// **Duas trocas seguidas** — o pincel que entra é o do slot, não o do meio do
/// caminho.
///
/// ⚠️ Com uma troca só, *"carreguei o slot"* e *"parei no verbo anterior"*
/// coincidem em qualquer campo que os dois verbos declarem igual; com duas, o
/// valor do meio é um terceiro número e a confusão fica visível.
#[test]
fn two_switches_in_a_row_land_on_the_slot_of_the_last_one() {
    let mut ui = Sculpt3dUi::default();
    switch_verb(&mut ui, Verb::Inflate);
    assert!(
        (ui.brush.strength - 0.3).abs() < 1e-6,
        "Inflate.js:10 diz 0,3 — o mais fraco do catálogo"
    );
    switch_verb(&mut ui, Verb::Mask);
    assert!(
        (ui.brush.strength - 1.0).abs() < 1e-6,
        "Masking.js:15 diz força CHEIA; veio {}",
        ui.brush.strength
    );
}

/// ⚠️ **Trocar para o verbo que já está em mãos é NO-OP** — e não por higiene:
/// sem a guarda, o gesto salva o vivo no próprio slot e o recarrega, o que é
/// inofensivo *hoje* e é a linha exata onde um caso especial futuro (um slot
/// escrito sob condição) passaria a apagar o estado vivo.
#[test]
fn switching_to_the_verb_already_in_hand_changes_nothing() {
    let mut ui = Sculpt3dUi::default();
    switch_verb(&mut ui, Verb::Clay);
    ui.brush.strength = 0.42;
    ui.radius_px = 77.0;
    let before = ui.clone();

    switch_verb(&mut ui, Verb::Clay);

    assert_eq!(before, ui, "a troca para o mesmo verbo mexeu no estado");
}

/// **A DEMÃO NASCE COM OS NÚMEROS QUE O ENIO APROVOU** — Strength `0,7`,
/// Hardness `0,4`, Auto Smooth `0,0` (smoke de 2026-08-17).
///
/// ⚠️ **O gate lê as portas, não os literais** — `0,7` escrito aqui e no
/// `default_strength` são duas cópias do mesmo número, e a segunda envelhece
/// sozinha. O que ele afirma é que *o slot de fábrica da demão É o que as portas
/// declaram*, mais o **CONTROLE** de que elas não são o neutro de todo mundo
/// (senão a asserção passaria sobre uma tabela que não decidiu nada).
#[test]
fn the_layer_is_born_with_the_numbers_the_smoke_approved() {
    let mut ui = Sculpt3dUi::default();
    switch_verb(&mut ui, Verb::Layer);

    assert!(
        (ui.brush.strength - Verb::Layer.default_strength()).abs() < 1e-6,
        "força"
    );
    assert!(
        (ui.brush.hardness - Verb::Layer.default_hardness()).abs() < 1e-6,
        "dureza"
    );
    assert!(
        (ui.brush.auto_smooth - Verb::Layer.default_auto_smooth()).abs() < 1e-6,
        "alisamento"
    );

    // CONTROLE: os dois primeiros TÊM de diferir do que o resto do catálogo
    // usa, senão este gate passa sobre uma tabela sem entrada nenhuma.
    assert!(
        (Verb::Layer.default_strength() - Verb::Smooth.default_strength()).abs() > 1e-6,
        "a força da demão colapsou no default genérico"
    );
    assert!(
        Verb::Layer.default_hardness() > 0.0,
        "a dureza da demão colapsou no neutro do apply_hardness_to_distances"
    );
    // E o alisamento é ZERO de propósito — o neutro do Blender.
    assert!(
        Verb::Layer.default_auto_smooth().abs() < 1e-6,
        "o alisamento da demão deixou de ser o neutro sem ninguém medir"
    );
}

/// ⚠️ **CARIMBAR UMA REFERÊNCIA NUM SLOT RE-RESOLVE A CURVA DELE** — e este gate
/// existe porque a wave dos slots INTRODUZIU o defeito que ele apanha.
///
/// Enquanto a tabela guardava só o modo (`mode_by_verb`) não havia nada
/// por-verbo que pudesse envelhecer. O slot passou a guardar o **pincel
/// inteiro**, e aí escrever o modo sem re-resolver deixa o `falloff` daquele
/// verbo apontado para a referência ANTERIOR: o artista carimba `B` em todos,
/// pega um, e recebe a quártica do SculptGL sob um chip que diz Blender.
///
/// ⚠️ **Quem o apanhou não fui eu — foi o `arch_mode_has_reconcile`**, que
/// exige que todo `set_*_mode` reconcilie o estado a jusante ou se declare
/// benigno. Ele nomeia a classe (*"o setter escreveu um campo e ignorou o que o
/// modo invalida"*) e o setter novo caía nela no mesmo commit em que nasceu.
#[test]
fn stamping_a_mode_onto_a_slot_re_resolves_that_slots_curve() {
    // O Draw declara `S` e `B`, e as curvas de fábrica dos dois DIFEREM — sem
    // isso o gate passaria sobre qualquer coisa.
    let a = Verb::Draw.default_falloff(ph2d_sculpt3d::RefMode::S);
    let b = Verb::Draw.default_falloff(ph2d_sculpt3d::RefMode::B);
    assert_ne!(a, b, "a fixture só discrimina com duas curvas diferentes");

    // O Draw NÃO é o verbo vivo (o Smooth é), então o alvo é o SLOT dele.
    let mut ui = Sculpt3dUi::default();
    switch_verb(&mut ui, Verb::Smooth);
    assert_eq!(ui.mode_of(Verb::Draw), ph2d_sculpt3d::RefMode::S);

    ui.set_mode_of(Verb::Draw, ph2d_sculpt3d::RefMode::B);

    assert_eq!(ui.mode_of(Verb::Draw), ph2d_sculpt3d::RefMode::B, "o modo");
    switch_verb(&mut ui, Verb::Draw);
    assert_eq!(
        ui.brush.falloff, b,
        "o slot guardou o modo novo e a curva VELHA — o chip diria Blender e o \
         kernel rodaria a quártica do SculptGL"
    );
}

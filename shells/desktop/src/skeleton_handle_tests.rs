//! Os gates da **ALÇA da âncora** — o que o dedo APONTA, e qual verbo ele vai executar.
//!
//! ⚠️ **Corte por RESPONSABILIDADE** (o teto de 600 LOC do HR-18 pediu-o a `718`), e é o mesmo
//! corte que o `bone_gesture_tests`/`bone_pose_tests` já pagou: o irmão [`super`] mede a **LEI** de
//! uma restrição (ela nasce sem mover, persiste, a mistura, o laço, a pré-visualização); aqui
//! mede-se o **alvo do dedo** — o anel, o miolo, e o que acontece quando dois alvos ficam
//! concêntricos.
//!
//! ⛔ **A pergunta que este ficheiro existe para responder** veio de um report do dono (2026-09-07):
//! *«quando colocamos um IK num bone no meio dos ossos, o losango do IK e o círculo do outro osso
//! ficam sobrepostos»*. Alvos concêntricos com verbos diferentes só se separam por **tamanho**.

use super::*;

/// ⭐⭐ **O LOSANGO SUBSTITUI O ANEL, e não se soma a ele** — um ponto, um verbo.
///
/// ⚠️ Duas alças desenhadas por cima uma da outra prometeriam dois verbos onde há um, e o dedo
/// escolheria pela ordem em que as perguntas correm — que é a forma de defeito que este módulo já
/// pagou com o anel do objecto vazio por cima da bolinha da junta.
#[test]
fn an_anchored_end_stops_showing_the_plain_tip_ring() {
    let (mut sim, [_, cotovelo]) = braco();
    assert_eq!(
        unanchored_ends(&sim),
        vec![cotovelo.to_bits()],
        "sem ancora, a ponta mostra o anel"
    );
    add(&mut sim, cotovelo).expect("a ancora");
    assert!(
        unanchored_ends(&sim).is_empty(),
        "a ponta ancorada continua a mostrar o anel - dois verbos no mesmo ponto"
    );
    assert_eq!(anchors(&sim).len(), 1, "e o losango passa a existir");
}

/// **A CORRENTE tem o comprimento que a âncora pede** — o *Chain Length* do Blender, e a razão de
/// o valor de nascimento ser `2` e não `0`.
#[test]
fn the_chain_length_decides_how_many_bones_bend() {
    let (mut sim, [ombro, cotovelo]) = braco();
    let punho = osso(&mut sim, "Wrist", [10.0, 0.0], 10.0, Some(cotovelo));
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    assert_eq!(governed(&sim, punho, 0).len(), 3, "0 = ate' a raiz");
    assert_eq!(
        governed(&sim, punho, 2),
        vec![cotovelo, punho],
        "os DOIS de baixo"
    );
    assert_eq!(governed(&sim, punho, 1), vec![punho], "so' a ponta");
    assert_eq!(
        governed(&sim, punho, 99).len(),
        3,
        "um numero absurdo e' aparado pela ARVORE, nao por uma constante"
    );
    let _ = ombro;
}
/// ⛔⛔ **O ALVO NÃO GANHA O ANEL DE OBJECTO VAZIO** — ele tem alça própria.
///
/// ⚠️ Um alvo é um objecto com `Transform` e sem pixels, então ele responde *sim* à pergunta do
/// objecto vazio **por construção**. Sem a marca, ele nasceria com um segundo anel concêntrico com
/// o losango — e o disco desse anel **disputaria o clique** com a alça que arrasta a corrente
/// inteira. É o report de 06/09 (*«alguns bones têm círculos grandes e pequenos»*) outra vez, e o
/// doc daquela cura já tinha escrito a lei.
#[test]
fn the_anchor_object_does_not_get_a_second_ring() {
    let (mut sim, [_, cotovelo]) = braco();
    let alvo = add(&mut sim, cotovelo).expect("a ancora");
    assert!(
        !crate::group_gizmo_view::is_empty_object(&sim, alvo),
        "o alvo respondeu 'sou um objecto vazio' - ele ganha um 2.o anel E um disco a disputar o \
         clique com a propria alca"
    );
    // A metade que prova que a fixtura produz o fenómeno: sem a marca, ele responderia sim.
    sim.world_mut()
        .entity_mut(alvo)
        .remove::<ph2d_skeleton_ecs::IkTarget>();
    assert!(
        crate::group_gizmo_view::is_empty_object(&sim, alvo),
        "a fixtura nao produz o fenomeno: sem a marca o alvo ja' nao era um objecto vazio, entao o \
         gate acima estava verde por outro motivo"
    );
}

/// ⭐⭐ **O QUE A RESTRIÇÃO ESCREVE É PRÉ-VISUALIZAÇÃO** — o documento é a pose da ÂNCORA.
///
/// ⚠️ Sem isto, cada clique com a âncora fora do sítio empilharia um passo de undo cujo conteúdo é
/// *«o solver mexeu»* — vinte cliques, vinte Ctrl+Z mudos, que é o defeito que a auditoria da §11
/// mediu e que o `preview_drive` existe para curar.
#[test]
fn the_pose_the_constraint_writes_is_preview_not_document() {
    let (mut sim, [ombro, cotovelo]) = braco();
    add(&mut sim, cotovelo).expect("a ancora");
    let autorada = *sim.world().get::<Transform>(ombro).expect("t");
    let mut pv = PreviewDrive::default();
    drag_anchor(&mut sim, cotovelo, [4.0, 9.0]);
    solve(&mut sim, &mut pv);
    assert!(
        sim.world().get::<Transform>(ombro).expect("t").rotation != autorada.rotation,
        "a fixtura nao produz o fenomeno: a restricao nao mexeu no ombro"
    );
    // A fotografia repõe o autorado…
    let vivo = pv.substitute_authored(&mut sim);
    assert_eq!(
        sim.world().get::<Transform>(ombro).expect("t").rotation,
        autorada.rotation,
        "a captura viu a pose CONDUZIDA - ela iria para o undo e para o save"
    );
    // …e devolve o vivo logo a seguir, senão o artista veria o braco saltar para tras.
    crate::preview_drive::PreviewDrive::restore_live(&mut sim, &vivo);
    assert!(
        sim.world().get::<Transform>(ombro).expect("t").rotation != autorada.rotation,
        "o vivo nao voltou"
    );
}
/// ⭐⭐⭐ **A ÂNCORA É AGARRÁVEL LONGE DE TODO OSSO** — a razão de o dedo a procurar ANTES do osso.
///
/// ⚠️ O `hit` só devolve um osso quando o ponteiro está a `BONE_HIT_PX` do **segmento**. Uma âncora
/// arrastada para fora de alcance fica longe de tudo, e testá-la depois do osso tornaria a alça
/// inalcançável exactamente no estado em que ela mais se distingue da ponta.
#[test]
fn the_anchor_is_grabbable_far_away_from_every_bone() {
    let (mut sim, [_, cotovelo]) = braco();
    add(&mut sim, cotovelo).expect("a ancora");
    // Bem para fora do alcance (a corrente mede 20) e longe de todo segmento.
    let longe = [80.0, 60.0];
    drag_anchor(&mut sim, cotovelo, longe);
    let mut pv = PreviewDrive::default();
    solve(&mut sim, &mut pv);
    assert!(
        crate::bone_gesture::hit(&sim, longe, 1.0).is_none(),
        "a fixtura nao produz o fenomeno: ha' um osso debaixo do ponteiro"
    );
    let h = crate::bone_gesture::hover(&sim, longe, 1.0, None).expect("a ancora tem de ser achada");
    assert_eq!(h.bone, cotovelo.to_bits());
    assert_eq!(h.part, ph2d_skeleton_render::BonePart::Tip);
}

/// ⭐⭐⭐ **UMA ÂNCORA NO MEIO DA CORRENTE: por FORA pega ela, por DENTRO pega o osso** (report do
/// dono, 2026-09-07: *«o losango do IK e o círculo do outro osso ficam sobrepostos»*).
///
/// ⛔ Uma âncora criada num osso do meio nasce **exactamente** sobre a junta do osso seguinte — dois
/// alvos concêntricos com verbos diferentes, e o dedo não tinha como escolher. A cura é a que ele
/// propôs, e é a única que serve para alvos concêntricos: **eles diferem em TAMANHO**, e o anel
/// entre os dois é a zona exclusiva da âncora.
#[test]
fn a_middle_anchor_takes_the_ring_and_the_bone_keeps_the_core() {
    let (mut sim, [ombro, cotovelo]) = braco();
    // A âncora no OMBRO: o alvo nasce na ponta dele, que é a origem do cotovelo.
    add(&mut sim, ombro).expect("a ancora");
    let (_, ancora, o, p) = crate::skeleton_goal::anchors(&sim)[0];
    let junta = crate::bone_gesture::test_segment(&sim, cotovelo.to_bits()).0;
    assert!(
        (ancora[0] - junta[0]).hypot(ancora[1] - junta[1]) < 1e-9,
        "a fixtura nao produz o fenomeno: a ancora nao caiu sobre a junta do osso seguinte"
    );
    let comp = (p[0] - o[0]).hypot(p[1] - o[1]);
    let miolo = ph2d_skeleton_render::joint_radius_px(comp);
    let anel = ph2d_skeleton_render::goal_radius_px(comp);
    assert!(anel > miolo, "o losango tem de ser MAIOR que a bolinha");
    // No MIOLO: o osso seguinte, com o verbo de deslocar.
    let dentro = crate::bone_gesture::hover(&sim, ancora, 1.0, None).expect("algo sob o dedo");
    assert_eq!(
        dentro.bone,
        cotovelo.to_bits(),
        "o centro tem de pegar o OSSO"
    );
    assert_eq!(dentro.part, ph2d_skeleton_render::BonePart::Joint);
    // No ANEL: a âncora.
    let no_anel = [ancora[0] + (miolo + anel) * 0.5, ancora[1]];
    let fora = crate::bone_gesture::hover(&sim, no_anel, 1.0, None).expect("algo sob o dedo");
    assert_eq!(fora.bone, ombro.to_bits(), "o anel tem de pegar a ANCORA");
    assert_eq!(fora.part, ph2d_skeleton_render::BonePart::Tip);
}

/// ⚠️ **E o que ele devolve é o AUTORADO, não a pose de repouso** — se o artista girou o ombro à
/// mão antes de criar a âncora, é ESSA pose que volta.
///
/// ⛔ Um gate que só medisse *«voltou ao que estava antes de arrastar»* ficaria verde sobre uma
/// implementação que endireitasse a corrente, e o artista perderia trabalho.
#[test]
fn what_comes_back_is_the_authored_pose_not_the_rest_pose() {
    let (mut sim, [ombro, cotovelo]) = braco();
    // O artista posa o ombro À MÃO, e só então cria a âncora.
    sim.world_mut()
        .get_mut::<Transform>(ombro)
        .expect("t")
        .rotation = 0.4;
    add(&mut sim, cotovelo).expect("a ancora");
    let autorada = crate::skeleton_live::bone_segments(&sim);
    let mut pv = PreviewDrive::default();
    drag_anchor(&mut sim, cotovelo, [2.0, 12.0]);
    solve(&mut sim, &mut pv);
    remove(&mut sim, cotovelo, &mut pv);
    // ⚠️ A comparação é em `f32`, que é o tipo em que a pose VIVE: alargá-la para `f64` e comparar
    // com o literal `0.4` mede o arredondamento do `f32`, não o produto.
    assert_eq!(
        sim.world().get::<Transform>(ombro).expect("t").rotation,
        0.4_f32,
        "o ombro voltou ao repouso em vez da pose que o artista tinha feito"
    );
    assert_eq!(crate::skeleton_live::bone_segments(&sim), autorada);
}
/// ⚠️ **E longe de tudo o losango é um DISCO inteiro** — sem osso por baixo, o miolo não pode ser um
/// buraco morto.
#[test]
fn an_anchor_far_from_every_bone_is_grabbable_at_its_centre() {
    let (mut sim, [_, cotovelo]) = braco();
    add(&mut sim, cotovelo).expect("a ancora");
    let longe = [90.0, 70.0];
    drag_anchor(&mut sim, cotovelo, longe);
    let h = crate::bone_gesture::hover(&sim, longe, 1.0, None).expect("o centro do losango");
    assert_eq!(h.bone, cotovelo.to_bits());
    assert_eq!(h.part, ph2d_skeleton_render::BonePart::Tip);
}

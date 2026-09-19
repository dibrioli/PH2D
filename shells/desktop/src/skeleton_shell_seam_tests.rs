//! **As duas metades do esqueleto cujo SUJEITO é a SHELL** — e só elas.
//!
//! ⛔⛔ A família saiu para [`ph2d_app_skeleton`] na Fase C da W2. Estes dois gates **não foram
//! com ela**, e a régua é o HOWTO §2.6: *o teste segue o sujeito, nunca o ficheiro.*
//!
//! | gate | o que ele mede | de quem é |
//! |---|---|---|
//! | `probe_does_the_anchor_cross_the_undo_capture` | se a âncora de IK atravessa a `crate::undo::ProjectState::capture` | **da shell** — a folha não sabe o que é uma captura |
//! | `an_ik_anchor_is_not_an_empty_object` | que o gizmo de grupo não reclama uma âncora de osso | **da costura** — o `group_gizmo_view` é da shell, a âncora é da família |
//!
//! ⚠️ O segundo é o mais subtil: ele afirma uma relação entre **duas** famílias, e por isso não é
//! de nenhuma das duas. Levá-lo para a crate do esqueleto obrigaria essa crate a depender do
//! gizmo de grupo da shell — a seta ao contrário (HOWTO §4).

// ⚠️ **Os dois gates viviam DENTRO da crate e liam `use super::*`.** Do lado de cá a família é uma
// dependência, logo cada nome é nomeado pelo sítio onde ele de facto vive — e a fixtura `braco`
// atravessa pela feature `test-support` (HOWTO §2.5).
use ph2d_app_skeleton::goal::{
    add, anchors, braco, drag_anchor, governed, osso, remove, solve, unanchored_ends,
};
use ph2d_ecs::SimWorld;
use ph2d_preview_drive::PreviewDrive;

/// ⚠️ **SONDA:** a âncora atravessa a captura do undo? (Report do dono, 2026-09-07: *«Undo não
/// funciona para add IK»*.)
///
/// Ela separa as DUAS metades que o sintoma não distingue: *a fotografia não vê a âncora* (e aí o
/// passo nasce vazio) contra *a fotografia vê e o passo não é registado* (e aí a causa é um dos
/// cinco motivos de supressão do `post_frame_undo`).
#[test]
#[ignore = "sonda de medição: imprime, não julga"]
fn probe_does_the_anchor_cross_the_undo_capture() {
    use ph2d_ecs::scene::{ComponentRegistry, register_ecs_components};
    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);
    ph2d_render::register_render_components(&mut reg);
    ph2d_skeleton_ecs::register_skeleton_components(&mut reg);

    let (mut sim, [_, cotovelo]) = braco();
    let vec = ph2d_vec_scene::VecScene::new();
    let mut cache = ph2d_ecs::scene::incremental::CaptureCache::new();
    let tirar = |sim: &mut SimWorld, cache: &mut ph2d_ecs::scene::incremental::CaptureCache| {
        crate::undo::ProjectState::capture(
            &PreviewDrive::default(),
            sim,
            &vec,
            &ph2d_flip::FlipDoc::new(),
            &ph2d_guides::GuideSet::default(),
            &ph2d_ui_state::StateSets::default(),
            &crate::project_library::LibraryDoc::default(),
            &[],
            &reg,
            cache,
            None,
        )
    };
    let antes = tirar(&mut sim, &mut cache);
    let alvo = add(&mut sim, cotovelo).expect("a ancora");
    let depois = tirar(&mut sim, &mut cache);
    eprintln!(
        "[probe] a captura VE' a ancora? {} (partes que diferem: {:?})",
        antes != depois,
        depois.parts_that_differ(&antes)
    );
    // E o restauro leva-a embora?
    let _ = antes.restore(&mut sim, &reg);
    eprintln!(
        "[probe] depois do restore: alvo vivo? {} · o osso ainda tem ancora? {}",
        sim.world().get_entity(alvo).is_ok(),
        sim.world()
            .iter_entities()
            .any(|er| er.contains::<ph2d_skeleton_ecs::IkGoal>())
    );
}

// ⭐ **O cabeçalho que este gate trazia quando vivia em `skeleton_handle_tests.rs`** — é
//    prosa histórica, e passa a comentário normal porque um `//!` só pode abrir um ficheiro.
// Os gates da **ALÇA da âncora** — o que o dedo APONTA, e qual verbo ele vai executar.
//
// ⚠️ **Corte por RESPONSABILIDADE** (o teto de 600 LOC do HR-18 pediu-o a `718`), e é o mesmo
// corte que o `bone_gesture_tests`/`bone_pose_tests` já pagou: o irmão [`super`] mede a **LEI** de
// uma restrição (ela nasce sem mover, persiste, a mistura, o laço, a pré-visualização); aqui
// mede-se o **alvo do dedo** — o anel, o miolo, e o que acontece quando dois alvos ficam
// concêntricos.
//
// ⛔ **A pergunta que este ficheiro existe para responder** veio de um report do dono (2026-09-07):
// *«quando colocamos um IK num bone no meio dos ossos, o losango do IK e o círculo do outro osso
// ficam sobrepostos»*. Alvos concêntricos com verbos diferentes só se separam por **tamanho**.

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
    ph2d_preview_drive::PreviewDrive::restore_live(&mut sim, &vivo);
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
        ph2d_app_skeleton::bone_pick::hit(&sim, longe, 1.0).is_none(),
        "a fixtura nao produz o fenomeno: ha' um osso debaixo do ponteiro"
    );
    let h = ph2d_app_skeleton::bone_pick::hover(
        &sim,
        longe,
        1.0,
        None,
        ph2d_tool_vector::BoneAction::Transform,
    )
    .expect("a ancora tem de ser achada");
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
    let junta = ph2d_app_skeleton::bone_gesture::test_segment(&sim, cotovelo.to_bits()).0;
    assert!(
        (ancora[0] - junta[0]).hypot(ancora[1] - junta[1]) < 1e-9,
        "a fixtura nao produz o fenomeno: a ancora nao caiu sobre a junta do osso seguinte"
    );
    let comp = (p[0] - o[0]).hypot(p[1] - o[1]);
    let miolo = ph2d_skeleton_render::joint_radius_px(comp);
    let anel = ph2d_skeleton_render::goal_radius_px(comp);
    assert!(anel > miolo, "o losango tem de ser MAIOR que a bolinha");
    // No MIOLO: o osso seguinte, com o verbo de deslocar.
    let dentro = ph2d_app_skeleton::bone_pick::hover(
        &sim,
        ancora,
        1.0,
        None,
        ph2d_tool_vector::BoneAction::Transform,
    )
    .expect("algo sob o dedo");
    assert_eq!(
        dentro.bone,
        cotovelo.to_bits(),
        "o centro tem de pegar o OSSO"
    );
    assert_eq!(dentro.part, ph2d_skeleton_render::BonePart::Joint);
    // No ANEL: a âncora.
    let no_anel = [ancora[0] + (miolo + anel) * 0.5, ancora[1]];
    let fora = ph2d_app_skeleton::bone_pick::hover(
        &sim,
        no_anel,
        1.0,
        None,
        ph2d_tool_vector::BoneAction::Transform,
    )
    .expect("algo sob o dedo");
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
    let h = ph2d_app_skeleton::bone_pick::hover(
        &sim,
        longe,
        1.0,
        None,
        ph2d_tool_vector::BoneAction::Transform,
    )
    .expect("o centro do losango");
    assert_eq!(h.bone, cotovelo.to_bits());
    assert_eq!(h.part, ph2d_skeleton_render::BonePart::Tip);
}

/// O despacho da caneta, lido como TEXTO — a agulha vive aqui e o sujeito lá, senão um
/// `include_str!` cuja agulha está dentro do ficheiro que ele lê conta-se a si mesmo.
const PREMIDO: &str = include_str!("input_dispatch/despacho_clique_vetor_premido.rs");
/// O passe que resolve *«o que está sob o cursor?»* uma vez por quadro.
const SUJEITOS: &str = include_str!("render_loop/fase_pointer_subjects.rs");
/// O passe que pinta os realces do vector.
const OVERLAYS: &str = include_str!("render_loop/fase_vector_overlays.rs");

/// ⭐⭐⭐ **O PONTO NOVO DA CANETA CHEGA À FONTE DA PELE** — a costura que só aqui pode ser afirmada.
///
/// ⛔⛔ **A lei tem os gates dela** (seis, em `ph2d_skeleton_live::ponto_novo`) e a sonda do defeito
/// tem os dela (`ph2d_app_skeleton::sonda_do_ponto_novo_tests`). O que só deste lado se pode dizer é
/// o FIO: a caneta reporta onde inseriu, a shell drena isso, e a família escreve na geometria
/// autorada. *Sem esta linha o ponto aparece sob o dedo e desaparece sozinho no quadro seguinte,
/// sem erro e sem aviso* — e nenhum dos gates da lei dá por isso, porque todos entram pela porta.
///
/// ⚠️ **Ele mede TEXTO e não uma chamada**, pela mesma razão do censo dos verbos: este despacho é um
/// método de `App`, que segura uma surface de janela real, logo nenhum teste o corre.
#[test]
fn o_ponto_novo_da_caneta_chega_a_fonte_da_pele() {
    assert!(
        PREMIDO.contains("take_insercao()"),
        "o despacho da caneta deixou de drenar onde ela inseriu: o ponto novo numa forma PRESA \
         volta a evaporar-se no quadro seguinte"
    );
    assert!(
        PREMIDO.contains("ponto_novo::insere_ponto("),
        "o despacho drena a insercao e nao a leva a lado nenhum — o `t` do dedo e' lido e deitado \
         fora, que e' o mesmo que nao o ler"
    );
}

/// ⭐⭐⭐ **A PRÉVIA DE INSERÇÃO CHEGA A PIXEL** — as três pontas do fio, e nenhuma se vê das outras.
///
/// ⛔⛔ **Report do dono, 2026-09-19: *«não tem indicação visual que você está em cima da linha para
/// criar um ponto»*.** A lei tem o gate dela na `ph2d-vec-edit` (a prévia acende onde o clique
/// insere, apaga-se onde ele não insere, e aterra no sítio onde o ponto nasce). O que só deste lado
/// se pode afirmar é que ela é **derivada por quadro** e **pintada** — *uma prévia que ninguém
/// calcula e uma que ninguém desenha dão o MESMO report, e as curas são diferentes*.
///
/// ⚠️ **A terceira metade é a LIMPEZA:** fora do modo Pen ela tem de ser apagada, e não apenas
/// não-actualizada. Um realce deixado a arder depois de trocar de ferramenta promete um ponto que
/// nenhum clique põe — a mesma lei que o realce do Trim já escreve.
#[test]
fn a_previa_de_insercao_chega_a_pixel() {
    assert!(
        SUJEITOS.contains("previa_de_insercao("),
        "ninguem calcula a previa por quadro: ela nunca acende"
    );
    assert!(
        SUJEITOS.contains("DrawMode::Pen"),
        "a previa deixou de ser gateada pelo modo: ela acende com a seta na mao, onde o clique nao \
         insere nada"
    );
    assert!(
        OVERLAYS.contains("draw_insert_preview("),
        "a previa e' calculada e nao e' PINTADA — o report do dono volta inteiro, com o trabalho \
         todo feito por baixo"
    );
}

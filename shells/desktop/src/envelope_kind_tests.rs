//! Os gates da **Fatia D do ADR-0129**: a gaiola tem DOIS gestos — `Perspective` (homografia,
//! lados retos) e `Mesh` (patch de Coons, lados que dobram).
//!
//! Eles NÃO são o mesmo mapa com um knob: com os lados retos o Coons é *bilinear* e a homografia é
//! *projetiva*, e as duas só coincidem com a gaiola em REPOUSO. O que estes gates fixam é
//! exatamente essa fronteira — trocar em repouso é seguro, e fora dele o gesto escolhido manda de
//! verdade (senão o chip do painel seria decorativo). A matemática dos dois mapas está gateada na
//! crate `ph2d-vec-envelope`; aqui é o comportamento do HOST.

use super::*;
/// Troca o gesto do container (o que os chips do painel fazem).
fn set_kind(sim: &mut SimWorld, bits: u64, kind: EnvelopeKind) {
    crate::envelope_gesture::set_kind(sim, bits, kind);
}

/// Dobra o lado de BAIXO da gaiola para fora — o gesto que só o Mesh oferece.
fn bend_bottom(sim: &mut SimWorld, bits: u64) {
    let mut env = sim
        .world_mut()
        .get_mut::<VecEnvelope>(Entity::from_bits(bits))
        .expect("VecEnvelope");
    let span = env.corners[1][0] - env.corners[0][0];
    let drop = env.corners[3][1] - env.corners[0][1];
    env.edges[0][0] = [
        env.corners[0][0] + span / 3.0,
        env.corners[0][1] - drop * 0.4,
    ];
    env.edges[0][1] = [
        env.corners[0][0] + span * 2.0 / 3.0,
        env.corners[0][1] - drop * 0.4,
    ];
}

/// A soma das âncoras — uma assinatura barata da geometria, sensível a qualquer deslocamento.
fn signature(p: &VecPath) -> [f64; 2] {
    p.verts_all()
        .fold([0.0, 0.0], |a, v| [a[0] + v.anchor[0], a[1] + v.anchor[1]])
}

/// **TROCAR DE GESTO NUMA GAIOLA EM REPOUSO NÃO MOVE A ARTE.** Em repouso a homografia e o patch de
/// Coons são ambos a identidade, então o chip é seguro de tocar antes de deformar.
///
/// A metade **presença** é a asserção de que a forma continua CHEIA: um recook que apagasse tudo
/// também deixaria as duas "iguais".
#[test]
fn switching_the_cage_gesture_at_rest_does_not_move_the_art() {
    let shape = ellipse([5.0, 5.0], 3.0);
    let (mut sim, mut scene, _map, container, ids) = envelope_over(vec![shape]);
    let before = frame(&mut sim, &mut scene, ids[0]);
    assert!(
        before.verts.len() >= 4,
        "a forma nasceu vazia: fixture morto"
    );

    set_kind(&mut sim, container, EnvelopeKind::Mesh);
    let after = frame(&mut sim, &mut scene, ids[0]);

    let (a, b) = (signature(&before), signature(&after));
    assert!(
        (a[0] - b[0]).abs() < 1e-9 && (a[1] - b[1]).abs() < 1e-9,
        "trocar o gesto em repouso moveu a arte: {a:?} -> {b:?}"
    );
}

/// **O LADO DOBRADO SÓ DEFORMA NO MESH** — e no Mesh ele deforma DE VERDADE.
///
/// Este é o par que prova que o chip não é decorativo. Se o `recook` ignorasse `env.kind` e usasse
/// sempre a homografia, a 2ª metade falharia; se ele usasse sempre o Coons, a 1ª falharia. Uma
/// mutação em qualquer um dos dois ramos sangra aqui.
#[test]
fn a_bent_side_only_deforms_in_the_mesh_gesture() {
    let shape = ellipse([5.0, 5.0], 3.0);
    let (mut sim, mut scene, _map, container, ids) = envelope_over(vec![shape]);
    let rest = signature(&frame(&mut sim, &mut scene, ids[0]));

    // Perspective: os lados existem no componente mas o mapa NÃO os lê.
    bend_bottom(&mut sim, container);
    let persp = signature(&frame(&mut sim, &mut scene, ids[0]));
    assert!(
        (rest[0] - persp[0]).abs() < 1e-9 && (rest[1] - persp[1]).abs() < 1e-9,
        "o gesto Perspective leu os lados curvos: {rest:?} -> {persp:?}"
    );

    // Mesh: o MESMO lado dobrado agora manda.
    set_kind(&mut sim, container, EnvelopeKind::Mesh);
    let mesh = signature(&frame(&mut sim, &mut scene, ids[0]));
    let moved = (rest[0] - mesh[0]).hypot(rest[1] - mesh[1]);
    assert!(
        moved > 1.0,
        "o gesto Mesh nao deformou pelo lado dobrado (deslocamento {moved:.3e})"
    );
}

/// **SAIR DO MESH ENDIREITA OS LADOS GUARDADOS.** Em Perspective os lados *são* retos — deixar os
/// controles dobrados no componente faria a volta ao Mesh ressuscitar uma gaiola que o mapa nunca
/// aplicou, e as alças apareceriam fora dos lados desenhados.
#[test]
fn leaving_the_mesh_gesture_straightens_the_stored_sides() {
    let (mut sim, _scene, _map, container, _ids) = envelope_over(vec![ellipse([5.0, 5.0], 3.0)]);
    set_kind(&mut sim, container, EnvelopeKind::Mesh);
    bend_bottom(&mut sim, container);
    assert_ne!(
        env_of(&sim, container).edges,
        ph2d_vec_envelope::rest_edges(&env_of(&sim, container).corners),
        "fixture morto: o lado nao ficou dobrado"
    );

    set_kind(&mut sim, container, EnvelopeKind::Perspective);
    let env = env_of(&sim, container);
    assert_eq!(
        env.edges,
        ph2d_vec_envelope::rest_edges(&env.corners),
        "voltar ao Perspective deixou lados curvos guardados"
    );
}

/// **A ALÇA DE LADO SÓ É AGARRÁVEL NO MESH.** O mesmo cursor, sobre o mesmo ponto: em Perspective o
/// pen fica com o clique (a alça não existe), no Mesh o envelope o toma.
///
/// Sem o 2º ramo o gate ficaria verde num `press` que nunca pega nada.
#[test]
fn a_side_handle_is_only_grabbable_in_the_mesh_gesture() {
    let (mut sim, scene, _map, container, _ids) = envelope_over(vec![ellipse([5.0, 5.0], 3.0)]);
    let on_handle = env_of(&sim, container).edges[0][0];
    let mut drag = None;
    // Escala de zoom que torna o raio da bolinha PEQUENO em unidades de mundo (0,3): o controle
    // fica exatamente sob o cursor e o canto vizinho, a ~2 unidades, fora de alcance. Com o raio
    // do fixture ingênuo (1 px/unidade = 6 unidades) o canto engoliria o controle e o gate mediria
    // outra coisa — [[feedback_test_with_product_numbers_not_convenient_ones]].
    const PX_TO_WORLD: f64 = 0.05;

    // ⚠️ A pergunta é se ele ARMA o controle de lado — não se consome o clique. Sobre a arte do
    // envelope o `press` consome sempre (o fix do "os pontos travam ao arrastar"), e medir o
    // retorno mediria a outra regra.
    let _ = crate::envelope_gesture::press(
        &mut sim,
        &scene,
        &Default::default(),
        Some(container),
        on_handle,
        PX_TO_WORLD,
        false,
        &mut drag,
    );
    assert_eq!(
        drag, None,
        "o Perspective armou uma alca de lado que o mapa dele ignora"
    );
    set_kind(&mut sim, container, EnvelopeKind::Mesh);
    assert!(
        crate::envelope_gesture::press(
            &mut sim,
            &scene,
            &Default::default(),
            Some(container),
            on_handle,
            PX_TO_WORLD,
            false,
            &mut drag
        ),
        "o Mesh nao pegou a alca de lado sob o cursor"
    );
    assert_eq!(
        drag.map(|(_, h)| h),
        Some(ph2d_vec_envelope::edge_handle_index(0, 0)),
        "pegou a alca errada"
    );
}

// ── Fatia C: os presets de gaiola ────────────────────────────────────────────────────────────

use ph2d_ecs::EnvelopeWarp;

/// **NENHUM PRESET REAL DOBRA, EM PONTO NENHUM DA FAIXA.** Varre os 9 × `bend ∈ [-1,1]` (inclui o
/// SHEAR do Rise — a re-verificação de não-dobra que a gaiola estendida prometeu).
///
/// A crate do motor gateia a garantia sobre barrigas escritas à mão; ela **não conhece a tabela**
/// (que é dado, e mora no componente). Este é o irmão que fecha o circuito: se alguém acrescentar
/// um preset que enverga os quatro lados de uma vez, é AQUI que sangra — e é justamente o caso que
/// a primeira `AMP` escolhida não cobria.
#[test]
fn no_real_preset_folds_anywhere_in_its_range() {
    for warp in EnvelopeWarp::ALL {
        let cage = warp.cage();
        let bows: ph2d_vec_envelope::EdgeBows =
            cage.bows.map(|s| s.map(|v| v * ph2d_vec_envelope::AMP));
        // ⚠️ Inclui o SHEAR do Rise (o canto move): é a re-verificação da garantia de não-dobra
        // que a extensão da gaiola prometeu — um shear puro é paralelogramo, sempre convexo.
        let shift: ph2d_vec_envelope::EdgeBows = cage
            .shift
            .map(|s| s.map(|v| v * ph2d_vec_envelope::AMP_SHEAR));
        for step in -20..=20 {
            let bend = f64::from(step) / 20.0;
            let (c, e) = ph2d_vec_envelope::preset_cage(&bows, &shift, bend);
            assert!(
                !ph2d_vec_envelope::cage_folds(&c, &e),
                "o preset {} dobrou em bend={bend}",
                warp.label()
            );
        }
    }
}

/// **CADA PRESET PRODUZ UMA GAIOLA DISTINTA.** Nove botões que fizessem a mesma coisa seriam oito
/// botões mortos — e nada no resto da suíte notaria. (O Rise entra pelos CANTOS, não pelas
/// barrigas, então a gaiola dele é distinta via `corners`, não `edges`.)
#[test]
fn every_preset_is_a_different_cage() {
    let cages: Vec<Vec<[f64; 2]>> = EnvelopeWarp::ALL
        .iter()
        .map(|w| {
            let cage = w.cage();
            let bows: ph2d_vec_envelope::EdgeBows =
                cage.bows.map(|s| s.map(|v| v * ph2d_vec_envelope::AMP));
            let shift: ph2d_vec_envelope::EdgeBows = cage
                .shift
                .map(|s| s.map(|v| v * ph2d_vec_envelope::AMP_SHEAR));
            let (c, e) = ph2d_vec_envelope::preset_cage(&bows, &shift, 1.0);
            // Os CANTOS entram na assinatura (o Rise só se distingue por eles).
            c.iter().chain(e.iter().flatten()).copied().collect()
        })
        .collect();
    for (i, a) in cages.iter().enumerate() {
        for (j, b) in cages.iter().enumerate().skip(i + 1) {
            assert_ne!(
                a,
                b,
                "os presets {} e {} produzem a MESMA gaiola",
                EnvelopeWarp::ALL[i].label(),
                EnvelopeWarp::ALL[j].label()
            );
        }
    }
}

/// **O PRESET DEFORMA A ARTE, E O `bend` MANDA.** Carimbar Arc muda a forma; carimbar com `bend = 0`
/// a devolve ao repouso ao bit.
///
/// A 2ª metade é a que impede a 1ª de ficar verde por acidente: se `apply_preset` escrevesse uma
/// gaiola qualquer, a forma também "mudaria".
#[test]
fn a_preset_deforms_the_art_and_zero_bend_returns_it() {
    let shape = ellipse([5.0, 5.0], 3.0);
    let (mut sim, mut scene, _map, container, ids) = envelope_over(vec![shape]);
    let rest = signature(&frame(&mut sim, &mut scene, ids[0]));

    assert!(crate::envelope_live::apply_preset(
        &mut sim,
        container,
        EnvelopeWarp::Arc,
        1.0
    ));
    let arced = signature(&frame(&mut sim, &mut scene, ids[0]));
    let moved = (rest[0] - arced[0]).hypot(rest[1] - arced[1]);
    assert!(
        moved > 1.0,
        "o preset Arc nao deformou (deslocamento {moved:.3e})"
    );

    crate::envelope_live::apply_preset(&mut sim, container, EnvelopeWarp::Arc, 0.0);
    let back = signature(&frame(&mut sim, &mut scene, ids[0]));
    assert!(
        (rest[0] - back[0]).abs() < 1e-9 && (rest[1] - back[1]).abs() < 1e-9,
        "bend=0 nao devolveu a arte ao repouso: {rest:?} -> {back:?}"
    );
}

/// **A GAIOLA APARECE — `view` devolve `Some` com o container selecionado** (o Enio reportou "o
/// cage sumiu de novo", 2026-07-25). Recém-criada (Perspective) E depois de um preset (Mesh), o
/// overlay tem o que desenhar. Se `view` devolvesse `None` aqui, o `render_loop` não pintaria a
/// gaiola — o sintoma exato. (Um envelope sem seleção não tem gaiola: é o `None` do caso vazio.)
#[test]
fn the_cage_view_is_present_for_a_fresh_and_a_preset_envelope() {
    let (mut sim, _scene, _map, container, _ids) = envelope_over(vec![ellipse([5.0, 3.0], 2.0)]);
    assert!(
        crate::envelope_gesture::view(&sim, Some(container), None).is_some(),
        "a gaiola sumiu num envelope recém-criado (Perspective)"
    );
    // Um preset põe a gaiola em Mesh — a `view` tem de continuar devolvendo a gaiola (com alças).
    crate::envelope_live::apply_preset(&mut sim, container, EnvelopeWarp::Rise, 0.5);
    assert!(
        crate::envelope_gesture::view(&sim, Some(container), None).is_some(),
        "a gaiola sumiu após um preset (Rise/Mesh)"
    );
    // Sem seleção não há gaiola — o caso vazio, para o Some acima não passar por vacuidade.
    assert!(
        crate::envelope_gesture::view(&sim, None, None).is_none(),
        "sem seleção não devia haver gaiola"
    );
}

/// **O PRESET PÕE A GAIOLA EM MESH.** Com lados retos não há preset a exprimir — um "Arc" de 4
/// cantos retos é um trapézio. Sem isto o botão pareceria não fazer nada em Perspective.
#[test]
fn a_preset_switches_the_cage_to_mesh() {
    let (mut sim, _scene, _map, container, _ids) = envelope_over(vec![ellipse([5.0, 5.0], 3.0)]);
    assert_eq!(env_of(&sim, container).kind, EnvelopeKind::Perspective);
    crate::envelope_live::apply_preset(&mut sim, container, EnvelopeWarp::Flag, 0.5);
    assert_eq!(env_of(&sim, container).kind, EnvelopeKind::Mesh);
    assert_eq!(env_of(&sim, container).warp, Some(EnvelopeWarp::Flag));
}

/// **ARRASTAR UMA ALÇA PROMOVE A GAIOLA A MANUAL** (ADR-0129 §4, *"promovível"*). Depois disso o
/// slider Bend não é oferecido — e, sobretudo, não re-carimba por cima do que a mão fez.
///
/// Sem esta regra o preset e a mão seriam **dois donos da mesma gaiola**, e o próximo toque no
/// slider apagaria o gesto do artista sem aviso.
#[test]
fn dragging_a_handle_releases_the_preset() {
    let (mut sim, _scene, _map, container, _ids) = envelope_over(vec![ellipse([5.0, 5.0], 3.0)]);
    crate::envelope_live::apply_preset(&mut sim, container, EnvelopeWarp::Bulge, 0.5);
    assert!(env_of(&sim, container).warp.is_some(), "fixture morto");

    let corner = env_of(&sim, container).corners[2];
    let moved = [corner[0] + 0.2, corner[1] + 0.2];
    assert!(crate::envelope_gesture::drag(
        &mut sim,
        Some((container, 2)),
        moved
    ));
    assert_eq!(
        env_of(&sim, container).warp,
        None,
        "a mao mexeu na gaiola e o preset continuou dono dela"
    );
}

/// **A GAIOLA DO PRESET COBRE O RETÂNGULO-FONTE** — os cantos voltam ao repouso, seja qual for a
/// gaiola de antes. É o *Reset with Warp*: pedir um arco depois de puxar um canto dá um arco, não
/// um arco torto.
#[test]
fn a_preset_resets_the_corners_to_rest() {
    let (mut sim, _scene, _map, container, _ids) = envelope_over(vec![ellipse([5.0, 5.0], 3.0)]);
    let rest = env_of(&sim, container).corners;
    // Puxa um canto para longe.
    assert!(crate::envelope_gesture::drag(
        &mut sim,
        Some((container, 2)),
        [12.0, 12.0]
    ));
    assert_ne!(env_of(&sim, container).corners, rest, "fixture morto");

    crate::envelope_live::apply_preset(&mut sim, container, EnvelopeWarp::Wave, 0.7);
    let after = env_of(&sim, container).corners;
    for i in 0..4 {
        assert!(
            (after[i][0] - rest[i][0]).abs() < 1e-9 && (after[i][1] - rest[i][1]).abs() < 1e-9,
            "o preset nao devolveu o canto {i} ao repouso: {:?} != {:?}",
            after[i],
            rest[i]
        );
    }
}

/// Os gates da **Fatia E** (os pinos / MLS) — módulo filho, teto de LOC. Herda os fixtures deste
/// arquivo por `use super::*`.
#[path = "envelope_pins_tests.rs"]
mod pins;
/// **O RECOOK É O DONO DA GEOMETRIA DOS FILHOS** — uma edição à mão não sobrevive um frame.
///
/// É o fato que explica o *"os pontos estão travando ao arrastar"* (Enio, 2026-07-18): a geometria
/// do filho é COZIDA a partir das fontes e da gaiola, então um ponto arrastado **anda e volta**. O
/// gate existe para que ninguém "conserte" o sintoma no lugar errado — a resposta não é deixar a
/// edição sobreviver (ela contradiria a gaiola), é **não oferecer** o ponto: o `press` engole o
/// clique na arte, e quem quer os pontos de volta usa **Expand**.
#[test]
fn the_recook_owns_the_child_geometry() {
    let (mut sim, mut scene, _map, container, ids) = envelope_over(vec![ellipse([5.0, 5.0], 3.0)]);
    let _ = container;
    let before = frame(&mut sim, &mut scene, ids[0]).verts[0].anchor;

    // O artista arrasta a âncora 0 da forma COZIDA (é o que o pen faz no modo Node).
    if let Some(p) = scene.path_mut(ids[0]) {
        p.verts[0].anchor = [before[0] + 2.0, before[1] + 2.0];
    }
    let dragged = scene.paths().iter().find(|p| p.id == ids[0]).unwrap().verts[0].anchor;

    // ...e o frame seguinte cozinha.
    let after = frame(&mut sim, &mut scene, ids[0]).verts[0].anchor;
    assert_ne!(dragged, before, "fixture morto: a âncora não foi arrastada");
    assert!(
        (after[0] - before[0]).hypot(after[1] - before[1]) < 1e-9,
        "a edição à mão sobreviveu ao recook ({before:?} -> {after:?}) — ou a gaiola deixou de \
         mandar na geometria, ou o recook parou de rodar"
    );
}

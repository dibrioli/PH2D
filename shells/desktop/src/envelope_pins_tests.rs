//! Os gates dos **PINOS** (ADR-0129 Fatia E) — o *puppet warp* MLS-rigid.
//!
//! A matemática está gateada na crate `ph2d-vec-envelope` (interpolação, precisão rígida, a
//! jacobiana contra diferença central); aqui é o **produto**: pregar, agarrar, o guard de dobra, a
//! saída, e o que se desenha. Mais o gate de undo, que é onde o overlay quase morreu em silêncio.

use super::*;

/// Põe o envelope no gesto Pinos e devolve a pose do container (para converter mundo↔local).
fn pins_mode(sim: &mut SimWorld, container: u64) {
    set_kind(sim, container, EnvelopeKind::Pins);
}

/// **CLICAR NO VAZIO PREGA UM PINO, E ELE NASCE EM REPOUSO.** Pregar é o gesto do puppet — não há
/// alça fixa a acertar. E o pino nascer parado é o que impede a arte de saltar só por ser pregada.
#[test]
fn pressing_empty_space_in_pins_mode_nails_a_resting_pin() {
    let (mut sim, scene, _map, container, _ids) = envelope_over(vec![ellipse([5.0, 5.0], 3.0)]);
    pins_mode(&mut sim, container);
    let mut drag = None;

    assert!(
        crate::envelope_gesture::press(
            &mut sim,
            &scene,
            Some(container),
            [4.0, 4.0],
            0.05,
            false,
            &mut drag
        ),
        "o gesto Pinos devia tomar o clique no vazio"
    );
    let pins = env_of(&sim, container).pins;
    assert_eq!(pins.len(), 1, "o clique não pregou pino nenhum");
    assert_eq!(pins[0][0], pins[0][1], "o pino nasceu já deslocado");
    assert_eq!(
        drag,
        Some((container, 0)),
        "o pino novo não ficou sob o dedo"
    );
}

/// **O SEGUNDO CLIQUE EM CIMA DE UM PINO O PEGA — não prega outro.** Sem isto, cada tentativa de
/// mover um pino criaria um pino novo empilhado, e a arte ficaria refém de uma pilha invisível.
#[test]
fn pressing_an_existing_pin_grabs_it_instead_of_nailing_another() {
    let (mut sim, scene, _map, container, _ids) = envelope_over(vec![ellipse([5.0, 5.0], 3.0)]);
    pins_mode(&mut sim, container);
    let mut drag = None;
    let _ = crate::envelope_gesture::press(
        &mut sim,
        &scene,
        Some(container),
        [4.0, 4.0],
        0.05,
        false,
        &mut drag,
    );
    assert_eq!(env_of(&sim, container).pins.len(), 1);

    // ⚠️ O pino é ARRASTADO antes do 2º clique, e isso não é decoração do fixture: com o pino em
    // repouso, `rest == moved` e o hit-test acerta perguntando pelo lugar ERRADO. A mutação
    // "compare contra a posição de repouso" sobreviveu à 1ª versão deste teste exatamente por isso.
    let _ = crate::envelope_gesture::drag(&mut sim, Some((container, 0)), [6.5, 4.0]);
    let moved = env_of(&sim, container).pins[0][1];
    assert_ne!(moved, [4.0, 4.0], "fixture morto: o pino não andou");

    let mut drag2 = None;
    let _ = crate::envelope_gesture::press(
        &mut sim,
        &scene,
        Some(container),
        moved,
        0.05,
        false,
        &mut drag2,
    );
    assert_eq!(
        env_of(&sim, container).pins.len(),
        1,
        "clicar em cima do pino MOVIDO pregou um pino a mais — o hit-test pergunta pelo lugar \
         onde ele já não está"
    );
    assert_eq!(drag2, Some((container, 0)));

    // E o lugar de REPOUSO, que o pino já deixou, não pega nada: clicar lá prega um pino novo.
    let mut drag3 = None;
    let _ = crate::envelope_gesture::press(
        &mut sim,
        &scene,
        Some(container),
        [4.0, 4.0],
        0.05,
        false,
        &mut drag3,
    );
    assert_eq!(
        env_of(&sim, container).pins.len(),
        2,
        "o lugar de repouso ainda agarrava o pino que já saiu de lá"
    );
}

/// **TRÊS PINOS DEFORMAM; DOIS NÃO** — a armadilha de dia-um, ao nível do PRODUTO.
///
/// Uma isometria de um par determina uma rigidez única, então mover 2 pinos devolve movimento rígido
/// e a arte não muda de forma. Quem for depurar "os pinos não fazem nada" tem de encontrar este
/// teste antes de mexer na matemática: não há o que consertar, é preciso o 3º pino.
#[test]
fn two_pins_do_not_deform_the_art_but_three_do() {
    let (mut sim, mut scene, _map, container, ids) = envelope_over(vec![ellipse([5.0, 5.0], 3.0)]);
    pins_mode(&mut sim, container);
    let rest = signature(&frame(&mut sim, &mut scene, ids[0]));

    // Dois pinos movidos ISOMETRICAMENTE (o par rodado 53° em torno do meio, rotação 3-4-5 exata):
    // a arte ANDA e não muda de FORMA. A forma é medida pelo tamanho da bbox — e o fixture é um
    // CÍRCULO de propósito, porque a bbox de um círculo é invariante à rotação; num fixture
    // alongado esta medida cairia sozinha e o gate mediria a rotação, não a deformação.
    //
    // ⚠️ Mover UM pino de dois NÃO é isometria (a distância entre eles muda), e aí o método deforma
    // mesmo — a afirmação do ADR é sobre movimento RÍGIDO do par. A 1ª versão deste fixture errou
    // exatamente isso e o gate o pegou.
    let before_box = spread_of(&frame(&mut sim, &mut scene, ids[0]));
    let (c, s) = (0.6_f64, 0.8_f64);
    let about = |p: [f64; 2]| {
        let d = [p[0] - 5.0, p[1] - 5.0];
        [5.0 + c * d[0] - s * d[1], 5.0 + s * d[0] + c * d[1]]
    };
    set_pins(
        &mut sim,
        container,
        vec![
            [[2.0, 5.0], about([2.0, 5.0])],
            [[8.0, 5.0], about([8.0, 5.0])],
        ],
    );
    let two_frame = frame(&mut sim, &mut scene, ids[0]);
    let two = spread_of(&two_frame);
    assert!(
        (two - before_box).abs() < 0.02,
        "2 pinos isométricos DEFORMARAM a arte (diâmetro {before_box:.4} -> {two:.4})"
    );

    // O 3º pino, não-colinear, deslocado: agora deforma de verdade.
    set_pins(
        &mut sim,
        container,
        vec![
            [[2.0, 5.0], [2.0, 5.0]],
            [[8.0, 5.0], [8.0, 5.0]],
            [[5.0, 7.5], [5.0, 10.0]],
        ],
    );
    let three = spread_of(&frame(&mut sim, &mut scene, ids[0]));
    assert!(
        (three - before_box).abs() > 0.2,
        "3 pinos não deformaram: diâmetro {before_box:.4} -> {three:.4}"
    );
    let _ = rest;
    let _ = two_frame;
}

/// **UM ARRASTO QUE DOBRARIA A ARTE É RECUSADO** — o pino para na fronteira, como o canto
/// não-convexo e o controle que dobraria o Coons. É esta recusa que mantém o `break_cusp` em `None`
/// honesto: o estado dobrado fica inalcançável pela mão.
#[test]
fn a_pin_drag_that_would_fold_the_art_is_refused() {
    let (mut sim, _scene, _map, container, _ids) = envelope_over(vec![ellipse([5.0, 5.0], 3.0)]);
    pins_mode(&mut sim, container);
    set_pins(
        &mut sim,
        container,
        vec![
            [[2.0, 5.0], [2.0, 5.0]],
            [[8.0, 5.0], [8.0, 5.0]],
            [[5.0, 6.0], [5.0, 6.0]],
        ],
    );
    // Um puxão moderado passa...
    assert!(crate::envelope_gesture::drag(
        &mut sim,
        Some((container, 2)),
        [5.5, 6.5]
    ));
    assert_ne!(
        env_of(&sim, container).pins[2][1],
        [5.0, 6.0],
        "o puxão moderado foi recusado"
    );

    // ...e um que atira o pino para muito além da arte, não.
    let before = env_of(&sim, container).pins[2][1];
    let _ = crate::envelope_gesture::drag(&mut sim, Some((container, 2)), [5.0, -60.0]);
    assert_eq!(
        env_of(&sim, container).pins[2][1],
        before,
        "o pino atravessou a arte e dobrou o mapa — o guard não recusou"
    );
}

/// **`Clear Pins` é a porta de saída** — e ela existe porque não há como apagar UM pino: sem
/// nenhuma porta, um pino mal pregado seria permanente.
#[test]
fn clear_pins_empties_the_list() {
    let (mut sim, _scene, _map, container, _ids) = envelope_over(vec![ellipse([5.0, 5.0], 3.0)]);
    pins_mode(&mut sim, container);
    set_pins(&mut sim, container, vec![[[2.0, 5.0], [2.5, 5.5]]]);
    assert!(crate::envelope_gesture::clear_pins(&mut sim, container));
    assert!(env_of(&sim, container).pins.is_empty());
    assert!(
        !crate::envelope_gesture::clear_pins(&mut sim, container),
        "apagar de novo devia reportar que não havia nada"
    );
}

/// **NO GESTO PINOS NÃO SE DESENHA GAIOLA** — e nos outros dois não se desenham pinos. Duas
/// moldurinhas ao mesmo tempo diriam ao artista que há duas coisas a arrastar.
#[test]
fn the_cage_and_the_pins_are_never_drawn_together() {
    let (mut sim, _scene, _map, container, _ids) = envelope_over(vec![ellipse([5.0, 5.0], 3.0)]);
    set_pins(&mut sim, container, vec![[[2.0, 5.0], [2.5, 5.5]]]);

    assert!(
        crate::envelope_gesture::view(&sim, Some(container), None).is_some(),
        "em Perspective a gaiola tem de aparecer"
    );
    pins_mode(&mut sim, container);
    assert!(
        crate::envelope_gesture::view(&sim, Some(container), None).is_none(),
        "no gesto Pinos ainda se desenhava a gaiola"
    );
    assert_eq!(
        crate::envelope_gesture::pins_world(&sim, container).len(),
        1,
        "os pinos não chegaram ao desenho"
    );
}

/// Sobrescreve a lista de pinos do container (o que uma sessão de cliques produziria).
fn set_pins(sim: &mut SimWorld, bits: u64, pins: Vec<[[f64; 2]; 2]>) {
    sim.world_mut()
        .get_mut::<VecEnvelope>(Entity::from_bits(bits))
        .expect("VecEnvelope")
        .pins = pins;
}

/// O **diâmetro** do conjunto de âncoras: a maior distância entre duas delas.
///
/// ⚠️ **A medida de forma tem de ser invariante a ROTAÇÃO, e uma bbox alinhada ao eixo não é.** A 1ª
/// versão deste oráculo mediu largura×altura e acusou o par isométrico de encolher 1% — o que
/// encolheu foi a *caixa* de um círculo cozido em 4 cúbicas, cujas âncoras deixam de estar nos
/// extremos depois de rodar. Rigidez preserva DISTÂNCIA; é distância que se mede.
fn spread_of(p: &VecPath) -> f64 {
    let a: Vec<[f64; 2]> = p.verts_all().map(|v| v.anchor).collect();
    let mut worst = 0.0_f64;
    for (i, u) in a.iter().enumerate() {
        for w in &a[i + 1..] {
            worst = worst.max((u[0] - w[0]).hypot(u[1] - w[1]));
        }
    }
    worst
}

/// **O UNDO NÃO PODE FAZER A GAIOLA/OS PINOS SUMIREM** (Enio, smoke da Fatia E).
///
/// O `ProjectState::restore` **despawna e re-spawna** o mundo: *"ids do mundo são novos"*. O recook
/// sobrevive porque varre por QUERY, e por isso a arte continuava deformada — mas **o desenho do
/// overlay é indexado pelos bits da seleção do gizmo**, e esses bits morrem no respawn. Resultado:
/// a ferramenta funcionando e invisível.
///
/// A seleção do PEN é estável (é `VecPathId`, e o snapshot leva a geometria), então a resposta é
/// **re-derivar** os bits dela. Este gate percorre o caminho real: captura → restaura → reconstrói
/// a ponte → `sync_selection` → o overlay tem de voltar a apontar para uma entidade VIVA.
#[test]
fn the_pins_survive_an_undo() {
    use ph2d_ecs::scene::{ComponentRegistry, register_ecs_components};

    let (mut sim, mut scene, mut map, container, ids) =
        envelope_over(vec![ellipse([5.0, 5.0], 3.0)]);
    pins_mode(&mut sim, container);
    set_pins(&mut sim, container, vec![[[4.0, 4.0], [4.5, 4.5]]]);

    // A seleção como o produto a tem: o pen com os FILHOS, o gizmo com o container.
    let mut pen = pen_with(&ids);
    let mut gizmo = ph2d_editor::screens::hero::GizmoStateGroup::default();
    let mut sel = crate::vec_selection::VecSelSync::default();
    crate::vec_selection::sync_selection(&mut gizmo, &sim, &scene, &map, &mut pen, &mut sel, true);
    assert_eq!(
        gizmo.selection,
        Some(container),
        "fixture morto: o gizmo devia estar no container"
    );
    assert_eq!(
        crate::envelope_gesture::pins_world(&sim, container).len(),
        1,
        "fixture morto: o pino devia estar desenhável"
    );

    // O undo, pelo caminho REAL: captura, mexe, restaura (ids do mundo ficam NOVOS).
    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);
    let snap = {
        let mut prop = ph2d_ecs::TransformPropagationState::new(sim.world_mut());
        let mut wl = ph2d_ecs::WorklistBuf::new();
        crate::undo::ProjectState::capture(
            &sim,
            &scene,
            &ph2d_flip::FlipDoc::new(),
            &reg,
            &mut prop,
            &mut wl,
        )
    };
    let (restored_scene, restored_map, _flip, _fm) = snap.restore(&mut sim, &reg);
    scene = restored_scene;
    map = restored_map;

    // O frame seguinte re-sincroniza a seleção — e é AQUI que os bits mortos têm de ser trocados.
    crate::vec_selection::sync_selection(&mut gizmo, &sim, &scene, &map, &mut pen, &mut sel, true);

    let live = gizmo.selection.expect("o gizmo perdeu a seleção no undo");
    assert!(
        sim.world().get_entity(Entity::from_bits(live)).is_ok(),
        "o gizmo ficou com bits de uma entidade MORTA — o overlay não desenha nada"
    );
    assert_eq!(
        crate::envelope_gesture::pins_world(&sim, live).len(),
        1,
        "os pinos sumiram do desenho depois do undo (a arte continua deformada, e é isso que \
         torna o sintoma confuso)"
    );

    // ⚠️ **A GAIOLA sofria do mesmo mal, em silêncio.** O bug foi relatado nos pinos porque é neles
    // que se está a olhar, mas o overlay dos três gestos é desenhado pelos MESMOS bits — então o
    // gate cobra os dois, senão alguém "conserta" só o lado reportado.
    set_kind(&mut sim, live, EnvelopeKind::Perspective);
    assert!(
        crate::envelope_gesture::view(&sim, Some(live), None).is_some(),
        "a gaiola também some depois do undo — é o mesmo bits morto"
    );
}

/// **ALT+CLIQUE REMOVE O PINO SOB O CURSOR** — o idioma do Puppet Warp do Photoshop.
///
/// Sem ele, pregar era porta de mão única: todo clique no vazio prega, e um smoke real acumulou
/// **13 pinos**. A essa densidade quase nenhum arrasto passa o guard, e o `Clear Pins` — tudo ou
/// nada — é um machado onde faltava uma pinça.
#[test]
fn alt_click_removes_the_pin_under_the_cursor() {
    let (mut sim, scene, _map, container, _ids) = envelope_over(vec![ellipse([5.0, 5.0], 3.0)]);
    pins_mode(&mut sim, container);
    set_pins(
        &mut sim,
        container,
        vec![[[4.0, 4.0], [4.0, 4.0]], [[6.0, 6.0], [6.0, 6.0]]],
    );
    let mut drag = None;

    assert!(crate::envelope_gesture::press(
        &mut sim,
        &scene,
        Some(container),
        [4.0, 4.0],
        0.05,
        true, // Alt
        &mut drag
    ));
    let pins = env_of(&sim, container).pins;
    assert_eq!(pins.len(), 1, "o Alt+clique nao removeu o pino");
    assert_eq!(pins[0][0], [6.0, 6.0], "removeu o pino errado");
    assert_eq!(drag, None, "o Alt+clique armou um arrasto");
}

/// **ALT NO VAZIO NÃO PREGA NADA.** O modificador diz *remover*; criar ali seria o oposto do que o
/// dedo pediu — e é o erro fácil, porque o caminho sem Alt prega exatamente nessa situação.
#[test]
fn alt_on_empty_space_nails_nothing() {
    let (mut sim, scene, _map, container, _ids) = envelope_over(vec![ellipse([5.0, 5.0], 3.0)]);
    pins_mode(&mut sim, container);
    set_pins(&mut sim, container, vec![[[4.0, 4.0], [4.0, 4.0]]]);
    let mut drag = None;

    let _ = crate::envelope_gesture::press(
        &mut sim,
        &scene,
        Some(container),
        [40.0, 40.0],
        0.05,
        true,
        &mut drag,
    );
    assert_eq!(
        env_of(&sim, container).pins.len(),
        1,
        "o Alt no vazio pregou um pino — o modificador diz o CONTRARIO"
    );
}

/// **O GUARD DE DOBRA PERGUNTA PELA ARTE, NÃO PELA CAIXA — e as duas respostas DIFEREM.**
///
/// A bbox-união é a caixa dos pontos de CONTROLE, então ela tem cantos por onde nenhum contorno
/// passa. Uma dobra ali não produz o auto-cruzamento que este guard existe para impedir — e mesmo
/// assim vetava o gesto. Medido antes do fix: 13 pinos recusavam qualquer arrasto além de **0,70
/// numa altura de 2,80**; é o *"os pontos estão travando"* do Enio.
///
/// ⚠️ **A 1ª versão deste gate NÃO discriminava** (12 pinos, arrasto de 0,5: as duas perguntas
/// respondiam o mesmo, e a mutação "volte a perguntar pela caixa" sobreviveu). Os números abaixo
/// foram MEDIDOS até achar o regime em que elas divergem — um fixture só prova o que contém.
#[test]
fn the_fold_guard_asks_about_the_art_not_the_bounding_box() {
    let (mut sim, _scene, _map, container, _ids) = envelope_over(vec![ellipse([5.0, 5.0], 3.0)]);
    pins_mode(&mut sim, container);
    // 13 pinos em anel sobre a arte — a densidade que o smoke real produziu.
    let pins: Vec<[[f64; 2]; 2]> = (0..13)
        .map(|i| {
            let a = f64::from(i) * std::f64::consts::TAU / 13.0;
            let p = [5.0 + 2.6 * a.cos(), 5.0 + 2.6 * a.sin()];
            [p, p]
        })
        .collect();
    set_pins(&mut sim, container, pins.clone());

    let art = crate::envelope_live::art_samples(&sim, container);
    assert!(!art.is_empty(), "fixture morto: a arte nao tem pontos");
    let env = env_of(&sim, container);
    let grid = ph2d_vec_envelope::domain_samples(
        env.corners[0],
        [
            env.corners[1][0] - env.corners[0][0],
            env.corners[3][1] - env.corners[0][1],
        ],
    );

    // Um arrasto de 1,2 unidade — gesto comum, e o regime em que as duas perguntas divergem.
    let mut moved = pins;
    moved[0][1] = [moved[0][0][0], moved[0][0][1] + 1.2];
    assert!(
        ph2d_vec_envelope::pins_fold_at(&moved, &grid),
        "fixture morto: a CAIXA tinha de recusar este arrasto (senao o gate nao mede nada)"
    );
    assert!(
        !ph2d_vec_envelope::pins_fold_at(&moved, &art),
        "o guard recusou um arrasto que a ARTE aguenta — e' o 'os pontos travam' de volta"
    );
}

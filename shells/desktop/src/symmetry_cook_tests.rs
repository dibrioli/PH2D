//! Gates do **COZIMENTO** da simetria — filho de `symmetry_live_tests.rs`.
//!
//! O pai responde *"o que este MODO é"* — quando a linha nasce, quem é adoptado, o que fica no
//! lugar. Este responde *"o que ele DESENHA"*: a ordem `reflectir → assar`, a contagem da rosácea,
//! o memo, o Apply e os eixos que o overlay recebe.
//!
//! ⚠️ **FILHO e não irmão**, e a razão é a fixture: `half_profile_scene`/`spawn_for`/`nudge` são a
//! porta por onde as duas metades montam a mesma cena, e um irmão obrigaria a duplicá-la — duas
//! fixtures do mesmo fenômeno derivam, e aí os dois lados deixam de falar do mesmo mundo.

use super::*;

/// Um **quadrado centrado em `x = 0`** — ele já é simétrico no eixo que a spec vai usar, e é isso
/// que o torna oráculo da ORDEM (ver o gate da pose).
fn symmetric_square_scene() -> (
    VecScene,
    ph2d_ecs::SimWorld,
    VecEntityMap,
    VecPathId,
    ph2d_ecs::Entity,
) {
    let mut sim = ph2d_ecs::SimWorld::default();
    let mut map = VecEntityMap::new();
    let mut scene = VecScene::new();
    let id = scene.push_path(ph2d_vec_scene::rectangle([-1.0, -1.0], [1.0, 1.0]));
    let e = spawn_for(&mut sim, &mut map, id, "Square", [0.0, 0.0]);
    (scene, sim, map, id, e)
}

/// A spec de espelho vertical pelo eixo local `x = 0` — o que os gates de COZIMENTO usam.
fn mirror_x() -> SymmetrySpec {
    SymmetrySpec {
        kind: SymmetryKind::MirrorX,
        center: [0.0, 0.0],
        ..SymmetrySpec::default()
    }
}

/// Todos os vértices AUTORADOS de um caminho, na ordem.
fn authored(scene: &VecScene, id: VecPathId) -> Vec<VecVertex> {
    scene
        .path(id)
        .expect("o caminho existe")
        .verts_all()
        .copied()
        .collect()
}

// ── O cozimento ────────────────────────────────────────────────────────────────────────

/// **A promessa central.** Mover o objeto no canvas leva a linha de simetria junto, *mantendo a
/// mesma distância relativa ao objeto* — e a forma executável disso é que o desenho INTEIRO (fonte
/// e cópia) translada pelo mesmo delta, ponto a ponto.
///
/// ⚠️ Este gate é também o do MEMO: o `recook` corre duas vezes sem ninguém limpar nada, então uma
/// chave que ignorasse a pose devolveria o desenho velho e a asserção falha.
#[test]
fn moving_the_shape_carries_the_axis_with_it() {
    let (scene, mut sim, map, xf, id, e) = half_profile_scene();
    assert_eq!(arm(&mut sim, &map, &[id], Some(mirror_x())), 1);

    let mut live = SymmetryLive::default();
    live.recook(&scene, &sim, &map, &xf, true);
    let before = drawn(&live, id);
    assert!(!before.is_empty(), "a simetria armada desenha alguma coisa");

    let delta = [5.0_f32, -2.0_f32];
    let xf2 = nudge(&mut sim, &map, e, delta);
    live.recook(&scene, &sim, &map, &xf2, true);
    let after = drawn(&live, id);

    assert_eq!(before.len(), after.len(), "o desenho não muda de tamanho");
    for (i, (a, b)) in before.iter().zip(after.iter()).enumerate() {
        let want = [a[0] + f64::from(delta[0]), a[1] + f64::from(delta[1])];
        assert!(
            (b[0] - want[0]).abs() < 1e-9 && (b[1] - want[1]).abs() < 1e-9,
            "ponto {i}: esperado {want:?}, veio {b:?} — o eixo ficou para trás"
        );
    }
}

/// **Desmarcar antes do Apply esconde as cópias e não destrói NADA.**
///
/// ⚠️ O componente FICA — o interruptor gateia o cozimento. É isso que faz religar devolver as
/// cópias inteiras, e é a leitura literal de *"as cópias somem mas não são destruídas"*.
#[test]
fn disarming_hides_the_copies_and_destroys_nothing() {
    let (scene, mut sim, map, xf, id, _e) = half_profile_scene();
    let pristine = authored(&scene, id);

    assert_eq!(arm(&mut sim, &map, &[id], Some(mirror_x())), 1);
    let mut live = SymmetryLive::default();
    live.recook(&scene, &sim, &map, &xf, true);
    assert!(
        !drawn(&live, id).is_empty(),
        "armada, a simetria põe geometria derivada na tela"
    );

    live.recook(&scene, &sim, &map, &xf, false);
    assert!(
        drawn(&live, id).is_empty(),
        "desligada, não sobra derivada nenhuma — o dispatch volta a desenhar a fonte"
    );
    assert!(
        spec_of(&sim, &map, id).is_some(),
        "e o COMPONENTE fica: nada foi destruído"
    );

    live.recook(&scene, &sim, &map, &xf, true);
    assert!(
        !drawn(&live, id).is_empty(),
        "religar devolve as cópias inteiras"
    );
    assert_eq!(
        authored(&scene, id),
        pristine,
        "a curva AUTORADA atravessou o ciclo sem ser tocada"
    );
}

/// **A ordem é `reflectir → assar a pose`, e este é o oráculo que a distingue.**
///
/// Um quadrado centrado em `x = 0` **já é** simétrico no eixo da spec, então a reflexão local é a
/// identidade sobre ele: a cópia tem de coincidir com a fonte sob QUALQUER pose. Reflectir depois
/// de assar usaria uma linha que a pose não-conforme torna outra, e as duas deixam de coincidir.
#[test]
fn the_pose_is_baked_after_the_reflection_not_before() {
    let (scene, mut sim, map, id, e) = symmetric_square_scene();
    // Rotação + escala NÃO-uniforme: é a pose em que as duas ordens divergem. Sob escala uniforme
    // (ou sem rotação) elas coincidem, e o gate não podia falhar.
    if let Some(mut t) = sim.world_mut().get_mut::<ph2d_ecs::Transform>(e) {
        t.rotation = 0.5;
        t.scale = ph2d_core::Vec2::new(2.0, 1.0);
    }
    let xf = crate::vec_transform::build(&sim, &map);
    assert_eq!(arm(&mut sim, &map, &[id], Some(mirror_x())), 1);

    let mut live = SymmetryLive::default();
    live.recook(&scene, &sim, &map, &xf, true);
    let paths = live.live().get(&id).expect("há derivada");
    assert_eq!(paths.len(), 2, "espelho simples: a fonte e a cópia");

    // A cópia é percorrida ao contrário (a reposição do winding), então o conjunto de âncoras é
    // que tem de coincidir — não a sequência.
    let mut src: Vec<[f64; 2]> = paths[0].verts_all().map(|v| v.anchor).collect();
    let mut mirrored: Vec<[f64; 2]> = paths[1].verts_all().map(|v| v.anchor).collect();
    let key = |p: &[f64; 2]| (p[0].to_bits(), p[1].to_bits());
    src.sort_by_key(key);
    mirrored.sort_by_key(key);
    assert_eq!(src.len(), mirrored.len());
    for (a, b) in src.iter().zip(mirrored.iter()) {
        assert!(
            (a[0] - b[0]).abs() < 1e-9 && (a[1] - b[1]).abs() < 1e-9,
            "a cópia de uma forma já simétrica saiu noutro lugar: {a:?} vs {b:?}"
        );
    }
}

/// **Radial põe `segments` formas na tela**, e a fonte é uma delas — o painel promete a contagem,
/// e é ela que tem de aparecer.
#[test]
fn radial_draws_one_shape_per_segment() {
    let (scene, mut sim, map, id, _e) = symmetric_square_scene();
    let xf = crate::vec_transform::build(&sim, &map);
    let spec = SymmetrySpec {
        kind: SymmetryKind::Radial,
        center: [3.0, 0.0],
        segments: 5,
        ..SymmetrySpec::default()
    };
    assert_eq!(arm(&mut sim, &map, &[id], Some(spec)), 1);

    let mut live = SymmetryLive::default();
    live.recook(&scene, &sim, &map, &xf, true);
    assert_eq!(
        live.live().get(&id).map(Vec::len),
        Some(5),
        "cinco pétalas, contando o original"
    );
}

/// **Re-armar com a MESMA spec não mexe em nada** — é o guard que torna seguro um painel que
/// re-emite o valor a cada frame de arrasto.
#[test]
fn arming_with_the_same_spec_touches_nothing() {
    let (_scene, mut sim, map, _xf, id, _e) = half_profile_scene();
    assert_eq!(arm(&mut sim, &map, &[id], Some(mirror_x())), 1);
    assert_eq!(
        arm(&mut sim, &map, &[id], Some(mirror_x())),
        0,
        "a segunda chamada é um no-op"
    );
}

/// **`forget` limpa memo E derivada** — o load de projeto troca a cena debaixo do cozimento, e os
/// `VecPathId` são reciclados entre documentos.
#[test]
fn forget_drops_the_memo_and_the_drawing() {
    let (scene, mut sim, map, xf, id, _e) = half_profile_scene();
    arm(&mut sim, &map, &[id], Some(mirror_x()));
    let mut live = SymmetryLive::default();
    live.recook(&scene, &sim, &map, &xf, true);
    assert!(!drawn(&live, id).is_empty());
    live.forget();
    assert!(drawn(&live, id).is_empty(), "nada sobrevive ao forget");
}

// ── O Apply ────────────────────────────────────────────────────────────────────────────

/// **O `Apply` põe no documento EXACTAMENTE o que estava na tela.**
///
/// ⚠️ O oráculo é a IDENTIDADE ponto-a-ponto entre a geometria cozida e a materializada: um gate
/// que só contasse formas ficaria verde sobre um Apply que produz a coisa certa *noutro lugar* —
/// que é o defeito que o ADR-0128 pagou cinco vezes (a forma SALTAVA no clique porque preview e
/// commit tinham portas diferentes).
#[test]
fn the_apply_lands_exactly_what_was_on_screen() {
    let (mut scene, mut sim, map, xf, id, _e) = half_profile_scene();
    // Sem fusão: assim a simetria produz DUAS formas, e o gate vê o conjunto inteiro.
    let spec = SymmetrySpec {
        fuse: false,
        ..mirror_x()
    };
    arm(&mut sim, &map, &[id], Some(spec));
    let mut live = SymmetryLive::default();
    live.recook(&scene, &sim, &map, &xf, true);
    let on_screen = drawn(&live, id);
    assert!(!on_screen.is_empty());

    let mut pen = ph2d_vec_edit::PenTool::default();
    let mut history = ph2d_vec_edit::History::default();
    assert!(materialise(
        &mut scene,
        &sim,
        &mut pen,
        &mut history,
        &map,
        &xf,
        &[id]
    ));

    let in_doc: Vec<[f64; 2]> = scene
        .paths()
        .iter()
        .flat_map(|p| p.verts_all().map(|v| v.anchor))
        .collect();
    assert_eq!(
        in_doc.len(),
        on_screen.len(),
        "o documento tem de receber a mesma quantidade de geometria que estava na tela"
    );
    for (i, (a, b)) in on_screen.iter().zip(in_doc.iter()).enumerate() {
        assert!(
            (a[0] - b[0]).abs() < 1e-9 && (a[1] - b[1]).abs() < 1e-9,
            "ponto {i}: a tela mostrava {a:?} e o documento recebeu {b:?} — a forma SALTOU"
        );
    }
    assert!(
        scene.path(id).is_none(),
        "a forma-fonte sai da cena: o que fica são as materializadas"
    );
}

/// **O Apply consolida TODA forma armada, não a selecção.**
///
/// ⚠️ A simetria é um MODO, então *"consolidar a forma e desativar a simetria"* vale para o que o
/// modo produziu. Consolidar só o seleccionado deixaria o resto das cópias na tela com o modo já
/// desligado — e o cozimento gateado deixaria de as desenhar, o que o artista leria como trabalho
/// evaporado.
#[test]
fn the_apply_reaches_every_armed_shape_not_just_the_selected_one() {
    let mut sim = ph2d_ecs::SimWorld::default();
    let mut map = VecEntityMap::new();
    let mut scene = VecScene::new();
    let a = scene.push_path(half_profile());
    spawn_for(&mut sim, &mut map, a, "A", [0.0, 0.0]);
    let b = scene.push_path(half_profile());
    spawn_for(&mut sim, &mut map, b, "B", [6.0, 0.0]);
    // Uma terceira, NUNCA desenhada sob o modo: ela não pode ser tocada.
    let plain = scene.push_path(half_profile());
    spawn_for(&mut sim, &mut map, plain, "Plain", [-6.0, 0.0]);
    let xf = crate::vec_transform::build(&sim, &map);

    arm(&mut sim, &map, &[a, b], Some(mirror_x()));
    let armed_ids = armed_paths(&sim, &map, &scene);
    assert_eq!(armed_ids, vec![a, b], "as duas armadas, e só elas");

    let mut pen = ph2d_vec_edit::PenTool::default();
    let mut history = ph2d_vec_edit::History::default();
    assert!(materialise(
        &mut scene,
        &sim,
        &mut pen,
        &mut history,
        &map,
        &xf,
        &armed_ids
    ));
    assert!(scene.path(a).is_none() && scene.path(b).is_none());
    assert!(
        scene.path(plain).is_some(),
        "a forma que o modo nunca alcançou fica exactamente onde estava"
    );
}

/// **Sem simetria viva o `Apply` não toca no documento** — nem a geometria, nem o undo.
#[test]
fn the_apply_is_a_no_op_without_a_live_symmetry() {
    let (mut scene, sim, map, xf, id, _e) = half_profile_scene();
    let before = scene.clone();
    let mut pen = ph2d_vec_edit::PenTool::default();
    let mut history = ph2d_vec_edit::History::default();
    assert!(
        !materialise(&mut scene, &sim, &mut pen, &mut history, &map, &xf, &[id]),
        "sem componente não há o que consolidar"
    );
    assert_eq!(
        scene.paths().len(),
        before.paths().len(),
        "e o documento fica intacto"
    );
}

// ── O desenho das linhas ───────────────────────────────────────────────────────────────

/// **O eixo que o overlay desenha é o MESMO que a geometria espelha.**
///
/// ⚠️ A direcção passa por `mirror_dir()` — a porta que o kernel usa para reflectir — e sobe pela
/// pose como VETOR (`apply_vec`), não como ponto. Levá-la como ponto a transladaria, e toda linha
/// apontaria para o canto do ecrã; uma segunda derivação desenharia um eixo onde a geometria não
/// espelha, e ninguém lê um número numa screenshot.
#[test]
fn the_drawn_axis_is_the_one_the_geometry_mirrors() {
    let (scene, mut sim, map, xf, id, _e) = half_profile_scene();
    arm(&mut sim, &map, &[id], Some(mirror_x()));
    let axes = live_axes(&scene, &sim, &map, &xf);
    assert_eq!(axes.len(), 1, "uma forma com simetria, um eixo");
    let a = axes[0];
    assert!(a.segments.is_none(), "um espelho não é uma rosácea");
    // A pose da fixture é uma translação de (3,1): o ponto vai junto, a direcção NÃO.
    assert!(
        (a.at[0] - 3.0).abs() < 1e-9 && (a.at[1] - 1.0).abs() < 1e-9,
        "o ponto do eixo sobe pela pose: {:?}",
        a.at
    );
    assert!(
        a.dir[0].abs() < 1e-9 && (a.dir[1] - 1.0).abs() < 1e-9,
        "a direcção é VETOR — uma translação não a move: {:?}",
        a.dir
    );
}

/// **Uma rosácea desenha `segments` raios, não uma linha** — são figuras diferentes porque são
/// fatos diferentes, e desenhar a rosácea como uma linha só a faria parecer um espelho quebrado.
#[test]
fn a_radial_draws_spokes_and_a_mirror_draws_a_line() {
    let (scene, mut sim, map, xf, id, _e) = half_profile_scene();
    arm(
        &mut sim,
        &map,
        &[id],
        Some(SymmetrySpec {
            kind: SymmetryKind::Radial,
            segments: 7,
            ..SymmetrySpec::default()
        }),
    );
    let axes = live_axes(&scene, &sim, &map, &xf);
    assert_eq!(axes[0].segments, Some(7));
}

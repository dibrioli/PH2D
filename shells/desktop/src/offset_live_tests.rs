//! Gates do **Offset VIVO** — arquivo irmão de `offset_live.rs`.
//!
//! O fato do USUÁRIO que esta família existe para pinar (Enio, 2026-07-21):
//!
//! > *"no momento que aperto Round a curva já tem todos os vertex novos criados antes de
//! > apertar apply … a idéia é bevel desfazer round e aplicar bevel. Só apply aplica
//! > definitivamente o efeito."*
//!
//! Ou seja: enquanto o offset é PREVIEW, o documento guarda a curva AUTORADA — e é ela que o
//! modo Node mostra. O resultado é DESENHO. `Apply Offset` (ou `Convert to Curves`) é o único
//! momento em que os vértices do offset passam a existir.
//!
//! ⚠️ **O oráculo NÃO é "os três joins diferem"** — isso é verdade mesmo compondo, e foi por
//! isso que a rodada anterior ficou verde sobre o produto que o Enio reprovou. Os oráculos
//! daqui são: (a) a curva autorada BYTE-IDÊNTICA através da cadeia de previews; (b) a
//! identidade com um resultado FRESCO da forma pristina, âncora a âncora.

use super::*;
use crate::vec_entities::VecEntityMap;
use ph2d_vec_scene::{Contour, VecVertex};

/// A **rosquinha do smoke 17**, posada — a fixture tem de conter o fenômeno: é o compound com
/// furo que expôs metade dos bugs desta linha, e é onde `Side` e `Corner` têm o que fazer.
/// A entidade carrega a pose `(4, 0)`, então o cozimento atravessa a fronteira local↔mundo.
fn donut_scene() -> (
    VecScene,
    ph2d_ecs::SimWorld,
    VecEntityMap,
    VecXforms,
    VecPathId,
) {
    let mut sim = ph2d_ecs::SimWorld::default();
    let mut map = VecEntityMap::new();
    let mut scene = VecScene::new();
    let mut p = ph2d_vec_scene::rectangle([-1.2, -1.2], [1.2, 1.2]);
    p.subpaths = vec![Contour::new_closed(
        [[-0.7, -0.7], [0.7, -0.7], [0.7, 0.7], [-0.7, 0.7]]
            .map(VecVertex::corner)
            .to_vec(),
    )];
    p.fill_rule = ph2d_vec_scene::FillRule::EvenOdd;
    let id = scene.push_path(p);
    let e = sim
        .world_mut()
        .spawn((
            ph2d_ecs::Transform {
                translation: ph2d_core::Vec2::new(4.0, 0.0),
                ..ph2d_ecs::Transform::IDENTITY
            },
            ph2d_ecs::Name::new("Donut"),
            ph2d_ecs::VecPathRef(id),
        ))
        .id();
    map.insert(id, e.to_bits());
    let xf = crate::vec_transform::build(&sim, &map);
    (scene, sim, map, xf, id)
}

/// Todos os vértices AUTORADOS de um caminho (contorno primário + subpaths), na ordem.
fn authored(scene: &VecScene, id: VecPathId) -> Vec<VecVertex> {
    scene
        .path(id)
        .expect("o caminho existe")
        .verts_all()
        .copied()
        .collect()
}

/// As âncoras da geometria DESENHADA de `id` neste frame.
fn drawn(live: &crate::offset_live::OffsetLive, id: VecPathId) -> Vec<[f64; 2]> {
    live.live()
        .get(&id)
        .map(|paths| {
            paths
                .iter()
                .flat_map(|p| p.verts_all().map(|v| v.anchor))
                .collect()
        })
        .unwrap_or_default()
}

// ─────────────────────────── O fato do report ───────────────────────────

/// **A curva AUTORADA não se move através da cadeia inteira de previews.** É a frase do Enio,
/// literal: apertar Round não pode criar vértice nenhum no documento.
///
/// O gate exige as DUAS metades — senão seria vacuoso: a curva autorada byte-idêntica **e** o
/// desenho de facto mudando a cada retune. Um produto que não cozinhasse nada passaria na
/// primeira metade e falharia na segunda.
#[test]
fn the_authored_curve_never_moves_through_a_chain_of_previews() {
    let (scene, mut sim, map, xf, id) = donut_scene();
    let before = authored(&scene, id);
    let mut live = crate::offset_live::OffsetLive::default();

    crate::offset_live::arm(&mut sim, &map, &[id], 0.5, 0, 2);
    live.recook(&scene, &sim, &map, &xf);
    let mut seen: Vec<Vec<[f64; 2]>> = vec![drawn(&live, id)];

    for join in [1u8, 2u8, 0u8] {
        crate::offset_live::retune(&mut sim, &map, &[id], (join, 2));
        live.recook(&scene, &sim, &map, &xf);
        assert_eq!(
            authored(&scene, id),
            before,
            "o Corner {join} escreveu no DOCUMENTO — a curva autorada tem de ficar \
             BYTE-IDÊNTICA enquanto o offset é preview (o report de 2026-07-21)"
        );
        seen.push(drawn(&live, id));
    }
    // A 2ª metade: o desenho MUDA. Round deixa arcos (muitas âncoras), Miter/Bevel não.
    assert!(
        seen[1].len() > seen[0].len() + 8,
        "o Round tem de desenhar arcos (âncoras: {} contra {}) — sem isto o gate seria \
         verde sobre um produto que não coze nada",
        seen[1].len(),
        seen[0].len()
    );
    assert_ne!(seen[1], seen[2], "Round e Bevel têm de desenhar diferente");
}

/// **Cada preview re-deriva da FONTE, nunca do preview anterior** — *"a idéia é bevel desfazer
/// round e aplicar bevel"*. Porte do gate de `vec_expand_retune_tests.rs`, agora sobre a
/// geometria DESENHADA (que é onde o resultado vive no modelo novo).
///
/// ⚠️ O oráculo é a IDENTIDADE com um Bevel **fresco da forma pristina**, âncora a âncora
/// (1e-6) — *"os dois diferem"* passaria mesmo compondo.
#[test]
fn each_preview_re_derives_from_the_source_never_from_the_previous_preview() {
    let (scene, mut sim, map, xf, id) = donut_scene();
    let d = 0.5;

    // O oráculo, computado FORA do caminho do preview: o Bevel sozinho, da fonte pristina.
    let mut world = scene.path(id).expect("existe").clone();
    ph2d_vec_scene::bake_xform(&mut world, &ph2d_vec_scene::xform_of(&xf, id));
    let fresh: Vec<[f64; 2]> = ph2d_vec_boolean::offset_path(
        &world,
        d,
        ph2d_vec_scene::LineJoin::Bevel,
        ph2d_vec_scene::OffsetSide::Both,
    )
    .iter()
    .flat_map(|p| p.verts_all().map(|v| v.anchor))
    .collect();

    // O caminho REAL: arma com Round, depois retuna para Bevel.
    let mut live = crate::offset_live::OffsetLive::default();
    crate::offset_live::arm(&mut sim, &map, &[id], d, 1, 2);
    live.recook(&scene, &sim, &map, &xf);
    assert!(
        drawn(&live, id).len() > 8,
        "o Round tem de materializar arcos, senão o teste não descreve o report"
    );
    crate::offset_live::retune(&mut sim, &map, &[id], (2, 2));
    live.recook(&scene, &sim, &map, &xf);

    let got = drawn(&live, id);
    assert_eq!(
        got.len(),
        fresh.len(),
        "o Bevel depois do Round tem {} âncoras; um Bevel fresco tem {} — o preview COMPÔS \
         sobre o anterior em vez de re-derivar da fonte",
        got.len(),
        fresh.len()
    );
    for (g, f) in got.iter().zip(&fresh) {
        assert!(
            (g[0] - f[0]).abs() < 1e-6 && (g[1] - f[1]).abs() < 1e-6,
            "âncora {g:?} ≠ {f:?} — o Bevel não saiu da forma pristina"
        );
    }
}

// ─────────────────────────── Armar, desarmar, materializar ───────────────────────────

/// **Um `d` inerte não deixa relação pendurada.** Arrastar o slider de volta ao zero devolve a
/// forma ao estado limpo — não a um documento com um efeito invisível que não desenha nada e
/// que o `Convert to Curves` ainda ofereceria congelar.
#[test]
fn a_zero_offset_leaves_no_live_effect() {
    let (scene, mut sim, map, xf, id) = donut_scene();
    let mut live = crate::offset_live::OffsetLive::default();
    crate::offset_live::arm(&mut sim, &map, &[id], 0.5, 0, 2);
    assert!(crate::offset_live::spec_of(&sim, &map, id).is_some());
    crate::offset_live::arm(&mut sim, &map, &[id], 0.0, 0, 2);
    assert!(
        crate::offset_live::spec_of(&sim, &map, id).is_none(),
        "o `d` inerte tem de REMOVER o componente"
    );
    live.recook(&scene, &sim, &map, &xf);
    assert!(
        live.live().is_empty(),
        "sem componente não há geometria derivada — e é isso que faz o `dispatch` voltar a \
         desenhar a FONTE"
    );
}

/// **Um offset que ANIQUILA a forma desenha NADA — e não apaga a arte.** A entrada existe e
/// está VAZIA; ausente significaria "desenhe a fonte", e a forma reapareceria inteira no
/// extremo do slider. Subir o `d` a traz de volta, porque a fonte nunca saiu.
#[test]
fn an_annihilating_offset_draws_nothing_but_keeps_the_source() {
    let (scene, mut sim, map, xf, id) = donut_scene();
    let before = authored(&scene, id);
    let mut live = crate::offset_live::OffsetLive::default();

    crate::offset_live::arm(&mut sim, &map, &[id], -1.2, 0, 2);
    live.recook(&scene, &sim, &map, &xf);
    assert_eq!(
        live.live().get(&id).map(Vec::len),
        Some(0),
        "aniquilado = entrada PRESENTE e vazia (ausente faria a forma reaparecer)"
    );
    assert_eq!(before, authored(&scene, id), "a arte tem de continuar lá");

    crate::offset_live::arm(&mut sim, &map, &[id], 0.3, 0, 2);
    live.recook(&scene, &sim, &map, &xf);
    assert!(
        !drawn(&live, id).is_empty(),
        "subir o `d` tem de trazer a forma de volta"
    );
}

/// **`Apply Offset` MATERIALIZA: os vértices do offset passam a existir no documento.** É a
/// outra metade da frase do Enio — *"só apply aplica definitivamente o efeito"*. Depois dele a
/// curva autorada É o que estava desenhado, e o efeito vivo acabou.
#[test]
fn applying_the_offset_materialises_the_drawn_geometry() {
    let (mut scene, mut sim, mut map, xf, id) = donut_scene();
    let before = authored(&scene, id);
    let mut live = crate::offset_live::OffsetLive::default();
    crate::offset_live::arm(&mut sim, &map, &[id], 0.5, 1, 2);
    live.recook(&scene, &sim, &map, &xf);
    let want = drawn(&live, id);
    assert!(
        want.len() > before.len(),
        "pré-condição: o Round engorda a contagem"
    );

    let mut pen = ph2d_vec_edit::PenTool::default();
    pen.select_many(&[id]);
    let mut hist = ph2d_vec_edit::History::default();
    assert!(crate::offset_live::materialise(
        &mut scene,
        &sim,
        &mut pen,
        &mut hist,
        &map,
        &xf,
        &[id]
    ));

    // O que está na CENA agora é, âncora a âncora, o que estava DESENHADO.
    let got: Vec<[f64; 2]> = scene
        .paths()
        .iter()
        .flat_map(|p| p.verts_all().map(|v| v.anchor))
        .collect();
    assert_eq!(
        got.len(),
        want.len(),
        "o documento ficou com outra geometria"
    );
    for (g, w) in got.iter().zip(&want) {
        assert!(
            (g[0] - w[0]).abs() < 1e-9 && (g[1] - w[1]).abs() < 1e-9,
            "âncora materializada {g:?} ≠ a desenhada {w:?}"
        );
    }
    assert!(hist.undo(&scene).is_some(), "UM passo de undo para o gesto");
    // A forma-fonte saiu da cena, e com ela o componente (a entidade dela morre no `sync`).
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    assert!(
        scene
            .paths()
            .iter()
            .all(|p| crate::offset_live::spec_of(&sim, &map, p.id).is_none()),
        "materializar encerra o efeito vivo — senão o offset seria aplicado DUAS vezes"
    );
}

/// **Materializar DUAS formas honra o offset de CADA UMA.** É a razão de o `expand_selection`
/// ter passado a perguntar por-caminho: o slider é um número só, mas o documento pode carregar
/// offsets diferentes em formas diferentes (basta selecionar uma, arrastar, selecionar outra).
/// Um comando único para a seleção inteira aplicaria a quina de uma na outra, e nada pareceria
/// errado — as duas ficariam offsetadas.
///
/// ⚠️ Esta fixture existe porque a mutação *"usa o primeiro spec para todos"* SOBREVIVEU a
/// todos os gates de uma forma só. [[feedback_a_fixture_only_proves_what_it_contains]]
#[test]
fn materialising_two_shapes_honours_each_ones_own_offset() {
    let mut sim = ph2d_ecs::SimWorld::default();
    let mut map = VecEntityMap::new();
    let mut scene = VecScene::new();
    let mut ids = Vec::new();
    for (n, x) in [("A", -4.0), ("B", 4.0)] {
        let id = scene.push_path(ph2d_vec_scene::rectangle([-1.0, -1.0], [1.0, 1.0]));
        let e = sim
            .world_mut()
            .spawn((
                ph2d_ecs::Transform {
                    translation: ph2d_core::Vec2::new(x, 0.0),
                    ..ph2d_ecs::Transform::IDENTITY
                },
                ph2d_ecs::Name::new(n),
                ph2d_ecs::VecPathRef(id),
            ))
            .id();
        map.insert(id, e.to_bits());
        ids.push(id);
    }
    let xf = crate::vec_transform::build(&sim, &map);
    // A com MITER, B com ROUND — o mesmo `d`, quinas diferentes.
    crate::offset_live::arm(&mut sim, &map, &ids[..1], 0.5, 0, 2);
    crate::offset_live::arm(&mut sim, &map, &ids[1..], 0.5, 1, 2);

    // O que cada uma DESENHA, antes de materializar.
    let mut live = crate::offset_live::OffsetLive::default();
    live.recook(&scene, &sim, &map, &xf);
    let (want_a, want_b) = (drawn(&live, ids[0]).len(), drawn(&live, ids[1]).len());
    assert!(
        want_b > want_a + 8,
        "pré-condição: o Round tem de deixar MUITO mais âncoras que o Miter ({want_a} vs \
         {want_b}) — sem esse fosso o gate não distingue as duas quinas"
    );

    let mut pen = ph2d_vec_edit::PenTool::default();
    pen.select_many(&ids);
    let mut hist = ph2d_vec_edit::History::default();
    assert!(crate::offset_live::materialise(
        &mut scene, &sim, &mut pen, &mut hist, &map, &xf, &ids
    ));

    // Cada forma materializada fica com a contagem da quina DELA. A ordem na cena é a de z,
    // que o `expand_selection` preserva (cada uma substituída no lugar).
    let counts: Vec<usize> = scene
        .paths()
        .iter()
        .map(|p| p.verts_all().count())
        .collect();
    assert_eq!(counts.len(), 2, "duas formas entram, duas saem");
    assert_eq!(
        (counts[0], counts[1]),
        (want_a, want_b),
        "cada forma tem de sair com a quina DELA — {counts:?} contra o esperado \
         ({want_a}, {want_b})"
    );
}

/// **Materializar sem offset armado não é o nosso caminho** — quem responde é o botão
/// numérico de sempre. Sem esta recusa, o `Apply Offset` teria duas implementações e a de
/// baixo venceria em silêncio.
#[test]
fn materialising_without_a_live_offset_refuses() {
    let (mut scene, sim, map, xf, id) = donut_scene();
    let mut pen = ph2d_vec_edit::PenTool::default();
    pen.select_many(&[id]);
    let mut hist = ph2d_vec_edit::History::default();
    assert!(!crate::offset_live::materialise(
        &mut scene,
        &sim,
        &mut pen,
        &mut hist,
        &map,
        &xf,
        &[id]
    ));
}

/// **Um chip de Corner sobre uma forma SEM offset não inventa geometria.** O retune ajusta o
/// que existe; armar é do slider. Sem isto, clicar "Round" a passear pelo painel offsetaria a
/// seleção sozinho.
#[test]
fn retuning_a_shape_without_a_live_offset_arms_nothing() {
    let (scene, mut sim, map, xf, id) = donut_scene();
    crate::offset_live::retune(&mut sim, &map, &[id], (1, 0));
    assert!(crate::offset_live::spec_of(&sim, &map, id).is_none());
    let mut live = crate::offset_live::OffsetLive::default();
    live.recook(&scene, &sim, &map, &xf);
    assert!(live.live().is_empty());
}

// ─────────────────────────── O cozimento ───────────────────────────

/// **O memo re-coze quando a FONTE muda.** A chave é a geometria de mundo que entra, não um
/// contador de versão — editar a forma tem de refazer o offset, senão o desenho descreveria a
/// forma onde ela ESTAVA (a armadilha que a chave da sessão do Build já pagou).
#[test]
fn the_cook_follows_the_source_when_it_changes() {
    let (mut scene, mut sim, map, xf, id) = donut_scene();
    let mut live = crate::offset_live::OffsetLive::default();
    crate::offset_live::arm(&mut sim, &map, &[id], 0.4, 0, 2);
    live.recook(&scene, &sim, &map, &xf);
    let before = drawn(&live, id);

    // O artista arrasta uma âncora no modo Node — a fonte É editável durante o preview.
    if let Some(p) = scene.path_mut(id) {
        p.verts[0].anchor[0] -= 0.6;
    }
    live.recook(&scene, &sim, &map, &xf);
    assert_ne!(
        before,
        drawn(&live, id),
        "editar a curva autorada tem de refazer o desenho — um memo cego a ela mostraria a \
         forma onde ela ESTAVA"
    );
}

/// **A pilha de efeitos corre ANTES do offset, e ANTES da pose.** A ordem que o módulo
/// declara: quina → pilha → assar a pose → offset.
///
/// ⚠️ **O oráculo não pode ser "com efeito desenha diferente de sem efeito"** — isso é verdade
/// por DUAS vias, e uma delas não é minha: o `to_bez_with` da booleana também chama `cooked()`,
/// então o efeito chegaria ao desenho mesmo que este módulo passasse a fonte crua
/// ([[feedback_layered_defenses_need_per_layer_gates]] — a mutação "ler `verts` cru"
/// SOBREVIVEU a esse oráculo). O que só ESTA camada decide é *em que espaço o efeito é
/// medido*: o `FxCtx::ref_size` sai da caixa de controle do caminho que ENTRA, então cozer
/// depois de assar a pose mede a onda no espaço errado. O oráculo é a identidade com
/// `offset(assar(cozer(fonte)))`.
///
/// ⚠️ **A pose tem de ser NÃO-UNIFORME, e isso é a fixture contendo o fenómeno.** Sob escala
/// uniforme os dois espaços dão o MESMO desenho — de propósito: o `Size` do Zig Zag é uma
/// percentagem da forma, e `ref_size` cresce junto com ela. Medido: com `scale (3,3)` a
/// mutação "ler `verts` cru" **sobrevive**; com `scale (3,1)` ela sangra, 36 âncoras contra
/// as 24 do oráculo. Uma fixture uniforme seria verde sobre o defeito.
#[test]
fn the_cook_runs_the_effect_stack_before_the_pose_and_the_offset() {
    let mut sim = ph2d_ecs::SimWorld::default();
    let mut map = VecEntityMap::new();
    let mut scene = VecScene::new();
    let id = scene.push_path(ph2d_vec_scene::rectangle([-1.0, -1.0], [1.0, 1.0]));
    // Uma pose com ESCALA: é ela que separa "cozer e então assar" de "assar e então cozer".
    let e = sim
        .world_mut()
        .spawn((
            ph2d_ecs::Transform {
                translation: ph2d_core::Vec2::new(3.0, 0.0),
                scale: ph2d_core::Vec2::new(3.0, 1.0),
                ..ph2d_ecs::Transform::IDENTITY
            },
            ph2d_ecs::Name::new("Scaled"),
            ph2d_ecs::VecPathRef(id),
        ))
        .id();
    map.insert(id, e.to_bits());
    let xf = crate::vec_transform::build(&sim, &map);
    // Um Zig Zag ativo pelo MESMO caminho do produto.
    crate::fx_bridge::add(&mut scene, id, 1);
    crate::fx_bridge::set_param(&mut scene, id, 0, 0, 40.0);

    // O oráculo, computado FORA do módulo: cozer em LOCAL, assar a pose, offsetar.
    let mut want_src = scene.path(id).expect("existe").cooked().into_owned();
    ph2d_vec_scene::bake_xform(&mut want_src, &ph2d_vec_scene::xform_of(&xf, id));
    let want: Vec<[f64; 2]> = ph2d_vec_boolean::offset_path(
        &want_src,
        0.3,
        ph2d_vec_scene::LineJoin::Miter,
        ph2d_vec_scene::OffsetSide::Both,
    )
    .iter()
    .flat_map(|p| p.verts_all().map(|v| v.anchor))
    .collect();

    let mut live = crate::offset_live::OffsetLive::default();
    crate::offset_live::arm(&mut sim, &map, &[id], 0.3, 0, 2);
    live.recook(&scene, &sim, &map, &xf);
    let got = drawn(&live, id);

    assert!(
        !want.is_empty(),
        "pré-condição: o oráculo tem de produzir geometria"
    );
    assert_eq!(
        got.len(),
        want.len(),
        "o desenho tem {} âncoras e o oráculo {} — a pilha de efeitos foi avaliada no espaço \
         errado (o `ref_size` mede a caixa do que ENTRA)",
        got.len(),
        want.len()
    );
    for (g, w) in got.iter().zip(&want) {
        assert!(
            (g[0] - w[0]).abs() < 1e-9 && (g[1] - w[1]).abs() < 1e-9,
            "âncora desenhada {g:?} ≠ a do oráculo {w:?}"
        );
    }
}

/// **O memo não sobrevive ao `forget`.** O restore de undo e o load de projeto RECICLAM os
/// `VecPathId`: um acerto de memo desenharia o offset da forma anterior sobre a nova, sem erro
/// nenhum.
#[test]
fn forgetting_drops_the_memo_and_the_drawing() {
    let (scene, mut sim, map, xf, id) = donut_scene();
    let mut live = crate::offset_live::OffsetLive::default();
    crate::offset_live::arm(&mut sim, &map, &[id], 0.4, 0, 2);
    live.recook(&scene, &sim, &map, &xf);
    assert!(!live.live().is_empty());
    live.forget();
    assert!(live.live().is_empty());
}

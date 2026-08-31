//! Testes de [`super`] (`vec_shape_params.rs`) — o alvo do painel, o campo genérico, a
//! fronteira de unidade (px ↔ mundo) e o re-cook in-place. Extraídos para um módulo irmão
//! (`#[path]`) sob o teto de LOC da shell (HR-18).
//!
//! Os testes são **sobre o catálogo**, não sobre formas nomeadas: uma forma nova entra na
//! tabela e já nasce coberta por eles.

use super::*;
use crate::vec_shape_live::recook_shape;
use ph2d_ecs::Transform;
use ph2d_tool_vector::DrawMode;
use ph2d_tool_vector::shapes::FieldUnit;
use ph2d_vec_scene::ALL_SHAPES;

/// Uma forma viva no mundo: path na cena + entidade + `VecShape`, com uma POSE (o "move
/// do usuário") para provar que o re-cook não a perde.
fn live_shape(
    kind: ShapeKind,
    values: ShapeValues,
    pose: [f32; 2],
) -> (SimWorld, VecScene, VecEntityMap, VecPathId) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let shape = VecShape::Param {
        kind: kind.as_u16(),
        w: 4.0,
        h: 4.0,
        values,
    };
    let id = scene.push_path(recook_shape(&shape).expect("forma paramétrica cozinha"));
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let entity = Entity::from_bits(*map.get(&id).expect("a entidade do path"));
    if let Ok(mut e) = sim.world_mut().get_entity_mut(entity) {
        if let Some(mut t) = e.get_mut::<Transform>() {
            t.translation = ph2d_core::Vec2::new(pose[0], pose[1]);
        }
        e.insert(shape);
    }
    (sim, scene, map, id)
}

fn anchors(scene: &VecScene, id: VecPathId) -> Vec<[f64; 2]> {
    scene
        .paths()
        .iter()
        .find(|p| p.id == id)
        .expect("o path")
        .verts_all()
        .map(|v| v.anchor)
        .collect()
}

fn field(i: usize) -> NodeId {
    ph2d_editor::ids::vector_shape_field_id(i)
}

/// O CORAÇÃO do ciclo paramétrico: mexer num campo de um polígono VIVO selecionado
/// re-cozinha a forma **no lugar** — a contagem de âncoras muda, mas o id do path, a pose
/// (`Transform`) e o centro da geometria ficam. Sem isso, "live shape" não existe: um
/// polígono de 5 lados nunca viraria de 7.
#[test]
fn a_shape_field_recooks_the_selected_live_shape_in_place() {
    let pose = [12.0, -7.0];
    let (mut sim, mut scene, map, id) =
        live_shape(ShapeKind::Polygon, ShapeKind::Polygon.defaults(), pose);
    let before = anchors(&scene, id).len();

    let edited = edit_selected_shape(
        &mut sim,
        &mut scene,
        &map,
        &[id],
        DrawMode::Select,
        false,
        |k, v| {
            apply_shape_field(k, v, field(0), 7.0, 1.0) // 7 lados
        },
    );

    assert!(edited, "havia forma viva selecionada");
    let (_, _, kind, values) =
        panel_shape_target(&sim, &map, &[id]).expect("segue sendo forma viva");
    assert_eq!(kind, ShapeKind::Polygon);
    assert!(
        (values[0] - 7.0).abs() < 1e-9,
        "o parâmetro foi para a entidade"
    );
    let after = anchors(&scene, id);
    assert_ne!(
        after.len(),
        before,
        "a geometria re-cozinhou (7 != 5 ancoras)"
    );
    assert_eq!(
        scene.paths().len(),
        1,
        "re-cook IN-PLACE: nada de path novo"
    );
    // A pose é do `Transform` — o re-cook nunca a toca (senão a forma pularia).
    let e = Entity::from_bits(*map.get(&id).expect("entidade"));
    let t = sim.world().get::<Transform>(e).expect("Transform");
    assert!(
        (t.translation.x - pose[0]).abs() < 1e-6 && (t.translation.y - pose[1]).abs() < 1e-6,
        "a pose (o move do usuario) sobreviveu ao re-cook"
    );
}

/// Um campo que não existe na forma selecionada não a toca (o painel nem o desenha, mas o
/// caminho recusa por construção).
#[test]
fn a_field_that_the_shape_does_not_have_leaves_it_alone() {
    // O retângulo não declara parâmetro nenhum: TODO campo tem de ser recusado.
    let (mut sim, mut scene, map, id) = live_shape(
        ShapeKind::Rectangle,
        ShapeKind::Rectangle.defaults(),
        [0.0, 0.0],
    );
    let before = anchors(&scene, id);
    let edited = edit_selected_shape(
        &mut sim,
        &mut scene,
        &map,
        &[id],
        DrawMode::Select,
        false,
        |k, v| apply_shape_field(k, v, field(0), 9.0, 1.0),
    );
    assert!(!edited, "o retangulo nao tem campo 0");
    assert_eq!(anchors(&scene, id), before, "a geometria nao foi re-cozida");
}

/// **Gate anti-campo-morto:** para TODA forma do catálogo, cada campo que ela declara move
/// de fato a geometria dela (dois valores distintos ⇒ formas distintas). Um campo novo que
/// o `cook` ignore fica editável, verde no CI, e morto na tela.
#[test]
fn every_declared_field_of_every_shape_moves_its_geometry() {
    for &k in ALL_SHAPES {
        let d = ph2d_tool_vector::shapes::desc(k);
        for (i, f) in d.fields.iter().enumerate() {
            let mut lo = k.defaults();
            let mut hi = k.defaults();
            assert!(
                apply_shape_field(k, &mut lo, field(i), f.min, 1.0)
                    && apply_shape_field(k, &mut hi, field(i), f.max, 1.0),
                "{k:?}.{}: o campo nao foi aceito",
                f.label
            );
            // A geometria são as âncoras E os handles: um parâmetro que só encurva (o
            // bico da gota, a cintura da chave) mexe nos handles sem mover uma âncora —
            // comparar só âncoras acusaria um campo vivo de estar morto.
            let cook = |v: ShapeValues| {
                let s = VecShape::Param {
                    kind: k.as_u16(),
                    w: 4.0,
                    h: 4.0,
                    values: v,
                };
                let p = recook_shape(&s).expect("cozinha");
                p.verts_all()
                    .map(|x| (x.anchor, x.in_handle, x.out_handle))
                    .collect::<Vec<_>>()
            };
            assert_ne!(
                cook(lo),
                cook(hi),
                "{k:?}.{}: os extremos do campo dao a MESMA geometria — o parametro nao faz nada",
                f.label
            );
        }
    }
}

/// A fronteira de unidade fecha para TODO campo `Px` de TODA forma: a caixa do painel fala
/// **pixels** (é o que o usuário digita — a unidade de mundo é pequena demais: a viewport
/// inteira tem ~10 unidades), a forma guarda **mundo**, e o ida-e-volta não pode mover o
/// número (senão ele saltaria de escala a cada clique).
#[test]
fn every_px_field_round_trips_across_the_unit_boundary() {
    const PTW: f64 = 0.01; // ~1000 px de tela = 10 unidades de mundo
    for &k in ALL_SHAPES {
        let d = ph2d_tool_vector::shapes::desc(k);
        for (i, f) in d.fields.iter().enumerate() {
            if f.unit != FieldUnit::Px {
                continue;
            }
            let mut world = k.defaults();
            assert!(apply_shape_field(k, &mut world, field(i), 30.0, PTW));
            assert!(
                (world[i] - 30.0 * PTW).abs() < 1e-9,
                "{k:?}.{}: guardado em MUNDO (px x px_to_world)",
                f.label
            );
            let ui = ui_values_of(k, &world, PTW);
            assert!(
                (ui[i] - 30.0).abs() < 1e-9,
                "{k:?}.{}: voltou a 30 px",
                f.label
            );
        }
    }
}

/// O ALVO dos campos de forma pula o TEXTO (que tem a seção própria) e ignora path cru.
#[test]
fn the_panel_target_is_the_live_parametric_shape_only() {
    let (sim, _scene, map, id) = live_shape(
        ShapeKind::Ellipse,
        ShapeKind::Ellipse.defaults(),
        [0.0, 0.0],
    );
    assert!(
        panel_shape_target(&sim, &map, &[id]).is_some(),
        "uma elipse viva e alvo"
    );
    assert!(
        panel_shape_target(&sim, &map, &[]).is_none(),
        "sem selecao, sem alvo"
    );
}

/// **RED-FIRST — trocar de FORMA no catálogo re-semeia os campos.**
///
/// ⚠️ O report do Enio: *"quando se escolhe outra shape, quase todos os parâmetros não são
/// atualizados … mostram os parâmetros da outra shape previamente modificada"*. O mecanismo é
/// que os slots do store são por **ÍNDICE** (`vector_shape_field_id(i)`), compartilhados entre
/// TODAS as formas, e a semente só disparava quando o OBJETO selecionado mudava — nunca quando o
/// tipo do catálogo mudava. Então o slot 0 seguia com o número da forma anterior.
///
/// O oráculo é o STORE, que é a fonte que o painel pinta.
#[test]
fn switching_the_catalog_shape_reseeds_the_fields() {
    // ⚠️ O store REGISTRADO, não um vazio: `set_number_value` só alcança um widget que o
    // `populate` do painel criou, e é esse o fluxo real. Um store vazio faria o gate falhar
    // (ou passar) por um motivo que não tem nada a ver com a semente.
    let mut store = WidgetStore::default();
    <ph2d_panel_vector::VectorPanel as ph2d_editor::panel::Panel>::populate(&mut store);
    let id0 = ph2d_editor::ids::vector_shape_field_id(0);

    // A forma A é semeada e o artista mexe no 1º campo.
    let a = ShapeKind::Star;
    let mut va = ShapeValues::default();
    va[0] = 9.0;
    seed_shape_fields(&mut store, a, &va);
    assert_eq!(store.number_value(id0), Some(9.0));

    // Agora ele escolhe a forma B no catálogo, que tem OUTRO valor no mesmo slot.
    let b = ShapeKind::Polygon;
    let mut vb = ShapeValues::default();
    vb[0] = 3.0;
    seed_shape_fields(&mut store, b, &vb);
    assert_eq!(
        store.number_value(id0),
        Some(3.0),
        "o campo continua mostrando o numero da forma ANTERIOR"
    );
}

/// **Quem manda na semente: o ALVO se houver, senão o CATÁLOGO** — e o par é o que decide
/// re-semear.
///
/// ⚠️ A memo antiga guardava só o `VecPathId`, então dois estados diferentes — *"nada
/// selecionado, catálogo em Star"* e *"nada selecionado, catálogo em Polygon"* — comparavam
/// IGUAIS (`None == None`) e a semente nunca corria.
#[test]
fn the_seed_focus_is_the_target_or_the_catalog() {
    let cat = ShapeKind::Polygon;
    let tgt: VecPathId = 7;
    assert_eq!(
        shape_seed_focus(Some((tgt, ShapeKind::Star)), cat),
        (Some(tgt), ShapeKind::Star),
        "com alvo, manda o alvo"
    );
    assert_eq!(
        shape_seed_focus(None, cat),
        (None, cat),
        "sem alvo, manda o catalogo"
    );
    // …e é por isso que a memo tem de ser o PAR: sem o tipo, estes dois são o mesmo estado.
    assert_ne!(
        shape_seed_focus(None, ShapeKind::Star),
        shape_seed_focus(None, ShapeKind::Polygon)
    );
}

/// ⭐⭐⭐ **O BUG (Enio, 2026-08-31):** *"Troco de Shape na tool Shape e as propriedades da shape
/// não trocam imediatamente."*
///
/// Desenhar deixa a forma nova SELECIONADA (`input_dispatch`, no `Up` do gesto). O alvo vivo
/// vencia sempre, então escolher outra forma no catálogo — que põe a tool em
/// [`DrawMode::Shape`] — deixava o painel a mostrar os parâmetros da forma **anterior** até
/// alguém desenhar a nova. ⚠️ **A regra já estava escrita** no doc do `shape_focus` do painel
/// (*"no modo Shape … valem mesmo com algo selecionado"*) e **não era alcançável**: o
/// `published.or_else(…)` lê o alvo primeiro.
#[test]
fn arming_a_shape_hands_the_fields_to_the_catalog_even_with_a_live_shape_selected() {
    let (sim, _scene, map, id) = live_shape(
        ShapeKind::Polygon,
        ShapeKind::Polygon.defaults(),
        [0.0, 0.0],
    );
    let alvo = |mode, armed| shape_field_target(&sim, &map, &[id], mode, armed).is_some();

    // Armado E a desenhar: os campos são o default do PRÓXIMO traço.
    assert!(
        !alvo(DrawMode::Shape, true),
        "armado para desenhar, o painel tem de mostrar a forma ARMADA"
    );
    // Em Select o ciclo Live Shape manda, mesmo com o latch ainda aceso: o artista trocou de
    // ferramenta, e ali não há forma armada nenhuma a mostrar.
    assert!(
        alvo(DrawMode::Select, true),
        "sem isto o ciclo Live Shape morre"
    );
    // …e na MOLDURA também: ali o gesto desenha um RoundRect, e um RoundRect selecionado é o
    // mesmo objeto. A cerca é o modo Shape, não «todo modo que desenha forma».
    assert!(alvo(DrawMode::Frame, true));
}

/// ⚠️⚠️ **O MODO SOZINHO NÃO RESPONDE, e esta é a metade que a 1.ª redacção da cura partia.**
///
/// *"Desenhei uma estrela, deixa-me ajustar as pontas dela"* e *"armei o Polígono, mostra-me o
/// Polígono"* são **os dois** `DrawMode::Shape` com uma forma viva selecionada — o que os separa é
/// qual gesto veio por último. Uma cerca só no modo teria matado o ciclo Live Shape dentro da
/// própria ferramenta que o cria, e nenhum gate desta suíte teria dito nada.
#[test]
fn drawing_a_shape_gives_the_fields_back_to_it_without_leaving_the_shape_tool() {
    let (sim, _scene, map, id) = live_shape(
        ShapeKind::Polygon,
        ShapeKind::Polygon.defaults(),
        [0.0, 0.0],
    );
    assert!(
        shape_field_target(&sim, &map, &[id], DrawMode::Shape, false).is_some(),
        "sem latch aceso, a forma viva manda mesmo no modo Shape — e desenhar apaga o latch \
         (a selecao muda para o path novo)"
    );
}

/// ⚠️⚠️ **A METADE DA ESCRITA, e ela é a perigosa.** Os slots do painel são por ÍNDICE
/// (`vector_shape_field_id(i)`), partilhados por todas as formas — então com a pintura a mostrar
/// a Estrela armada e a escrita a alcançar o Polígono selecionado, digitar *"Pontas = 9"* punha
/// **9 lados** no polígono, sem erro nenhum. *Pintar por uma porta e escrever por outra é o
/// defeito; a divergência de valores é só o sintoma.*
#[test]
fn an_armed_artist_does_not_write_into_the_shape_that_is_still_selected() {
    let (mut sim, mut scene, map, id) = live_shape(
        ShapeKind::Polygon,
        ShapeKind::Polygon.defaults(),
        [0.0, 0.0],
    );
    let before = anchors(&scene, id);

    let touched = edit_selected_shape(
        &mut sim,
        &mut scene,
        &map,
        &[id],
        DrawMode::Shape,
        true,
        |k, v| apply_shape_field(k, v, field(0), 9.0, 1.0),
    );
    assert!(
        !touched,
        "armado para desenhar, a caixa NAO edita a selecao"
    );
    assert_eq!(
        anchors(&scene, id),
        before,
        "a geometria da selecao nao move"
    );

    // Controle: a MESMA escrita com o latch APAGADO alcança a forma — senão este gate passaria
    // por o caminho estar morto, e não pela cerca.
    assert!(edit_selected_shape(
        &mut sim,
        &mut scene,
        &map,
        &[id],
        DrawMode::Shape,
        false,
        |k, v| apply_shape_field(k, v, field(0), 9.0, 1.0),
    ));
    assert_ne!(anchors(&scene, id).len(), before.len());
}

/// ⚠️ **O clique que RE-ARMA a forma já acesa também conta.** O sinal é *"o artista carregou no
/// catálogo"*, não *"o tipo mudou"* — um diff de valor perderia o gesto de voltar a ver o que se
/// vai desenhar, que é exactamente o report.
#[test]
fn clicking_the_shape_that_is_already_active_still_arms_it() {
    let mut t = ph2d_tool_vector::VectorTool::new();
    t.set_shape(ShapeKind::Star);
    assert!(t.take_shape_armed(), "o 1o clique arma");
    assert!(
        !t.take_shape_armed(),
        "e o evento drena — e' um EVENTO, nao um estado"
    );
    t.set_shape(ShapeKind::Star); // a MESMA forma
    assert!(
        t.take_shape_armed(),
        "re-armar a forma ja' acesa e' um gesto, e um diff de valor nao o ve"
    );
}

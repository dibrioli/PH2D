//! Gates da **LARGURA VIVA** — arquivo irmão de `profile_live.rs` (ADR-0145).
//!
//! O fato que esta família pina: os quatro sliders deixaram de ser parâmetros de um comando e
//! passaram a AUTORAR um perfil que se vê antes de assar. Os oráculos são (a) o preview e o Apply
//! produzirem a MESMA geometria, âncora a âncora — a promessa da porta única —, e (b) o neutro ser
//! a AUSÊNCIA do componente, não um perfil de multiplicadores `1.0`.

use super::*;
use crate::vec_entities::VecEntityMap;
use ph2d_vec_scene::{Rgba8, StrokeSpec, VecVertex};

/// Um traço ABERTO, posado fora da origem: a fixture tem de atravessar a fronteira local↔mundo
/// (a pose entra na geometria antes do motor), e tem de ter TRAÇO — uma forma sem traço não tem
/// fita, que é um caso próprio e testado à parte.
fn stroked_scene() -> (
    VecScene,
    ph2d_ecs::SimWorld,
    VecEntityMap,
    VecXforms,
    VecPathId,
) {
    let mut sim = ph2d_ecs::SimWorld::default();
    let mut map = VecEntityMap::new();
    let mut scene = VecScene::new();
    let mut p = VecPath {
        verts: [[-2.0, 0.0], [0.0, 1.0], [2.0, 0.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: false,
        ..VecPath::default()
    };
    p.stroke = Some(StrokeSpec::new(Rgba8::new(20, 20, 20, 255), 0.4));
    let id = scene.push_path(p);
    let e = sim
        .world_mut()
        .spawn((
            ph2d_ecs::Transform {
                translation: ph2d_core::Vec2::new(4.0, -3.0),
                ..ph2d_ecs::Transform::IDENTITY
            },
            ph2d_ecs::Name::new("Traço"),
            ph2d_ecs::VecPathRef(id),
        ))
        .id();
    map.insert(id, e.to_bits());
    let xf = crate::vec_transform::build(&sim, &map);
    (scene, sim, map, xf, id)
}

/// Um afinamento nas duas pontas — o perfil que o painel oferece por default.
fn taper() -> WidthStops {
    WidthProfile {
        start: 0.25,
        mid: 1.6,
        end: 0.25,
        position: 0.5,
    }
    .to_stops()
}

/// As âncoras da geometria DESENHADA de `id` neste frame.
fn drawn(live: &ProfileLive, id: VecPathId) -> Vec<[f64; 2]> {
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

/// **O PREVIEW É O QUE O APPLY ASSA.** O gate central do ADR-0145 §3: uma segunda rota faria a
/// forma SALTAR no instante do clique, que é o defeito que o ADR-0128 pagou cinco vezes.
///
/// O oráculo é âncora a âncora, com `==` em `f64`: um épsilon aqui aceitaria um "aproximador só
/// para o preview", que é exatamente o que este gate existe para recusar.
#[test]
fn the_preview_is_exactly_what_apply_bakes() {
    let (mut scene, mut sim, map, xf, id) = stroked_scene();
    arm(&mut sim, &map, &[id], &taper());

    let mut live = ProfileLive::default();
    live.recook(&scene, &sim, &map, &xf);
    let preview = drawn(&live, id);
    assert!(!preview.is_empty(), "o preview não desenhou nada");

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
    let baked: Vec<[f64; 2]> = scene
        .paths()
        .iter()
        .flat_map(|p| p.verts_all().map(|v| v.anchor))
        .collect();

    assert_eq!(
        preview, baked,
        "a fita do preview e a assada divergem — o Apply faria a forma saltar"
    );
}

/// **O documento NÃO é tocado enquanto o perfil é preview.** A curva que o modo Node edita
/// continua sendo a autorada, byte a byte, com o perfil armado e cozido.
#[test]
fn the_authored_curve_survives_the_preview_untouched() {
    let (scene, mut sim, map, xf, id) = stroked_scene();
    let before: Vec<VecVertex> = scene
        .path(id)
        .expect("existe")
        .verts_all()
        .copied()
        .collect();

    arm(&mut sim, &map, &[id], &taper());
    let mut live = ProfileLive::default();
    live.recook(&scene, &sim, &map, &xf);
    live.recook(&scene, &sim, &map, &xf); // e um 2º frame não deriva nada

    let after: Vec<VecVertex> = scene
        .path(id)
        .expect("existe")
        .verts_all()
        .copied()
        .collect();
    assert_eq!(before, after, "o preview escreveu no documento");
}

/// **O neutro é a AUSÊNCIA.** Um perfil uniforme REMOVE o componente em vez de guardar
/// multiplicadores `1.0` — senão um documento acumularia relações invisíveis que não desenham
/// nada, e "arrastar de volta ao neutro" deixaria um efeito pendurado.
#[test]
fn a_uniform_profile_removes_the_component_instead_of_storing_it() {
    let (scene, mut sim, map, xf, id) = stroked_scene();
    arm(&mut sim, &map, &[id], &taper());
    assert!(spec_of(&sim, &map, id).is_some(), "não armou");

    arm(
        &mut sim,
        &map,
        &[id],
        &ph2d_vec_scene::WidthProfile::UNIFORM.to_stops(),
    );
    assert!(
        spec_of(&sim, &map, id).is_none(),
        "o perfil uniforme ficou guardado"
    );

    let mut live = ProfileLive::default();
    live.recook(&scene, &sim, &map, &xf);
    assert!(
        drawn(&live, id).is_empty(),
        "sem componente ainda há geometria derivada"
    );
}

/// **Uma forma SEM TRAÇO não desaparece.** `power_stroke` devolve vazio sem tinta a moldar, e uma
/// entrada vazia na `LiveGeometry` substituiria a forma por NADA. A ausência de entrada é o que
/// manda o `dispatch` desenhar a fonte — que é exatamente o que ela é.
///
/// ⚠️ Este é o ponto em que a lei diverge do irmão do offset (lá, vazio é a aniquilação e
/// desenhar nada é a resposta certa); copiar aquele ramo aqui apagaria a arte da tela.
#[test]
fn a_shape_without_a_stroke_is_still_drawn() {
    let (mut scene, mut sim, map, xf, id) = stroked_scene();
    scene.path_mut(id).expect("existe").stroke = None;
    arm(&mut sim, &map, &[id], &taper());

    let mut live = ProfileLive::default();
    live.recook(&scene, &sim, &map, &xf);
    assert!(
        live.live().get(&id).is_none(),
        "a forma sem traço foi substituída por uma fita vazia — ela some da tela"
    );
}

/// **O memo não re-cozinha uma cena parada** — e é ele que torna o preview por-frame viável
/// (`power_stroke` custa 0,26–0,55 ms por forma). O oráculo é a IDENTIDADE do resultado entre
/// frames: um memo que não pega devolveria geometria nova (igual em valor), então o que se
/// afirma é que a saída não muda e que o memo sobrevive à ausência de mudança.
#[test]
fn a_still_scene_keeps_its_cooked_ribbon() {
    let (scene, mut sim, map, xf, id) = stroked_scene();
    arm(&mut sim, &map, &[id], &taper());
    let mut live = ProfileLive::default();
    live.recook(&scene, &sim, &map, &xf);
    let first = drawn(&live, id);
    live.recook(&scene, &sim, &map, &xf);
    assert_eq!(first, drawn(&live, id), "a fita mudou numa cena parada");
}

/// **Trocar as paradas re-cozinha.** O memo é chaveado no que de facto determina a saída; se ele
/// ignorasse as paradas, arrastar um slider não mudaria um pixel — o bug que o memo do FX raster
/// pagou (a cor do fill não chegava à tela).
#[test]
fn changing_the_stops_recooks() {
    let (scene, mut sim, map, xf, id) = stroked_scene();
    arm(&mut sim, &map, &[id], &taper());
    let mut live = ProfileLive::default();
    live.recook(&scene, &sim, &map, &xf);
    let thin = drawn(&live, id);

    arm(
        &mut sim,
        &map,
        &[id],
        &WidthProfile {
            start: 2.0,
            mid: 0.3,
            end: 2.0,
            position: 0.5,
        }
        .to_stops(),
    );
    live.recook(&scene, &sim, &map, &xf);
    assert_ne!(thin, drawn(&live, id), "o perfil novo não chegou à fita");
}

/// **A pose entra na fita.** Mover a forma re-cozinha: a geometria de mundo é parte da chave, e
/// sem ela a fita ficaria desenhada onde a forma ESTAVA.
#[test]
fn moving_the_shape_moves_the_ribbon() {
    let (scene, mut sim, map, _xf, id) = stroked_scene();
    arm(&mut sim, &map, &[id], &taper());
    let mut live = ProfileLive::default();
    let xf0 = crate::vec_transform::build(&sim, &map);
    live.recook(&scene, &sim, &map, &xf0);
    let at_rest = drawn(&live, id);

    let e = ph2d_ecs::Entity::from_bits(*map.get(&id).expect("mapeada"));
    if let Some(mut t) = sim.world_mut().get_mut::<ph2d_ecs::Transform>(e) {
        t.translation.x += 5.0;
    }
    let xf1 = crate::vec_transform::build(&sim, &map);
    live.recook(&scene, &sim, &map, &xf1);
    let moved = drawn(&live, id);

    assert_eq!(at_rest.len(), moved.len());
    let dx = moved[0][0] - at_rest[0][0];
    assert!(
        (dx - 5.0).abs() < 1e-9,
        "a fita andou {dx} em vez dos 5.0 da pose"
    );
}

/// **O preset atravessa os sliders e volta o MESMO.** Os quatro knobs são a face do perfil, e o
/// espelho da seleção os reescreve — um round-trip com perda faria a forma mudar sozinha ao ser
/// re-selecionada.
///
/// ⚠️ **A fixture REGISTA os quatro sliders**, e a premissa é load-bearing: `set_slider_value` é
/// um no-op num id que não está no store (*"No-op if `id` is not a registered slider"*), então
/// sem o registo o round-trip leria os DEFAULTS e passaria dizendo o contrário do que afirma. É a
/// mesma cicatriz do `MockPanelHost::new()` que pulava o `populate`. Quem prova que o painel de
/// facto os regista é o seam do `ph2d-panel-vector`; aqui isso é premissa, não asserção.
#[test]
fn the_preset_round_trips_through_the_sliders() {
    let mut store = ph2d_editor::WidgetStore::default();
    for id in [
        ph2d_editor::ids::VECTOR_EXPAND_W_START,
        ph2d_editor::ids::VECTOR_EXPAND_W_MID,
        ph2d_editor::ids::VECTOR_EXPAND_W_END,
        ph2d_editor::ids::VECTOR_EXPAND_W_POS,
    ] {
        store.register(
            id,
            ph2d_editor::interaction::InteractiveState::Slider {
                state: ph2d_editor::widget::SliderState::Normal,
                value: 0.0,
                orientation: ph2d_editor::widget::SliderOrientation::Horizontal,
            },
        );
    }
    let p = WidthProfile {
        start: 0.4,
        mid: 2.1,
        end: 0.9,
        position: 0.3,
    };
    write_preset_to_store(&mut store, &p);
    let back = preset_from_store(&store);
    for (a, b, name) in [
        (p.start, back.start, "start"),
        (p.mid, back.mid, "mid"),
        (p.end, back.end, "end"),
        (p.position, back.position, "position"),
    ] {
        assert!((a - b).abs() < 1e-6, "{name}: {a} virou {b}");
    }
}

/// **As paradas voltam a ser um preset** quando ELAS são um preset — é o que permite ao espelho
/// da seleção mostrar nos knobs o perfil da forma escolhida.
#[test]
fn a_three_stop_list_reads_back_as_its_preset() {
    let p = WidthProfile {
        start: 0.25,
        mid: 1.6,
        end: 0.25,
        position: 0.35,
    };
    let back = preset_of(&p.to_stops()).expect("três paradas são um preset");
    assert!((back.start - p.start).abs() < 1e-12);
    assert!((back.mid - p.mid).abs() < 1e-12);
    assert!((back.end - p.end).abs() < 1e-12);
    assert!((back.position - p.position).abs() < 1e-12);
}

/// **Um perfil de alças arbitrárias NÃO vira quatro números.** Inventar quatro faria os sliders
/// mentirem sobre a forma; o espelho então deixa os knobs onde estão.
#[test]
fn an_arbitrary_stop_list_has_no_preset() {
    let s = WidthStops::new(vec![
        ph2d_vec_scene::WidthStop {
            pos: 0.0,
            mult: 1.0,
        },
        ph2d_vec_scene::WidthStop {
            pos: 0.2,
            mult: 2.0,
        },
        ph2d_vec_scene::WidthStop {
            pos: 0.6,
            mult: 0.4,
        },
        ph2d_vec_scene::WidthStop {
            pos: 1.0,
            mult: 1.0,
        },
    ]);
    assert!(preset_of(&s).is_none(), "quatro paradas viraram um preset");
}

/// **Uma forma com PREENCHIMENTO e traço rende DUAS camadas** — o miolo continua existindo (sem
/// traço) e a fita entra por cima. É a regra que o Illustrator segue, e é onde ela aparece: se a
/// camada de baixo fosse esquecida, engrossar um disco APAGARIA o preenchimento dele.
///
/// ⚠️ A regra mora numa função só (`vec_expand::ink_layers`), então este gate e o do Apply falam
/// da mesma lei — e é o gate `the_preview_is_exactly_what_apply_bakes` que garante que continuam
/// falando dela juntos.
#[test]
fn a_filled_and_stroked_shape_keeps_its_middle() {
    let mut sim = ph2d_ecs::SimWorld::default();
    let mut map = VecEntityMap::new();
    let mut scene = VecScene::new();
    let mut disc = ph2d_vec_scene::ellipse([0.0, 0.0], 1.0, 1.0);
    disc.fill = Some(ph2d_vec_scene::Paint::solid(Rgba8::new(200, 200, 230, 255)));
    disc.stroke = Some(StrokeSpec::new(Rgba8::new(20, 20, 20, 255), 0.2));
    let id = scene.push_path(disc);
    let e = sim
        .world_mut()
        .spawn((
            ph2d_ecs::Transform::IDENTITY,
            ph2d_ecs::Name::new("Disco"),
            ph2d_ecs::VecPathRef(id),
        ))
        .id();
    map.insert(id, e.to_bits());
    let xf = crate::vec_transform::build(&sim, &map);

    arm(&mut sim, &map, &[id], &taper());
    let mut live = ProfileLive::default();
    live.recook(&scene, &sim, &map, &xf);
    let layers = live.live().get(&id).expect("a fita foi cozida");
    assert!(
        layers.len() >= 2,
        "a fita substituiu o disco INTEIRO — o preenchimento sumiu ({} camada[s])",
        layers.len()
    );
    assert!(
        layers[0].fill.is_some() && layers[0].stroke.is_none(),
        "a camada de baixo não é o miolo sem traço"
    );
}

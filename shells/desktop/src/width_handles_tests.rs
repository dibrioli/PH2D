//! Gates do **WIDTH TOOL** — arquivo irmão de `width_handles.rs` (plano 25 §5).
//!
//! Os oráculos são do GESTO, não da fórmula: *a alça pousa na borda da fita*, *arrastar para fora
//! engrossa*, *o clique na curva não faz o desenho saltar*. Um gate que recomputasse a normal para
//! conferir a normal seria o espelho sempre-verde que esta linha já pegou.

use super::*;
use ph2d_vec_scene::{Rgba8, StrokeSpec, VecPath, VecVertex};

/// Uma linha RETA horizontal, com traço, posada fora da origem — a fixture atravessa a fronteira
/// local↔mundo, e numa reta a normal é constante: o que se mede é a alça, não a geometria.
fn line_scene() -> (VecScene, SimWorld, VecEntityMap, VecPathId) {
    let mut sim = SimWorld::default();
    let mut map = VecEntityMap::new();
    let mut scene = VecScene::new();
    let mut p = VecPath {
        verts: [[0.0, 0.0], [4.0, 0.0]].map(VecVertex::corner).to_vec(),
        closed: false,
        ..VecPath::default()
    };
    p.stroke = Some(StrokeSpec::new(Rgba8::new(20, 20, 20, 255), 0.4));
    let id = scene.push_path(p);
    let e = sim
        .world_mut()
        .spawn((
            ph2d_ecs::Transform {
                translation: ph2d_core::Vec2::new(3.0, -2.0),
                ..ph2d_ecs::Transform::IDENTITY
            },
            ph2d_ecs::Name::new("Traco"),
            ph2d_ecs::VecPathRef(id),
        ))
        .id();
    map.insert(id, e.to_bits());
    (scene, sim, map, id)
}

/// Um perfil de teste, pela face de preset.
fn taper(start: f64, mid: f64, end: f64) -> WidthStops {
    ph2d_vec_scene::WidthProfile {
        start,
        mid,
        end,
        position: 0.5,
    }
    .to_stops()
}

/// **A alça pousa na BORDA da fita.** Ela é a manipulação direta de uma largura, então tem de
/// estar onde a tinta acaba — meia-largura vezes o multiplicador, sobre a normal. Uma alça sobre a
/// curva pediria ao artista que imaginasse a distância que está a editar.
#[test]
fn a_handle_sits_on_the_edge_of_the_ribbon() {
    let (scene, sim, map, id) = line_scene();
    let hs = handles(&sim, &scene, &map, id);
    assert_eq!(hs.len(), 2, "o neutro tem duas paradas: {hs:?}");
    for h in &hs {
        assert!(
            (h[1].abs() - (2.0 - 0.2)).abs() < 1e-9 || (h[1] - (-2.0 + 0.2)).abs() < 1e-9,
            "a alca nao pousou na borda da fita: {h:?}"
        );
    }
    assert!(
        (hs[0][0] - 3.0).abs() < 1e-9,
        "a 1a alca nao esta no comeco"
    );
    assert!((hs[1][0] - 7.0).abs() < 1e-9, "a ultima nao esta no fim");
}

/// **Uma forma sem traço não tem largura a editar** — e a resposta é nenhuma alça, não uma alça
/// colada à curva que não faria nada.
#[test]
fn a_shape_without_a_stroke_has_no_handles() {
    let (mut scene, sim, map, id) = line_scene();
    scene.path_mut(id).expect("existe").stroke = None;
    assert!(handles(&sim, &scene, &map, id).is_empty());
}

/// **Arrastar para FORA engrossa; para dentro afina.** É o gesto inteiro numa frase, e a direção
/// invertida faria a ferramenta desenhar o oposto do que a mão pede.
#[test]
fn dragging_outward_thickens_and_inward_thins() {
    let (scene, mut sim, map, id) = line_scene();
    let hs = handles(&sim, &scene, &map, id);
    let grab = press(&mut sim, &scene, &map, id, hs[0], 0.05).expect("agarrou a 1a alca");
    let y = hs[0][1];
    let side = (y + 2.0).signum();

    drag(&mut sim, &scene, &map, grab, [3.0, -2.0 + side * 0.4]);
    let thick = crate::profile_live::spec_of(&sim, &map, id).expect("armado");
    assert!(
        (thick.at(0.0) - 2.0).abs() < 1e-6,
        "afastar nao engrossou: {:.3}",
        thick.at(0.0)
    );

    drag(&mut sim, &scene, &map, grab, [3.0, -2.0 + side * 0.1]);
    let thin = crate::profile_live::spec_of(&sim, &map, id).expect("armado");
    assert!(
        (thin.at(0.0) - 0.5).abs() < 1e-6,
        "aproximar nao afinou: {:.3}",
        thin.at(0.0)
    );
}

/// **O lado não importa: a distância é ABSOLUTA.** A alça vive de um lado só; deixar o
/// multiplicador ficar negativo do outro viraria a fita do avesso, e uma largura negativa não é
/// uma largura.
#[test]
fn crossing_to_the_other_side_does_not_invert_the_ribbon() {
    let (scene, mut sim, map, id) = line_scene();
    let hs = handles(&sim, &scene, &map, id);
    let grab = press(&mut sim, &scene, &map, id, hs[0], 0.05).expect("agarrou");
    let side = (hs[0][1] + 2.0).signum();
    drag(&mut sim, &scene, &map, grab, [3.0, -2.0 - side * 0.4]);
    let st = crate::profile_live::spec_of(&sim, &map, id).expect("armado");
    assert!(
        st.at(0.0) > 0.0,
        "atravessar a curva deu multiplicador nao-positivo: {:.3}",
        st.at(0.0)
    );
}

/// **Arrastar ao longo da curva MOVE a parada.** O dedo aponta um lugar, e o lugar responde as
/// duas perguntas (que espessura, e onde) — separá-las pediria ao artista que soubesse qual metade
/// da alça ele está a mover.
#[test]
fn dragging_along_the_curve_moves_the_stop() {
    let (scene, mut sim, map, id) = line_scene();
    // ⚠️ A fixture parte de um perfil NÃO-uniforme de propósito: mover uma parada de um perfil
    // uniforme deixa-o uniforme, e um perfil uniforme não é guardado (o neutro-é-ausência). O
    // gate mediria a ausência e falharia sobre produto correto.
    crate::profile_live::arm(&mut sim, &map, &[id], &taper(0.4, 1.4, 0.4));
    let hs = handles(&sim, &scene, &map, id);
    let grab = press(&mut sim, &scene, &map, id, hs[0], 0.05).expect("agarrou");
    let side = (hs[0][1] + 2.0).signum();
    drag(&mut sim, &scene, &map, grab, [5.0, -2.0 + side * 0.2]);
    let st = crate::profile_live::spec_of(&sim, &map, id).expect("armado");
    let moved = st.as_slice()[grab.stop].pos;
    assert!(
        (moved - 0.5).abs() < 1e-6,
        "a parada nao andou para o meio: {moved:.3}"
    );
}

/// **Clicar na curva ACRESCENTA uma parada com o multiplicador que o perfil já tem ali** — a
/// espessura NO PONTO CLICADO não muda, e o arrasto seguinte é que a move. Uma parada que
/// nascesse em `1.0` faria a fita saltar sob o dedo antes de o artista pedir qualquer coisa.
///
/// ⚠️ **O que NÃO é preservado é a forma entre as paradas vizinhas, e isso é uma propriedade da
/// representação — não um defeito a consertar.** O `smoothstep` liga paradas CONSECUTIVAS, então
/// inserir uma re-parametriza os dois vãos que ela divide: medido, o desvio máximo num afinamento
/// de ponta é **0,058 de multiplicador** (~7% da faixa, ~0,012 unidade de mundo num traço de
/// 0,4 — sub-pixel no zoom de trabalho). Trocar por interpolação LINEAR tornaria a inserção
/// exata e poria um VINCO em cada parada, que é o que o `WidthProfile` recusou desde o 1º dia;
/// o trade está tomado, e é este gate que o pina para ninguém "consertá-lo" de volta.
#[test]
fn clicking_the_curve_adds_a_stop_at_the_thickness_it_already_had() {
    let (scene, mut sim, map, id) = line_scene();
    let t = taper(1.0, 1.0, 0.2);
    crate::profile_live::arm(&mut sim, &map, &[id], &t);

    let n0 = t.as_slice().len();
    let grab = press(&mut sim, &scene, &map, id, [6.0, -2.0], 0.05).expect("acrescentou");
    let after = crate::profile_live::spec_of(&sim, &map, id).expect("armado");
    assert_eq!(
        after.as_slice().len(),
        n0 + 1,
        "o clique na curva nao acrescentou parada"
    );
    // A parada nova está onde o dedo apontou, e com a espessura que havia ali.
    let st = after.as_slice()[grab.stop];
    assert!(
        (st.pos - 0.75).abs() < 1e-6,
        "a parada nasceu em {:.3}",
        st.pos
    );
    assert!(
        (st.mult - t.at(0.75)).abs() < 1e-9,
        "a espessura no ponto clicado MUDOU: {:.4} contra {:.4}",
        st.mult,
        t.at(0.75)
    );
    // E o resto do perfil segue o mesmo perfil — o desvio é o da re-parametrização, nomeado
    // acima, e não uma forma diferente.
    let worst = (0..=100)
        .map(|k| {
            let x = f64::from(k) / 100.0;
            (after.at(x) - t.at(x)).abs()
        })
        .fold(0.0_f64, f64::max);
    // ⚠️ **13,1% da faixa, e é ESTRUTURAL** — medido nos quatro perfis do sweep, sempre o mesmo
    // (é o máximo entre um smoothstep e dois meio-smoothsteps, não um acidente de dados). O
    // número está aqui para ninguém o descobrir de novo, e a razão de o artista nunca o ver é o
    // `Grab::created`: um clique que não arrastou é desfeito no release.
    let range = 1.0 - 0.2;
    assert!(
        worst / range < 0.14,
        "inserir uma parada mudou a forma em {worst:.4} ({:.1}% da faixa) — mais que a \
         re-parametrizacao estrutural de 13,1%",
        100.0 * worst / range
    );
}

/// **Um clique que NÃO arrastou não deixa nada** — é o que torna os 13,1% da re-parametrização
/// invisíveis. Com o Width Tool cria-se um ponto de largura ARRASTANDO a partir da curva; um
/// clique solto é um clique solto.
#[test]
fn a_click_that_never_dragged_leaves_the_profile_untouched() {
    let (scene, mut sim, map, id) = line_scene();
    let t = taper(1.0, 1.0, 0.2);
    crate::profile_live::arm(&mut sim, &map, &[id], &t);
    let before: Vec<f64> = (0..=20).map(|k| t.at(f64::from(k) / 20.0)).collect();

    let grab = press(&mut sim, &scene, &map, id, [6.0, -2.0], 0.05).expect("acrescentou");
    assert!(grab.created, "a parada nasceu neste gesto");
    discard_if_untouched(&mut sim, &map, grab);

    let after = crate::profile_live::spec_of(&sim, &map, id).expect("o perfil sobreviveu");
    assert_eq!(after.as_slice().len(), t.as_slice().len());
    for (k, b) in before.iter().enumerate() {
        let x = f64::from(u8::try_from(k).unwrap_or(0)) / 20.0;
        assert!(
            (after.at(x) - b).abs() < 1e-12,
            "t={x}: o clique solto mexeu no desenho ({:.5} contra {b:.5})",
            after.at(x)
        );
    }
}

/// **Uma alça AGARRADA não é desfeita.** Ela já existia antes do gesto; tratá-la como recém-criada
/// apagaria uma parada do artista a cada clique que não arrastasse.
#[test]
fn grabbing_an_existing_handle_is_never_discarded() {
    let (scene, mut sim, map, id) = line_scene();
    crate::profile_live::arm(&mut sim, &map, &[id], &taper(0.4, 1.4, 0.4));
    let hs = handles(&sim, &scene, &map, id);
    let grab = press(&mut sim, &scene, &map, id, hs[1], 0.05).expect("agarrou");
    assert!(!grab.created, "uma alca agarrada nao 'nasceu' no gesto");
    discard_if_untouched(&mut sim, &map, grab);
    assert_eq!(
        crate::profile_live::spec_of(&sim, &map, id)
            .expect("intacto")
            .as_slice()
            .len(),
        3
    );
}

/// **Um clique LONGE da curva não faz nada.** Sem isto todo clique no vazio acrescentaria uma
/// parada na projeção mais próxima, e o artista acumularia paradas que não pediu.
#[test]
fn clicking_far_from_the_curve_does_nothing() {
    let (scene, mut sim, map, id) = line_scene();
    assert!(press(&mut sim, &scene, &map, id, [5.0, 5.0], 0.05).is_none());
    assert!(crate::profile_live::spec_of(&sim, &map, id).is_none());
}

/// **O botão direito APAGA a parada sob a alça**, e abaixo de duas o perfil inteiro sai — o traço
/// volta ao uniforme em vez de ficar com uma parada solta a governar a largura por um caminho que
/// mais nenhuma rota usa.
#[test]
fn removing_below_two_stops_clears_the_profile() {
    let (scene, mut sim, map, id) = line_scene();
    crate::profile_live::arm(&mut sim, &map, &[id], &taper(0.3, 1.4, 0.3));

    let hs = handles(&sim, &scene, &map, id);
    assert_eq!(hs.len(), 3);
    assert!(
        remove(&mut sim, &scene, &map, id, hs[1], 0.05),
        "nao apagou"
    );
    assert_eq!(
        crate::profile_live::spec_of(&sim, &map, id)
            .expect("ainda ha perfil")
            .as_slice()
            .len(),
        2
    );
    let hs = handles(&sim, &scene, &map, id);
    assert!(remove(&mut sim, &scene, &map, id, hs[0], 0.05));
    assert!(
        crate::profile_live::spec_of(&sim, &map, id).is_none(),
        "sobrou um perfil de uma parada so"
    );
}

/// **O direito longe de uma alça não apaga nada** — o mesmo cuidado do clique no vazio.
#[test]
fn removing_far_from_a_handle_does_nothing() {
    let (scene, mut sim, map, id) = line_scene();
    crate::profile_live::arm(&mut sim, &map, &[id], &taper(0.3, 1.4, 0.3));
    assert!(!remove(&mut sim, &scene, &map, id, [5.0, 5.0], 0.05));
    assert_eq!(
        crate::profile_live::spec_of(&sim, &map, id)
            .expect("intacto")
            .as_slice()
            .len(),
        3
    );
}

/// **A alça segue a POSE.** A forma é desenhada onde a entidade a põe, e uma alça que ignorasse o
/// `Transform` pousaria longe da tinta assim que o artista movesse a forma.
#[test]
fn the_handles_follow_the_shapes_pose() {
    let (scene, mut sim, map, id) = line_scene();
    let before = handles(&sim, &scene, &map, id);
    let e = ph2d_ecs::Entity::from_bits(*map.get(&id).expect("mapeada"));
    if let Some(mut t) = sim.world_mut().get_mut::<ph2d_ecs::Transform>(e) {
        t.translation.x += 5.0;
    }
    let after = handles(&sim, &scene, &map, id);
    assert_eq!(before.len(), after.len());
    for (a, b) in before.iter().zip(&after) {
        assert!(
            (b[0] - a[0] - 5.0).abs() < 1e-9,
            "a alca andou {:.3} em vez dos 5,0 da pose",
            b[0] - a[0]
        );
    }
}

/// **A alça NÃO escala com a pose**, porque a fita também não: o `bake_xform` transforma pontos e
/// comprimentos de path e deixa `stroke.width` como está, então o `power_stroke` molda a fita na
/// largura autorada mesmo sob uma pose escalada. Uma alça que multiplicasse pela escala pousaria
/// fora da tinta — as duas TÊM de concordar.
#[test]
fn the_handle_offset_does_not_scale_with_the_pose() {
    let (scene, mut sim, map, id) = line_scene();
    let plain = handles(&sim, &scene, &map, id)[0][1];
    let e = ph2d_ecs::Entity::from_bits(*map.get(&id).expect("mapeada"));
    if let Some(mut t) = sim.world_mut().get_mut::<ph2d_ecs::Transform>(e) {
        t.scale = ph2d_core::Vec2::new(3.0, 3.0);
    }
    let scaled = handles(&sim, &scene, &map, id)[0][1];
    assert!(
        (scaled - plain).abs() < 1e-9,
        "o desvio da alca escalou com a pose: {plain:.4} -> {scaled:.4}"
    );
}

//! Gates da **PELE por-widget** — arquivo irmão de `widget_live.rs` (plano UI/UX W6.2).
//!
//! ⚠️ Estes gates dirigem a PONTE, não o pintor. Os gates do pintor (a pele emite exactamente o
//! que o painel nativo emite) vivem em `ph2d-editor-core::widget::skin_tests`; aqui o que se
//! prova é o que **só a shell** pode errar: quem é marcado, onde a moldura pousa, e o que
//! acontece com um código que este build não conhece.

use super::*;

use ph2d_editor::widget::WidgetKind;

/// Uma forma quadrada com pose, opcionalmente vestida e opcionalmente nomeada.
fn scene_with(
    kind: Option<u16>,
    name: Option<&str>,
) -> (
    VecScene,
    ph2d_ecs::SimWorld,
    VecEntityMap,
    VecXforms,
    VecPathId,
) {
    let mut sim = ph2d_ecs::SimWorld::default();
    let mut map = VecEntityMap::new();
    let mut scene = VecScene::new();
    let id = scene.push_path(ph2d_vec_scene::rectangle([-1.0, -0.4], [1.0, 0.4]));
    let mut e = sim
        .world_mut()
        .spawn((ph2d_ecs::Transform::IDENTITY, ph2d_ecs::VecPathRef(id)));
    if let Some(k) = kind {
        e.insert(VecWidget { kind: k });
    }
    if let Some(n) = name {
        e.insert(ph2d_ecs::Name::new(n));
    }
    let bits = e.id().to_bits();
    map.insert(id, bits);
    let xf = crate::vec_transform::build(&sim, &map);
    (scene, sim, map, xf, id)
}

/// A câmera do editor a um zoom qualquer — o que importa é ela ser a MESMA nos dois lados.
fn camera() -> Affine {
    ph2d_render::Camera2d::new([0.0, 0.0], 10.0).world_to_screen_affine(ph2d_host::WindowSize {
        width: 1200,
        height: 800,
    })
}

fn build_for(
    scene: &VecScene,
    sim: &ph2d_ecs::SimWorld,
    map: &VecEntityMap,
    xf: &VecXforms,
) -> WidgetSkins {
    let mut text = TextSystem::without_system_fonts();
    build(
        scene,
        sim,
        map,
        xf,
        &LiveGeometry::new(),
        camera(),
        &mut text,
        Theme::Forge,
    )
}

/// **Uma forma comum não ganha pele** — o caminho que TODA arte do canvas percorre.
///
/// ⚠️ É o controle da wave: se ele falhasse, marcar uma forma seria irrelevante porque todas já
/// estariam vestidas, e os outros gates ficariam verdes sobre um produto que substituiu o
/// desenho de todo mundo.
#[test]
fn an_unmarked_shape_gets_no_skin() {
    let (scene, sim, map, xf, _) = scene_with(None, Some("Rect"));
    assert!(build_for(&scene, &sim, &map, &xf).is_empty());
}

/// **Uma forma marcada é pintada pelo catálogo, no lugar dela.**
#[test]
fn a_marked_shape_is_painted_by_the_catalogue() {
    let (scene, sim, map, xf, id) = scene_with(Some(WidgetKind::Button.code()), Some("Save"));
    let skins = build_for(&scene, &sim, &map, &xf);
    let skin = skins.get(&id).expect("a forma marcada tem pele");
    let e = skin.inner().encoding();
    assert!(
        e.n_paths > 0 || !e.resources.glyph_runs.is_empty(),
        "a pele nao desenhou nada"
    );
}

/// **Um código que este build não conhece DEGRADA para o desenho** — o gate que o plano pede.
///
/// ⚠️ A ausência é a resposta certa, e ela é diferente de uma pele vazia: presente-e-vazia
/// pintaria NADA no lugar da forma (o buraco), ausente devolve o desenho vetorial. Um documento
/// autorado por um build mais novo tem de abrir mostrando a arte, não um vão.
#[test]
fn an_unknown_kind_degrades_to_the_drawing() {
    let (scene, sim, map, xf, id) = scene_with(Some(9999), Some("From the future"));
    let skins = build_for(&scene, &sim, &map, &xf);
    assert!(
        !skins.contains_key(&id),
        "um tipo desconhecido produziu pele — a forma desapareceria"
    );
}

/// **A moldura É a da forma, termo a termo** — e não meramente *alguma coisa que muda com a pose*.
///
/// ⚠️ Este gate nasceu porque o irmão abaixo é FRACO: ele compara a pele de duas poses e exige
/// que difiram, e uma ponte que fixasse a ORIGEM mas mantivesse o tamanho dependente da pose
/// passava por ele (medido). *Diferir* e *pousar no lugar certo* não são a mesma pergunta, e um
/// widget desenhado ao lado da forma que o artista marcou é exactamente o tipo de defeito que só
/// uma screenshot mostra.
#[test]
fn the_frame_is_the_shape_screen_box() {
    let (scene, sim, map, xf, id) = scene_with(Some(WidgetKind::Card.code()), Some("Card"));
    let _ = (&sim, &map);
    let cam = camera();
    let bounds = ph2d_vec_render::path_screen_bounds(&scene, &xf, &LiveGeometry::new(), id, cam)
        .expect("a forma tem bbox de tela");
    let r = frame_of(&scene, &xf, &LiveGeometry::new(), id, cam).expect("a moldura existe");
    assert!(
        (r.x - bounds.0 as f32).abs() < 1e-3,
        "x: {r:?} vs {bounds:?}"
    );
    assert!(
        (r.y - bounds.1 as f32).abs() < 1e-3,
        "y: {r:?} vs {bounds:?}"
    );
    assert!(
        (r.w - (bounds.2 - bounds.0) as f32).abs() < 1e-3,
        "w: {r:?} vs {bounds:?}"
    );
    assert!(
        (r.h - (bounds.3 - bounds.1) as f32).abs() < 1e-3,
        "h: {r:?} vs {bounds:?}"
    );
}

/// **A moldura ANDA com a pose** — o irmão de comportamento, que atravessa o `build` inteiro.
///
/// ⚠️ Ele é mais fraco que o de cima (ver o doc lá) e fica pela cobertura que o de cima não tem:
/// este percorre a ponte REAL — a leitura do componente, a pintura, o mapa —, e o de cima só
/// pergunta à moldura.
#[test]
fn the_frame_follows_the_pose() {
    let (scene, mut sim, map, _, id) = scene_with(Some(WidgetKind::Card.code()), Some("Card"));
    let here = {
        let xf = crate::vec_transform::build(&sim, &map);
        build_for(&scene, &sim, &map, &xf)
    };
    // Move a entidade e re-coza: a pele tem de sair diferente.
    let bits = *map.get(&id).unwrap();
    sim.world_mut()
        .get_mut::<ph2d_ecs::Transform>(ph2d_ecs::Entity::from_bits(bits))
        .unwrap()
        .translation = ph2d_core::Vec2::new(3.0, 1.5);
    let there = {
        let xf = crate::vec_transform::build(&sim, &map);
        build_for(&scene, &sim, &map, &xf)
    };

    let a = here.get(&id).unwrap().inner().encoding().path_data.clone();
    let b = there.get(&id).unwrap().inner().encoding().path_data.clone();
    assert!(!a.is_empty(), "o Card nao emitiu geometria");
    assert_ne!(
        a, b,
        "a pele pintou no MESMO lugar depois de a forma se mover — a moldura nao e' a da forma"
    );
}

/// **O rótulo é o `Name`** — e uma entidade sem nome ainda desenha a moldura.
///
/// ⚠️ A metade sem nome é a que importa: se o rótulo fosse obrigatório, uma forma anônima
/// marcada sumiria da tela em vez de mostrar um controle em branco.
#[test]
fn the_label_is_the_entity_name_and_a_nameless_shape_still_draws() {
    let (s1, w1, m1, x1, id) = scene_with(Some(WidgetKind::Button.code()), Some("Save"));
    let (s2, w2, m2, x2, _) = scene_with(Some(WidgetKind::Button.code()), Some("Cancel"));
    let (s3, w3, m3, x3, _) = scene_with(Some(WidgetKind::Button.code()), None);

    let glyphs = |sk: &WidgetSkins| -> Vec<(u32, u32, u32)> {
        sk.get(&id)
            .unwrap()
            .inner()
            .encoding()
            .resources
            .glyphs
            .iter()
            .map(|g| (g.id, g.x.to_bits(), g.y.to_bits()))
            .collect()
    };
    let named = glyphs(&build_for(&s1, &w1, &m1, &x1));
    let other = glyphs(&build_for(&s2, &w2, &m2, &x2));
    let anon = build_for(&s3, &w3, &m3, &x3);

    assert!(!named.is_empty(), "o Name nao virou texto");
    assert_ne!(named, other, "dois Names diferentes pintaram o mesmo texto");
    assert!(
        anon.contains_key(&id),
        "uma forma SEM nome perdeu a pele — ela deveria desenhar a moldura vazia"
    );
}

/// **A swatch do canvas veste o preenchimento da PRÓPRIA forma** — e ela é VIVA.
///
/// ⚠️ Esta é a metade do canal que só a shell pode errar, e o modo de falha dela é mudo: a ponte
/// podia passar `SkinParam::default()` e a swatch pintaria o xadrez para sempre, com o pintor
/// correto, o painel gerado correto, e toda a suíte verde. O que se vê é uma screenshot.
///
/// ⚠️ E a metade oposta é o que mantém a fronteira escrita: um `Slider` sobre a MESMA forma tem de
/// pintar igual com as duas tintas — se ele respondesse, recolorir uma forma mudaria a aparência
/// de um controle que não fala de cor nenhuma.
#[test]
fn the_swatch_on_the_canvas_wears_its_own_fill() {
    use ph2d_vec_scene::{Paint, Rgba8};

    let skin_of = |kind: WidgetKind, rgb: [u8; 3]| {
        let (mut scene, sim, map, xf, id) = scene_with(Some(kind.code()), Some("Tint"));
        scene
            .path_mut(id)
            .expect("a forma existe")
            .fill
            .replace(Paint::Solid(Rgba8::new(rgb[0], rgb[1], rgb[2], 255)));
        let skins = build_for(&scene, &sim, &map, &xf);
        let e = skins
            .get(&id)
            .expect("a forma marcada tem pele")
            .inner()
            .encoding();
        (e.n_paths, e.path_data.clone(), e.draw_data.clone())
    };

    let red = skin_of(WidgetKind::ColorSwatch, [200, 40, 40]);
    let blue = skin_of(WidgetKind::ColorSwatch, [40, 40, 200]);
    assert_ne!(
        red, blue,
        "a swatch do canvas pintou igual com duas tintas — a ponte nao le o preenchimento"
    );

    let a = skin_of(WidgetKind::Slider, [200, 40, 40]);
    let b = skin_of(WidgetKind::Slider, [40, 40, 200]);
    assert_eq!(
        a, b,
        "o Slider respondeu ao preenchimento da forma — recolorir a arte mudaria um controle \
         que nao fala de cor"
    );
}

/// **O botão de ícone do canvas veste o DESENHO da própria forma** — o irmão exato do gate acima.
///
/// ⚠️ **As duas formas têm a MESMA caixa, e a fixture a obtém por CONSTRUÇÃO, não por sorte.** A
/// moldura do widget sai dos limites de tela da forma, então duas formas de caixas diferentes
/// dariam peles diferentes por um motivo que não é o glifo — e o gate passaria sobre uma ponte que
/// entrega `SkinParam::default()`. Duas estrelas de mesmos vértices EXTERNOS e miolos diferentes
/// têm a mesma caixa (os vértices internos são estritamente mais próximos do centro) e desenhos
/// distintos, e é a única coisa que pode diferir.
///
/// ⚠️ **A primeira fixture era um retângulo contra uma estrela "inscrita nele", e estava ERRADA:**
/// os vértices externos de uma estrela de cinco pontas não tocam os cantos da caixa, então as
/// caixas divergiam e a metade do `Slider` sangrou. O gate apanhou a própria premissa — e por isso
/// ela é agora **afirmada**, não assumida.
///
/// ⚠️ E a metade oposta mantém a fronteira escrita: um `Slider` sobre as MESMAS duas formas tem de
/// pintar igual — se ele respondesse, editar a geometria mudaria a aparência de um controle que
/// não fala de desenho nenhum.
#[test]
fn the_icon_button_on_the_canvas_wears_its_own_drawing() {
    let shape = |waist: f64| ph2d_vec_scene::star([0.0, 0.0], 1.0, 0.4, 5, waist);
    let scene_of = |kind: WidgetKind, waist: f64| {
        let (mut scene, sim, map, xf, id) = scene_with(Some(kind.code()), Some("Play"));
        *scene.path_mut(id).expect("a forma existe") = shape(waist);
        (scene, sim, map, xf, id)
    };
    let skin_of = |kind: WidgetKind, waist: f64| {
        let (scene, sim, map, xf, id) = scene_of(kind, waist);
        let skins = build_for(&scene, &sim, &map, &xf);
        let e = skins
            .get(&id)
            .expect("a forma marcada tem pele")
            .inner()
            .encoding();
        (e.n_paths, e.path_data.clone(), e.draw_data.clone())
    };

    // A premissa da fixture, AFIRMADA: as duas formas ocupam a mesma moldura.
    let frame_of_waist = |waist: f64| {
        let (scene, _, _, xf, id) = scene_of(WidgetKind::IconButton, waist);
        crate::widget_live::frame_of(&scene, &xf, &LiveGeometry::new(), id, camera())
            .map(|r| (r.x.to_bits(), r.y.to_bits(), r.w.to_bits(), r.h.to_bits()))
    };
    assert_eq!(
        frame_of_waist(0.45),
        frame_of_waist(0.15),
        "as duas formas nao partilham a moldura — o gate mediria outra coisa"
    );

    assert_ne!(
        skin_of(WidgetKind::IconButton, 0.45),
        skin_of(WidgetKind::IconButton, 0.15),
        "o botao de icone pintou igual com dois desenhos — a ponte nao le a geometria"
    );
    assert_eq!(
        skin_of(WidgetKind::Slider, 0.45),
        skin_of(WidgetKind::Slider, 0.15),
        "o Slider respondeu ao desenho da forma — editar a arte mudaria um controle que nao \
         fala de geometria"
    );
}

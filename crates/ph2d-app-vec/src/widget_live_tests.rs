//! Gates da **PELE por-widget** — arquivo irmão de `widget_live.rs` (plano UI/UX W6.2).
//!
//! ⚠️ Estes gates dirigem a PONTE, não o pintor. Os gates do pintor (a pele emite exactamente o
//! que o painel nativo emite) vivem em `ph2d-editor-core::widget::skin_tests`; aqui o que se
//! prova é o que **só a shell** pode errar: quem é marcado, onde a moldura pousa, e o que
//! acontece com um código que este build não conhece.

use super::*;

use ph2d_editor_core::widget::WidgetKind;

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
    let xf = ph2d_vec_entities::transform::build(&sim, &map);
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
        &ph2d_vec_scene::VecViewState::default(),
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
    let e = skin.scene.inner().encoding();
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
        let xf = ph2d_vec_entities::transform::build(&sim, &map);
        build_for(&scene, &sim, &map, &xf)
    };
    // Move a entidade e re-coza: a pele tem de sair diferente.
    let bits = *map.get(&id).unwrap();
    sim.world_mut()
        .get_mut::<ph2d_ecs::Transform>(ph2d_ecs::Entity::from_bits(bits))
        .unwrap()
        .translation = ph2d_core::Vec2::new(3.0, 1.5);
    let there = {
        let xf = ph2d_vec_entities::transform::build(&sim, &map);
        build_for(&scene, &sim, &map, &xf)
    };

    let a = here.get(&id).unwrap().scene.inner().encoding().path_data.clone();
    let b = there.get(&id).unwrap().scene.inner().encoding().path_data.clone();
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
            .scene.inner()
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
            .scene.inner()
            .encoding();
        (e.n_paths, e.path_data.clone(), e.draw_data.clone())
    };

    let red = skin_of(WidgetKind::ColorSwatch, [200, 40, 40]);
    let blue = skin_of(WidgetKind::ColorSwatch, [40, 40, 200]);
    assert_ne!(
        red, blue,
        "a swatch do canvas pintou igual com duas tintas — a ponte nao le o preenchimento"
    );

    // ⭐ F18: o Slider NÃO pinta a cor — o desenho a pinta por baixo. Recolorir a forma não lhe
    // muda a GEOMETRIA nem põe a tinta do artista dentro da pele; só a tinta do conteúdo se
    // reescolhe contra o corpo novo (`skin_body`).
    let a = skin_of(WidgetKind::Slider, [200, 40, 40]);
    let b = skin_of(WidgetKind::Slider, [40, 40, 200]);
    assert_eq!((a.0, &a.1), (b.0, &b.1), "recolorir mudou a geometria do Slider");
    for (skin, rgb) in [(&a, [200, 40, 40]), (&b, [40, 40, 200])] {
        let fill = u32::from_ne_bytes([rgb[0], rgb[1], rgb[2], 255]);
        assert!(
            !skin.2.contains(&fill),
            "o Slider pintou o preenchimento do artista DENTRO da pele — o corpo é o desenho"
        );
    }
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
            .scene.inner()
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

/// **A ESCOLHA vence o desenho, no canvas** — e tirá-la devolve o desenho.
///
/// ⚠️ A segunda metade é a que prova o modelo: não há um terceiro estado a dizer qual rota está
/// activa, então *voltar ao desenho* tem de ser exactamente *não ter o componente*. Sem ela, um
/// `chosen_icon` que devolvesse `Some` para todo mundo ficaria verde na primeira asserção e o
/// artista nunca mais veria a forma que desenhou.
#[test]
fn the_chosen_icon_beats_the_drawing_and_removing_it_gives_the_drawing_back() {
    let skin_of = |chosen: Option<&str>| {
        let (mut scene, mut sim, map, xf, id) =
            scene_with(Some(WidgetKind::IconButton.code()), Some("Play"));
        *scene.path_mut(id).expect("a forma existe") =
            ph2d_vec_scene::star([0.0, 0.0], 1.0, 0.4, 5, 0.45);
        if let Some(slug) = chosen {
            let e = ph2d_ecs::Entity::from_bits(*map.get(&id).expect("a forma tem entidade"));
            sim.world_mut()
                .entity_mut(e)
                .insert(ph2d_ecs::VecWidgetIcon { slug: slug.into() });
        }
        let skins = build_for(&scene, &sim, &map, &xf);
        let e = skins
            .get(&id)
            .expect("a forma marcada tem pele")
            .scene.inner()
            .encoding();
        (e.n_paths, e.path_data.clone(), e.draw_data.clone())
    };

    let drawn = skin_of(None);
    let trash = skin_of(Some("trash"));
    assert_ne!(
        drawn, trash,
        "a escolha nao venceu o desenho — a ponte nao le o componente"
    );
    assert_ne!(
        trash,
        skin_of(Some("bolt")),
        "dois glifos escolhidos pintaram igual — so' a PRESENCA do componente chega ao pintor"
    );
    // ⚠️ Um slug que este build não conhece **degrada para o desenho**, byte a byte — o canal de
    // compatibilidade, e a razão de o `from_slug` devolver `Option`.
    assert_eq!(
        skin_of(Some("nao-existe")),
        drawn,
        "um slug desconhecido nao caiu no desenho"
    );
}

// ── F18: o DESENHO é o corpo (`vetor_nova_ui/29_plano_F18_…md` §6, gates 1, 7 e 8) ──────────

/// Uma forma vestida de `kind`, chamada «Play», pintada de `fill`.
fn dressed_in(kind: WidgetKind, fill: [u8; 4]) -> (VecScene, ph2d_ecs::SimWorld, VecEntityMap, VecXforms) {
    let (mut scene, sim, map, xf, id) = scene_with(Some(kind.code()), Some("Play"));
    let [r, g, b, a] = fill;
    scene.path_mut(id).expect("a forma existe").fill =
        Some(ph2d_vec_scene::Paint::solid(ph2d_vec_scene::Rgba8::new(r, g, b, a)));
    (scene, sim, map, xf)
}

/// O canvas inteiro desta cena: a ponte REAL e o `dispatch` REAL.
fn canvas_of(scene: &VecScene, sim: &ph2d_ecs::SimWorld, map: &VecEntityMap, xf: &VecXforms) -> ph2d_vector::VectorScene {
    let skins = build_for(scene, sim, map, xf);
    let mut out = ph2d_vector::VectorScene::new();
    ph2d_vec_render::dispatch(
        scene,
        &ph2d_vec_scene::VecViewState::default(),
        xf,
        &LiveGeometry::new(),
        &ph2d_vec_render::FxImages::new(),
        &skins,
        &ph2d_vec_render::PatternTiles::new(),
        &ph2d_vec_render::BrushArts::new(),
        &ph2d_vec_render::DilatedPaints::new(),
        camera(),
        &mut out,
    );
    out
}

/// O índice, no fluxo de tinta, da cor opaca `c` — ou `None`.
fn ink_index(scene: &ph2d_vector::VectorScene, c: [u8; 4]) -> Option<usize> {
    scene.inner().encoding().draw_data.iter().position(|w| w.to_ne_bytes() == c)
}

/// As corridas de glifos: `(índice no fluxo de tinta, tinta)`.
fn glyph_inks(scene: &ph2d_vector::VectorScene) -> Vec<(usize, [u8; 4])> {
    let e = scene.inner().encoding();
    e.resources
        .glyph_runs
        .iter()
        .map(|r| (r.stream_offsets.draw_data, e.draw_data[r.stream_offsets.draw_data].to_ne_bytes()))
        .collect()
}

fn readable(c: [u8; 4]) -> [u8; 4] {
    let [r, g, b, a] = c;
    let k = ph2d_tokens::skin_body::readable_on(ph2d_tokens::Color { r, g, b, a }, Theme::Forge);
    [k.r, k.g, k.b, k.a]
}

const BLUE: [u8; 4] = [40, 120, 240, 255];
const ORANGE: [u8; 4] = [240, 140, 40, 255];

/// **O botão do dono aparece AZUL, com o rótulo legível POR CIMA** (cena 25).
#[test]
fn o_corpo_da_pele_vestida_e_o_desenho() {
    let (scene, sim, map, xf) = dressed_in(WidgetKind::Button, BLUE);
    let out = canvas_of(&scene, &sim, &map, &xf);
    let body = ink_index(&out, BLUE).expect("o fill azul do artista não foi desenhado");
    let label = glyph_inks(&out);
    assert!(!label.is_empty(), "o rótulo sumiu");
    for (at, ink) in &label {
        assert!(*at > body, "o rótulo tem de vir DEPOIS do corpo");
        assert_eq!(*ink, readable(BLUE), "o rótulo não é a tinta legível sobre o azul");
    }
}

/// **O Hover gravado chega ao rótulo**: a pose escreve o fill (laranja) e a tinta reescolhe-se.
#[test]
fn o_fill_do_estado_chega_ao_corpo_e_ao_rotulo() {
    let (scene, sim, map, xf) = dressed_in(WidgetKind::Button, ORANGE);
    let out = canvas_of(&scene, &sim, &map, &xf);
    assert!(ink_index(&out, ORANGE).is_some(), "o laranja do Hover não foi desenhado");
    assert!(ink_index(&out, BLUE).is_none());
    assert!(glyph_inks(&out).iter().all(|(_, c)| *c == readable(ORANGE)));
    // Sobre o azul e o laranja do dono a tinta legível é a MESMA (escura): a viragem mede-se
    // contra um fill escuro.
    const DARK: [u8; 4] = [32, 36, 44, 255];
    assert_ne!(readable(DARK), readable(ORANGE), "a fixture tem de mudar a tinta");
    let (scene, sim, map, xf) = dressed_in(WidgetKind::Button, DARK);
    let out = canvas_of(&scene, &sim, &map, &xf);
    assert!(glyph_inks(&out).iter().all(|(_, c)| *c == readable(DARK)));
}

/// **O ícone e a amostra continuam a SUBSTITUIR** — o desenho deles já foi consumido.
#[test]
fn o_icone_e_a_amostra_nao_desenham_duas_vezes() {
    for kind in WidgetKind::ALL {
        let (scene, sim, map, xf) = dressed_in(kind, BLUE);
        let skins = build_for(&scene, &sim, &map, &xf);
        let skin = skins.values().next().expect("vestida tem pele");
        assert_eq!(skin.over_drawing, kind.wears_the_drawing(), "{kind:?}");
    }
    let (scene, sim, map, xf) = dressed_in(WidgetKind::IconButton, BLUE);
    assert!(
        ink_index(&canvas_of(&scene, &sim, &map, &xf), BLUE).is_none(),
        "o desenho de um botão de ícone é o GLIFO — desenhá-lo também seria o ícone duas vezes"
    );
}

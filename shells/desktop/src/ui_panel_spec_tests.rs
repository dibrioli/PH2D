//! Gates do **plano do painel** (plano UI/UX W8b).

use super::*;
use ph2d_ecs::{ChildOf, Transform, VecPathRef};
use ph2d_vec_scene::VecScene;

/// Uma moldura com quatro filhos: três vestidos e **um que é só desenho**.
fn frame_with_children() -> (SimWorld, Entity) {
    let mut sim = SimWorld::new();
    let frame = sim
        .world_mut()
        .spawn((Name("Painel de Cor".into()), Transform::IDENTITY))
        .id();
    let add = |sim: &mut SimWorld, name: &str, kind: Option<u16>| {
        let e = sim
            .world_mut()
            .spawn((Name(name.into()), Transform::IDENTITY, VecPathRef(1)))
            .id();
        sim.world_mut().entity_mut(e).insert(ChildOf(frame));
        if let Some(k) = kind {
            sim.world_mut().entity_mut(e).insert(VecWidget { kind: k });
        }
        e
    };
    add(&mut sim, "Título", Some(WidgetKind::SectionHeader.code()));
    add(&mut sim, "Fundo", None);
    add(&mut sim, "Opacity", Some(WidgetKind::Slider.code()));
    add(&mut sim, "Visível", Some(WidgetKind::Toggle.code()));
    (sim, frame)
}

/// **Só quem VESTE vira row** — e a ordem é a dos filhos.
#[test]
fn the_spec_is_the_dressed_children_in_tree_order() {
    let (sim, frame) = frame_with_children();
    let spec = of(&sim, &VecScene::new(), frame);

    assert_eq!(spec.title, "Painel de Cor");
    assert_eq!(
        spec.id, "painel_de_cor",
        "o slug nao saiu do nome da moldura"
    );
    let got: Vec<(&str, &str)> = spec
        .rows
        .iter()
        .map(|r| (r.kind.as_str(), r.label.as_str()))
        .collect();
    assert_eq!(
        got,
        vec![
            ("SectionHeader", "Título"),
            ("Slider", "Opacity"),
            ("Toggle", "Visível"),
        ],
        "o desenho puro virou row, ou a ordem nao e a dos filhos"
    );
}

/// **A moldura não é a primeira row de si mesma.**
///
/// ⚠️ Vestir a moldura é legítimo (ela É um retângulo desenhado, e a pele dela é o fundo do
/// painel). O que não pode acontecer é ela aparecer também como CONTROLE — seria a árvore lida um
/// nível acima do que ela é, e o painel gerado abriria com uma linha que é ele próprio.
#[test]
fn a_dressed_frame_is_the_panel_not_its_first_row() {
    let (mut sim, frame) = frame_with_children();
    sim.world_mut().entity_mut(frame).insert(VecWidget {
        kind: WidgetKind::Card.code(),
    });
    let spec = of(&sim, &VecScene::new(), frame);
    assert_eq!(spec.rows.len(), 3, "a moldura virou row de si mesma");
}

/// **Um `kind` que este build não conhece não vira row.**
///
/// ⚠️ É o canal que o `from_code` existe para suportar: um documento autorado por um build mais
/// novo. Inventar um tipo aqui seria gerar código para um widget que não existe — e o erro
/// apareceria no `cargo`, longe do desenho que o causou.
#[test]
fn an_unknown_kind_does_not_become_a_row() {
    let (mut sim, frame) = frame_with_children();
    let e = sim
        .world_mut()
        .spawn((Name("Do futuro".into()), Transform::IDENTITY, VecPathRef(9)))
        .id();
    sim.world_mut()
        .entity_mut(e)
        .insert((ChildOf(frame), VecWidget { kind: 60_000 }));
    assert_eq!(
        of(&sim, &VecScene::new(), frame).rows.len(),
        3,
        "um tipo desconhecido virou row"
    );
}

/// **A chave é estável e legível** — é ela que vira `NodeId` por hash.
#[test]
fn the_key_is_a_slug_of_the_label() {
    assert_eq!(key_of("Opacity"), "opacity");
    assert_eq!(key_of("Fill / Stroke"), "fill___stroke");
    assert_eq!(
        key_of(""),
        "_",
        "um rotulo vazio ainda tem de dar uma chave"
    );
}

/// **Uma moldura com uma swatch e um slider, cada um sobre a sua forma PINTADA.**
///
/// ⚠️ Os dois têm preenchimento de propósito: é o que torna o gate da fronteira capaz de falhar.
/// Com só a swatch pintada, *"a cor sai do fill"* e *"a cor sai do fill de quem a consome"* seriam
/// indistinguíveis.
fn frame_with_a_swatch() -> (SimWorld, VecScene, Entity) {
    use ph2d_vec_scene::{Paint, Rgba8, VecPath, rectangle};
    let mut sim = SimWorld::new();
    let mut scene = VecScene::new();
    let frame = sim
        .world_mut()
        .spawn((Name("Color".into()), Transform::IDENTITY))
        .id();
    let mut add = |sim: &mut SimWorld, name: &str, kind: u16, fill: Option<[u8; 3]>| {
        let mut p: VecPath = rectangle([-1.0, -0.4], [1.0, 0.4]);
        p.fill = fill.map(|c| Paint::Solid(Rgba8::new(c[0], c[1], c[2], 255)));
        let id = scene.push_path(p);
        let e = sim
            .world_mut()
            .spawn((
                Name(name.into()),
                Transform::IDENTITY,
                VecPathRef(id),
                VecWidget { kind },
            ))
            .id();
        sim.world_mut().entity_mut(e).insert(ChildOf(frame));
    };
    add(
        &mut sim,
        "Tint",
        WidgetKind::ColorSwatch.code(),
        Some([214, 92, 64]),
    );
    add(
        &mut sim,
        "Opacity",
        WidgetKind::Slider.code(),
        Some([10, 200, 30]),
    );
    (sim, scene, frame)
}

/// **A swatch carrega o preenchimento da forma que a veste** — e só ela.
///
/// ⚠️ As duas metades são perguntas diferentes e o gate falha por qualquer uma: sem a primeira, a
/// swatch pinta o xadrez para sempre com a suíte verde; sem a segunda, o dia em que alguém tirasse
/// o guarda do `takes_colour` faria todo widget carregar uma cor que ninguém pediu — e o `Slider`
/// só mudaria de tinta no dia em que um braço passasse a lê-la.
#[test]
fn only_the_swatch_carries_the_shapes_fill() {
    let (sim, scene, frame) = frame_with_a_swatch();
    let spec = of(&sim, &scene, frame);
    let got: Vec<(&str, Option<[u8; 4]>)> = spec
        .rows
        .iter()
        .map(|r| (r.kind.as_str(), r.rgba))
        .collect();
    assert_eq!(
        got,
        vec![("ColorSwatch", Some([214, 92, 64, 255])), ("Slider", None),],
        "a cor nao saiu do preenchimento, ou vazou para quem nao a consome"
    );
}

/// **Uma forma SEM preenchimento deixa a swatch vazia** — e vazia É a resposta.
///
/// ⚠️ `None` não é um erro nem um fallback de conveniência: o pintor desenha o tabuleiro de
/// transparência, e *nenhuma cor escolhida* é exactamente o que ele significa. Inventar aqui um
/// preto ou um cinza seria pintar uma escolha que o artista não fez.
#[test]
fn a_shape_without_fill_leaves_the_swatch_empty() {
    use ph2d_vec_scene::rectangle;
    let mut sim = SimWorld::new();
    let mut scene = VecScene::new();
    let frame = sim
        .world_mut()
        .spawn((Name("Color".into()), Transform::IDENTITY))
        .id();
    let id = scene.push_path(rectangle([-1.0, -0.4], [1.0, 0.4]));
    let e = sim
        .world_mut()
        .spawn((
            Name("Tint".into()),
            Transform::IDENTITY,
            VecPathRef(id),
            VecWidget {
                kind: WidgetKind::ColorSwatch.code(),
            },
        ))
        .id();
    sim.world_mut().entity_mut(e).insert(ChildOf(frame));
    let spec = of(&sim, &scene, frame);
    assert_eq!(spec.rows.len(), 1);
    assert_eq!(spec.rows[0].rgba, None, "uma forma sem tinta inventou uma");
}

/// **O botão de ícone carrega o DESENHO da forma que o veste** — e só ele.
///
/// ⚠️ Irmão exato de [`only_the_swatch_carries_the_shapes_fill`], e a fixture segue a mesma lei:
/// as DUAS formas têm geometria, senão *"o glifo sai do desenho"* e *"ninguém carrega glifo"*
/// seriam indistinguíveis.
///
/// ⚠️ E o glifo é conferido contra a **porta única**, não contra uma string escrita à mão: um
/// literal aqui seria uma segunda normalização, e ela divergiria da do canvas exactamente no dia
/// em que alguém mexesse na lei — que é a divergência que a porta existe para impedir.
#[test]
fn only_the_icon_button_carries_the_shapes_drawing() {
    use ph2d_vec_scene::{VecPath, rectangle, star};

    let mut sim = SimWorld::new();
    let mut scene = VecScene::new();
    let frame = sim
        .world_mut()
        .spawn((Name("Bar".into()), Transform::IDENTITY))
        .id();
    let mut add = |sim: &mut SimWorld, name: &str, kind: u16, p: VecPath| {
        let id = scene.push_path(p);
        let e = sim
            .world_mut()
            .spawn((
                Name(name.into()),
                Transform::IDENTITY,
                VecPathRef(id),
                VecWidget { kind },
            ))
            .id();
        sim.world_mut().entity_mut(e).insert(ChildOf(frame));
        id
    };
    let star_id = add(
        &mut sim,
        "Play",
        WidgetKind::IconButton.code(),
        star([0.0, 0.0], 1.0, 1.0, 5, 0.45),
    );
    add(
        &mut sim,
        "Opacity",
        WidgetKind::Slider.code(),
        rectangle([-1.0, -0.4], [1.0, 0.4]),
    );

    let want = crate::widget_icon::icon_face(scene.path(star_id).expect("a forma existe"))
        .expect("a estrela tem figura")
        .to_svg();
    let spec = of(&sim, &scene, frame);
    let got: Vec<(&str, Option<&str>)> = spec
        .rows
        .iter()
        .map(|r| (r.kind.as_str(), r.icon.as_deref()))
        .collect();
    assert_eq!(
        got,
        vec![("IconButton", Some(want.as_str())), ("Slider", None)],
        "o glifo nao saiu do desenho, ou vazou para quem nao o consome"
    );
}

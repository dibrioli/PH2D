//! Gates da **APARÊNCIA CONDUZIDA** — arquivo irmão de `vec_driven_style.rs`.

use super::*;

/// Uma cena com uma forma vetorial, opcionalmente já com a opacidade conduzida.
fn cena(alpha: Option<f32>) -> (SimWorld, VecEntityMap, VecPathId) {
    let mut sim = SimWorld::default();
    let mut map = VecEntityMap::new();
    let id: VecPathId = 7;
    let mut e = sim.world_mut().spawn((
        ph2d_ecs::Transform::IDENTITY,
        VecPathRef(id),
        ph2d_ecs::Name::new("Star"),
    ));
    if alpha.is_some() {
        e.insert(VecDrivenStyle { alpha });
    }
    map.insert(id, e.id().to_bits());
    (sim, map, id)
}

/// ⭐ **Sem ninguém a conduzir, a projecção não ganha entrada nenhuma.**
///
/// ⚠️ É a metade que mantém o desenho de todo documento existente **byte-idêntico** ao mundo sem
/// linha do tempo: uma entrada publicada é uma pergunta que o renderer passa a fazer em toda forma
/// da cena, e uma que nunca tem o que responder é custo puro.
#[test]
fn an_undriven_path_publishes_nothing() {
    let (sim, map, _) = cena(None);
    assert!(resolve(&sim, &map).is_empty());
}

/// ⛔ **Uma entidade que não é caminho vetorial não é lida** — o componente vive na ponte, e a
/// ponte é o `VecPathRef`.
#[test]
fn an_entity_without_a_vector_path_is_skipped() {
    let mut sim = SimWorld::default();
    let mut map = VecEntityMap::new();
    let e = sim
        .world_mut()
        .spawn((
            ph2d_ecs::Transform::IDENTITY,
            VecDrivenStyle { alpha: Some(0.5) },
        ))
        .id();
    map.insert(3, e.to_bits());
    assert!(resolve(&sim, &map).is_empty());
}

/// ⭐⭐⭐ **A OPACIDADE CONDUZIDA FUNDE-SE NA ENTRADA QUE JÁ EXISTE, nunca ao lado dela.**
///
/// ⚠️⚠️ **O modo de falha de um `push` é MUDO:** o consumidor lê **uma** entrada por forma
/// (`bound_style(id)` devolve a primeira), então a segunda é descartada — e qual das duas some
/// depende da ordem de iteração de um mapa. Este gate mede a **contagem** e a **sobrevivência do
/// vizinho**, porque só as duas juntas distinguem fundir de empilhar.
#[test]
fn a_driven_alpha_merges_into_the_entry_the_tokens_already_made() {
    let (sim, map, id) = cena(Some(0.5));
    let mut view = VecViewState::default();
    // O que o passe dos TOKENS já publicou para esta mesma forma.
    view.bound.push(BoundStyle {
        path: id,
        fill: Some(ph2d_vec_scene::Rgba8::new(10, 20, 30, 255)),
        ..BoundStyle::default()
    });
    apply(&resolve(&sim, &map), &mut view);
    assert_eq!(view.bound.len(), 1, "a entrada foi EMPILHADA, nao fundida");
    assert_eq!(view.bound[0].alpha, Some(128), "a opacidade nao chegou");
    assert_eq!(
        view.bound[0].fill,
        Some(ph2d_vec_scene::Rgba8::new(10, 20, 30, 255)),
        "a tinta do token foi atropelada pela fusao"
    );
}

/// ⭐ **Sem entrada anterior, ela nasce** — a outra metade do `find`-ou-`push`.
#[test]
fn a_driven_alpha_creates_the_entry_when_there_is_none() {
    let (sim, map, id) = cena(Some(0.25));
    let mut view = VecViewState::default();
    apply(&resolve(&sim, &map), &mut view);
    assert_eq!(view.bound.len(), 1);
    assert_eq!(view.bound[0].path, id);
    assert_eq!(view.bound[0].alpha, Some(64));
}

/// ⛔⛔ **O TOPO DE UM FADE TEM DE FECHAR EM OPACO.**
///
/// ⚠️ **Medido:** `1.0 * 255 = 254,99999` em `f32`, e um `as u8` **trunca** — a forma pararia em
/// `254` e ficaria a um degrau da arte que o artista desenhou. Invisível numa cor chapada, visível
/// numa borda contra o fundo. E `alpha == Some(255)` é também o que o `painted` reconhece como
/// identidade para devolver `Cow::Borrowed`: um degrau abaixo disso clona **toda forma, todo
/// quadro**, para não mudar um pixel.
#[test]
fn the_top_of_a_fade_closes_on_opaque_instead_of_one_step_below() {
    let (sim, map, _) = cena(Some(1.0));
    let mut view = VecViewState::default();
    apply(&resolve(&sim, &map), &mut view);
    assert_eq!(view.bound[0].alpha, Some(255));
    // E o fundo fecha em zero pela mesma conta.
    let (sim, map, _) = cena(Some(0.0));
    let mut view = VecViewState::default();
    apply(&resolve(&sim, &map), &mut view);
    assert_eq!(view.bound[0].alpha, Some(0));
}

/// ⭐⭐⭐ **APAGAR A TRACK DEVOLVE A FORMA AO DOCUMENTO** (v19) — o defeito que o
/// [`super::settle_to_authored`] fecha.
///
/// ⚠️ A ponte da linha do tempo **escreve** o componente e nunca o apaga: sem a reposição, uma
/// forma cuja track foi apagada ficava congelada no último valor da curva **para sempre**, com o
/// documento a dizer outra coisa e nenhum gesto de volta.
///
/// **Mutação que tem de sangrar:** repor a `1.0` em vez do valor do documento (a forma autorada a
/// 40 % saltaria para opaca ao parar a animação), ou repor ANTES de a projecção ser lida (a curva
/// deste quadro desapareceria).
#[test]
fn settling_returns_the_component_to_the_authored_value() {
    use ph2d_vec_scene::{Opacity, VecPath, VecScene, VecVertex};
    let mut sim = SimWorld::default();
    let mut map = VecEntityMap::new();
    let mut scene = VecScene::default();
    let id = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        opacity: Opacity::new(0.4),
        ..VecPath::default()
    });
    let e = sim
        .world_mut()
        .spawn((
            ph2d_ecs::Transform::default(),
            ph2d_ecs::VecPathRef(id),
            // O que a curva deixou no último quadro em que ela existia.
            VecDrivenStyle { alpha: Some(0.9) },
        ))
        .id();
    map.insert(id, e.to_bits());

    super::settle_to_authored(&mut sim, &map, &scene);
    let a = sim
        .world()
        .get::<VecDrivenStyle>(ph2d_ecs::Entity::from_bits(e.to_bits()))
        .and_then(|d| d.alpha)
        .expect("o componente fica — o que muda e' o valor");
    assert!(
        (a - 0.4).abs() <= 1.0 / f32::from(u8::MAX),
        "a forma tem de voltar ao que o DOCUMENTO diz (0,4), e nao ao ultimo valor da curva: {a}"
    );
}

/// **A reposição não CRIA o componente** — a população fica pequena de propósito.
///
/// ⚠️ Semeá-lo em toda forma poria uma entrada de estilo por forma na projecção do quadro, e o
/// `bound_style` é uma varredura linear: `O(N²)` por quadro numa cena grande. *Uma cura que custa
/// um quadrado não é uma cura.*
#[test]
fn settling_never_creates_the_component() {
    use ph2d_vec_scene::{Opacity, VecPath, VecScene, VecVertex};
    let mut sim = SimWorld::default();
    let mut map = VecEntityMap::new();
    let mut scene = VecScene::default();
    let id = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        opacity: Opacity::new(0.4),
        ..VecPath::default()
    });
    let e = sim
        .world_mut()
        .spawn((ph2d_ecs::Transform::default(), ph2d_ecs::VecPathRef(id)))
        .id();
    map.insert(id, e.to_bits());
    super::settle_to_authored(&mut sim, &map, &scene);
    assert!(
        sim.world()
            .get::<VecDrivenStyle>(ph2d_ecs::Entity::from_bits(e.to_bits()))
            .is_none(),
        "uma forma que a linha do tempo nunca conduziu nao pode ganhar o componente aqui"
    );
}

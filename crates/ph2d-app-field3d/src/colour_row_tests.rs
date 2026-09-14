//! ⭐⭐⭐ **A LINHA DA COR** — os três canais dobrados numa amostra, e a travessia sRGB ↔ linear.
//!
//! Enio, 2026-09-14: *«em vez de 3 sliders de RGB, deveríamos ter uma caixa seletora de cor»*.
//!
//! # ⚠️ As três perguntas, e porque são três gates
//!
//! | pergunta | gate |
//! |---|---|
//! | a linha nasce **uma** e traz a cor da peça? | [`a_colour_is_one_row_and_it_carries_the_swatch`] |
//! | o que o artista escolhe **chega ao documento**? | [`the_colour_intent_reaches_the_document`] |
//! | escolher a cor que já lá está **não escreve nada**? | [`the_round_trip_through_the_document_is_exact`] |
//!
//! ⛔ A terceira parece cosmética e é a que impede um **passo de desfazer por quadro**: o painel
//! pergunta *«mudou?»* comparando bytes, e um ida-e-volta que não fecha pede a edição para sempre.
//!
//! ⚠️ **Irmão por assunto** (o `scene_tests.rs` está a `552` de `600`) — ⛔ *split, nunca allowlist*.

use bevy_ecs::entity::Entity;
use ph2d_ecs::SimWorld;
use ph2d_field::{FieldDoc, NodeId, Primitive, Xform};

/// Uma peça de UMA folha — uma esfera. A cor é da folha, e é ela que o painel mostra.
fn a_ball() -> (SimWorld, Entity) {
    let mut sim = SimWorld::new();
    let doc = FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            Primitive::Sphere { radius: 0.3 },
            Xform::IDENTITY,
        )],
        NodeId(0),
    )
    .expect("uma esfera");
    let root = ph2d_field_ecs::spawn_doc(sim.world_mut(), &doc, "peça");
    let folha = sim
        .world()
        .get::<bevy_ecs::hierarchy::Children>(root)
        .and_then(|c| c.iter().next().copied())
        .unwrap_or(root);
    (sim, folha)
}

/// As linhas que o painel publica para uma entidade — pelo caminho de produção, que **drena os
/// pedidos antes de publicar**.
///
/// ⛔ **Ela NÃO drena por si.** A 1.ª redacção começava por um `drain_intents()` defensivo e com
/// isso **comia o pedido que o gate acabara de empurrar** — o `the_colour_intent_reaches_the_document`
/// leu a cor de omissão e acusou uma costura partida que estava inteira. *Um arnês que limpa a fila
/// mede o programa em que ninguém pediu nada.*
fn rows_of(sim: &mut SimWorld, e: Entity) -> Vec<ph2d_panel_model3d::ParamRow> {
    crate::scene::sync_scene_and_birth(sim, None, &[e], 0.0, &crate::scene::no_drawing());
    ph2d_panel_model3d::state::current().rows
}

/// ⭐⭐⭐ **OS TRÊS CANAIS SÃO UMA LINHA, E ELA TRAZ A COR** — em sRGB8, que é o que se vê.
///
/// # ⛔ As duas metades, e porque nenhuma basta
///
/// *Uma linha* sem *a cor certa* é uma amostra cinzenta sobre uma peça vermelha; *a cor certa* em
/// *três linhas* é a dívida que esta wave paga. O gate afirma as duas, e afirma também que a cor
/// **SEGUE** o documento — senão ele passaria sobre uma amostra congelada no valor de omissão.
///
/// **Mutações que devem sangrar:** apagar o `filter` dos canais `1 | 2`; devolver `None` no
/// `swatch`; trocar `base_color_srgb8` por uma conversão linear (a peça a `0,8` leria `204` em vez
/// de `231`).
#[test]
fn a_colour_is_one_row_and_it_carries_the_swatch() {
    let _ = ph2d_panel_model3d::drain_intents();
    let (mut sim, folha) = a_ball();
    let rows = rows_of(&mut sim, folha);
    let cor: Vec<&ph2d_panel_model3d::ParamRow> =
        rows.iter().filter(|r| r.swatch.is_some()).collect();
    assert_eq!(
        cor.len(),
        1,
        "a cor base tem de ser UMA linha-amostra e são {}: {:?}",
        cor.len(),
        rows.iter().map(|r| r.key).collect::<Vec<_>>()
    );
    assert_eq!(cor[0].param, ph2d_field::Param::Material(0), "a âncora");
    assert_eq!(cor[0].key, "field.dim.base_color");
    // ⛔ **E os outros dois canais NÃO são linha** — senão o artista teria a amostra *e* dois
    // sliders do mesmo facto, que é a lei que o `ParamRow::swatch` declara.
    assert!(
        !rows
            .iter()
            .any(|r| matches!(r.param, ph2d_field::Param::Material(1 | 2))),
        "os canais verde e azul continuam a ser linhas próprias: {:?}",
        rows.iter().map(|r| r.key).collect::<Vec<_>>()
    );
    // ⚠️ **E o resto do material continua lá** — a dobra é dos três canais, não da secção.
    assert!(
        rows.iter()
            .any(|r| r.param == ph2d_field::Param::Material(3))
            && rows
                .iter()
                .any(|r| r.param == ph2d_field::Param::Material(4)),
        "a rugosidade e o metal desapareceram com a dobra"
    );

    // ⭐ **A cor de omissão, vista**: o documento guarda `0,8` LINEAR, que em sRGB8 é `231`.
    // ⛔ Um `0,8 × 255 = 204` aqui seria a conversão ingénua — e a amostra sairia visivelmente
    // mais escura do que a peça que o traçado desenha ao lado dela.
    assert_eq!(
        cor[0].swatch,
        Some([231, 231, 231]),
        "a amostra não mostra a cor de omissão do material"
    );

    // ⭐⭐ **E ela SEGUE o documento.** Sem esta metade, uma amostra congelada no default passaria.
    ph2d_field_ecs::set_param(sim.world_mut(), folha, ph2d_field::Param::Material(1), 0.0)
        .expect("o verde");
    ph2d_field_ecs::set_param(sim.world_mut(), folha, ph2d_field::Param::Material(2), 0.0)
        .expect("o azul");
    let rows = rows_of(&mut sim, folha);
    let agora = rows
        .iter()
        .find_map(|r| r.swatch)
        .expect("a linha da cor continua lá");
    assert_eq!(
        agora,
        [231, 0, 0],
        "a amostra não seguiu o documento — ela mostra uma cor que a peça não tem"
    );
}

/// ⭐⭐⭐ **O QUE O ARTISTA ESCOLHE CHEGA AO DOCUMENTO** — a costura do pedido, sem app.
///
/// ⚠️ **Os TRÊS canais num pedido só, e isso é o que o torna UM passo de desfazer**: o registo é por
/// diff uma vez por quadro, então o que importa não é a chamada ser uma, é não haver quadro entre as
/// três escritas.
///
/// **Mutação que deve sangrar:** escrever só o canal `0` no braço do `SetColor`.
#[test]
fn the_colour_intent_reaches_the_document() {
    let _ = ph2d_panel_model3d::drain_intents();
    let (mut sim, folha) = a_ball();
    let _ = rows_of(&mut sim, folha);
    ph2d_panel_model3d::state::push_intent_for_test(ph2d_panel_model3d::ModelIntent::SetColor {
        entity: folha.to_bits(),
        srgb: [255, 0, 128],
    });
    let rows = rows_of(&mut sim, folha);
    assert_eq!(
        rows.iter().find_map(|r| r.swatch),
        Some([255, 0, 128]),
        "a cor escolhida não voltou pela linha — ou o pedido não chegou, ou não fecha o \
         ida-e-volta"
    );
    // ⭐ **E ela mora no COMPONENTE**, que é o que o traçado lê e o que o ficheiro grava.
    let m = sim
        .world()
        .get::<ph2d_field_ecs::FieldMaterial>(folha)
        .copied()
        .expect("escrever a cor materializa o material");
    assert_eq!(
        m.base_color,
        crate::materials::base_color_from_srgb8([255, 0, 128]),
        "os três canais do componente não são os três que o artista apontou"
    );
}

/// ⭐⭐⭐ **O IDA-E-VOLTA PELO DOCUMENTO É EXACTO, nos 256 bytes.**
///
/// # ⛔ Porque isto não é uma curiosidade de colorimetria
///
/// O painel pergunta *«a cor mudou?»* comparando **bytes**: ele lê o selector, converte o documento
/// para sRGB8, e só pede a edição se os dois diferirem. Um único byte que não voltasse ao mesmo
/// valor faria essa comparação ser **sempre verdadeira** naquela cor ⇒ um pedido de edição **por
/// quadro** enquanto o selector estivesse aberto, e um passo de desfazer por cada largar de botão.
///
/// ⚠️ **E o caminho medido é o do PRODUTO**, `f32` incluído: a cor passa pelo `FieldMaterial`, que
/// guarda `f32`. Medir só as duas funções da `ph2d-color` mediria outro programa.
#[test]
fn the_round_trip_through_the_document_is_exact() {
    let mau: Vec<u8> = (0u8..=255)
        .filter(|&b| {
            let ida = crate::materials::base_color_from_srgb8([b, b, b]);
            crate::materials::base_color_srgb8(ida) != [b, b, b]
        })
        .collect();
    assert!(
        mau.is_empty(),
        "⛔ {} byte(s) não sobrevivem ao ida-e-volta pelo documento: {mau:?} — nessas cores o \
         painel pediria uma edição por quadro",
        mau.len()
    );
}

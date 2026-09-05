//! ⭐⭐⭐ **A OPACIDADE DE UM CAMINHO VETORIAL** — o canal que existia e era MUDO (2026-09-04).
//!
//! Irmão de [`apply_from_doc`](../src/apply.rs) e de `prop_readback`, e existe **separado** por uma
//! razão de fixtura: a sonda daqueles spawna um `ph2d_render::Sprite` para o braço de `Opacity`
//! ter o que escrever, e um caminho vetorial é exactamente a entidade que **não** tem um. Medir os
//! dois substratos na mesma fixtura seria medir um deles duas vezes.
//!
//! # O defeito que estes gates fecham
//!
//! Até esta data, `+Track → Opacity` num caminho vetorial criava uma row, aceitava chaves,
//! desenhava a curva no editor de gráfico — e **não movia um pixel**: o braço exigia um `Sprite`, e
//! a entidade de um caminho vetorial nasce com `(Transform, Name, VecPathRef, RootOrder)`. *Um
//! controlo pintado e inerte*, que é a espécie de defeito que este repositório caça há waves.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::{Transform, VecDrivenStyle, VecPathRef, World};
use ph2d_timeline::{PropKind, TimelineDoc, apply_from_doc};

fn s(t: f64) -> RationalTime {
    RationalTime::from_seconds(t)
}

/// Uma entidade de caminho vetorial **como a shell a cria** — e, de propósito, **sem `Sprite`**.
fn vector_path(w: &mut World) -> u64 {
    w.spawn((Transform::default(), VecPathRef(42))).id().to_bits()
}

/// ⭐⭐⭐ **UMA TRACK DE OPACIDADE DESVANECE UM CAMINHO VETORIAL.**
///
/// A rampa de 2 s lida ao meio tem de dar `0,5` — e ela tem de aterrar no
/// [`VecDrivenStyle`], que é a ponte que a shell funde na projecção do quadro.
#[test]
fn an_opacity_track_fades_a_vector_path() {
    let mut w = World::new();
    let bits = vector_path(&mut w);
    let e = ph2d_ecs::Entity::from_bits(bits);

    let mut doc = TimelineDoc::new();
    doc.insert_key(
        bits,
        PropKind::Opacity,
        s(0.0),
        AnimValue::Float(1.0),
        Interp::Linear,
    );
    doc.insert_key(
        bits,
        PropKind::Opacity,
        s(2.0),
        AnimValue::Float(0.0),
        Interp::Hold,
    );

    apply_from_doc(&mut w, &mut doc, 1.0);

    let d = w
        .get::<VecDrivenStyle>(e)
        .expect("a track tem de deixar a opacidade onde a shell a le^");
    let a = d.alpha.expect("o canal alpha e' o que esta track conduz");
    assert!((a - 0.5).abs() < 1e-5, "meio da rampa: {a}");
    assert!(
        !doc.binding_for(bits, PropKind::Opacity)
            .expect("o binding existe")
            .missing,
        "a entidade esta' viva — nada de badge de ausente"
    );
}

/// O `rest` que a linha do tempo semeia para este canal — o valor de ANTES de ela assumir.
/// É o consumidor real do braço de LEITURA, e por isso é por ele que este ficheiro o mede.
fn rest_of(w: &mut World, bits: u64) -> Option<f32> {
    let mut doc = TimelineDoc::new();
    doc.insert_key(
        bits,
        PropKind::Opacity,
        s(0.0),
        AnimValue::Float(0.0),
        Interp::Hold,
    );
    apply_from_doc(w, &mut doc, 0.0);
    doc.binding_for(bits, PropKind::Opacity)
        .expect("o binding existe")
        .rest
}

/// ⛔⛔ **UM CAMINHO AINDA NÃO CONDUZIDO LÊ-SE OPACO, NUNCA ZERO.**
///
/// ⚠️ A ausência do componente quer dizer *"ninguém a conduz"*, e o valor que esta leitura semeia é
/// o **`rest`** — o estado de antes de a linha do tempo assumir. Devolver `0.0` faria **toda track
/// nova** nascer com um fade a partir do invisível, e o artista veria a arte desaparecer no
/// instante em que criasse a row.
#[test]
fn an_undriven_vector_path_reads_opaque_instead_of_zero() {
    let mut w = World::new();
    let bits = vector_path(&mut w);
    assert_eq!(
        rest_of(&mut w, bits),
        Some(1.0),
        "sem componente = opaco, nao invisivel"
    );
}

/// ⭐⭐ **LER É A INVERSA DE ESCREVER** — o par que o `apply_prop` promete no cabeçalho dele.
///
/// ⚠️ Sem este gate o braço de leitura podia ficar preso no `Sprite`, e o `rest` de um caminho
/// vetorial seria semeado do substrato errado: um fade retomado partiria do valor de uma sprite
/// que não existe. A fixtura põe o componente **à mão**, que é exactamente o estado em que uma
/// reprodução anterior o deixa.
#[test]
fn the_rest_of_a_vector_path_comes_from_the_bridge_not_from_a_sprite() {
    let mut w = World::new();
    let bits = vector_path(&mut w);
    w.entity_mut(ph2d_ecs::Entity::from_bits(bits))
        .insert(VecDrivenStyle { alpha: Some(0.25) });
    assert_eq!(rest_of(&mut w, bits), Some(0.25));
}

/// ⛔ **A ESCRITA É EXCLUSIVA**: um caminho vetorial não ganha um `Sprite` de contrabando.
///
/// ⚠️ As duas metades importam. Se o braço novo não tivesse `return`, a escrita cairia também no
/// braço da sprite — inofensivo hoje (não há `Sprite`), e uma bomba no dia em que alguma entidade
/// carregasse os dois.
#[test]
fn fading_a_vector_path_never_invents_a_sprite() {
    let mut w = World::new();
    let bits = vector_path(&mut w);
    let e = ph2d_ecs::Entity::from_bits(bits);

    let mut doc = TimelineDoc::new();
    doc.insert_key(
        bits,
        PropKind::Opacity,
        s(0.0),
        AnimValue::Float(0.5),
        Interp::Hold,
    );
    apply_from_doc(&mut w, &mut doc, 0.0);

    assert!(w.get::<VecDrivenStyle>(e).is_some(), "a ponte foi escrita");
    #[cfg(feature = "render")]
    assert!(
        w.get::<ph2d_render::Sprite>(e).is_none(),
        "a entidade continua a nao ser uma sprite"
    );
}

/// ⛔⛔ **A PONTE É EXCLUSIVA: escrever no vetor NÃO cai também no braço da sprite.**
///
/// ⚠️ **Esta fixtura é SINTÉTICA de propósito, e nasceu de uma mutação que SOBREVIVEU:** tirar o
/// `return` do braço novo não partia gate nenhum, porque nenhuma entidade da suíte carregava os
/// dois componentes — e sem o fenómeno na fixtura, *"o código é redundante"* e *"falta um gate"*
/// leem-se igual. Uma entidade com `VecPathRef` **e** `Sprite` não existe no produto de hoje; o que
/// este gate guarda é o dia em que existir, e nesse dia a escrita dupla seria um valor a saltar
/// entre dois substratos consoante a ordem dos braços.
#[test]
#[cfg(feature = "render")]
fn the_bridge_is_exclusive_and_never_writes_both_substrates() {
    let mut w = World::new();
    let bits = vector_path(&mut w);
    let e = ph2d_ecs::Entity::from_bits(bits);
    w.entity_mut(e)
        .insert(ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]));

    let mut doc = TimelineDoc::new();
    doc.insert_key(
        bits,
        PropKind::Opacity,
        s(0.0),
        AnimValue::Float(0.5),
        Interp::Hold,
    );
    apply_from_doc(&mut w, &mut doc, 0.0);

    assert_eq!(
        w.get::<VecDrivenStyle>(e).and_then(|d| d.alpha),
        Some(0.5),
        "a ponte do vetor tem de receber o valor"
    );
    assert!(
        (w.get::<ph2d_render::Sprite>(e).expect("a sprite esta' la'").tint[3] - 1.0).abs() < 1e-6,
        "a MESMA escrita caiu tambem na sprite — o valor passa a saltar entre dois substratos"
    );
}

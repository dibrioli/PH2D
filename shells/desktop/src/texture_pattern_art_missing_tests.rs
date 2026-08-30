//! ⭐⭐⭐ **A ARTE QUE SUMIU** — os gates do vínculo morto (plano 33, W11).
//!
//! Irmão do [`super::tests`], e separado dele por RESPONSABILIDADE: aquele mede o **memo do
//! assado** (o que re-assa, o que reaproveita, o que a chave contém); este mede o que acontece
//! quando a forma que serve de arte **deixa de existir**.
//!
//! ⚠️ O ficheiro único batia no tecto de LOC do shell — que vive em `shells/desktop/tests/` e
//! **não é alcançado por `cargo test --bins`**.

use super::*;
use ph2d_vec_scene::{PatternFill, Rgba8, VecPath, VecVertex};

/// ⭐⭐⭐ **APAGAR A FORMA-FONTE deixa a estampa sem arte, e o app tem de o saber** (plano 33, W11).
///
/// # O defeito que isto fecha
///
/// Sem esta pergunta, apagar a forma que serve de arte faz a estampa voltar a **cor chapada** — e
/// cor chapada é exactamente o que um preenchimento sólido correcto parece. O `PatternSource` não
/// tem variante vazia, então a secção do painel sobe inteira e normal por cima de um vínculo morto.
///
/// # As quatro respostas, e a terceira é a que uma consulta directa erraria
///
/// ⚠️ `art_is_missing` pergunta pela porta que ASSA, e essa porta recusa também a
/// **auto-referência**: uma forma que se nomeia a si própria como arte não tem arte utilizável (o
/// ladrilho exigiria desenhá-la, e desenhá-la exigiria o ladrilho). Um `scene.path(id).is_none()`
/// escrito à mão daria essa forma como **presente**, e a secção continuaria muda.
#[test]
fn a_deleted_source_shape_is_reported_as_missing_art() {
    let mut scene = VecScene::default();
    let motivo = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    });
    let host = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [8.0, 0.0], [8.0, 8.0], [0.0, 8.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    });

    let viva = PatternSource::Shape(motivo);
    assert!(
        !art_is_missing(&scene, host, &viva),
        "a forma-fonte EXISTE e foi dada como apagada - a seccao ia acusar toda estampa viva"
    );

    let morta = PatternSource::Shape(motivo + 1_000);
    assert!(
        art_is_missing(&scene, host, &morta),
        "um id que nao resolve tem de ser 'arte apagada' - senao a estampa volta a cor chapada e \
         nada no ecra' diz porque'"
    );

    assert!(
        art_is_missing(&scene, host, &PatternSource::Shape(host)),
        "uma forma que se nomeia a si propria nao tem arte utilizavel - a porta que assa recusa-a, \
         e uma consulta directa `scene.path(id).is_none()` da-la-ia como presente"
    );

    let imagem = PatternSource::Image(ph2d_asset::AssetId::from_bytes(&[7; 32]));
    assert!(
        !art_is_missing(&scene, host, &imagem),
        "uma fonte-IMAGEM nunca acusa: os pixels dela viajam no ficheiro, e uma ausencia aqui e' \
         transitoria (a carregar). Um aviso permanente sobre estado transitorio ensina a ignorar."
    );
}

/// ⭐⭐ **O VÍNCULO SOBREVIVE AO UNDO, e é isso que limita o estrago** (plano 33, W11).
///
/// O `VecPathId` é um `u64` guardado **dentro** da `VecScene`, e o `ProjectState` leva a cena
/// inteira como dado — o undo repõe-na verbatim, com os ids originais. ⇒ desfazer o apagar devolve
/// a forma com o MESMO id e o vínculo cura-se sozinho.
///
/// ⚠️ **Sem esta folha a afirmação seria leitura, não medição** — e ela é o que separa *"perde-se
/// trabalho para sempre"* de *"perde-se se apagar, gravar e só reparar depois"*. Se um dia a cena
/// passar a re-atribuir ids ao restaurar (um `push_path` no caminho do restore basta), este gate
/// reprova e o aviso do painel deixa de ser a cura suficiente.
#[test]
fn the_art_link_survives_the_round_trip_that_undo_uses() {
    let mut scene = VecScene::default();
    let motivo = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    });
    let (scene, host) = {
        let mut s = scene;
        let h = s.push_path(VecPath {
            verts: [[0.0, 0.0], [8.0, 0.0], [8.0, 8.0], [0.0, 8.0]]
                .map(VecVertex::corner)
                .to_vec(),
            closed: true,
            fill: Some(Paint::Pattern(Box::new(PatternFill::new(
                PatternSource::Shape(motivo),
                [1.0, 1.0],
                Rgba8::new(10, 20, 30, 255),
            )))),
            ..VecPath::default()
        });
        (s, h)
    };
    // O undo captura e repõe a cena inteira como DADO — o `clone` é essa viagem, e o postcard é a
    // do ficheiro. As duas têm de devolver o mesmo id.
    let clonada = scene.clone();
    let bytes = postcard::to_allocvec(&scene).expect("a cena serializa");
    let lida: VecScene = postcard::from_bytes(&bytes).expect("a cena volta");

    for (nome, s) in [
        ("o clone (undo)", &clonada),
        ("o postcard (ficheiro)", &lida),
    ] {
        let Some(Paint::Pattern(p)) = s.path(host).and_then(|p| p.fill.as_ref()) else {
            panic!("{nome}: a estampa nao voltou");
        };
        assert_eq!(
            p.source,
            PatternSource::Shape(motivo),
            "{nome}: o id da arte MUDOU - desfazer o apagar deixaria de curar o vinculo, e o \
             estrago passaria de recuperavel a permanente"
        );
        assert!(
            !art_is_missing(s, host, &p.source),
            "{nome}: a arte deixou de resolver depois da viagem"
        );
    }
}

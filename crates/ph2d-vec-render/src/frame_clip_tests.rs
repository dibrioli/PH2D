//! Os gates do recorte da MOLDURA (plano UI/UX W0) — irmão de `frame_clip.rs`.
//!
//! # O oráculo, e o que ele prova (e o que NÃO prova)
//!
//! O Vello publica os próprios contadores de camada (`Encoding::n_clips` / `n_open_clips` /
//! `n_paths`), e é deles que estes gates leem — não da escrituração do `OpenParents`, que é a
//! função sob teste. É o mesmo oráculo que os gates de geometria viva deste crate já usam.
//!
//! ⚠️ **Que os pixels fora da moldura desapareçam é o `push_clip` do Vello**, a MESMA camada que
//! todo painel rolável do editor usa — esta wave não reimplementa recorte, então um gate aqui não
//! pode afirmar aparência sem estar a testar o Vello. O que ele prova é o que esta wave de fato
//! decide: **a camada abre sobre o intervalo certo, o fundo sai UMA vez, e o par sempre fecha.**
//! Os pixels são a pergunta do smoke.
//!
//! ⚠️ E há duas coisas no instrumento que se MEDEM, não se supõem, e a 1ª versão destes gates
//! errou as duas:
//!
//! - **uma camada custa DOIS `n_paths`** — a silhueta que recorta é ela própria um caminho
//!   encodado, e o `end_clip` empurra um caminho-fantasma. (Eu havia contado um.)
//! - `encode_end_clip` é **um no-op silencioso quando não há camada aberta**, então um `pop`
//!   sobrando não aparece no `n_open_clips`. Por isso os gates afirmam **os dois** contadores —
//!   `n_clips` conta abre E fecha, e um par a menos aparece lá.

use super::*;
use crate::{FxImages, dispatch};
use ph2d_vec_scene::{Paint, Rgba8, VecParentSpan, VecVertex, VecXforms};
use ph2d_vector::Point;

/// Um quadrado preenchido de lado `s`, na origem.
fn square(scene: &mut VecScene, s: f64) -> VecPathId {
    scene.push_path(VecPath {
        verts: [[0.0, 0.0], [s, 0.0], [s, s], [0.0, s]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        fill: Some(Paint::Solid(Rgba8::new(200, 30, 30, 255))),
        ..VecPath::default()
    })
}

/// A cena da moldura, **na ordem da pilha de z que a árvore dita**: os filhos ao fundo, a moldura
/// por cima (o DFS lista o pai antes, e a pilha é o inverso — ver `vec_zorder::z_order`).
///
/// Devolve `(scene, frame_id, [kid_a, kid_b])`.
fn framed_scene() -> (VecScene, VecPathId, [VecPathId; 2]) {
    let mut scene = VecScene::new();
    let a = square(&mut scene, 1.0);
    let b = square(&mut scene, 1.0);
    let frame = square(&mut scene, 4.0);
    (scene, frame, [a, b])
}

/// Roda o `dispatch` e devolve `(n_clips, n_open_clips, n_paths)` do encoding.
fn encode(scene: &VecScene, view: &VecViewState) -> (u32, u32, u32) {
    let mut target = VectorScene::new();
    dispatch(
        scene,
        view,
        &VecXforms::default(),
        &LiveGeometry::new(),
        &FxImages::new(),
        Affine::IDENTITY,
        &mut target,
    );
    let e = target.inner().encoding();
    (e.n_clips, e.n_open_clips, e.n_paths)
}

/// **O CONTROLE, e a afirmação mais forte da wave:** sem moldura, o encode é o de sempre — zero
/// camadas. É o que sustenta *"o caminho comum é byte-idêntico ao mundo pré-moldura"*.
#[test]
fn a_scene_without_frames_encodes_no_layer() {
    let (scene, _, _) = framed_scene();
    let (clips, open, paths) = encode(&scene, &VecViewState::default());
    assert_eq!(clips, 0, "nenhuma camada");
    assert_eq!(open, 0);
    assert_eq!(paths, 3, "três formas, três desenhos");
}

/// A moldura abre UMA camada, fecha-a, e desenha o próprio fundo UMA vez.
///
/// `n_paths` é o que distingue *desenhada uma vez* de *duas*: três formas + os DOIS caminhos que
/// uma camada custa (a silhueta que recorta, e o fantasma do `end_clip`) = 5. Se a moldura fosse
/// desenhada na abertura **e** na vez dela, sairiam 6.
#[test]
fn the_frame_opens_one_layer_closes_it_and_draws_its_background_once() {
    let (scene, frame, kids) = framed_scene();
    let view = VecViewState {
        parent_spans: vec![VecParentSpan {
            parent: frame,
            first: kids[0],
            clip: true,
        }],
        ..Default::default()
    };
    let (clips, open, paths) = encode(&scene, &view);
    assert_eq!(clips, 2, "um abre e um fecha");
    assert_eq!(open, 0, "a camada FECHA");
    assert_eq!(paths, 5, "3 formas + os 2 caminhos de uma camada");
}

/// **UMA MOLDURA SEM RECORTE AINDA É O FUNDO** (Enio 2026-08-02: *"os filhos estão ficando atrás
/// do pai, logo não podem ser vistos a menos que reduza a opacidade"*).
///
/// A pilha de z é o DFS invertido, então um pai desenha na FRENTE dos filhos; o intervalo existe
/// para antecipar o desenho dele. Enquanto só molduras que RECORTAM tinham intervalo, uma moldura
/// de LAYOUT (recorte desligado) não tinha — e pintava por cima do próprio conteúdo.
///
/// ⚠️ O oráculo separa as duas metades por CONTAGEM, e é isso que o torna capaz de falhar nos
/// dois sentidos: **zero camadas** (ela não recorta) e **3 caminhos** (as três formas, cada uma
/// uma vez). Nasceu VERMELHO — sem intervalo a moldura desenhava na vez dela, por cima; e uma
/// cura que empurrasse camada mesmo assim daria 2 clips.
#[test]
fn a_frame_that_does_not_clip_is_still_the_backdrop() {
    let (scene, frame, kids) = framed_scene();
    let view = VecViewState {
        parent_spans: vec![VecParentSpan {
            parent: frame,
            first: kids[0],
            clip: false,
        }],
        ..Default::default()
    };
    let (clips, open, paths) = encode(&scene, &view);
    assert_eq!(
        clips, 0,
        "ela nao recorta — nenhuma camada devia ser empurrada"
    );
    assert_eq!(open, 0, "e nenhuma ficou pendurada");
    assert_eq!(paths, 3, "as tres formas, cada uma UMA vez");
}

/// ⚠️ **O gate que decide a ordem dentro do laço.** A abertura corre ANTES do filtro de escondido:
/// se ela dependesse de o primeiro descendente desenhar, um filho escondido deixaria a camada por
/// abrir — e o `pop` da moldura fecharia a camada de outra pessoa.
#[test]
fn a_hidden_first_child_still_pairs_the_layer() {
    let (scene, frame, kids) = framed_scene();
    let view = VecViewState {
        hidden: vec![kids[0]],
        parent_spans: vec![VecParentSpan {
            parent: frame,
            first: kids[0],
            clip: true,
        }],
        ..Default::default()
    };
    let (clips, open, _) = encode(&scene, &view);
    assert_eq!(clips, 2, "abre e fecha mesmo com o conteúdo escondido");
    assert_eq!(open, 0);
}

/// Uma moldura ESCONDIDA não desenha o fundo — e a camada continua emparelhada (a visibilidade é
/// `AND` da cadeia, então o conteúdo dela também não desenha).
#[test]
fn a_hidden_frame_draws_no_background_but_still_pairs() {
    let (scene, frame, kids) = framed_scene();
    let view = VecViewState {
        hidden: vec![frame, kids[0], kids[1]],
        parent_spans: vec![VecParentSpan {
            parent: frame,
            first: kids[0],
            clip: true,
        }],
        ..Default::default()
    };
    let (clips, open, paths) = encode(&scene, &view);
    assert_eq!(clips, 2);
    assert_eq!(open, 0);
    assert_eq!(paths, 2, "nada desenhado: só os 2 caminhos da camada");
}

/// Molduras aninhadas fecham em LIFO. As duas abrem no mesmo path (o descendente mais ao fundo é
/// comum), e é a ORDEM da lista — de fora para dentro — que faz o par certo.
#[test]
fn nested_frames_pair_lifo() {
    let mut scene = VecScene::new();
    let leaf = square(&mut scene, 1.0);
    let inner = square(&mut scene, 2.0);
    let outer = square(&mut scene, 4.0);
    let view = VecViewState {
        parent_spans: vec![
            VecParentSpan {
                parent: outer,
                first: leaf,
                clip: true,
            },
            VecParentSpan {
                parent: inner,
                first: leaf,
                clip: true,
            },
        ],
        ..Default::default()
    };
    let (clips, open, paths) = encode(&scene, &view);
    assert_eq!(clips, 4, "duas camadas: dois abres e dois fechas");
    assert_eq!(open, 0);
    assert_eq!(paths, 7, "3 formas + 2 camadas x 2 caminhos");
}

/// Uma lista MAL-FORMADA não pode deixar camada pendurada: o resto do frame — o chrome inteiro —
/// desenha na MESMA cena, e uma camada aberta recortaria tudo o que vier depois.
///
/// ⚠️ **A 1ª fixture disto não continha o fenômeno e a mutação SOBREVIVEU:** ela usava uma moldura
/// que não está na cena (`parent: 9999`), e aí o `resolve` recusa e a camada **nunca abre** — não
/// havia o que fechar. O caso mal-formado que de fato deixa camada pendurada é o do intervalo
/// INVERTIDO: abre num path que vem DEPOIS da moldura, então a vez dela já passou.
#[test]
fn a_malformed_span_never_leaves_a_layer_open() {
    let (scene, frame, kids) = framed_scene();
    let view = VecViewState {
        parent_spans: vec![VecParentSpan {
            // Invertido: a "moldura" é a primeira da pilha e o intervalo abre na última.
            parent: kids[0],
            first: frame,
            clip: true,
        }],
        ..Default::default()
    };
    let (clips, open, _) = encode(&scene, &view);
    assert!(clips > 0, "a fixture TEM de abrir uma camada");
    assert_eq!(open, 0, "o `close_all` fecha o que sobrou");
}

/// **AS ÂNCORAS SEGUEM A POSE DO LAYOUT** (Enio 2026-08-02: *"os Path das formas aparecem no
/// lugar de origem"*).
///
/// O overlay de edição desenha a partir da curva LOCAL + o transform do caminho, e o auto layout
/// não escreve transform nenhum (ADR-0153: *"o passe publica ONDE as coisas ficam"*). Sem a pose
/// as âncoras ficam onde a forma foi autorada enquanto o desenho está onde a moldura a pôs.
///
/// ⚠️ O oráculo é o PONTO que a origem local vai parar, não "o afim mudou": um gate sobre a
/// matriz passaria com a composição na ordem errada, que desloca pela pose ESCALADA.
#[test]
fn the_anchors_follow_the_pose_the_layout_gave() {
    use ph2d_vec_scene::{VecViewState, Xform};
    let xforms = VecXforms::default();
    let cam = Affine::IDENTITY;

    let bare = VecViewState::default();
    let at_rest = crate::overlay_transform(&bare, &xforms, 7, cam) * Point::new(0.0, 0.0);
    assert!(
        (at_rest.x - 0.0).abs() < 1e-9 && (at_rest.y - 0.0).abs() < 1e-9,
        "sem pose a ancora fica onde foi autorada: {at_rest:?}"
    );

    // A moldura empurrou-a 10 para a direita e dobrou-a.
    let placed = VecViewState {
        poses: vec![(7, Xform([2.0, 0.0, 0.0, 2.0, 10.0, 0.0]))],
        ..Default::default()
    };
    let moved = crate::overlay_transform(&placed, &xforms, 7, cam) * Point::new(1.0, 0.0);
    assert!(
        (moved.x - 12.0).abs() < 1e-9 && (moved.y - 0.0).abs() < 1e-9,
        "a ancora local (1,0) devia pousar em 10 + 2*1 = 12: {moved:?}"
    );
}

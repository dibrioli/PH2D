//! Gates da metade do shell do `source.text` (doc 89, folha 14 §3 item 1).
//!
//! ⚠️ O gate que carrega a wave é o da **PORTA CRUZADA**: o shell publica sob a
//! chave que ELE computa e o nó lê sob a chave que o `ctx.param`/`ctx.text_param`
//! DELE computa. Se as duas divergissem, o nó clonaria o externo vazio e emitiria
//! nada — sem erro, sem aviso, com o artista a ver um bloco de texto que sumiu.

use super::{build_stream, publish};
use crate::motion_state::MotionState;
use crate::render_loop::motion_shape_gen::VecPathStore;
use ph2d_node_source_text::{Align, Pivot, TextParams};
use ph2d_nodegraph::attr::{Column, Stream};

fn params(pivot: Pivot) -> TextParams {
    TextParams {
        size: 1.0,
        tracking: 0.0,
        line_height: 1.2,
        align: Align::Left,
        weight: 400.0,
        pivot,
    }
}

/// As colunas que o stream publica, como pares.
fn rows(s: &Stream) -> Vec<([f32; 2], u32)> {
    let Some(Column::Vec2(p)) = Stream::get(s, "P") else {
        return Vec::new();
    };
    let Some(Column::Scalar(g)) = Stream::get(s, "geometry_id") else {
        return Vec::new();
    };
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "o handle e' inteiro pequeno guardado num f32, a convencao do `geometry_id`"
    )]
    p.iter().zip(g).map(|(&a, &b)| (a, b as u32)).collect()
}

/// **A PORTA CRUZADA.** O shell publica; o nó, cozido de verdade, lê e emite uma
/// linha por letra com um handle vivo no store. FALSIFICADO por qualquer
/// divergência entre as duas chaves — o nó leria o externo vazio e a contagem
/// cairia a zero.
#[test]
fn publish_then_cook_the_node_reads_its_own_text() {
    let mut state = MotionState::new();
    let n = state.doc.graph.add_node("source.text");
    state.doc.graph.set_text_param(n, "text", "AB");

    publish(&mut state, 0.0);

    let out = state
        .pump
        .cook
        .cook(&state.doc.graph, &state.registry, n, 0.0)
        .expect("a cena coze");
    let stream = out[0].as_stream();
    assert_eq!(stream.count(), 2, "o no leu o bloco que o shell publicou");
    let r = rows(stream);
    assert!(
        r.iter()
            .all(|&(_, h)| h >= 1 && state.shape_store.get(h).is_some()),
        "todo handle emitido tem geometria no store"
    );
}

/// **Uma instância por CARACTERE** — a wave inteira numa asserção. Um bloco como
/// UMA instância desenharia a mesma imagem e a animação por caractere seria
/// inexprimível.
#[test]
fn a_block_emits_one_instance_per_glyph() {
    let mut store = VecPathStore::default();
    let s = build_stream(&mut store, &params(Pivot::Center), "", "Text");
    assert_eq!(s.count(), 4, "quatro letras, quatro instancias");
}

/// **O espaço não é uma instância, mas É um vão.** Ele não tem contorno (nada a
/// desenhar, e contá-lo faria um `motion.stagger` atrasar por nada), e ainda
/// assim as duas letras têm de ficar mais longe do que ficariam coladas.
#[test]
fn a_space_is_not_an_instance_but_it_is_a_gap() {
    let mut store = VecPathStore::default();
    let p = params(Pivot::Pen);
    let with = build_stream(&mut store, &p, "", "A B");
    let without = build_stream(&mut store, &p, "", "AB");
    assert_eq!(with.count(), 2, "o espaco nao vira instancia");
    assert_eq!(without.count(), 2);
    let (a, b) = (rows(&with), rows(&without));
    assert!(
        a[1].0[0] > b[1].0[0] + 0.05,
        "o espaco AVANCA o pen: {} contra {}",
        a[1].0[0],
        b[1].0[0]
    );
}

/// **Letras repetidas partilham UMA geometria** — é para isto que a chave de
/// glifo existe, e é o que faz um parágrafo custar o alfabeto em vez do
/// comprimento.
#[test]
fn repeated_letters_share_one_geometry() {
    let mut store = VecPathStore::default();
    let s = build_stream(&mut store, &params(Pivot::Center), "", "AAA");
    let r = rows(&s);
    assert_eq!(r.len(), 3);
    assert!(
        r[0].1 == r[1].1 && r[1].1 == r[2].1,
        "os tres A sao o mesmo handle: {:?}",
        r.iter().map(|x| x.1).collect::<Vec<_>>()
    );
    // ...e mesmo handle NAO quer dizer mesmo lugar.
    assert!(r[0].0[0] < r[1].0[0] && r[1].0[0] < r[2].0[0], "o pen anda");
}

/// **Letras DIFERENTES têm geometrias diferentes** — o irmão obrigatório do gate
/// acima, e sem ele a partilha por chave é indistinguível de *todo glifo desenha
/// a mesma letra*. ⚠️ Foi o buraco que a mutação achou: tirar o caractere da
/// chave passava em toda a suíte, e o produto pintaria "AAAA" onde se digitou
/// "Text" — a contagem certa, o lugar certo, e a palavra errada.
///
/// ⚠️ **O pivô é `Pen` e isso é load-bearing.** Sob `Center` o deslocamento é
/// metade do AVANÇO, que já difere entre A e B, então a chave desambigua sozinha
/// e o gate passa **com o caractere fora dela** — verde por acidente do pivô, que
/// foi exatamente o que a 1ª versão deste teste fez. Com `Pen` o shift é zero nos
/// dois e o caractere é o ÚNICO discriminante.
#[test]
fn different_letters_get_different_geometry() {
    let mut store = VecPathStore::default();
    let s = build_stream(&mut store, &params(Pivot::Pen), "", "AB");
    let r = rows(&s);
    assert_eq!(r.len(), 2);
    assert_ne!(r[0].1, r[1].1, "A e B nao podem partilhar geometria");
}

/// **Os dois pivôs desenham a MESMA imagem em repouso.** A geometria anda `−c` e
/// o `P` anda `+c`, então em tamanho unitário e rotação zero a soma é a mesma —
/// e é isso que torna a escolha do default invisível até alguém animar.
///
/// ⚠️ O oráculo é a caixa DESENHADA (`P` + a bbox da geometria interna), nunca o
/// `P` sozinho: comparar `P` faria o gate afirmar que os dois pivôs são iguais,
/// que é precisamente o oposto do que eles são.
#[test]
fn the_two_pivots_draw_the_same_picture_at_rest() {
    let mut store = VecPathStore::default();
    let pen = build_stream(&mut store, &params(Pivot::Pen), "", "Ag");
    let ctr = build_stream(&mut store, &params(Pivot::Center), "", "Ag");
    let (a, b) = (rows(&pen), rows(&ctr));
    assert_eq!(a.len(), b.len());
    for (i, (ra, rb)) in a.iter().zip(&b).enumerate() {
        let ba = drawn_left(&store, *ra);
        let bb = drawn_left(&store, *rb);
        assert!(
            (ba - bb).abs() < 1e-4,
            "glifo {i}: a imagem desenhada tem de coincidir ({ba} vs {bb})"
        );
    }
    // O CONTROLE: os dois pivôs de facto DIFEREM no `P` — senão o gate acima
    // seria verdade por vacuo (dois pivots que nao fazem nada coincidem sempre).
    assert!(
        (a[0].0[0] - b[0].0[0]).abs() > 0.05,
        "os pivots pousam o P em lugares diferentes"
    );
}

/// A borda esquerda do que este par (P, geometria) de facto desenha.
fn drawn_left(store: &VecPathStore, row: ([f32; 2], u32)) -> f64 {
    let path = store.get(row.1).expect("handle vivo");
    let lo = path
        .verts_all()
        .map(|v| v.anchor[0])
        .fold(f64::INFINITY, f64::min);
    lo + f64::from(row.0[0])
}

/// Uma string vazia publica um stream vazio — nada desenhado, nunca um pânico.
#[test]
fn an_empty_string_publishes_an_empty_stream() {
    let mut store = VecPathStore::default();
    let s = build_stream(&mut store, &params(Pivot::Center), "", "");
    assert_eq!(s.count(), 0);
}

/// **O alinhamento move o bloco, e o gate mede o BLOCO** (não um glifo): ao
/// centro, o meio da primeira e da última letra fica em torno da origem.
#[test]
fn align_centre_puts_the_block_around_the_origin() {
    let mut store = VecPathStore::default();
    let mut p = params(Pivot::Pen);
    let left = build_stream(&mut store, &p, "", "Text");
    p.align = Align::Center;
    let mid = build_stream(&mut store, &p, "", "Text");
    let (l, c) = (rows(&left), rows(&mid));
    let span = |r: &Vec<([f32; 2], u32)>| (r[0].0[0] + r[r.len() - 1].0[0]) / 2.0;
    assert!(
        span(&l) > 0.2,
        "alinhado a esquerda o bloco comeca na origem"
    );
    assert!(
        span(&c).abs() < 0.2,
        "ao centro o bloco senta na origem: {}",
        span(&c)
    );
}

/// **Soltar o nó o deixa VIVO** — a 4ª condição de UI (*a sequência leva a algum
/// lugar*), e o gesto aqui é literalmente soltar. O texto de fábrica é escrito no
/// GRAFO, então o painel e o cozido leem o mesmo valor.
///
/// ⚠️ Dirige o `reconcile` — a MESMA porta que o braço `GraphIntent::AddNode`
/// chama e onde a semeadura mora —, e não um `add_node` cru, que é o que uma demo
/// ou um gate faz e que deixa o nó mudo de propósito (o controle abaixo).
#[test]
fn dropping_the_node_leaves_it_alive() {
    let mut state = MotionState::new();
    let before = state.doc.graph.clone();
    let id = state.doc.graph.add_node("source.text");
    crate::render_loop::motion_bridge::reconcile(&mut state, &before);

    assert_eq!(
        state
            .doc
            .graph
            .node_text_param_overrides(id)
            .and_then(|m| m.get("text"))
            .map(String::as_str),
        Some(ph2d_node_source_text::DEFAULT_TEXT),
        "o texto de fabrica esta no GRAFO, onde o painel tambem o le"
    );

    publish(&mut state, 0.0);
    let out = state
        .pump
        .cook
        .cook(&state.doc.graph, &state.registry, id, 0.0)
        .expect("coze");
    assert_eq!(
        out[0].as_stream().count(),
        4,
        "o no solto ja desenha `Text`"
    );
}

/// O CONTROLE: um nó montado por `add_node` — sem gesto — desenha NADA, e é
/// honesto. Sem ele o gate acima não distinguiria *semeado* de *inferido*.
#[test]
fn a_node_built_without_a_gesture_draws_nothing() {
    let mut state = MotionState::new();
    let n = state.doc.graph.add_node("source.text");
    publish(&mut state, 0.0);
    let out = state
        .pump
        .cook
        .cook(&state.doc.graph, &state.registry, n, 0.0)
        .expect("coze");
    assert_eq!(out[0].as_stream().count(), 0);
}

/// ⚠️ **Um texto que o artista ESVAZIOU fica vazio.** A semeadura só toca chave
/// AUSENTE, senão um `reconcile` posterior devolveria "Text" por cima do que ele
/// apagou — o defeito clássico de um default que se re-aplica.
#[test]
fn seeding_never_overwrites_what_the_artist_typed() {
    let mut state = MotionState::new();
    let before = state.doc.graph.clone();
    let id = state.doc.graph.add_node("source.text");
    state.doc.graph.set_text_param(id, "text", "");
    crate::render_loop::motion_bridge::reconcile(&mut state, &before);
    assert_eq!(
        state
            .doc
            .graph
            .node_text_param_overrides(id)
            .and_then(|m| m.get("text"))
            .map(String::as_str),
        Some(""),
        "o vazio do artista sobrevive"
    );
}

//! **O REALCE TEM UMA FONTE SÓ** — a linha que acende e a forma que ganha contorno são o **mesmo
//! objecto**, ou não são nada.
//!
//! # A classe de defeito
//!
//! O realce de proveniência tem **dois produtores** (o ponteiro sobre o canvas · o ponteiro sobre
//! uma linha da Hierarquia) e **dois consumidores** em pontos DIFERENTES do quadro: a Hierarquia
//! publica cedo (`snapshots::publish`, ~2450) e o contorno desenha tarde (~8520).
//!
//! ⚠️ **Se cada consumidor picasse por si, eles picariam contra mapas vivos diferentes** — o
//! `vec_live_drawn` é reescrito no fim do quadro. A linha acesa e a forma contornada passariam a
//! ser objectos diferentes **em movimento**, e cada metade continuaria correcta sozinha: é a
//! assinatura exacta do defeito que esta linha corrigiu três vezes em 2026-08-23 (*pintar e
//! despachar têm de ler a MESMA fonte*).
//!
//! # A lei
//!
//! *Há UM pick de hover por quadro, no topo do `run_render_frame`, e os dois consumidores leem o
//! campo que ele escreve.*

use std::path::{Path, PathBuf};

fn shell(rel: &str) -> String {
    let p: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}

/// ⛔ **O PICK DE HOVER ACONTECE UMA VEZ.**
///
/// A agulha é o nome da porta. Um segundo `pick_hovered_object` em qualquer sítio da shell é, por
/// construção, um segundo pick contra outro estado do quadro.
#[test]
fn the_hover_pick_happens_exactly_once() {
    let mut total = 0;
    for rel in [
        "src/render_loop/mod.rs",
        "src/render_loop/snapshots.rs",
        "src/input_dispatch.rs",
    ] {
        total += shell(rel).matches("pick_hovered_object(").count();
    }
    assert_eq!(
        total, 1,
        "o realce passou a ser picado {total} vezes por quadro — os consumidores estão em pontos \
         diferentes do frame, e dois picks dão dois objectos assim que o mapa vivo mudar entre eles"
    );
}

/// ⛔ **E OS DOIS CONSUMIDORES LEEM O MESMO CAMPO.**
///
/// ⚠️ É a outra metade: um pick só não basta se um dos consumidores voltar a derivar a resposta
/// por outro caminho (o `hot_id` cru, a seleção, o primeiro da lista). O campo é o contrato.
#[test]
fn both_consumers_read_the_one_field() {
    let frame = shell("src/render_loop/mod.rs");
    assert!(
        frame.contains("self.hovered_object = hovered"),
        "o campo do quadro deixou de ser escrito — sem ele não há fonte única a ler"
    );
    // O contorno do canvas.
    assert!(
        frame.contains("let Some(bits) = self.hovered_object"),
        "o contorno do canvas deixou de ler o campo do quadro"
    );
    // A Hierarquia, pela mesma resposta passada ao `publish`.
    assert!(
        frame.contains("self.hovered_object,"),
        "a Hierarquia deixou de receber a resposta do quadro — ela voltaria a derivar a sua"
    );
    assert!(
        shell("src/render_loop/snapshots.rs").contains("entry.hovered = true"),
        "a linha da Hierarquia deixou de acender"
    );
}

/// ⛔ **A HIERARQUIA NÃO PICA O CANVAS.**
///
/// ⚠️ O `snapshots.rs` recebe a resposta RESOLVIDA, e nunca os ingredientes. Se ele ganhasse um
/// pick próprio, ele picaria com o que tem à mão — os pedaços destruturados do `AppGfx`, sem o
/// mapa vivo fundido — e acenderia a linha de um objecto que o clique não pega.
#[test]
fn the_hierarchy_does_not_pick_the_canvas() {
    let snap = shell("src/render_loop/snapshots.rs");
    assert!(
        !snap.contains("pick_all_at_world"),
        "o publicador da Hierarquia ganhou um pick próprio — ele não tem o mapa vivo FUNDIDO à \
         mão, então a linha acesa deixaria de ser a forma que o clique pega"
    );
}

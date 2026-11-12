//! B39.A3: o instantâneo da pilha publicado pelo host lê-se sem clonar.

use crate::state::{current_layers, set_current_layers};
use ph2d_tool_painter::LayerStack;
use std::sync::Arc;

/// B39.A3: ler o instantâneo N vezes no quadro não clona a pilha — todas as leituras são o MESMO
/// `Arc`, e o conteúdo é o publicado.
#[test]
fn ler_a_pilha_publicada_nao_clona_a_pilha() {
    let mut pilha = LayerStack::new();
    pilha.add_raster("Paint", 8, 8);
    set_current_layers(Some(pilha.clone()));
    let a = current_layers().expect("publicada");
    let b = current_layers().expect("publicada");
    assert!(Arc::ptr_eq(&a, &b), "cada leitura clonava a pilha");
    assert_eq!(*a, pilha);
    set_current_layers(None);
    assert!(current_layers().is_none());
}

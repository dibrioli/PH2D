//! A pilha das abas laterais, medida sem pintar (a largura do texto: 6 px por carácter).

use super::*;

fn largura(t: &str) -> f32 {
    t.chars().count() as f32 * 6.0
}

fn snap(crumbs: &[&str]) -> TimelineViewSnapshot {
    TimelineViewSnapshot {
        crumbs: crumbs
            .iter()
            .enumerate()
            .map(|(i, n)| (i, (*n).to_string()))
            .collect(),
        ..TimelineViewSnapshot::default()
    }
}

const LISTA: Rect = Rect {
    x: 100.0,
    y: 50.0,
    w: 800.0,
    h: 300.0,
};

/// ⭐ Dono 13/11: as três abas empilham-se na borda ESQUERDA, por fora, de cima para baixo na ordem
/// de sempre; a ativa mede o nome inteiro, as inativas só o começo, e os alvos não se cruzam.
/// *Mutações: a ativa com `ESPIA` ⇒ o nome corta; o `x` dentro da lista ⇒ vermelho.*
#[test]
fn a_ativa_inteira_e_as_inativas_mostram_o_comeco() {
    for (tab, escolhida) in [(Tab::Keys, 0), (Tab::Containers, 1), (Tab::Arrange, 2)] {
        let abas = pilha(LISTA, &snap(&[]), tab, &mut largura);
        assert_eq!(abas.len(), 3);
        for (i, a) in abas.iter().enumerate() {
            assert!(
                (a.alvo.x + a.alvo.w - LISTA.x).abs() < 0.01,
                "colada à borda"
            );
            assert!(a.alvo.w >= 24.0 && a.alvo.h >= 24.0, "alvo {:?}", a.alvo);
            assert_eq!(a.ativa, i == escolhida);
            if i > 0 {
                let ant = &abas[i - 1].alvo;
                assert!((ant.y + ant.h - a.alvo.y).abs() < 0.01, "pilha sem vão");
            }
            if a.ativa {
                assert_eq!(a.nome, a.inteiro, "a ativa mostra o nome inteiro");
            } else {
                assert!(
                    a.inteiro.starts_with(&a.nome) && !a.nome.is_empty(),
                    "{:?} mostra o começo",
                    a.nome
                );
            }
        }
        assert_eq!(
            abas.iter().map(|a| a.id).collect::<Vec<_>>(),
            vec![
                crate::ids::TIMELINE_TAB_KEYS,
                crate::ids::TIMELINE_TAB_CONTAINERS,
                crate::ids::TIMELINE_TAB_ARRANGE
            ]
        );
    }
    // «Containers» inativa não cabe inteira: o começo, sem reticências.
    let abas = pilha(LISTA, &snap(&[]), Tab::Keys, &mut largura);
    assert!(abas[1].nome.len() < "Containers".len(), "{}", abas[1].nome);
}

/// A inativa estende-se para DEBAIXO da vizinha do lado da ativa (a sobreposição); a ativa entra na
/// moldura. *Mutação: a inativa de baixo estendida para baixo ⇒ vermelho.*
#[test]
fn as_inativas_escondem_um_pedaco_debaixo_da_vizinha() {
    let abas = pilha(LISTA, &snap(&[]), Tab::Containers, &mut largura);
    let (cima, ativa, baixo) = (&abas[0], &abas[1], &abas[2]);
    assert!(cima.desenho.y + cima.desenho.h > ativa.alvo.y);
    assert!(baixo.desenho.y < ativa.alvo.y + ativa.alvo.h);
    assert!(ativa.desenho.x + ativa.desenho.w > LISTA.x);
}

/// Dentro de um container o nível dele é mais uma aba, entre Containers e Arrange, e é a escolhida.
#[test]
fn o_container_aberto_e_mais_uma_aba() {
    let abas = pilha(LISTA, &snap(&["Walk"]), Tab::Containers, &mut largura);
    assert_eq!(abas.len(), 4);
    assert!(abas[2].ativa && abas[2].inteiro == "Walk");
}

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

/// O topo da aba dos comandos, acima da lista (dono 13/11: as abas sobem até ele).
const TOPO: f32 = 20.0;

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
        let abas = pilha((LISTA, TOPO), &snap(&[]), tab, &mut largura);
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
    let abas = pilha((LISTA, TOPO), &snap(&[]), Tab::Keys, &mut largura);
    assert!(abas[1].nome.len() < "Containers".len(), "{}", abas[1].nome);
}

/// A inativa estende-se para DEBAIXO da vizinha do lado da ativa (a sobreposição); a ativa entra na
/// moldura. *Mutação: a inativa de baixo estendida para baixo ⇒ vermelho.*
#[test]
fn as_inativas_escondem_um_pedaco_debaixo_da_vizinha() {
    let abas = pilha((LISTA, TOPO), &snap(&[]), Tab::Containers, &mut largura);
    let (cima, ativa, baixo) = (&abas[0], &abas[1], &abas[2]);
    assert!(cima.desenho.y + cima.desenho.h > ativa.alvo.y);
    assert!(baixo.desenho.y < ativa.alvo.y + ativa.alvo.h);
    assert!(ativa.desenho.x + ativa.desenho.w > LISTA.x);
}

/// Dentro de um container o nível dele é mais uma aba, entre Containers e Arrange, e é a escolhida.
#[test]
fn o_container_aberto_e_mais_uma_aba() {
    let abas = pilha(
        (LISTA, TOPO),
        &snap(&["Walk"]),
        Tab::Containers,
        &mut largura,
    );
    assert_eq!(abas.len(), 4);
    assert!(abas[2].ativa && abas[2].inteiro == "Walk");
}

/// ⭐ Dono 13/11 (2.ª volta): as abas ocupam a lateral INTEIRA da lista (entre as quinas) e crescem
/// com ela; a ativa é a maior. *Mutação: altura fixa por aba ⇒ a soma não acompanha ⇒ vermelho.*
#[test]
fn as_abas_ocupam_a_lateral_inteira_e_crescem_com_ela() {
    use ph2d_editor_core::panel::cards::banda::metrics as m;
    let mut alturas = Vec::new();
    for h in [300.0, 600.0] {
        let lista = Rect { h, ..LISTA };
        let abas = pilha((lista, TOPO), &snap(&[]), Tab::Containers, &mut largura);
        let topo = abas[0].alvo.y;
        let fim = abas.last().map_or(0.0, |a| a.alvo.y + a.alvo.h);
        assert!(
            (topo - TOPO).abs() < 0.01,
            "começa no topo da aba dos comandos"
        );
        assert!(
            (fim - (lista.y + lista.h - m::LIST_RADIUS)).abs() < 0.01,
            "{fim}"
        );
        assert!(abas[1].alvo.h > abas[0].alvo.h && abas[1].alvo.h > abas[2].alvo.h);
        alturas.push(abas[1].alvo.h);
    }
    assert!(
        alturas[1] > alturas[0] * 1.9,
        "a ativa cresce com a lista: {alturas:?}"
    );
}

/// O contorno é uma aba: a base (lado da moldura) mais comprida que o lado de fora.
#[test]
fn o_contorno_tem_a_base_mais_comprida_que_o_lado_de_fora() {
    let p = contorno(
        Rect::new(0.0, 0.0, 28.0, 100.0),
        (Ponta::Inclinada, Ponta::Inclinada),
    );
    let extensao = |x: f32| {
        let ys: Vec<f32> = p
            .iter()
            .filter(|q| (q.0 - x).abs() < 0.01)
            .map(|q| q.1)
            .collect();
        ys.iter().copied().fold(f32::MIN, f32::max) - ys.iter().copied().fold(f32::MAX, f32::min)
    };
    assert!(extensao(28.0) > extensao(0.0) + 1.0);
}

/// ⭐ Dono 13/11 (3.ª volta): sem vão em «V» entre abas — de cada par de vizinhas, a de trás passa
/// RETA por baixo da da frente; as pontas da pilha são retas; e a ativa mostra o nome inteiro.
/// *Mutação: `ponta` sempre `Inclinada` ⇒ vermelho.*
#[test]
fn a_de_tras_passa_por_baixo_e_a_ativa_mostra_o_nome_inteiro() {
    let abas = pilha((LISTA, TOPO), &snap(&[]), Tab::Arrange, &mut largura);
    assert_eq!(abas[0].pontas.0, Ponta::Reta, "o topo da pilha é reto");
    assert_eq!(abas[2].pontas.1, Ponta::Reta, "o pé da pilha é reto");
    // Keys (mais longe) por baixo de Containers; Containers por baixo de Arrange (a ativa).
    assert_eq!(abas[0].pontas.1, Ponta::PorBaixo);
    assert_eq!(abas[1].pontas.0, Ponta::Inclinada);
    assert_eq!(abas[1].pontas.1, Ponta::PorBaixo);
    assert_eq!(abas[2].pontas.0, Ponta::Inclinada);
    assert_eq!(abas[2].nome, "Arrange");
    // Uma lista baixa: a fatia da ativa cresce até ao nome, que nunca corta.
    let baixa = Rect { h: 80.0, ..LISTA };
    for tab in [Tab::Keys, Tab::Containers, Tab::Arrange] {
        let abas = pilha((baixa, TOPO), &snap(&[]), tab, &mut largura);
        let a = abas.iter().find(|a| a.ativa).expect("ativa");
        assert_eq!(a.nome, a.inteiro);
    }
}

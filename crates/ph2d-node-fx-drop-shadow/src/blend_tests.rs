//! **O MODO DA SOMBRA** — os gates do [`super::SHADOW_BLEND`] (doc 89, folha 11).
//!
//! Assunto próprio, arquivo próprio: a maciez responde *que forma o fantasma tem* e isto
//! responde *como ele se mistura com o que está por baixo*. O corte é o mesmo do irmão
//! `softness_tests.rs`.

use super::*;

const BLACK_35: [f32; 4] = [0.0, 0.0, 0.0, 0.35];

fn row(n: usize) -> Stream {
    #[expect(clippy::cast_precision_loss, reason = "uma fixture pequena")]
    let p: Vec<[f32; 2]> = (0..n).map(|i| [i as f32, 0.0]).collect();
    Stream::new(n)
        .with("P", Column::Vec2(p))
        .with("size", Column::Vec2(vec![[0.5, 0.5]; n]))
}

fn blend_of(s: &Stream) -> Option<Vec<f32>> {
    match s.get(BLEND_COLUMN) {
        Some(Column::Scalar(v)) => Some(v.clone()),
        _ => None,
    }
}

/// **`Sink` NÃO ESCREVE A COLUNA**, e é isso que faz o default ser byte-idêntico.
///
/// ⚠️ **A metade que interessa é a segunda:** com um `blend` autorado a montante, escrever `0`
/// por cima seria APAGAR a escolha do artista — e o sintoma seria o rastro a montante a perder o
/// modo dele só por alguém ter posto uma sombra a jusante, sem erro nenhum.
#[test]
fn the_sink_mode_leaves_the_column_exactly_as_it_found_it() {
    let bare = cast(&row(3), 315.0, 0.2, BLACK_35, 0.0, 0.0);
    assert!(
        blend_of(&bare).is_none(),
        "sem coluna a montante e em `Sink`, nada de coluna: o caminho literal"
    );
    let with_upstream = row(3).with(BLEND_COLUMN, Column::Scalar(vec![4.0, 4.0, 4.0]));
    let kept = cast(&with_upstream, 315.0, 0.2, BLACK_35, 0.0, 0.0);
    assert_eq!(
        blend_of(&kept),
        Some(vec![4.0; 6]),
        "em `Sink` a coluna de montante passa TAL E QUAL (o `tile` das outras colunas)"
    );
    // E os valores não-modo caem no mesmo braço, pela mesma porta.
    for junk in [f32::NAN, f32::INFINITY, -3.0, 0.4] {
        assert!(
            blend_of(&cast(&row(2), 315.0, 0.2, BLACK_35, 0.0, junk)).is_none(),
            "um modo lixo ({junk}) conta como `Sink`, nunca como um modo qualquer"
        );
    }
}

/// **SÓ AS LINHAS DO FANTASMA LEVAM O MODO ESCOLHIDO.**
///
/// ⚠️ Uma sombra que impusesse o próprio modo às PEÇAS que a projectam estaria a decidir sobre
/// linhas que não são dela — e o defeito leria como *"pus uma sombra e o meu objecto mudou de
/// cor"*, que é a última coisa em que se olharia.
#[test]
fn only_the_ghost_rows_carry_the_chosen_mode() {
    let n = 3;
    // `4` = Multiply, o default do Photoshop para uma sombra (e o motivo da célula).
    let out = cast(&row(n), 315.0, 0.2, BLACK_35, 0.0, 4.0);
    let col = blend_of(&out).expect("um modo autorado escreve a coluna");
    assert_eq!(col.len(), n * 2, "um fantasma e uma peça por elemento");
    assert_eq!(&col[..n], &[4.0; 3], "os fantasmas multiplicam");
    assert_eq!(
        &col[n..],
        &[0.0; 3],
        "e as pecas continuam a herdar o modo do sink"
    );
}

/// **UM `blend` DE MONTANTE SOBREVIVE NAS PEÇAS** — o rastro escolheu, e a sombra não desfaz.
#[test]
fn an_upstream_mode_survives_on_the_elements_it_shadows() {
    let n = 2;
    let src = row(n).with(BLEND_COLUMN, Column::Scalar(vec![2.0, 5.0]));
    let out = cast(&src, 315.0, 0.2, BLACK_35, 0.0, 4.0);
    let col = blend_of(&out).expect("coluna");
    assert_eq!(&col[..n], &[4.0, 4.0], "o fantasma leva o modo DA SOMBRA");
    assert_eq!(
        &col[n..],
        &[2.0, 5.0],
        "e cada peca leva o modo que ELA trazia"
    );
}

/// **A MACIEZ NÃO MUDA A LEI** — com um disco, TODOS os taps são fantasma.
///
/// ⚠️ Sem isto, o corte entre fantasmas e peças (`n · taps`) poderia ficar preso ao caso de UM
/// tap e a última banda do disco herdaria o modo das peças — visível só com maciez ligada, que
/// é onde ninguém procuraria.
#[test]
fn every_tap_of_a_soft_shadow_is_a_ghost() {
    let n = 2;
    let out = cast(&row(n), 315.0, 0.2, BLACK_35, 0.4, 5.0);
    let col = blend_of(&out).expect("coluna");
    let ghosts = out.count() - n;
    assert!(ghosts > n, "fixture: a maciez tem de multiplicar os taps");
    assert!(
        col[..ghosts].iter().all(|v| (*v - 5.0).abs() < 1e-6),
        "todo tap do disco e' fantasma: {col:?}"
    );
    assert_eq!(&col[ghosts..], &[0.0; 2], "e so' as pecas ficam no sink");
}

/// **O MODO CABE NO QUE O RENDERER SABE DESENHAR.**
///
/// Um nó é uma FOLHA e não alcança o array de pipelines, então ele clampa pela própria lista;
/// o gate que liga as duas pontas vive na shell (`no_node_offers_a_mode_the_renderer_cannot_draw`).
#[test]
fn a_mode_beyond_the_list_is_clamped_to_the_last_one() {
    let top = (SHADOW_BLEND_LABELS.len() - 1) as f32;
    for asked in [top, top + 1.0, 999.0] {
        let out = cast(&row(1), 315.0, 0.2, BLACK_35, 0.0, asked);
        let col = blend_of(&out).expect("coluna");
        assert_eq!(col[0], top, "pediu {asked}, o teto e' {top}");
    }
}

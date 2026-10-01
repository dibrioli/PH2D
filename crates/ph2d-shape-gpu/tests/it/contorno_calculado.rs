//! ⭐⭐ **O CONTORNO CALCULADO desenha o que o EIXO desenha** (doc 121 §9.5) — as MESMAS cópias
//! esticadas pelos dois caminhos do traço: o de sempre (cada pixel refaz a geometria de cada peça
//! do eixo) e o novo (um passe de cálculo escreve as arestas do contorno no ecrã, uma vez por
//! cópia, e o desenho só as soma).
//!
//! ⚠️ **Por que é uma régua de PAR e não contra o Vello:** as duas rotas são a MESMA lei — as
//! arestas de `emite_peca` são as de `peca_do_eixo`, com o sentido que o `orienta` lhes dava, e o
//! que se tira são pares que se anulam. Logo o que sobra entre elas é arredondamento, e a barra é
//! a de arredondamento; contra o Vello a barra das curvas é `100`, e um defeito do contorno caberia
//! inteiro lá dentro.
//!
//! ⚠️⚠️ **E a imagem sozinha NÃO prova que o caminho novo CORREU:** uma cópia cuja escrita não
//! coube ou não bateu na contagem cai no caminho de sempre e desenha a MESMA imagem. ⇒ o gate lê
//! de volta quantas cópias ganharam contorno, e exige-as TODAS depois de a capacidade crescer para
//! o total medido (lido dois quadros depois), com o CONTROLO desligado a ler `0`. No 1.º quadro os
//! círculos não cabem na capacidade de fábrica e caem em parte no caminho de sempre: é aí que se
//! mede que a MISTURA dos dois caminhos desenha a mesma imagem.
//!
//! ```text
//! cargo test -p ph2d-shape-gpu --test it -- --ignored --nocapture contorno
//! ```

use ph2d_shape_gpu::FillRule;
use ph2d_vector::{Cap, Join, Stroke};

use super::paridade_com_o_vello::{
    Copia, Forma, circulo, esticadas, estrela, gpu, pelo_passe_com, zigue_zague,
};

/// O pior desvio de alfa entre as duas imagens, quantos pixels desviam mais de `1`, e quantos têm
/// tinta (a população — uma fixtura que não desenha passa em qualquer barra).
fn desvio(a: &[u8], b: &[u8]) -> (u8, usize, usize) {
    let (mut pior, mut acima, mut tinta) = (0u8, 0usize, 0usize);
    for (x, y) in a.as_chunks::<4>().0.iter().zip(b.as_chunks::<4>().0.iter()) {
        let d = x[3].abs_diff(y[3]);
        pior = pior.max(d);
        if d > 1 {
            acima += 1;
        }
        if x[3] > 0 || y[3] > 0 {
            tinta += 1;
        }
    }
    (pior, acima, tinta)
}

/// ⭐⭐ **As famílias do traço esticado — esquadria, chanfro, redondo, pontas, traço fino e a cerca
/// da faixa —, pelos dois caminhos, pixel a pixel.**
#[test]
#[ignore = "precisa de adapter de GPU"]
fn o_contorno_calculado_desenha_o_que_o_eixo_desenha() {
    let Some(gpu) = gpu() else {
        eprintln!("sem adaptador — nada a medir");
        return;
    };
    let (est, circ, zz) = (estrela(), circulo(), zigue_zague());
    let traco = |w: f64, j: Join| Stroke::new(w).with_join(j);
    let casos: Vec<(&str, Forma<'_>, Vec<Copia>)> = vec![
        (
            "estrela, esquadria",
            Forma {
                bp: &est,
                linha: None,
                regra: FillRule::NonZero,
                traco: Some((traco(0.06, Join::Miter), [0.1, 0.1, 0.1, 1.0])),
            },
            esticadas(40, 40.0, 220.0, 6),
        ),
        (
            "circulo, esquadria",
            Forma {
                bp: &circ,
                linha: None,
                regra: FillRule::NonZero,
                traco: Some((traco(0.08, Join::Miter), [0.9, 0.2, 0.1, 1.0])),
            },
            esticadas(40, 30.0, 200.0, 7),
        ),
        (
            "zigue-zague, chanfro e pontas quadradas",
            Forma {
                bp: &zz,
                linha: Some(&zz),
                regra: FillRule::NonZero,
                traco: Some((
                    traco(0.07, Join::Bevel).with_caps(Cap::Square),
                    [0.1, 0.5, 0.2, 1.0],
                )),
            },
            esticadas(40, 40.0, 220.0, 8),
        ),
        (
            "zigue-zague, redondo",
            Forma {
                bp: &zz,
                linha: Some(&zz),
                regra: FillRule::NonZero,
                traco: Some((
                    traco(0.07, Join::Round).with_caps(Cap::Round),
                    [0.3, 0.1, 0.6, 1.0],
                )),
            },
            esticadas(40, 40.0, 220.0, 9),
        ),
        (
            "traco fino",
            Forma {
                bp: &est,
                linha: None,
                regra: FillRule::NonZero,
                traco: Some((traco(0.01, Join::Miter), [0.0, 0.0, 0.0, 1.0])),
            },
            esticadas(60, 40.0, 220.0, 10),
        ),
        (
            "estrelas pequenas, traco grosso",
            Forma {
                bp: &est,
                linha: None,
                regra: FillRule::NonZero,
                traco: Some((traco(0.12, Join::Miter), [0.1, 0.1, 0.1, 1.0])),
            },
            esticadas(120, 10.0, 24.0, 12),
        ),
        (
            "estrela, limite 2",
            Forma {
                bp: &est,
                linha: None,
                regra: FillRule::NonZero,
                traco: Some((
                    traco(0.06, Join::Miter).with_miter_limit(2.0),
                    [0.1, 0.1, 0.1, 1.0],
                )),
            },
            esticadas(40, 40.0, 220.0, 13),
        ),
    ];
    let fmt = wgpu::TextureFormat::Rgba16Float;
    let mut transbordou = Vec::new();
    for (nome, forma, cs) in &casos {
        let n = u32::try_from(cs.len()).expect("cabem");
        let eixo = pelo_passe_com(&gpu, forma, cs, fmt, false, QUADROS);
        let contorno = pelo_passe_com(&gpu, forma, cs, fmt, true, QUADROS);
        // O CONTROLO: desligado, nenhuma cópia ganha contorno em quadro nenhum — senão a comparação
        // mede o caminho novo contra ele próprio.
        assert!(
            eixo.iter().all(|(_, sem)| *sem == 0),
            "{nome}: o caminho de sempre nao correu"
        );
        let referencia = &eixo[QUADROS - 1].0;
        for (q, (img, com)) in contorno.iter().enumerate() {
            let (pior, acima, tinta) = desvio(referencia, img);
            eprintln!(
                "  {nome:<40} quadro {q}: contorno em {com}/{n} · alfa max {pior} · px > 1: {acima} de {tinta}"
            );
            assert!(tinta > 1000, "{nome}: a fixtura quase nao desenha");
            // ⭐ Em TODO quadro, também no 1.º, em que as cópias que não cabem caem no caminho de
            // sempre — uma mistura dos dois tem de desenhar a mesma imagem que cada um.
            assert!(
                pior <= ALFA_MAX && acima <= tinta / 1000,
                "{nome}, quadro {q}: o contorno calculado desenha outra coisa que o eixo (alfa {pior}, {acima} px > 1)"
            );
        }
        if contorno[0].1 < n {
            transbordou.push(*nome);
        }
        assert_eq!(
            contorno[QUADROS - 1].1,
            n,
            "{nome}: a capacidade nao cresceu para o total medido — o contorno nao correu em todas as copias"
        );
    }
    // ⚠️ A metade que torna o 1.º quadro uma régua: alguma fixtura tem de TRANSBORDAR a capacidade
    // de fábrica (os círculos: `32` arestas por cópia não cobrem o leque de uma curva), senão o
    // caminho de recurso por cópia nunca é exercido e a mistura dos dois nunca é medida.
    assert!(
        !transbordou.is_empty(),
        "nenhuma fixtura transbordou no 1.º quadro: o recurso por copia nao foi medido"
    );
    eprintln!("  transbordaram no 1.º quadro: {transbordou:?}");
}

/// Quadros por corrida: o total é copiado no 1.º, mapeado no 2.º e colhido no 3.º, que já desenha
/// com a capacidade nova (`contorno.rs`). O 4.º confirma que ela FICA.
const QUADROS: usize = 4;

/// A barra: as duas rotas são a mesma lei, logo o que sobra é arredondamento de `f32` — as arestas
/// no ecrã saem de um passe de cálculo e as do eixo de um de fragmento.
const ALFA_MAX: u8 = 2;

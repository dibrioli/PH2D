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
use ph2d_vector::{BezPath, Cap, Circle, Join, Shape, Stroke};

use super::paridade_com_o_vello::{
    Copia, Forma, anel, circulo, copias, esticadas, estrela, gpu, pelo_passe_com, pelo_passe_rota,
    zigue_zague,
};

/// As MARCAS de um traço: um disco pequeno pintado com a cor dele, fora do contorno da estrela.
fn marca() -> BezPath {
    Circle::new((0.32, 0.0), 0.12).to_path(0.01)
}

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
    let (est, circ, zz, an, mc) = (estrela(), circulo(), zigue_zague(), anel(), marca());
    let traco = |w: f64, j: Join| Stroke::new(w).with_join(j);
    let casos: Vec<(&str, Forma<'_>, Vec<Copia>)> = vec![
        (
            "estrela, esquadria",
            Forma {
                bp: &est,
                linha: None,
                regra: FillRule::NonZero,
                marcas: None,
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
                marcas: None,
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
                marcas: None,
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
                marcas: None,
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
                marcas: None,
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
                marcas: None,
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
                marcas: None,
                traco: Some((
                    traco(0.06, Join::Miter).with_miter_limit(2.0),
                    [0.1, 0.1, 0.1, 1.0],
                )),
            },
            esticadas(40, 40.0, 220.0, 13),
        ),
        // ⭐⭐ doc 121 §9.6 — as arestas no ECRÃ deixaram de ser só o contorno do eixo: o
        // preenchimento, as marcas e o traço CONFORME vão pelo mesmo cálculo, e as máscaras de
        // linha passam de uma palavra (`> 32` blocos) nas cópias GRANDES.
        (
            "estrelas pequenas, so preenchimento",
            Forma {
                bp: &est,
                linha: None,
                regra: FillRule::NonZero,
                marcas: None,
                traco: None,
            },
            copias(400, 6.0, 40.0, 1),
        ),
        (
            "aneis even-odd",
            Forma {
                bp: &an,
                linha: None,
                regra: FillRule::EvenOdd,
                marcas: None,
                traco: None,
            },
            copias(60, 30.0, 400.0, 11),
        ),
        (
            "circulos com traco, conformes",
            Forma {
                bp: &circ,
                linha: None,
                regra: FillRule::NonZero,
                marcas: None,
                traco: Some((traco(0.08, Join::Miter), [0.9, 0.2, 0.1, 1.0])),
            },
            copias(60, 30.0, 300.0, 3),
        ),
        (
            "circulos grandes com traco, conformes",
            Forma {
                bp: &circ,
                linha: None,
                regra: FillRule::NonZero,
                marcas: None,
                traco: Some((traco(0.04, Join::Round), [0.2, 0.2, 0.9, 1.0])),
            },
            copias(6, 400.0, 900.0, 14),
        ),
        (
            "circulos grandes esticados, redondo",
            Forma {
                bp: &circ,
                linha: None,
                regra: FillRule::NonZero,
                marcas: None,
                traco: Some((traco(0.04, Join::Round), [0.2, 0.6, 0.3, 1.0])),
            },
            esticadas(6, 400.0, 900.0, 15),
        ),
        (
            "estrela com traco e marcas, conforme",
            Forma {
                bp: &est,
                linha: None,
                regra: FillRule::NonZero,
                marcas: Some(&mc),
                traco: Some((traco(0.06, Join::Miter), [0.1, 0.1, 0.1, 1.0])),
            },
            copias(40, 40.0, 220.0, 16),
        ),
        (
            "estrela esticada com traco e marcas",
            Forma {
                bp: &est,
                linha: None,
                regra: FillRule::NonZero,
                marcas: Some(&mc),
                traco: Some((traco(0.06, Join::Miter), [0.1, 0.1, 0.1, 1.0])),
            },
            esticadas(40, 40.0, 220.0, 17),
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

/// ⭐ doc 121 §9.6 — **A ROTA: uma cópia CONFORME pequena fica no caminho de sempre, uma grande e
/// uma ESTICADA vão pelas arestas no ecrã.** O cálculo é pago por cópia, e na escada de `32 768`
/// estrelas pequenas custava `+3,9 ms` por zero ganho no desenho. ⚠️ Com a área mínima a `0` as
/// mesmas cópias pequenas vão TODAS — é o CONTROLO de que a rota é a área e não outra coisa.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn so_as_copias_conformes_grandes_pagam_o_calculo() {
    let Some(gpu) = gpu() else {
        eprintln!("sem adaptador — nada a medir");
        return;
    };
    let est = estrela();
    let forma = Forma {
        bp: &est,
        linha: None,
        regra: FillRule::NonZero,
        marcas: None,
        traco: None,
    };
    let fmt = wgpu::TextureFormat::Rgba16Float;
    let area = ph2d_shape_gpu::AREA_MINIMA_CONFORME;
    let lado = |a: f32| a.sqrt();
    // Pequenas: a caixa (a de uma estrela rodada cabe num quadrado do lado) bem abaixo da área.
    let pequenas = copias(60, 0.3 * lado(area), 0.5 * lado(area), 21);
    let grandes = copias(20, 1.5 * lado(area), 3.0 * lado(area), 22);
    let esticadas_pequenas = esticadas(60, 0.3 * lado(area), 0.5 * lado(area), 23);
    let com = |f: &Forma<'_>, cs: &[Copia], a: f32| {
        pelo_passe_rota(&gpu, f, cs, fmt, true, 3, a)
            .pop()
            .expect("tres quadros")
            .1
    };
    assert_eq!(
        com(&forma, &pequenas, area),
        0,
        "as conformes pequenas pagaram o calculo"
    );
    assert_eq!(
        com(&forma, &pequenas, 0.0),
        60,
        "CONTROLO: com a area a 0 vao todas"
    );
    assert_eq!(
        com(&forma, &grandes, area),
        20,
        "as conformes grandes ficaram no caminho de sempre"
    );
    // Uma ESTICADA com traço vai sempre, pequena ou não: é ela que deixa de refazer o eixo por pixel.
    let com_traco = Forma {
        traco: Some((
            Stroke::new(0.06).with_join(Join::Miter),
            [0.1, 0.1, 0.1, 1.0],
        )),
        ..forma
    };
    assert_eq!(
        com(&com_traco, &esticadas_pequenas, area),
        60,
        "as esticadas com traco ficaram no caminho de sempre"
    );
}

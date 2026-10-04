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
    Copia, Forma, anel, bytes_de_textura, circulo, copias, esticadas, estrela, gpu,
    pelo_passe_celulas, pelo_passe_com, pelo_passe_em_etapas, pelo_passe_observado,
    pelo_passe_rota, separa, textura, zigue_zague,
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
        let contorno = pelo_passe_celulas(&gpu, forma, cs, fmt, true, QUADROS, 0.0, u64::MAX);
        // O CONTROLO: desligado, nenhuma cópia ganha contorno em quadro nenhum — senão a comparação
        // mede o caminho novo contra ele próprio.
        assert!(
            eixo.iter().all(|(_, sem)| *sem == 0),
            "{nome}: o caminho de sempre nao correu"
        );
        let referencia = &eixo[QUADROS - 1].0;
        for (q, (img, com, _)) in contorno.iter().enumerate() {
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
        // ⭐ doc 121 §9.12 — e as CÉLULAS também: no regime do produto todas cabem (senão a régua mede o
        // recurso por cópia e não a acumulação).
        let (pedido, cap) = contorno[QUADROS - 1].2;
        assert!(
            pedido > 0 && pedido <= cap,
            "{nome}: as celulas pediram {pedido} com capacidade {cap} — alguma copia nao correu pelas celulas"
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

/// Quadros por corrida: os totais das arestas e das células são copiados no 1.º, mapeados no 2.º e
/// colhidos no 3.º, que já desenha tudo pelas células (`contorno.rs`, doc 121 §9.12). O 4.º confirma
/// que a capacidade FICA.
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

/// ⭐ doc 121 §9.12 — **UMA CÓPIA QUE NÃO CABE NAS CÉLULAS DESENHA A MESMA IMAGEM.** A capacidade das
/// células cresce para o total medido dois quadros depois; até lá (e no tecto do recurso) uma cópia
/// que não cabe vai INTEIRA pelo caminho de sempre. Aqui o tecto é METADE das células pedidas, em todos
/// os quadros: parte das cópias pelas células, o resto pelo caminho de sempre, e a imagem é a do eixo
/// à barra de arredondamento — a acumulação de uma não pode tocar na de outra.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn as_copias_que_nao_cabem_nas_celulas_desenham_o_mesmo() {
    let Some(gpu) = gpu() else {
        eprintln!("sem adaptador — nada a medir");
        return;
    };
    let (est, circ, an) = (estrela(), circulo(), anel());
    let casos: Vec<(&str, Forma<'_>, Vec<Copia>)> = vec![
        (
            "estrela esticada, esquadria",
            Forma {
                bp: &est,
                linha: None,
                regra: FillRule::NonZero,
                marcas: None,
                traco: Some((
                    Stroke::new(0.06).with_join(Join::Miter),
                    [0.1, 0.1, 0.1, 1.0],
                )),
            },
            esticadas(40, 40.0, 220.0, 31),
        ),
        (
            "circulos grandes esticados, redondo",
            Forma {
                bp: &circ,
                linha: None,
                regra: FillRule::NonZero,
                marcas: None,
                traco: Some((
                    Stroke::new(0.04).with_join(Join::Round),
                    [0.2, 0.6, 0.3, 1.0],
                )),
            },
            esticadas(6, 400.0, 900.0, 32),
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
            copias(60, 30.0, 400.0, 33),
        ),
    ];
    let fmt = wgpu::TextureFormat::Rgba16Float;
    for (nome, forma, cs) in &casos {
        let n = u32::try_from(cs.len()).expect("cabem");
        let eixo = pelo_passe_com(&gpu, forma, cs, fmt, false, 1)
            .pop()
            .expect("um quadro")
            .0;
        let livre = pelo_passe_celulas(&gpu, forma, cs, fmt, true, QUADROS, 0.0, u64::MAX);
        let pedido = livre[QUADROS - 1].2.0;
        let metade = pelo_passe_celulas(&gpu, forma, cs, fmt, true, QUADROS, 0.0, pedido / 2);
        for (q, (img, com, (pediu, cap))) in metade.iter().enumerate() {
            let (pior, acima, tinta) = desvio(&eixo, img);
            eprintln!(
                "  {nome:<40} quadro {q}: contorno em {com}/{n} · celulas {pediu}/{cap} · alfa max {pior} · px > 1: {acima} de {tinta}"
            );
            assert!(tinta > 1000, "{nome}: a fixtura quase nao desenha");
            assert!(
                pior <= ALFA_MAX && acima <= tinta / 1000,
                "{nome}, quadro {q}: com copias fora das celulas o desenho e outro (alfa {pior}, {acima} px > 1)"
            );
        }
        // O CONTROLO: o tecto mordeu (cópias caíram no caminho de sempre) e outras continuaram nas
        // células — senão o que se mediu foi só um dos dois caminhos.
        let (_, com, (pediu, cap)) = metade[QUADROS - 1];
        assert!(
            pediu > cap,
            "{nome}: o tecto nao mordeu ({pediu} de {cap}) — o recurso por copia nao foi medido"
        );
        assert!(
            com > 0 && com < n,
            "{nome}: {com} de {n} copias nas celulas — a mistura dos dois caminhos nao foi medida"
        );
    }
}

/// ⭐ doc 121 §9.8 — **UMA CENA QUE MUDA NÃO LÊ AS ARESTAS DO QUADRO ANTERIOR.** A contagem RESERVA para
/// cada cópia um pior caso e a escrita usa só parte; os passes das células correm um fio por aresta
/// RESERVADA, e só as escritas contam. Num passe novo o resto da reserva é zero e não soma nada — por
/// isso nenhuma régua de quadro único o vê —, mas numa cena animada os buffers trazem o que o quadro
/// anterior lá escreveu, noutro sítio (e a ACUMULAÇÃO do §9.12, os depósitos dele, se o `cs_zera` não
/// os apagar). Aqui o MESMO passe desenha estrelas grandes de junta redonda e
/// depois outras, menores e noutros sítios: cada quadro da 2.ª etapa tem de ser o de um passe novo.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn uma_cena_que_muda_nao_le_as_arestas_do_quadro_anterior() {
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
        traco: Some((
            Stroke::new(0.08).with_join(Join::Round),
            [0.1, 0.1, 0.1, 1.0],
        )),
    };
    let fmt = wgpu::TextureFormat::Rgba16Float;
    let antes = esticadas(40, 60.0, 260.0, 41);
    let depois = esticadas(60, 20.0, 140.0, 42);
    let novo = pelo_passe_celulas(&gpu, &forma, &depois, fmt, true, QUADROS, 0.0, u64::MAX)
        .pop()
        .expect("um quadro")
        .0;
    let seguido = pelo_passe_em_etapas(
        &gpu,
        &forma,
        &[(&antes, QUADROS), (&depois, QUADROS)],
        fmt,
        true,
        0.0,
        u64::MAX,
    );
    let n = u32::try_from(depois.len()).expect("cabem");
    for (q, (img, com, (pediu, cap))) in seguido[QUADROS..].iter().enumerate() {
        let (pior, acima, tinta) = desvio(&novo, img);
        eprintln!(
            "  depois de outra cena, quadro {q}: contorno em {com}/{n} · celulas {pediu}/{cap} · alfa max {pior} · px > 1: {acima} de {tinta}"
        );
        assert!(tinta > 1000, "a fixtura quase nao desenha");
        assert_eq!(
            *com, n,
            "quadro {q}: nem todas as copias foram pelas celulas"
        );
        assert!(
            pior <= ALFA_MAX && acima <= tinta / 1000,
            "quadro {q}: depois de outra cena o desenho e outro (alfa {pior}, {acima} px > 1)"
        );
    }
}

/// ⭐ doc 121 §9.14 (c) — **o REDESENHO é o mesmo desenho**: o halo do `fx.glow` repete o último
/// desenho no RT dele (`ShapePass::redesenha`), sem recalcular as células, e é dele que sai a forma
/// que brilha num quadro do dispositivo. Byte a byte, em cada quadro (o 1.º com a mistura dos dois
/// caminhos, os seguintes pelas células). CONTROLO: o alvo do redesenho começa transparente, logo um
/// redesenho mudo lê zero de tinta.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn o_redesenho_e_o_mesmo_desenho() {
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
        traco: Some((
            Stroke::new(0.06).with_join(Join::Miter),
            [0.1, 0.2, 0.9, 1.0],
        )),
    };
    let cs = esticadas(40, 40.0, 220.0, 6);
    let fmt = wgpu::TextureFormat::Rgba8Unorm;
    let le = |b: Vec<u8>| -> Vec<u8> {
        b.as_chunks::<4>()
            .0
            .iter()
            .flat_map(|px| separa(px.map(|c| f32::from(c) / 255.0)))
            .collect()
    };
    let mut redesenhos = Vec::new();
    let quadros = pelo_passe_observado(
        &gpu,
        &forma,
        &[(&cs, 3)],
        fmt,
        (true, 0.0, u64::MAX),
        &mut |g, p| {
            let tex = textura(g, wgpu::TextureUsages::RENDER_ATTACHMENT, fmt);
            let vista = tex.create_view(&wgpu::TextureViewDescriptor::default());
            let mut enc = g
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
            drop(enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("limpa"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &vista,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            }));
            p.redesenha(g, &mut enc, &vista);
            g.queue.submit(Some(enc.finish()));
            redesenhos.push(le(bytes_de_textura(g, &tex, 4)));
        },
    );
    assert_eq!(quadros.len(), redesenhos.len());
    for (k, ((img, _, _), red)) in quadros.iter().zip(&redesenhos).enumerate() {
        let tinta = red.as_chunks::<4>().0.iter().filter(|px| px[3] > 0).count();
        let difere = img
            .as_chunks::<4>()
            .0
            .iter()
            .zip(red.as_chunks::<4>().0)
            .filter(|(a, b)| a != b)
            .count();
        eprintln!("  quadro {k}: {tinta} px de tinta no redesenho · {difere} px diferentes");
        assert!(
            tinta > 10_000,
            "quadro {k}: o redesenho nao desenhou ({tinta} px)"
        );
        assert_eq!(difere, 0, "quadro {k}: o redesenho difere do desenho");
    }
}

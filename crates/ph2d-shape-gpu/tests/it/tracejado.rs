//! ⭐⭐ **O TRACEJADO SOB ESCALA NÃO UNIFORME** (doc 121 §9.9) — o eixo percorrido no ecrã contra a lei
//! da casa (o Vello sobre a geometria transformada, caneta e padrão `× √|det|`), e o contorno
//! calculado contra o caminho pixel a pixel.
//!
//! ```text
//! cargo test -p ph2d-shape-gpu --test it -- --ignored --nocapture tracejado
//! ```

use ph2d_shape_gpu::{EixoItem, FillRule, ShapeGeometry, ShapeInput, ShapePass, StrokeInput};
use ph2d_vector::{BezPath, Cap, Join, Stroke};

use super::paridade_com_o_vello::{
    Copia, Forma, circulo, copias, corre, esticadas, estrela, gpu, pelo_passe_com,
    pelo_passe_observado, zigue_zague,
};

/// ⭐ **A barra, do VALE medido** (2026-10-02, RTX, meio-float; arnês
/// `docs/Motion Nodes/ferramentas/mutacao_o_tracejado_no_ecra_2026-10-02.py`): a rota que shipa lê
/// alfa `≤ 84` · cor `≤ 89` em todas as famílias; as 16 mutações do percurso leem alfa `126`–`255`.
/// ⇒ `100`/`100`, a barra das curvas. ⚠️ A FRACÇÃO de pixels com alfa `> 1` (a régua do traço
/// esticado contínuo) NÃO separa aqui: o controlo conforme — o caminho pré-expandido, intocado — lê
/// `5,7 %` (cada traço com pontas redondas é borda curva), e a emenda que falta lê `2,4 %`.
const BARRA: u8 = 100;

/// Uma estrela com um FURO hexagonal: dois sub-caminhos fechados de comprimentos diferentes. O ajuste
/// no ecrã fecha o mais LONGO num número inteiro de períodos — o de dentro não, e é nele que o último
/// traço EMENDA no primeiro. ⚠️ Hexágono e não estrela: numa quina de `120°` a faixa SERVE (numa ponta
/// de estrela o recuo passa sempre dos pedaços), e é aí que o recuo da emenda decide alguma coisa.
pub(super) fn estrela_com_furo() -> BezPath {
    let mut bp = estrela();
    for i in 0..6 {
        let a = std::f64::consts::PI * f64::from(i) / 3.0 + 0.2;
        let p = (0.14 * a.cos(), 0.14 * a.sin());
        if i == 0 {
            bp.move_to(p);
        } else {
            bp.line_to(p);
        }
    }
    bp.close_path();
    bp
}

fn casos<'a>(
    est: &'a BezPath,
    circ: &'a BezPath,
    zz: &'a BezPath,
    furo: &'a BezPath,
) -> Vec<(&'static str, Forma<'a>, Vec<Copia>)> {
    let forma = |bp: &'a BezPath, linha: Option<&'a BezPath>, s: Stroke, cor: [f32; 4]| Forma {
        bp,
        linha,
        regra: FillRule::NonZero,
        marcas: None,
        traco: Some((s, cor)),
    };
    vec![
        (
            "estrela tracejada esticada",
            forma(
                est,
                None,
                Stroke::new(0.06)
                    .with_join(Join::Miter)
                    .with_dashes(0.0, [0.12, 0.08]),
                [0.1, 0.1, 0.1, 1.0],
            ),
            esticadas(40, 40.0, 220.0, 21),
        ),
        // Traços mais longos que um troço: a faixa e a junta DENTRO de um traço, e a emenda no início.
        (
            "estrela de tracos longos esticada",
            forma(
                est,
                None,
                Stroke::new(0.05)
                    .with_join(Join::Miter)
                    .with_dashes(0.0, [0.9, 0.25]),
                [0.6, 0.1, 0.4, 1.0],
            ),
            esticadas(40, 40.0, 220.0, 22),
        ),
        (
            "circulo tracejado esticado, pontas redondas",
            forma(
                circ,
                None,
                Stroke::new(0.08)
                    .with_caps(Cap::Round)
                    .with_dashes(0.0, [0.15, 0.1]),
                [0.9, 0.2, 0.1, 1.0],
            ),
            esticadas(40, 30.0, 200.0, 23),
        ),
        // As pontas do INÍCIO e do FIM de cada traço DIFERENTES (o kurbo põe a `start_cap` e a
        // `end_cap` em cada traço) — sem esta família a troca das duas passava (mutação T8).
        // ⚠️ O fim QUADRADO e não rente: um traço que acaba a menos de meia largura depois de uma
        // quina sai do traçador da casa com uma MORDIDA no lado de dentro (a junta interior passa
        // pelo pivô e o pedaço curto cruza-se; a régua EXACTA — o kurbo a expandir, o Vello só a
        // preencher — tem a mesma mordida), e o passe desenha a união verdadeira: `140` de alfa num
        // pixel (medido 02/10, uma cópia, pedaço de `9,75 px` com raio `11,4`). Divergência
        // DECLARADA (doc 121 §9.9); a ponta quadrada cobre-a e a família continua a distinguir as
        // duas pontas.
        (
            "zigue-zague tracejado esticado, pontas diferentes",
            forma(
                zz,
                Some(zz),
                Stroke::new(0.07)
                    .with_join(Join::Bevel)
                    .with_start_cap(Cap::Round)
                    .with_end_cap(Cap::Square)
                    .with_dashes(0.0, [0.14, 0.1]),
                [0.2, 0.2, 0.7, 1.0],
            ),
            esticadas(40, 40.0, 220.0, 27),
        ),
        (
            "zigue-zague tracejado esticado, chanfro e pontas quadradas",
            forma(
                zz,
                Some(zz),
                Stroke::new(0.07)
                    .with_join(Join::Bevel)
                    .with_caps(Cap::Square)
                    .with_dashes(0.0, [0.1, 0.06]),
                [0.1, 0.5, 0.2, 1.0],
            ),
            esticadas(40, 40.0, 220.0, 24),
        ),
        // ⭐ A EMENDA: desde o ajuste no ecrã o contorno mais longo nunca emenda (a folga põe o fim no
        // último vão); o de DENTRO emenda conforme o esticão — pontas quadradas e esquadria, para a
        // emenda (junta) e a falta dela (duas pontas) desenharem diferente.
        (
            "estrela com furo tracejada esticada",
            forma(
                furo,
                None,
                Stroke::new(0.05)
                    .with_join(Join::Miter)
                    .with_caps(Cap::Square)
                    .with_dashes(0.0, [0.17, 0.09]),
                [0.5, 0.1, 0.1, 1.0],
            ),
            esticadas(40, 40.0, 220.0, 28),
        ),
        // CONTROLO: as mesmas estrelas CONFORMES vão pelo contorno pré-expandido (o do Vello ao bit).
        (
            "estrela tracejada conforme",
            forma(
                est,
                None,
                Stroke::new(0.06)
                    .with_join(Join::Miter)
                    .with_dashes(0.0, [0.12, 0.08]),
                [0.1, 0.1, 0.1, 1.0],
            ),
            copias(40, 40.0, 220.0, 25),
        ),
    ]
}

/// ⭐⭐ **O passe traceja como a casa** — pixel a pixel contra o Vello.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn o_tracejado_esticado_desenha_o_que_o_vello_desenha() {
    let (est, circ, zz, furo) = (estrela(), circulo(), zigue_zague(), estrela_com_furo());
    // Todas as famílias medidas antes de reprovar — o placar inteiro é o que decide uma barra.
    let mut falhas = Vec::new();
    for (nome, forma, cs) in &casos(&est, &circ, &zz, &furo) {
        let Some(d) = corre(nome, forma, cs) else {
            eprintln!("sem adaptador — nada a medir");
            return;
        };
        assert!(
            d.pixels_com_tinta > 1000,
            "{nome}: a fixtura quase nao desenha"
        );
        eprintln!(
            "  PLACAR {nome}: alfa {} · cor {} · {} px acima de 1",
            d.alfa_max, d.cor_max, d.alfa_acima_de_1
        );
        if d.alfa_max > BARRA || d.cor_max > BARRA {
            falhas.push(format!("{nome}: {d:?}"));
        }
    }
    assert!(
        falhas.is_empty(),
        "o passe traceja outra coisa que a casa: {falhas:#?}"
    );
}

/// ⭐⭐ **O contorno calculado traceja o que o caminho pixel a pixel traceja** — uma régua de PAR: as
/// duas rotas são a mesma lei, e o que sobra é arredondamento. E todas as cópias têm de ter ganho o
/// contorno (senão a imagem igual não prova que ele correu).
///
/// ⭐ doc 121 §9.14 (a) — pelas DUAS escritas: um fio por cópia e um GRUPO por cópia (a porta
/// `itens_do_grupo` manda todas as cópias tracejadas a uma e a outra). As duas desenham a mesma imagem,
/// logo o gate lê de volta quantas foram pelo grupo — todas as esticadas num caso, nenhuma no outro.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn o_tracejado_calculado_desenha_o_que_o_pixel_desenha() {
    let Some(gpu) = gpu() else {
        eprintln!("sem adaptador — nada a medir");
        return;
    };
    let (est, circ, zz, furo) = (estrela(), circulo(), zigue_zague(), estrela_com_furo());
    let mut falhas = Vec::new();
    for (nome, forma, cs) in &casos(&est, &circ, &zz, &furo) {
        let fmt = wgpu::TextureFormat::Rgba16Float;
        let n = u32::try_from(cs.len()).expect("cabem");
        let (pixel, sem) = pelo_passe_com(&gpu, forma, cs, fmt, false, 1)
            .pop()
            .expect("um quadro");
        assert_eq!(sem, 0, "{nome}: CONTROLO — desligado, nenhuma o ganha");
        let conforme = nome.contains("conforme");
        for (escrita, itens) in [("grupo", 0), ("fio", u32::MAX)] {
            let mut grupo = 0;
            let (calc, com, _) = pelo_passe_observado(
                &gpu,
                forma,
                &[(cs, 4)],
                fmt,
                (true, 0.0, u64::MAX, itens),
                &mut |g, p| grupo = p.copias_do_grupo_do_ultimo_quadro(g),
            )
            .pop()
            .expect("um quadro");
            let (mut pior, mut acima, mut tinta) = (0u8, 0usize, 0usize);
            for (x, y) in calc.as_chunks::<4>().0.iter().zip(pixel.as_chunks::<4>().0) {
                let d = x[3].abs_diff(y[3]);
                pior = pior.max(d);
                acima += usize::from(d > 1);
                tinta += usize::from(x[3] > 0 || y[3] > 0);
            }
            eprintln!(
                "  PAR {nome} ({escrita}): alfa max {pior} · {acima} px > 1 · {tinta} px · {com}/{n} com contorno · {grupo} pelo grupo"
            );
            assert!(tinta > 1000, "{nome}: a fixtura quase nao desenha");
            // A conforme não tem eixo: nunca vai ao grupo (o controlo da porta).
            let esperado = if itens == 0 && !conforme { n } else { 0 };
            if com != n || pior > 2 || grupo != esperado {
                falhas.push(format!(
                    "{nome} ({escrita}): alfa {pior}, {acima} px, {com}/{n} com contorno, {grupo}/{esperado} pelo grupo"
                ));
            }
        }
    }
    assert!(
        falhas.is_empty(),
        "as escritas tracejam diferente do pixel a pixel: {falhas:#?}"
    );
}

/// ⭐ doc 121 §9.10 — **só um eixo com troço TRACEJADO pede a variante COMPLETA.** Inline, o ramo do
/// tracejado dobrava os registos do fragmento e do `cs_escreve` em TODA a cena (iGPU `56 → 128`
/// VGPRs, as estrelas esticadas `1,74 → 2,45 ms`). Os gates de pixel só provam que a cena tracejada
/// escolhe a completa: escolhê-la sempre desenha a mesma imagem, mais devagar — esta régua é a que vê.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn so_um_eixo_tracejado_pede_a_variante_completa() {
    let Some(gpu) = gpu() else {
        eprintln!("sem adaptador — nada a medir");
        return;
    };
    let est = estrela();
    let prepara = |s: &Stroke| {
        ShapeGeometry::prepare(&ShapeInput {
            fill: Some((&est, FillRule::NonZero)),
            strokes: vec![StrokeInput {
                path: &est,
                style: s,
                color: [0.1, 0.1, 0.1, 1.0],
            }],
            stroke_fills: vec![],
        })
        .expect("a forma prepara")
    };
    let continua = prepara(&Stroke::new(0.06));
    let tracejada = prepara(&Stroke::new(0.06).with_dashes(0.0, [0.12, 0.08]));
    // CONTROLO: a contínua TEM eixo — sem ele, a enxuta seria escolhida por não haver nada a percorrer.
    assert!(!continua.eixo.is_empty() && !continua.eixo.iter().any(EixoItem::tracejado));
    assert!(tracejada.eixo.iter().any(EixoItem::tracejado));
    let mut p = ShapePass::new(&gpu, wgpu::TextureFormat::Rgba16Float);
    p.set_geometries(&gpu, [(1u32, &continua)]);
    assert!(!p.usa_o_tracejado(), "sem tracejado, a variante ENXUTA");
    p.set_geometries(&gpu, [(1u32, &continua), (2u32, &tracejada)]);
    assert!(
        p.usa_o_tracejado(),
        "uma geometria tracejada no conjunto pede a COMPLETA"
    );
    p.set_geometries(&gpu, [(1u32, &continua)]);
    assert!(!p.usa_o_tracejado(), "sem ela, volta à ENXUTA");
}

/// ⭐ doc 121 §9.13 — **numa cena tracejada a PLACA escolhe a variante do desenho por quadro**: a
/// completa (o fragmento a `128` VGPRs na iGPU) só quando uma cópia tracejada vai pixel a pixel — o 1.º
/// quadro, antes da capacidade medida, e as cópias que não cabem nas células; no regime, a ENXUTA. Os
/// gates de pixel não vêem «completa sempre» (a mesma imagem, mais devagar): esta régua é a que vê. E
/// a metade das células desenha o MESMO que todas (a completa a tracejar as que ficaram de fora).
#[test]
#[ignore = "precisa de adapter de GPU"]
fn a_placa_escolhe_a_variante_completa_so_quando_um_tracejado_vai_pixel_a_pixel() {
    let Some(gpu) = gpu() else {
        eprintln!("sem adaptador — nada a medir");
        return;
    };
    let (est, circ, zz, furo) = (estrela(), circulo(), zigue_zague(), estrela_com_furo());
    let todos = casos(&est, &circ, &zz, &furo);
    let (nome, forma, cs) = &todos[0];
    let n = u32::try_from(cs.len()).expect("cabem");
    let fmt = wgpu::TextureFormat::Rgba16Float;
    let corre = |celulas_no_maximo: u64| {
        let mut por_quadro = Vec::new();
        let quadros = pelo_passe_observado(
            &gpu,
            forma,
            &[(cs.as_slice(), 4)],
            fmt,
            (true, 0.0, celulas_no_maximo, ph2d_shape_gpu::ITENS_DO_GRUPO),
            &mut |g, p| por_quadro.push(p.copias_por_variante(g)),
        );
        (quadros, por_quadro)
    };
    let (livre, v_livre) = corre(u64::MAX);
    eprintln!("  {nome}: (enxuta, completa) por quadro {v_livre:?}");
    // CONTROLO: o 1.º quadro, sem capacidade, desenha tudo pixel a pixel — pela completa.
    assert_eq!(v_livre[0], (0, n), "{nome}: o 1.º quadro pede a COMPLETA");
    let (_, com, _) = livre.last().expect("quadros");
    assert_eq!(*com, n, "{nome}: no regime toda cópia tem contorno");
    assert_eq!(
        *v_livre.last().expect("quadros"),
        (n, 0),
        "{nome}: no regime, a ENXUTA"
    );
    let (_, _, (pedido, _)) = livre.last().expect("quadros");
    let (metade, v_metade) = corre(pedido / 2);
    let (img_metade, com_metade, _) = metade.last().expect("quadros");
    assert!(
        *com_metade > 0 && *com_metade < n,
        "{nome}: CONTROLO — o tecto morde e alguma cópia continua nas células ({com_metade}/{n})"
    );
    assert_eq!(
        *v_metade.last().expect("quadros"),
        (0, n),
        "{nome}: uma cópia tracejada sem células pede a COMPLETA"
    );
    let pior = livre
        .last()
        .expect("quadros")
        .0
        .as_chunks::<4>()
        .0
        .iter()
        .zip(img_metade.as_chunks::<4>().0)
        .map(|(a, b)| a[3].abs_diff(b[3]))
        .max()
        .unwrap_or(0);
    assert!(
        pior <= 2,
        "{nome}: metade das células desenha outra coisa (alfa {pior})"
    );
}

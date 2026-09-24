//! Os gates do KERNEL do `motion.duplicator` (ciclo 10 W1b / ciclo 12) — **o que o hospedeiro
//! decide**.
//!
//! ⚠️ **O que ELES não medem, e quem mede:** o módulo WGSL gerado é parseado e validado pelo naga
//! no `every_registered_kernel_validates_across_the_whole_presence_space` da `ph2d-gpu-cook` (toda
//! a grelha de presença, **sem placa**), e a igualdade das duas rotas é a paridade GPU↔CPU na
//! bancada daquela crate. Aqui prova-se a metade que corre em Rust: a lei de contagem, o uniforme
//! derivado, as recusas e a forma das ligações.

use super::{DERIVADOS, GPU_KERNEL, NP, TOTAL};
use crate::{MAX_INSTANCIAS_POR_NO, POINT_SCALE, Pick, duplicate, points_within_budget, transfer};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::gpu::{ColumnAccess, CountLawCtx, ROWS_COL};

/// Os params do manifesto, resolvidos como o `eval` os resolve (default, com sobreposições).
fn params(over: Vec<(&'static str, f32)>) -> impl Fn(&str) -> f32 {
    move |nome: &str| {
        over.iter()
            .find(|(n, _)| *n == nome)
            .map(|(_, v)| *v)
            .or_else(|| crate::MANIFEST.param_default(nome))
            .unwrap_or(0.0)
    }
}

fn ctx<'a>(inputs: &'a [u32], p: &'a dyn Fn(&str) -> f32) -> CountLawCtx<'a> {
    CountLawCtx {
        inputs,
        param: p,
        playhead: 0.0,
        dt: 0.0,
    }
}

fn nuvem(n: usize) -> Stream {
    Stream::new(n).with("P", Column::Vec2(vec![[0.0, 0.0]; n]))
}

fn derivado(param: &str, c: &CountLawCtx<'_>) -> f32 {
    let d = DERIVADOS
        .iter()
        .find(|d| d.param == param)
        .unwrap_or_else(|| panic!("o kernel declara o uniforme derivado `{param}`"));
    (d.derive)(c)
}

/// ⭐⭐⭐ **A LEI DE CONTAGEM É A MESMA EXPRESSÃO DO `eval`** — sobre formas e pontos, incluindo o
/// corte do orçamento e os dois degenerados (sem formas · sem pontos).
///
/// ⚠️ **Não é uma re-derivação:** o lado esperado é a saída de [`duplicate`], a função que o `eval`
/// chama, com o `np` que o `eval` lhe dá. *Duas leis de contagem que discordam não falham —
/// desenham um número diferente de coisas.*
#[test]
fn a_lei_de_contagem_e_a_do_eval() {
    let p = params(vec![]);
    for ns in [0usize, 1, 3] {
        for np_pedidos in [0usize, 1, 7, 40_000] {
            let entradas = [
                u32::try_from(ns).unwrap(),
                u32::try_from(np_pedidos).unwrap(),
            ];
            let lei = (GPU_KERNEL.count_law.expect("o carimbo declara a lei"))(&ctx(&entradas, &p));
            let np = points_within_budget(Pick::Off, ns, np_pedidos, MAX_INSTANCIAS_POR_NO);
            let cpu = duplicate(
                &nuvem(ns),
                &nuvem(np_pedidos),
                np,
                Pick::Off,
                0,
                0.0,
                transfer::Transfer::ShapeWins,
            );
            assert_eq!(
                lei.count,
                cpu.count(),
                "ns = {ns}, np = {np_pedidos}: a lei do kernel tem de dar a contagem do `eval`"
            );
            assert_eq!(
                derivado(TOTAL, &ctx(&entradas, &p)) as usize,
                cpu.count(),
                "o `total` derivado é a `Count` que a CPU escreve"
            );
        }
    }
}

/// ⭐⭐ **O `np` DERIVADO É O DO ORÇAMENTO** — o mesmo que o `eval` passa ao `duplicate`, e é por
/// ele que o corpo divide. O caso que o torna necessário: `3` formas × `40 000` pontos passa o
/// tecto, e o `np` cortado é `⌊32 768 / 3⌋`, não os `40 000` pedidos.
#[test]
fn o_np_derivado_e_o_do_orcamento() {
    let p = params(vec![]);
    for (ns, np_pedidos) in [(1usize, 5usize), (3, 40_000), (0, 9), (2, 0)] {
        let entradas = [
            u32::try_from(ns).unwrap(),
            u32::try_from(np_pedidos).unwrap(),
        ];
        let esperado = points_within_budget(Pick::Off, ns, np_pedidos, MAX_INSTANCIAS_POR_NO);
        assert_eq!(
            derivado(NP, &ctx(&entradas, &p)) as usize,
            esperado,
            "ns = {ns}, np = {np_pedidos}"
        );
    }
    // O controlo: o corte existe mesmo na faixa medida (senão o gate não exercita o orçamento).
    assert!(
        points_within_budget(Pick::Off, 3, 40_000, MAX_INSTANCIAS_POR_NO) < 40_000,
        "a fixtura tem de passar o tecto"
    );
}

/// ⭐ **AS TRÊS RECUSAS são as do cabeçalho, e o modo de fábrica é reclamado.** Com o controlo: os
/// valores de fábrica (`pick 0`, `transfer 0`, `point_scale 0`) reclamam.
#[test]
fn as_tres_recusas_entregam_o_no_a_cpu() {
    let ap = GPU_KERNEL.applicable.expect("o carimbo declara as recusas");
    assert!(
        ap(&params(vec![])),
        "o modo de fábrica é reclamado pelo dispositivo"
    );
    for (param, v) in [
        ("pick", 1.0f32),
        ("pick", 2.0),
        (transfer::TRANSFER, 1.0),
        (transfer::TRANSFER, 3.0),
        (POINT_SCALE, 0.5),
        (POINT_SCALE, f32::NAN),
    ] {
        assert!(
            !ap(&params(vec![(param, v)])),
            "`{param} = {v}` tem de ir à CPU"
        );
    }
}

/// ⭐ **A FORMA DAS LIGAÇÕES** — as duas leituras de `P` são das DUAS portas, a `rot` dos pontos é
/// uma recusa e não uma leitura, a da forma escreve só se existe, e `Index`/`Count` são escritas
/// SEMPRE (a CPU cunha-as).
#[test]
fn as_ligacoes_sao_as_da_lei_da_cpu() {
    let b = GPU_KERNEL.bindings;
    let tem = |col: &str, acesso: ColumnAccess, porta: usize| {
        b.iter()
            .any(|x| x.column == col && x.access == acesso && x.port == porta)
    };
    assert!(tem("P", ColumnAccess::SourceRead, 0), "o P da forma");
    assert!(tem("P", ColumnAccess::SourceRead, 1), "o P do ponto");
    assert!(tem("P", ColumnAccess::Write, 0), "o P da saída");
    assert!(tem("rot", ColumnAccess::SourceReadWriteExisting, 0));
    assert!(tem("rot", ColumnAccess::RefuseIfPresent, 1));
    assert!(tem(ROWS_COL, ColumnAccess::Write, 0));
    assert!(tem("Index", ColumnAccess::Write, 0));
    assert!(tem("Count", ColumnAccess::Write, 0));
    // Nenhuma outra coluna dos pontos é tocada: o `Shape Wins` deita-as fora.
    assert!(
        b.iter()
            .filter(|x| x.port == 1)
            .all(|x| x.column == "P" || x.column == "rot"),
        "da porta dos pontos só viajam P (e a recusa da rot)"
    );
}

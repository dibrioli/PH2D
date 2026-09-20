//! Os gates do KERNEL do `motion.clone` (ciclo 10 W1a) — **o que o hospedeiro decide**.
//!
//! ⚠️ **O que ELES não medem, e quem mede:** o módulo WGSL gerado é parseado e validado pelo naga
//! no `generated_wgsl_validates` da `ph2d-gpu-cook` (toda a grelha de presença, **sem placa**), e a
//! igualdade das duas rotas é a paridade GPU↔CPU na bancada daquela crate. Aqui prova-se a metade
//! que corre em Rust: a lei de contagem, os uniformes derivados, as recusas e a forma das ligações.

use super::{DERIVADOS, GPU_KERNEL, K, STEP_X, STEP_Y};
use crate::{MODE, ROT_TAPER, SCALE_TAPER, clone_row, copy_rank, fan, radial};
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

/// ⭐⭐⭐ **A LEI DE CONTAGEM É A MESMA EXPRESSÃO DO `eval`** — sobre a faixa inteira do knob,
/// incluindo o corte do orçamento.
///
/// ⚠️ **Não é uma re-derivação:** o lado esperado é a saída de [`clone_row`], a função que o `eval`
/// chama. *Duas leis de contagem que discordam não falham — desenham um número diferente de coisas.*
#[test]
fn a_lei_de_contagem_e_a_do_eval() {
    for n in [0usize, 1, 7, 1024] {
        for pedidas in [1.0f32, 3.0, 32.0, 10_000.0] {
            let p = params(vec![("count", pedidas)]);
            let lei = (GPU_KERNEL.count_law.expect("o cloner declara a lei"))(&ctx(
                &[u32::try_from(n).unwrap()],
                &p,
            ));
            let k = derivados_do(K, &ctx(&[u32::try_from(n).unwrap()], &p)) as usize;
            let cpu = clone_row(&nuvem(n), k, 0.0, 0.0, false, 1.0, 0.0);
            assert_eq!(
                lei.count,
                cpu.count(),
                "n = {n}, count = {pedidas}: a lei do kernel tem de dar a contagem do `eval`"
            );
        }
    }
}

fn derivados_do(param: &str, c: &CountLawCtx<'_>) -> f32 {
    let d = DERIVADOS
        .iter()
        .find(|d| d.param == param)
        .unwrap_or_else(|| panic!("o kernel declara o uniforme derivado `{param}`"));
    (d.derive)(c)
}

/// ⭐⭐ **O PASSO DERIVADO É O DA CPU, AO BIT — e o produto `posto × passo` também.**
///
/// ⛔ É isto que tira a trigonometria do dispositivo (HR-5): a seno parabólica da casa corre **uma
/// vez no hospedeiro**, e o corpo multiplica o mesmo `f32`.
#[test]
fn o_passo_derivado_e_o_da_cpu_ao_bit() {
    for (angulo, distancia) in [(0.0f32, 2.0f32), (37.0, 1.5), (-110.0, 0.25), (450.0, 3.0)] {
        let p = params(vec![("angle", angulo), ("distance", distancia)]);
        let c = ctx(&[4], &p);
        let (dx, dy) = radial::linear_step(angulo, distancia);
        assert_eq!(derivados_do(STEP_X, &c), dx, "angulo {angulo}");
        assert_eq!(derivados_do(STEP_Y, &c), dy, "angulo {angulo}");
        // E o que o corpo calcula (`p + posto · passo`) é o que a `Placement` da CPU aplica.
        for copia in 0..5usize {
            let posto = copy_rank(copia, 5, false);
            let esperado = radial::Placement::of(false, posto, 0.0, angulo, distancia, [0.0, 0.0])
                .apply([0.25, -0.5]);
            assert_eq!(
                esperado,
                [0.25 + posto * dx, -0.5 + posto * dy],
                "copia {copia}"
            );
        }
    }
}

/// ⭐ **O `k` do uniforme é o do ORÇAMENTO**, o mesmo número que a lei de contagem usa.
#[test]
fn o_k_derivado_e_o_do_orcamento() {
    for (n, pedidas, esperado) in [(1u32, 3.0f32, 3.0f32), (0, 5.0, 5.0), (7, 1.0, 1.0)] {
        let p = params(vec![("count", pedidas)]);
        assert_eq!(
            derivados_do(K, &ctx(&[n], &p)),
            esperado,
            "n = {n}, count = {pedidas}"
        );
    }
    // O corte: um milhão de cópias sobre uma nuvem grande não cabe no orçamento.
    let p = params(vec![("count", 10_000.0)]);
    let k = derivados_do(K, &ctx(&[100_000], &p));
    assert!(
        k > 0.0 && k < 10_000.0,
        "o orçamento tem de cortar as cópias (leu {k})"
    );
}

/// ⭐⭐⭐ **AS QUATRO RECUSAS SÃO AS QUE A POPULAÇÃO MEDIU** (doc 116 §5.2/§5.3), **com o CONTROLO**:
/// nos valores de fábrica o kernel é aplicável.
///
/// ⚠️ *Sem a 1.ª linha este gate passaria com um `applicable` que recusa sempre.*
#[test]
fn as_quatro_recusas_sao_as_medidas() {
    let aplicavel = GPU_KERNEL.applicable.expect("o cloner declara recusas");
    assert!(
        aplicavel(&params(vec![])),
        "CONTROLO: nos valores de fábrica o dispositivo corre"
    );
    for (nome, over) in [
        ("o leque de relógios", [(fan::TIME_OFFSET, 0.5f32)]),
        ("o modo Radial", [(MODE, radial::MODE_RADIAL as f32)]),
        ("o taper de escala", [(SCALE_TAPER, 0.5)]),
        ("o taper de rotação", [(ROT_TAPER, 90.0)]),
    ] {
        assert!(
            !aplicavel(&params(over.to_vec())),
            "{nome}: o kernel tem de RECUAR para a CPU"
        );
    }
}

/// ⭐⭐⭐ **A RENUMERAÇÃO ENTRA PELA CRUZA, e é isso que a torna possível.**
///
/// ⛔ **As duas metades, e nenhuma das outras nove variantes as tem juntas:** um `SourceRead` lê na
/// fonte e **nunca escreve** (a renumeração evaporava), um `ReadWriteExisting` escreve só-se-presente
/// e **lê no índice do dispatch** (a porta template tem `n` linhas e o dispatch tem `n · k`, logo a
/// coluna seria julgada AUSENTE e a escrita descartada).
#[test]
fn a_renumeracao_entra_pela_cruza() {
    let acesso = |col: &str, i: usize| {
        GPU_KERNEL
            .bindings
            .iter()
            .filter(|b| b.column == col)
            .nth(i)
            .unwrap_or_else(|| panic!("o kernel liga `{col}`"))
            .access
    };
    for col in ["Index", "Count"] {
        let a = acesso(col, 0);
        assert!(a.is_source_mapped(), "{col}: a porta e' o TEMPLATE");
        assert!(a.writes(true), "{col}: escreve quando a entrada a traz");
        assert!(!a.writes(false), "{col}: NAO a cunha quando ela falta");
    }
    // ⛔ E as duas NÃO são o mesmo verbo: o `Index` soma o da fonte e LÊ; a `Count` vale `total`
    // e nunca lê — ligada como a cruza, o módulo declarava um buffer que ninguém chama, e o bind
    // group do sequenciador ficava com uma entrada a mais.
    assert_eq!(acesso("Index", 0), ColumnAccess::SourceReadWriteExisting);
    assert!(acesso("Index", 0).reads(), "o Index LE a fonte");
    assert_eq!(acesso("Count", 0), ColumnAccess::SourceWriteExisting);
    assert!(!acesso("Count", 0).reads(), "a Count NAO le");
    // O `P` é DUAS ligações (a leitura da fonte e a escrita da saída) e o `cp_rows` é a máquina.
    assert_eq!(acesso("P", 0), ColumnAccess::SourceRead);
    assert_eq!(acesso("P", 1), ColumnAccess::Write);
    assert_eq!(acesso(ROWS_COL, 0), ColumnAccess::Write);
}

/// ⚠️ **O corpo só chama acessores que as ligações declaram** — a metade que o naga confirma na
/// `ph2d-gpu-cook`, afirmada aqui onde ela é barata e sem placa.
#[test]
fn o_corpo_chama_o_que_as_ligacoes_declaram() {
    for agulha in [
        "read_P(cl_row)",
        "write_P(i,",
        "write_cp_rows(i,",
        "write_Index(i,",
        "write_Count(i,",
        "params.window_src_n",
        "params.cl_k",
        "params.cl_step_x",
        "params.cl_step_y",
        "params.center",
    ] {
        assert!(
            GPU_KERNEL.wgsl.contains(agulha),
            "o corpo tem de conter `{agulha}`"
        );
    }
    // E todo param que ele lê está declarado (o planeador recusa o kernel se não estiver).
    assert_eq!(GPU_KERNEL.params, &["center", K, STEP_X, STEP_Y]);
}

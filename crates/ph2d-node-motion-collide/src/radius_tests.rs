//! Gates do RAIO POR ELEMENTO e do `falloff` (doc 89, folha 03, linhas 60 e 62).
//!
//! Irmão de `tests.rs` por assunto, não por tamanho: aquele mede *o que a
//! relaxação FAZ* (empacota, é independente de ordem, honra o pino); este mede
//! *de que TAMANHO é cada disco e quanto o nó age sobre ele* — as duas entradas
//! que o nó passou a ler.
//!
//! Filho por `#[path]`, então `use super::*` alcança `push_apart`,
//! `radius_scale` e `falloff_col`, que são privados de propósito.

use super::*;

fn dist(a: [f32; 2], b: [f32; 2]) -> f32 {
    let (dx, dy) = (a[0] - b[0], a[1] - b[1]);
    (dx * dx + dy * dy).sqrt()
}

/// Um stream de pontos com uma coluna opcional.
fn dots(p: &[[f32; 2]]) -> Stream {
    Stream::new(p.len()).with("P", Column::Vec2(p.to_vec()))
}

// ─────────────────────────── o raio por elemento ───────────────────────────

/// **A COLUNA `size` AUSENTE É `1` EM TODO ELEMENTO — o mundo que já shipava.**
///
/// ⚠️ Este é o gate que torna a wave inteira segura de integrar: `radius · 1.0` é
/// `radius` exato e `r + r` é `2.0 * radius` exato em IEEE-754, então toda cena
/// sem `size` empacota **byte a byte** como empacotava. O oráculo é a comparação
/// das DUAS rotas (ausente × explicitamente unitária), porque é ali que um
/// `radius_scale` com fallback errado apareceria.
#[test]
fn a_stream_without_a_size_column_packs_exactly_as_a_unit_sized_one() {
    let p = [[0.0, 0.0], [0.1, 0.0], [0.05, 0.09], [-0.2, 0.15]];
    let bare = dots(&p);
    let mut unit = dots(&p);
    unit.set("size", Column::Vec2(vec![[1.0, 1.0]; p.len()]));

    let a = radius_scale(&bare, p.len());
    let b = radius_scale(&unit, p.len());
    assert_eq!(a, vec![1.0; p.len()], "ausente e' a identidade");
    assert_eq!(a, b, "ausente e unitaria dao a MESMA escala, bit a bit");

    let w = vec![1.0; p.len()];
    let fall = vec![1.0; p.len()];
    let radii: Vec<f32> = a.iter().map(|s| 0.3 * s).collect();
    let out = push_apart(&p, &w, &radii, &fall, 8, 1.0);
    // E o número que ela reproduz é o `2·radius` da lei antiga.
    let uniform = push_apart(&p, &w, &vec![0.3; p.len()], &fall, 8, 1.0);
    assert_eq!(
        out, uniform,
        "a rota do tamanho reduz LITERALMENTE a de antes"
    );
}

/// **UM DISCO GRANDE E UM PEQUENO ASSENTAM EM `r_i + r_j`** — a lei, e ela é
/// simétrica.
///
/// ⚠️ O oráculo nomeia as DUAS respostas erradas: `2·r_pequeno` (o nó ignorando o
/// tamanho, que é o mundo de ontem) e `2·r_grande` (o nó lendo só o máximo). A
/// soma fica exatamente no meio delas, então nenhuma das duas passa por acidente.
#[test]
fn a_big_disc_and_a_small_one_settle_at_the_sum_of_their_radii() {
    let p = [[0.0, 0.0], [0.1, 0.0]];
    let w = [1.0, 1.0];
    let fall = [1.0, 1.0];
    // base 0.3, escalas 1 e 2 ⇒ raios 0.3 e 0.6.
    let radii = [0.3, 0.6];
    let out = push_apart(&p, &w, &radii, &fall, 64, 1.0);
    let d = dist(out[0], out[1]);
    assert!(
        (d - 0.9).abs() < 1e-3,
        "a lei e' r_i + r_j = 0.9; nem 0.6 (so' o pequeno) nem 1.2 (so' o grande). Medido {d}"
    );

    // Simétrica: trocar quem é grande espelha o resultado, não o muda.
    let swapped = push_apart(&p, &w, &[0.6, 0.3], &fall, 64, 1.0);
    let d2 = dist(swapped[0], swapped[1]);
    assert!((d - d2).abs() < 1e-4, "a lei e' simetrica: {d} vs {d2}");
}

/// **O DISCO CONTÉM A ARTE:** um `size` não-uniforme lê pelo maior eixo.
///
/// ⚠️ É a decisão que o doc de [`radius_scale`] argumenta, e ela tem consequência
/// visível: com `min` ou com a média, as pontas de uma instância larga invadiriam
/// a vizinha — exatamente o que este nó existe para impedir.
#[test]
fn a_wide_instance_is_measured_by_the_axis_that_sticks_out() {
    let mut s = dots(&[[0.0, 0.0]]);
    s.set("size", Column::Vec2(vec![[2.0, 1.0]]));
    assert_eq!(radius_scale(&s, 1), vec![2.0], "o disco CONTEM a arte");
}

/// **UMA INSTÂNCIA ESPELHADA TEM O MESMO TAMANHO.**
///
/// ⚠️ Uma extensão não tem sinal — contraste deliberado com o `offset` do collider
/// da física, onde o sinal É a lateralidade porque ali o número é uma POSIÇÃO.
/// Sem o `abs` um `scale.x = -1` daria raio negativo e o par seria pulado: a
/// instância espelhada atravessaria as vizinhas em silêncio.
#[test]
fn a_mirrored_instance_is_the_same_size() {
    let mut s = dots(&[[0.0, 0.0], [1.0, 0.0]]);
    s.set("size", Column::Vec2(vec![[-2.0, 1.0], [2.0, -1.0]]));
    assert_eq!(radius_scale(&s, 2), vec![2.0, 2.0]);
}

/// **UM TAMANHO ENVENENADO LÊ COMO A IDENTIDADE**, nunca como um raio infinito.
///
/// Um `NaN` propagado de um documento editado à mão faria `min_dist` ser `NaN`,
/// toda comparação falsa, e o nó **pararia de empacotar em silêncio**; um `inf`
/// engoliria a cena inteira num par só.
#[test]
fn a_non_finite_size_reads_as_the_identity() {
    let mut s = dots(&[[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]]);
    s.set(
        "size",
        Column::Vec2(vec![
            [f32::NAN, 1.0],
            [f32::INFINITY, 1.0],
            [3.0, f32::NEG_INFINITY],
        ]),
    );
    assert_eq!(radius_scale(&s, 3), vec![1.0, 1.0, 1.0]);
}

// ⚠️ **NÃO há gate de "coluna de comprimento errado", e a ausência é medida:**
// `Stream::set` tem um `assert_eq!` do comprimento contra a contagem do stream,
// então o estado é INCONSTRUÍVEL — a fixture panica em `attr.rs:176` antes de o
// nó ver coisa alguma. A guarda `v.len() == n` do `radius_scale` fica por
// simetria com o [`inv_mass`] irmão, que a carrega pelo mesmo motivo, e um gate
// aqui não poderia falhar pela razão que alegaria.

/// **SEM RAIO EM LUGAR NENHUM, A ENTRADA VOLTA INTOCADA** — a identidade precoce
/// que o `2·radius <= 0` fazia, agora expressa sobre o MAIOR disco do conjunto.
#[test]
fn a_set_of_zero_sized_discs_is_returned_untouched() {
    let p = [[0.0, 0.0], [0.01, 0.0]];
    let out = push_apart(&p, &[1.0, 1.0], &[0.0, 0.0], &[1.0, 1.0], 8, 1.0);
    assert_eq!(out, p);
}

// ────────────────────────────── o falloff ──────────────────────────────

/// **`falloff = 0` TORNA O DISCO TRANSPARENTE: ele não é empurrado E NÃO EMPURRA.**
///
/// ⚠️ Esta é a distinção inteira da linha 62 da folha, e ela precisa das duas
/// metades: *pinar* (`inv_mass = 0`) deixa o disco imóvel **e obstáculo** — o
/// vizinho leva a penetração toda —, enquanto *mutar* (`falloff = 0`) tira o par
/// da simulação. Um gate que só medisse "o mudo não se mexe" seria satisfeito
/// pelo pino, que é a outra coisa.
#[test]
fn a_falloff_of_zero_makes_the_disc_transparent_which_is_not_being_pinned() {
    let p = [[0.0, 0.0], [0.1, 0.0]];
    let radii = [0.3, 0.3];

    // MUDO: o par desaparece — ninguém se mexe.
    let muted = push_apart(&p, &[1.0, 1.0], &radii, &[0.0, 1.0], 8, 1.0);
    assert_eq!(muted, p, "um disco mudo e' transparente: o par nao existe");

    // PINADO: o par existe, o livre leva a penetração inteira.
    let pinned = push_apart(&p, &[0.0, 1.0], &radii, &[1.0, 1.0], 8, 1.0);
    assert_eq!(pinned[0], p[0], "o pino nao se move");
    assert!(
        dist(pinned[0], pinned[1]) > 0.5,
        "e o livre e' empurrado para fora dele: {}",
        dist(pinned[0], pinned[1])
    );
}

/// **`falloff = 1` EM TODO ELEMENTO É O MUNDO QUE JÁ SHIPAVA**, bit a bit — e a
/// coluna ausente dá a mesma resposta.
#[test]
fn a_full_falloff_is_byte_identical_to_no_falloff_column() {
    let p = [[0.0, 0.0], [0.1, 0.0], [0.05, 0.09], [-0.2, 0.15]];
    let n = p.len();
    let bare = dots(&p);
    let mut full = dots(&p);
    full.set("falloff", Column::Scalar(vec![1.0; n]));
    assert_eq!(falloff_col(&bare, n), vec![1.0; n]);
    assert_eq!(falloff_col(&bare, n), falloff_col(&full, n));

    let w = vec![1.0; n];
    let radii = vec![0.3; n];
    let a = push_apart(&p, &w, &radii, &falloff_col(&bare, n), 8, 1.0);
    let b = push_apart(&p, &w, &radii, &falloff_col(&full, n), 8, 1.0);
    assert_eq!(a, b);
}

/// **UM PAR MUDO NÃO DILUI OS CONTATOS REAIS** — a razão de a média ser PONDERADA
/// em vez de uma contagem.
///
/// ⚠️ Sem o peso na tally, um disco cercado de vizinhos mudos dividiria a correção
/// que o único vizinho VIVO pediu pelo número de vizinhos, e a separação sairia
/// uma fração do que devia. A fixture põe um disco entre um vivo e dois mudos, e
/// o oráculo é ele assentar **onde assentaria se os mudos não existissem**.
#[test]
fn a_muted_pair_does_not_dilute_the_contacts_that_are_alive() {
    // 0 é o disco medido; 1 é o vizinho VIVO; 2 e 3 são mudos e sobrepostos a 0.
    let p = [[0.0, 0.0], [0.1, 0.0], [0.0, 0.05], [0.0, -0.05]];
    let w = [1.0; 4];
    let radii = [0.3; 4];
    let with_mutes = push_apart(&p, &w, &radii, &[1.0, 1.0, 0.0, 0.0], 8, 1.0);

    // O mesmo par, sozinho no mundo.
    let alone = push_apart(&p[..2], &[1.0, 1.0], &[0.3, 0.3], &[1.0, 1.0], 8, 1.0);

    let d_mixed = dist(with_mutes[0], with_mutes[1]);
    let d_alone = dist(alone[0], alone[1]);
    assert!(
        (d_mixed - d_alone).abs() < 1e-4,
        "os mudos nao podem diluir o contato vivo: {d_mixed} contra {d_alone}"
    );
    // E o CONTROLE: os mudos ficam exatamente onde estavam.
    assert_eq!(with_mutes[2], p[2]);
    assert_eq!(with_mutes[3], p[3]);
}

/// **O PESO DO PAR É SIMÉTRICO** — trocar quem carrega o `falloff` parcial não
/// muda a força do par (o produto comuta), então a lei não depende da ordem em
/// que o stream foi listado.
#[test]
fn the_pair_weight_does_not_care_which_side_carries_it() {
    let p = [[0.0, 0.0], [0.1, 0.0]];
    let a = push_apart(&p, &[1.0, 1.0], &[0.3, 0.3], &[0.5, 1.0], 8, 1.0);
    let b = push_apart(&p, &[1.0, 1.0], &[0.3, 0.3], &[1.0, 0.5], 8, 1.0);
    assert!(
        (dist(a[0], a[1]) - dist(b[0], b[1])).abs() < 1e-6,
        "o produto comuta"
    );
}

/// **UM `falloff` PARCIAL SEPARA PARCIALMENTE** — monotônico, sem degrau: meio
/// peso separa menos que peso cheio e mais que peso nenhum.
///
/// ⚠️ **É este gate que prova que o knob NÃO é decorativo, e ele nasceu VERMELHO**
/// (`0.1 < 0.6 < 0.6`): a primeira versão da lei normalizava por uma soma de
/// PESOS, e num par isolado o peso aparecia no numerador e no divisor — ele
/// **CANCELAVA**, e `falloff = 0.5` desenhava exatamente o mesmo que `1.0`. A
/// tally passou a ser uma CONTAGEM, e a medição vive no `push_apart`.
///
/// UMA varredura, de propósito: com as oito do default a relaxação converge e os
/// dois pesos chegam ao mesmo lugar — *um gate de MAGNITUDE tem de medir antes de
/// o solver saturar*.
#[test]
fn a_partial_falloff_separates_partially() {
    let p = [[0.0, 0.0], [0.1, 0.0]];
    let g = |f: f32| {
        let out = push_apart(&p, &[1.0, 1.0], &[0.3, 0.3], &[f, f], 1, 1.0);
        dist(out[0], out[1])
    };
    let (zero, half, full) = (g(0.0), g(0.5), g(1.0));
    assert!(
        zero < half && half < full,
        "monotonico e ESTRITO: {zero} < {half} < {full}"
    );
}

/// **UM `falloff` FORA DA FAIXA É CLAMPADO, NUNCA INVERTIDO** — um documento
/// editado à mão com `-1` puxaria os discos PARA DENTRO um do outro.
#[test]
fn an_out_of_range_falloff_is_clamped_never_inverted() {
    let mut s = dots(&[[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]]);
    s.set("falloff", Column::Scalar(vec![-1.0, 2.0, 0.5]));
    assert_eq!(falloff_col(&s, 3), vec![0.0, 1.0, 0.5]);
}

// ───────────────────── a costura com o device (dados puros) ─────────────────

/// **A REDUÇÃO E A BINDING CONCORDAM SOBRE O QUE UMA COLUNA AUSENTE VALE.**
///
/// ⚠️ O doc do `ReduceSpec::identity` é explícito: *"o valor que uma coluna
/// AUSENTE lê, por elemento — a mesma identidade que a `ColumnBinding` declara, e
/// tem de concordar com ela"*. Se divergissem, um stream sem `size` reduziria a
/// um `r_max` que o corpo do kernel não usa, e a varredura do device olharia um
/// número de células diferente do que a lei pede — sem erro em lugar nenhum.
#[test]
fn the_reduce_and_the_binding_agree_on_what_an_absent_size_is_worth() {
    let spec = gpu::REDUCES
        .iter()
        .find(|r| r.name == "rmax")
        .expect("a reducao do maior disco existe");
    let binding = gpu::GPU_KERNEL
        .bindings
        .iter()
        .find(|b| b.column == "size" && b.port == 0)
        .expect("a binding de `size` existe");
    assert_eq!(spec.column, "size");
    assert_eq!(spec.port, binding.port);
    assert_eq!(spec.dim, binding.dim);
    assert_eq!(
        spec.identity, binding.identity,
        "a identidade da reducao E a da binding sao a MESMA"
    );
    // E ela é o que a CPU responde a uma coluna ausente.
    assert_eq!(radius_scale(&dots(&[[0.0, 0.0]]), 1), vec![1.0]);
}

/// **A REDUÇÃO É `Max`, e é isso que a torna bit-exata entre as duas rotas.**
///
/// O doc do `ReduceOp` diz por quê: `Max` é associativo **e exato** sobre floats,
/// então a árvore do device e o `fold` da CPU dão o mesmo padrão de bits em
/// qualquer ordem de visita — ao contrário de `Sum`, cuja paridade carrega ε.
/// Trocar o operador aqui tornaria o alcance da varredura dependente de ordem.
#[test]
fn the_biggest_disc_is_found_with_an_operator_that_is_exact_in_any_order() {
    let spec = gpu::REDUCES.iter().find(|r| r.name == "rmax").unwrap();
    assert_eq!(spec.op, ph2d_nodegraph::reduce_meta::ReduceOp::Max);
}

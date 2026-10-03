//! ⭐⭐⭐⭐ **OS GATES DAS CURAS DA `W9`** que sobreviveram ao Render traçado (03/10): o vaso por
//! fórmula, o modo de omissão pintado no dispositivo e o desconto da compilação. (A fita inerte, a
//! lei do dono e a cache do chão eram curas do pintor de material, e saíram com ele.)
//!
//! ⚠️ **Eles saíram do [`super`] por um tecto de LOC** (2026-09-23, `1 951` contra `700`): o irmão
//! mede *o divisor depois do dispositivo* e estes medem *o que as curas da `W9` compraram*.

/// ⛔⛔⛔ **O VASO DESCE POR FÓRMULA, E A FITA DELE CABE NUMA MÃO** — e este gate SUBSTITUI um cuja
/// premissa morreu no mesmo dia.
///
/// # A premissa que morreu, e ela era minha
///
/// De manhã este sítio tinha o `nenhuma_regiao_do_vaso_paga_a_arvore_inteira`: ele varria uma
/// grelha `32×32` em `(u, v)`, compilava a região de cada célula e exigia que o pior caso cortasse
/// pelo menos `2×` — a cura da costura do eixo. ⚠️ **À tarde o dono mandou o torno descer por
/// FÓRMULA**, e uma fórmula **não tem arestas para cortar**: a região dela é a identidade, o pior
/// caso é `1,0×`, e o gate reprovou sobre produto correcto.
///
/// ⭐ A LEI da costura do eixo continua gateada, no sítio onde ela vive e sobre uma fixtura que a
/// contém — `ph2d-field-eval`,
/// `profile_index::eixo_tests::uma_regiao_sobre_a_costura_nao_paga_a_arvore_inteira`. O que se
/// perdeu foi a metade *«na peça do dono»*, porque a peça do dono **deixou de tomar aquele
/// caminho**. ⇒ *o que fica aqui é a afirmação NOVA que o produto passou a fazer.*
///
/// # O que ele afirma
///
/// Que a peça da cena `5` — `24` primitivas desenhadas, `931` linhas pela lei exacta — desce em
/// **`124`** linhas, e que a fita dela é pequena o bastante para o prévio não ter de baixar a
/// resolução (medido: o divisor foi de `2` para `1`, `32,53 → 16,63 ms`).
///
/// ⭐ **E o CONTROLO é a lei exacta ao lado**: sem ele, uma fórmula que devolvesse uma constante
/// passaria a primeira metade.
#[test]
fn o_vaso_desce_por_formula_e_a_fita_cabe_numa_mao() {
    // ⚠️ Medido: `124` linhas pela fórmula contra `931` pela lei exacta. A barra é `200` — folga de
    // `60 %` sobre o medido e `4,6×` abaixo da lei exacta —, ⛔ não um número redondo: ela é o degrau
    // em que a fita deixa de caber no orçamento do prévio com divisor `1` (`4,32 + 0,030 × 200 ≈
    // 10,3 ms` contra os `16,7`).
    const TECTO: usize = 200;
    let doc = crate::smoke::scenes::vaso(ph2d_field::DEFAULT_PROFILE_RESOLUTION);
    let ph2d_field::NodeKind::Leaf(ph2d_field::Primitive::Revolve { profile }) =
        &doc.nodes()[0].kind
    else {
        panic!("a cena 5 é um Revolve — se deixou de ser, este gate mede outra peça");
    };
    let pela_formula = ph2d_field_eval::Field::new(&doc)
        .tape_shape()
        .expect("a fita do torno")
        .guardados;
    let exacta = ph2d_field_eval::Field::from_tree(
        &ph2d_field_eval::profile::probe_sd_revolve_exacto(profile),
    )
    .tape_shape()
    .expect("a fita do contorno desenhado")
    .guardados;
    assert!(
        pela_formula <= TECTO,
        "a peça da cena 5 desce em {pela_formula} linhas contra o tecto de {TECTO} — ou a cerca da \
         fidelidade passou a recusá-la (e ela voltou ao contorno desenhado), ou o grau subiu"
    );
    // ⭐ **O CONTROLO**: a lei exacta tem de ser MUITO maior, senão este gate não mede cura nenhuma.
    assert!(
        exacta >= 4 * pela_formula,
        "CONTROLO: a lei exacta desce em {exacta} linhas contra {pela_formula} da fórmula — sem uma \
         ordem de grandeza entre as duas, a metade de cima não afirma nada"
    );
}

/// ⭐⭐⭐⭐ **O MODO DE OMISSÃO DO MODELADOR VAI À PLACA** — o Matcap é pintado no dispositivo quando
/// a placa sabe marchar a peça, e só cai na CPU quando não sabe.
///
/// Sem ele, o arrasto de uma cena recém-aberta era **todo** de CPU (`90,17 ms` e `D=3` contra
/// `16,63` e `D=1`, report de 2026-09-23 *«ao arrastar fica grosseiro ainda»*). ⛔ E abrir a MARCHA
/// ao matcap não é a cura: o [`crate::gpu_frame::march`] devolve o G-buffer pelo barramento
/// (`49,8 MB` a `1920×1080`) — *mais lento do que a CPU inteira*. O que ganha é a IMAGEM não
/// atravessar o barramento. (Até 03/10 havia uma segunda lei de pintura, a de material do Render
/// traçado; saiu com ele.)
///
/// ⚠️ **A régua lê o CÓDIGO e não a prosa**: cada metade recorta a expressão dela.
#[test]
fn o_modo_de_omissao_e_pintado_no_dispositivo() {
    const FONTE: &str = include_str!("smoke_draw_thread.rs");
    let codigo = &FONTE[FONTE
        .find("pub(crate) fn traca(")
        .expect("a resposta ao pedido")..];

    // ⭐⭐⭐ **METADE 1 — o ramo da placa passa pela porta do dispositivo e pinta.**
    let i = codigo.find("if crate::gpu_frame::takes_the_frame(").expect(
        "⛔ O RAMO DO MATCAP NO DISPOSITIVO DESAPARECEU: o modo de OMISSÃO do modelador volta a \
         traçar inteiro na CPU (`90,17 ms` e `D=3` contra `16,63` e `D=1`), que é o report \
         «ao arrastar fica grosseiro ainda» de 2026-09-23 a voltar",
    );
    let ramo = &codigo[i..];
    let ramo = &ramo[..ramo.find("return;").expect("o ramo da placa devolve")];
    assert!(
        ramo.contains("pinta_matcap("),
        "o ramo da placa existe e NÃO chama o passe que pinta — um ramo que cai para a CPU em \
         silêncio lê-se, no relógio, exactamente como não ter ramo nenhum"
    );
    // ⭐⭐ **METADE 2 — o CONTROLO: o recuo da CPU vem DEPOIS do ramo da placa.** Sem ela, um recuo
    // posto antes ficaria verde na metade de cima e o dispositivo nunca seria tentado.
    let cpu = codigo
        .find("ph2d_field_render::trace_cancellable(")
        .expect("o recuo da CPU saiu do despacho do quadro");
    assert!(
        cpu > i,
        "o recuo da CPU corre ANTES do ramo da placa — o modo de omissão nunca chega ao dispositivo"
    );
}

/// ⭐ **O DESCONTO DA COMPILAÇÃO está no quadro da placa** — o tempo que alimenta o laço da
/// resolução tira o que o quadro pagou a compilar (`Pintado::compilado_ms`): ele decide o tamanho
/// do quadro seguinte, que não o paga. Censo de texto porque a thread de desenho pede uma janela e
/// um dispositivo. ⚠️ Eram DOIS quadros da placa até 03/10 (o pintor de material e o matcap); o
/// Render traçado saiu e fica o do matcap.
#[test]
fn o_laco_da_resolucao_desconta_a_compilacao() {
    let fonte = include_str!("smoke_draw_thread.rs");
    let n = fonte
        .matches("t0.elapsed().as_secs_f64() * 1000.0 - pintura.compilado_ms")
        .count();
    assert_eq!(
        n, 1,
        "o desconto da compilação devia estar no quadro da placa (o matcap) e está em {n}"
    );
}

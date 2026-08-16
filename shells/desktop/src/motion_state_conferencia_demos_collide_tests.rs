//! Gates da cena `=48` — **O DISCO TEM O TAMANHO QUE SE VÊ**.
//!
//! Eles medem a geometria que a cena COZINHA, não a intenção com que foi escrita.
//!
//! ⚠️ **A régua de um empacotamento é o VÃO entre vizinhos, nunca o Y** — cada
//! fileira carrega o próprio `offset_y`, e comparar Y entre bandas mede o
//! `BAND_GAP`. É a quarta vez que este repo paga essa lição (o `1,15` do grupo B,
//! o `offset_y` do Range no C, os gêmeos do D).

use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

/// Cozinha a cena e devolve, por banda, `(x ordenado, meia-largura por peça)`.
fn cook_band(lane: usize) -> (Vec<f32>, Vec<f32>) {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).unwrap();
    let mut doc = MotionDoc::default();
    let sinks = build_collide_demo_document(&mut doc, &reg).expect("a cena monta");
    assert_eq!(sinks.len(), BANDS, "uma fileira por banda");

    let mut cook = Cook::new();
    let s = cook.cook(&doc.graph, &reg, sinks[lane], 0.0).expect("cook")[0]
        .as_stream()
        .clone();
    let n = s.count();
    let p = match s.get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => panic!("P"),
    };
    let half: Vec<f32> = match s.get("size") {
        Some(Column::Vec2(v)) => v.iter().map(|e| e[0].abs().max(e[1].abs())).collect(),
        _ => vec![1.0; n],
    };
    // Ordena por x, carregando o tamanho junto — a lei é sobre VIZINHOS na
    // fileira, e o solver não preserva a ordem do índice.
    let mut pairs: Vec<(f32, f32)> = p.iter().map(|q| q[0]).zip(half).collect();
    pairs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    (
        pairs.iter().map(|e| e.0).collect(),
        pairs.iter().map(|e| e.1).collect(),
    )
}

/// Os vãos entre vizinhos consecutivos.
fn gaps(xs: &[f32]) -> Vec<f32> {
    xs.windows(2).map(|w| w[1] - w[0]).collect()
}

/// **A CENA MONTA AS SEIS BANDAS QUE O LOG NOMEIA.**
///
/// ⚠️ A contagem é DERIVADA (`BAND_LABELS.len()`), nunca um literal: um número
/// escrito à mão só sabe dizer *"mudou"*, e a pergunta é se a lista que o artista
/// lê corresponde ao que a cena de facto empilha.
#[test]
fn the_scene_builds_every_band_the_log_names() {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).unwrap();
    let mut doc = MotionDoc::default();
    let sinks = build_collide_demo_document(&mut doc, &reg).expect("a cena monta");
    assert_eq!(sinks.len(), BAND_LABELS.len());
    assert_eq!(sinks.len(), BANDS);
}

/// **A FIXTURE CONTÉM O FENÔMENO: a fileira nasce SOBREPOSTA e o nó a desfaz.**
///
/// Sem isto todo par abaixo compararia dois empacotamentos que nunca precisaram
/// acontecer — a cena diria *"funciona"* sobre um solver inerte.
#[test]
fn the_row_starts_crowded_and_the_node_spreads_it() {
    // ⚠️ A PREMISSA, afirmada em vez de assumida: o passo de partida é menor que o
    // diâmetro de um disco, logo toda vizinha nasce sobreposta.
    // Um `const` block: a premissa é de COMPILAÇÃO, e é assim que ela falha no
    // instante em que alguém move uma das três constantes sem mover as outras.
    const { assert!(PITCH < 2.0 * RADIUS * DOT) };
    let (xs, _) = cook_band(0);
    assert_eq!(xs.len(), COLS);
    // E a promessa que o proprio doc do no' faz: nenhum par mais perto que isso.
    // ⚠️ A barra admite o RESÍDUO DE CONVERGÊNCIA, e não é frouxidão: o próprio
    // doc do nó mede a folga mínima como fração do alvo e ela sobe com as
    // varreduras (0,270 a oito). Exigir o alvo EXATO seria pedir a um solver
    // iterativo o que ele não promete em passos finitos — o gate falharia sobre
    // código correto. Medido a 64 varreduras: 0,4095 de 0,42, ou 97,5%.
    let target = 2.0 * RADIUS * DOT;
    let min = gaps(&xs).iter().fold(f32::MAX, |m, v| m.min(*v));
    assert!(
        min > target * 0.95,
        "todo par tem de acabar praticamente a TOCAR: menor vao {min} contra {target}"
    );
}

/// **O CONTROLE TEM UM PASSO SÓ, E A FILEIRA VARIADA NÃO.**
///
/// ⚠️ É o par 1-2 inteiro. *"A fileira 2 tem vãos diferentes"* sozinho não diria
/// nada — qualquer coisa por-índice o produziria; a metade que decide é a de cima
/// ser UNIFORME sobre a MESMA entrada.
#[test]
fn the_uniform_row_packs_on_one_pitch_and_the_varied_one_does_not() {
    let (a, _) = cook_band(0);
    let (b, _) = cook_band(1);
    let (ga, gb) = (gaps(&a), gaps(&b));
    let spread = |g: &[f32]| {
        let (lo, hi) = g
            .iter()
            .fold((f32::MAX, f32::MIN), |(l, h), v| (l.min(*v), h.max(*v)));
        hi - lo
    };
    let (sa, sb) = (spread(&ga), spread(&gb));
    let target = 2.0 * RADIUS * DOT;
    assert!(
        sa < 0.05 * target,
        "CONTROLE: com tamanho uniforme o passo e' o MESMO em toda a fileira \
         (a menos do residuo de convergencia); medido {sa} contra um alvo de {target}"
    );
    // ⚠️ A régua é a RAZÃO entre as duas dispersões, não um número absoluto: o
    // resíduo de convergência move `sa` com as varreduras, e um bar fixo seria
    // recalibrado toda vez que o solver melhorasse. Medido: 0,0084 contra 0,441.
    assert!(
        sb > 10.0 * sa,
        "e com tamanho variado ele nao pode ser: {sb} contra {sa}"
    );
}

/// **O VÃO SEGUE O TAMANHO — `r_i + r_j`, medido no cozido.**
///
/// ⚠️ O oráculo não é *"os vãos crescem"*: é que cada vão seja PROPORCIONAL à soma
/// das duas meias-larguras vizinhas. Uma lei que lesse só o disco da esquerda, ou
/// só o máximo, produziria vãos crescentes e falharia aqui.
#[test]
fn each_gap_follows_the_sum_of_the_two_discs_that_share_it() {
    let (xs, half) = cook_band(1);
    let g = gaps(&xs);
    let mut ratios = Vec::with_capacity(g.len());
    for (k, gap) in g.iter().enumerate() {
        let sum = half[k] + half[k + 1];
        assert!(sum > 0.0);
        ratios.push(gap / sum);
    }
    let (lo, hi) = ratios
        .iter()
        .fold((f32::MAX, f32::MIN), |(l, h), v| (l.min(*v), h.max(*v)));
    // A razão TEM de ser o próprio `radius` do nó, e a tolerância é a do resíduo
    // de convergência acima (a banda 1 mostra 2%).
    assert!(
        hi - lo < 0.05 * hi,
        "o vao e' proporcional a r_i + r_j: as razoes tem de coincidir, medido [{lo}, {hi}]"
    );
    assert!(
        (hi - RADIUS).abs() < 0.1 * RADIUS,
        "e a razao E' o proprio raio do no' ({RADIUS}); medido [{lo}, {hi}]"
    );
    // E o CONTROLE de que a fixture varia mesmo: o maior disco é bem maior.
    let (hlo, hhi) = half
        .iter()
        .fold((f32::MAX, f32::MIN), |(l, h), v| (l.min(*v), h.max(*v)));
    assert!(hhi > 2.0 * hlo, "os tamanhos tem de variar: [{hlo}, {hhi}]");
}

/// **A CAIXA DESLIGA O NÓ NO MEIO DA FILEIRA.**
///
/// As peças dentro dela se atravessam (o vão colapsa para perto do passo de
/// partida) e as de fora seguem afastadas — contra um controle que empacota
/// inteiro.
#[test]
fn inside_the_box_the_discs_pass_through_each_other() {
    let (a, _) = cook_band(2);
    let (b, _) = cook_band(3);
    let ga = gaps(&a);
    let gb = gaps(&b);
    let target = 2.0 * RADIUS * DOT;
    let min_a = ga.iter().fold(f32::MAX, |m, v| m.min(*v));
    assert!(
        min_a > target * 0.95,
        "CONTROLE: sem campo, todo vizinho fica a um diametro; medido {min_a}"
    );

    // ⚠️ **O oráculo tem de dizer ONDE, e a primeira versão não dizia.** Ela
    // afirmava *"algum par sobrepõe E algum par toca"*, que é SIMÉTRICO: trocar o
    // `invert` da caixa (miolo ativo, pontas mudas) satisfaz as duas metades, e a
    // mutação SOBREVIVEU. O que a cena promete é que o vão colapsado está no MEIO.
    let mid_of: Vec<f32> = b.windows(2).map(|w| (w[0] + w[1]) * 0.5).collect();
    let mut by_centre: Vec<usize> = (0..gb.len()).collect();
    by_centre.sort_by(|&i, &j| mid_of[i].abs().partial_cmp(&mid_of[j].abs()).unwrap());
    let innermost = gb[by_centre[0]];
    let outermost = gb[*by_centre.last().unwrap()];
    assert!(
        innermost < target * 0.9,
        "o vao COLAPSADO tem de estar no MEIO (x = {:.3}): medido {innermost} contra o \
         diametro {target}",
        mid_of[by_centre[0]]
    );
    assert!(
        outermost > target * 0.95,
        "e o das PONTAS (x = {:.3}) tem de seguir a tocar: medido {outermost}",
        mid_of[*by_centre.last().unwrap()]
    );
}

/// **MUTAR NÃO É PINAR** — o par 5-6, e é a distinção inteira da linha 62.
///
/// ⚠️ As duas fileiras têm a MESMA peça no meio e o MESMO nó de constraint; o que
/// difere é para onde o número vai. A peça MUTADA é atravessada (uma vizinha
/// chega mais perto dela que um raio); a PINADA é obstáculo — ela não se move e
/// nenhuma vizinha a atravessa.
#[test]
fn a_muted_disc_is_walked_through_and_a_pinned_one_is_an_obstacle() {
    let mid = COLS / 2;
    let (muted, _) = cook_band(4);
    let (pinned, _) = cook_band(5);

    // Onde a peça do meio NASCEU (a grade é centrada, o passo é `PITCH`).
    #[expect(clippy::cast_precision_loss, reason = "quinze colunas")]
    let born = 0.0f32.mul_add(1.0, 0.0) + ((mid as f32) - ((COLS - 1) as f32) / 2.0) * PITCH;

    // A PINADA não se move — é o que `inv_mass = 0` promete.
    let nearest_pinned = pinned.iter().fold(f32::MAX, |m, x| m.min((x - born).abs()));
    assert!(
        nearest_pinned < 1e-3,
        "a peca pinada fica onde nasceu ({born}); a mais proxima ficou a {nearest_pinned}"
    );

    // E a MUTADA é ATRAVESSADA: alguma vizinha chega a menos de um raio dela,
    // que é impossível numa fileira que a respeitasse.
    let min_gap_muted = gaps(&muted).iter().fold(f32::MAX, |m, v| m.min(*v));
    let min_gap_pinned = gaps(&pinned).iter().fold(f32::MAX, |m, v| m.min(*v));
    assert!(
        min_gap_muted < RADIUS * DOT,
        "a peca mutada e' TRANSPARENTE: alguem tem de passar por dentro dela; \
         menor vao {min_gap_muted}"
    );
    assert!(
        min_gap_pinned > RADIUS * DOT,
        "CONTROLE: a pinada e' OBSTACULO, entao ninguem a atravessa; menor vao {min_gap_pinned}"
    );
}

/// **SONDA — os números que a mensagem do smoke cita.**
///
/// Não afirma nada: imprime o que as seis bandas de facto cozinham, para a
/// mensagem do roteador sair de uma MEDIÇÃO e não de uma expectativa. Rodar:
/// `cargo test -p ph2d-host-desktop --release conferencia_demos_collide -- --ignored --nocapture`
#[test]
#[ignore = "sonda: mede, nao afirma"]
fn measure_what_the_six_bands_draw() {
    println!("\n== a cena =48, medida ==");
    for (b, label) in BAND_LABELS.iter().enumerate() {
        let (xs, half) = cook_band(b);
        let g = gaps(&xs);
        let (glo, ghi) = g
            .iter()
            .fold((f32::MAX, f32::MIN), |(l, h), v| (l.min(*v), h.max(*v)));
        let (hlo, hhi) = half
            .iter()
            .fold((f32::MAX, f32::MIN), |(l, h), v| (l.min(*v), h.max(*v)));
        println!(
            "  {label}\n      vao [{glo:.4} .. {ghi:.4}]  meia-largura [{hlo:.4} .. {hhi:.4}]  \
             extensao {:.4}",
            xs[xs.len() - 1] - xs[0]
        );
    }
}

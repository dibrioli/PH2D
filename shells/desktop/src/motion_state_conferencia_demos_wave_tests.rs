//! Os gates da cena `=57` — N produtores num campo de onda, por COMPOSIÇÃO.

use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

/// Tiques que a sonda deixa correr antes de medir. A onda viaja a `speed` células
/// por tique, então isto é o que dá tempo à frente injetada de sair da máscara.
const TICKS: usize = 240;
const DT: f64 = 1.0 / 60.0;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    reg
}

/// Coze a cena por `TICKS` e devolve, por banda, `(wave_h, P)`.
///
/// ⚠️ **O `advance_tick` é o que faz a aresta `pre` CARREGAR estado** — sem ele a
/// cadeia nunca integra, os dois campos ficam em zero e todo gate abaixo passa a
/// comparar duas grades mortas (verde por vácuo, a lição que o `rope_thickness`
/// pagou duas vezes).
fn settle() -> Vec<(Vec<f32>, Vec<[f32; 2]>)> {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_wave_demo_document(&mut doc, &reg).expect("a cena monta");
    let mut cook = Cook::new();
    let mut out = vec![(Vec::new(), Vec::new()); sinks.len()];
    for k in 0..TICKS {
        let t = k as f64 * DT;
        cook.advance_tick(&doc.graph, &reg, t)
            .expect("o tique avança");
        for (i, s) in sinks.iter().enumerate() {
            let v = cook.cook(&doc.graph, &reg, *s, t).expect("a banda coze");
            let st = v[0].as_stream();
            let h = match st.get("wave_h") {
                Some(Column::Scalar(v)) => v.clone(),
                _ => panic!("a banda emite wave_h"),
            };
            let p = match st.get("P") {
                Some(Column::Vec2(v)) => v.clone(),
                _ => panic!("a banda emite P"),
            };
            out[i] = (h, p);
        }
    }
    out
}

fn max_abs(v: &[f32]) -> f32 {
    v.iter().fold(0.0f32, |m, x| m.max(x.abs()))
}

/// Onde o campo tem o maior |h|, em `x` de MUNDO (já com o deslocamento da banda).
fn peak_x(h: &[f32], p: &[[f32; 2]]) -> f32 {
    let mut best = (0.0f32, 0.0f32);
    for (i, z) in h.iter().enumerate() {
        if z.abs() > best.0 {
            best = (z.abs(), p[i][0]);
        }
    }
    best.1
}

/// **SONDA — os números que a mensagem do smoke cita.** Imprime e não afirma.
#[test]
#[ignore = "sonda: imprime numeros, nao afirma"]
fn probe_what_the_scene_draws() {
    let f = settle();
    let (ctrl_h, ctrl_p) = &f[0];
    let (comp_h, comp_p) = &f[1];
    eprintln!("\n[=57] {} pecas por banda", ctrl_h.len());
    eprintln!(
        "  controle  max |h| = {:.4}   pico em x = {:+.2}",
        max_abs(ctrl_h),
        peak_x(ctrl_h, ctrl_p)
    );
    eprintln!(
        "  composto  max |h| = {:.4}   pico em x = {:+.2}  (caixa em {BOX_X})",
        max_abs(comp_h),
        peak_x(comp_h, comp_p)
    );
    let moved = ctrl_h
        .iter()
        .zip(comp_h.iter())
        .filter(|(a, b)| (*a - *b).abs() > 1e-4)
        .count();
    eprintln!("  celulas que a cadeia move: {moved} de {}", ctrl_h.len());
    let size_span = |h: &[f32]| {
        let m = max_abs(h);
        (0.22, 0.22 + 1.4 * m) // SIZE_BASE + SIZE_GAIN * |h|, a lei do no'
    };
    for (tag, h) in [("controle", ctrl_h), ("composto", comp_h)] {
        let over = h.iter().filter(|z| 0.22 + 1.4 * z.abs() > 0.5).count();
        let mean = h.iter().map(|z| 0.22 + 1.4 * z.abs()).sum::<f32>() / h.len() as f32;
        eprintln!(
            "  {tag}: pecas MAIORES que o passo (0,50): {over} de {}   media {mean:.3}",
            h.len()
        );
    }
    eprintln!(
        "  tamanho das pecas  controle {:.2}..{:.2}   composto {:.2}..{:.2}",
        size_span(ctrl_h).0,
        size_span(ctrl_h).1,
        size_span(comp_h).0,
        size_span(comp_h).1
    );
}

/// **A CENA MONTA AS DUAS BANDAS E CADA UMA TERMINA NUM OUTPUT.**
///
/// ⚠️ O sink não é decoração: o laço de render re-resolve os sinks a partir dos nós
/// de saída do grafo, então uma banda sem `motion.output` cozinha certo, satisfaz os
/// gates de campo e **desenha nada** — indistinguível da feature quebrada.
#[test]
fn the_scene_builds_two_bands_that_both_end_in_an_output() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_wave_demo_document(&mut doc, &reg).expect("a cena monta");
    assert_eq!(sinks.len(), 2, "duas bandas: o controle e o composto");
    for s in &sinks {
        assert_eq!(
            doc.graph.node(*s).map(|n| n.type_name.as_str()),
            Some("motion.output"),
            "toda banda termina num Output"
        );
    }
    assert_eq!(band_labels().count(), 2, "um rótulo por banda");
}

/// **O SEGUNDO PRODUTOR MOVE O BERÇO DAS ONDAS.**
///
/// ⚠️ O oráculo é **ONDE o campo tem o pico**, não *quanto* ele mede: um campo
/// simplesmente mais agitado também teria `max |h|` maior, e não seria um produtor.
/// O controle tem a fonte no centro (`x ≈ 0`); o composto tem de puxar o pico para
/// perto da caixa, que a cena põe em `BOX_X`.
#[test]
fn the_injected_producer_moves_where_the_waves_are_born() {
    let f = settle();
    let (ctrl_h, ctrl_p) = &f[0];
    let (comp_h, comp_p) = &f[1];

    // As duas bandas são deslocadas em Y pelo `motion.move`, então o `x` é comparável.
    let cx = peak_x(ctrl_h, ctrl_p);
    let bx = peak_x(comp_h, comp_p);
    assert!(
        cx.abs() < 1.0,
        "o CONTROLE nasce no centro da grade: pico em x = {cx:.2}"
    );
    assert!(
        (bx - BOX_X).abs() < 1.0,
        "o composto nasce na CAIXA (x = {BOX_X}): pico em x = {bx:.2}"
    );
    assert!(
        (bx - cx).abs() > 1.5,
        "os dois berços têm de estar longe um do outro (controle {cx:.2}, composto {bx:.2})"
    );
}

/// **O QUE ELE DEPOSITA PROPAGA — logo é um PRODUTOR, não tinta no campo.**
///
/// A distinção é o item inteiro: escrever um número na coluna de altura e ele ficar
/// parado ali seria pintar; o que a onda faz com ele é o que o torna uma fonte.
#[test]
fn what_the_injector_writes_travels_across_the_field() {
    let f = settle();
    let (ctrl_h, _) = &f[0];
    let (comp_h, comp_p) = &f[1];

    // A máscara da caixa tem meia-largura 0,4 + `soft` 0,3 ⇒ tudo além de ~1,2 do
    // centro dela é campo que só a PROPAGAÇÃO pode ter movido.
    const REACH: f32 = 1.2;
    let mut far = 0.0f32;
    let mut far_cells = 0usize;
    for (i, p) in comp_p.iter().enumerate() {
        let d = (comp_h[i] - ctrl_h[i]).abs();
        let dx = p[0] - BOX_X;
        // O `y` das duas bandas difere pelo `motion.move`, então a distância à caixa
        // é medida só em `x` mais a linha da grade, que é a mesma nas duas.
        if dx.abs() > REACH {
            far = far.max(d);
            if d > 1e-4 {
                far_cells += 1;
            }
        }
    }
    assert!(
        far > 0.05,
        "longe da máscara o campo tem de ter mudado de verdade (pior {far:.6})"
    );
    assert!(
        far_cells > comp_p.len() / 2,
        "a frente atravessa a grade: {far_cells} de {} células longe da máscara se moveram",
        comp_p.len()
    );
}

/// **AS DUAS BANDAS SÃO COMPARÁVEIS EM AMPLITUDE — o confundidor tem barra.**
///
/// ⚠️ É o precedente do **Grupo N** feito executável: *se as duas diferissem muito,
/// "a de baixo mexe mais" responderia por qualquer coisa*, e a cena provaria um
/// segundo controle de amplitude em vez de um segundo PRODUTOR. A barra é `1,5×`
/// contra os **1,24×** medidos — e ⚠️ ela **não pode ser 1,0×**: a varredura do
/// `scale` mostra que abaixo de ~0,25 o berço injetado deixa de vencer e o pico volta
/// ao centro, então igualdade exata compraria a leitura ao preço do item.
#[test]
fn the_two_bands_are_comparable_in_amplitude() {
    let f = settle();
    let (ctrl, comp) = (max_abs(&f[0].0), max_abs(&f[1].0));
    let ratio = comp / ctrl;
    assert!(
        (1.0..1.5).contains(&ratio),
        "as duas bandas têm de ser comparáveis, não 'a de baixo é maior' \
         (controle {ctrl:.4}, composto {comp:.4}, razão {ratio:.2})"
    );
}

/// **A CENA AUTORA A COLUNA PELO NOME, e é este nome que o artista tem de saber.**
///
/// ⚠️ É a metade ERGONÔMICA do veredito `P2`: a capacidade existe, mas ela é
/// alcançada digitando `wave_h` num campo de texto — um nome de ESTADO que nenhum
/// picker de canal oferece.
#[test]
fn the_chain_names_the_state_column_in_a_text_param() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    build_wave_demo_document(&mut doc, &reg).expect("a cena monta");
    let drives: Vec<_> = doc
        .graph
        .nodes()
        .iter()
        .filter(|n| n.type_name == "motion.drive")
        .map(|n| n.id)
        .collect();
    assert_eq!(drives.len(), 1, "só a banda composta tem escritor");
    assert_eq!(
        doc.graph
            .node_text_param_overrides(drives[0])
            .and_then(|m| m.get("column"))
            .map(String::as_str),
        Some(state_column()),
        "o drive escreve na coluna de estado da onda, pelo NOME"
    );
}

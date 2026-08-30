//! ⭐⭐⭐ **OS GATES DO DESEMARANHADOR** — e o que decide se ele existe é o primeiro: *ele
//! desfaz uma dobra a partir de um estado JÁ dobrado.*

use super::{Element, Settings, chi, energy, energy_and_gradient, flipped, min_det, untangle};

/// Uma grelha `n × n` sobre o quadrado unitário, triangulada, com o repouso **igual** ao
/// estado inicial — logo a identidade, com `det J = 1` em toda parte.
fn grelha(n: usize) -> (Vec<Element>, Vec<[f64; 2]>, Vec<bool>) {
    #[expect(
        clippy::cast_precision_loss,
        reason = "n <= 5 nestas fixturas; a posicao nao precisa de mais que isto"
    )]
    let passo = 1.0 / (n - 1) as f64;
    let mut uv: Vec<[f64; 2]> = Vec::new();
    for j in 0..n {
        for i in 0..n {
            #[expect(
                clippy::cast_precision_loss,
                reason = "indices de grelha pequenos, convertidos para posicao"
            )]
            uv.push([i as f64 * passo, j as f64 * passo]);
        }
    }
    let rest = uv.clone();
    let idx = |i: usize, j: usize| u32::try_from(j * n + i).expect("grelha pequena");
    let mut elements = Vec::new();
    for j in 0..n - 1 {
        for i in 0..n - 1 {
            for tri in [
                [idx(i, j), idx(i + 1, j), idx(i + 1, j + 1)],
                [idx(i, j), idx(i + 1, j + 1), idx(i, j + 1)],
            ] {
                elements.push(
                    Element::from_rest(
                        tri,
                        rest[tri[0] as usize],
                        rest[tri[1] as usize],
                        rest[tri[2] as usize],
                    )
                    .expect("a fixtura nao tem triangulo degenerado"),
                );
            }
        }
    }
    // A fronteira fica presa; o interior é livre.
    let mut locked = vec![false; uv.len()];
    for j in 0..n {
        for i in 0..n {
            if i == 0 || j == 0 || i == n - 1 || j == n - 1 {
                locked[j * n + i] = true;
            }
        }
    }
    (elements, uv, locked)
}

/// ⭐⭐⭐ **O GATE DA CRATE — ele desemaranha a partir de um estado JÁ dobrado.**
///
/// ⛔ **É a propriedade que a torna útil aqui e que quase toda a família da injectividade não
/// tem**: os métodos que exigem partida válida são inúteis para nós, porque a nossa partida
/// é o mapa da cadeia — com `3,12 %` de dobras no ombro de um espinho.
#[test]
fn ele_desfaz_uma_dobra_a_partir_de_um_estado_ja_dobrado() {
    let (el, mut uv, locked) = grelha(3);
    // ⛔ O CONTROLE: a fixtura tem MESMO o fenómeno. Sem esta asserção o gate ficaria verde
    // sobre uma malha que nunca esteve dobrada.
    uv[4] = [1.4, 1.4];
    let antes = flipped(&el, &uv);
    assert!(
        antes > 0,
        "⛔ a fixtura tem de conter dobras, senao este gate nao prova nada"
    );
    assert!(
        min_det(&el, &uv) < 0.0,
        "⛔ e o pior determinante e' negativo"
    );

    let rep = untangle(&el, &mut uv, &locked, Settings::default());

    assert_eq!(
        rep.flipped_after, 0,
        "⛔ sobraram {} elementos invertidos (min det {:.6})",
        rep.flipped_after, rep.min_det.1
    );
    assert!(
        rep.min_det.1 > 0.0,
        "⛔ o determinante minimo tem de ficar POSITIVO: {:.6}",
        rep.min_det.1
    );
    assert!(!rep.gave_up, "⛔ e ele nao pode declarar desistencia");
    assert_eq!(rep.flipped_before, antes, "⛔ o relatorio diz o que entrou");
}

/// ⭐⭐⭐ **GATE — a energia é FINITA sobre um emaranhado, e é isso que a regularização compra.**
///
/// ⚠️ Sem `χ`, a energia tem `det J` no denominador e vale `+∞` (ou muda de sinal) sobre um
/// elemento invertido — *e uma energia infinita não dá direcção nenhuma a uma descida.*
#[test]
fn a_energia_e_finita_sobre_um_emaranhado_e_a_regularizacao_e_o_motivo() {
    let (el, mut uv, _) = grelha(3);
    uv[4] = [1.4, 1.4];
    assert!(flipped(&el, &uv) > 0);

    let e = energy(&el, &uv, 0.1, 1.0);
    assert!(
        e.is_finite() && e > 0.0,
        "⛔ a energia regularizada tem de ser finita e positiva sobre a malha dobrada: {e}"
    );

    // ⛔ O CONTROLE: é a `χ` que o faz — ela é estritamente positiva mesmo para `D` negativo.
    assert!(
        chi(-5.0, 0.1) > 0.0,
        "⛔ chi tem de ser positiva para D negativo, senao o denominador troca de sinal"
    );
    assert!(
        chi(-5.0, 0.0) <= 0.0 || chi(-5.0, 0.0).abs() < 1e-12,
        "⛔ e com epsilon zero ela tem de colapsar -- e' o limite que torna a dobra proibida"
    );
    // ⭐ E para `D` positivo com `ε → 0` ela devolve o próprio `D`.
    assert!((chi(3.0, 1e-9) - 3.0).abs() < 1e-6);
}

/// ⭐⭐⭐ **GATE — o gradiente bate com as diferenças finitas.**
///
/// ⛔ **É o único gate que defende a MATEMÁTICA.** Uma derivada mal escrita ainda desce em
/// muitos casos (a busca linear salva-a) e o defeito só aparece como «convergiu devagar» —
/// que se lê como afinação, não como erro.
#[test]
fn o_gradiente_bate_com_as_diferencas_finitas() {
    for (nome, dobrado) in [("limpa", false), ("dobrada", true)] {
        let (el, mut uv, _) = grelha(3);
        if dobrado {
            uv[4] = [1.4, 1.4];
        } else {
            // Um estado perturbado mas válido — a identidade tem gradiente quase nulo e não
            // discriminaria nada.
            uv[4] = [0.62, 0.44];
        }
        let (eps, lambda) = (0.3, 1.0);
        let mut grad = vec![[0.0f64; 2]; uv.len()];
        energy_and_gradient(&el, &uv, eps, lambda, &mut grad);

        let h = 1e-6;
        for v in [4usize, 1, 7] {
            for ax in 0..2 {
                let mut mais = uv.clone();
                let mut menos = uv.clone();
                mais[v][ax] += h;
                menos[v][ax] -= h;
                let num = (energy(&el, &mais, eps, lambda) - energy(&el, &menos, eps, lambda))
                    / (2.0 * h);
                let ana = grad[v][ax];
                let escala = num.abs().max(ana.abs()).max(1.0);
                assert!(
                    (num - ana).abs() / escala < 1e-5,
                    "⛔ [{nome}] vertice {v} eixo {ax}: numerico {num:.9} contra analitico \
                     {ana:.9}"
                );
            }
        }
    }
}

/// ⭐⭐ **GATE — um mapa JÁ válido não é estragado, e o relatório di-lo.**
#[test]
fn um_mapa_ja_valido_sai_valido() {
    let (el, mut uv, locked) = grelha(4);
    assert_eq!(flipped(&el, &uv), 0, "⛔ a identidade nao tem dobras");
    let rep = untangle(&el, &mut uv, &locked, Settings::default());
    assert_eq!(rep.flipped_before, 0);
    assert_eq!(rep.flipped_after, 0);
    assert!(!rep.gave_up);
    assert!(rep.min_det.1 > 0.0);
}

/// ⭐⭐⭐ **GATE — os vértices presos NÃO se movem.**
///
/// ⚠️ É a cerca de que o chamador depende: no nosso caso o que estiver preso é o que a costura
/// e as singularidades já fixaram, e movê-lo destruiria a propriedade que custou uma wave.
#[test]
fn os_presos_nao_se_movem() {
    let (el, mut uv, locked) = grelha(3);
    uv[4] = [1.4, 1.4];
    let antes = uv.clone();
    untangle(&el, &mut uv, &locked, Settings::default());
    for (i, l) in locked.iter().enumerate() {
        if *l {
            assert!(
                (uv[i][0] - antes[i][0]).abs() < 1e-15 && (uv[i][1] - antes[i][1]).abs() < 1e-15,
                "⛔ o vertice preso {i} mexeu-se: {:?} -> {:?}",
                antes[i],
                uv[i]
            );
        }
    }
    // ⛔ E o CONTROLE: o que estava livre TEM de se ter mexido, senao o gate acima é vacuo.
    assert!(
        (uv[4][0] - antes[4][0]).abs() > 1e-6,
        "⛔ o vertice livre nao se mexeu -- o gate dos presos ficaria vacuo"
    );
}

/// ⭐⭐ **GATE — «não medido» não se lê como «positivo».**
///
/// ⛔ *Um zero de «não medido» e um de «perfeito» são o mesmo byte* — esta linha pagou-o três
/// vezes noutras crates. Aqui o valor honesto é `+∞`: nenhum elemento tem determinante algum.
#[test]
fn sem_elementos_o_minimo_e_infinito_e_nao_zero() {
    assert_eq!(min_det(&[], &[]), f64::INFINITY);
    assert_eq!(flipped(&[], &[]), 0);
    let rep = untangle(&[], &mut [], &[], Settings::default());
    assert!(!rep.gave_up, "⛔ nada a desemaranhar nao e' desistencia");
    assert_eq!(rep.outer, 0, "⛔ e nao gasta iteracao nenhuma");
}

/// ⭐⭐ **GATE — um repouso degenerado é RECUSADO, e não remendado com um epsilon.**
#[test]
fn um_repouso_degenerado_e_recusado() {
    // Três pontos colineares — área zero, referencial não invertível.
    assert!(Element::from_rest([0, 1, 2], [0.0, 0.0], [1.0, 0.0], [2.0, 0.0]).is_none());
    // ⭐ E o controlo: um triângulo de verdade passa, com a área certa.
    let ok = Element::from_rest([0, 1, 2], [0.0, 0.0], [2.0, 0.0], [0.0, 3.0])
        .expect("triangulo valido");
    assert!((ok.area - 3.0).abs() < 1e-12, "area {}", ok.area);
}

/// ⭐⭐⭐ **GATE — o caso do «troca de lugares», que é o teste de sanidade da literatura.**
///
/// ⚠️ Dois vértices interiores trocam de posição: um emaranhado que nenhuma projecção local
/// desfaz, e que a literatura usa como o *desk-reject* de qualquer método que se diga injectivo.
#[test]
fn dois_interiores_que_trocam_de_lugar_desemaranham() {
    let (el, mut uv, locked) = grelha(4);
    let (a, b) = (5usize, 10usize); // os dois interiores de uma grelha 4×4
    assert!(!locked[a] && !locked[b], "⛔ os dois tem de ser livres");
    uv.swap(a, b);
    assert!(
        flipped(&el, &uv) > 0,
        "⛔ trocar dois interiores tem de dobrar alguma coisa"
    );
    let rep = untangle(&el, &mut uv, &locked, Settings::default());
    assert_eq!(
        rep.flipped_after, 0,
        "⛔ min det {:.9}, desistiu {}",
        rep.min_det.1, rep.gave_up
    );
}

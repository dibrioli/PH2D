//! ⏱️⭐⭐⭐⭐ **O VASO POR FÓRMULA** — a sonda que imprime a tabela; a lei vive na crate do campo.
//!
//! ⚠️ **Ela mora AQUI porque a peça mora aqui** (o vaso é o desenho da cena `5`), e o cálculo mora
//! na [`ph2d_field_eval::profile_formula::probe_formula_do_perfil`] porque a `Tree` da `fidget` mora
//! lá. *Trazer a `fidget` a esta crate por uma sonda seria uma dependência nova numa crate de
//! composição.*
//!
//! O cabeçalho da porta tem a pergunta do dono, a forma da fórmula e as duas metades que ela mede.

/// ⏱️⭐⭐⭐⭐ **A sonda.** Ver [`ph2d_field_eval::profile_formula::probe_formula_do_perfil`].
#[test]
#[ignore = "sonda de diagnóstico: mede o vaso por fórmula"]
fn diag_o_vaso_por_formula() {
    let doc = crate::smoke::scenes::vaso(ph2d_field::DEFAULT_PROFILE_RESOLUTION);
    let ph2d_field::NodeKind::Leaf(ph2d_field::Primitive::Revolve { profile }) =
        &doc.nodes()[0].kind
    else {
        panic!("a cena 5 é um Revolve");
    };
    // ⚠️ **A linha de base é a lei EXACTA, pela porta que não passa pela bissecção.** A 1.ª
    // redacção usava `Field::new(&doc)`, que desde esta wave desce pela FÓRMULA ⇒ a tabela comparava
    // a fórmula consigo própria e lia `1,0×` de ganho.
    let desenhada = ph2d_field_eval::Field::from_tree(
        &ph2d_field_eval::profile::probe_sd_revolve_exacto(profile),
    )
    .tape_shape()
    .expect("a fita do contorno desenhado")
    .guardados;
    let s = ph2d_field_eval::profile_formula::probe_silhueta_do_perfil(profile)
        .expect("a silhueta do vaso é a região entre duas funções da altura");
    let tabela =
        ph2d_field_eval::profile_formula::probe_formula_do_perfil(profile, &[4, 6, 8, 12, 16, 24])
            .expect("a régua acha a peça do vaso");
    println!(
        "\n  o vaso DESENHADO: {} primitivas, {desenhada} linhas de WGSL\n  a silhueta: {} \
         alturas em [{:.3} {:.3}], parede externa {:.3}..{:.3}",
        profile.prim_count(),
        s.len(),
        s[0].0,
        s[s.len() - 1].0,
        s.iter().map(|x| x.2).fold(f64::MAX, f64::min),
        s.iter().map(|x| x.2).fold(0.0, f64::max),
    );
    // ⚠️ A silhueta CRUA, a cada 24 alturas — sem ela, um erro de ajuste que não converge lê-se
    // como uma propriedade do desenho quando pode ser da régua.
    println!("  v · dentro · fora");
    for (v, d, f) in s.iter().step_by(6) {
        println!("  {v:>7.3} · {d:>7.3} · {f:>7.3}");
    }
    println!(
        "  grau · erro fora · erro dentro · linhas · × desenhada · inclin. amostrada · MAJORANTE"
    );
    for l in tabela {
        println!(
            "  {:>4} · {:>9.4} · {:>11.4} · {:>6} · {:>10.1}× · {:>17.1} · {:>9.1}",
            l.grau,
            l.erro_fora,
            l.erro_dentro,
            l.linhas,
            desenhada as f32 / l.linhas as f32,
            l.inclinacao,
            l.majorante,
        );
    }
    println!();
}

/// ⏱️⛔⛔⛔ **Sonda: QUANTO CUSTA AJUSTAR A FÓRMULA, E QUANTAS VEZES ISSO CORRE POR QUADRO.**
///
/// # A suspeita, e ela é sobre o meu próprio diff
///
/// O [`ph2d_field_eval::RegionCompiler`] chama o `specialised_profile` **por ladrilho × fatia de
/// profundidade** — `2 716` regiões a `1920×1080` com ladrilho `32`, `39 406` com ladrilho `8`. ⚠️ E
/// o ramo do torno por fórmula faz **ali** a extracção da silhueta (`128` alturas × as primitivas) e
/// **dois** ajustes de mínimos quadrados (`17×17`). *Se isso custar, o ganho da fita menor é pago de
/// volta na montagem, e foi por isso que o A/B de CPU leu a fórmula PIOR a `1400×900`.*
///
/// ⭐ O gémeo desta casa é o próprio `RegionCompiler`, que constrói a
/// [`ph2d_field_eval::profile_index::ProfileIndex`] **uma vez** e não por região, com a razão
/// escrita: *«construir o índice de um contorno de 168 arestas custa 0,2 ms, e um quadro pede uma
/// região por ladrilho — dezenas delas»*.
#[test]
#[ignore = "sonda de diagnóstico: mede o ajuste da fórmula por região"]
fn diag_o_custo_de_ajustar_a_formula() {
    const N: usize = 200;
    let doc = crate::smoke::scenes::vaso(ph2d_field::DEFAULT_PROFILE_RESOLUTION);
    let ph2d_field::NodeKind::Leaf(ph2d_field::Primitive::Revolve { profile }) =
        &doc.nodes()[0].kind
    else {
        panic!("a cena 5 é um Revolve");
    };
    let mut melhor = f64::MAX;
    for _ in 0..N {
        let t0 = std::time::Instant::now();
        let t = ph2d_field_eval::profile_formula::sd_revolve_por_formula(profile)
            .expect("o vaso desce por fórmula");
        let ms = t0.elapsed().as_secs_f64() * 1e3;
        std::hint::black_box(&t);
        melhor = melhor.min(ms);
    }
    // ⭐ E a mesma conta pela lei EXACTA, que é o que ela substitui na montagem.
    let mut exacta = f64::MAX;
    for _ in 0..N {
        let t0 = std::time::Instant::now();
        let t = ph2d_field_eval::profile::probe_sd_revolve_exacto(profile);
        let ms = t0.elapsed().as_secs_f64() * 1e3;
        std::hint::black_box(&t);
        exacta = exacta.min(ms);
    }
    println!(
        "\n  {}\n  montar a árvore do torno, mínimo de {N}:\n    por FÓRMULA {melhor:.4} ms · \
         pela lei EXACTA (a peça inteira) {exacta:.4} ms",
        super::super::contexto()
    );
    for (ladrilho, regioes) in [(64usize, 750usize), (32, 2716), (16, 10151), (8, 39406)] {
        println!(
            "    ladrilho {ladrilho:>2} px · {regioes:>6} regiões/quadro ⇒ \
             {:>8.1} ms só a ajustar",
            melhor * regioes as f64
        );
    }
    println!();
}

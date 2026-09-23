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

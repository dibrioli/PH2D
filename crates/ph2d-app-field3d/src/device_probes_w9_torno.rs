//! ⏱️⭐⭐⭐⭐ **O TECTO DA WAVE DO TORNO** — quanto da cena `5` é o contorno DESENROLADO, e quanto
//! sobra se ele custar nada.
//!
//! # Porque esta sonda vem ANTES de uma linha de WGSL
//!
//! A `W9` escreveu que tirar `~880` instruções ao torno vale *«mais do que os `31,96 ms` da cena
//! inteira»*, e escreveu-o a partir do **modelo de custo** (`0,039 ms` por instrução), ajustado
//! sobre `22` cenas. ⚠️ Um modelo ajustado sobre um corpus prevê a MÉDIA dele; ele não é uma
//! medição desta peça. *O tecto de um ganho mede-se, e a forma de o medir é fazer o custo
//! desaparecer.*
//!
//! # ⭐ A régua: a MESMA silhueta com o contorno mais grosso
//!
//! O contorno do vaso é cozido do desenho e sai em polilinha densa. Reamostrá-la de `k` em `k`
//! pontos deixa a **figura praticamente igual** — a marcha vê a mesma peça, com os mesmos passos
//! por acerto — e move **só** a contagem de primitivas. ⇒ a coluna do relógio contra a coluna das
//! instruções é a curva desta cena, e o limite dela em `k → ∞` é o tecto da wave.
//!
//! ⛔ **Não é o mesmo que baixar o `Resolution`**: aquele botão está no mínimo (`1`), e mexê-lo para
//! cima só acrescenta. A reamostragem é a única direcção que ESTE corpus permite descer.
//!
//! ⚠️ **O piso é uma peça ANALÍTICA** (um cilindro) e não um perfil de quatro pontos: o que se quer
//! saber é quanto a cena custa quando a folha não tem contorno nenhum.

/// Quantos pontos o contorno do vaso tem quando reamostrado de `k` em `k`.
fn vaso_reamostrado(k: usize) -> ph2d_field::Profile {
    let denso = crate::smoke::scenes::vaso(ph2d_field::DEFAULT_PROFILE_RESOLUTION);
    let ph2d_field::NodeKind::Leaf(ph2d_field::Primitive::Revolve { profile }) =
        &denso.nodes()[0].kind
    else {
        panic!("a cena 5 é um Revolve");
    };
    // ⚠️ **Só a POLILINHA**, de propósito: reamostrar arcos daria uma figura diferente, e o que esta
    // sonda varia é a CONTAGEM e não a forma.
    let contours: Vec<Vec<[f32; 2]>> = profile
        .contours()
        .iter()
        .map(|c| {
            let mut out: Vec<[f32; 2]> = c.iter().step_by(k.max(1)).copied().collect();
            // Um contorno com menos de três pontos não é uma figura.
            while out.len() < 3 {
                out.push(c[c.len() - out.len()]);
            }
            out
        })
        .collect();
    ph2d_field::Profile::new(contours, profile.fill(), profile.tolerance())
        .expect("a reamostragem de um perfil válido é um perfil válido")
}

/// O documento do torno com um perfil dado.
fn torno(profile: ph2d_field::Profile) -> ph2d_field::FieldDoc {
    ph2d_field::FieldDoc::new(
        vec![ph2d_field::Node {
            xform: ph2d_field::Xform::IDENTITY,
            kind: ph2d_field::NodeKind::Leaf(ph2d_field::Primitive::Revolve { profile }),
            mods: Vec::new(),
            verb: None,
        }],
        ph2d_field::NodeId(0),
    )
    .expect("um torno é um documento válido")
}

/// ⏱️⭐⭐⭐⭐ **Sonda: o tecto da wave do torno.**
///
/// Ela imprime, para a MESMA silhueta, a contagem de primitivas · as linhas de WGSL · os valores
/// vivos · o relógio do quadro de movimento (mínimo de [`super::super::QUADROS_MEDIDOS`]) · e os
/// passos por acerto, que é o CONTROLO de que a figura não mudou.
#[test]
#[ignore = "sonda de diagnóstico: mede o tecto da wave do torno"]
fn diag_o_tecto_da_wave_do_torno() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let materiais = [ph2d_material::OpenPbr::default().prepare()];
    let olhar = ph2d_view_transform::Look::default();
    const BG: [u8; 4] = [0, 0, 0, 0];
    let cam = ph2d_field_render::Orbit::default();
    let luz = [crate::gpu_frame::tests_lampada(&cam)];
    let reg = crate::smoke::sampled_registry();

    println!(
        "\n  {}\n  peça · primitivas · guardados · vivos · quadro ms · 1.ª ms",
        super::super::contexto()
    );

    let mut casos: Vec<(String, ph2d_field::FieldDoc)> = Vec::new();
    // ⭐ O PISO: uma peça analítica com a mesma silhueta grosseira.
    casos.push((
        "cilindro (piso)".to_string(),
        ph2d_field::FieldDoc::new(
            vec![ph2d_field::Node {
                xform: ph2d_field::Xform::IDENTITY,
                kind: ph2d_field::NodeKind::Leaf(ph2d_field::Primitive::Cylinder {
                    radius: 0.30,
                    half_height: 0.48,
                    round: 0.0,
                    chamfer: 0.0,
                }),
                mods: Vec::new(),
                verb: None,
            }],
            ph2d_field::NodeId(0),
        )
        .expect("o cilindro"),
    ));
    for k in [64usize, 32, 16, 8, 4, 2, 1] {
        let p = vaso_reamostrado(k);
        casos.push((format!("vaso de {k} em {k}"), torno(p)));
    }

    for (nome, doc) in &casos {
        let surfaces = ph2d_field_render::Surfaces {
            all: &materiais,
            owners: None,
        };
        let pinta = || {
            crate::gpu_frame::paint(
                t,
                doc,
                &reg,
                &cam,
                &luz,
                &surfaces,
                &ph2d_field_render::Presentation::of(olhar),
                BG,
                None,
                super::super::LW,
                super::super::LH,
                false,
            )
        };
        if pinta().is_none() {
            println!("  {nome:>20} ·        na CPU");
            continue;
        }
        let mut tempos = Vec::with_capacity(super::super::QUADROS_MEDIDOS);
        for _ in 0..super::super::QUADROS_MEDIDOS {
            let t0 = std::time::Instant::now();
            let _ = pinta().expect("o pintor");
            #[allow(clippy::cast_possible_truncation)]
            tempos.push(t0.elapsed().as_secs_f32() * 1e3);
        }
        let campo = ph2d_field_eval::device::DeviceField::new(doc, &reg).expect("a peça");
        let forma = campo.tape_shape().expect("a fita");
        let prim = match &doc.nodes()[0].kind {
            ph2d_field::NodeKind::Leaf(ph2d_field::Primitive::Revolve { profile }) => {
                profile.prim_count()
            }
            _ => 0,
        };
        let minimo = tempos.iter().copied().fold(f32::INFINITY, f32::min);
        println!(
            "  {nome:>20} · {prim:>10} · {:>9} · {:>5} · {minimo:>9.2} · {:>7.2}",
            forma.guardados, forma.vivos, tempos[0]
        );
    }
    println!();
}

/// ⏱️⭐⭐⭐⭐ **Sonda: QUANTAS LINHAS A PODA DEIXA — o número que decide a wave.**
///
/// # Porque esta sonda é uma CONTAGEM e não um relógio
///
/// A tabela da [`diag_o_tecto_da_wave_do_torno`] mostra o relógio **linear** em `0,028`–`0,031 ms`
/// por linha de WGSL ao longo de duas ordens de grandeza (`18` → `1 342`). ⭐ Isso é a assinatura de
/// **trabalho dinâmico por amostra**, não de tamanho de texto: se o custo fosse a ocupação, a curva
/// seria um degrau. ⇒ *uma consulta que troque texto por laço não compra nada; o que compra é a
/// PODA* — e a poda mede-se contando, o que não depende da carga da máquina.
///
/// # ⭐⭐⭐ A régua não tem uma única constante escolhida
///
/// A 1.ª redacção desta sonda somava `14` linhas por aresta de distância, `8` por aresta de sinal e
/// `22` por meia-lua — **três palpites**, e os três errados: o perfil da cena `5` paga `934 / 24 =`
/// **`38,9`** linhas por primitiva, contra as `26,7` de uma primitiva RECTA, porque metade delas são
/// ARCOS. ⇒ o que se conta aqui é a árvore **ESPECIALIZADA** que o produto já sabe construir
/// ([`ph2d_field_eval::RegionCompiler`]), passada pela mesma porta de fita que o dispositivo usa
/// ([`ph2d_field_eval::Field::tape_shape`]). *Zero constantes inventadas: é o número que uma consulta
/// de facto executaria naquela célula.*
///
/// ⚠️ **A população é uniforme na caixa da PEÇA e isso é uma escolha declarada**: a marcha não
/// amostra uniformemente (ela dá passos grandes longe da superfície), logo esta tabela
/// **sobre-representa** as células longe do contorno, que são as baratas. *A coluna do PIOR CASO é a
/// que não mente* — e é ela que tem de caber no orçamento.
#[test]
#[ignore = "sonda de diagnóstico: conta o que a poda do perfil deixa"]
fn diag_quantas_linhas_a_poda_deixa() {
    let doc = crate::smoke::scenes::vaso(ph2d_field::DEFAULT_PROFILE_RESOLUTION);
    let ph2d_field::NodeKind::Leaf(ph2d_field::Primitive::Revolve { profile }) =
        &doc.nodes()[0].kind
    else {
        panic!("a cena 5 é um Revolve");
    };
    let rc = ph2d_field_eval::RegionCompiler::new(&doc);
    assert!(rc.is_worth_it(), "o torno é uma forma de perfil");
    let inteira = ph2d_field_eval::Field::new(&doc)
        .tape_shape()
        .expect("a fita do torno")
        .guardados;
    let (plo, phi) = profile.bounds();
    println!(
        "\n  o perfil da cena 5: {} primitivas ({} arcos), caixa [{:.3} {:.3}]–[{:.3} {:.3}]",
        profile.prim_count(),
        profile.arc_count(),
        plo[0],
        plo[1],
        phi[0],
        phi[1],
    );
    println!("  a fita INTEIRA paga {inteira} linhas em toda amostra");
    // ⭐⭐⭐ **A população é a CÉLULA EM `(u, v)`, e não uma região de MUNDO.** Medido primeiro em
    // mundo, o pior caso ganhava só `2,5×` — e a causa é geométrica: `u = √(x² + z²)`, logo **toda**
    // região que toque o eixo vê a largura INTEIRA do perfil. Uma consulta por amostra não tem esse
    // problema: ela conhece o `u` do ponto. ⇒ a caixa de mundo usada aqui é **degenerada em `z`**
    // (`z ∈ [0, 0]`), que é o que faz `axis_gap` devolver exactamente o rectângulo pedido — *o
    // rectângulo `(u, v)` sai da porta do produto, e não de uma segunda cópia da conta*.
    let linhas_da_celula = |ulo: f32, uhi: f32, vlo: f32, vhi: f32| {
        let t = rc.compile(&doc, [ulo, vlo, 0.0], [uhi, vhi, 0.0]);
        ph2d_field_eval::Field::from_tree(&t)
            .tape_shape()
            .map_or(0, |s| s.guardados)
    };
    for (nome, ulo, uhi, vlo, vhi) in [
        ("a caixa do perfil", 0.0, phi[0], plo[1], phi[1]),
        (
            "a caixa da peça 2×",
            0.0,
            phi[0] * 2.0,
            plo[1] * 2.0,
            phi[1] * 2.0,
        ),
    ] {
        println!("  {nome} · lado · células · p50 · p90 · pior · ganho p50 · ganho pior");
        for lado in [4usize, 8, 16, 32, 64] {
            #[allow(clippy::cast_precision_loss)]
            let (pu, pv) = ((uhi - ulo) / lado as f32, (vhi - vlo) / lado as f32);
            let mut linhas = Vec::with_capacity(lado * lado);
            for iv in 0..lado {
                for iu in 0..lado {
                    #[allow(clippy::cast_precision_loss)]
                    let (a, b) = (ulo + iu as f32 * pu, vlo + iv as f32 * pv);
                    linhas.push(linhas_da_celula(a, a + pu, b, b + pv));
                }
            }
            linhas.sort_unstable();
            #[allow(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                clippy::cast_precision_loss
            )]
            let q = |f: f64| linhas[(((linhas.len() - 1) as f64) * f) as usize];
            let (p50, p90, pior) = (q(0.5), q(0.9), q(1.0));
            #[allow(clippy::cast_precision_loss)]
            let g = |n: usize| inteira as f32 / n.max(1) as f32;
            println!(
                "      {lado:>4} · {:>7} · {p50:>3} · {p90:>3} · {pior:>4} · {:>8.1}× · {:>10.1}×",
                lado * lado,
                g(p50),
                g(pior),
            );
        }
    }
    println!();
}

/// ⏱️⛔⛔ **Sonda: QUE CÉLULAS DEGENERAM para a fita inteira, e porquê.**
///
/// A [`diag_quantas_linhas_a_poda_deixa`] mede o pior caso a voltar a `931` linhas a partir de uma
/// grelha de `32×32`. O [`ph2d_field_eval::profile::sd_profile_in_region`] tem um degenerado com
/// esta nota escrita ao lado: *«um perfil cujo corte não deixou aresta nenhuma é **impossível** — a
/// regra do corte guarda sempre pelo menos a aresta que realiza o `dmax`»*.
///
/// ⚠️ **A nota fala do CORTE e o código tem DOIS filtros.** Depois do corte vem o `continue` da
/// costura do eixo, que o `Revolve` liga — e o corte não sabe dele. Esta sonda mede se é isso.
#[test]
#[ignore = "sonda de diagnóstico: acha as células que degeneram"]
fn diag_que_celulas_degeneram() {
    let doc = crate::smoke::scenes::vaso(ph2d_field::DEFAULT_PROFILE_RESOLUTION);
    let ph2d_field::NodeKind::Leaf(ph2d_field::Primitive::Revolve { profile }) =
        &doc.nodes()[0].kind
    else {
        panic!("a cena 5 é um Revolve");
    };
    let idx = ph2d_field_eval::profile_index::ProfileIndex::build(profile);
    let rc = ph2d_field_eval::RegionCompiler::new(&doc);
    let inteira = ph2d_field_eval::Field::new(&doc)
        .tape_shape()
        .expect("a fita")
        .guardados;
    let tol = profile.tolerance();
    let (plo, phi) = profile.bounds();
    let no_eixo = |i: u32| {
        let (a, b) = idx.edge(i);
        a[0].abs() <= tol && b[0].abs() <= tol
    };
    println!(
        "\n  tolerância {tol:.5} · {} das {} arestas assentam no eixo",
        (0..idx.edge_count() as u32).filter(|i| no_eixo(*i)).count(),
        idx.edge_count(),
    );
    const LADO: usize = 32;
    #[allow(clippy::cast_precision_loss)]
    let (pu, pv) = (phi[0] / LADO as f32, (phi[1] - plo[1]) / LADO as f32);
    let mut degeneradas = 0usize;
    let mut so_eixo = 0usize;
    let mut primeira: Option<String> = None;
    for iv in 0..LADO {
        for iu in 0..LADO {
            #[allow(clippy::cast_precision_loss)]
            let (a, b) = (iu as f32 * pu, plo[1] + iv as f32 * pv);
            let (clo, chi) = ([a, b], [a + pu, b + pv]);
            let t = rc.compile(&doc, [clo[0], clo[1], 0.0], [chi[0], chi[1], 0.0]);
            let linhas = ph2d_field_eval::Field::from_tree(&t)
                .tape_shape()
                .map_or(0, |s| s.guardados);
            if linhas < inteira {
                continue;
            }
            degeneradas += 1;
            let cortadas = idx.distance_edges(clo, chi);
            let todas_no_eixo = !cortadas.is_empty() && cortadas.iter().copied().all(no_eixo);
            if todas_no_eixo {
                so_eixo += 1;
            }
            if primeira.is_none() {
                primeira = Some(format!(
                    "u∈[{:.3} {:.3}] v∈[{:.3} {:.3}] · o corte deixou {:?} · todas no eixo: {}",
                    clo[0], chi[0], clo[1], chi[1], cortadas, todas_no_eixo
                ));
            }
        }
    }
    println!(
        "  de {} células, {degeneradas} pagam a fita INTEIRA ({inteira} linhas), e {so_eixo} \
         delas porque o corte só deixou arestas DO EIXO",
        LADO * LADO
    );
    if let Some(p) = primeira {
        println!("  a primeira: {p}");
    }
    println!();
}

/// ⏱️⭐⭐⭐⭐ **O custo de uma ARESTA lida contra dobrada** — ver o cabeçalho do [`aresta`].
#[path = "device_probes_w9_aresta.rs"]
mod aresta;

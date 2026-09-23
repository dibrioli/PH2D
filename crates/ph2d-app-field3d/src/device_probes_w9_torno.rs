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
        "\n  {}\n  peça · primitivas · guardados · vivos · quadro ms · 1.ª ms · passos/acerto",
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
        // ⭐⭐⭐ **QUANTAS VEZES CADA RAIO AVALIA O CAMPO** — sem esta coluna o relógio não se
        // atribui: um campo que é um mau minorante de distância faz TODO raio rastejar, e isso não
        // aparece em contagem de linhas nenhuma. ⚠️ Medida na CPU (o contador é dela) sobre uma tela
        // pequena — o que se quer é a RAZÃO, que não depende do tamanho.
        use std::sync::atomic::Ordering;
        ph2d_field_render::STEP_SAMPLES.store(0, Ordering::Relaxed);
        let g = ph2d_field_render::trace(doc, &reg, &cam, 96, 54);
        let acertos = g.hit.iter().filter(|h| **h).count().max(1);
        #[allow(clippy::cast_precision_loss)]
        let por_acerto =
            ph2d_field_render::STEP_SAMPLES.load(Ordering::Relaxed) as f32 / acertos as f32;
        println!(
            "  {nome:>20} · {prim:>10} · {:>9} · {:>5} · {minimo:>9.2} · {:>7.2} · {por_acerto:>8.1}",
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

/// ⏱️⭐⭐⭐⭐ **Sonda: quanto a poda compra na granularidade COERENTE POR GRUPO.**
///
/// A [`aresta::diag_o_custo_de_uma_aresta_lida_contra_dobrada`] mediu que uma consulta cuja lista
/// varia **por amostra** paga `12,1×`–`17,3×` por aresta, e que o desenho que sobrevive é **uma
/// lista por grupo de threads** — cujo imposto é `1,58×` num buffer `uniform`. ⇒ a pergunta é
/// quanto a poda ainda compra no **ladrilho do ecrã**, que é coerente por construção.
///
/// ⚠️ **A régua passa pela porta do produto** ([`ph2d_field_render::linhas_por_ladrilho_for_test`]),
/// que percorre a MESMA aritmética de região que a marcha percorre. *Reconstruí-la aqui mediria
/// outro programa.*
#[test]
#[ignore = "sonda de diagnóstico: conta as linhas por ladrilho"]
fn diag_a_poda_coerente_por_grupo() {
    let doc = crate::smoke::scenes::vaso(ph2d_field::DEFAULT_PROFILE_RESOLUTION);
    let reg = crate::smoke::sampled_registry();
    let cam = ph2d_field_render::Orbit::default();
    let inteira = ph2d_field_eval::Field::new(&doc)
        .tape_shape()
        .expect("a fita do torno")
        .guardados;
    println!("\n  a fita INTEIRA paga {inteira} linhas em toda amostra");
    println!(
        "  ladrilho px · fatias · regiões · p50 · p90 · pior · ganho p50 · ganho pior · \
         com o imposto de 1,58×"
    );
    for (tile, slabs) in [
        (64usize, 1usize),
        (64, 4),
        (32, 1),
        (32, 4),
        (16, 1),
        (16, 4),
        (8, 1),
        (8, 4),
    ] {
        let Some(mut linhas) = ph2d_field_render::linhas_por_ladrilho_for_test(
            &doc,
            &reg,
            &cam,
            super::super::LW,
            super::super::LH,
            tile,
            slabs,
        ) else {
            println!("  {tile:>11} · {slabs:>6} ·   sem regiões");
            continue;
        };
        let n = linhas.len();
        let mut v: Vec<usize> = linhas.drain(..).map(|(_, l)| l).collect();
        v.sort_unstable();
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss
        )]
        let q = |f: f64| v[(((v.len() - 1) as f64) * f) as usize];
        let (p50, p90, pior) = (q(0.5), q(0.9), q(1.0));
        #[allow(clippy::cast_precision_loss)]
        let g = |x: usize| inteira as f32 / x.max(1) as f32;
        println!(
            "  {tile:>11} · {slabs:>6} · {n:>7} · {p50:>4} · {p90:>4} · {pior:>4} · {:>8.1}× · \
             {:>10.1}× · p50 {:>4.1}× · pior {:>4.1}×",
            g(p50),
            g(pior),
            g(p50) / 1.58,
            g(pior) / 1.58,
        );
    }
    println!();
}

/// ⏱️⭐⭐⭐⭐ **O vaso por FÓRMULA** — ver o cabeçalho do [`formula`].
#[path = "device_probes_w9_formula.rs"]
mod formula;

/// ⏱️⛔⛔⛔ **Sonda: O ARRASTO NO MODO DE OMISSÃO — o caminho que o artista de facto toma.**
///
/// # O report do dono, e o defeito que ele apanhou
///
/// *«ao arrastar fica grosseiro ainda»* (2026-09-23), depois de a wave do torno por fórmula ter
/// medido `32,53 → 16,63 ms` e o divisor do prévio a cair de `2` para `1`.
///
/// ⛔⛔⛔ **Aquelas medições são todas do DISPOSITIVO, e o dispositivo só pinta em
/// [`crate::shading::Shading::Render`]** — o `#[default]` é `Matcap`, que é *«a omissão de um
/// modelador»*. ⇒ ao abrir a cena, o arrasto vai pelo traçado de **CPU**, e a cura foi medida num
/// caminho que ele não toma. ⚠️ *O cabeçalho do [`crate::smoke_draw_thread`] avisa desta classe de
/// defeito por escrito, três parágrafos acima da linha que a contém.*
///
/// Esta sonda mede o que ele vê: o traçado de CPU da cena `5`, A/B pela porta de bissecção, com o
/// **divisor que o produto escolheria** a partir da própria medição.
#[test]
#[ignore = "sonda de diagnóstico: mede o arrasto no modo de omissão"]
fn diag_o_arrasto_no_modo_de_omissao() {
    let doc = crate::smoke::scene(5);
    let reg = crate::smoke::sampled_registry();
    let cam = ph2d_field_render::Orbit::default();
    let linhas = ph2d_field_eval::Field::new(&doc)
        .tape_shape()
        .expect("a fita")
        .guardados;
    println!(
        "\n  {}\n  a fita desta corrida: {linhas} linhas (a lei exacta são 934)",
        super::super::contexto()
    );
    println!("  tela · CPU 1.ª · CPU mín · D(CPU) · PLACA mín · D(placa) · ganho");
    for (w, h) in [(1920u32, 1080u32), (1400, 900), (960, 540)] {
        let mut tempos = Vec::new();
        for _ in 0..super::super::QUADROS_MEDIDOS {
            let t0 = std::time::Instant::now();
            let _ = ph2d_field_render::trace(&doc, &reg, &cam, w, h);
            #[allow(clippy::cast_possible_truncation)]
            tempos.push(t0.elapsed().as_secs_f32() * 1e3);
        }
        let minimo = tempos.iter().copied().fold(f32::INFINITY, f32::min);
        // ⭐ **O divisor sai da PORTA DO PRODUTO** ([`crate::preview::preview_size`]) — reconstruí-lo
        // aqui mediria outra lei.
        let medida = crate::preview::Measured {
            pixels: u64::from(w) * u64::from(h),
            millis: minimo,
        };
        let (pw, _) = crate::preview::preview_size(
            (w, h),
            Some(medida),
            crate::preview::PREVIEW_BUDGET_MS,
            16,
        );
        // ⭐⭐⭐ **E O QUE A PLACA CUSTA NA ROTA DO PRODUTO** — o [`crate::gpu_frame::paint`], que
        // devolve a IMAGEM.
        //
        // ⚠️⚠️ **A 1.ª redacção media o [`crate::gpu_frame::march`] e isso era outra rota:** ele
        // traz o G-BUFFER de volta pelo barramento (`~50 MB` a `1920×1080`) e o Render só o pede
        // quando REFINA. Medido, ele lia `119`–`123 ms` onde o pintor lê `16,6` — *uma coluna de
        // uma rota que o produto não toma lê-se como o preço da placa.*
        let materiais = [ph2d_material::OpenPbr::default().prepare()];
        let surfaces = ph2d_field_render::Surfaces {
            all: &materiais,
            owners: None,
        };
        let olhar = ph2d_view_transform::Look::default();
        let placa = crate::gpu_frame::shared().and_then(|t| {
            let luz = [crate::gpu_frame::tests_lampada(&cam)];
            let mut melhor = f32::INFINITY;
            for _ in 0..super::super::QUADROS_MEDIDOS {
                let t0 = std::time::Instant::now();
                crate::gpu_frame::paint(
                    t,
                    &doc,
                    &reg,
                    &cam,
                    &luz,
                    &surfaces,
                    &ph2d_field_render::Presentation::of(olhar),
                    [0, 0, 0, 0],
                    None,
                    w,
                    h,
                    false,
                )?;
                #[allow(clippy::cast_possible_truncation)]
                let ms = t0.elapsed().as_secs_f32() * 1e3;
                melhor = melhor.min(ms);
            }
            Some(melhor)
        });
        let dp = placa.map_or(0, |ms| {
            let m = crate::preview::Measured {
                pixels: u64::from(w) * u64::from(h),
                millis: ms,
            };
            let (a, _) = crate::preview::preview_size(
                (w, h),
                Some(m),
                crate::preview::PREVIEW_BUDGET_MS,
                16,
            );
            (w / a.max(1)).max(1)
        });
        println!(
            "  {w}×{h} · {:>8.2} · {minimo:>7.2} · D={} · {:>9.2} · D={dp} · {:>5.1}×",
            tempos[0],
            (w / pw.max(1)).max(1),
            placa.unwrap_or(f32::NAN),
            minimo / placa.unwrap_or(f32::NAN),
        );
    }
    println!();
}

/// ⏱️⭐⭐⭐⭐ **Sonda: QUANTO DO QUADRO É O CAMPO, E QUANTO É A MARCHA.**
///
/// # A pergunta que decide se uma GRELHA ASSADA paga
///
/// A proposta da grelha de volume (o mecanismo do MagicaCSG) troca **avaliar o campo** por **uma
/// consulta trilinear**. ⇒ ela só paga se o campo for a maior parte do quadro. ⚠️ Se o que domina
/// for a MARCHA — os raios, os passos, a memória —, uma consulta mais barata por passo não move o
/// relógio, e a wave inteira seria construída contra a grandeza errada.
///
/// ⭐ **A régua é a mesma cena com peças de tamanhos de fita muito diferentes**, no motor de
/// **CPU** (o caminho do modo de omissão) e no **dispositivo**, com os passos por acerto ao lado —
/// sem eles, um relógio que não se move lê-se como *«o campo não custa»* quando pode ser *«a peça
/// mais simples dá mais passos»*.
#[test]
#[ignore = "sonda de diagnóstico: mede quanto do quadro é o campo"]
fn diag_quanto_do_quadro_e_o_campo() {
    let reg = crate::smoke::sampled_registry();
    let cam = ph2d_field_render::Orbit::default();
    const W: u32 = 1920;
    const H: u32 = 1080;
    let pecas: Vec<(&str, ph2d_field::FieldDoc)> = vec![
        (
            "esfera",
            ph2d_field::FieldDoc::new(
                vec![ph2d_field::Node {
                    xform: ph2d_field::Xform::IDENTITY,
                    kind: ph2d_field::NodeKind::Leaf(ph2d_field::Primitive::Sphere { radius: 0.5 }),
                    mods: Vec::new(),
                    verb: None,
                }],
                ph2d_field::NodeId(0),
            )
            .expect("a esfera"),
        ),
        ("o vaso (cena 5)", crate::smoke::scene(5)),
        ("o nó de toro (cena 28)", crate::smoke::scene(28)),
    ];
    println!(
        "\n  {}\n  peça · linhas · passos/acerto · CPU ms · ns/amostra · placa ms · ns/amostra",
        super::super::contexto()
    );
    for (nome, doc) in &pecas {
        let linhas = ph2d_field_eval::Field::new(doc)
            .tape_shape()
            .map_or(0, |s| s.guardados);
        use std::sync::atomic::Ordering;
        ph2d_field_render::STEP_SAMPLES.store(0, Ordering::Relaxed);
        let g = ph2d_field_render::trace(doc, &reg, &cam, 96, 54);
        let acertos = g.hit.iter().filter(|h| **h).count().max(1);
        #[allow(clippy::cast_precision_loss)]
        let por_acerto =
            ph2d_field_render::STEP_SAMPLES.load(Ordering::Relaxed) as f32 / acertos as f32;
        let mut cpu = f32::INFINITY;
        for _ in 0..super::super::QUADROS_MEDIDOS {
            let t0 = std::time::Instant::now();
            let _ = ph2d_field_render::trace(doc, &reg, &cam, W, H);
            #[allow(clippy::cast_possible_truncation)]
            let ms = t0.elapsed().as_secs_f32() * 1e3;
            cpu = cpu.min(ms);
        }
        // ⚠️ **As amostras são `pixels × passos por acerto`** — é a conta que o próprio gate do
        // corpus usa, e ela é a única que torna dois relógios comparáveis entre peças.
        #[allow(clippy::cast_precision_loss)]
        let amostras = (f64::from(W) * f64::from(H) * f64::from(por_acerto)).max(1.0);
        // ⭐⭐⭐ **A COLUNA DA PLACA É A ROTA DO PRODUTO — o passe que devolve a IMAGEM.**
        //
        // ⛔⛔ **A 1.ª redacção media o [`crate::gpu_frame::march`], e era a SEGUNDA vez que esta
        // família o fazia** (a sonda irmã `diag_o_arrasto_no_modo_de_omissao` já carrega a mesma
        // correcção escrita ao lado dela). Ele traz o G-BUFFER de volta pelo barramento (`~50 MB` a
        // `1920×1080`) e o produto só o pede quando REFINA: medido, ele lia `90`–`177 ms` onde o
        // passe que pinta lê `13`. ⇒ *uma coluna de uma rota que o produto não toma lê-se como o
        // preço da placa, e neste probe ela decidiria uma WAVE.*
        //
        // ⭐ **E a rota certa aqui é o MATCAP**, porque a pergunta é sobre o quadro que o artista vê
        // no modo de omissão — ver [`crate::gpu_frame::pinta_matcap`].
        let (lado, foto) = crate::smoke::matcap_para_sonda();
        let olhar = ph2d_view_transform::Look::default();
        let placa = crate::gpu_frame::shared().and_then(|t| {
            let mut melhor = f32::INFINITY;
            for _ in 0..super::super::QUADROS_MEDIDOS {
                let t0 = std::time::Instant::now();
                crate::gpu_frame::pinta_matcap(
                    t,
                    doc,
                    &reg,
                    &cam,
                    &ph2d_field_gpu::matcap::MatcapSetup {
                        rgb_linear: &foto,
                        side: lado,
                        chave: 1,
                        stops: olhar.exposure_stops,
                        view: ph2d_view_transform::wgsl::view_code(olhar.view),
                        background: [0, 0, 0, 0],
                    },
                    W,
                    H,
                )?;
                #[allow(clippy::cast_possible_truncation)]
                let ms = t0.elapsed().as_secs_f32() * 1e3;
                melhor = melhor.min(ms);
            }
            Some(melhor)
        });
        let ns = |ms: f32| f64::from(ms) * 1.0e6 / amostras;
        println!(
            "  {nome:>22} · {linhas:>6} · {por_acerto:>13.1} · {cpu:>6.1} · {:>10.3} · {:>8.1} · \
             {:>10.3}",
            ns(cpu),
            placa.unwrap_or(f32::NAN),
            ns(placa.unwrap_or(f32::NAN)),
        );
    }
    println!();
}

/// ⏱️⭐⭐⭐⭐ **O QUADRO DO MODO DE OMISSÃO, NOS DOIS MOTORES** — a resposta ao report
/// *«ao arrastar fica grosseiro ainda»*.
///
/// # ⚠️ Porque este probe existe ao lado do [`diag_o_arrasto_no_modo_de_omissao`]
///
/// Aquele mede a lei do **MATERIAL** (CPU `trace` contra `gpu_frame::paint`), e o caminho que o
/// artista de facto toma é o **matcap** — o `#[default]` do [`crate::shading::Shading`]. ⛔ *Uma
/// coluna de uma lei que o produto não corre no modo de omissão lê-se como o preço daquele modo.*
///
/// ⚠️⚠️ **A coluna da CPU tem de levar o SOMBREAMENTO:** o `trace` sozinho devolve o G-buffer, e o
/// modo de omissão paga também o [`ph2d_field_render::shade_with`], que corre em todos os núcleos
/// (`par_chunks_mut`). *Medir só a marcha entregaria um número que o quadro nunca teve.*
///
/// ⭐ **O divisor sai da PORTA DO PRODUTO** ([`crate::preview::preview_size`]) nas duas colunas —
/// é ele que o dono vê como «grosseiro»: `D=3` são **um nono** dos píxeis.
///
/// ```text
/// PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-field3d --lib --release -- \
///   --ignored --exact device_probes::w9::torno::diag_o_quadro_do_matcap --nocapture
/// ```
#[test]
#[ignore = "sonda de relógio; precisa de adaptador e de máquina calma"]
fn diag_o_quadro_do_matcap() {
    let doc = crate::smoke::scene(5);
    let reg = crate::smoke::sampled_registry();
    let cam = ph2d_field_render::Orbit::default();
    let olhar = ph2d_view_transform::Look::default();
    // ⚠️ **A fotografia é a MESMA que o smoke carrega** — ver [`crate::smoke::matcap_para_sonda`].
    // ⛔ Um matcap sintético aqui mediria outro tamanho de armazém e outra aritmética de índice.
    let (lado, rgb) = crate::smoke::matcap_para_sonda();
    println!(
        "\n  {}\n  o matcap desta corrida: lado {lado} ({} texels)",
        super::super::contexto(),
        (lado as usize) * (lado as usize)
    );
    println!("  tela · CPU mín · D(CPU) · PLACA mín · D(placa) · ganho");
    for (w, h) in [(1920u32, 1080u32), (1400, 900), (960, 540)] {
        // ⭐ **O QUADRO INTEIRO da CPU: marchar E sombrear**, que é o que o modo de omissão custa.
        let mut cpu = f32::INFINITY;
        for _ in 0..super::super::QUADROS_MEDIDOS {
            let t0 = std::time::Instant::now();
            let g = ph2d_field_render::trace(&doc, &reg, &cam, w, h);
            let _ = ph2d_field_render::shade_with(
                &g,
                &ph2d_field_render::Matcap {
                    side: lado,
                    rgb_linear: &rgb,
                },
                olhar,
                [0, 0, 0, 0],
            );
            #[allow(clippy::cast_possible_truncation)]
            let ms = t0.elapsed().as_secs_f32() * 1e3;
            cpu = cpu.min(ms);
        }
        let divisor = |ms: f32| {
            let medida = crate::preview::Measured {
                pixels: u64::from(w) * u64::from(h),
                millis: ms,
            };
            let (pw, _) = crate::preview::preview_size(
                (w, h),
                Some(medida),
                crate::preview::PREVIEW_BUDGET_MS,
                16,
            );
            (w / pw.max(1)).max(1)
        };
        let placa = crate::gpu_frame::shared().and_then(|t| {
            let mut melhor = f32::INFINITY;
            for _ in 0..super::super::QUADROS_MEDIDOS {
                let t0 = std::time::Instant::now();
                crate::gpu_frame::pinta_matcap(
                    t,
                    &doc,
                    &reg,
                    &cam,
                    &ph2d_field_gpu::matcap::MatcapSetup {
                        rgb_linear: &rgb,
                        side: lado,
                        chave: 1,
                        stops: olhar.exposure_stops,
                        view: ph2d_view_transform::wgsl::view_code(olhar.view),
                        background: [0, 0, 0, 0],
                    },
                    w,
                    h,
                )?;
                #[allow(clippy::cast_possible_truncation)]
                let ms = t0.elapsed().as_secs_f32() * 1e3;
                melhor = melhor.min(ms);
            }
            Some(melhor)
        });
        let p = placa.unwrap_or(f32::NAN);
        println!(
            "  {w}×{h} · {cpu:>8.2} · D={} · {p:>9.2} · D={} · {:>5.2}×",
            divisor(cpu),
            divisor(p),
            cpu / p,
        );
    }
    println!();
}

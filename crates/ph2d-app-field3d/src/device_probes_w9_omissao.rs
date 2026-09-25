//! ⏱️⭐⭐⭐⭐ **O MODO DE OMISSÃO — o arrasto, o campo contra a marcha, e o quadro do matcap nos dois
//! motores.** Saíram do [`super::torno`] por um tecto de LOC (`767` contra `700`), e a fronteira é a
//! do assunto: o irmão mede o TORNO (o contorno desenrolado), estas medem o caminho que o artista
//! toma ao abrir a cena — o matcap, que é o `#[default]` do `Shading`.

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

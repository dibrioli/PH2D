//! ⭐⭐ **Quanto o preview CUSTA** — as sondas, irmãs dos gates em
//! [`field3d_preview_tests`](super::tests).
//!
//! ⚠️ **O corte é por ASSUNTO e nasceu de um teto:** o irmão passou os `600` LOC do HR-18, e o que
//! saiu foi o que responde *«quanto custa?»* — os gates que respondem *«a lei vale?»* ficaram lá.
//! Uma sonda traz consigo a tabela medida e o comando que a re-corre, e é isso que a faz crescer.

// ⚠️ **A resolução cheia é a MESMA dos gates** — uma cópia local seria duas respostas para
// *«qual é o alvo?»*, e a que envelhecesse seria a deste ficheiro.
use super::tests::FULL;

/// ⭐⭐⭐ **O QUE UM PERFIL FINO CUSTA EM CADA DIVISOR** — a coluna que a [`MEASURED_MS`] não tinha.
///
/// ⛔ **A tabela media duas cenas até `D=4`**, e o [`measured_cost`] escolhe a **linha mais
/// próxima** — então `D=5..8` recebiam todos o número do `D=4`. Enquanto o piso era `3` isso nunca
/// importou; quando o `MAX_PROFILE_RESOLUTION` subiu, o laço passou a precisar de divisores que a
/// tabela **não sabia medir**. *Uma tabela que satura na última linha responde a todas as perguntas
/// e só acerta nas que já sabia.*
///
/// ⚠️ A peça é uma extrusão no **teto** do `Resolution` — o pior caso que o produto consegue pedir.
///
/// ```text
/// cargo test -p ph2d-host-desktop --release --bin ph2d-host-desktop -- \
///     --ignored --nocapture measure_a_fine_profile_at_every_divisor
/// ```
#[test]
#[ignore = "sonda; roda com --ignored --nocapture"]
fn measure_a_fine_profile_at_every_divisor() {
    use ph2d_field::{FieldDoc, FillRule, NodeId, Primitive, Profile, Xform};
    // ⭐ **O PISO por nível**: o que a pré-visualização não consegue baixar, por mais grossa que
    // fique — medido em D=6, que a varredura achou ser o fundo (abaixo dele o custo volta a subir).
    println!("nivel | arestas | D=1 | D=3 | D=6 (o PISO)");
    for level in [1u32, 8, 16, 32, 64] {
        // ⛔ **A 1.ª versão desta fixtura construía um polígono de 168 pontos e passava a
        // tolerância como METADADO** — o `Profile::new` recebe uma polilinha **já achatada**, então
        // todos os níveis saíam com as mesmas 168 arestas e a tabela não media o nível nenhum.
        // *Uma fixtura que não contém o fenómeno mede outra coisa.* ⇒ aqui o contorno é **cozido**
        // de uma curva, que é o que o produto faz.
        let prof = {
            use ph2d_vec_scene::{VecPath, VecVertex};
            let (k, r) = (0.552_284_75_f64, 0.6_f64);
            let mut verts = Vec::new();
            for (i, (ax, ay)) in [(r, 0.0), (0.0, r), (-r, 0.0), (0.0, -r)]
                .into_iter()
                .enumerate()
            {
                let (tx, ty) = [(0.0, k * r), (-k * r, 0.0), (0.0, -k * r), (k * r, 0.0)][i];
                verts.push(VecVertex {
                    in_handle: [ax - tx, ay - ty],
                    out_handle: [ax + tx, ay + ty],
                    ..VecVertex::corner([ax, ay])
                });
            }
            let path = VecPath {
                verts,
                closed: true,
                ..VecPath::default()
            };
            // ⚠️ **Por fora da trava**, como a sonda do teto: a pergunta é o que cada nível CUSTA,
            // e passar pelo clamp mediria o teto de hoje em todas as linhas.
            let tol = ph2d_field_profile::span_of(&path.cooked())
                * ph2d_field_profile::TOLERANCE_RATIO
                / f64::from(level);
            ph2d_field_profile::cook_path(&path, tol).expect("perfil")
        };
        let arestas = prof.segment_count();
        let d = FieldDoc::new(
            vec![ph2d_field_eval::leaf(
                Primitive::Extrude {
                    profile: prof,
                    half_height: 0.4,
                    round: 0.06,
                    chamfer: 0.0,
                },
                Xform::IDENTITY,
            )],
            NodeId(0),
        )
        .expect("extrusão");
        let r = ph2d_field_eval::hybrid::Registry::new();
        let c = ph2d_field_render::Orbit::default();
        let mut row = format!("{level:5} | {arestas:7} |");
        for dv in [1u32, 3, 6] {
            let (w, h) = (FULL.0 / dv, FULL.1 / dv);
            let _ = ph2d_field_render::trace(&d, &r, &c, w, h);
            let mut v: Vec<f64> = (0..3)
                .map(|_| {
                    let t = std::time::Instant::now();
                    let _ = ph2d_field_render::trace(&d, &r, &c, w, h);
                    t.elapsed().as_secs_f64() * 1000.0
                })
                .collect();
            v.sort_by(f64::total_cmp);
            row.push_str(&format!(" {:7.1} |", v[1]));
        }
        println!("{row}");
    }
    let n = 168usize;
    let contour: Vec<[f32; 2]> = (0..n)
        .map(|i| {
            let a = std::f64::consts::TAU * (i as f64) / (n as f64);
            [(0.6 * a.cos()) as f32, (0.6 * a.sin()) as f32]
        })
        .collect();
    // A tolerância do TETO, pela porta do produto.
    let tol =
        ph2d_field_profile::TOLERANCE_RATIO / f64::from(ph2d_field::MAX_PROFILE_RESOLUTION) * 1.2;
    let profile = Profile::new(vec![contour], FillRule::NonZero, tol as f32).expect("perfil");
    let doc = FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            Primitive::Extrude {
                profile,
                half_height: 0.4,
                round: 0.06,
                chamfer: 0.0,
            },
            Xform::IDENTITY,
        )],
        NodeId(0),
    )
    .expect("extrusão");
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let cam = ph2d_field_render::Orbit::default();
    println!("divisor | {}x{} | ms", FULL.0, FULL.1);
    for d in [1u32, 3, 6] {
        let (w, h) = (FULL.0 / d, FULL.1 / d);
        let _ = ph2d_field_render::trace(&doc, &reg, &cam, w, h);
        let mut v: Vec<f64> = (0..3)
            .map(|_| {
                let t = std::time::Instant::now();
                let _ = ph2d_field_render::trace(&doc, &reg, &cam, w, h);
                t.elapsed().as_secs_f64() * 1000.0
            })
            .collect();
        v.sort_by(f64::total_cmp);
        println!("{d:7} | {w:4}x{h:4} | {:8.1}", v[1]);
    }
}

/// ⭐⭐⭐ **QUANTO A CURA COMPRA** — o traçado de movimento, com e sem o contorno engrossado.
///
/// ⚠️ **As duas configurações no MESMO processo, por mediana** — a lição que a W64 pagou: subtrair
/// dois relógios de corridas separadas dá a soma dos dois ruídos.
///
/// ```text
/// cargo test -p ph2d-host-desktop --release --bin ph2d-host-desktop -- \
///     --ignored --nocapture measure_what_the_coarse_contour_buys
/// ```
#[test]
#[ignore = "sonda; roda com --ignored --nocapture"]
fn measure_what_the_coarse_contour_buys() {
    use ph2d_field::{FieldDoc, FillRule, NodeId, Primitive, Profile, Xform};
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let cam = ph2d_field_render::Orbit::default();
    let median = |mut v: Vec<f64>| -> f64 {
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    };
    println!("arestas | pedido | sem a cura | com a cura | ganho");
    for n in [168usize, 472, 940] {
        let contour: Vec<[f32; 2]> = (0..n)
            .map(|i| {
                let a = std::f64::consts::TAU * (i as f64) / (n as f64);
                [(0.5 * a.cos()) as f32, (0.5 * a.sin()) as f32]
            })
            .collect();
        let profile = Profile::new(vec![contour], FillRule::NonZero, 1e-4).expect("perfil");
        let doc = FieldDoc::new(
            vec![ph2d_field::Node {
                xform: Xform::IDENTITY,
                kind: ph2d_field::NodeKind::Leaf(Primitive::Extrude {
                    profile,
                    half_height: 0.4,
                    round: 0.06,
                    chamfer: 0.0,
                }),
                mods: Vec::new(),
                verb: None,
            }],
            NodeId(0),
        )
        .expect("extrusão");
        let asked = (640u32, 360u32);
        let grosso = super::coarse_doc(&doc, true).unwrap_or_else(|| doc.clone());
        for d in [&doc, &grosso] {
            let _ = ph2d_field_render::trace(d, &reg, &cam, asked.0, asked.1);
        }
        let mut sem = Vec::new();
        let mut com = Vec::new();
        for _ in 0..5 {
            let t = std::time::Instant::now();
            let _ = ph2d_field_render::trace(&doc, &reg, &cam, asked.0, asked.1);
            sem.push(t.elapsed().as_secs_f64() * 1000.0);
            let t = std::time::Instant::now();
            let _ = ph2d_field_render::trace(&grosso, &reg, &cam, asked.0, asked.1);
            com.push(t.elapsed().as_secs_f64() * 1000.0);
        }
        let (a, b) = (median(sem), median(com));
        println!(
            "{n:7} | {}x{} | {a:10.1} | {b:10.1} | {:5.2}x",
            asked.0,
            asked.1,
            a / b
        );
    }
}

/// ⭐⭐⭐ **ONDE O QUADRO ESTÁ, DEPOIS DE TUDO** (W86) — a pergunta que abriu o item 1 do Enio.
///
/// Ela mede o **ciclo inteiro** com o produto de hoje: o quadro de movimento (contorno engrossado a
/// `1,0°` de erro, sem anti-serrilhado, com a cache), e os dois degraus do assentar (contorno a
/// `0,5°`, com anti-serrilhado).
///
/// ⚠️ **A cache é AQUECIDA com um arrasto antes de medir** — é assim que ela chega a qualquer quadro
/// que o artista veja depois do primeiro, e medir a frio mediria a abertura do painel.
///
/// ⚠️ Precisa da máquina a `load < 5`.
///
/// ```text
/// cargo test -p ph2d-host-desktop --profile ci-test -- --exact \
///     field3d_preview::cost_tests::measure_where_the_frame_stands_after_all_of_it \
///     --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda; roda com --ignored --nocapture"]
fn measure_where_the_frame_stands_after_all_of_it() {
    use ph2d_field::{FieldDoc, FillRule, NodeId, Primitive, Profile, Xform};
    use ph2d_field_render::{Orbit, TapeCache};
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let med = |mut v: Vec<f64>| -> f64 {
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    };
    for autoral in [168usize, 940] {
        let contour: Vec<[f32; 2]> = (0..autoral)
            .map(|i| {
                let a = std::f64::consts::TAU * (i as f64) / (autoral as f64);
                [(0.6 * a.cos()) as f32, (0.6 * a.sin()) as f32]
            })
            .collect();
        let doc = FieldDoc::new(
            vec![ph2d_field::Node {
                xform: Xform::IDENTITY,
                kind: ph2d_field::NodeKind::Leaf(Primitive::Extrude {
                    profile: Profile::new(vec![contour], FillRule::NonZero, 1e-4).expect("perfil"),
                    half_height: 0.4,
                    round: 0.06,
                    chamfer: 0.0,
                }),
                mods: Vec::new(),
                verb: None,
            }],
            NodeId(0),
        )
        .expect("extrusão");
        let movimento = super::coarse_doc(&doc, true).unwrap_or_else(|| doc.clone());
        let assente = super::coarse_doc(&doc, false).unwrap_or_else(|| doc.clone());
        let cache = TapeCache::new();
        let arestas = |d: &FieldDoc| -> usize {
            d.nodes()
                .iter()
                .filter_map(|n| match &n.kind {
                    ph2d_field::NodeKind::Leaf(Primitive::Extrude { profile, .. }) => {
                        Some(profile.segment_count())
                    }
                    _ => None,
                })
                .sum()
        };
        println!(
            "--- contorno autoral {autoral} · movimento {} · assente {} ---",
            arestas(&movimento),
            arestas(&assente)
        );
        // Aquecimento: um arrasto inteiro, como o artista faz antes de parar.
        for i in 0..8 {
            let cam = Orbit {
                rotation: Orbit::from_yaw_pitch(0.72 + (i as f32) * 2.0f32.to_radians(), 0.52)
                    .rotation,
                ..Orbit::default()
            };
            let _ = ph2d_field_render::trace_cached_for_test(
                &movimento,
                &reg,
                &cam,
                640,
                360,
                false,
                Some(&cache),
            );
        }
        let cam = Orbit::default();
        let casos: [(&str, &FieldDoc, u32, u32, bool); 3] = [
            ("movimento (640x360, sem AA)", &movimento, 640, 360, false),
            ("assentar 1 (640x360, com AA)", &assente, 640, 360, true),
            ("assentar 2 (1280x720, com AA)", &assente, 1280, 720, true),
        ];
        for (nome, d, w, h, aa) in casos {
            let _ = ph2d_field_render::trace_cached_for_test(d, &reg, &cam, w, h, aa, Some(&cache));
            let ms = med((0..5)
                .map(|_| {
                    let t0 = std::time::Instant::now();
                    let _ = ph2d_field_render::trace_cached_for_test(
                        d,
                        &reg,
                        &cam,
                        w,
                        h,
                        aa,
                        Some(&cache),
                    );
                    t0.elapsed().as_secs_f64() * 1000.0
                })
                .collect());
            // O orçamento de 60 fps é `16,7 ms`, e ele só se aplica ao quadro de MOVIMENTO.
            println!(
                "{nome:32} | {ms:8.2} ms | {:5.2} do orçamento de 16,7",
                ms / 16.7
            );
        }
    }
}

/// ⭐⭐⭐ **A TRAVADINHA QUE UMA MÃO QUE HESITA PAGA** (W89) — o report do Enio de 26/08:
/// *«de tempos em tempos dá pequenas travadinhas»*.
///
/// # ⚠️ Porque nenhuma sonda deste módulo a podia ver
///
/// Todas as outras medem uma **mediana** de traçados do mesmo tipo. Uma travadinha periódica é um
/// facto de **cauda** e de **sequência**: ela não existe dentro de um tipo de quadro, existe na
/// TRANSIÇÃO entre eles. *Uma bancada que mede um arrasto contínuo não pode ver o que só acontece
/// quando a mão pára um instante* — a mesma cegueira que a [`super::super::field3d_preview`] já
/// pagou na cache (ver `DOCS`).
///
/// # A régua é o ATRASO DA IMAGEM, em milissegundos
///
/// O que o artista vê não é o custo de um quadro: é **quão velha** é a pose na tela. Cada traçado
/// publica a imagem da câmera **do instante em que foi pedido**, então o atraso é
/// `agora − instante do pedido da imagem exposta`. Uma travadinha é um pico nessa curva.
///
/// # O modelo
///
/// Os **custos são medidos** (traçado real, com a cache quente, mediana de 3) e a **decisão é
/// simulada** pelo laço verdadeiro ([`super::next_trace`] + [`super::cancels_the_inflight`]),
/// avaliado a cada quadro de ecrã como o `field3d_smoke_draw::draw` faz. ⇒ o A/B entre as duas leis
/// é **determinístico**, que é a única forma honesta de o ler nesta workstation (`load` 3–26).
///
/// ```text
/// cargo test -p ph2d-host-desktop --profile ci-test -- --exact \
///     field3d_preview::cost_tests::measure_the_stall_a_hesitating_hand_pays \
///     --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda; roda com --ignored --nocapture"]
fn measure_the_stall_a_hesitating_hand_pays() {
    use ph2d_field::{FieldDoc, FillRule, NodeId, Primitive, Profile, Xform};
    use ph2d_field_render::{Orbit, TapeCache};
    /// O quadro de ecrã: é a ele que o `draw` acorda, e a imagem não pode envelhecer mais devagar.
    const QUADRO: f64 = 1000.0 / 60.0;
    /// A área do canvas do MODEL nesta bancada.
    const CHEIO: (u32, u32) = (1280, 720);
    /// A que velocidade a mão arrasta — `90°/s` é o número que a cache já usa.
    const TAXA: f64 = 90.0;
    let reg = ph2d_field_eval::hybrid::Registry::new();
    for autoral in [168usize, 940] {
        let contour: Vec<[f32; 2]> = (0..autoral)
            .map(|i| {
                let a = std::f64::consts::TAU * (i as f64) / (autoral as f64);
                [(0.6 * a.cos()) as f32, (0.6 * a.sin()) as f32]
            })
            .collect();
        let doc = FieldDoc::new(
            vec![ph2d_field::Node {
                xform: Xform::IDENTITY,
                kind: ph2d_field::NodeKind::Leaf(Primitive::Extrude {
                    profile: Profile::new(vec![contour], FillRule::NonZero, 1e-4).expect("perfil"),
                    half_height: 0.4,
                    round: 0.06,
                    chamfer: 0.0,
                }),
                mods: Vec::new(),
                verb: None,
            }],
            NodeId(0),
        )
        .expect("extrusão");
        let movimento = super::coarse_doc(&doc, true).unwrap_or_else(|| doc.clone());
        let assente = super::coarse_doc(&doc, false).unwrap_or_else(|| doc.clone());
        let cache = TapeCache::new();
        let camera = |yaw: f64| Orbit {
            rotation: Orbit::from_yaw_pitch(0.72 + yaw.to_radians() as f32, 0.52).rotation,
            ..Orbit::default()
        };
        // Aquecimento: um arrasto inteiro, como o artista faz antes de hesitar.
        for i in 0..8 {
            let _ = ph2d_field_render::trace_cached_for_test(
                &movimento,
                &reg,
                &camera(f64::from(i) * 2.0),
                640,
                360,
                false,
                Some(&cache),
            );
        }
        // ⚠️ O custo de cada TIPO de traçado, medido uma vez e reutilizado pelas duas leis — é isto
        // que tira o relógio da comparação.
        let mut memo: Vec<((u32, u32, bool), f64)> = Vec::new();
        let mut custo = |w: u32, h: u32, coarse: bool| -> f64 {
            if let Some((_, ms)) = memo.iter().find(|(k, _)| *k == (w, h, coarse)) {
                return *ms;
            }
            let d = if coarse { &movimento } else { &assente };
            let mut v: Vec<f64> = (0..3)
                .map(|_| {
                    let t0 = std::time::Instant::now();
                    let _ = ph2d_field_render::trace_cached_for_test(
                        d,
                        &reg,
                        &camera(0.0),
                        w,
                        h,
                        !coarse,
                        Some(&cache),
                    );
                    t0.elapsed().as_secs_f64() * 1000.0
                })
                .collect();
            v.sort_by(f64::total_cmp);
            memo.push(((w, h, coarse), v[1]));
            v[1]
        };
        println!(
            "--- contorno autoral {autoral} · área {}x{} ---",
            CHEIO.0, CHEIO.1
        );
        println!(
            "pausa da mão | LEI DE HOJE erro angular máx | LEI NOVA erro angular máx | refinamentos começados/abandonados"
        );
        for pausa_ms in [0.0f64, 17.0, 34.0, 68.0, 136.0] {
            let mut linha = [(0.0f64, 0usize, 0usize); 2];
            for (li, lei_nova) in [false, true].into_iter().enumerate() {
                // O gesto: arrasta 300 ms, hesita `pausa_ms`, arrasta mais 300 ms.
                let (t_pausa, fim) = (300.0, 600.0 + pausa_ms);
                let angulo = |t: f64| -> f64 {
                    let parado = (t - t_pausa).clamp(0.0, pausa_ms);
                    TAXA * (t - parado) / 1000.0
                };
                let mexe = |t: f64| !(t_pausa..t_pausa + pausa_ms).contains(&t);
                let mut requested: Option<(Orbit, u32, u32, FieldDoc, bool)> = None;
                let mut measured: Option<super::Measured> = None;
                // (fim, instante do pedido, w, h, coarse)
                let mut voo: Option<(f64, f64, u32, u32, bool)> = None;
                let mut exposta: Option<f64> = None;
                // ⚠️ **O BOOT não conta, e a 1.ª versão desta sonda media-o.** O primeiro traçado é
                // sempre CHEIO por lei (é ele a medição que fecha o laço), e a `183 ms` ele
                // dominava o máximo de TODAS as linhas da tabela — as duas leis imprimiam o mesmo
                // número e a sonda parecia dizer que a lei não importa. *Uma sonda que mede a
                // abertura do painel não pode responder sobre o meio de um arrasto.*
                let mut publicadas = 0usize;
                let (mut atraso_max, mut comecados, mut abandonados) = (0.0f64, 0usize, 0usize);
                let mut t = 0.0f64;
                while t <= fim {
                    if let Some((f, pedido, w, h, _)) = voo
                        && f <= t
                    {
                        exposta = Some(pedido);
                        publicadas += 1;
                        measured = Some(super::Measured {
                            pixels: u64::from(w) * u64::from(h),
                            millis: (f - pedido) as f32,
                        });
                        voo = None;
                    }
                    let cam = camera(angulo(t));
                    let ask = super::next_trace(
                        requested.as_ref().map(|(c, w, h, d, k)| (c, *w, *h, d, *k)),
                        &cam,
                        &doc,
                        CHEIO,
                        measured,
                        exposta.is_some(),
                        16,
                    );
                    if let (Some((_, _, jw, jh, jc)), Some((aw, ah, ac))) = (voo, ask) {
                        let corta = if lei_nova {
                            // ⭐ A lei que ship desde a W89: um refinamento cede à mão.
                            super::cancels_the_inflight(!jc, ac)
                        } else {
                            // ⛔ A lei ANTIGA, guardada aqui para a coluna do A/B: ela perguntava
                            // ao TAMANHO, e desde a W73 dois traçados de espécies diferentes
                            // partilham o grosso.
                            (jw, jh) == CHEIO && (aw < CHEIO.0 || ah < CHEIO.1)
                        };
                        if corta {
                            voo = None;
                            abandonados += 1;
                        }
                    }
                    if voo.is_none()
                        && let Some((w, h, coarse)) = ask
                    {
                        requested = Some((cam, w, h, doc.clone(), coarse));
                        voo = Some((t + custo(w, h, coarse), t, w, h, coarse));
                        if !coarse {
                            comecados += 1;
                        }
                    }
                    // ⚠️ **A régua é ANGULAR, e a 1.ª versão media milissegundos.** Durante uma
                    // pausa a imagem «velha» mostra uma câmera que **não se mexeu** — ela está
                    // certa, e contar o tempo dela faria a tabela acusar de defeito o próprio
                    // gesto do artista. *O que se vê é a peça noutro sítio, não a idade da foto.*
                    if mexe(t)
                        && publicadas >= 2
                        && let Some(p) = exposta
                    {
                        atraso_max = atraso_max.max((angulo(t) - angulo(p)).abs());
                    }
                    t += QUADRO;
                }
                linha[li] = (atraso_max, comecados, abandonados);
            }
            println!(
                "{pausa_ms:9.0} ms | {:17.2}° | {:14.2}° | {}/{} -> {}/{}",
                linha[0].0, linha[1].0, linha[0].1, linha[0].2, linha[1].1, linha[1].2
            );
        }
        for ((w, h, coarse), ms) in &memo {
            println!(
                "    custo medido {w}x{h} {:12} | {ms:7.1} ms",
                if *coarse { "movimento" } else { "refinamento" }
            );
        }
    }
}

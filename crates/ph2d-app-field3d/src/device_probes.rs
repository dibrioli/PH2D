//! ⭐⭐⭐ **AS SONDAS QUE COMPARAM OS DOIS MOTORES** — o que MEDE, separado do que AFIRMA.
//!
//! ⚠️ **Elas moram aqui e não ao lado dos gates porque o tecto de LOC da workspace o obrigou**
//! (2026-09-15, `718` contra `700`) — e a fronteira que ele forçou é a certa: o
//! [`super::device_tests`] **afirma** propriedades (com barras, controlos e mutações a matá-las) e
//! isto **mede** (imprime tabelas e não afirma quase nada). *Uma sonda e um gate têm leitores
//! diferentes: a primeira responde «quanto», o segundo «ainda é verdade».*
//!
//! ⛔⛔ **As duas medem o quadro INTEIRO nos dois lados** (`device_tests::quadro_na_cpu`), e foi
//! essa correcção que moveu o tecto **sete vezes** — ver `docs/Render3d/05` §43.5.

use super::device_tests::{LH, LW, anel, cpu_ociosa_pct, quadro_na_cpu};

#[test]
#[ignore = "medição — precisa de GPU"]
fn mede_o_preco_de_uma_aresta_de_perfil() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let cam = ph2d_field_render::Orbit::default();
    let luz = [crate::gpu_frame::tests_lampada(&cam)];
    let materiais = [ph2d_material::OpenPbr::default().prepare()];
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: None,
    };
    let olhar = ph2d_view_transform::Look::default();
    const BG: [u8; 4] = [0, 0, 0, 0];

    println!(
        "\n  arestas · guardados · vivos CRU→ESC · quadro CRU→ESC a 1920×1080 · CPU · load {} · ociosa {:.0} %",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim(),
        cpu_ociosa_pct()
    );
    let mut pontos: Vec<(u32, usize, usize, f32)> = Vec::new();
    // ⛔⛔ **O TOPO DA VARREDURA É `768` E NÃO `1024`, e o motivo é MEDIDO** (2026-09-15): o ponto de
    // `1024` arestas produz um shader de ~`27 500` valores guardados, e ele **pendurou o driver** —
    // o binário ficou em `S (sleeping)` com o tempo de CPU a não avançar, por mais de meia hora,
    // até ser morto. ⚠️ Não é determinista (ele já tinha corrido uma vez, em `3 508,70 ms`), e é
    // exactamente por isso que não fica numa sonda: *uma sonda que PODE pendurar a placa gasta a
    // máquina de quem a corre e não avisa.* O número que ele deu fica na tabela do
    // `docs/Render3d/05` §43.8, com esta nota ao lado.
    for n in [32u32, 64, 96, 128, 192, 256, 384, 512, 768] {
        let doc = ph2d_field::FieldDoc::new(
            vec![ph2d_field_eval::leaf(
                ph2d_field::Primitive::Extrude {
                    profile: anel(n),
                    half_height: 0.25,
                    round: 0.0,
                    chamfer: 0.0,
                },
                ph2d_field::Xform::IDENTITY,
            )],
            ph2d_field::NodeId(0),
        )
        .expect("a peça extrudada");

        let retrato = |escalonar: bool| {
            let campo = ph2d_field_eval::device::DeviceField::new_com(&doc, &reg, escalonar)
                .expect("a peça");
            let forma = campo.tape_shape().expect("a forma");
            (
                campo.tape_wgsl().expect("a fita").source.lines().count(),
                forma.vivos,
                forma.guardados,
            )
        };
        let ((instrs, vivos, guardados), (_, vivos_cru, _)) = (retrato(true), retrato(false));

        // ⭐⭐⭐ **O MESMO QUADRO NAS DUAS ORDENS DA MESMA FITA** — ver
        // [`ph2d_field_eval::tape_schedule`]. ⚠️ **A/B na MESMA corrida e na mesma máquina**: a
        // coluna crua é a que diz se o escalonador comprou relógio, e compará-la com um número
        // escrito noutro dia mediria a carga daquele dia.
        let mede = |escalonar: bool| {
            let sonda = crate::gpu_frame::Sonda {
                escalonar,
                ..crate::gpu_frame::Sonda::default()
            };
            // Aquecimento fora da conta: a primeira compila o pipeline. ⚠️ **Sem o tecto** — esta
            // é a sonda que o calibra, e ela tem de atravessar o degrau para o poder ver.
            let _ = crate::gpu_frame::paint_com(
                t, &doc, &reg, &cam, &luz, &surfaces, olhar, BG, None, LW, LH, false, sonda,
            );
            let mut v: Vec<f32> = Vec::new();
            for _ in 0..5 {
                let t0 = std::time::Instant::now();
                let p = crate::gpu_frame::paint_com(
                    t, &doc, &reg, &cam, &luz, &surfaces, olhar, BG, None, LW, LH, false, sonda,
                )
                .expect("o pintor");
                std::hint::black_box(p.rgba.len());
                #[allow(clippy::cast_possible_truncation)]
                v.push(t0.elapsed().as_secs_f32() * 1e3);
            }
            v.sort_by(f32::total_cmp);
            v[0]
        };
        let ms = mede(true);
        let ms_cru = mede(false);
        // ⛔⛔ **E O MESMO QUADRO NA CPU** — a pergunta que esta sonda tem de responder não é «quanto
        // custa na placa», é **«a placa ficou mais lenta que o caminho que ela substituiu?»**. Uma
        // tabela só com a coluna nova não sabe dizer se a wave foi uma regressão.
        // ⚠️⚠️ **A coluna da CPU SAIU, e a razão fica escrita:** a máquina esteve a `load 80–100` a
        // jornada inteira (outra linha a correr a suíte dela), e ali o mesmo traçado leu `71` e
        // `482 ms`. *Uma régua que varia `7×` entre corridas do mesmo código não mede código.*
        //
        // ⭐ O que fica é a coluna que NÃO depende da carga: o custo **por aresta** na placa. Ele é
        // plano enquanto a fita cabe nos registos e cresce quando ela deixa de caber — e é essa
        // quebra, e não uma comparação, que nomeia o recurso.
        #[allow(clippy::cast_precision_loss)]
        let por_aresta = ms / n as f32;
        // ⭐⭐⭐ **E A TRAVESSIA COM A CPU** — o número que o §42 declarou não-medível.
        //
        // ⚠️⚠️ **Ele só vale com a máquina CALMA, e a linha imprime a carga ao lado.** A mesma
        // medição a `load 91` leu `71` e `482 ms` para o mesmo traçado: *uma régua que varia `7×`
        // entre corridas do mesmo código não mede código.* ⇒ quem ler esta coluna lê primeiro o
        // `load` do cabeçalho — senão está a medir quem mais está a usar a máquina.
        let mut c: Vec<f32> = Vec::new();
        for _ in 0..3 {
            let t0 = std::time::Instant::now();
            // ⚠️ **O QUADRO INTEIRO** — ver [`quadro_na_cpu`]. Medir só o traçado aqui pedia à
            // placa mais trabalho do que à CPU.
            std::hint::black_box(quadro_na_cpu(&doc, &reg, &cam, &luz, &surfaces, olhar));
            #[allow(clippy::cast_possible_truncation)]
            c.push(t0.elapsed().as_secs_f32() * 1e3);
        }
        c.sort_by(f32::total_cmp);
        println!(
            "  {n:>7} · {guardados:>9} · {vivos_cru:>5} → {vivos:>4} · {ms_cru:>8.2} → {ms:>7.2} ms \
             ({:>4.1}×) · {por_aresta:>6.3} ms/aresta · CPU {:>8.2} · {:>5.2}×",
            ms_cru / ms,
            c[0],
            c[0] / ms
        );
        pontos.push((n, instrs, vivos, ms));
    }

    // ⭐ **A inclinação, entre os dois extremos** — o número que decide a wave.
    let (n0, _, _, ms0) = pontos[0];
    let (n1, _, _, ms1) = *pontos.last().expect("há pontos");
    println!(
        "  ⇒ de {n0} para {n1} arestas: {:.1}× o relógio · {:.4} ms por aresta",
        ms1 / ms0,
        (ms1 - ms0) / (n1 - n0) as f32
    );
    assert!(
        ms1 > ms0,
        "o relógio não cresceu com as arestas ({ms0:.2} → {ms1:.2}) — a sonda não isolou a variável"
    );
}

/// ⭐⭐⭐ **AS PEÇAS REAIS NOS DOIS MOTORES** — a sonda que impede o tecto de ser calibrado numa
/// família e aplicado a outra.
///
/// # ⛔⛔ Por que ela existe
///
/// O tecto que aqui viveu saía de um **polígono extrudado de N arestas**, que é a
/// família que o `+ Extrude` produz e a única em que se pode varrer uma variável só. ⚠️ Mas quem o
/// tecto decide são as **cenas do produto**, e essas têm furos, aros arredondados, booleanas e
/// torno — *um tecto calibrado numa família e aplicado a outra é uma procuração*.
///
/// ⇒ esta sonda mede, cena a cena, o quadro nos DOIS motores, e imprime de que lado do tecto cada
/// uma cai. Uma cena cuja razão seja `> 1` e que o tecto RECUSE é um defeito de calibração —
/// e uma que ele aceite com razão `< 1` é o defeito que ele existe para impedir.
#[test]
#[ignore = "medição — precisa de GPU"]
fn mede_as_cenas_reais_nos_dois_motores() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let materiais = [ph2d_material::OpenPbr::default().prepare()];
    let olhar = ph2d_view_transform::Look::default();
    const BG: [u8; 4] = [0, 0, 0, 0];
    println!(
        "\n  cena · guardados · placa · CPU · razão · load {} · ociosa {:.0} %",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim(),
        cpu_ociosa_pct()
    );
    for n in 0..crate::smoke::scenes::CENAS {
        if crate::smoke::scenes::PODADAS.contains(&n) {
            continue;
        }
        let doc = crate::smoke::scene(n);
        let reg = crate::smoke::sampled_registry();
        let cam = ph2d_field_render::Orbit::default();
        let luz = [crate::gpu_frame::tests_lampada(&cam)];
        let surfaces = ph2d_field_render::Surfaces {
            all: &materiais,
            owners: None,
        };
        let Some(guardados) = ph2d_field_eval::device::DeviceField::new(&doc, &reg)
            .and_then(|c| c.tape_shape())
            .map(|s| s.guardados)
        else {
            continue;
        };
        // ⚠️ **SEM o tecto** — esta é a sonda que o calibra.
        let sonda = crate::gpu_frame::Sonda::default();
        let mede = |f: &dyn Fn()| {
            let mut v: Vec<f32> = Vec::new();
            for _ in 0..3 {
                let t0 = std::time::Instant::now();
                f();
                #[allow(clippy::cast_possible_truncation)]
                v.push(t0.elapsed().as_secs_f32() * 1e3);
            }
            v.sort_by(f32::total_cmp);
            v[0]
        };
        let na_placa = || {
            if let Some(p) = crate::gpu_frame::paint_com(
                t, &doc, &reg, &cam, &luz, &surfaces, olhar, BG, None, LW, LH, false, sonda,
            ) {
                std::hint::black_box(p.rgba.len());
            }
        };
        if crate::gpu_frame::paint_com(
            t, &doc, &reg, &cam, &luz, &surfaces, olhar, BG, None, LW, LH, false, sonda,
        )
        .is_none()
        {
            println!("  {n:>4} · {guardados:>9} · (a placa recusa)");
            continue;
        }
        let ms_gpu = mede(&na_placa);
        let na_cpu = || {
            std::hint::black_box(quadro_na_cpu(&doc, &reg, &cam, &luz, &surfaces, olhar));
        };
        let ms_cpu = mede(&na_cpu);
        println!(
            "  {n:>4} · {guardados:>9} · {ms_gpu:>8.2} · {ms_cpu:>8.2} · {:>5.2}×",
            ms_cpu / ms_gpu
        );
    }
}

/// ⭐⭐ **A AUDITORIA DO VASO** (ordem do dono, 2026-09-15: *«o render do vaso deveria ser mais
/// rápido»*).
///
/// Ela separa as **duas** hipóteses que explicam um quadro caro, e que se leem iguais num número
/// só: custo **POR PIXEL** (a marcha a avaliar a fita) contra custo **FIXO POR QUADRO** (montar a
/// fita, escalonar, compilar o shader, ligar buffers). ⚠️ *Um quadro de `95 ms` pode ser `95` de
/// marcha ou `90` de montagem e `5` de marcha, e a tabela das cenas reais não distingue os dois.*
///
/// A régua é uma **varredura de resolução**: o custo por pixel escala com a área, o custo fixo não.
/// Com dois pontos, `fixo = (c₁·a₂ − c₂·a₁)/(a₂ − a₁)` — e o terceiro ponto é o **controlo** que
/// diz se o modelo de duas parcelas descreve a curva ou se há um terceiro termo.
///
/// ⚠️ O histograma sai do **WGSL emitido**, que é o que a placa de facto corre — não do `Instr`,
/// que é o que eu *penso* que ela corre.
#[test]
#[ignore = "sonda de auditoria: pede adaptador e uma máquina calma"]
fn audita_o_vaso() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let materiais = [ph2d_material::OpenPbr::default().prepare()];
    let olhar = ph2d_view_transform::Look::default();
    const BG: [u8; 4] = [0, 0, 0, 0];
    println!(
        "\n  ociosa {:.0} % · load {}",
        cpu_ociosa_pct(),
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );

    // O vaso é a `5`; a `4` é o MESMO contorno extrudado (mesma fita, outra marcha) e a `2` é o
    // cubo — o piso do que um quadro custa quando a fita não é o problema.
    for cena in [5u32, 4, 2] {
        let doc = crate::smoke::scene(cena);
        let reg = crate::smoke::sampled_registry();
        let cam = ph2d_field_render::Orbit::default();
        let luz = [crate::gpu_frame::tests_lampada(&cam)];
        let surfaces = ph2d_field_render::Surfaces {
            all: &materiais,
            owners: None,
        };
        let sonda = crate::gpu_frame::Sonda::default();
        let Some(dev) = ph2d_field_eval::device::DeviceField::new(&doc, &reg) else {
            continue;
        };
        let Some(forma) = dev.tape_shape() else {
            continue;
        };
        let wgsl = dev.tape_wgsl();
        let fonte = wgsl.as_ref().map(|w| w.source.as_str()).unwrap_or("");
        let lets = fonte.matches("let ").count();
        let mut hist: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
        for op in [
            "sqrt(", "min(", "max(", "abs(", "select(", "length(", "clamp(", "cos(", "sin(",
            "atan2(", "pow(", "exp(", "log(", "floor(", "dot(",
        ] {
            let n = fonte.matches(op).count();
            if n > 0 {
                hist.insert(op.trim_end_matches('('), n);
            }
        }
        let aritmetica = fonte.matches(" * ").count()
            + fonte.matches(" + ").count()
            + fonte.matches(" - ").count()
            + fonte.matches(" / ").count();

        println!(
            "\n  ── cena {cena} · fita {} ops · vivos {} · guardados {} · WGSL {} lets, {} B",
            forma.ops,
            forma.vivos,
            forma.guardados,
            lets,
            fonte.len()
        );
        print!("     transcendentais/chamadas:");
        for (k, v) in &hist {
            print!(" {k}×{v}");
        }
        println!("  ·  aritmética ×{aritmetica}");

        // ── a varredura de resolução ──────────────────────────────────────────────────────
        let mede = |w: u32, h: u32| -> f32 {
            let f = || {
                if let Some(p) = crate::gpu_frame::paint_com(
                    t, &doc, &reg, &cam, &luz, &surfaces, olhar, BG, None, w, h, false, sonda,
                ) {
                    std::hint::black_box(p.rgba.len());
                }
            };
            f(); // aquece: a 1.ª corrida compila o pipeline
            let mut v: Vec<f32> = Vec::new();
            for _ in 0..5 {
                let t0 = std::time::Instant::now();
                f();
                #[allow(clippy::cast_possible_truncation)]
                v.push(t0.elapsed().as_secs_f32() * 1e3);
            }
            v.sort_by(f32::total_cmp);
            v[0]
        };
        let pontos = [(480u32, 270u32), (960, 540), (1920, 1080)];
        let mut lidos: Vec<(f64, f64)> = Vec::new();
        println!("     resolução ·   quadro ·  ns/pixel");
        for (w, h) in pontos {
            let ms = mede(w, h);
            let area = f64::from(w) * f64::from(h);
            lidos.push((area, f64::from(ms)));
            println!(
                "     {w:>4}×{h:<4} · {ms:>7.2} ms · {:>8.1}",
                f64::from(ms) * 1e6 / area
            );
        }
        // fixo e por-pixel a partir dos DOIS extremos; o ponto do meio é o controlo.
        let (a1, c1) = lidos[0];
        let (a3, c3) = lidos[2];
        let fixo = (c1 * a3 - c3 * a1) / (a3 - a1);
        let por_pixel = (c3 - c1) / (a3 - a1);
        let (a2, c2) = lidos[1];
        let previsto = fixo + por_pixel * a2;
        println!(
            "     ⇒ FIXO por quadro {fixo:.2} ms · marcha {:.2} ms a 1920×1080 ({:.0} % do quadro)",
            por_pixel * a3,
            100.0 * por_pixel * a3 / c3
        );
        println!(
            "       controlo (960×540): previsto {previsto:.2} · lido {c2:.2} · erro {:.1} %",
            100.0 * (previsto - c2).abs() / c2
        );
    }
}

/// ⭐⭐⭐ **O QUE O ARREDONDAMENTO DAS QUINAS CUSTA NO VASO** (auditoria de 2026-09-15).
///
/// A [`audita_o_vaso`] mostrou que `89 %` do quadro é a **marcha**, e que a marcha avalia uma fita
/// de `2 969` operações porque o contorno de **12 pontos** que o artista desenha vira **94 arestas**.
/// ⚠️ E o botão *Resolution* dele **não pode ajudar**: `DEFAULT_PROFILE_RESOLUTION = 1` já é o nível
/// mais grosseiro, e subir o nível só acrescenta arestas.
///
/// ⇒ resta perguntar de onde vêm as `94`. O [`ph2d_field::Profile`] é **polilinha pura**
/// (`Vec<Vec<[f32; 2]>>`): não existe primitiva de ARCO, logo cada raio de quina é **tesselado**.
/// Esta sonda mede o piso — o MESMO vaso com as quinas **vivas** (raio `0`) — e a escada entre os
/// dois, variando quantas quinas são arredondadas.
///
/// ⚠️ **A forma muda entre as linhas da tabela, de propósito.** Isto não é uma comparação de
/// qualidade: é o PREÇO de uma capacidade. A pergunta que ela responde é *«quanto do quadro do
/// artista é tesselação de arco?»*, e a cura que ela precifica (um arco EXACTO na primitiva, que
/// custa ~uma aresta e é mais preciso que oito) não paga esse preço nenhum.
#[test]
#[ignore = "sonda de auditoria: pede adaptador e uma máquina calma"]
fn audita_o_arredondamento_do_vaso() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    // Os MESMOS 12 pontos da cena 5 (o vaso oco), com os raios autorados.
    const VASO: [([f64; 2], f64); 12] = [
        ([0.00, -0.45], 0.0),
        ([0.26, -0.45], 0.05),
        ([0.30, -0.34], 0.05),
        ([0.15, -0.10], 0.06),
        ([0.33, 0.22], 0.06),
        ([0.27, 0.44], 0.04),
        ([0.33, 0.52], 0.02),
        ([0.27, 0.52], 0.02),
        ([0.21, 0.44], 0.04),
        ([0.09, -0.08], 0.05),
        ([0.19, -0.32], 0.04),
        ([0.00, -0.32], 0.0),
    ];
    let materiais = [ph2d_material::OpenPbr::default().prepare()];
    let olhar = ph2d_view_transform::Look::default();
    const BG: [u8; 4] = [0, 0, 0, 0];
    println!(
        "\n  ociosa {:.0} % · load {}",
        cpu_ociosa_pct(),
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    println!("  quinas redondas · arestas · fita(ops) · guardados ·   quadro · ms/aresta");

    // `n` = quantas das 10 quinas com raio ficam redondas; as restantes ficam VIVAS.
    for n in [0usize, 2, 5, 10] {
        let mut vistas = 0usize;
        let verts: Vec<ph2d_vec_scene::VecVertex> = VASO
            .iter()
            .map(|&(p, r)| {
                let manter = if r > 0.0 {
                    vistas += 1;
                    vistas <= n
                } else {
                    false
                };
                ph2d_vec_scene::VecVertex {
                    corner_radius: if manter { r } else { 0.0 },
                    ..ph2d_vec_scene::VecVertex::corner(p)
                }
            })
            .collect();
        let path = ph2d_vec_scene::VecPath {
            verts,
            closed: true,
            ..ph2d_vec_scene::VecPath::default()
        };
        let Ok(profile) = ph2d_field_profile::cook_path_auto(&path) else {
            continue;
        };
        let arestas = profile.segment_count();
        let Ok(doc) = ph2d_field::FieldDoc::new(
            vec![crate::smoke::scenes::leaf(
                ph2d_field::Primitive::Revolve { profile },
                ph2d_field::Xform::IDENTITY,
            )],
            ph2d_field::NodeId(0),
        ) else {
            continue;
        };
        let reg = crate::smoke::sampled_registry();
        let cam = ph2d_field_render::Orbit::default();
        let luz = [crate::gpu_frame::tests_lampada(&cam)];
        let surfaces = ph2d_field_render::Surfaces {
            all: &materiais,
            owners: None,
        };
        let sonda = crate::gpu_frame::Sonda::default();
        let (ops, guardados) = ph2d_field_eval::device::DeviceField::new(&doc, &reg)
            .and_then(|d| d.tape_shape())
            .map_or((0, 0), |s| (s.ops, s.guardados));
        let f = || {
            if let Some(p) = crate::gpu_frame::paint_com(
                t, &doc, &reg, &cam, &luz, &surfaces, olhar, BG, None, LW, LH, false, sonda,
            ) {
                std::hint::black_box(p.rgba.len());
            }
        };
        f();
        let mut v: Vec<f32> = Vec::new();
        for _ in 0..5 {
            let t0 = std::time::Instant::now();
            f();
            #[allow(clippy::cast_possible_truncation)]
            v.push(t0.elapsed().as_secs_f32() * 1e3);
        }
        v.sort_by(f32::total_cmp);
        let ms = v[0];
        println!(
            "  {n:>15} · {arestas:>7} · {ops:>9} · {guardados:>9} · {ms:>7.2} ms · {:>9.3}",
            f64::from(ms) / f64::from(u32::try_from(arestas).unwrap_or(1))
        );
    }
}

/// ⭐⭐⭐ **AS FAIXAS DO VASO** (smoke do dono, 2026-09-16: *«arestas ainda visíveis»*, com foto).
///
/// A régua é a **assinatura de uma FACETA**: ao descer uma coluna, a normal fica quase parada,
/// **salta** de uma vez, e volta a ficar parada — *plano, plano, SALTO, plano, plano*. Uma superfície
/// lisa muda a normal DEVAGAR e por igual; uma curva de raio pequeno muda-a DEPRESSA, mas também
/// por igual. Só a faceta faz um PICO isolado entre vizinhos calmos.
///
/// ⛔ **A 1.ª versão desta régua media o salto MÁXIMO e caiu:** os dois motores leram `82–85°` no
/// mesmo pixel, que era a coluna do EIXO (o pólo do torno) e as fronteiras de OCLUSÃO (o lábio à
/// frente da parede interna — dois pixels que acertam superfícies diferentes). ⇒ esta exige
/// **continuidade no mundo** entre os pixels e conta **picos**, não máximos.
///
/// Mede os DOIS motores do produto pela mesma régua, com a câmara na **vista de frente** — a da foto.
#[test]
#[ignore = "sonda de auditoria: pede adaptador"]
fn mede_as_faixas_do_vaso() {
    // A régua é a do gate — uma só (`vaso_sem_facetas_tests::facetas`).
    let facetas = |g: &ph2d_field_render::Gbuffer, p: f32| {
        let (n, med, mx, _) = super::vaso_sem_facetas_tests::facetas(g, p);
        (n, med, mx)
    };
    let doc = crate::smoke::scene(5);
    let reg = crate::smoke::sampled_registry();
    let cam = super::vaso_sem_facetas_tests::camara_de_frente();
    // O tamanho de um pixel no mundo, para a continuidade: a meia-largura da vista sobre meia tela.
    #[allow(clippy::cast_precision_loss)]
    let passo = 2.0 * cam.half_extent / LH as f32;
    println!(
        "\n  ociosa {:.0} % · vista de FRENTE · motor · picos de faceta · passo mediano · maior pico",
        cpu_ociosa_pct()
    );
    let cpu = ph2d_field_render::trace(&doc, &reg, &cam, LW, LH);
    let (n, med, mx) = facetas(&cpu, passo);
    println!("  CPU (traçado)       · {n:>7} · {med:>6.3}° · {mx:>6.2}°");
    if let Some(t) = crate::gpu_frame::shared() {
        let luz = [crate::gpu_frame::tests_lampada(&cam).world];
        if let Some((dev, _)) =
            crate::gpu_frame::march(t, &doc, &reg, &cam, &luz, None, LW, LH, false)
        {
            let (n, med, mx) = facetas(&dev, passo);
            println!("  placa (dispositivo) · {n:>7} · {med:>6.3}° · {mx:>6.2}°");
        } else {
            println!("  placa: recusou a peça");
        }
    } else {
        println!("  placa: sem adaptador");
    }
}

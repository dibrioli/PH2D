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
            let sonda = crate::gpu_frame::Sonda { escalonar };
            // Aquecimento fora da conta: a primeira compila o pipeline. ⚠️ **Sem o tecto** — esta
            // é a sonda que o calibra, e ela tem de atravessar o degrau para o poder ver.
            let _ = crate::gpu_frame::paint_com(
                t, &doc, &reg, &cam, &luz, &surfaces, olhar, BG, LW, LH, false, sonda,
            );
            let mut v: Vec<f32> = Vec::new();
            for _ in 0..5 {
                let t0 = std::time::Instant::now();
                let p = crate::gpu_frame::paint_com(
                    t, &doc, &reg, &cam, &luz, &surfaces, olhar, BG, LW, LH, false, sonda,
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
        let sonda = crate::gpu_frame::Sonda { escalonar: true };
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
                t, &doc, &reg, &cam, &luz, &surfaces, olhar, BG, LW, LH, false, sonda,
            ) {
                std::hint::black_box(p.rgba.len());
            }
        };
        if crate::gpu_frame::paint_com(
            t, &doc, &reg, &cam, &luz, &surfaces, olhar, BG, LW, LH, false, sonda,
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

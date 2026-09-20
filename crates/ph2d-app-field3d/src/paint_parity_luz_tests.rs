//! ⭐⭐⭐ **A PARIDADE DA LUZ QUE O CHÃO RECEBE E DA QUE ATRAVESSA A PEÇA** — os gates das duas waves
//! que chegaram depois do pintor (`docs/Render3d/09` e `docs/Render3d/10`).
//!
//! # Por que um ficheiro irmão
//!
//! O [`super::paint_parity_tests`] é o gate do **pintor**: a imagem do dispositivo contra a do
//! `shade_render`, e as réguas que o defendem. Estes cinco são um assunto fechado dentro dele — *o
//! que a peça PÕE no chão, e o que a luz faz ao ATRAVESSAR* —, e o ficheiro passou o tecto de LOC.
//!
//! ⛔ **Split, nunca allowlist** (`CLAUDE.md` §5.0), e o corte é por RESPONSABILIDADE: os dois
//! primeiros gates de cada lado partilham a fixtura e a lei, e estes cinco partilham a `chao`.
//!
//! ⚠️ **O arnês fica no irmão** — a [`super::paint_parity_tests::dois_caminhos_com`] é a porta por
//! onde os dois lados correm sobre a MESMA marcha, e duplicá-la faria os dois ficheiros medirem
//! dois programas.

use super::paint_parity_tests::{FUNDO, H, W, combina, dois_caminhos_com, lampada};
use ph2d_field::{FieldDoc, NodeId, Op};

#[ignore = "precisa de GPU"]
/// ⭐⭐⭐ **O CHÃO DO DISPOSITIVO É O DA CPU** (`docs/Render3d/07`) — a sombra, a oclusão e a
/// escurecida do fundo, nos dois motores.
///
/// ⚠️ **A população vem primeiro, e ela é a DIFERENÇA**: quantos bytes o chão muda contra o mesmo
/// quadro sem chão. Sem isso, dois fundos transparentes iguais leriam `100 %` de paridade sobre um
/// chão que nenhum dos dois desenhou.
#[test]
fn o_chao_do_dispositivo_e_o_da_cpu() {
    if crate::gpu_frame::shared().is_none() {
        println!("sem adaptador — saltado");
        return;
    }
    let folha = |p: ph2d_field::Primitive, x: ph2d_field::Xform| ph2d_field_eval::leaf(p, x);
    let doc = FieldDoc::new(
        vec![
            folha(
                ph2d_field::Primitive::Sphere { radius: 0.25 },
                ph2d_field::Xform::at(-0.25, 0.25, 0.0),
            ),
            folha(
                ph2d_field::Primitive::Box {
                    half: [0.16; 3],
                    round: 0.02,
                    chamfer: 0.0,
                },
                ph2d_field::Xform::at(0.3, 0.16, 0.1),
            ),
            combina(
                Op::Union(ph2d_field::Blend::Sharp),
                vec![NodeId(0), NodeId(1)],
            ),
        ],
        NodeId(2),
    )
    .expect("a peça pousada");
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let materiais = [ph2d_material::OpenPbr::default().prepare()];
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: None,
    };
    let cam = ph2d_field_render::Orbit::default();
    let luz = [lampada(&cam)];
    let chao = ph2d_field_render::lowest_point(&doc, &reg)
        .map(|height| ph2d_field_render::Ground { height });
    assert!(chao.is_some(), "a peça tem chão");
    let (cpu_sem, _, _) =
        dois_caminhos_com(&surfaces, &doc, &luz, None).expect("o dispositivo toma a peça");
    let (cpu, gpu, bordas) =
        dois_caminhos_com(&surfaces, &doc, &luz, chao).expect("o dispositivo toma a peça");

    let mudados = cpu.iter().zip(&cpu_sem).filter(|(a, b)| a != b).count();
    assert!(
        mudados > 4_000,
        "o chão só mudou {mudados} bytes — a fixtura não o mostra, e o gate não afirma nada"
    );
    assert!(
        bordas > 100,
        "só {bordas} pixels de borda — a metade da borda ficou por exercitar"
    );

    let mut hist = [0usize; 256];
    let mut pior = (0u8, 0usize, 0usize);
    for (i, (a, b)) in cpu.iter().zip(gpu.iter()).enumerate() {
        let d = a.abs_diff(*b);
        hist[d as usize] += 1;
        if d > pior.0 {
            pior = (d, i / 4 % W as usize, i / 4 / W as usize);
        }
    }
    #[allow(clippy::cast_precision_loss)]
    let total = hist.iter().sum::<usize>() as f64;
    #[allow(clippy::cast_precision_loss)]
    let fraccao = hist[..2].iter().sum::<usize>() as f64 / total;
    println!(
        "chão · {mudados} bytes mudados pelo chão · ≤1 nivel em {:.3} % · pior {} em ({}, {})",
        fraccao * 100.0,
        pior.0,
        pior.1,
        pior.2
    );
    assert!(
        fraccao >= 0.995,
        "só {:.3} % dos canais estão a ≤1 nível — a lei do chão divergiu entre os motores",
        fraccao * 100.0
    );
    assert!(
        pior.0 <= 2,
        "o pior canal diverge {} níveis em ({}, {})",
        pior.0,
        pior.1,
        pior.2
    );
}

/// ⭐⭐⭐ **A COR QUE A PEÇA DEVOLVE AO CHÃO É A MESMA NOS DOIS MOTORES** (`docs/Render3d/09`).
///
/// A peça é **vermelha forte e sem especular** de propósito: com o material de omissão (cinzento) o
/// sangramento é cinzento, e um gate de paridade sobre ele não distinguiria a lei nova de um
/// arredondamento. *A fixtura tem de conter o fenómeno que o gate afirma.*
///
/// ⚠️ **A população vem primeiro, e ela é a DIFERENÇA** contra o mesmo quadro de CPU sem campo:
/// sem isso, dois quadros iguais leriam `100 %` de paridade sobre uma wave que não pintou nada.
#[test]
#[ignore = "precisa de GPU"]
fn a_cor_que_a_peca_devolve_ao_chao_e_a_mesma_nos_dois_motores() {
    if crate::gpu_frame::shared().is_none() {
        println!("sem adaptador — saltado");
        return;
    }
    let doc = FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            ph2d_field::Primitive::Sphere { radius: 0.3 },
            ph2d_field::Xform::at(0.0, 0.3, 0.0),
        )],
        NodeId(0),
    )
    .expect("a bola pousada");
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let materiais = [ph2d_material::OpenPbr {
        base_color: [0.75, 0.06, 0.06],
        specular_weight: 0.0,
        ..ph2d_material::OpenPbr::default()
    }
    .prepare()];
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: None,
    };
    let cam = ph2d_field_render::Orbit::default();
    let luz = [lampada(&cam)];
    let chao = ph2d_field_render::lowest_point(&doc, &reg)
        .map(|height| ph2d_field_render::Ground { height });
    assert!(chao.is_some(), "a peça tem chão");

    // ── a população: quantos bytes o CAMPO move, no caminho de CPU ────────────────────────────
    let mundos: Vec<[f32; 3]> = luz.iter().map(|l| l.world).collect();
    let g = ph2d_field_render::trace(&doc, &reg, &cam, W, H);
    let mut sh = ph2d_field_render::shadow_pass_on(&doc, &reg, &cam, &g, &mundos, chao);
    let sem_ecra: [ph2d_field_render::Lamp; 0] = [];
    let pinta = |sh: &ph2d_field_render::Shadows| {
        ph2d_field_render::shade_render(
            &g,
            &cam,
            &surfaces,
            &ph2d_field_render::Lighting {
                lamps: &sem_ecra,
                points: &luz,
                sky: &crate::render_light::StudioSky,
                shadows: Some(sh),
            },
            &ph2d_field_render::Presentation::of(ph2d_view_transform::Look::default()),
            FUNDO,
        )
    };
    let sem_campo = pinta(&sh);
    sh.set_ground_bounce(ph2d_field_render::ground_bounce::bake_ground_bounce(
        &doc,
        &reg,
        &cam,
        chao.expect("o chão"),
        &surfaces,
        &luz,
        ph2d_field_render::ground_bounce::GROUND_BOUNCE_GRID,
        ph2d_field_render::ground_bounce::GROUND_BOUNCE_DIRS,
        W.min(H) as usize,
    ));
    let com_campo = pinta(&sh);
    let movidos = com_campo
        .iter()
        .zip(&sem_campo)
        .filter(|(a, b)| a != b)
        .count();
    assert!(
        movidos > 2_000,
        "o campo só moveu {movidos} bytes — a fixtura não o mostra, e o gate não afirma nada"
    );

    // ── e os dois motores concordam ───────────────────────────────────────────────────────────
    let (cpu, gpu, bordas) =
        dois_caminhos_com(&surfaces, &doc, &luz, chao).expect("o dispositivo toma a peça");
    assert!(
        bordas > 50,
        "só {bordas} bordas — a metade da borda ficou por exercitar"
    );
    let mut hist = [0usize; 256];
    let mut pior = (0u8, 0usize, 0usize);
    for (i, (a, b)) in cpu.iter().zip(gpu.iter()).enumerate() {
        let d = a.abs_diff(*b);
        hist[d as usize] += 1;
        if d > pior.0 {
            pior = (d, i / 4 % W as usize, i / 4 / W as usize);
        }
    }
    #[allow(clippy::cast_precision_loss)]
    let total = hist.iter().sum::<usize>() as f64;
    #[allow(clippy::cast_precision_loss)]
    let fraccao = hist[..2].iter().sum::<usize>() as f64 / total;
    println!(
        "campo do chão · {movidos} bytes movidos · ≤1 nivel em {:.3} % · pior {} em ({}, {})",
        fraccao * 100.0,
        pior.0,
        pior.1,
        pior.2
    );
    assert!(
        fraccao >= 0.995,
        "só {:.3} % dos canais estão a ≤1 nível — a lei do campo divergiu entre os motores",
        fraccao * 100.0
    );
}

/// ⏱️⭐⭐ **O QUE A COR NO CHÃO CUSTA** — os dois lados no MESMO processo, intercalados.
///
/// ⚠️ *Nenhuma leitura de relógio desta workstation vale nada acima de `load ~5`*, e entre duas
/// corridas o mesmo passe já deu `11,36` e `5,50 ms` — por isso a [`crate::gpu_frame::Sonda`]
/// existe: os dois lados do A/B correm alternados, na mesma máquina, no mesmo segundo.
#[test]
#[ignore = "precisa de GPU; sonda de relógio"]
fn mede_o_que_a_cor_no_chao_custa() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    const LW: u32 = 1920;
    const LH: u32 = 1080;
    let doc = FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            ph2d_field::Primitive::Sphere { radius: 0.3 },
            ph2d_field::Xform::at(0.0, 0.3, 0.0),
        )],
        NodeId(0),
    )
    .expect("a bola pousada");
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let materiais = [ph2d_material::OpenPbr {
        base_color: [0.75, 0.06, 0.06],
        specular_weight: 0.0,
        ..ph2d_material::OpenPbr::default()
    }
    .prepare()];
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: None,
    };
    let cam = ph2d_field_render::Orbit::default();
    let luz = [lampada(&cam)];
    let chao = ph2d_field_render::lowest_point(&doc, &reg)
        .map(|height| ph2d_field_render::Ground { height });
    let olhar = ph2d_view_transform::Look::default();

    let corre = |recebe: bool| -> f64 {
        let sonda = crate::gpu_frame::Sonda {
            chao_recebe_cor: recebe,
            ..crate::gpu_frame::Sonda::default()
        };
        let t0 = std::time::Instant::now();
        let saida = crate::gpu_frame::paint_com(
            t,
            &doc,
            &reg,
            &cam,
            &luz,
            &surfaces,
            &ph2d_field_render::Presentation::of(olhar),
            FUNDO,
            chao,
            LW,
            LH,
            true,
            sonda,
        );
        assert!(saida.is_some(), "o dispositivo toma a peça");
        t0.elapsed().as_secs_f64() * 1000.0
    };
    // Aquecer: a 1.ª corrida compila o pipeline.
    let _ = corre(true);
    let _ = corre(false);
    let (mut com, mut sem) = (f64::MAX, f64::MAX);
    for _ in 0..5 {
        sem = sem.min(corre(false));
        com = com.min(corre(true));
    }
    println!(
        "  /proc/loadavg: {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    println!("  {LW}×{LH}, mínimo de 5, intercalado:");
    println!("  sem a cor no chão · {sem:>8.2} ms");
    println!(
        "  com a cor no chão · {com:>8.2} ms   (+{:.2} ms)",
        com - sem
    );
}

/// ⭐⭐⭐ **A LUZ QUE ATRAVESSA A PEÇA É A MESMA NOS DOIS MOTORES** — os DOIS caminhos
/// (`docs/Render3d/10`).
///
/// # ⚠️ As duas metades, e a segunda é a que faz o gate valer
///
/// **(a)** Cada caminho MOVE a imagem — sem isso os dois motores concordariam sobre uma lei
/// inerte, que é o defeito que a fixtura sem o fenómeno tem por escrito neste ficheiro.
/// **(b)** E eles concordam.
///
/// ⚠️ **A parede fina precisa da luz ATRÁS** e a maciça não: a primeira é a lambertiana do lado de
/// lá (`max(−N·L, 0)`) e sem uma luz por detrás ela lê zero em quase todo pixel visível. ⇒ a
/// fixtura tem as duas lâmpadas, e a de trás é a que acende a folha.
///
/// ⛔ **E a curvatura entra na referência pela mesma porta** — ver o topo do
/// [`dois_caminhos_com`]: sem ela este gate compararia dois programas.
#[test]
#[ignore = "precisa de GPU"]
fn a_luz_que_atravessa_a_peca_e_a_mesma_nos_dois_motores() {
    if crate::gpu_frame::shared().is_none() {
        println!("sem adaptador — saltado");
        return;
    }
    let doc = FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            ph2d_field::Primitive::Sphere { radius: 0.45 },
            ph2d_field::Xform::IDENTITY,
        )],
        NodeId(0),
    )
    .expect("a bola");
    let cam = ph2d_field_render::Orbit::default();
    // ⭐ **Duas lâmpadas: uma de frente e uma ATRÁS** — a de trás é a que a parede fina lê.
    let base = lampada(&cam);
    let atras = ph2d_field_render::PointLamp {
        world: [-base.world[0], base.world[1], -base.world[2]],
        ..base
    };
    let luz = [base, atras];

    let opaco = ph2d_material::OpenPbr {
        base_color: [0.2, 0.5, 0.1],
        specular_weight: 0.0,
        ..ph2d_material::OpenPbr::default()
    };
    let fina = ph2d_material::OpenPbr {
        subsurface_weight: 1.0,
        geometry_thin_walled: true,
        subsurface_color: [0.35, 0.75, 0.2],
        ..opaco
    };
    let macica = ph2d_material::OpenPbr {
        subsurface_weight: 1.0,
        geometry_thin_walled: false,
        subsurface_color: [0.3, 0.8, 0.5],
        subsurface_radius: 0.5,
        ..opaco
    };

    let pinta = |m: ph2d_material::OpenPbr| {
        let mats = [m.prepare()];
        let s = ph2d_field_render::Surfaces {
            all: &mats,
            owners: None,
        };
        dois_caminhos_com(&s, &doc, &luz, None).expect("o dispositivo toma a peça")
    };
    let (cpu_opaco, _, _) = pinta(opaco);
    for (nome, m) in [("parede fina", fina), ("maciça", macica)] {
        let (cpu, gpu, bordas) = pinta(m);
        assert!(bordas > 50, "{nome}: só {bordas} bordas");
        // (a) o caminho MOVE a imagem.
        let movidos = cpu
            .iter()
            .zip(&cpu_opaco)
            .filter(|(a, b)| a.abs_diff(**b) > 1)
            .count();
        assert!(
            movidos > 2_000,
            "{nome}: a lei só moveu {movidos} bytes contra o material opaco — a fixtura não a \
             mostra, e o gate não afirma nada"
        );
        // (b) e os dois motores concordam.
        let mut hist = [0usize; 256];
        let mut pior = (0u8, 0usize, 0usize);
        for (i, (a, b)) in cpu.iter().zip(gpu.iter()).enumerate() {
            let d = a.abs_diff(*b);
            hist[d as usize] += 1;
            if d > pior.0 {
                pior = (d, i / 4 % W as usize, i / 4 / W as usize);
            }
        }
        #[allow(clippy::cast_precision_loss)]
        let total = hist.iter().sum::<usize>() as f64;
        #[allow(clippy::cast_precision_loss)]
        let fraccao = hist[..2].iter().sum::<usize>() as f64 / total;
        println!(
            "{nome} · {movidos} bytes movidos · ≤1 nível em {:.3} % · pior {} em ({}, {})",
            fraccao * 100.0,
            pior.0,
            pior.1,
            pior.2
        );
        assert!(
            fraccao >= 0.995,
            "{nome}: só {:.3} % dos canais estão a ≤1 nível — a lei divergiu entre os motores",
            fraccao * 100.0
        );
    }
}

/// ⭐⭐⭐ **A BORDA MOLE DA SOMBRA É A MESMA NOS DOIS MOTORES** — a `W10`, o gémeo que faltava
/// (`docs/Render3d/10` §12 e §24).
///
/// # ⛔⛔ Porque ele precisou de uma fixtura NOVA, e a antiga não servia
///
/// A única fixtura translúcida deste ficheiro é uma **bola SOZINHA**, e uma bola sozinha não é
/// tapada por nada: o canal de visibilidade é **constante**, o borrão de uma constante é ela
/// própria, e as duas colunas concordavam sobre uma passagem que **não fazia nada**. *É o buraco de
/// régua que o plano da `W10` nomeia por escrito, e construir o gémeo contra ele repetiria, um nível
/// acima, o defeito que esta wave existe para curar.*
///
/// ⇒ a cena é a do REPORT (`=33`): a lâmina lança sombra sobre o jade, e é a borda dela que o dono
/// fotografou como *«uma linha dura»*.
///
/// # As duas metades
///
/// **(a)** O canal mole é DIFERENTE do duro — sem isto o gate compara dois motores sobre uma
/// passagem inerte. A régua é a do próprio canal, na CPU, antes de qualquer pintura.
/// **(b)** E as duas imagens concordam, com a barra de sempre deste ficheiro.
#[test]
#[ignore = "precisa de GPU"]
fn a_borda_mole_da_sombra_e_a_mesma_nos_dois_motores() {
    if crate::gpu_frame::shared().is_none() {
        println!("sem adaptador — saltado");
        return;
    }
    let doc = crate::smoke::scenes::edge::cena_33().expect("a cena do report");
    let cam = ph2d_field_render::Orbit::default();
    let (onde, luz_do_rig) = crate::lights::opening_light(&cam);
    let luz = [ph2d_field_render::PointLamp {
        world: onde,
        radiance_at_one: crate::lights::radiance_at_one(luz_do_rig),
    }];
    // O jade do report — `Subsurface` no máximo, `Thin Walled: Solid`.
    let jade = ph2d_material::OpenPbr {
        subsurface_weight: 1.0,
        geometry_thin_walled: false,
        subsurface_color: [0.75, 0.35, 0.35],
        base_color: [0.75, 0.35, 0.35],
        ..ph2d_material::OpenPbr::default()
    };
    let mats = [jade.prepare()];
    let surfaces = ph2d_field_render::Surfaces {
        all: &mats,
        owners: None,
    };

    // ⭐ **(a) A FIXTURA CONTÉM O FENÓMENO** — o canal mole afasta-se do duro. Medido na CPU, pela
    // mesma porta que o produto usa, e ANTES de qualquer pintura: *se ele não se afastar, o gate
    // abaixo compara dois motores sobre uma passagem que não fez nada.*
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let (g, sh) = crate::gpu_frame::march(
        crate::gpu_frame::shared().expect("a placa"),
        &doc,
        &reg,
        &cam,
        &[onde],
        None,
        W,
        H,
        true,
    )
    .expect("a marcha toma a cena do report");
    let dura = sh.lamp_channel(0).to_vec();
    let mole = ph2d_field_render::sss_shadow::blur_por_material(&g, &dura, &surfaces, &cam, H);
    assert!(
        !mole.is_empty(),
        "o canal mole saiu VAZIO — o material da fixtura deixou de ser translúcido"
    );
    let afastados = dura
        .iter()
        .zip(&mole)
        .filter(|(d, m)| (**d - m[0]).abs() > 0.01)
        .count();
    assert!(
        afastados > 500,
        "o canal mole afasta-se do duro em só {afastados} píxeis — a fixtura não tem sombra sobre \
         o material translúcido, e a comparação abaixo não afirmaria nada"
    );

    // ⭐ **(b) E OS DOIS MOTORES CONCORDAM.**
    let (cpu, gpu, bordas) =
        dois_caminhos_com(&surfaces, &doc, &luz, None).expect("o dispositivo toma a cena");
    assert!(
        bordas > 50,
        "só {bordas} bordas — a silhueta ficou por exercitar"
    );
    let mut hist = [0usize; 256];
    let mut pior = (0u8, 0usize, 0usize);
    for (i, (a, b)) in cpu.iter().zip(gpu.iter()).enumerate() {
        let d = a.abs_diff(*b);
        hist[d as usize] += 1;
        if d > pior.0 {
            pior = (d, i / 4 % W as usize, i / 4 / W as usize);
        }
    }
    #[allow(clippy::cast_precision_loss)]
    let total = hist.iter().sum::<usize>() as f64;
    #[allow(clippy::cast_precision_loss)]
    let fraccao = hist[..2].iter().sum::<usize>() as f64 / total;
    println!(
        "  borda mole · {afastados} píxeis com o canal afastado · ≤1 nível em {:.3} % · pior {} em \
         ({}, {})",
        fraccao * 100.0,
        pior.0,
        pior.1,
        pior.2
    );
    assert!(
        fraccao >= 0.995,
        "só {:.3} % dos canais estão a ≤1 nível — a borda mole divergiu entre os motores",
        fraccao * 100.0
    );
    // ⛔⛔⛔ **E O PIOR BYTE, porque a FRACÇÃO quase não viu o defeito.** Medido em 2026-09-19 com
    // o dispositivo a NÃO pedir o canal (a mutação `M5`): a fracção lê `99,579 %` — **`0,079` acima
    // da barra**, logo este gate ficava VERDE — e o pior byte lê **`12`** contra o `1` da árvore que
    // ship. *Uma fracção sobre uma imagem onde o fenómeno ocupa 10 % dos píxeis afoga-o na média;
    // o extremo não.* A barra sai desse vale e não de um número escolhido.
    assert!(
        pior.0 <= 4,
        "o pior byte é {} em ({}, {}) — a fracção passou e o extremo não: o canal mole deixou de \
         chegar ao pixel num dos motores",
        pior.0,
        pior.1,
        pior.2
    );
}

//! ⭐⭐⭐ **O BRILHO nos DOIS motores** (`docs/Render3d/12` §11) — a cadeia de mip, a composição e a
//! cobertura, medidas pelo caminho do PRODUTO.
//!
//! # ⛔⛔ Porque este gate tinha de existir antes de o halo chegar ao artista
//!
//! A lei vive numa folha e atravessa para o dispositivo como texto
//! ([`ph2d_bloom::wgsl`]) — a quarta a fazê-lo, a seguir ao material, ao olhar e à lei do dono. Um
//! peso trocado, uma amostragem por *sampler* em vez da bilinear escrita à mão, um `fma` que o
//! compilador funde de um lado e não do outro: **nada disso falha a compilar**, e a única coisa que
//! o apanha é comparar os dois quadros.
//!
//! ⚠️ **A fixtura tem de EMITIR.** Uma peça que não passa do limiar dá halo zero nos dois lados, e
//! um gate sobre ela seria verde sobre dois programas mortos — é por isso que a metade do CONTROLO
//! (*o brilho MOVEU píxeis*) não é opcional aqui.

use ph2d_material::OpenPbr;

/// ⭐ Um material que EMITE muito acima do limiar — o mesmo idioma da fixtura do passe de CPU.
fn aceso() -> Vec<ph2d_material::Surface> {
    vec![
        OpenPbr {
            base_color: [0.05, 0.05, 0.06],
            emission_luminance: 40.0,
            emission_color: [1.0, 0.9, 0.7],
            ..OpenPbr::default()
        }
        .prepare(),
    ]
}

fn ligado(p: ph2d_field_render::BloomParams) -> ph2d_field_render::Bloom {
    ph2d_field_render::Bloom {
        enabled: true,
        params: p,
    }
}

/// ⭐⭐⭐ **A TABELA: o halo do dispositivo contra o da referência**, bateria a bateria.
///
/// ```text
/// bash scripts/ph2d-run.sh env PH2D_GPU=1 cargo test -p ph2d-app-field3d \
///   brilho_parity -- --ignored --nocapture
/// ```
#[test]
#[ignore = "precisa de adaptador de GPU"]
fn o_halo_e_o_mesmo_nos_dois_motores() {
    let (doc, _, _) = crate::gpu_frame::paint_parity_tests::fixtura();
    let materiais = aceso();
    let cam = ph2d_field_render::Orbit::default();
    let luz = [crate::gpu_frame::paint_parity_tests::lampada(&cam)];
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: None,
    };
    let estilo = ph2d_style::Style::default();
    let base = ph2d_field_render::BloomParams::default();
    let baterias: Vec<(&str, ph2d_field_render::Bloom)> = vec![
        ("fábrica", ligado(base)),
        (
            "raio 8",
            ligado(ph2d_field_render::BloomParams {
                radius: 8.0,
                ..base
            }),
        ),
        (
            "tingido",
            ligado(ph2d_field_render::BloomParams {
                tint: [1.0, 0.25, 0.10, 1.0],
                saturation: 0.0,
                ..base
            }),
        ),
        (
            "tecto 2",
            ligado(ph2d_field_render::BloomParams { clamp: 2.0, ..base }),
        ),
        (
            "anamórfico",
            ligado(ph2d_field_render::BloomParams {
                stretch: 3.0,
                angle: 30.0,
                ..base
            }),
        ),
    ];

    // ⚠️⚠️ **O CONTROLO vem primeiro e é o mesmo par SEM brilho** — sem ele, uma cadeia que não
    // corresse em nenhum dos dois lados daria a tabela inteira a zeros e leria-se como vitória.
    let Some((cpu_sem, gpu_sem, _)) = crate::gpu_frame::paint_parity_tests::dois_caminhos_vestidos(
        &surfaces,
        &doc,
        &luz,
        None,
        estilo,
        ph2d_field_render::Bloom::default(),
    ) else {
        println!("sem adaptador — saltado");
        return;
    };

    println!("\n  BATERIA          acendeu(cpu)  acendeu(gpu)   fora(>1)   pior");
    let mut pior_global = 0u8;
    for (rot, bloom) in baterias {
        let Some((cpu, gpu, _)) = crate::gpu_frame::paint_parity_tests::dois_caminhos_vestidos(
            &surfaces, &doc, &luz, None, estilo, bloom,
        ) else {
            continue;
        };
        let acendeu = |a: &[u8], b: &[u8]| a.iter().zip(b).filter(|(x, y)| x != y).count();
        let (mut fora, mut pior) = (0usize, 0u8);
        for (c, g) in cpu.iter().zip(&gpu) {
            let d = c.abs_diff(*g);
            if d > 1 {
                fora += 1;
            }
            pior = pior.max(d);
        }
        pior_global = pior_global.max(pior);
        println!(
            "  {rot:<14} {:>12} {:>13} {fora:>10} {pior:>6}",
            acendeu(&cpu_sem, &cpu),
            acendeu(&gpu_sem, &gpu)
        );
        // ⭐ **A metade que faz a de cima ser uma afirmação:** os DOIS lados têm de ter acendido.
        assert!(
            acendeu(&cpu_sem, &cpu) > 500,
            "{rot}: a REFERÊNCIA mal acendeu — a fixtura não contém o fenómeno"
        );
        assert!(
            acendeu(&gpu_sem, &gpu) > 500,
            "{rot}: o DISPOSITIVO não acendeu — a cadeia não correu lá"
        );
        // ⚠️ **A barra é `2` bytes**, e não é escolhida: é a mesma que os outros passes deste módulo
        // usam para uma cadeia de `f32` que atravessa duas aritméticas (o `fma` que o compilador da
        // placa funde e o `pow` do WGSL que não é o `powf` do Rust — ver o `docs/Render3d/11` §10.8).
        assert!(
            pior <= 2,
            "{rot}: os dois motores discordam em {pior} bytes (fora da barra: {fora} canais)"
        );
    }
    println!("  pior de todas as baterias: {pior_global} bytes\n");
}

/// ⭐⭐ **E A COBERTURA CHEGA NO DISPOSITIVO** — a metade que o report de 19/09 pagou, do outro lado.
///
/// ⚠️ Sem ela o halo é calculado, somado e **apagado pelo canvas**: o quadro do modelador tem fundo
/// transparente, e o halo mora exactamente onde a peça não está.
#[test]
#[ignore = "precisa de adaptador de GPU"]
fn no_dispositivo_a_luz_do_halo_leva_cobertura() {
    let (doc, _, _) = crate::gpu_frame::paint_parity_tests::fixtura();
    let materiais = aceso();
    let cam = ph2d_field_render::Orbit::default();
    let luz = [crate::gpu_frame::paint_parity_tests::lampada(&cam)];
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: None,
    };
    let Some((_, gpu_sem, _)) = crate::gpu_frame::paint_parity_tests::dois_caminhos_vestidos(
        &surfaces,
        &doc,
        &luz,
        None,
        ph2d_style::Style::default(),
        ph2d_field_render::Bloom::default(),
    ) else {
        println!("sem adaptador — saltado");
        return;
    };
    let Some((_, gpu, _)) = crate::gpu_frame::paint_parity_tests::dois_caminhos_vestidos(
        &surfaces,
        &doc,
        &luz,
        None,
        ph2d_style::Style::default(),
        ligado(ph2d_field_render::BloomParams::default()),
    ) else {
        return;
    };
    let (mut acesos, mut sem_cobertura) = (0usize, 0usize);
    for i in 0..(crate::gpu_frame::paint_parity_tests::W * crate::gpu_frame::paint_parity_tests::H)
        as usize
    {
        // ⚠️ **Só onde o fundo era transparente** — a silhueta tem cobertura parcial por desenho, e
        // ali a pergunta não é esta.
        if gpu_sem[i * 4 + 3] != 0 {
            continue;
        }
        let luz_px = gpu[i * 4].max(gpu[i * 4 + 1]).max(gpu[i * 4 + 2]);
        if luz_px > 0 {
            acesos += 1;
            sem_cobertura += usize::from(gpu[i * 4 + 3] == 0);
        }
    }
    assert!(
        acesos > 500,
        "a fixtura não contém o fenómeno: só {acesos} píxeis de fundo acenderam"
    );
    assert_eq!(
        sem_cobertura, 0,
        "{sem_cobertura} de {acesos} píxeis do halo saem com cobertura ZERO no dispositivo"
    );
}

/// ⏱️ **QUANTO O BRILHO CUSTA NO DISPOSITIVO** (`#[ignore]`).
///
/// ```text
/// bash scripts/ph2d-run.sh env PH2D_GPU=1 cargo test --release -p ph2d-app-field3d \
///   quanto_o_brilho_custa -- --ignored --nocapture
/// ```
///
/// ⚠️ **Em `--release`**: em debug o lado Rust que monta os buffers lê muito mais lento e o número
/// mediria o perfil de build. ⚠️ E o relógio é o do **quadro inteiro** (marcha + pintura + leitura),
/// que é o que o artista sente — medir só o passe esconderia a leitura de volta.
///
/// **Medido** (`--release`, melhor de 5, nesta máquina), contra a mesma cadeia em CPU:
///
/// | tamanho | quadro | no dispositivo | na CPU |
/// |---|---|---:|---:|
/// | `445×305` | MOVIMENTO | **`0,35 ms`** | `24,1 ms` |
/// | `1898×916` | assente | **`1,7`–`4,1 ms`** | `347,2 ms` |
///
/// ⚠️ **O número do assente BALANÇA e a razão é conhecida:** cada quadro cria os buffers da cadeia
/// (`~50 MB` a esta resolução) e deita-os fora, logo o relógio segue o alocador do driver. *É uma
/// optimização com endereço — uma cache por tamanho, como o traçador já tem — e não um defeito:
/// `4 ms` num quadro que assenta é a folga inteira do efeito.*
#[test]
#[ignore = "sonda: precisa de adaptador de GPU"]
fn quanto_o_brilho_custa_no_dispositivo() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let (doc, _, _) = crate::gpu_frame::paint_parity_tests::fixtura();
    let materiais = aceso();
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let cam = ph2d_field_render::Orbit::default();
    let luz = [crate::gpu_frame::paint_parity_tests::lampada(&cam)];
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: None,
    };
    println!("\n  TAMANHO        sem brilho     com brilho      delta");
    for (w, h) in [(445u32, 305u32), (1898, 916), (1920, 1080)] {
        let mut linha = [0.0f64; 2];
        for (k, bloom) in [
            ph2d_field_render::Bloom::default(),
            ligado(ph2d_field_render::BloomParams::default()),
        ]
        .into_iter()
        .enumerate()
        {
            let pres = ph2d_field_render::Presentation {
                look: ph2d_view_transform::Look::default(),
                style: ph2d_style::Style::default(),
                piece_radius: ph2d_field_eval::bounds::bounding_ball(&doc, &reg)
                    .map_or(1.0, |b| b.radius),
                bloom,
            };
            // ⚠️ **Uma corrida a frio antes de medir** — a primeira compila os pipelines, e o cache
            // é por TEXTO: sem ela o «com brilho» pagaria a compilação da cadeia inteira.
            let _ = crate::gpu_frame::paint(
                t,
                &doc,
                &reg,
                &cam,
                &luz,
                &surfaces,
                &pres,
                crate::gpu_frame::paint_parity_tests::FUNDO,
                None,
                w,
                h,
                true,
            );
            let mut melhor = f64::MAX;
            for _ in 0..5 {
                let t0 = std::time::Instant::now();
                let r = crate::gpu_frame::paint(
                    t,
                    &doc,
                    &reg,
                    &cam,
                    &luz,
                    &surfaces,
                    &pres,
                    crate::gpu_frame::paint_parity_tests::FUNDO,
                    None,
                    w,
                    h,
                    true,
                );
                let ms = t0.elapsed().as_secs_f64() * 1000.0;
                assert!(r.is_some(), "o dispositivo recusou o quadro");
                melhor = melhor.min(ms);
            }
            linha[k] = melhor;
        }
        println!(
            "  {:<12} {:>9.2} ms {:>11.2} ms {:>9.2} ms",
            format!("{w}×{h}"),
            linha[0],
            linha[1],
            linha[1] - linha[0]
        );
    }
    println!();
}

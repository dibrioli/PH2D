//! Gates da FRESCURA (doc 12 §22.3) — a impressão digital e a lei do skip.
//!
//! ⚠️ **A metade que estes gates NÃO alcançam é a costura:** as duas funções podem estar perfeitas e
//! o laço do `composite_layers` nunca perguntar a elas (o *registrado ≠ despachado* que este repo já
//! pagou). Quem cobre isso é o arch-gate `the_stage_loop_asks_before_it_rasterises`.

use super::*;
use ph2d_flip_render::{GpuPoint, GpuStroke};

fn cam() -> CameraRaw {
    CameraRaw::new(
        [
            [0.002, 0.0, 0.0, 0.0],
            [0.0, -0.002, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0, 1.0],
        ],
        [1024.0, 768.0],
        1.0,
    )
}

fn fp_of(cam: &CameraRaw) -> u64 {
    fingerprint(0xABCD, None, cam, (1024, 768), true)
}

/// A impressão é **estável** para entradas idênticas — sem isso o skip nunca dispara e a wave é
/// código morto com todos os gates verdes.
#[test]
fn the_same_layer_twice_has_the_same_fingerprint() {
    assert_eq!(fp_of(&cam()), fp_of(&cam()));
}

/// ⭐ **CADA entrada da rasterização move a impressão** — é este gate que impede a camada de
/// congelar na tela, e ele falha ALTO (uma entrada esquecida = um `assert_ne` vermelho aqui).
#[test]
fn every_input_the_raster_consumes_moves_the_fingerprint() {
    let base = fp_of(&cam());

    // A GEOMETRIA (o hash de conteúdo do desenho).
    assert_ne!(
        base,
        fingerprint(0x1234, None, &cam(), (1024, 768), true),
        "editar o desenho tem de mover a impressao"
    );

    // O ALVO.
    assert_ne!(
        base,
        fingerprint(0xABCD, None, &cam(), (1920, 1080), true),
        "um resize muda todo pixel"
    );

    // O MOTOR.
    assert_ne!(
        base,
        fingerprint(0xABCD, None, &cam(), (1024, 768), false),
        "trocar de motor nao pode reusar os pixels do outro"
    );

    // O ZOOM (`px_per_world`), a projeção e o VIEWPORT — todos dentro do `CameraRaw`.
    let mut zoom = cam();
    zoom.px_per_world = 2.0;
    assert_ne!(base, fp_of(&zoom), "o zoom muda a espessura em px");

    let mut pan = cam();
    pan.world_to_clip[3][0] = -0.5;
    assert_ne!(base, fp_of(&pan), "panhar move a projecao");

    let mut vp = cam();
    vp.viewport = [800.0, 600.0];
    assert_ne!(base, fp_of(&vp), "o viewport entra na fita em screen-space");

    // O TINT DE FANTASMA — a mesma cobertura com outra cor É outro pixel.
    let ghost = cam().with_ghost_tint([0.2, 0.9, 0.3], 0.5);
    assert_ne!(
        base,
        fp_of(&ghost),
        "um fantasma nao pode reusar a fatia da arte real"
    );
    let ghost2 = cam().with_ghost_tint([0.2, 0.9, 0.3], 0.25);
    assert_ne!(
        fp_of(&ghost),
        fp_of(&ghost2),
        "dois fantasmas de fade diferente sao dois pixels diferentes"
    );
}

/// O traço VIVO entra na impressão — senão o preview congelaria no 1º frame do gesto, que é o
/// defeito mais visível que esta wave poderia introduzir.
#[test]
fn the_live_stroke_moves_the_fingerprint_as_it_grows() {
    let mut pv = FlipGpuData::default();
    pv.strokes.push(GpuStroke {
        first_point: 0,
        point_count: 0,
        flags: 0,
        hardness: 0.5,
        material: 0,
        tip: 0,
        dot_spacing: 0.0,
        ref_width: 8.0,
    });
    let empty = fingerprint(0xABCD, Some(&pv), &cam(), (1024, 768), true);
    assert_ne!(
        empty,
        fingerprint(0xABCD, None, &cam(), (1024, 768), true),
        "com preview e sem preview sao estados diferentes"
    );

    let push = |g: &mut FlipGpuData, x: f32| {
        g.points.push(GpuPoint {
            pos: [x, 10.0],
            width: 8.0,
            opacity: 1.0,
            color: [0.0, 0.0, 0.0, 1.0],
        });
        g.strokes[0].point_count = g.points.len() as u32;
    };
    push(&mut pv, 10.0);
    let one = fingerprint(0xABCD, Some(&pv), &cam(), (1024, 768), true);
    push(&mut pv, 20.0);
    let two = fingerprint(0xABCD, Some(&pv), &cam(), (1024, 768), true);
    assert_ne!(empty, one, "o 1o ponto do gesto move a impressao");
    assert_ne!(one, two, "cada ponto novo move a impressao");

    // ⚠️ E a MÃO PARADA volta a ser grátis — é o ganho inteiro da wave num traço em curso.
    assert_eq!(
        two,
        fingerprint(0xABCD, Some(&pv), &cam(), (1024, 768), true),
        "mao parada = mesma impressao = frame gratis"
    );
}

/// ⭐ **A LEI DO SKIP, e a metade que importa é a 2ª:** o memo bater não basta — o compositor tem de
/// AINDA ter a fatia.
#[test]
fn the_skip_needs_both_the_memo_and_the_compositors_own_word() {
    let mut memo = StageMemo::default();
    let (k, fp) = (7u64, 0xF00Du64);

    assert!(
        memo.needs_stage(k, fp, true),
        "1a vez: nada foi rasterizado ainda"
    );
    memo.record(k, fp);
    assert!(
        !memo.needs_stage(k, fp, true),
        "memo bate + o compositor tem a fatia => PULA"
    );

    // A fatia foi DESPEJADA (LRU) ou LIMPA (rebuild do array): o memo segue batendo e mente.
    assert!(
        memo.needs_stage(k, fp, false),
        "sem a fatia, o memo batendo mandaria compor lixo — tem de rasterizar"
    );

    // O conteúdo mudou.
    assert!(
        memo.needs_stage(k, 0xBEEF, true),
        "impressao diferente => rasteriza"
    );

    // Uma chave que nunca foi vista.
    assert!(memo.needs_stage(99, fp, true), "chave nova => rasteriza");
}

/// 📏 **SONDA — o que a própria impressão digital CUSTA.**
///
/// ⚠️ A pergunta que decide se a wave é lucro: ela roda por camada por frame, e no traço VIVO ela
/// varre os pontos (`O(n)`). Se custasse a ordem do que economiza, seria uma troca ruim disfarçada de
/// otimização. O número é comparado com **4,33 ms**, o custo medido de rasterizar UMA camada de 200
/// traços a 1080p (§21.2).
#[test]
#[ignore = "sonda; roda com --ignored --nocapture"]
fn measure_what_the_fingerprint_costs() {
    let c = cam();
    for n in [0_usize, 200, 2_000, 20_000] {
        let mut pv = FlipGpuData::default();
        pv.strokes.push(GpuStroke {
            first_point: 0,
            point_count: n as u32,
            flags: 0,
            hardness: 0.5,
            material: 0,
            tip: 0,
            dot_spacing: 0.0,
            ref_width: 8.0,
        });
        for i in 0..n {
            pv.points.push(GpuPoint {
                pos: [i as f32 * 0.5, 10.0],
                width: 8.0,
                opacity: 1.0,
                color: [0.0, 0.0, 0.0, 1.0],
            });
        }
        let alvo = if n == 0 { None } else { Some(&pv) };
        // 12 corridas, a 1ª descartada, MÍNIMO — toda amostra faz trabalho idêntico, então o mínimo
        // é o que a máquina consegue e o resto é carga alheia (a régua do §21.1).
        let mut melhor = f64::MAX;
        for it in 0..12 {
            let t0 = std::time::Instant::now();
            let mut acc = 0u64;
            for _ in 0..64 {
                acc ^= fingerprint(0xABCD, alvo, &c, (1920, 1080), true);
            }
            let dt = t0.elapsed().as_secs_f64() * 1000.0 / 64.0;
            if it > 0 {
                melhor = melhor.min(dt);
            }
            assert_ne!(
                acc,
                u64::MAX,
                "o compilador nao pode otimizar a chamada fora"
            );
        }
        println!("  preview de {n:6} pontos: {melhor:.4} ms por camada");
    }
}

/// As estatísticas contam o que aconteceu — é o que o `PH2D_FLIP_STATS=1` mostra no smoke, e sem
/// elas *"o cache está funcionando?"* é opinião.
#[test]
fn the_stats_count_staged_and_skipped() {
    let mut memo = StageMemo::default();
    memo.reset_stats();
    memo.needs_stage(1, 10, true);
    memo.record(1, 10);
    memo.needs_stage(1, 10, true);
    memo.needs_stage(1, 10, true);
    assert_eq!(memo.stats(), (1, 2), "1 rasterizada (a 1a vez) e 2 puladas");
    memo.reset_stats();
    assert_eq!(memo.stats(), (0, 0), "o reset zera por frame");
}

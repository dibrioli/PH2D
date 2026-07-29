//! **A CÉLULA do atlas dá o MESMO pixel que a forma sozinha** — o gate que autoriza a wave.
//!
//! O `fx_live` deixou de rasterizar uma forma por render do Vello e passa a rasterizar todas numa
//! textura partilhada, cada uma na sua célula (`shells/desktop/src/fx_atlas.rs`). A pilha de cada
//! forma lê a célula dela via [`FxStackPass::run_from`], e o deslocamento entra num sítio só: o
//! `cs_ingest`.
//!
//! ⚠️ **O oráculo é o produto ANTIGO.** Nada aqui descreve o que a saída "deve" parecer — ela é
//! comparada, byte a byte, com o que o `run` de sempre produz sobre a MESMA arte numa textura
//! própria. É isso que torna a wave uma mudança de *quando o trabalho é feito* e não de *o que ele
//! desenha*; um limiar de tolerância aceitaria justamente a classe de erro que um deslocamento
//! errado produz (a forma inteira arrastada por alguns texels).
//!
//! Rodar: `cargo test -p ph2d-render --test fx_stack_atlas_gpu -- --ignored --nocapture`.

use ph2d_ecs::FxOp;
use ph2d_render::{FxOpGpu, FxStackPass, make_output_texture};

mod fx_stack_common;
use fx_stack_common::{BLACK, make_src, op, readback, try_headless_gpu};

/// A célula da forma dentro do atlas — deslocada nos DOIS eixos e por números diferentes, senão um
/// `src_org.yx` trocado passaria.
const CELL: (u32, u32) = (37, 11);
/// O tamanho da forma (= o scratch dela).
const W: u32 = 96;
const H: u32 = 72;
/// O atlas: a célula mais a forma, com sobra à direita e em baixo (é o que um empacotador real
/// deixa, e um `textureLoad` fora dos limites teria clampado em silêncio se a sobra não existisse).
const AW: u32 = CELL.0 + W + 40;
const AH: u32 = CELL.1 + H + 25;

/// A arte de uma forma: um losango de cor, com **cobertura parcial** na borda.
///
/// ⚠️ A cobertura parcial é a fixture, não enfeite: num texel opaco e num vazio um deslocamento
/// errado ainda pode acertar por acaso (dentro da forma tudo é igual); é na RAMPA da borda que
/// cada texel tem um valor próprio, e é lá que um atlas mal lido se denuncia.
fn diamond(w: u32, h: u32, ox: u32, oy: u32, into: &mut [u8], stride: u32) {
    let (cx, cy) = (f64::from(w) * 0.5, f64::from(h) * 0.5);
    for y in 0..h {
        for x in 0..w {
            let d = ((f64::from(x) - cx).abs() / cx) + ((f64::from(y) - cy).abs() / cy);
            let a = (1.0 - d).clamp(0.0, 1.0);
            let o = (((oy + y) * stride + ox + x) * 4) as usize;
            into[o] = 235;
            into[o + 1] = 175;
            into[o + 2] = 60;
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            {
                into[o + 3] = (a * 255.0).round() as u8;
            }
        }
    }
}

/// Uma pilha que exercita as três famílias de plano ao mesmo tempo: um borrão com deslocamento
/// (kernel), um contorno (campo de distância) e um pontual. Se o ingest lesse a célula errada, as
/// três leriam a arte errada — mas cada uma falharia de um jeito, e o gate quer as três.
fn stack() -> Vec<FxOpGpu> {
    vec![
        FxOpGpu {
            offset_px: [5, -4],
            ..op(FxOp::DROP_SHADOW, 4.0, BLACK)
        },
        op(FxOp::OUTLINE, 2.0, [0.1, 0.9, 0.4, 1.0]),
        op(FxOp::COLOR_OVERLAY, 0.0, [0.2, 0.5, 1.0, 0.5]),
    ]
}

/// **O gate.** A mesma arte, filtrada de duas maneiras: sozinha numa textura sua (o produto de
/// ontem) e numa célula de um atlas (o produto de hoje). Os bytes têm de ser os mesmos.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn a_cell_of_the_atlas_filters_exactly_like_the_shape_alone() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx-atlas] sem adapter — skip");
        return;
    };
    // A fonte "sozinha": só a forma, na origem.
    let mut lone_bytes = vec![0u8; (W * H * 4) as usize];
    diamond(W, H, 0, 0, &mut lone_bytes, W);
    let lone = make_src(&gpu, W, H, &lone_bytes);

    // A fonte "atlas": a MESMA forma, na célula, com lixo à volta — o vizinho que um deslocamento
    // errado traria para dentro.
    let mut atlas_bytes = vec![0u8; (AW * AH * 4) as usize];
    for (i, b) in atlas_bytes.iter_mut().enumerate() {
        *b = u8::try_from((i * 7) % 251).unwrap_or(0);
    }
    diamond(W, H, CELL.0, CELL.1, &mut atlas_bytes, AW);
    let atlas = make_src(&gpu, AW, AH, &atlas_bytes);

    let ops = stack();
    let segs: [[f32; 4]; 0] = [];
    let mut pass = FxStackPass::new(&gpu);

    let a = make_output_texture(&gpu, W, H);
    pass.run(&gpu, &lone, &a, W, H, &ops, &segs);
    let want = readback(&gpu, &a, W, H);

    let b = make_output_texture(&gpu, W, H);
    let org = [
        i32::try_from(CELL.0).expect("cabe"),
        i32::try_from(CELL.1).expect("cabe"),
    ];
    pass.run_from(&gpu, &atlas, org, &b, W, H, &ops, &segs);
    let got = readback(&gpu, &b, W, H);

    let diff = want.iter().zip(&got).filter(|(x, y)| x != y).count();
    let worst = want
        .iter()
        .zip(&got)
        .map(|(x, y)| u32::from(x.abs_diff(*y)))
        .max()
        .unwrap_or(0);
    eprintln!(
        "[fx-atlas] {diff} de {} bytes diferem, pior delta {worst}",
        want.len()
    );
    assert_eq!(
        diff, 0,
        "a célula do atlas não deu o mesmo pixel que a forma sozinha \
         ({diff} bytes diferem, pior delta {worst})"
    );
    // Controle positivo: sem ele, um `readback` que devolvesse zeros faria o gate passar sobre
    // duas imagens VAZIAS. A arte tem de estar lá.
    assert!(
        want.iter().skip(3).step_by(4).any(|&a| a > 8),
        "a fixture não desenhou nada — o gate estaria a comparar dois vazios"
    );
}

/// **`run` é `run_from` com origem zero** — as duas portas não podem divergir, e a maneira de
/// garantir isso é uma delegar na outra. Este gate pina a delegação pelo comportamento.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn the_old_door_is_the_new_one_at_the_origin() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx-atlas] sem adapter — skip");
        return;
    };
    let mut bytes = vec![0u8; (W * H * 4) as usize];
    diamond(W, H, 0, 0, &mut bytes, W);
    let src = make_src(&gpu, W, H, &bytes);
    let ops = stack();
    let segs: [[f32; 4]; 0] = [];
    let mut pass = FxStackPass::new(&gpu);

    let a = make_output_texture(&gpu, W, H);
    pass.run(&gpu, &src, &a, W, H, &ops, &segs);
    let want = readback(&gpu, &a, W, H);

    let b = make_output_texture(&gpu, W, H);
    pass.run_from(&gpu, &src, [0, 0], &b, W, H, &ops, &segs);
    assert_eq!(
        want,
        readback(&gpu, &b, W, H),
        "`run` divergiu de `run_from([0,0])`"
    );
}

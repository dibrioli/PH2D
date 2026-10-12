//! ⭐⭐ **A VIGIA DAS IMAGENS** (B6 do Motion, `ph2d_gpu_cook::GpuCook::toma_imagens_vigiadas`) —
//! numa saída que mistura em grupo, uma linha de IMAGEM iria ao passe de sprites (luz linear, sem
//! alcance). A placa conta-as sem a ponte esperar por ela; o host recusa já a saída que é TODA
//! imagens.
//!
//! A fixtura é a do irmão ([`super::as_formas_no_dispositivo::fronteira`]): cinco linhas, uma delas
//! imagem (`geometry_id = 0`). Controlos: a vigia desligada conta zero; todas formas contam zero.
//!
//! `#[ignore]`: precisa de adaptador. Na pista da GPU:
//!   `PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-gpu-cook --test it \
//!        a_vigia_das_imagens --release -- --ignored --nocapture`

use super::as_formas_no_dispositivo::{fronteira, grafo, registry};
use super::placa::try_headless_gpu;
use ph2d_gpu::GpuContext;
use ph2d_gpu_cook::{CookClock, GpuCook, GpuCookError, plan};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_render::SinkStyle;

/// Coze a fronteira com a vigia dada.
fn coze(gpu: &GpuContext, gc: &mut GpuCook, vigia: bool, s: &Stream) -> Result<u32, GpuCookError> {
    let reg = registry();
    let (g, obj, _, out) = grafo();
    let p = plan(&g, &reg, &reg, out);
    gc.set_vigia_de_imagens(vec![vigia]);
    gc.cook(
        gpu,
        &g,
        &reg,
        &reg,
        &p,
        &[(obj, s)],
        CookClock {
            playhead: 0.0,
            tick: None,
        },
        [0.0, 0.0, 1.0, 1.0],
        [0.4, 0.4],
        SinkStyle::PLAIN,
    )
}

/// A leitura da vigia depois de a placa acabar (o produto lê-a num quadro seguinte, sem esperar;
/// o gate espera por ela).
fn toma(gpu: &GpuContext, gc: &mut GpuCook) -> u32 {
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    gc.toma_imagens_vigiadas(gpu)
}

/// ⭐⭐ **A placa conta a linha de imagem de uma saída vigiada**, e só dela; a leitura decide UMA
/// vez; a vigia desligada e uma saída toda de formas contam zero; e o `descarta` esquece o voo.
#[test]
#[ignore = "precisa de adaptador"]
fn a_placa_conta_as_imagens_de_uma_saida_vigiada() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem adaptador — saltado");
        return;
    };
    let mut gc = GpuCook::new();
    let mista = fronteira();
    let feito = coze(&gpu, &mut gc, true, &mista);
    assert!(
        feito.is_ok(),
        "uma saída mista coze (a vigia conta, não recusa)"
    );
    assert_eq!(toma(&gpu, &mut gc), 1, "uma linha de imagem entre cinco");
    assert_eq!(
        gc.toma_imagens_vigiadas(&gpu),
        0,
        "a leitura decide uma vez só"
    );

    // Controlo: a mesma corrente sem vigia não conta nada.
    coze(&gpu, &mut gc, false, &mista).expect("coze");
    assert_eq!(toma(&gpu, &mut gc), 0, "sem vigia a placa não conta");

    // Controlo: todas formas.
    let mut formas = fronteira();
    formas.set(
        "geometry_id",
        Column::Scalar(vec![11.0, 12.0, 14.0, 11.0, 13.0]),
    );
    coze(&gpu, &mut gc, true, &formas).expect("coze");
    assert_eq!(
        toma(&gpu, &mut gc),
        0,
        "uma saída só de formas não acende a vigia"
    );

    // O `descarta` (a rota caiu): a leitura em voo não decide o regresso.
    coze(&gpu, &mut gc, true, &mista).expect("coze");
    gc.descarta_a_vigia();
    assert_eq!(toma(&gpu, &mut gc), 0, "descartada, nada volta");
}

/// ⛔ **Uma saída vigiada SEM `geometry_id` é toda imagens: o host recusa já** — o controlo é a
/// mesma corrente sem vigia, que coze.
#[test]
#[ignore = "precisa de adaptador"]
fn uma_saida_vigiada_so_de_imagens_recusa_o_cozimento() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem adaptador — saltado");
        return;
    };
    let mut gc = GpuCook::new();
    // A mesma fronteira SEM a coluna `geometry_id`.
    let f = fronteira();
    let mut imagens = Stream::new(5);
    for c in ["P", "size", "rot", "tint"] {
        imagens.set(c, f.get(c).cloned().expect("a coluna da fixtura"));
    }
    assert!(
        coze(&gpu, &mut gc, false, &imagens).is_ok(),
        "sem vigia, as imagens vão às sprites"
    );
    assert_eq!(
        coze(&gpu, &mut gc, true, &imagens),
        Err(GpuCookError::ImagemEmGrupo)
    );
}

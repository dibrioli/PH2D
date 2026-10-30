//! Gates da F40 (`docs/UI_New_and_Simple/F38_plano.md`, cena 41 com a placa cheia): a memória da pilha
//! de FX nasce sob o escopo de memória. Recusada, a pilha não faz trabalho de placa nem volta a pedir
//! memória até ao fim do episódio; a saída recusada é a substituta e renasce depois dele.

use crate::renasce::testes::forca_recusas;
use crate::{FxOpGpu, FxStackPass, Origem, nasce_saida, saida_renasce};
use ph2d_gpu::GpuContext;
use ph2d_gpu::memoria::{JANELA_MS, VIGIA, geracao};

fn gpu() -> Option<GpuContext> {
    GpuContext::new(GpuContext::default_instance(), None).ok()
}

/// Um episódio de falta inteiro no vigia do processo: a geração sobe uma vez.
fn acaba_um_episodio() {
    let _ = VIGIA.mudou_em(u64::MAX / 2);
    let g = geracao();
    VIGIA.anota_em(0);
    assert_eq!(VIGIA.mudou_em(0), Some(true));
    assert_eq!(VIGIA.mudou_em(JANELA_MS), Some(false));
    assert_eq!(geracao(), g + 1, "o episódio devia ter acabado");
}

fn borrao() -> FxOpGpu {
    FxOpGpu {
        sigma_px: 3.0,
        ..FxOpGpu::resolve(
            &ph2d_fx_op::FxOp::new(ph2d_fx_op::FxOp::BLUR),
            [1.0, 0.0, 0.0, 1.0],
        )
    }
}

fn corre(
    gpu: &GpuContext,
    pilha: &mut FxStackPass,
    src: &wgpu::Texture,
    dst: &wgpu::Texture,
) -> bool {
    let r = pilha.run_from_com_origem(
        gpu,
        src,
        [0, 0],
        dst,
        64,
        48,
        &[borrao()],
        &[],
        Origem::Margem,
    );
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    r
}

/// **G30 — a pilha cujo conjunto de trabalho a placa recusou não faz trabalho de placa, não volta a
/// pedir memória no mesmo episódio e tenta de novo no fim dele** — zero `[gpu-erro]` pelo caminho.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn a_refused_fx_work_set_does_no_gpu_work_until_the_episode_ends() {
    let Some(gpu) = gpu() else {
        return;
    };
    let erros = ph2d_gpu::context::erros_relatados();
    let src = crate::make_output_texture(&gpu, 64, 48);
    let dst = crate::make_output_texture(&gpu, 64, 48);
    let mut pilha = FxStackPass::new(&gpu);
    forca_recusas(1);
    assert!(
        !corre(&gpu, &mut pilha, &src, &dst),
        "a corrida recusada fez trabalho"
    );
    assert!(pilha.em_recusa());
    assert_eq!(
        pilha.work_size(),
        None,
        "a recusa deixou texturas de trabalho"
    );
    assert!(
        !corre(&gpu, &mut pilha, &src, &dst),
        "o mesmo episódio voltou a correr"
    );
    assert_eq!(
        pilha.work_size(),
        None,
        "o mesmo episódio voltou a pedir memória"
    );
    acaba_um_episodio();
    assert!(!pilha.em_recusa());
    assert!(
        corre(&gpu, &mut pilha, &src, &dst),
        "o fim do episódio não voltou a correr"
    );
    assert!(pilha.work_size().is_some_and(|(w, h)| w >= 64 && h >= 48));
    assert_eq!(
        ph2d_gpu::context::erros_relatados(),
        erros,
        "a recusa deu [gpu-erro]"
    );
}

/// **G31 — a saída recusada é a substituta (a pilha não lhe faz trabalho nenhum) e renasce do seu
/// tamanho no fim do episódio**, sem `[gpu-erro]`.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn a_refused_fx_output_is_a_substitute_and_comes_back_after_the_episode() {
    let Some(gpu) = gpu() else {
        return;
    };
    let erros = ph2d_gpu::context::erros_relatados();
    let src = crate::make_output_texture(&gpu, 64, 48);
    forca_recusas(1);
    let (saida, recusada) = nasce_saida(&gpu, 64, 48);
    assert_eq!(recusada, Some(geracao()), "a recusa da saída não foi vista");
    assert_eq!(
        (saida.width(), saida.height()),
        (1, 1),
        "não é a substituta"
    );
    assert!(!saida_renasce(recusada), "renasceu dentro do episódio");
    let mut pilha = FxStackPass::new(&gpu);
    assert!(
        !corre(&gpu, &mut pilha, &src, &saida),
        "a pilha trabalhou sobre a substituta"
    );
    assert_eq!(
        pilha.work_size(),
        None,
        "a substituta pediu memória de trabalho"
    );
    acaba_um_episodio();
    assert!(saida_renasce(recusada), "o fim do episódio não a refaz");
    let (saida, recusada) = nasce_saida(&gpu, 64, 48);
    assert_eq!((saida.width(), saida.height(), recusada), (64, 48, None));
    assert!(corre(&gpu, &mut pilha, &src, &saida));
    assert_eq!(
        ph2d_gpu::context::erros_relatados(),
        erros,
        "a recusa deu [gpu-erro]"
    );
}

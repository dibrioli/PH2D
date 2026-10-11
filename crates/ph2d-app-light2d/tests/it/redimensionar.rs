//! ⭐⭐ **Redimensionar a janela não recompila a luz** (bug do dono, 11/10: com a luz ligada o app
//! custava a MAXIMIZAR e a encaixar na tela). Medido: a grade da luz segue o tamanho da janela e cada
//! tamanho novo recriava o solver — ~2 100 ms por quadro, dos quais ~2 090 a recompilar as pipelines
//! da C3; o maximizar muda o tamanho a cada quadro. A lei: a grade nova só realoca (texturas,
//! buffers); as pipelines — que dependem só do shader e das constantes — vêm da `Fabrica` da luz.
//!
//! A régua não é o relógio (a máquina é partilhada): é quantas pipelines o solver de cada tamanho
//! novo COMPILOU ao nascer, pelo caminho do produto (`LuzDoQuadro::preparar` + `gravar`), num
//! maximizar de 12 quadros. O relógio fica impresso ao lado.

use bevy_ecs::world::World;
use ph2d_app_light2d::{DEGRAU_PADRAO, LuzDoQuadro, smoke};
use ph2d_ecs::PresentWorld;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use std::time::Instant;

use crate::comum::gpu;

/// Um quadro do produto no tamanho `tam`: `(ms com a espera da placa, a grade mudou, pipelines que o
/// solver deste quadro compilou ao nascer)`.
fn quadro(
    gpu: &ph2d_gpu::GpuContext,
    l: &mut LuzDoQuadro,
    w: &World,
    r: &ph2d_render::SpriteRenderer,
    tam: (u32, u32),
) -> (f64, bool, u64) {
    let afim = Camera2d::default().world_to_screen_affine(WindowSize::new(tam.0, tam.1));
    let antes = l.grade();
    let t = Instant::now();
    l.preparar(w, afim, tam);
    l.gravar(gpu, &mut PresentWorld::new(), r);
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    let ms = t.elapsed().as_secs_f64() * 1e3;
    let mudou = antes.map(|g| (g.largura, g.altura)) != l.grade().map(|g| (g.largura, g.altura));
    let compiladas = l.solver().map_or(0, |s| s.pipelines_compiladas_ao_nascer());
    (ms, mudou, compiladas)
}

/// ⭐⭐ **GATE — um tamanho novo de janela não compila nenhuma pipeline.**
#[test]
fn redimensionar_a_janela_nao_recompila_a_luz() {
    let Some(gpu) = gpu() else {
        eprintln!("sem GPU — gate saltado");
        return;
    };
    let mut w = World::new();
    assert!(smoke::encenar(&mut w, 3));
    let r = crate::comum::renderer(&gpu);
    let mut l = LuzDoQuadro::com_degrau(DEGRAU_PADRAO);
    let (_, _, primeiras) = quadro(&gpu, &mut l, &w, &r, (1024, 768));
    assert!(
        primeiras > 0,
        "o primeiro solver compila as pipelines ({primeiras})"
    );
    // O maximizar: 12 quadros de 1024×768 a 1920×1040 (o compositor anima o tamanho).
    let (mut grades_novas, mut compiladas, mut ms) = (0, 0, Vec::new());
    for k in 1..=12u32 {
        let tam = (1024 + (1920 - 1024) * k / 12, 768 + (1040 - 768) * k / 12);
        let (t, mudou, c) = quadro(&gpu, &mut l, &w, &r, tam);
        if mudou {
            grades_novas += 1;
            compiladas += c;
        }
        ms.push(format!("{t:.0}"));
    }
    eprintln!(
        "maximizar: {grades_novas} grades novas, {compiladas} pipelines compiladas (o 1.º solver: {primeiras}); ms por quadro: {}",
        ms.join(" ")
    );
    assert!(
        grades_novas >= 6,
        "a régua não mudou a grade ({grades_novas})"
    );
    assert_eq!(
        compiladas, 0,
        "redimensionar recompilou a luz: {compiladas} pipelines em {grades_novas} grades novas"
    );
}

//! ⭐⭐ W5i — **a luz nasce na ESTUFA, nunca no fio do quadro** (smoke do dono, 11/10: «o app demora a
//! abrir»; W5f §3: o 1.º `gravar_com` custava 3,5 s, o solver a compilar as pipelines com a janela
//! presa; e cada nível novo 2–3 s). Com a montagem em fundo, o 1.º quadro sai SEM luz e volta logo; a
//! luz acende quando a estufa acaba; e com a estufa ociosa a troca de nível monta no próprio quadro
//! sem compilar NENHUMA pipeline (a estufa aqueceu a configuração de cada nível numa grade mínima — a
//! régua de que a grade não entra nas pipelines).

use std::time::{Duration, Instant};

use bevy_ecs::world::World;
use ph2d_app_light2d::{DEGRAU_PADRAO, LuzDoQuadro, smoke};
use ph2d_ecs::PresentWorld;
use ph2d_editor_core::luz_edits::{NivelDaLuz, PreferenciaDaLuz};
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;

use crate::comum::gpu;

const TAM: (u32, u32) = (1280, 720);

/// Um quadro do produto: os ms do `preparar` + `gravar` (sem esperar a placa).
fn quadro(
    gpu: &ph2d_gpu::GpuContext,
    l: &mut LuzDoQuadro,
    w: &World,
    r: &ph2d_render::SpriteRenderer,
) -> f64 {
    let afim = Camera2d::default().world_to_screen_affine(WindowSize::new(TAM.0, TAM.1));
    let t = Instant::now();
    l.preparar(w, afim, TAM);
    l.gravar(gpu, &mut PresentWorld::new(), r);
    t.elapsed().as_secs_f64() * 1e3
}

#[test]
fn a_luz_nasce_na_estufa_e_a_troca_de_nivel_nao_compila() {
    let Some(gpu) = gpu() else {
        eprintln!("sem GPU — gate saltado");
        return;
    };
    let mut w = World::new();
    assert!(smoke::encenar(&mut w, 3));
    let r = crate::comum::renderer(&gpu);

    // Controlo: sem a estufa, o 1.º quadro monta e compila no próprio fio.
    let mut sincrona = LuzDoQuadro::com_degrau(DEGRAU_PADRAO);
    let ms_sincrona = quadro(&gpu, &mut sincrona, &w, &r);
    let compiladas = sincrona
        .solver()
        .map_or(0, |s| s.pipelines_compiladas_ao_nascer());
    assert!(
        sincrona.acesa() && compiladas > 0,
        "o controlo não compilou no quadro ({compiladas})"
    );

    let mut l = LuzDoQuadro::com_degrau(DEGRAU_PADRAO).com_montagem_em_fundo();
    let ms_1 = quadro(&gpu, &mut l, &w, &r);
    assert!(
        l.aquecendo() && !l.acesa() && l.solver().is_none(),
        "o 1.º quadro a frio tem de sair sem luz, à espera da estufa"
    );
    // A luz acende quando a estufa entrega (o quadro segue a correr).
    let (mut pior, mut quadros) = (0.0_f64, 1);
    let prazo = Instant::now() + Duration::from_secs(120);
    while !l.acesa() && Instant::now() < prazo {
        std::thread::sleep(Duration::from_millis(10));
        pior = pior.max(quadro(&gpu, &mut l, &w, &r));
        quadros += 1;
    }
    assert!(l.acesa(), "a luz não acendeu em 120 s de estufa");
    while !l.estufa_ociosa() && Instant::now() < prazo {
        std::thread::sleep(Duration::from_millis(10));
        quadro(&gpu, &mut l, &w, &r);
    }
    assert!(
        l.estufa_ociosa(),
        "a estufa não acabou os níveis adiantados"
    );
    eprintln!(
        "sem estufa: 1.º quadro {ms_sincrona:.0} ms ({compiladas} pipelines); com ela: 1.º {ms_1:.0} ms, \
         acesa em {quadros} quadros, o pior {pior:.0} ms"
    );

    // Com a estufa ociosa, cada nível monta no quadro sem compilar nada.
    for nivel in [NivelDaLuz::Baixa, NivelDaLuz::Media, NivelDaLuz::Alta] {
        l.preferir(PreferenciaDaLuz {
            nivel,
            automatica: false,
        });
        quadro(&gpu, &mut l, &w, &r);
        let c = l
            .solver()
            .map_or(u64::MAX, |s| s.pipelines_compiladas_ao_nascer());
        assert!(
            l.acesa() && c == 0,
            "{nivel:?}: a troca de nível compilou no quadro ({c} pipelines; acesa {})",
            l.acesa()
        );
    }
}

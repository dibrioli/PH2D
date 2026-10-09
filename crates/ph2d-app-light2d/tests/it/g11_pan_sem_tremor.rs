//! **G11 — sem tremor sob pan** (plano §5, S6): a câmera anda em passos SUB-texel e a luz lida em
//! pontos FIXOS do mundo (na penumbra e no claro) fica parada — a grade encaixa a origem na rede da
//! cascata de topo, e as sondas caem nos mesmos pontos do mundo.
//!
//! ⛔ A grade NÃO ancorada (a mesma, com a origem a seguir a câmera) treme: é o controlo.

use bevy_ecs::world::World;
use half::f16;
use ph2d_app_light2d::{DEGRAU_PADRAO, LuzDoQuadro, grade_do_quadro};
use ph2d_ecs::{GlobalTransform, Transform};
use ph2d_gpu::GpuContext;
use ph2d_host::WindowSize;
use ph2d_light2d_ecs::{BloqueiaLuz, Luz, TipoDeLuz};
use ph2d_light2d_gpu::{Config, Solver};
use ph2d_render::{Camera2d, Sprite};
use vello::kurbo::Affine;

use crate::comum::amostra as amostrar;
use crate::comum::{gpu, ler};

const ALVO: (u32, u32) = (256, 128);
/// 32 px/m: com `c0 = 2 px` o texel é 1/16 m.
const ALTURA_M: f32 = 4.0;

/// **O limiar MEDIDO** (RTX 5060 Ti, 2026-10-09): ancorada **0** no degrau padrão (texel 4 px) e
/// **8,1e-15** no de 2 px — a fluência nos pontos
/// estáticos sai BIT-idêntica de quadro para quadro, e o resto é o arredondamento `f64` da amostragem
/// na CPU; solta **4,9** (padrão) / **2,6** (2 px), relativo, na penumbra. O limiar fica muito abaixo de um ulp de `f16`
/// (~1e-3): qualquer mudança na fluência guardada reprova.
const LIMIAR: f64 = 1e-9;

fn afim(centro: [f32; 2]) -> Affine {
    Camera2d::new(centro, ALTURA_M).world_to_screen_affine(WindowSize::new(ALVO.0, ALVO.1))
}

fn cena() -> World {
    let mut w = World::new();
    let pose = |x: f32, y: f32| {
        let mut t = Transform::IDENTITY;
        t.translation.x = x;
        t.translation.y = y;
        (t, GlobalTransform::from_transform(t))
    };
    let (t, g) = pose(-1.5, 0.3);
    w.spawn((
        Luz {
            cor: [1.0, 0.9, 0.7],
            intensidade: 6.0,
            tamanho_m: 0.15,
            alcance_m: f32::INFINITY,
            tipo: TipoDeLuz::Ponto,
        },
        t,
        g,
    ));
    // Uma parede a meio caminho: a penumbra cai sobre os pontos da direita.
    let (t, g) = pose(-0.6, 0.0);
    w.spawn((
        Sprite::atlas(0, [0.2, 0.9], [1.0; 4]),
        BloqueiaLuz { opacidade: 1.0 },
        t,
        g,
    ));
    w
}

/// Pontos estáticos: uma fila que atravessa a sombra, a penumbra e o claro.
fn pontos() -> Vec<[f64; 2]> {
    (0..24)
        .map(|k| [0.2 + 0.03 * f64::from(k % 6), -0.6 + 0.05 * f64::from(k)])
        .collect()
}

fn ler_fluencia(gpu: &GpuContext, s: &Solver) -> Vec<[f32; 3]> {
    ler(gpu, s.fluencia(), 8)
        .as_chunks::<8>()
        .0
        .iter()
        .map(|c| {
            let h = |k: usize| f16::from_le_bytes([c[2 * k], c[2 * k + 1]]).to_f32();
            [h(0), h(1), h(2)]
        })
        .collect()
}

/// A maior variação RELATIVA de um ponto estático ao longo do pan, nas duas grades.
fn variacoes(gpu: &GpuContext) -> (f64, f64) {
    let w = cena();
    let renderer = crate::comum::renderer(gpu);
    let mut ancorada = LuzDoQuadro::new();
    let mut solta: Option<Solver> = None;
    let (mut ref_a, mut ref_s): (Vec<f64>, Vec<f64>) = (vec![], vec![]);
    let (mut pior_a, mut pior_s) = (0.0_f64, 0.0_f64);
    for passo in 0..24 {
        // 0,37 de texel por quadro: sub-texel, e atravessa vários blocos da cascata de topo.
        let centro = [0.37 * 0.0625 * passo as f32, 0.11 * 0.0625 * passo as f32];
        assert!(ancorada.preparar(&w, afim(centro), ALVO));
        ancorada.gravar(gpu, &mut ph2d_ecs::PresentWorld::new(), &renderer);
        let g = ancorada.grade().expect("acesa");
        let fa = ler_fluencia(gpu, ancorada.solver().expect("solver"));

        // A MESMA grade com a origem a seguir a câmera (sem encaixe).
        let mut gs = grade_do_quadro(afim(centro), ALVO, DEGRAU_PADRAO).expect("grade");
        gs.origem_m = [
            f64::from(centro[0]) - f64::from(gs.largura) * gs.texel_m / 2.0,
            f64::from(centro[1]) - f64::from(gs.altura) * gs.texel_m / 2.0,
        ];
        let s = solta.get_or_insert_with(|| {
            Solver::novo(gpu, Config::padrao_para(&gpu.device, gs)).expect("solver solto")
        });
        s.reposicionar(gs.origem_m, gs.texel_m)
            .expect("reposicionar");
        s.quadro(gpu, ancorada.cena());
        let fs = ler_fluencia(gpu, s);

        let va: Vec<f64> = pontos().iter().map(|&p| amostrar(&fa, g, p)).collect();
        let vs: Vec<f64> = pontos().iter().map(|&p| amostrar(&fs, gs, p)).collect();
        if passo == 0 {
            assert!(
                va.iter().any(|&v| v > 0.05) && va.iter().any(|&v| v < 0.5 * va[va.len() - 1]),
                "controlo: os pontos têm de cruzar luz e penumbra ({va:?})"
            );
            (ref_a, ref_s) = (va, vs);
            continue;
        }
        let rel = |v: &[f64], r: &[f64]| {
            v.iter()
                .zip(r)
                .map(|(a, b)| (a - b).abs() / b.abs().max(0.05))
                .fold(0.0, f64::max)
        };
        pior_a = pior_a.max(rel(&va, &ref_a));
        pior_s = pior_s.max(rel(&vs, &ref_s));
    }
    (pior_a, pior_s)
}

#[test]
fn g11_um_ponto_estatico_nao_treme_sob_pan_sub_texel() {
    let Some(gpu) = gpu() else {
        eprintln!("g11: sem placa — saltado");
        return;
    };
    let (ancorada, solta) = variacoes(&gpu);
    eprintln!("g11: variação relativa máxima — ancorada {ancorada:.2e}, solta {solta:.2e}");
    assert!(
        ancorada <= LIMIAR,
        "a grade ANCORADA tremeu {ancorada:.2e} (> {LIMIAR:.0e}) num ponto estático"
    );
    assert!(
        solta > 10.0 * LIMIAR,
        "controlo: a grade SOLTA devia tremer (variou só {solta:.2e}) — o gate não separa as duas"
    );
}

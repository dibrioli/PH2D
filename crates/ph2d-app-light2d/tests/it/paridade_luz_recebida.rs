//! **A paridade da luz RECEBIDA** (DIRETIVA §4: efeito perceptual fecha com paridade numérica): um
//! sprite BRANCO cobre a vista sob uma cena de discos e polígonos; cada pixel do `game_rt` é a
//! `luz_em` que o `receber.wgsl` calculou na placa. Duas réguas:
//!
//! 1. **a conta do receptor** — a placa × a MESMA conta na CPU sobre a fluência lida de volta
//!    (`F/2π`, bilinear entre centros): mede a transformação mundo → grade, a amostragem e o fator;
//! 2. **a resposta** — a placa × o gabarito (`ph2d-light2d-ref::luz_em`): o erro do solver na
//!    grade do editor, na régua do G3 (RMSE relativo).
//!
//! E o `NaoRecebeLuz`, ponta a ponta: dois sprites na cena (pelo `PresentWorld`, casados pelo
//! `SimRef`), só o marcado sai na cor de sempre sob uma sala escura.

use std::f64::consts::TAU;

use bevy_ecs::world::World;
use half::f16;
use ph2d_app_light2d::LuzDoQuadro;
use ph2d_ecs::{GlobalTransform, PresentWorld, SimRef, Transform};
use ph2d_host::WindowSize;
use ph2d_light2d_ecs::{BloqueiaLuz, Luz, NaoRecebeLuz, TipoDeLuz};
use ph2d_render::{
    Camera2d, GameRt, LiftedInstances, RenderInstance, Sprite, SpriteRenderer, TextureAtlas,
};
use vello::kurbo::Affine;

use crate::comum::{Cadeia, H, W, afim, gpu, ler, quad};

const ALVO: (u32, u32) = (256, 128);
/// 32 px/m: a vista `x ∈ [−4, 4]`, `y ∈ [−2, 2]`; `c0 = 2 px` ⇒ texel 1/16 m.
const ALTURA_M: f32 = 4.0;

/// **Tolerâncias MEDIDAS + 10 %** (RTX 5060 Ti, Vulkan, 2026-10-09; o número que este teste imprime).
/// A conta: medido `1,21e-3` — o `game_rt` é `f16` (meio ulp relativo ≈ 4,9e-4) e o filtro bilinear
/// da placa tem 8 bits de fração. A resposta: medido `0,0475` (1 786 pontos) — a régua do G3, cuja
/// penumbra no degrau Alto mede `0,090`.
const TOL_CONTA_REL: f64 = 1.35e-3;
const TOL_RESPOSTA_RMSE: f64 = 0.0525;

fn janela() -> WindowSize {
    WindowSize::new(ALVO.0, ALVO.1)
}

fn afim_alvo() -> Affine {
    Camera2d::new([0.0, 0.0], ALTURA_M).world_to_screen_affine(janela())
}

fn pose(x: f32, y: f32, rot: f32) -> (Transform, GlobalTransform) {
    let mut t = Transform::IDENTITY;
    t.translation.x = x;
    t.translation.y = y;
    t.rotation = rot;
    (t, GlobalTransform::from_transform(t))
}

/// Dois discos de cores diferentes, uma parede opaca rodada e um vidro a 50 %.
fn cena() -> World {
    let mut w = World::new();
    let luz = |cor: [f32; 3], i: f32, r: f32| Luz {
        cor,
        intensidade: i,
        tamanho_m: r,
        alcance_m: f32::INFINITY,
        tipo: TipoDeLuz::Ponto,
    };
    let (t, g) = pose(-2.6, 0.8, 0.0);
    w.spawn((luz([1.0, 0.85, 0.6], 8.0, 0.3), t, g));
    let (t, g) = pose(2.4, -1.0, 0.0);
    w.spawn((luz([0.4, 0.6, 1.0], 5.0, 0.25), t, g));
    let (t, g) = pose(-0.8, 0.2, 0.35);
    w.spawn((
        Sprite::atlas(0, [0.25, 1.6], [1.0; 4]),
        BloqueiaLuz { opacidade: 1.0 },
        t,
        g,
    ));
    let (t, g) = pose(1.0, 0.4, -0.2);
    w.spawn((
        Sprite::atlas(0, [1.2, 0.2], [1.0; 4]),
        BloqueiaLuz { opacidade: 0.5 },
        t,
        g,
    ));
    w
}

/// O centro do pixel `(x, y)` no mundo.
fn mundo_do_pixel(x: u32, y: u32) -> [f64; 2] {
    let p =
        afim_alvo().inverse() * vello::kurbo::Point::new(f64::from(x) + 0.5, f64::from(y) + 0.5);
    [p.x, p.y]
}

fn ler_rgba16(gpu: &ph2d_gpu::GpuContext, t: &wgpu::Texture) -> Vec<[f64; 3]> {
    ler(gpu, t, 8)
        .chunks_exact(8)
        .map(|c| {
            let h = |k: usize| f64::from(f16::from_le_bytes([c[2 * k], c[2 * k + 1]]).to_f32());
            [h(0), h(1), h(2)]
        })
        .collect()
}

/// O pixel está DENTRO de um objeto (ou a menos de 2 texels dele)? Lá a resposta muda de natureza
/// (o emissor é opaco, o interior de um oclusor não é medido) — fora da régua, como no G3.
fn perto_de_objeto(c: &ph2d_light2d_ecs::CenaDeLuz, p: [f64; 2]) -> bool {
    let folga = 2.0 / 16.0;
    let em_poligono = |v: &[[f32; 2]]| {
        // A caixa do polígono com folga: conservador e suficiente para excluir a borda.
        let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
        for q in v {
            x0 = x0.min(f64::from(q[0]));
            y0 = y0.min(f64::from(q[1]));
            x1 = x1.max(f64::from(q[0]));
            y1 = y1.max(f64::from(q[1]));
        }
        p[0] > x0 - folga && p[0] < x1 + folga && p[1] > y0 - folga && p[1] < y1 + folga
    };
    c.oclusores.iter().any(|o| em_poligono(&o.vertices))
        || c.emissores.iter().any(|e| match e {
            ph2d_light2d_ecs::Emissor::Disco { centro, raio, .. } => {
                (p[0] - f64::from(centro[0])).hypot(p[1] - f64::from(centro[1]))
                    < f64::from(*raio) + folga
            }
            ph2d_light2d_ecs::Emissor::Poligono { vertices, .. } => em_poligono(vertices),
            ph2d_light2d_ecs::Emissor::Ponto { .. } => false,
        })
}

#[test]
fn a_luz_recebida_na_placa_bate_a_conta_e_o_gabarito() {
    let Some(gpu) = gpu() else {
        eprintln!("paridade: sem placa — saltado");
        return;
    };
    let w = cena();
    let mut luz = LuzDoQuadro::new();
    assert!(luz.preparar(&w, afim_alvo(), ALVO));
    // O sprite branco que cobre a vista, pelo passe da CENA.
    let mut renderer = SpriteRenderer::new(
        gpu.clone(),
        wgpu::TextureFormat::Rgba16Float,
        TextureAtlas::new(&gpu, 256),
        4,
    );
    luz.gravar(&gpu, &mut PresentWorld::new(), &renderer);
    let g = luz.grade().expect("acesa");
    let branco = renderer
        .acquire_individual(1, 1, &[255, 255, 255, 255])
        .expect("branco");
    let mut fundo = quad(branco, [0.0, 0.0]);
    fundo.size = [8.0, 4.0];
    let mut extra = LiftedInstances::default();
    extra.push(fundo, None);
    renderer.receber_luz(luz.recebida().as_ref());
    let game = GameRt::new(&gpu, ALVO);
    renderer.render_with_streams(
        game.view(),
        &mut PresentWorld::new(),
        &Camera2d::new([0.0, 0.0], ALTURA_M),
        janela(),
        wgpu::Color::TRANSPARENT,
        &extra,
        None,
        None,
        None,
        None,
    );
    let placa = ler_rgba16(&gpu, game.texture());
    let fluencia = ler_rgba16(&gpu, luz.solver().expect("solver").fluencia());

    // 1. A conta do receptor, em todo pixel.
    let bilinear = |p: [f64; 2], k: usize| {
        let tx = (p[0] - g.origem_m[0]) / g.texel_m - 0.5;
        let ty = (p[1] - g.origem_m[1]) / g.texel_m - 0.5;
        let (i0, j0) = (tx.floor(), ty.floor());
        let (fx, fy) = (tx - i0, ty - j0);
        let em = |i: f64, j: f64| {
            let i = (i.max(0.0) as u32).min(g.largura - 1);
            let j = (j.max(0.0) as u32).min(g.altura - 1);
            fluencia[(j * g.largura + i) as usize][k]
        };
        let a = em(i0, j0) * (1.0 - fx) + em(i0 + 1.0, j0) * fx;
        let b = em(i0, j0 + 1.0) * (1.0 - fx) + em(i0 + 1.0, j0 + 1.0) * fx;
        (a * (1.0 - fy) + b * fy) / TAU
    };
    let mut pior_conta = 0.0_f64;
    for y in 0..ALVO.1 {
        for x in 0..ALVO.0 {
            let p = mundo_do_pixel(x, y);
            for k in 0..3 {
                let cpu = bilinear(p, k);
                let gpu_v = placa[(y * ALVO.0 + x) as usize][k];
                pior_conta = pior_conta.max((gpu_v - cpu).abs() / cpu.max(0.02));
            }
        }
    }

    // 2. A resposta contra o gabarito, num pixel a cada 4 (fora dos objetos).
    let (mut soma_err2, mut soma_ref, mut n) = (0.0_f64, 0.0_f64, 0u32);
    for y in (2..ALVO.1).step_by(4) {
        for x in (2..ALVO.0).step_by(4) {
            let p = mundo_do_pixel(x, y);
            if perto_de_objeto(luz.cena(), p) {
                continue;
            }
            let r = ph2d_light2d_ref::luz_em(luz.cena(), p);
            for k in 0..3 {
                let e = placa[(y * ALVO.0 + x) as usize][k] - r[k];
                soma_err2 += e * e;
                soma_ref += r[k];
            }
            n += 3;
        }
    }
    let rmse = (soma_err2 / f64::from(n)).sqrt() / (soma_ref / f64::from(n));
    eprintln!(
        "paridade: conta {pior_conta:.2e} (relativo, pior pixel) · gabarito RMSE relativo {rmse:.4} \
         sobre {} pontos",
        n / 3
    );
    assert!(
        pior_conta <= TOL_CONTA_REL,
        "a luz recebida na placa afasta-se da conta da CPU {pior_conta:.2e}"
    );
    assert!(
        rmse <= TOL_RESPOSTA_RMSE,
        "a luz recebida afasta-se do gabarito: RMSE relativo {rmse:.4}"
    );
}

/// `NaoRecebeLuz`: numa sala escura (uma `Luz` a zero), o sprite marcado sai na cor de sempre e o
/// outro apaga. Os dois chegam pela CENA (`PresentWorld` + `SimRef`), como no editor.
#[test]
fn quem_nao_recebe_luz_fica_na_cor_de_sempre() {
    let Some(gpu) = gpu() else {
        eprintln!("nao_recebe: sem placa — saltado");
        return;
    };
    const COR: [u8; 4] = [180, 200, 90, 255];
    let mut w = World::new();
    let (t, g) = pose(0.0, 0.0, 0.0);
    w.spawn((
        Luz {
            cor: [1.0; 3],
            intensidade: 0.0,
            tamanho_m: 0.2,
            alcance_m: f32::INFINITY,
            tipo: TipoDeLuz::Ponto,
        },
        t,
        g,
    ));
    let recebe = w.spawn((Sprite::atlas(0, [1.0; 2], [1.0; 4]), t, g)).id();
    let nao = w
        .spawn((Sprite::atlas(0, [1.0; 2], [1.0; 4]), t, g, NaoRecebeLuz))
        .id();

    let mut c = Cadeia::nova(&gpu);
    let tex = c.textura(COR);
    let mut present = PresentWorld::new();
    present
        .world_mut()
        .spawn((quad(tex, [-1.0, 0.0]), SimRef(recebe)));
    present
        .world_mut()
        .spawn((quad(tex, [1.0, 0.0]), SimRef(nao)));
    let mut luz = LuzDoQuadro::new();
    assert!(luz.preparar(&w, afim(), (W, H)));
    luz.gravar(&gpu, &mut present, &c.renderer);
    luz.ligar(&gpu, &mut c.renderer, &mut c.colagem, &c.tonemap);
    let bgra = c.quadro(
        &gpu,
        &mut present,
        &LiftedInstances::default(),
        &vello::Scene::new(),
    );
    let px = |x: u32, y: u32| {
        let i = ((y * W + x) * 4) as usize;
        [bgra[i + 2], bgra[i + 1], bgra[i]]
    };
    assert_eq!(
        px(48, 16),
        [COR[0], COR[1], COR[2]],
        "o marcado tem de sair intacto"
    );
    assert!(
        px(16, 16).iter().all(|&c| c < 8),
        "o outro, sob uma luz a zero, apaga: {:?}",
        px(16, 16)
    );
    // E a interface que se marca a si mesma (barras do HUD) também passa intacta.
    let mut hud = quad(tex, [0.0, 0.6]);
    hud.size = [0.5, 0.25];
    hud.flip_uv |= RenderInstance::SEM_LUZ_BIT;
    let mut extra = LiftedInstances::default();
    extra.push(hud, None);
    let bgra = c.quadro(&gpu, &mut present, &extra, &vello::Scene::new());
    let i = ((((H / 2) - 10) * W + W / 2) * 4) as usize;
    assert_eq!(
        [bgra[i + 2], bgra[i + 1], bgra[i]],
        [COR[0], COR[1], COR[2]],
        "a barra do HUD (marcada pelo produtor) não recebe a luz"
    );
}

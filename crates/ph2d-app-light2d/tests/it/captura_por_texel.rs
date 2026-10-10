//! **A captura de sprite POR TEXEL** (W3; o gap da W1) — o alfa de cada texel é a cobertura, a cor
//! × o multiplicador é a emissão do dono, nos MESMOS três alvos da C0. Ponta a ponta: o sprite entra
//! pela instância que o passe de sprite desenha (`PresentWorld` + `SimRef`), e a fluência que sai é
//! comparada com a de uma cena de POLÍGONOS equivalente (a régua da C0 da W2, já medida contra o
//! gabarito no G3).
//!
//! 1. opaco, rodado e espelhado ⇒ o mesmo que o QUAD dele (a instância → mundo bate a coleta);
//! 2. com uma FENDA transparente de lado a lado ⇒ a luz passa como entre dois retângulos — e o
//!    quad (o comportamento de antes) é o controlo que reprova. ⚠️ Em 2D um buraco FECHADO no meio
//!    do sprite é uma cavidade: nenhum raio de fora lá chega (medido: as três cenas davam a mesma
//!    fluência) — a pergunta é a fenda;
//! 3. emissor de duas cores ⇒ dois retângulos emissores, cada um com a sua cor.

use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use ph2d_app_light2d::{LuzDoQuadro, regiao_da_janela};
use ph2d_ecs::{GlobalTransform, PresentWorld, SimRef, SpriteEmissive, Transform};
use ph2d_host::WindowSize;
use ph2d_light2d_ecs::{BloqueiaLuz, CenaDeLuz, Emissor, Luz, Oclusor, TipoDeLuz};
use ph2d_light2d_gpu::{Config, Grade, Solver};
use ph2d_render::{Camera2d, RenderInstance, Sprite, SpriteRenderer};
use vello::kurbo::Affine;

use crate::comum::{fluencia_lida, gpu, quad, renderer};

const ALVO: (u32, u32) = (256, 128);
const ALTURA_M: f32 = 4.0;

/// **Tolerâncias MEDIDAS + 10 %** (RTX 5060 Ti, Vulkan, 2026-10-09; os números que este teste
/// imprime, no degrau `ALTO` das leis), RMSE relativo da fluência atrás do objeto. Medido: opaco × quad `0,0000` (a mesma
/// cobertura subamostra a subamostra; o piso é o `f16`); fenda `0,0131` (o quad: `0,611`); duas cores
/// `0,0036`.
const TOL_QUAD: f64 = 1e-3;
/// ⚠️ Desde a luz DIRETA (C3, W4) a fenda mede `0,0182`: as cascatas borravam a diferença entre as
/// duas capturas (a textura por texel × as duas metades analíticas), e o caminho direto é nítido — a
/// diferença é da captura, não da luz (o quad: `0,661`, 36× acima). W4b: a régua passou a medir só a
/// REGIÃO da janela (a única que o quadro calcula) — a mesma placa mede `0,0247` sem a grade fina e
/// `0,0233` com ela (o quad: `0,905`): mudou a população (a grade longe da fenda diluía o erro
/// relativo), não a lei.
const TOL_FENDA: f64 = 0.026;
const TOL_CORES: f64 = 0.004;

fn afim() -> Affine {
    Camera2d::new([0.0, 0.0], ALTURA_M).world_to_screen_affine(WindowSize::new(ALVO.0, ALVO.1))
}

fn pose(x: f32, y: f32, rot: f32) -> (Transform, GlobalTransform) {
    let mut t = Transform::IDENTITY;
    t.translation.x = x;
    t.translation.y = y;
    t.rotation = rot;
    (t, GlobalTransform::from_transform(t))
}

fn luz_a_esquerda(w: &mut World) {
    let (t, g) = pose(-2.5, 0.1, 0.0);
    w.spawn((
        Luz {
            cor: [1.0, 0.95, 0.85],
            intensidade: 8.0,
            tamanho_m: 0.2,
            alcance_m: f32::INFINITY,
            tipo: TipoDeLuz::Ponto,
        },
        t,
        g,
    ));
}

/// A instância que o extract da shell monta para `sprite` em `g` (a base é a do `GlobalTransform`).
fn instancia(tex: u32, sprite: &Sprite, g: &GlobalTransform, flip: u32) -> RenderInstance {
    let [ax, ay, bx, by, tx, ty] = g.affine();
    let mut i = quad(tex, [tx, ty]);
    i.size = sprite.size;
    i.anchor = sprite.anchor;
    i.basis = [ax, ay, bx, by];
    i.flip_uv = flip;
    i
}

/// A fluência da cena `w`, com `present` (os sprites por texel) — e a grade dela.
fn fluencia(
    gpu: &ph2d_gpu::GpuContext,
    w: &World,
    present: &mut PresentWorld,
    r: &SpriteRenderer,
) -> (Vec<[f32; 3]>, Config, CenaDeLuz) {
    let mut luz = LuzDoQuadro::com_degrau(crate::comum::ALTO);
    assert!(luz.preparar(w, afim(), ALVO));
    luz.gravar(gpu, present, r);
    eprintln!("captura: sprites por texel {:?}", luz.sprites_por_texel());
    let s = luz.solver().expect("solver");
    (fluencia_lida(gpu, s).v, *s.config(), luz.cena().clone())
}

/// A fluência de uma `CenaDeLuz` de polígonos na MESMA configuração (a régua).
fn fluencia_de_poligonos(
    gpu: &ph2d_gpu::GpuContext,
    cena: &CenaDeLuz,
    cfg: Config,
) -> Vec<[f32; 3]> {
    // O MESMO programa do quadro: a região da janela.
    let mut s = Solver::novo(gpu, cfg).expect("solver");
    let (ini, fim) = regiao_da_janela(s.grade_da_fluencia(), afim(), ALVO);
    s.regiao_fina(ini, fim);
    s.quadro(gpu, cena);
    fluencia_lida(gpu, &s).v
}

/// RMSE relativo de `a` contra `b`, canais `canais`, nos texels cujo centro está à direita de
/// `x_min` (metros): ATRÁS do objeto, onde a sombra, o buraco ou a cor dele se lêem — e dentro da
/// região da janela (a única que o quadro calcula).
fn rmse(a: &[[f32; 3]], b: &[[f32; 3]], g: Grade, x_min: f64, canais: &[usize]) -> f64 {
    let (ini, fim) = regiao_da_janela(g, afim(), ALVO);
    let (mut e2, mut soma, mut n) = (0.0_f64, 0.0_f64, 0u32);
    for (k_texel, (x, y)) in a.iter().zip(b).enumerate() {
        let (i, j) = (k_texel as u32 % g.largura, k_texel as u32 / g.largura);
        if !(ini[0]..fim[0]).contains(&i) || !(ini[1]..fim[1]).contains(&j) {
            continue;
        }
        if g.origem_m[0] + (f64::from(i) + 0.5) * g.texel_m < x_min {
            continue;
        }
        for &k in canais {
            let d = f64::from(x[k]) - f64::from(y[k]);
            e2 += d * d;
            soma += f64::from(y[k]);
            n += 1;
        }
    }
    (e2 / f64::from(n)).sqrt() / (soma / f64::from(n))
}

fn retangulo(c: [f32; 2], meia: [f32; 2]) -> Vec<[f32; 2]> {
    vec![
        [c[0] - meia[0], c[1] - meia[1]],
        [c[0] + meia[0], c[1] - meia[1]],
        [c[0] + meia[0], c[1] + meia[1]],
        [c[0] - meia[0], c[1] + meia[1]],
    ]
}

/// Um oclusor sprite em `(0, 0)`, 0,5 × 1,5 m, com a textura `rgba` (`lado`²), rotação e inversão.
fn cena_com_oclusor(
    r: &mut SpriteRenderer,
    rgba: &[u8],
    lado: u32,
    rot: f32,
    flip: u32,
) -> (World, PresentWorld, Entity) {
    let mut w = World::new();
    luz_a_esquerda(&mut w);
    let (t, g) = pose(0.0, 0.0, rot);
    let mut sprite = Sprite::atlas(0, [0.5, 1.5], [1.0; 4]);
    sprite.anchor = [0.05, -0.1];
    let e = w.spawn((sprite, BloqueiaLuz { opacidade: 1.0 }, t, g)).id();
    let tex = r.acquire_individual(lado, lado, rgba).expect("textura");
    let mut present = PresentWorld::new();
    present
        .world_mut()
        .spawn((instancia(tex, &sprite, &g, flip), SimRef(e)));
    (w, present, e)
}

#[test]
fn um_sprite_opaco_rodado_e_espelhado_captura_o_mesmo_que_o_seu_quad() {
    let Some(gpu) = gpu() else {
        eprintln!("captura: sem placa — saltado");
        return;
    };
    let mut r = renderer(&gpu);
    let opaco = [[90u8, 90, 90, 255]; 16].concat();
    let (w, mut present, _) = cena_com_oclusor(
        &mut r,
        &opaco,
        4,
        0.4,
        RenderInstance::FLIP_X_BIT | RenderInstance::FLIP_Y_BIT,
    );
    let (por_texel, cfg, cena) = fluencia(&gpu, &w, &mut present, &r);
    let quad = fluencia_de_poligonos(&gpu, &cena, cfg);
    let e = rmse(&por_texel, &quad, cfg.grade_da_fluencia(), 1.0, &[0, 1, 2]);
    eprintln!("captura: opaco rodado × quad — RMSE relativo {e:.4}");
    assert!(
        e <= TOL_QUAD,
        "o sprite opaco não captura o seu quad: {e:.4}"
    );
}

#[test]
fn uma_fenda_transparente_deixa_a_luz_passar_e_o_quad_nao() {
    let Some(gpu) = gpu() else {
        eprintln!("captura: sem placa — saltado");
        return;
    };
    let mut r = renderer(&gpu);
    // 4×4, as duas linhas do meio transparentes de lado a lado: uma fenda horizontal.
    let mut fenda = Vec::new();
    for j in 0..4 {
        for _ in 0..4 {
            fenda.extend_from_slice(&[60, 60, 60, if (1..3).contains(&j) { 0 } else { 255 }]);
        }
    }
    let (w, mut present, _) = cena_com_oclusor(&mut r, &fenda, 4, 0.0, 0);
    let (por_texel, cfg, cena) = fluencia(&gpu, &w, &mut present, &r);
    // A régua: as duas metades em retângulos (o sprite está em (0,05, −0,1) pela âncora).
    let c = [0.05_f32, -0.1];
    let (sx, sy) = (0.5_f32, 1.5_f32);
    let mut duas_metades = cena.clone();
    duas_metades.oclusores = vec![
        retangulo([c[0], c[1] + 3.0 * sy / 8.0], [sx / 2.0, sy / 8.0]),
        retangulo([c[0], c[1] - 3.0 * sy / 8.0], [sx / 2.0, sy / 8.0]),
    ]
    .into_iter()
    .map(|vertices| Oclusor {
        vertices,
        opacidade: 1.0,
    })
    .collect();
    let referencia = fluencia_de_poligonos(&gpu, &duas_metades, cfg);
    let quad = fluencia_de_poligonos(&gpu, &cena, cfg);
    let (e, controlo) = (
        rmse(
            &por_texel,
            &referencia,
            cfg.grade_da_fluencia(),
            0.5,
            &[0, 1, 2],
        ),
        rmse(&quad, &referencia, cfg.grade_da_fluencia(), 0.5, &[0, 1, 2]),
    );
    eprintln!("captura: fenda × duas metades — RMSE relativo {e:.4} (o quad: {controlo:.4})");
    assert!(
        e <= TOL_FENDA,
        "a luz não passa pela fenda como entre as duas metades: {e:.4}"
    );
    assert!(
        controlo > 3.0 * TOL_FENDA,
        "controlo: o QUAD devia tapar a fenda (mediu {controlo:.4})"
    );
}

#[test]
fn um_emissor_de_duas_cores_emite_cada_cor_no_seu_lado() {
    let Some(gpu) = gpu() else {
        eprintln!("captura: sem placa — saltado");
        return;
    };
    let mut r = renderer(&gpu);
    let mut w = World::new();
    // Uma luz apagada só para ACENDER a cena: o emissor é o sprite.
    let (t, g) = pose(3.0, 1.5, 0.0);
    w.spawn((
        Luz {
            cor: [1.0; 3],
            intensidade: 0.0,
            tamanho_m: 0.1,
            alcance_m: f32::INFINITY,
            tipo: TipoDeLuz::Ponto,
        },
        t,
        g,
    ));
    let (t, g) = pose(-1.0, 0.0, 0.0);
    let sprite = Sprite::atlas(0, [1.0, 0.5], [1.0; 4]);
    let e = w.spawn((sprite, SpriteEmissive(2.0), t, g)).id();
    // ⭐ Duas vezes: direito e ESPELHADO — o espelho troca o lado das cores.
    for espelho in [false, true] {
        // 32×1: vermelho à esquerda, azul à direita (sRGB 255 = linear 1). ⚠️ A captura amostra
        // BILINEAR: com 2×1 a mistura das duas cores ocupava meio sprite (8 texels da grade); com 32
        // texels ela cabe em meio texel — a régua de dois retângulos com corte seco pergunta isto.
        let metade = |cor: [u8; 4]| std::iter::repeat_n(cor, 16).flatten();
        let rgba: Vec<u8> = metade([255, 0, 0, 255])
            .chain(metade([0, 0, 255, 255]))
            .collect();
        let tex = r.acquire_individual(32, 1, &rgba).expect("textura");
        let mut present = PresentWorld::new();
        let flip = if espelho {
            RenderInstance::FLIP_X_BIT
        } else {
            0
        };
        present
            .world_mut()
            .spawn((instancia(tex, &sprite, &g, flip), SimRef(e)));
        let (por_texel, cfg, cena) = fluencia(&gpu, &w, &mut present, &r);
        let mut duas = cena.clone();
        let k = 2.0;
        let (esq, dir) = if espelho {
            ([0.0, 0.0, k], [k, 0.0, 0.0])
        } else {
            ([k, 0.0, 0.0], [0.0, 0.0, k])
        };
        duas.emissores = vec![
            Emissor::Poligono {
                vertices: retangulo([-1.25, 0.0], [0.25, 0.25]),
                radiancia: esq,
                alcance_m: f32::INFINITY,
            },
            Emissor::Poligono {
                vertices: retangulo([-0.75, 0.0], [0.25, 0.25]),
                radiancia: dir,
                alcance_m: f32::INFINITY,
            },
        ];
        let referencia = fluencia_de_poligonos(&gpu, &duas, cfg);
        // O verde é zero nas duas; a régua usa o vermelho e o azul, à direita do sprite.
        let e = rmse(
            &por_texel,
            &referencia,
            cfg.grade_da_fluencia(),
            -0.4,
            &[0, 2],
        );
        eprintln!(
            "captura: duas cores (espelho = {espelho}) × dois retângulos — RMSE relativo {e:.4}"
        );
        assert!(
            e <= TOL_CORES,
            "a emissão por texel não segue a cor da textura: {e:.4}"
        );
    }
}

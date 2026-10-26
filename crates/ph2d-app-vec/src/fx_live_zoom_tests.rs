//! **O FX de uma forma a ZOOM crescente** — a régua do achado do Enio no smoke F33 (26/10): *«zoom
//! muito perto numa forma com filtro faz a forma sumir e o FPS cair»*. Plano e tabelas:
//! `docs/UI_New_and_Simple/F34_plano.md`.
//!
//! A imagem de FX TOMA O LUGAR do desenho, então o que a tela mostra da forma é o que cai dentro do
//! `rect` do job. Os gates sem placa leem o job; o de placa compara pixels do job recortado com os do
//! job inteiro.

use super::{Job, job_for};
use ph2d_ecs::{FxOp, Name, SimWorld, Transform, VecFilter, VecPathRef};
use ph2d_render::stack_read_por_lado;
use ph2d_vec_entities::entities::VecEntityMap;
use ph2d_vec_render::LiveGeometry;
use ph2d_vec_scene::{Paint, Rgba8, VecPathId, VecScene, VecViewState, VecXforms};
use ph2d_vector::Affine;

/// A janela da cena da sonda (a do monitor do dono).
const VIEW: [f64; 4] = [0.0, 0.0, 1920.0, 1080.0];
/// Uma janela que nunca recorta — o job «de antes», para a paridade.
const SEM_RECORTE: [f64; 4] = [-1.0e6, -1.0e6, 1.0e6, 1.0e6];
/// O raio da forma, em mundo.
const R: f64 = 100.0;
const ZOOMS: [f64; 7] = [1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0];
const BORDA: [f64; 2] = [R, 0.0];
const CENTRO: [f64; 2] = [0.0, 0.0];

struct Cena {
    scene: VecScene,
    sim: SimWorld,
    map: VecEntityMap,
    id: VecPathId,
    xf: VecXforms,
    live: LiveGeometry,
}

fn cena(kind: u8) -> Cena {
    cena_op(FxOp::new(kind))
}

fn cena_op(op: FxOp) -> Cena {
    let mut sim = SimWorld::default();
    let mut map = VecEntityMap::new();
    let mut scene = VecScene::new();
    let mut path = ph2d_vec_scene::ellipse([0.0, 0.0], R, R);
    path.fill = Some(Paint::solid(Rgba8::new(235, 175, 60, 255)));
    let id = scene.push_path(path);
    let e = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Shape"), VecPathRef(id)))
        .id();
    map.insert(id, e.to_bits());
    crate::fx_live::set_filter(&mut sim, &map, &[id], Some(VecFilter::from_ops(vec![op])));
    Cena {
        scene,
        sim,
        map,
        id,
        xf: VecXforms::new(),
        live: LiveGeometry::new(),
    }
}

impl Cena {
    fn job(&self, cam: Affine, vista: [f64; 4]) -> Option<Job> {
        let view = VecViewState::default();
        let (l, s, x) = (&self.live, &self.live, &self.xf);
        job_for(
            &self.scene,
            &self.sim,
            &self.map,
            x,
            l,
            s,
            &view,
            cam,
            vista,
            self.id,
        )
    }

    fn caixa(&self, cam: Affine) -> [f64; 4] {
        let b =
            ph2d_vec_render::path_screen_bounds(&self.scene, &self.xf, &self.live, self.id, cam)
                .expect("caixa");
        [b.0, b.1, b.2, b.3]
    }

    fn segs(&self, cam: Affine, job: &Job, janela: [f32; 4]) -> Vec<[f32; 4]> {
        let (l, x) = (&self.live, &self.xf);
        let off = Affine::translate((-job.ex0, -job.ey0));
        ph2d_vec_render::silhouette_segments_within(&self.scene, x, l, l, self.id, cam, off, janela)
    }
}

/// A câmera que põe o ponto de mundo `foco` no centro da janela, a `zoom`, deslocada `pan` px.
fn camera(zoom: f64, foco: [f64; 2], pan: f64) -> Affine {
    Affine::translate((VIEW[2] / 2.0 + pan, VIEW[3] / 2.0))
        * Affine::scale(zoom)
        * Affine::translate((-foco[0], -foco[1]))
}

fn inter(a: [f64; 4], b: [f64; 4]) -> [f64; 4] {
    [
        a[0].max(b[0]),
        a[1].max(b[1]),
        a[2].min(b[2]),
        a[3].min(b[3]),
    ]
}

fn area(a: [f64; 4]) -> f64 {
    (a[2] - a[0]).max(0.0) * (a[3] - a[1]).max(0.0)
}

/// O que vai à TELA (a parte `mostra` da janela), em pixels de tela.
fn rect(job: &Job) -> [f64; 4] {
    let [x0, y0, x1, y1] = job.key.mostra.map(f64::from);
    [job.ex0 + x0, job.ey0 + y0, job.ex0 + x1, job.ey0 + y1]
}

fn kinds() -> impl Iterator<Item = u8> {
    (0..FxOp::KINDS).map(|k| u8::try_from(k).expect("kind"))
}

/// **G1 — a parte VISTA da forma é toda coberta pela imagem de FX, a todo zoom e para todo tipo, e o
/// scratch nunca passa da vista alargada pelo que a pilha lê.** Antes da F34: 0 % a 64× na borda.
#[test]
fn a_filtered_shape_never_vanishes_and_never_pays_beyond_the_view() {
    for kind in kinds() {
        let c = cena(kind);
        for foco in [CENTRO, BORDA] {
            for z in ZOOMS {
                let cam = camera(z, foco, 0.0);
                let job = c.job(cam, VIEW).expect("a forma está na vista");
                let visivel = inter(c.caixa(cam), VIEW);
                let coberta = area(inter(visivel, rect(&job))) / area(visivel);
                assert!(
                    coberta >= 1.0 - 1e-9,
                    "tipo {kind} a {z}× (foco {foco:?}): a imagem cobre {:.1} % do que se vê",
                    coberta * 100.0
                );
                let [l, t, r, b] = stack_read_por_lado(&job.key.ops).map(f64::from);
                let tecto = (VIEW[2] + l + r + 2.0) * (VIEW[3] + t + b + 2.0);
                let texels = f64::from(job.key.w) * f64::from(job.key.h);
                assert!(
                    texels <= tecto,
                    "tipo {kind} a {z}×: scratch {}x{} passa da vista alargada",
                    job.key.w,
                    job.key.h
                );
            }
        }
    }
}

/// **G2 — com recorte, o pan muda o que está na célula e a chave TEM de o ver; sem recorte, o pan
/// continua fora da chave** (é o gesto onde o memo se paga).
#[test]
fn the_pan_re_cooks_a_clipped_shape_and_only_a_clipped_one() {
    let c = cena(FxOp::BEVEL);
    // No CENTRO a 64× a forma cobre a vista inteira: o pan não muda o tamanho da janela, só o que ela
    // mostra — então só o deslocamento da janela na chave pode ver o gesto.
    let perto = |pan| c.job(camera(64.0, CENTRO, pan), VIEW).expect("job").key;
    assert!(
        perto(0.0).w == perto(10.0).w,
        "a régua precisa de um pan que não mude o tamanho"
    );
    assert!(
        perto(0.0) != perto(10.0),
        "a 64× o pan reusou os pixels da janela anterior"
    );
    let longe = |pan| c.job(camera(1.0, CENTRO, pan), VIEW).expect("job").key;
    assert!(
        longe(0.0) == longe(10.0),
        "a 1× (sem recorte) o pan re-cozinhou a forma"
    );
}

/// **G3 — a origem do ruído é o canto da FORMA, com e sem recorte**: `ex0 + ruido` cai sobre a caixa
/// da forma menos a margem da pilha, então a Turbulence não anda com a janela.
#[test]
fn the_noise_is_anchored_to_the_shape_not_to_the_window() {
    let c = cena(FxOp::TURBULENCE);
    for (z, foco) in [(1.0, CENTRO), (64.0, BORDA)] {
        for pan in [0.0, 7.0, 13.0] {
            let cam = camera(z, foco, pan);
            let job = c.job(cam, VIEW).expect("job");
            let (ml, _, _, _) = ph2d_render::stack_reach(&job.key.ops);
            // A `Origem::Margem` de antes: o pixel local `ml` da caixa inteira.
            let canto = (c.caixa(cam)[0] - f64::from(ml)).floor() + f64::from(ml);
            let ancora = job.ex0 + f64::from(job.ruido[0]);
            assert!(
                (ancora - canto).abs() < 1e-3,
                "a {z}×, pan {pan}: o ruído ancora em {ancora}, a forma em {canto}"
            );
        }
    }
}

/// **G4 — o campo recebe só os segmentos que alcançam a janela, e nunca a lista vazia.** Vazia
/// mandaria o degrau para o raster; e semear a forma inteira é o que custava ~234 ms a 64×.
#[test]
fn the_field_is_seeded_only_by_the_segments_that_reach_the_window() {
    let c = cena(FxOp::BEVEL);
    let cam = camera(64.0, BORDA, 0.0);
    let job = c.job(cam, VIEW).expect("job");
    let tudo = [
        f32::NEG_INFINITY,
        f32::NEG_INFINITY,
        f32::INFINITY,
        f32::INFINITY,
    ];
    let janela = [0.0, 0.0, job.key.w as f32, job.key.h as f32];
    let (todos, perto) = (
        c.segs(cam, &job, tudo).len(),
        c.segs(cam, &job, janela).len(),
    );
    assert!(
        perto > 0 && perto * 4 < todos,
        "{perto} de {todos} segmentos semeiam a janela"
    );
    // A 64× no CENTRO a janela fica toda dentro da forma, longe da borda: nenhum segmento a alcança.
    let cam = camera(64.0, CENTRO, 0.0);
    let job = c.job(cam, VIEW).expect("job");
    let janela = [0.0, 0.0, job.key.w as f32, job.key.h as f32];
    assert_eq!(
        c.segs(cam, &job, janela).len(),
        1,
        "a janela sem borda perdeu o plano de geometria"
    );
}

/// **G6 — um GESTO no zoom máximo quase não troca a textura de saída.** Cada troca re-regista a
/// imagem no Vello, e trocá-la a cada quadro tirava-a do atlas em ~¼ dos quadros a partir de
/// ~2 000 px: o piscar do smoke F34 (sonda `fx_zoom_sonda_do_piscar`). Um zoom de 32× a 64× em 60
/// quadros e um pan de 300 px na borda, contando as trocas pela lei do `ensure_output`.
#[test]
fn a_gesture_at_max_zoom_almost_never_swaps_the_output_texture() {
    use crate::fx_live::{capacidade, serve};
    let c = cena(FxOp::BEVEL);
    let trocas = |quadros: Vec<Affine>| {
        let mut cap: Option<(u32, u32)> = None;
        let mut n = 0;
        for cam in quadros {
            let job = c.job(cam, VIEW).expect("job");
            let [x0, y0, x1, y1] = job.key.mostra;
            let (w, h) = (x1 - x0, y1 - y0);
            if !cap.is_some_and(|(cw, ch)| serve(cw, w) && serve(ch, h)) {
                cap = Some((capacidade(w), capacidade(h)));
                n += 1;
            }
        }
        n
    };
    let zoom = trocas(
        (0..60)
            .map(|q| camera(32.0 * 2f64.powf(f64::from(q) / 59.0), CENTRO, 0.0))
            .collect(),
    );
    let pan = trocas(
        (0..60)
            .map(|q| camera(64.0, BORDA, f64::from(q) * 5.0))
            .collect(),
    );
    assert!(
        zoom <= 3 && pan <= 3,
        "trocas de textura: zoom {zoom}, pan {pan} em 60 quadros"
    );
}

/// O filtro da cena 41 com o Bevel a `depth` (o dono: «para ver o defeito, Depth 1.4»).
fn filtro_com_depth(depth: f32) -> VecFilter {
    let mut ops = crate::vector_mode_filtro_efeito::filtro().ops().to_vec();
    ops[0].radius = depth;
    VecFilter::from_ops(ops)
}

/// A câmera do zoom máximo do app na janela da sonda (`Camera2d::ZOOM_MIN_HEIGHT_WORLD`).
fn zoom_maximo(foco: [f64; 2]) -> Affine {
    let k = VIEW[3] / f64::from(ph2d_render::Camera2d::ZOOM_MIN_HEIGHT_WORLD);
    Affine::translate((VIEW[2] / 2.0, VIEW[3] / 2.0))
        * Affine::scale(k)
        * Affine::translate((-foco[0], -foco[1]))
}

/// **G10 — com o contorno exacto, a margem do Bevel não cresce com o Depth.** O Bevel mede o pé por
/// texel e não lê vizinho; a banda só escolhe os segmentos. Depth 1,4 no zoom máximo pedia ~3 000 px de
/// cada lado, e o job passava do lado da placa (o piscar e o sumiço do smoke F34).
#[test]
fn a_deep_bevel_with_geometry_does_not_widen_the_window() {
    let mut c = cena(FxOp::BEVEL);
    crate::fx_live::set_filter(&mut c.sim, &c.map, &[c.id], Some(filtro_com_depth(1.4)));
    // Uma nuvem de raio 100 m: no zoom máximo a vista cai inteira dentro dela.
    let cam = zoom_maximo([R - 1.0, 0.0]);
    let job = c.job(cam, VIEW).expect("job");
    let sombra = stack_read_por_lado(&job.key.ops[1..]).map(f64::from);
    let teto_w = VIEW[2] + sombra[0] + sombra[2] + 8.0;
    let teto_h = VIEW[3] + sombra[1] + sombra[3] + 8.0;
    assert!(
        job.geom.is_some(),
        "o job recortado devia trazer a geometria"
    );
    assert!(
        f64::from(job.key.w) <= teto_w && f64::from(job.key.h) <= teto_h,
        "janela {}x{} (o teto, só com a margem da sombra, é {teto_w}x{teto_h})",
        job.key.w,
        job.key.h
    );
}

/// **G11 — a janela cabe no lado da placa SEM perder a vista.** Uma sombra de borrão enorme no zoom
/// máximo pede mais que 8192: o excesso sai das margens, e a vista fica inteira dentro da janela.
/// Antes o `.min` cortava pelo fim, e a vista caía fora (auditoria F34 #3).
#[test]
fn a_window_beyond_the_device_side_keeps_the_view_inside() {
    let mut c = cena(FxOp::DROP_SHADOW);
    let sombra = FxOp {
        radius: 2.0,
        ..FxOp::new(FxOp::DROP_SHADOW)
    };
    crate::fx_live::set_filter(
        &mut c.sim,
        &c.map,
        &[c.id],
        Some(VecFilter::from_ops(vec![sombra])),
    );
    let cam = zoom_maximo([R - 1.0, 0.0]);
    let job = c.job(cam, VIEW).expect("job");
    let lado = f64::from(super::MAX_FX_SIDE);
    let janela = [
        job.ex0,
        job.ey0,
        job.ex0 + f64::from(job.key.w),
        job.ey0 + f64::from(job.key.h),
    ];
    assert!(
        f64::from(job.key.w) <= lado && f64::from(job.key.h) <= lado,
        "janela {}x{}",
        job.key.w,
        job.key.h
    );
    let fora =
        VIEW[0] < janela[0] || VIEW[1] < janela[1] || VIEW[2] > janela[2] || VIEW[3] > janela[3];
    let visivel = inter(c.caixa(cam), VIEW);
    let coberta = area(inter(visivel, rect(&job))) / area(visivel);
    assert!(
        !fora && coberta >= 1.0 - 1e-9,
        "a vista {VIEW:?} saiu da janela {janela:?} (cobre {coberta})"
    );
}

/// **A sonda** — imprime a tabela do plano §1 (cobertura, texels, segmentos).
/// `cargo test -p ph2d-app-vec --lib fx_zoom_regua -- --ignored --nocapture`.
#[test]
#[ignore = "sonda: imprime a tabela de cobertura e texels por zoom (F34)"]
fn fx_zoom_regua_cobertura() {
    for (nome, foco) in [("centro", CENTRO), ("borda", BORDA)] {
        for kind in [FxOp::BEVEL, FxOp::BLUR, FxOp::DROP_SHADOW] {
            let c = cena(kind);
            for z in ZOOMS {
                let cam = camera(z, foco, 0.0);
                let job = c.job(cam, VIEW).expect("job");
                let visivel = inter(c.caixa(cam), VIEW);
                let coberta = area(inter(visivel, rect(&job))) / area(visivel);
                let janela = [0.0, 0.0, job.key.w as f32, job.key.h as f32];
                eprintln!(
                    "[fx-zoom] foco {nome} kind {kind:>2} zoom {z:>4}× cobertura {:>6.1} % \
                     scratch {}x{} ({:.1} Mtexel) segs {}",
                    coberta * 100.0,
                    job.key.w,
                    job.key.h,
                    f64::from(job.key.w) * f64::from(job.key.h) / 1e6,
                    c.segs(cam, &job, janela).len()
                );
            }
        }
    }
}

#[path = "fx_live_zoom_gpu_tests.rs"]
mod gpu;

/// Sonda: o alcance de cada degrau do filtro da cena 41, por zoom real da câmera (`k` px/m).
#[test]
#[ignore = "sonda F34: alcance por degrau"]
fn fx_zoom_sonda_do_alcance() {
    let mut f = crate::vector_mode_filtro_efeito::filtro();
    if let Some(d) = std::env::var("PH2D_F34_DEPTH")
        .ok()
        .and_then(|v| v.parse::<f32>().ok())
    {
        let mut ops = f.ops().to_vec();
        ops[0].radius = d;
        f = ph2d_ecs::VecFilter::from_ops(ops);
    }
    for k in [100.0, 500.0, 1000.0, 2140.0, 5000.0] {
        let ops = crate::fx_live::resolve_ops(&f, Affine::scale(k));
        for o in &ops {
            eprintln!(
                "[alcance] k {k:>6} tipo {:>2} sigma {:>8.1} off {:?} · lê {:?} · espalha {:?}",
                o.kind,
                o.sigma_px,
                o.offset_px,
                ph2d_render::stack_read_por_lado(std::slice::from_ref(o)),
                ph2d_render::stack_reach(std::slice::from_ref(o))
            );
        }
    }
}

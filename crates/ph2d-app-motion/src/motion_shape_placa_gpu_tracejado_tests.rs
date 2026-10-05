//! ⭐⭐ **O TRACEJADO ESTICADO PELA ROTA DO PRODUTO** (doc 121 §9.9) — formas de fábrica com traço
//! tracejado (o período AJUSTADO ao contorno pela `dash_fit`, as pontas e juntas do `StrokeSpec`),
//! esticadas, pelo `encode` do produto (a lei da casa: a geometria transformada, caneta e padrão
//! `× √|det|`) e pela placa (o eixo cortado pelo comprimento de arco no ecrã). Até ao §9.9 este quadro
//! ia inteiro ao Vello.

use super::*;
use ph2d_vec_scene::{LineCap, LineJoin, Rgba8, StrokeSpec};

fn tracejada(
    mut forma: VecPath,
    largura: f64,
    dash: (f64, f64),
    cap: LineCap,
    join: LineJoin,
) -> VecPath {
    let mut s = StrokeSpec::new(Rgba8::new(20, 30, 40, 240), largura);
    s.dash = Some(dash);
    s.cap = cap;
    s.join = join;
    forma.stroke = Some(s);
    forma
}

fn store() -> (VecPathStore, Vec<u32>) {
    let formas = [
        // Ponta QUADRADA: com a rente, um traço que acaba a menos de meia largura depois de uma quina
        // saía do traçador do Vello com uma MORDIDA no lado de dentro. Curada no doc 121 §9.18 (C) — a
        // família dela é a `a_rota_vello_traceja_o_pedaco_rente_como_a_placa`.
        tracejada(
            ph2d_vec_scene::star([0.0, 0.0], 0.5, 0.5, 5, 0.4),
            0.05,
            (2.0, 1.0),
            LineCap::Square,
            LineJoin::Miter,
        ),
        tracejada(
            ph2d_vec_scene::ellipse([0.0, 0.0], 0.5, 0.5),
            0.06,
            (1.5, 1.5),
            LineCap::Round,
            LineJoin::Round,
        ),
        tracejada(
            ph2d_vec_scene::regular_polygon_rounded([0.0, 0.0], 0.5, 0.5, 6, 0.08),
            0.05,
            (6.0, 2.0),
            LineCap::Square,
            LineJoin::Bevel,
        ),
        tracejada(
            ph2d_vec_scene::star_rounded([0.0, 0.0], 0.5, 0.5, 6, 0.5, 0.08, 0.04),
            0.04,
            (3.0, 2.0),
            LineCap::Round,
            LineJoin::Miter,
        ),
    ];
    let mut s = VecPathStore::default();
    let hs = formas.into_iter().map(|f| s.push(f)).collect();
    (s, hs)
}

/// ⭐⭐ **A placa traceja o esticado como a cena Vello** — as barras do traço esticado do produto.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn a_rota_da_placa_traceja_o_esticado_como_a_casa() {
    let Some(gpu) = gpu() else {
        eprintln!("sem adapter — o gate não correu");
        return;
    };
    let (store, hs) = store();
    let mut insts = copias(&hs, 160);
    for (i, c) in insts.iter_mut().enumerate() {
        #[expect(clippy::cast_precision_loss, reason = "uma fixtura pequena")]
        let k = (i as f32 * 0.618_034).fract();
        c.size[1] *= 0.35 + 2.45 * k;
    }
    assert!(
        insts.iter().filter(|c| !super::super::conforme(c)).count() > 120,
        "controlo: a fixtura tem de ser feita de copias NAO conformes"
    );
    let v = pelo_vello(&gpu, &insts, &store);
    let p = pela_placa(&gpu, &insts, &store);
    let (alfa, cor, nv, np, fora) = compara(&v, &p);
    // doc 121 §9.18 (C): a rota de antes (o traçador do Vello), só para a tabela.
    let (alfa_antes, cor_antes, _, _, fora_antes) =
        compara(&pelo_tracador_do_vello(&gpu, &insts, &store), &p);
    fotografa("tracejado", &v, &p);
    eprintln!(
        "  tracejado esticado: alfa max {alfa} · cor max {cor} · {fora} px fora · vello {nv} px · placa {np} px \
         · ANTES alfa {alfa_antes} · cor {cor_antes} · {fora_antes} px fora"
    );
    assert!(nv > 20_000, "controlo: a cena pinta pouco ({nv} px)");
    let diff = nv.abs_diff(np);
    assert!(diff * 100 <= nv, "área pintada diverge: {nv} contra {np}");
    assert!(alfa <= 100, "alfa {alfa} acima da barra das curvas (100)");
    assert!(cor <= 100, "cor {cor} acima da barra das curvas (100)");
    assert!(
        fora * 20 <= nv,
        "{fora} pixels desviam > 16 — acima de 5 % dos {nv} pintados"
    );
}

/// A família do pedaço RENTE (doc 121 §9.18 C): estrelas de arestas rectas — o preenchimento não põe
/// diferenças de aplanamento na régua — com traços curtos (`2` larguras, vão `1`) que acabam a qualquer
/// distância depois de uma quina; ponta REDONDA e ponta rente. E uma estrela ARREDONDADA (o nível de
/// aplanamento muda o eixo), cujas cópias têm o preenchimento transparente: só o traço entra na régua.
fn store_rente() -> (VecPathStore, Vec<u32>) {
    let formas = [
        tracejada(
            ph2d_vec_scene::star([0.0, 0.0], 0.5, 0.5, 5, 0.4),
            0.05,
            (2.0, 1.0),
            LineCap::Round,
            LineJoin::Miter,
        ),
        tracejada(
            ph2d_vec_scene::star([0.0, 0.0], 0.5, 0.5, 7, 0.55),
            0.04,
            (2.0, 1.0),
            LineCap::Butt,
            LineJoin::Miter,
        ),
        tracejada(
            ph2d_vec_scene::star_rounded([0.0, 0.0], 0.5, 0.5, 6, 0.5, 0.08, 0.04),
            0.04,
            (2.5, 1.5),
            LineCap::Round,
            LineJoin::Round,
        ),
    ];
    let mut s = VecPathStore::default();
    let hs = formas.into_iter().map(|f| s.push(f)).collect();
    (s, hs)
}

/// A rota de ANTES deste gate: o lote sem o traço pela lei (o traçador do Vello).
fn pelo_tracador_do_vello(
    gpu: &GpuContext,
    insts: &[VectorInstance],
    store: &VecPathStore,
) -> Vec<u8> {
    let mut cena = ph2d_vector::VectorScene::new();
    let janela = ph2d_vector::Rect::new(0.0, 0.0, f64::from(LADO), f64::from(LADO));
    ph2d_vec_render::draw_shared_instances(
        insts.iter().map(|i| {
            (
                i.geometry_id,
                crate::motion_shape_gen::instance_pose(i, camara()),
                i.tint,
            )
        }),
        |h| store.get(h),
        Some(janela),
        &mut cena,
    );
    let mut vp =
        ph2d_render::VelloPass::new(gpu, wgpu::TextureFormat::Bgra8UnormSrgb, (LADO, LADO))
            .expect("o VelloPass do produto nasce");
    vp.render_and_readback(gpu, cena.inner(), (LADO, LADO))
        .expect("o Vello desenha")
}

/// ⭐⭐ doc 121 §9.18 (C) — **a rota Vello do Motion traceja o pedaço RENTE depois de uma quina como a
/// placa**: o traçador do Vello mordia-o (a junta interior pelo pivô), e a rota passou a PREENCHER os
/// polígonos da placa — as marcas numa cópia conforme, o eixo no ecrã numa esticada. ⚠️ O controlo: a
/// rota de antes, na MESMA cena, morde (senão a família não exercitava a mordida).
#[test]
#[ignore = "precisa de adapter de GPU"]
fn a_rota_vello_traceja_o_pedaco_rente_como_a_placa() {
    let Some(gpu) = gpu() else {
        eprintln!("sem adapter — o gate não correu");
        return;
    };
    let (store, hs) = store_rente();
    let mut insts = copias(&hs, 160);
    for (i, c) in insts.iter_mut().enumerate() {
        if i % 3 != 0 {
            #[expect(clippy::cast_precision_loss, reason = "uma fixtura pequena")]
            let k = (i as f32 * 0.618_034).fract();
            c.size[1] *= 0.35 + 2.45 * k;
        }
    }
    for c in insts.iter_mut().filter(|c| c.geometry_id == hs[2]) {
        c.tint[3] = 0.0;
    }
    let conformes = insts.iter().filter(|c| super::super::conforme(c)).count();
    assert!(
        (40..=60).contains(&conformes),
        "controlo: a família leva conformes ({conformes}) e esticadas"
    );
    let p = pela_placa(&gpu, &insts, &store);
    let v = pelo_vello(&gpu, &insts, &store);
    let antes = pelo_tracador_do_vello(&gpu, &insts, &store);
    let (alfa, cor, nv, np, fora) = compara(&v, &p);
    let (alfa_antes, _, _, _, fora_antes) = compara(&antes, &p);
    fotografa("tracejado_rente", &v, &p);
    eprintln!(
        "  pedaço rente: alfa max {alfa} · cor max {cor} · {fora} px fora · vello {nv} px · placa {np} px \
         · ANTES (o traçador do Vello) alfa max {alfa_antes} · {fora_antes} px fora"
    );
    assert!(nv > 20_000, "controlo: a cena pinta pouco ({nv} px)");
    assert!(
        alfa_antes > 100 && fora_antes > 0,
        "controlo: a rota de antes não mordeu esta cena (alfa {alfa_antes})"
    );
    assert!(alfa <= 1, "a rota Vello difere da placa: alfa {alfa}");
}

/// ⭐ doc 121 §9.18 (C) — **o custo do `encode` na CPU**, a rota pela lei contra a de antes (o traçador do
/// Vello), INTERCALADAS no mesmo processo (`docs/DevOps/MEDIR_VELOCIDADE.md`): o arranjo da `=127` densa
/// tracejada (`35 × 35`, com a janela) e as `16 384` cópias dela (`128 × 128`, sem recorte). O resumo é o
/// MÍNIMO das rodadas, a mediana ao lado.
///
/// ```text
/// cargo test -p ph2d-app-motion --lib --profile smoke -- --ignored --nocapture custo_do_encode_tracejado
/// ```
#[test]
#[ignore = "sonda de relógio"]
fn custo_do_encode_tracejado() {
    let (store, densas) = super::estrelas_da_sonda("", "2", true);
    let base = densas[0];
    #[expect(clippy::cast_precision_loss, reason = "uma grelha pequena")]
    let grande: Vec<VectorInstance> = (0..128 * 128)
        .map(|k| VectorInstance {
            world_pos: [-7.65 + 0.45 * (k % 128) as f32, -7.65 + 0.45 * (k / 128) as f32],
            ..base
        })
        .collect();
    let janela = ph2d_vector::Rect::new(0.0, 0.0, f64::from(LADO), f64::from(LADO));
    let novo = |insts: &[VectorInstance], j: Option<ph2d_vector::Rect>| {
        let mut c = ph2d_vector::VectorScene::new();
        crate::motion_shape_gen::encode(
            insts,
            &store,
            &mut |_, _| None,
            camara(),
            j,
            ph2d_render::ImageFilterMode::Smooth,
            &mut c,
        );
        c
    };
    let antes = |insts: &[VectorInstance], j: Option<ph2d_vector::Rect>| {
        let mut c = ph2d_vector::VectorScene::new();
        ph2d_vec_render::draw_shared_instances(
            insts.iter().map(|i| {
                (
                    i.geometry_id,
                    crate::motion_shape_gen::instance_pose(i, camara()),
                    i.tint,
                )
            }),
            |h| store.get(h),
            j,
            &mut c,
        );
        c
    };
    let carga = || std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    eprintln!("CUSTO_ENCODE load {}", carga().trim());
    for (nome, insts, j, rodadas) in [
        ("=127 densa 35x35", &densas, Some(janela), 15),
        ("=127 16 384 copias", &grande, None, 5),
    ] {
        let _ = (novo(insts, j), antes(insts, j));
        let mut t = [Vec::new(), Vec::new()];
        for r in 0..rodadas {
            for k in [r % 2, 1 - r % 2] {
                let t0 = std::time::Instant::now();
                let c = if k == 0 { novo(insts, j) } else { antes(insts, j) };
                t[k].push(t0.elapsed().as_secs_f64() * 1e3);
                drop(c);
            }
        }
        let resumo = |v: &mut Vec<f64>| {
            v.sort_by(f64::total_cmp);
            (v[0], v[v.len() / 2])
        };
        let ((nmin, nmed), (amin, amed)) = (resumo(&mut t[0].clone()), resumo(&mut t[1].clone()));
        eprintln!(
            "CUSTO_ENCODE {nome}: pela lei min {nmin:.2} med {nmed:.2} ms · antes min {amin:.2} med {amed:.2} ms · {:+.1} %",
            (nmin / amin - 1.0) * 100.0
        );
        // A parede do Vello (o desenho na placa gráfica, com a leitura), intercalada, quando há placa.
        if let Some(gpu) = gpu() {
            let cenas = [novo(insts, j), antes(insts, j)];
            let mut vp = ph2d_render::VelloPass::new(
                &gpu,
                wgpu::TextureFormat::Bgra8UnormSrgb,
                (LADO, LADO),
            )
            .expect("vello");
            let mut p = [Vec::new(), Vec::new()];
            for r in 0..rodadas {
                for k in [r % 2, 1 - r % 2] {
                    let t0 = std::time::Instant::now();
                    let _ = vp.render_and_readback(&gpu, cenas[k].inner(), (LADO, LADO));
                    p[k].push(t0.elapsed().as_secs_f64() * 1e3);
                }
            }
            let ((nmin, _), (amin, _)) = (resumo(&mut p[0].clone()), resumo(&mut p[1].clone()));
            eprintln!(
                "CUSTO_ENCODE {nome}: parede do Vello pela lei min {nmin:.2} ms · antes min {amin:.2} ms · {:+.1} %",
                (nmin / amin - 1.0) * 100.0
            );
        }
    }
    eprintln!("CUSTO_ENCODE fim · load {}", carga().trim());
}

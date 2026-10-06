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
/// aplanamento muda o eixo) e uma engrenagem com FURO (dois contornos de comprimentos diferentes: o ajuste fecha o
/// mais longo, e o 1.º traço do outro EMENDA no último), cujas cópias têm o preenchimento transparente: só o traço
/// entra na régua.
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
        tracejada(
            ph2d_vec_scene::gear([-0.5, -0.5], [0.5, 0.5], 9.0, 0.25, 0.3),
            0.04,
            (2.0, 1.0),
            LineCap::Round,
            LineJoin::Miter,
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
    for c in insts
        .iter_mut()
        .filter(|c| c.geometry_id == hs[2] || c.geometry_id == hs[3])
    {
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
            world_pos: [
                -7.65 + 0.45 * (k % 128) as f32,
                -7.65 + 0.45 * (k / 128) as f32,
            ],
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
    // As variantes, intercaladas em ordem rodada: `0` a de antes (o traçador do Vello), `1` pela lei (o
    // produto). §9.19 (2): um candidato novo entra aqui como mais uma.
    let variante = |k: usize, insts: &[VectorInstance], j: Option<ph2d_vector::Rect>| {
        if k == 0 {
            antes(insts, j)
        } else {
            novo(insts, j)
        }
    };
    const NOMES: [&str; 2] = ["antes", "pela lei"];
    const N: usize = NOMES.len();
    for (nome, insts, j, rodadas) in [
        ("=127 densa 35x35", &densas, Some(janela), 15),
        ("=127 16 384 copias", &grande, None, 5),
    ] {
        let resumo = |v: &[f64]| {
            let mut o = v.to_vec();
            o.sort_by(f64::total_cmp);
            (o[0], o[o.len() / 2])
        };
        let mut t: [Vec<f64>; N] = Default::default();
        for k in 0..N {
            let _ = variante(k, insts, j);
        }
        for r in 0..rodadas {
            for d in 0..N {
                let k = (r + d) % N;
                let t0 = std::time::Instant::now();
                let c = variante(k, insts, j);
                t[k].push(t0.elapsed().as_secs_f64() * 1e3);
                drop(c);
            }
        }
        let base = resumo(&t[0]).0;
        for k in 0..N {
            let (min, med) = resumo(&t[k]);
            // O que o Vello recebe: caminhos e segmentos da codificação (o traço de antes expande-se na placa).
            let c = variante(k, insts, j);
            let e = c.inner().encoding();
            eprintln!(
                "CUSTO_ENCODE {nome}: encode {:<8} min {min:.2} med {med:.2} ms · {:+.1} % de antes · {} caminhos · {} segmentos",
                NOMES[k],
                (min / base - 1.0) * 100.0,
                e.n_paths,
                e.n_path_segments
            );
        }
        // A parede do Vello (o desenho na placa gráfica, com a leitura), intercalada, quando há placa.
        if let Some(gpu) = gpu() {
            let cenas: [_; N] = std::array::from_fn(|k| variante(k, insts, j));
            let mut vp = ph2d_render::VelloPass::new(
                &gpu,
                wgpu::TextureFormat::Bgra8UnormSrgb,
                (LADO, LADO),
            )
            .expect("vello");
            let mut p: [Vec<f64>; N] = Default::default();
            for r in 0..rodadas {
                for d in 0..N {
                    let k = (r + d) % N;
                    let t0 = std::time::Instant::now();
                    let _ = vp.render_and_readback(&gpu, cenas[k].inner(), (LADO, LADO));
                    p[k].push(t0.elapsed().as_secs_f64() * 1e3);
                }
            }
            let base = resumo(&p[0]).0;
            for k in 0..N {
                let (min, med) = resumo(&p[k]);
                eprintln!(
                    "CUSTO_ENCODE {nome}: parede do Vello {:<8} min {min:.2} med {med:.2} ms · {:+.1} % de antes",
                    NOMES[k],
                    (min / base - 1.0) * 100.0
                );
            }
        }
    }
    eprintln!("CUSTO_ENCODE fim · load {}", carga().trim());
}

/// Uma forma do CARTÃO do `source.shape` com o traço tracejado e a ponta e a junta escolhidas — o caminho do
/// produto (`ShapeParams::read` → `build_shape_path`).
fn do_cartao(kind: ph2d_node_motion_shape::ShapeKind, cap: f32, join: f32) -> VecPath {
    use ph2d_node_motion_shape::param;
    #[expect(clippy::cast_precision_loss, reason = "um indice de especie")]
    let kind = kind as i32 as f32;
    let get = |n: &str| match n {
        param::KIND => kind,
        param::SIZE => 0.5,
        param::STROKE_WIDTH => 0.05,
        param::STROKE_A => 1.0,
        param::DASH => 2.0,
        param::DASH_GAP => 1.0,
        param::STROKE_CAP => cap,
        param::STROKE_JOIN => join,
        _ => param::SPECS
            .iter()
            .find(|s| s.name == n)
            .map_or(0.0, |s| s.default),
    };
    crate::motion_shape_gen::build_shape_path(&ph2d_node_motion_shape::ShapeParams::read(get))
}

/// ⭐ doc 121 §9.19 (3) — **a PONTA e a JUNTA do cartão desenham-se como a placa**: as `3 × 3` combinações
/// pelo caminho do produto, esticadas e conformes, a rota Vello contra a placa (alfa `≤ 1`). ⚠️ O controlo:
/// a ponta e a junta MUDAM a imagem (senão a família só provava a omissão nove vezes).
#[test]
#[ignore = "precisa de adapter de GPU"]
fn a_ponta_e_a_junta_do_cartao_desenham_como_a_placa() {
    use ph2d_node_motion_shape::ShapeKind;
    let Some(gpu) = gpu() else {
        eprintln!("sem adapter — o gate não correu");
        return;
    };
    let mut pintados = Vec::new();
    for cap in 0..3u8 {
        for join in 0..3u8 {
            let mut store = VecPathStore::default();
            let hs = vec![
                store.push(do_cartao(ShapeKind::Star, f32::from(cap), f32::from(join))),
                store.push(do_cartao(ShapeKind::Gear, f32::from(cap), f32::from(join))),
            ];
            let mut insts = copias(&hs, 60);
            for (i, c) in insts.iter_mut().enumerate() {
                if i % 3 != 0 {
                    #[expect(clippy::cast_precision_loss, reason = "uma fixtura pequena")]
                    let k = (i as f32 * 0.618_034).fract();
                    c.size[1] *= 0.35 + 2.45 * k;
                }
                c.tint[3] = 0.0;
            }
            let p = pela_placa(&gpu, &insts, &store);
            let v = pelo_vello(&gpu, &insts, &store);
            let (alfa, cor, nv, np, fora) = compara(&v, &p);
            eprintln!(
                "  ponta {cap} junta {join}: alfa max {alfa} · cor max {cor} · {fora} px fora · vello {nv} px · placa {np} px"
            );
            assert!(nv > 5_000, "controlo: a cena pinta pouco ({nv} px)");
            assert!(
                alfa <= 1,
                "ponta {cap} junta {join}: a rota Vello difere da placa: alfa {alfa}"
            );
            pintados.push(np);
        }
    }
    // A ponta muda a área (a redonda e a quadrada passam do fim do traço); a junta muda-a nas quinas.
    assert!(
        pintados[3] > pintados[0] && pintados[6] > pintados[3],
        "a ponta nao chegou ao desenho: {pintados:?}"
    );
    assert!(
        pintados[0] != pintados[2],
        "a junta nao chegou ao desenho: {pintados:?}"
    );
}

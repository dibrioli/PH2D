//! ⭐⭐⭐ **A pilha da peça COMPOSTA NA PLACA** (`docs/3D/30` §13, W1b) — os gates
//! que pedem a placa: o composto da placa é o da CPU byte a byte, as faixas
//! sujas sobem, e o preço de um passo do arrasto de opacidade.

use std::time::Instant;

use ph2d_tool_painter::{AdjustmentKind, AdjustmentParams, BlendMode, HsbParams, LayerId};

use crate::composto_na_placa::CompostoNaPlaca;
use crate::pilha_da_peca::PilhaDaPeca;

/// Píxeis de hash por amostra (cor e alfa variados).
fn px(n: usize, semente: u32) -> Vec<[u8; 4]> {
    (0..n)
        .map(|i| {
            let h = (i as u32).wrapping_mul(2_654_435_761) ^ semente.wrapping_mul(40_503);
            let h = h ^ (h >> 15);
            [(h >> 8) as u8, (h >> 16) as u8, (h >> 24) as u8, h as u8]
        })
        .collect()
}

fn plano_da_licao(k: u8) -> ph2d_mesh_colors::Tinta {
    let mesh = crate::scenes::tinta_fina::peca();
    let faces = || mesh.faces().iter().map(ph2d_mesh::Face::verts);
    match mesh.colors() {
        Some(c) => ph2d_mesh_colors::Tinta::semeada(c, faces(), k),
        None => ph2d_mesh_colors::Tinta::nova(mesh.vert_count(), faces(), k),
    }
}

fn pinta(p: &mut PilhaDaPeca, id: LayerId, semente: u32) {
    let n = p.amostras();
    p.plano_mut(id)
        .expect("plano")
        .escreve(&px(n, semente), None);
}

/// ⭐ **A pilha RICA** — a base translúcida, modos separáveis e não
/// separáveis, opacidade, uma máscara pintada, um recorte e os ajustes de
/// ponto (HSB, Invert com opacidade, Curves pela tabela).
fn pilha_rica(k: u8) -> (PilhaDaPeca, LayerId) {
    let mut p = PilhaDaPeca::de_tinta(&plano_da_licao(k));
    let base = p.base().expect("base");
    p.define_opacidade(base, 0.85);
    let mult = p.nova_camada("mult").expect("camada");
    p.define_modo(mult, BlendMode::Multiply);
    p.define_opacidade(mult, 0.7);
    pinta(&mut p, mult, 1);
    let mascara = p.nova_mascara(mult).expect("máscara");
    pinta(&mut p, mascara, 2);
    let over = p.nova_camada("over").expect("camada");
    p.define_modo(over, BlendMode::Overlay);
    p.define_recorte(over, true);
    pinta(&mut p, over, 3);
    for (s, modo, op) in [
        (4, BlendMode::Screen, 0.55),
        (5, BlendMode::SoftLight, 1.0),
        (6, BlendMode::Color, 0.8),
    ] {
        let id = p.nova_camada("c").expect("camada");
        p.define_modo(id, modo);
        p.define_opacidade(id, op);
        pinta(&mut p, id, s);
    }
    let hsb = p
        .novo_ajuste(AdjustmentKind::HueSaturationBrightness)
        .expect("ajuste");
    p.define_parametros(
        hsb,
        AdjustmentParams::HueSaturationBrightness(HsbParams {
            h: 30.0,
            s: 0.2,
            b: 0.1,
        }),
    )
    .expect("parâmetros");
    let inv = p.novo_ajuste(AdjustmentKind::Invert).expect("ajuste");
    p.define_opacidade(inv, 0.4);
    p.novo_ajuste(AdjustmentKind::Curves).expect("ajuste");
    assert!(p.sincronizada());
    (p, mult)
}

/// Quantos bytes diferem e a maior diferença.
fn compara(cpu: &[u8], placa: &[u8]) -> (usize, u8) {
    assert_eq!(cpu.len(), placa.len());
    cpu.iter()
        .zip(placa)
        .filter(|(a, b)| a != b)
        .fold((0, 0), |(n, m), (a, b)| (n + 1, m.max(a.abs_diff(*b))))
}

/// O contrato placa↔CPU (`docs/3D/30` §13, decisão do dono 03/10): nenhum byte
/// a mais de UM degrau, e poucos a um degrau — a divisão do WGSL (`2,5 ULP`) num
/// valor na fronteira. Medido: `23` de `188 424` bytes (`0,012 %`) na pilha
/// rica a `8x`; o tecto da fracção é `10×` isso — uma lei DIFERENTE que só
/// erre por um degrau espalha-se por muito mais.
const FRACCAO_A_UM_DEGRAU: f64 = 0.001_2;

/// ⭐⭐⭐⭐ **O composto da placa é o da CPU, a um degrau de sRGB8** — na pilha
/// rica, nos degraus `8x..32x`; depois de mudar só o METADADO (nada sobe); e
/// depois de sujar uma FAIXA de uma camada (só ela sobe).
#[test]
#[ignore = "precisa de placa"]
fn a_placa_compoe_a_pilha_rica_como_a_cpu() {
    let gpu = gpu_or_skip!();
    for k in 3u8..=5 {
        let (mut p, mult) = pilha_rica(k);
        let n = p.amostras();
        let mut placa = CompostoNaPlaca::novo(&gpu);
        let mut confere = |p: &mut PilhaDaPeca, o_que: &str| {
            placa.compoe(&gpu, p).expect("a pilha rica é representável");
            let lida = placa.le(&gpu).expect("composto");
            let (difs, pior) = compara(&p.compor(), &lida[..n * 4]);
            assert!(
                pior <= 1 && (difs as f64) <= FRACCAO_A_UM_DEGRAU * (n * 4) as f64,
                "{}x, {o_que}: {difs} bytes de {} diferem da CPU (pior {pior})",
                1u32 << k,
                n * 4
            );
        };
        confere(&mut p, "a pilha inteira");
        let mut nova = p.pilha().clone();
        nova.set_opacity(mult, 0.3);
        p.troca_metadado(nova).expect("metadado");
        confere(&mut p, "a opacidade da camada multiply");
        let idx: Vec<u32> = (2_000..2_300).chain(n as u32 - 50..n as u32).collect();
        let novos = vec![[200, 10, 30, 255]; idx.len()];
        p.troca_janela(mult, &idx, &novos).expect("janela");
        confere(&mut p, "uma faixa suja da camada multiply");
    }
}

/// 🔎 **SONDA — de que INGREDIENTE vem cada byte que a placa difere da CPU**:
/// a base (opaca ou translúcida) + UM ingrediente da pilha rica de cada vez.
#[test]
#[ignore = "sonda: imprime a tabela"]
fn diag_que_ingrediente_difere_da_cpu() {
    let gpu = gpu_or_skip!();
    type Ingrediente = fn(&mut PilhaDaPeca);
    let ingredientes: [(&str, Ingrediente); 14] = [
        ("nada", |_| {}),
        ("normal 1.0", |p| {
            let _ = nova(p, BlendMode::Normal, 1.0);
        }),
        ("normal 0.55", |p| {
            let _ = nova(p, BlendMode::Normal, 0.55);
        }),
        ("multiply 0.7", |p| {
            let _ = nova(p, BlendMode::Multiply, 0.7);
        }),
        ("multiply + máscara", |p| {
            let id = nova(p, BlendMode::Multiply, 1.0);
            let m = p.nova_mascara(id).expect("máscara");
            pinta(p, m, 2);
        }),
        ("overlay recortado", |p| {
            let _ = nova(p, BlendMode::Multiply, 1.0);
            let o = nova(p, BlendMode::Overlay, 1.0);
            p.define_recorte(o, true);
        }),
        ("screen 0.55", |p| {
            let _ = nova(p, BlendMode::Screen, 0.55);
        }),
        ("softlight", |p| {
            let _ = nova(p, BlendMode::SoftLight, 1.0);
        }),
        ("color 0.8", |p| {
            let _ = nova(p, BlendMode::Color, 0.8);
        }),
        ("hsb", |p| {
            let h = p
                .novo_ajuste(AdjustmentKind::HueSaturationBrightness)
                .expect("ajuste");
            p.define_parametros(
                h,
                AdjustmentParams::HueSaturationBrightness(HsbParams {
                    h: 30.0,
                    s: 0.2,
                    b: 0.1,
                }),
            )
            .expect("parâmetros");
        }),
        ("hsb neutro", |p| {
            p.novo_ajuste(AdjustmentKind::HueSaturationBrightness)
                .expect("ajuste");
        }),
        ("invert 0.4", |p| {
            let i = p.novo_ajuste(AdjustmentKind::Invert).expect("ajuste");
            p.define_opacidade(i, 0.4);
        }),
        ("invert 1.0", |p| {
            let _ = p.novo_ajuste(AdjustmentKind::Invert).expect("ajuste");
        }),
        ("curves", |p| {
            let _ = p.novo_ajuste(AdjustmentKind::Curves).expect("ajuste");
        }),
    ];
    for base_op in [1.0f32, 0.85] {
        for (nome, faz) in ingredientes {
            let mut p = PilhaDaPeca::de_tinta(&plano_da_licao(3));
            let base = p.base().expect("base");
            p.define_opacidade(base, base_op);
            faz(&mut p);
            let n = p.amostras();
            let mut placa = CompostoNaPlaca::novo(&gpu);
            placa.compoe(&gpu, &mut p).expect("representável");
            let lida = placa.le(&gpu).expect("composto");
            let (difs, pior) = compara(&p.compor(), &lida[..n * 4]);
            eprintln!("base {base_op} · {nome}: {difs} bytes diferem (pior {pior})");
        }
    }
}

fn nova(p: &mut PilhaDaPeca, modo: BlendMode, op: f32) -> LayerId {
    let id = p.nova_camada("c").expect("camada");
    p.define_modo(id, modo);
    p.define_opacidade(id, op);
    pinta(p, id, 7);
    id
}

/// 🔎 **SONDA — um passo do arrasto de opacidade composto NA PLACA** (o
/// critério de desistência da W1b, `docs/3D/30` §7: `< 4 ms` a `64x`). A
/// pilha das sondas da W3 (3 camadas + HSB); cada passo = o metadado pela porta
/// + a composição na placa até ela acabar (`poll` à espera).
#[test]
#[ignore = "sonda: imprime a tabela"]
fn diag_o_preco_de_compor_na_placa() {
    let gpu = gpu_or_skip!();
    for k in 3u8..=6 {
        let mut p = PilhaDaPeca::de_tinta(&plano_da_licao(k));
        let n = p.amostras();
        let mut cima = None;
        for (s, modo) in [(1, BlendMode::Multiply), (2, BlendMode::Overlay)] {
            let id = p.nova_camada("c").expect("camada");
            p.define_modo(id, modo);
            pinta(&mut p, id, s);
            cima = Some(id);
        }
        let cima = cima.expect("cima");
        p.novo_ajuste(AdjustmentKind::HueSaturationBrightness)
            .expect("ajuste");
        let mut placa = CompostoNaPlaca::novo(&gpu);
        let t = Instant::now();
        placa.compoe(&gpu, &mut p).expect("compõe");
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        let primeira = t.elapsed().as_secs_f64() * 1e3;
        let mut passos = Vec::new();
        for q in 0..20u32 {
            let mut nova = p.pilha().clone();
            nova.set_opacity(cima, 1.0 - q as f32 * 0.04);
            let t = Instant::now();
            p.troca_metadado(nova).expect("metadado");
            placa.compoe(&gpu, &mut p).expect("compõe");
            let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
            passos.push(t.elapsed().as_secs_f64() * 1e3);
        }
        passos.sort_by(f64::total_cmp);
        eprintln!(
            "degrau {k} ({}x) · {n} amostras · a 1.ª composição (sobe as camadas) {primeira:.2} ms · \
             um passo do arrasto NA PLACA: mediana {:.3} ms · pior {:.3} ms",
            1u32 << k,
            passos[10],
            passos[19]
        );
    }
}

/// O slot do device onde mora a peça activa.
fn slot_da_activa(s: &crate::Sculpt3dScene) -> usize {
    let id = s.objects[s.active].id;
    s.slots
        .iter()
        .position(|&o| o == id)
        .expect("a peça activa está à vista")
}

/// ⛔ O plano LIDO DA PLACA é a peça da CPU (a referência, `para_ler`) a um
/// degrau de sRGB8, poucos a um degrau; e a cor por vértice é o prefixo dela,
/// ao bit. Devolve a referência.
fn a_placa_tem_a_peca(
    s: &crate::Sculpt3dScene,
    gpu: &ph2d_gpu::GpuContext,
    quando: &str,
) -> Vec<[f32; 3]> {
    let o = &s.objects[s.active];
    let plano_cpu = o.tinta.as_ref().expect("plano");
    let referencia = crate::tinta_da_peca::pilha::para_ler(o, plano_cpu)
        .amostras()
        .to_vec();
    let placa = s
        .renderer
        .le_tinta_at(&gpu.device, &gpu.queue, slot_da_activa(s))
        .expect("o plano está armado na placa");
    assert_eq!(placa.len(), referencia.len(), "{quando}");
    let degrau = 1.0 / 255.0 + 1e-5;
    let mut a_um_degrau = 0usize;
    for (i, (g, c)) in placa.iter().zip(&referencia).enumerate() {
        let d = (0..3).map(|j| (g[j] - c[j]).abs()).fold(0.0f32, f32::max);
        assert!(
            d <= degrau,
            "{quando}, amostra {i}: placa {g:?} contra a CPU {c:?}"
        );
        if d > 1e-5 {
            a_um_degrau += 1;
        }
    }
    assert!(
        (a_um_degrau as f64) <= FRACCAO_A_UM_DEGRAU * (referencia.len() * 3) as f64,
        "{quando}: {a_um_degrau} amostras a um degrau de {}",
        referencia.len()
    );
    let v = o.stack.mesh().vert_count();
    assert_eq!(
        o.stack.mesh().colors().expect("cor por vértice"),
        &referencia[..v],
        "{quando}: a cor por vértice é o prefixo da peça, ao bit"
    );
    referencia
}

/// A cena `52` armada, com uma camada Multiply sobre a base TRANSLÚCIDA (o
/// fundo vê-se) — e o que o `faz` acrescentar à pilha.
fn cena_com_camada(
    gpu: &ph2d_gpu::GpuContext,
    faz: impl FnOnce(&mut PilhaDaPeca),
) -> crate::Sculpt3dScene {
    let mut s = super::cena_52(&gpu.device);
    s.sync_mesh(gpu);
    let p = s.objects[s.active]
        .pilha
        .as_mut()
        .expect("a peça com plano tem pilha");
    let cima = p.nova_camada("cima").expect("camada");
    p.define_modo(cima, BlendMode::Multiply);
    pinta(p, cima, 9);
    let base = p.base().expect("base");
    p.define_opacidade(base, 0.8);
    faz(p);
    s
}

/// ⭐⭐⭐⭐ **GATE — O painel muda a pilha e a PLACA tem a peça** (`docs/3D/30`
/// §13): pela cena real, a recomposição do painel (`recompoe`) e um quadro
/// (`sync_mesh`) — o plano LIDO DA PLACA é a peça da CPU (`a_placa_tem_a_peca`).
/// CONTROLO: o plano da CPU ficou para trás — quem compôs a peça foi a placa. E
/// uma subida INTEIRA do plano (a CPU atrasada) volta a compor na placa, senão
/// mostrava a peça de antes.
#[test]
#[ignore = "precisa de placa"]
fn o_painel_muda_a_pilha_e_a_placa_tem_a_peca() {
    let gpu = gpu_or_skip!();
    let mut s = cena_com_camada(&gpu, |_| {});
    let a = s.active;
    let antes = s.objects[a]
        .tinta
        .as_ref()
        .expect("plano")
        .amostras()
        .to_vec();
    crate::tinta_da_peca::pilha::recompoe(&mut s.objects[a]);
    s.sync_mesh(&gpu);
    let o = &s.objects[a];
    assert!(
        o.pilha.as_ref().expect("pilha").atrasada(),
        "a recomposição do painel é da placa"
    );
    let v = o.stack.mesh().vert_count();
    assert_eq!(
        o.tinta.as_ref().expect("plano").amostras()[v..],
        antes[v..],
        "CONTROLO: fora dos vértices o plano da CPU ficou como estava"
    );
    let referencia = a_placa_tem_a_peca(&s, &gpu, "depois do painel");
    assert_ne!(referencia, antes, "a camada mudou a peça");

    // A peça sobe INTEIRA (o device esqueceu-a) com o plano da CPU atrasado.
    s.objects[a].uploaded = false;
    s.sync_mesh(&gpu);
    a_placa_tem_a_peca(&s, &gpu, "depois de uma subida inteira");
}

/// ⭐⭐⭐ **GATE — Uma pilha que a placa NÃO exprime compõe-se na CPU** e a peça
/// é a mesma: um ajuste sem código de placa (`ColorBalance`). CONTROLO: a
/// porta da placa recusa-a mesmo; e o plano da CPU fica em dia.
#[test]
#[ignore = "precisa de placa"]
fn uma_pilha_que_a_placa_recusa_compoe_na_cpu() {
    let gpu = gpu_or_skip!();
    let mut s = cena_com_camada(&gpu, |p| {
        p.novo_ajuste(AdjustmentKind::ColorBalance)
            .expect("ajuste de ponto");
    });
    let a = s.active;
    assert!(
        ph2d_painter_layer_ops::flatten_for_gpu(
            s.objects[a].pilha.as_ref().expect("pilha").pilha()
        )
        .is_none(),
        "CONTROLO: a placa não exprime o ColorBalance"
    );
    crate::tinta_da_peca::pilha::recompoe(&mut s.objects[a]);
    s.sync_mesh(&gpu);
    assert!(
        !s.objects[a].pilha.as_ref().expect("pilha").atrasada(),
        "a CPU compôs: o plano está em dia"
    );
    a_placa_tem_a_peca(&s, &gpu, "com o ajuste que a placa recusa");
}

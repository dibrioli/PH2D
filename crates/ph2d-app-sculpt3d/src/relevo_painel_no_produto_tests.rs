//! ⭐⭐⭐⭐ **O RELEVO POR CAMADA PELO PAINEL, PELO CAMINHO DO PRODUTO** (`docs/3D/30` §15 — a W4) —
//! irmão (`#[path]`) do [`super`], com o arnês do painel (`super::painel`). Pede a placa.
//!
//! Dois traços de IMPASTO do Painter (um na base, outro numa camada nova por cima), o arrasto REAL da
//! profundidade da de cima e o clique REAL no chip `Add`/`Level`, o `Ctrl+Z` pela tecla — e o relevo
//! lido da PEÇA (a CPU) e da PLACA (o que o artista vê).

use super::painel::{Painel, painter_vermelho, traco};
use super::{cena_52, tecla};
use crate::painter_na_malha::quadro;
use crate::pilha_da_peca::PilhaDaPeca;
use ph2d_tool_painter::ids::{PAINTER_LAYERS_ADD, PainterLayerWidget, painter_layer_widget_id};
use ph2d_tool_painter::{LayerId, PaintMedia, PainterTool, ReliefComposite};

fn pilha(s: &crate::Sculpt3dScene) -> &PilhaDaPeca {
    s.objects[s.active]
        .pilha
        .as_ref()
        .expect("a peça tem pilha")
}

/// O relevo da PEÇA (na CPU ele está sempre em dia — só a cor se atrasa).
fn relevo(s: &crate::Sculpt3dScene) -> Vec<[f32; 2]> {
    s.objects[s.active]
        .tinta
        .as_ref()
        .and_then(|t| t.relevo())
        .expect("a peça tem relevo")
        .to_vec()
}

fn da_camada(s: &crate::Sculpt3dScene, id: LayerId) -> Option<Vec<[f32; 2]>> {
    pilha(s).plano(id)?.relevo().map(<[_]>::to_vec)
}

fn bits(r: &[[f32; 2]]) -> Vec<[u32; 2]> {
    r.iter().map(|x| x.map(f32::to_bits)).collect()
}

/// `a` e `b` ao bit — e, se não, quantas amostras diferem e a primeira.
fn iguais(a: &[[f32; 2]], b: &[[f32; 2]], quando: &str) {
    let difs: Vec<usize> = (0..a.len().max(b.len()))
        .filter(|&i| a.get(i).map(|x| x.map(f32::to_bits)) != b.get(i).map(|x| x.map(f32::to_bits)))
        .collect();
    assert!(
        difs.is_empty(),
        "{quando}: {} de {} amostras diferem; a 1.ª, {:?}: {:?} contra {:?}",
        difs.len(),
        a.len(),
        difs.first(),
        difs.first().and_then(|&i| a.get(i)),
        difs.first().and_then(|&i| b.get(i)),
    );
}

/// O relevo e as inclinações LIDOS DA PLACA, depois do `sync_mesh` do quadro.
fn da_placa(
    s: &mut crate::Sculpt3dScene,
    gpu: &ph2d_gpu::GpuContext,
) -> (Vec<[f32; 2]>, Vec<[f32; 3]>) {
    s.sync_mesh(gpu);
    let id = s.objects[s.active].id;
    let slot = s.slots.iter().position(|&o| o == id).expect("à vista");
    s.renderer
        .le_relevo_at(&gpu.device, &gpu.queue, slot)
        .expect("a placa tem o relevo")
}

/// As inclinações que a CPU calcula do zero para o relevo `r` sobre a peça.
fn inclinacoes_de(s: &crate::Sculpt3dScene, r: &[[f32; 2]]) -> Vec<[f32; 3]> {
    let o = &s.objects[s.active];
    let mut t = o.tinta.clone().expect("plano");
    t.com_relevo(Some(r.to_vec()));
    let mesh = o.stack.mesh();
    ph2d_mesh_colors::Inclinacoes::nova(&t, |f| mesh.faces()[f].verts(), mesh.positions())
        .por_amostra()
        .to_vec()
}

fn pior(a: &[[f32; 3]], b: &[[f32; 3]]) -> f32 {
    a.iter()
        .zip(b)
        .flat_map(|(x, y)| (0..3).map(move |j| (x[j] - y[j]).abs()))
        .fold(0.0, f32::max)
}

fn painter_impasto() -> PainterTool {
    let mut p = painter_vermelho();
    p.set_paint_media(PaintMedia::Impasto);
    p.set_brush_size_px(28.0);
    p
}

/// ⭐⭐⭐⭐ **GATE (seam, W4) — baixar a PROFUNDIDADE da camada de cima pelo painel achata a pincelada
/// DELA sem mexer na de baixo, a placa mostra-o, e o `Ctrl+Z` devolve o relevo AO BIT; o chip `Level`
/// faz a de cima enterrar o que está por baixo.**
///
/// Valores exactos: o relevo da peça é a dobra da pilha ao bit (CPU), a placa tem esses bits, e as
/// inclinações da placa são as que a CPU calcula do zero para eles. CONTROLOS: o arrasto MUDA o
/// relevo (e as inclinações) onde a de cima tem relevo, e só aí.
#[test]
#[ignore = "precisa de adaptador"]
fn a_profundidade_pelo_painel_achata_a_camada_de_cima_e_o_ctrl_z_a_devolve_ao_bit() {
    let gpu = gpu_or_skip!();
    let mut s = cena_52(&gpu.device);
    s.sync_mesh(&gpu);
    let mut p = painter_impasto();
    let mut painel = Painel::novo();
    quadro(Some(&mut s), Some(&mut p));
    let base = pilha(&s).base().expect("base");

    // 1. Um traço de impasto na base; 2. uma camada nova e um traço nela, por cima.
    traco(&mut s, &mut p, 400.0);
    s.sync_mesh(&gpu);
    quadro(Some(&mut s), Some(&mut p));
    let r_base = da_camada(&s, base).expect("o impasto deu relevo à base");
    let (alt, _) = da_placa(&mut s, &gpu);
    iguais(&alt, &relevo(&s), "a placa tem o relevo do 1.º traço");
    painel.clica(&mut p, PAINTER_LAYERS_ADD);
    quadro(Some(&mut s), Some(&mut p));
    let cima = pilha(&s).pilha().active().expect("a nova é a activa");
    assert_ne!(cima, base);
    traco(&mut s, &mut p, 430.0);
    s.sync_mesh(&gpu);
    quadro(Some(&mut s), Some(&mut p));
    assert_eq!(
        da_camada(&s, base).as_deref().map(bits),
        Some(bits(&r_base)),
        "o traço de cima não mexeu no relevo da base"
    );
    let r_cima = da_camada(&s, cima).expect("o impasto deu relevo à de cima");
    let ambos = relevo(&s);
    assert_eq!(
        Some(bits(&ambos)),
        pilha(&s).relevo_composto().as_deref().map(bits),
        "a peça é a dobra da pilha"
    );
    assert!(
        p.panel_layers()
            .and_then(|l| l.get(cima))
            .is_some_and(|l| l.has_relief),
        "o painel sabe que a de cima tem relevo (a linha da profundidade)"
    );
    let (alt, _) = da_placa(&mut s, &gpu);
    iguais(&alt, &ambos, "a placa tem o relevo dos dois traços");

    // 3. Baixar a profundidade da de cima — o arrasto REAL, em dois quadros.
    let fundo = painter_layer_widget_id(cima.0, PainterLayerWidget::ImpastoDepth);
    painel.arrasta(&mut p, fundo, 0.99, 0.75);
    quadro(Some(&mut s), Some(&mut p));
    painel.arrasta(&mut p, fundo, 0.75, 0.55);
    quadro(Some(&mut s), Some(&mut p));
    let d = pilha(&s).pilha().get(cima).map(|c| c.impasto_depth);
    assert!(d.is_some_and(|d| d < 0.5), "a profundidade desceu: {d:?}");
    let baixo = relevo(&s);
    assert_eq!(
        Some(bits(&baixo)),
        pilha(&s).relevo_composto().as_deref().map(bits),
        "a peça segue a dobra com a profundidade nova"
    );
    let mudaram: Vec<usize> = (0..baixo.len())
        .filter(|&i| baixo[i][0].to_bits() != ambos[i][0].to_bits())
        .collect();
    assert!(!mudaram.is_empty(), "CONTROLO: o arrasto mudou o relevo");
    assert!(
        mudaram.iter().all(|&i| r_cima[i][0] != 0.0),
        "só onde a de cima tem relevo — a de baixo fica"
    );
    let (alt, inc) = da_placa(&mut s, &gpu);
    iguais(&alt, &baixo, "a placa tem o relevo novo, ao bit");
    let cpu = inclinacoes_de(&s, &baixo);
    let antes = inclinacoes_de(&s, &ambos);
    let erro = pior(&inc, &cpu);
    let mexeu = pior(&antes, &cpu);
    assert!(
        erro <= mexeu * 1e-4,
        "as inclinações da placa são as do relevo novo ({erro} contra o que o arrasto mexeu, {mexeu})"
    );
    assert!(mexeu > 0.0, "CONTROLO: as inclinações mudaram");

    // 4. Ctrl+Z — o arrasto inteiro é UM passo; o relevo volta AO BIT, na peça e na placa.
    assert!(tecla(&mut s, false));
    quadro(Some(&mut s), Some(&mut p));
    assert_eq!(
        bits(&relevo(&s)),
        bits(&ambos),
        "o Ctrl+Z devolveu o relevo ao bit"
    );
    assert_eq!(
        pilha(&s).pilha().get(cima).map(|c| c.impasto_depth),
        Some(1.0)
    );
    let (alt, _) = da_placa(&mut s, &gpu);
    iguais(&alt, &ambos, "e a placa também");

    // 5. O chip Level: onde a de cima é tinta sólida, a superfície é a dela.
    let chip = painter_layer_widget_id(cima.0, PainterLayerWidget::ImpastoLevel);
    painel.clica(&mut p, chip);
    quadro(Some(&mut s), Some(&mut p));
    assert_eq!(
        pilha(&s).pilha().get(cima).map(|c| c.impasto_composite),
        Some(ReliefComposite::Level)
    );
    let lv = relevo(&s);
    let solidas: Vec<usize> = (0..lv.len())
        .filter(|&i| r_cima[i][1] >= 1.0 && r_base[i][0] != 0.0)
        .collect();
    assert!(!solidas.is_empty(), "a fixtura sobrepõe os dois traços");
    assert!(
        solidas
            .iter()
            .all(|&i| lv[i][0].to_bits() == r_cima[i][0].to_bits()),
        "Level: a pincelada de cima enterra a de baixo"
    );
    assert_ne!(bits(&lv), bits(&ambos), "CONTROLO: o Level mudou a peça");
}

/// ⭐ **SONDA — o preço de um passo do arrasto da PROFUNDIDADE**, de ponta a ponta, no PIOR caso: a
/// camada de cima com relevo em TODAS as amostras (a dobra, o relevo inteiro e as inclinações de
/// todas mudam em cada passo). Por degrau da peça da lição: a porta + a redobra na CPU, e o
/// `sync_mesh` (o relevo sobe sozinho e as inclinações refazem-se; a cor compõe-se na placa).
#[test]
#[ignore = "sonda: precisa de adaptador e imprime a tabela"]
fn diag_o_preco_de_arrastar_a_profundidade() {
    use ph2d_editor_core::tool::Tool as _;
    use ph2d_tool_painter::PieceLayerOp;
    use std::time::Instant;
    let gpu = gpu_or_skip!();
    for k in 3u8..=6 {
        let mut s = cena_52(&gpu.device);
        s.tinta_nivel = Some(k);
        s.sync_mesh(&gpu);
        let mut p = painter_impasto();
        quadro(Some(&mut s), Some(&mut p));
        traco(&mut s, &mut p, 400.0);
        quadro(Some(&mut s), Some(&mut p));
        p.handle_panel_event(ph2d_editor_core::tool::PanelEvent::Click(
            PAINTER_LAYERS_ADD,
        ));
        quadro(Some(&mut s), Some(&mut p));
        let cima = pilha(&s).pilha().active().expect("cima");
        let n = pilha(&s).amostras();
        let todas: Vec<u32> = (0..n as u32).collect();
        let relevo_todo: Vec<[f32; 2]> = (0..n).map(|i| [(i % 13) as f32 * 1e-3, 1.0]).collect();
        let o = &mut s.objects[s.active];
        o.pilha
            .as_mut()
            .and_then(|pl| pl.troca_relevo(cima, &todas, &relevo_todo))
            .expect("relevo em todas");
        crate::tinta_da_peca::pilha::recompoe(o);
        s.sync_mesh(&gpu);
        quadro(Some(&mut s), Some(&mut p));
        let gesto = painter_layer_widget_id(cima.0, PainterLayerWidget::ImpastoDepth);
        let (mut cpu, mut placa, mut dobra) = (Vec::new(), Vec::new(), Vec::new());
        for q in 0..20u32 {
            let mut nova = p.panel_layers().expect("pilha").clone();
            nova.set_impasto_depth(cima, 0.98 - q as f32 * 0.04);
            let t = Instant::now();
            let recusa = s.aplica_pedidos_da_pilha(vec![PieceLayerOp::Metadata {
                stack: nova,
                gesture: Some(gesto),
            }]);
            cpu.push(t.elapsed().as_secs_f64() * 1e3);
            assert!(recusa.is_none());
            assert!(s.objects[s.active].relevo_sujo, "o passo redobrou o relevo");
            let t = Instant::now();
            std::hint::black_box(pilha(&s).relevo_composto());
            dobra.push(t.elapsed().as_secs_f64() * 1e3);
            quadro(Some(&mut s), Some(&mut p));
            let t = Instant::now();
            s.sync_mesh(&gpu);
            gpu.device.poll(wgpu::PollType::wait_indefinitely()).ok();
            placa.push(t.elapsed().as_secs_f64() * 1e3);
        }
        for v in [&mut cpu, &mut placa, &mut dobra] {
            v.sort_by(f64::total_cmp);
        }
        eprintln!(
            "degrau {k} ({}x) · {n} amostras · um passo da profundidade: porta+redobra (CPU) mediana \
             {:.2} · pior {:.2} ms (só a dobra {:.2}) | sync_mesh (relevo+inclinações+cor) mediana \
             {:.2} · pior {:.2} ms",
            1u32 << k,
            cpu[10],
            cpu[19],
            dobra[10],
            placa[10],
            placa[19]
        );
    }
}

/// ⭐⭐ **GATE — com a camada de cima a cobrir a peça inteira, um passo da profundidade refaz TODAS
/// as inclinações na placa** (o ramo do `Inclinacoes::refaz` que recalcula do zero e sobe o buffer
/// inteiro): a placa tem os bits do relevo novo e as inclinações que a CPU calcula do zero para eles.
/// CONTROLO: as inclinações mudaram em mais de metade das amostras.
#[test]
#[ignore = "precisa de adaptador"]
fn a_profundidade_de_uma_camada_que_cobre_a_peca_refaz_as_inclinacoes_todas() {
    use ph2d_tool_painter::PieceLayerOp;
    let gpu = gpu_or_skip!();
    let mut s = cena_52(&gpu.device);
    s.sync_mesh(&gpu);
    let mut p = painter_impasto();
    let mut painel = Painel::novo();
    quadro(Some(&mut s), Some(&mut p));
    painel.clica(&mut p, PAINTER_LAYERS_ADD);
    quadro(Some(&mut s), Some(&mut p));
    let cima = pilha(&s).pilha().active().expect("cima");
    let n = pilha(&s).amostras();
    let todas: Vec<u32> = (0..n as u32).collect();
    let r: Vec<[f32; 2]> = (0..n).map(|i| [(i % 13) as f32 * 1e-3, 1.0]).collect();
    let o = &mut s.objects[s.active];
    o.pilha
        .as_mut()
        .and_then(|pl| pl.troca_relevo(cima, &todas, &r))
        .expect("relevo em todas");
    crate::tinta_da_peca::pilha::recompoe(o);
    let (_, antes) = da_placa(&mut s, &gpu);
    let mut nova = pilha(&s).pilha().clone();
    nova.set_impasto_depth(cima, -0.5);
    assert!(
        s.aplica_pedidos_da_pilha(vec![PieceLayerOp::Metadata {
            stack: nova,
            gesture: None,
        }])
        .is_none()
    );
    let novo = relevo(&s);
    let (alt, inc) = da_placa(&mut s, &gpu);
    iguais(&alt, &novo, "a placa tem o relevo novo");
    let cpu = inclinacoes_de(&s, &novo);
    assert!(
        pior(&inc, &cpu) <= pior(&antes, &cpu) * 1e-4,
        "as inclinações da placa são as da CPU"
    );
    let mudaram = antes.iter().zip(&cpu).filter(|(a, b)| a != b).count();
    assert!(
        mudaram * 2 > n,
        "CONTROLO: mudaram {mudaram} de {n} — o ramo de recalcular tudo"
    );
}

/// ⭐⭐⭐ **GATE (report do dono, 04/10: *«ao usar impasto e pintar numa Layer 3 a cor do pincel não
/// apareceu, mas apenas o relevo»*) — o impasto numa camada NOVA pinta a COR dela e o relevo**, como
/// na base. A tela do Painter é semeada com a camada activa (transparente numa camada nova) e a
/// diferença entrava como «sem cor» (`docs/3D/30` §16). CONTROLO: a mesma pincelada na base.
#[test]
#[ignore = "precisa de adaptador"]
fn o_impasto_numa_camada_nova_pinta_a_cor_e_o_relevo() {
    let gpu = gpu_or_skip!();
    let mut s = cena_52(&gpu.device);
    s.sync_mesh(&gpu);
    let mut p = painter_impasto();
    let mut painel = Painel::novo();
    quadro(Some(&mut s), Some(&mut p));
    let conta = |s: &crate::Sculpt3dScene, id: LayerId, antes: &[u8]| {
        let pl = pilha(s).plano(id).expect("plano");
        let n = pilha(s).amostras();
        let cor = pl
            .rgba8(n)
            .chunks(4)
            .zip(antes.chunks(4))
            .filter(|(a, b)| a != b)
            .count();
        let rel = pl
            .relevo()
            .map_or(0, |r| r.iter().filter(|x| x[0] != 0.0).count());
        (cor, rel)
    };
    let base = pilha(&s).base().expect("base");
    let antes = pilha(&s)
        .plano(base)
        .expect("p")
        .rgba8(pilha(&s).amostras())
        .to_vec();
    traco(&mut s, &mut p, 400.0);
    s.sync_mesh(&gpu);
    quadro(Some(&mut s), Some(&mut p));
    let (cor_base, rel_base) = conta(&s, base, &antes);
    assert!(
        cor_base > 100 && rel_base > 100,
        "CONTROLO: na base ({cor_base}, {rel_base})"
    );
    painel.clica(&mut p, PAINTER_LAYERS_ADD);
    quadro(Some(&mut s), Some(&mut p));
    let cima = pilha(&s).pilha().active().expect("cima");
    let n = pilha(&s).amostras();
    let antes = pilha(&s).plano(cima).expect("p").rgba8(n).to_vec();
    traco(&mut s, &mut p, 430.0);
    s.sync_mesh(&gpu);
    quadro(Some(&mut s), Some(&mut p));
    let (cor, rel) = conta(&s, cima, &antes);
    assert!(rel > 100, "o relevo entrou ({rel})");
    assert!(
        cor * 2 > cor_base,
        "a COR entrou na camada nova: {cor} amostras (a base: {cor_base})"
    );
    let px = pilha(&s).plano(cima).expect("p").rgba8(n).to_vec();
    let vermelhas = px
        .chunks(4)
        .filter(|q| q[3] > 200 && q[0] > 150 && q[1] < 80 && q[2] < 80)
        .count();
    assert!(
        vermelhas > 50,
        "a tinta é a do pincel (vermelha, opaca): {vermelhas}"
    );
}

/// SONDA do 2.º report do dono (04/10): *«a tinta ficou com um offset em relação ao relevo»*. O centro
/// (no mundo) das amostras que ganharam COR e das que ganharam RELEVO numa pincelada de impasto — na
/// base e numa camada nova — e a largura da pincelada para escala.
#[test]
#[ignore = "sonda: precisa de adaptador"]
fn diag_onde_cai_a_cor_e_onde_cai_o_relevo() {
    let gpu = gpu_or_skip!();
    let mut s = cena_52(&gpu.device);
    s.sync_mesh(&gpu);
    let mut p = painter_impasto();
    let mut painel = Painel::novo();
    quadro(Some(&mut s), Some(&mut p));
    let mede = |s: &crate::Sculpt3dScene, id: LayerId, antes: &[u8], nome: &str| {
        let o = &s.objects[s.active];
        let xs = crate::vizinhanca_da_peca::posicoes(o.tinta.as_ref().expect("t"), o.stack.mesh());
        let pl = pilha(s).plano(id).expect("plano");
        let n = pilha(s).amostras();
        let centro = |sel: &dyn Fn(usize) -> bool| {
            let (mut c, mut k) = ([0.0f64; 3], 0usize);
            for i in (0..n).filter(|&i| sel(i)) {
                for e in 0..3 {
                    c[e] += f64::from(xs[i][e]);
                }
                k += 1;
            }
            (c.map(|v| v / k.max(1) as f64), k)
        };
        let px = pl.rgba8(n);
        let (cc, kc) = centro(&|i| px[i * 4..i * 4 + 4] != antes[i * 4..i * 4 + 4]);
        let r = pl.relevo().map(<[_]>::to_vec).unwrap_or_default();
        let (cr, kr) = centro(&|i| r.get(i).is_some_and(|x| x[0] != 0.0));
        let d =
            ((cc[0] - cr[0]).powi(2) + (cc[1] - cr[1]).powi(2) + (cc[2] - cr[2]).powi(2)).sqrt();
        eprintln!(
            "{nome}: cor {kc} amostras em {cc:.4?} · relevo {kr} em {cr:.4?} · distância {d:.4}"
        );
    };
    let base = pilha(&s).base().expect("base");
    let n = pilha(&s).amostras();
    let antes = pilha(&s).plano(base).expect("p").rgba8(n).to_vec();
    traco(&mut s, &mut p, 400.0);
    s.sync_mesh(&gpu);
    quadro(Some(&mut s), Some(&mut p));
    mede(&s, base, &antes, "BASE");
    painel.clica(&mut p, PAINTER_LAYERS_ADD);
    quadro(Some(&mut s), Some(&mut p));
    let cima = pilha(&s).pilha().active().expect("cima");
    let antes = pilha(&s).plano(cima).expect("p").rgba8(n).to_vec();
    traco(&mut s, &mut p, 430.0);
    s.sync_mesh(&gpu);
    quadro(Some(&mut s), Some(&mut p));
    mede(&s, cima, &antes, "CAMADA NOVA");
}

/// SONDA (foto) do 2.º report do dono: uma pincelada de impasto numa camada nova a CRUZAR uma
/// pincelada de impasto da base — a orla de relevo fora da tinta de cima não pode acender sobre a de
/// baixo. Grava `orla.png` em `$PH2D_SONDA_DIR`.
#[test]
#[ignore = "sonda: precisa de adaptador e de PH2D_SONDA_DIR"]
fn diag_a_orla_da_camada_de_cima() {
    let Some(dir) = std::env::var_os("PH2D_SONDA_DIR") else {
        return;
    };
    let dir = std::path::PathBuf::from(dir);
    let gpu = gpu_or_skip!();
    let mut s = cena_52(&gpu.device);
    s.sync_mesh(&gpu);
    let mut p = painter_impasto();
    p.set_brush_color_srgb8([40, 90, 200]);
    let mut painel = Painel::novo();
    quadro(Some(&mut s), Some(&mut p));
    let horizontal: Vec<(f32, f32)> = (0..=30).map(|k| (330.0 + 8.0 * k as f32, 350.0)).collect();
    super::painter::traco_por(&mut s, &mut p, &horizontal);
    s.sync_mesh(&gpu);
    quadro(Some(&mut s), Some(&mut p));
    painel.clica(&mut p, PAINTER_LAYERS_ADD);
    quadro(Some(&mut s), Some(&mut p));
    p.set_brush_color_srgb8([230, 30, 30]);
    let vertical: Vec<(f32, f32)> = (0..=30).map(|k| (450.0, 250.0 + 7.0 * k as f32)).collect();
    super::painter::traco_por(&mut s, &mut p, &vertical);
    s.sync_mesh(&gpu);
    quadro(Some(&mut s), Some(&mut p));
    super::painter::fotografa(&gpu, &mut s, dir.join("orla.png").as_os_str());
    let n = pilha(&s).amostras();
    let base = pilha(&s).base().expect("base");
    let cima = pilha(&s).pilha().active().expect("cima");
    let (pb, pc) = (
        pilha(&s).plano(base).expect("b").rgba8(n).to_vec(),
        pilha(&s).plano(cima).expect("c").rgba8(n).to_vec(),
    );
    let azul = |i: usize| pb[i * 4 + 2] > 150 && pb[i * 4] < 120;
    let (mut dentro, mut fora) = (Vec::new(), Vec::new());
    for i in (0..n).filter(|&i| pc[i * 4 + 3] > 0) {
        if azul(i) {
            dentro.push(pc[i * 4 + 3])
        } else {
            fora.push(pc[i * 4 + 3])
        }
    }
    let media = |v: &[u8]| v.iter().map(|&x| f64::from(x)).sum::<f64>() / v.len().max(1) as f64;
    let solidas = |v: &[u8]| v.iter().filter(|&&x| x == 255).count();
    eprintln!(
        "ALFA da vermelha: sobre a azul {} amostras, média {:.1}, {} a 255 · fora {} amostras, média {:.1}, {} a 255",
        dentro.len(),
        media(&dentro),
        solidas(&dentro),
        fora.len(),
        media(&fora),
        solidas(&fora)
    );
}

/// SONDA (foto) do 2.º report, no caso EXACTO do dono: a base com riscas em relevo e tinta cheia em
/// toda a peça (a cena `=55`) e uma pincelada de impasto vermelha numa camada nova. Grava
/// `orla_riscas.png` em `$PH2D_SONDA_DIR`.
#[test]
#[ignore = "sonda: precisa de adaptador e de PH2D_SONDA_DIR"]
fn diag_a_orla_sobre_as_riscas() {
    let Some(dir) = std::env::var_os("PH2D_SONDA_DIR") else {
        return;
    };
    let dir = std::path::PathBuf::from(dir);
    let gpu = gpu_or_skip!();
    let mut s = cena_52(&gpu.device);
    s.sync_mesh(&gpu);
    let mut p = painter_impasto();
    let mut painel = Painel::novo();
    quadro(Some(&mut s), Some(&mut p));
    {
        let o = &mut s.objects[s.active];
        let xs = crate::vizinhanca_da_peca::posicoes(o.tinta.as_ref().expect("t"), o.stack.mesh());
        let pl = o.pilha.as_mut().expect("pilha");
        let base = pl.base().expect("base");
        let cor = vec![[214u8, 208, 196, 255]; xs.len()];
        let riscas = xs
            .iter()
            .map(|&x| crate::scenes::relevo_camadas::relevo_da_base(x))
            .collect();
        assert!(pl.pinta_camada(base, &cor, Some(riscas)));
        crate::tinta_da_peca::pilha::recompoe(o);
        o.tinta_suja = true;
    }
    s.sync_mesh(&gpu);
    quadro(Some(&mut s), Some(&mut p));
    painel.clica(&mut p, PAINTER_LAYERS_ADD);
    quadro(Some(&mut s), Some(&mut p));
    let curva: Vec<(f32, f32)> = (0..=30)
        .map(|k| {
            let t = k as f32 / 30.0;
            (430.0 + 30.0 * (t * 3.0).sin(), 260.0 + 180.0 * t)
        })
        .collect();
    super::painter::traco_por(&mut s, &mut p, &curva);
    s.sync_mesh(&gpu);
    quadro(Some(&mut s), Some(&mut p));
    super::painter::fotografa(&gpu, &mut s, dir.join("orla_riscas.png").as_os_str());
    let n = pilha(&s).amostras();
    let cima = pilha(&s).pilha().active().expect("cima");
    let pc = pilha(&s).plano(cima).expect("c").rgba8(n).to_vec();
    let rc = pilha(&s)
        .plano(cima)
        .expect("c")
        .relevo()
        .map(<[_]>::to_vec)
        .unwrap_or_default();
    let mut hist = [0usize; 5];
    let mut cor_fraca = [0f64; 3];
    let mut k = 0usize;
    for i in 0..n {
        let a = pc[i * 4 + 3];
        let faixa = match a {
            0 => 0,
            1..=63 => 1,
            64..=191 => 2,
            192..=254 => 3,
            _ => 4,
        };
        hist[faixa] += 1;
        if (1..=191).contains(&a) {
            for e in 0..3 {
                cor_fraca[e] += f64::from(pc[i * 4 + e]);
            }
            k += 1;
        }
    }
    let base = pilha(&s).base().expect("base");
    let rb = pilha(&s)
        .plano(base)
        .expect("b")
        .relevo()
        .map(<[_]>::to_vec)
        .unwrap_or_default();
    let peca = relevo(&s);
    let (alt, inc) = da_placa(&mut s, &gpu);
    let cpu = inclinacoes_de(&s, &peca);
    let difere =
        |i: usize| (0..3).any(|e| (inc[i][e] - cpu[i][e]).abs() > 1e-4 * (1.0 + cpu[i][e].abs()));
    let inc_dif = (0..n).filter(|&i| difere(i)).count();
    let pior_inc = (0..n)
        .map(|i| {
            (0..3)
                .map(|e| (inc[i][e] - cpu[i][e]).abs())
                .fold(0.0f32, f32::max)
        })
        .fold(0.0f32, f32::max);
    eprintln!("INCLINACOES: a placa difere da CPU do zero em {inc_dif} amostras (pior {pior_inc})");
    let encosta: Vec<usize> = (0..n)
        .filter(|&i| pc[i * 4 + 3] == 0 && rc.get(i).is_some_and(|r| r[0] != 0.0))
        .collect();
    let peca_dif = encosta.iter().filter(|&&i| peca[i][0] != rb[i][0]).count();
    let placa_dif = encosta.iter().filter(|&&i| alt[i][0] != rb[i][0]).count();
    let alturas_encosta: Vec<f32> = encosta.iter().take(5).map(|&i| rc[i][0]).collect();
    let alturas_base: Vec<f32> = encosta.iter().take(5).map(|&i| rb[i][0]).collect();
    {
        let o = &s.objects[s.active];
        let xs = crate::vizinhanca_da_peca::posicoes(o.tinta.as_ref().expect("t"), o.stack.mesh());
        let orig: Vec<[f32; 2]> = xs
            .iter()
            .map(|&x| crate::scenes::relevo_camadas::relevo_da_base(x))
            .collect();
        let mudadas = (0..n).filter(|&i| rb[i] != orig[i]).count();
        let peca_vs_orig = (0..n)
            .filter(|&i| (peca[i][0] - orig[i][0]).abs() > 1e-6 && pc[i * 4 + 3] == 0)
            .count();
        eprintln!(
            "BASE: {mudadas} amostras do relevo da base mudaram com o traço de cima; a peça difere das riscas ORIGINAIS fora da tinta de cima em {peca_vs_orig}"
        );
    }
    eprintln!(
        "ENCOSTA {}: a peça difere das riscas em {peca_dif}, a placa em {placa_dif}; alturas de cima {alturas_encosta:?}, das riscas {alturas_base:?}",
        encosta.len()
    );
    {
        let o = &mut s.objects[s.active];
        o.pilha.as_mut().expect("p").define_visivel(cima, false);
        crate::tinta_da_peca::pilha::recompoe(o);
    }
    s.sync_mesh(&gpu);
    super::painter::fotografa(&gpu, &mut s, dir.join("orla_escondida.png").as_os_str());
    {
        let peca_e = relevo(&s);
        let (alt_e, inc_e) = da_placa(&mut s, &gpu);
        let cpu_e = inclinacoes_de(&s, &peca_e);
        let a_dif = (0..n).filter(|&i| alt_e[i] != peca_e[i]).count();
        let i_dif = (0..n)
            .filter(|&i| {
                (0..3).any(|e| (inc_e[i][e] - cpu_e[i][e]).abs() > 1e-4 * (1.0 + cpu_e[i][e].abs()))
            })
            .count();
        let o = &s.objects[s.active];
        let xs = crate::vizinhanca_da_peca::posicoes(o.tinta.as_ref().expect("t"), o.stack.mesh());
        let orig = (0..n)
            .filter(|&i| {
                (peca_e[i][0] - crate::scenes::relevo_camadas::relevo_da_base(xs[i])[0]).abs()
                    > 1e-6
            })
            .count();
        eprintln!(
            "ESCONDIDA: peça vs riscas originais {orig} · alturas placa≠CPU {a_dif} · inclinações placa≠CPU {i_dif}"
        );
    }
    {
        let mut linhas = Vec::new();
        for (lo, hi) in [(1u8, 63u8), (64, 191), (192, 254), (255, 255)] {
            let sel: Vec<usize> = (0..n)
                .filter(|&i| {
                    (lo..=hi).contains(&pc[i * 4 + 3]) && rc.get(i).is_some_and(|r| r[0] != 0.0)
                })
                .collect();
            let m = |f: &dyn Fn(usize) -> f32| {
                sel.iter().map(|&i| f64::from(f(i))).sum::<f64>() / sel.len().max(1) as f64
            };
            linhas.push(format!(
                "alfa {lo}-{hi}: {} amostras, corpo médio {:.2}, altura média {:.4}",
                sel.len(),
                m(&|i| rc[i][1]),
                m(&|i| rc[i][0])
            ));
        }
        eprintln!("FIO: {}", linhas.join(" | "));
    }
    let rel_sem_tinta = (0..n)
        .filter(|&i| pc[i * 4 + 3] == 0 && rc.get(i).is_some_and(|r| r[0] != 0.0))
        .count();
    let corpo_sem_tinta = (0..n)
        .filter(|&i| pc[i * 4 + 3] == 0 && rc.get(i).is_some_and(|r| r[1] > 0.0))
        .count();
    eprintln!(
        "ORLA: alfa da de cima [0 | 1-63 | 64-191 | 192-254 | 255] = {hist:?} · cor média das fracas {:?} · relevo sem tinta {rel_sem_tinta} · corpo sem tinta {corpo_sem_tinta}",
        cor_fraca.map(|v| (v / k.max(1) as f64).round())
    );
}

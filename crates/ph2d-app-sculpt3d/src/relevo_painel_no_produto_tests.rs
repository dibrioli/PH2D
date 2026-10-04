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
            nova.set_impasto_depth(cima, 1.0 - q as f32 * 0.04);
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

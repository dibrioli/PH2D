//! 🔎 **A PAREDE DO IMPASTO NA PEÇA CONTRA O PAINTER 2D** (o oráculo, `docs/3D/30` §19) — a MESMA
//! pincelada (mesmo pincel, mesma cor, mesmos pontos, as mesmas lâmpadas) pintada numa tela 2D da cor
//! da base e na peça, e o perfil transversal das duas: o efeito do relevo é a razão acesa / sem relevo,
//! por canal, no espaço que o ecrã mostra (sRGB8).

use super::cena_52;
use super::painel::painter_vermelho;
use super::painter::{retrato, traco_por};
use crate::painter_na_malha::quadro;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_mesh_render::Lighting;
use ph2d_tool_painter::{PaintMedia, PainterTool};

const W: u32 = 900;
const H: u32 = 700;
const COR_DA_BASE: [u8; 4] = [214, 208, 196, 255];
const TINTA: [u8; 3] = [230, 30, 30];

fn pincel(p: &mut PainterTool, px: f32) {
    p.set_paint_media(match std::env::var("PH2D_SONDA_MEIO").as_deref() {
        Ok("digital") => PaintMedia::Digital,
        _ => PaintMedia::Impasto,
    });
    p.set_brush_color_srgb8(TINTA);
    p.set_brush_size_px(px);
}

/// Uma pincelada horizontal pelo meio da bola (de frente: a normal da base é a do ecrã).
fn pontos() -> Vec<(f32, f32)> {
    (0..=30).map(|k| (330.0 + 8.0 * k as f32, 350.0)).collect()
}

/// O ORÁCULO — a pincelada numa tela 2D de `900×700` da cor da base: `(acesa, sem luz, altura,
/// cobertura)`, por píxel.
fn no_2d(px: f32) -> (Vec<u8>, Vec<u8>, Vec<f32>, Vec<f32>) {
    let mut t = painter_vermelho();
    pincel(&mut t, px);
    t.set_source(COR_DA_BASE.repeat((W * H) as usize), W, H);
    let cp = |(x, y): (f32, f32), phase| CanvasPointer {
        pos: [x, y],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    let pts = pontos();
    t.on_canvas_pointer(cp(pts[0], PointerPhase::Down));
    for &q in &pts[1..] {
        t.on_canvas_pointer(cp(q, PointerPhase::Move));
    }
    t.on_canvas_pointer(cp(pts[pts.len() - 1], PointerPhase::Up));
    let acesa = t.take_preview_arc().expect("a tela acesa").0.to_vec();
    t.toggle_impasto_show();
    let apagada = t.take_preview_arc().expect("a tela sem luz").0.to_vec();
    let id = t.layers().active().expect("a camada");
    let jan = (0, 0, W, H);
    let n = (W * H) as usize;
    let altura = t
        .layer_height_px_in(id, jan)
        .unwrap_or_else(|| vec![0.0; n]);
    let cobertura = t.layer_cover_in(id, jan).unwrap_or_else(|| vec![0.0; n]);
    (acesa, apagada, altura, cobertura)
}

/// A peça: `(com o relevo, com a profundidade da camada pintada a 0)`, fotografadas no modo `luz`;
/// a pincelada na base ou, com `camada`, numa camada nova por cima dela.
fn na_peca(gpu: &ph2d_gpu::GpuContext, px: f32, luz: Lighting, camada: bool) -> (Vec<u8>, Vec<u8>) {
    let mut s = cena_52(&gpu.device);
    s.sync_mesh(gpu);
    let mut p = painter_vermelho();
    pincel(&mut p, px);
    quadro(Some(&mut s), Some(&mut p));
    {
        let o = &mut s.objects[s.active];
        let xs = crate::vizinhanca_da_peca::posicoes(o.tinta.as_ref().expect("t"), o.stack.mesh());
        let pl = o.pilha.as_mut().expect("pilha");
        let base = pl.base().expect("base");
        let riscas = std::env::var_os("PH2D_SONDA_RISCAS").map(|_| {
            xs.iter()
                .map(|&x| crate::scenes::relevo_camadas::relevo_da_base(x))
                .collect()
        });
        assert!(pl.pinta_camada(base, &vec![COR_DA_BASE; xs.len()], riscas));
        crate::tinta_da_peca::pilha::recompoe(o);
        o.tinta_suja = true;
    }
    s.sync_mesh(gpu);
    quadro(Some(&mut s), Some(&mut p));
    if camada {
        super::painel::Painel::novo().clica(&mut p, ph2d_tool_painter::ids::PAINTER_LAYERS_ADD);
        quadro(Some(&mut s), Some(&mut p));
    }
    traco_por(&mut s, &mut p, &pontos());
    s.sync_mesh(gpu);
    quadro(Some(&mut s), Some(&mut p));
    s.lighting = luz;
    let com = retrato(gpu, &mut s);
    {
        let o = &mut s.objects[s.active];
        let pl = o.pilha.as_mut().expect("p");
        let pintada = pl.pilha().active().expect("a camada pintada");
        let mut m = pl.pilha().clone();
        m.set_impasto_depth(pintada, 0.0);
        pl.troca_metadado(m).expect("profundidade 0");
        crate::tinta_da_peca::pilha::recompoe(o);
    }
    s.sync_mesh(gpu);
    let sem = retrato(gpu, &mut s);
    (com, sem)
}

/// A média de uma linha `y` sobre as colunas `440..=460` (rgb, `passo` 3 ou 4 bytes por píxel).
fn media(img: &[u8], passo: usize, y: u32) -> [f32; 3] {
    let mut m = [0.0f32; 3];
    for x in 440..=460u32 {
        let i = ((y * W + x) as usize) * passo;
        for c in 0..3 {
            m[c] += f32::from(img[i + c]) / 21.0;
        }
    }
    m
}

fn media1(v: &[f32], y: u32) -> f32 {
    (440..=460u32).map(|x| v[(y * W + x) as usize]).sum::<f32>() / 21.0
}

fn razao(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [0, 1, 2].map(|c| a[c] / b[c].max(1.0))
}

/// Byte sRGB médio → luz.
fn luz(v: f32) -> f32 {
    ph2d_color::srgb::srgb_to_linear_unit(v / 255.0)
}

/// ⭐⭐⭐ **GATE — A PAREDE DO IMPASTO NA PEÇA LÊ-SE COMO NO 2D** (`docs/3D/30` §19; reports do dono
/// de 04/10, a orla cinzenta à volta de uma pincelada grossa). O oráculo é o Painter 2D a pintar a
/// MESMA pincelada (`48 px`, `Impasto`, as mesmas lâmpadas) numa tela da cor da base; o efeito do
/// relevo é a razão «com relevo / profundidade 0», por canal, no perfil transversal. Na base e numa
/// camada nova:
///
/// 1. **a orla** — onde o 2D tem `≤ 20 %` de tinta, o efeito da peça é o do 2D a `0,1` (a lei velha:
///    `0,60` contra `0,90` a 15 % de tinta; o resto do desvio é o brilho da PRÓPRIA esfera, que a
///    parede inclinada deixa e o 2D não tem);
/// 2. **o véu do céu** — na parede de sombra o relevo não tira o croma (`(G/R)` com relevo `≤ 1,5×`
///    o sem relevo; o céu reflectido na parede dava `3×`), em PBR e com as lâmpadas;
/// 3. **a cor** — no modo plano a tinta e a base estão na razão das suas LUZES (`0,021` no verde; o
///    código cru a multiplicar a luz dava `0,144`).
///
/// ⛔ CONTROLO no mesmo gate: o relevo é real — a parede de sombra escurece (`R ≤ 0,8`) e a iluminada
/// clareia (`R ≥ 1,05`) em PBR e com as lâmpadas; nenhum dos três passa por apagar o relevo.
#[test]
#[ignore = "precisa de adaptador"]
fn a_parede_do_impasto_na_peca_le_como_no_2d() {
    let gpu = gpu_or_skip!();
    let (acesa, apagada, _, cob) = no_2d(48.0);
    let linhas = 296..=404u32;
    for camada in [false, true] {
        let onde = if camada { "camada nova" } else { "base" };
        for (nome, modo) in [
            ("PBR", Lighting::Pbr),
            ("RIG", Lighting::Rig),
            ("MATCAP0", Lighting::Matcap(0)),
        ] {
            let (com, sem) = na_peca(&gpu, 48.0, modo, camada);
            let (mut sombra, mut luz_max, mut orla, mut pior) = (f32::MAX, 0.0f32, 0usize, 0.0f32);
            for y in linhas.clone() {
                let r2 = razao(media(&acesa, 4, y), media(&apagada, 4, y));
                let (c, s) = (media(&com, 3, y), media(&sem, 3, y));
                let rp = razao(c, s);
                let c2 = media1(&cob, y);
                if c2 <= 0.2 {
                    orla += usize::from(c2 > 0.0);
                    for k in 0..3 {
                        pior = pior.max((rp[k] - r2[k]).abs());
                        assert!(
                            (rp[k] - r2[k]).abs() <= 0.1,
                            "{onde} {nome} y {y} (2D com {c2:.2} de tinta): o efeito do relevo é \
                             {rp:.2?} na peça e {r2:.2?} no 2D — a orla"
                        );
                    }
                }
                if c2 >= 0.99 && r2[0] <= 0.5 {
                    sombra = sombra.min(rp[0]);
                    if modo != Lighting::Matcap(0) {
                        let croma = (c[1] / c[0].max(1.0)) / (s[1] / s[0].max(1.0)).max(1e-3);
                        assert!(
                            croma <= 1.5,
                            "{onde} {nome} y {y}: a parede de sombra perde o croma ({c:.0?} contra \
                             {s:.0?}, ×{croma:.2}) — o véu do céu"
                        );
                    }
                }
                luz_max = luz_max.max(rp[0]);
            }
            eprintln!("{onde} {nome}: na orla o pior desvio do 2D é {pior:.3}");
            assert!(
                orla >= 2,
                "a fixtura: {orla} linhas na borda da tinta do 2D"
            );
            assert!(
                sombra < f32::MAX,
                "a fixtura: o 2D não tem parede de sombra"
            );
            if modo != Lighting::Matcap(0) {
                assert!(
                    sombra <= 0.8 && luz_max >= 1.05,
                    "CONTROLO {onde} {nome}: o relevo não se vê (sombra {sombra:.2}, luz {luz_max:.2})"
                );
            }
        }
        let (plano, _) = na_peca(&gpu, 48.0, Lighting::Flat, camada);
        let (tinta, base) = (media(&plano, 3, 345), media(&plano, 3, 296));
        for (k, (t, b)) in [(TINTA[0], COR_DA_BASE[0]), (TINTA[1], COR_DA_BASE[1])]
            .into_iter()
            .enumerate()
        {
            let quer = luz(f32::from(t)) / luz(f32::from(b));
            let tem = luz(tinta[k]) / luz(base[k]);
            assert!(
                (tem / quer - 1.0).abs() <= 0.2,
                "{onde}: no modo plano a tinta e a base estão na razão {tem:.4} (canal {k}); as luzes \
                 delas pedem {quer:.4} — a cor pintada multiplica a luz como código"
            );
        }
    }
}

/// 🔎 O perfil transversal (`y`, de cima para baixo) da pincelada no 2D e na peça, nos modos de luz da
/// peça. `PH2D_SONDA_PINCEL` (px, `48`), `PH2D_SONDA_RISCAS` (a base com as riscas da cena `=55`),
/// `PH2D_SONDA_CAMADA` (numa camada nova), `PH2D_SONDA_MEIO=digital`.
#[test]
#[ignore = "sonda: precisa de adaptador"]
fn diag_a_parede_contra_o_2d() {
    let gpu = gpu_or_skip!();
    let px = std::env::var("PH2D_SONDA_PINCEL")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(48.0f32);
    let (acesa, apagada, alt, cob) = no_2d(px);
    let modos = [
        ("PLANO", Lighting::Flat),
        ("PBR", Lighting::Pbr),
        ("RIG", Lighting::Rig),
        ("MATCAP0", Lighting::Matcap(0)),
    ];
    let pecas: Vec<_> = modos
        .iter()
        .map(|&(_, l)| na_peca(&gpu, px, l, std::env::var_os("PH2D_SONDA_CAMADA").is_some()))
        .collect();
    if let Some(dir) = std::env::var_os("PH2D_SONDA_DIR") {
        let dir = std::path::PathBuf::from(dir);
        let rgb: Vec<u8> = acesa.chunks(4).flat_map(|q| [q[0], q[1], q[2]]).collect();
        image::save_buffer(
            dir.join("parede_2d.png"),
            &rgb,
            W,
            H,
            image::ColorType::Rgb8,
        )
        .expect("png");
        for ((nome, _), (com, sem)) in modos.iter().zip(&pecas) {
            for (sufixo, img) in [("com", com), ("sem", sem)] {
                image::save_buffer(
                    dir.join(format!("parede_{nome}_{sufixo}.png")),
                    img,
                    W,
                    H,
                    image::ColorType::Rgb8,
                )
                .expect("png");
            }
        }
    }
    eprintln!(
        "y   | 2D: cob  alt | sem luz        acesa          razão          | {}",
        modos
            .iter()
            .map(|(n, _)| format!("{n}: sem relevo / com / razão"))
            .collect::<Vec<_>>()
            .join(" | ")
    );
    let f = |v: [f32; 3], d: usize| {
        format!(
            "{:>w$.d$} {:>w$.d$} {:>w$.d$}",
            v[0],
            v[1],
            v[2],
            w = d + 4,
            d = d
        )
    };
    for y in (300..=400u32).step_by(2) {
        let (l, a) = (media(&acesa, 4, y), media(&apagada, 4, y));
        let mut linha = format!(
            "{y} | {:.2} {:>5.1} | {} | {} | {}",
            media1(&cob, y),
            media1(&alt, y),
            f(a, 0),
            f(l, 0),
            f(razao(l, a), 2)
        );
        for (com, sem) in &pecas {
            let (c, s) = (media(com, 3, y), media(sem, 3, y));
            linha.push_str(&format!(" | {} {} {}", f(s, 0), f(c, 0), f(razao(c, s), 2)));
        }
        eprintln!("{linha}");
    }
}

// ── As sondas da 2.ª e da 3.ª foto do dono (§18), vindas do `relevo_painel` (o tecto de linhas) ──

mod orla {
    use super::super::painel::Painel;
    use super::super::relevo_painel::{da_placa, inclinacoes_de, painter_impasto, pilha, relevo};
    use super::cena_52;
    use crate::painter_na_malha::quadro;
    use ph2d_tool_painter::ids::PAINTER_LAYERS_ADD;

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
        let horizontal: Vec<(f32, f32)> =
            (0..=30).map(|k| (330.0 + 8.0 * k as f32, 350.0)).collect();
        super::super::painter::traco_por(&mut s, &mut p, &horizontal);
        s.sync_mesh(&gpu);
        quadro(Some(&mut s), Some(&mut p));
        painel.clica(&mut p, PAINTER_LAYERS_ADD);
        quadro(Some(&mut s), Some(&mut p));
        p.set_brush_color_srgb8([230, 30, 30]);
        let vertical: Vec<(f32, f32)> = (0..=30).map(|k| (450.0, 250.0 + 7.0 * k as f32)).collect();
        super::super::painter::traco_por(&mut s, &mut p, &vertical);
        s.sync_mesh(&gpu);
        quadro(Some(&mut s), Some(&mut p));
        super::super::painter::fotografa(&gpu, &mut s, dir.join("orla.png").as_os_str());
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
        p.set_brush_size_px(
            std::env::var("PH2D_SONDA_PINCEL")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(28.0),
        );
        let mut painel = Painel::novo();
        quadro(Some(&mut s), Some(&mut p));
        {
            let o = &mut s.objects[s.active];
            let xs =
                crate::vizinhanca_da_peca::posicoes(o.tinta.as_ref().expect("t"), o.stack.mesh());
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
        if std::env::var_os("PH2D_SONDA_NA_BASE").is_none() {
            painel.clica(&mut p, PAINTER_LAYERS_ADD);
            quadro(Some(&mut s), Some(&mut p));
        }
        let curva: Vec<(f32, f32)> = (0..=30)
            .map(|k| {
                let t = k as f32 / 30.0;
                (430.0 + 30.0 * (t * 3.0).sin(), 260.0 + 180.0 * t)
            })
            .collect();
        super::super::painter::traco_por(&mut s, &mut p, &curva);
        s.sync_mesh(&gpu);
        quadro(Some(&mut s), Some(&mut p));
        super::super::painter::fotografa(&gpu, &mut s, dir.join("orla_riscas.png").as_os_str());
        {
            let cima = pilha(&s).pilha().active().expect("cima");
            let o = &mut s.objects[s.active];
            let pl = o.pilha.as_mut().expect("p");
            let mut m = pl.pilha().clone();
            m.set_impasto_depth(cima, 0.0);
            pl.troca_metadado(m).expect("profundidade 0");
            crate::tinta_da_peca::pilha::recompoe(o);
        }
        s.sync_mesh(&gpu);
        super::super::painter::fotografa(&gpu, &mut s, dir.join("orla_sem_relevo.png").as_os_str());
        {
            let cima = pilha(&s).pilha().active().expect("cima");
            let o = &mut s.objects[s.active];
            let pl = o.pilha.as_mut().expect("p");
            let mut m = pl.pilha().clone();
            m.set_impasto_depth(cima, 1.0);
            pl.troca_metadado(m).expect("profundidade 1");
            crate::tinta_da_peca::pilha::recompoe(o);
        }
        s.sync_mesh(&gpu);
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
        let difere = |i: usize| {
            (0..3).any(|e| (inc[i][e] - cpu[i][e]).abs() > 1e-4 * (1.0 + cpu[i][e].abs()))
        };
        let inc_dif = (0..n).filter(|&i| difere(i)).count();
        let pior_inc = (0..n)
            .map(|i| {
                (0..3)
                    .map(|e| (inc[i][e] - cpu[i][e]).abs())
                    .fold(0.0f32, f32::max)
            })
            .fold(0.0f32, f32::max);
        eprintln!(
            "INCLINACOES: a placa difere da CPU do zero em {inc_dif} amostras (pior {pior_inc})"
        );
        let encosta: Vec<usize> = (0..n)
            .filter(|&i| pc[i * 4 + 3] == 0 && rc.get(i).is_some_and(|r| r[0] != 0.0))
            .collect();
        let peca_dif = encosta.iter().filter(|&&i| peca[i][0] != rb[i][0]).count();
        let placa_dif = encosta.iter().filter(|&&i| alt[i][0] != rb[i][0]).count();
        let alturas_encosta: Vec<f32> = encosta.iter().take(5).map(|&i| rc[i][0]).collect();
        let alturas_base: Vec<f32> = encosta.iter().take(5).map(|&i| rb[i][0]).collect();
        {
            let o = &s.objects[s.active];
            let xs =
                crate::vizinhanca_da_peca::posicoes(o.tinta.as_ref().expect("t"), o.stack.mesh());
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
        super::super::painter::fotografa(&gpu, &mut s, dir.join("orla_escondida.png").as_os_str());
        {
            let peca_e = relevo(&s);
            let (alt_e, inc_e) = da_placa(&mut s, &gpu);
            let cpu_e = inclinacoes_de(&s, &peca_e);
            let a_dif = (0..n).filter(|&i| alt_e[i] != peca_e[i]).count();
            let i_dif = (0..n)
                .filter(|&i| {
                    (0..3).any(|e| {
                        (inc_e[i][e] - cpu_e[i][e]).abs() > 1e-4 * (1.0 + cpu_e[i][e].abs())
                    })
                })
                .count();
            let o = &s.objects[s.active];
            let xs =
                crate::vizinhanca_da_peca::posicoes(o.tinta.as_ref().expect("t"), o.stack.mesh());
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
}

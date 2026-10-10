//! **As opções da W1g** (dono 10/10: *«criando novas opções»*) — desligadas de fábrica, o traço de HOJE
//! ao byte. A cerca: o S do oráculo e a bancada do doc 06 §2, nas três inclinações, gravados em `fcc685e43`
//! (a ponta da W1f) ANTES do primeiro commit da W1g. Um hash que muda aqui com as opções no neutro é a
//! opção a vazar para o produto: a cura é desfazer, nunca regravar o número.
use super::sonda_achatamento::{Variante, cena};
use super::sonda_gota::bancada;

fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, &b| {
        (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

const AGUAS: [f64; 4] = [1.0, 0.7, 0.4, 0.2];

/// (cena, inclinação, hash). Gravados em `fcc685e43`; regravados UMA vez a 10/10 pela ordem do dono sobre
/// o papel (sem Kind, o grão sem desenho em vez do Cold — o traço sem papel mudou de propósito; as opções
/// continuam a não mexer um byte).
const ESPERADO: [(&str, Option<u8>, u64); 6] = [
    ("S", None, 0x2422_98ff_e3b5_cc05),
    ("S", Some(4), 0xdda0_0431_822c_7a2e),
    ("S", Some(8), 0xd86f_70c4_a969_e482),
    ("bancada", None, 0xc9f6_4e0a_65c8_fe4a),
    ("bancada", Some(4), 0x07c7_18e0_9450_5a29),
    ("bancada", Some(8), 0xaa38_a84f_8d40_328a),
];

#[test]
fn com_as_opcoes_desligadas_o_s_e_a_bancada_sao_os_de_hoje_ao_byte() {
    let mut mudou = Vec::new();
    for (nome, anel, esperado) in ESPERADO {
        let px = if nome == "S" {
            cena(
                &Variante::default(),
                anel.map(|a| (3, a)),
                30,
                &mut |_, _| {},
            )
            .0
        } else {
            bancada(0.005, 0.0, &[], anel, AGUAS)
        };
        let h = fnv1a(&px);
        eprintln!("[opções] {nome} anel {anel:?}: {h:#018x}");
        if h != esperado {
            mudou.push(format!(
                "{nome} anel {anel:?}: {h:#018x} (esperado {esperado:#018x})"
            ));
        }
    }
    assert!(
        mudou.is_empty(),
        "o traço de hoje mudou:\n{}",
        mudou.join("\n")
    );
}

/// **Heavy Edge a meio, a tela a meia: a orla de baixo pesa como no Rebelle** (1,3–1,6 no traço de água
/// 1; Rebelle 1,6), com o papel Cold escolhido — onde o Rebelle foi medido. O controlo é o Cold sem a
/// opção (0,77) a reprovar a mesma barra. SEM papel (o grão sem desenho, dono 10/10) a orla já nasce a
/// 1,45 e a opção tem de a aumentar. Com a opção ligada a régua do recto continua verde no S (meia e
/// máxima) e a tela DIREITA não muda um byte (sem `|g|` a lei não age). Pelo caminho do produto.
#[test]
fn a_heavy_edge_a_meio_pesa_na_orla_de_baixo_como_no_rebelle() {
    let orla = |v: f64, anel: Option<u8>, cold: bool| {
        let px = super::sonda_gota::bancada_toda(0.005, 0.0, &[], anel, AGUAS, &|_| {}, &|t| {
            t.set_wet_borda_que_pesa(v);
            if cold {
                t.set_brush_paper_kind(ph2d_painter_brush::TextureKind::PaperCold.to_u8());
            }
        });
        (super::gota_tests::le(&px, 0).0, px)
    };
    let (hoje, _) = orla(0.0, Some(4), true);
    let (com, _) = orla(0.5, Some(4), true);
    let (nada, _) = orla(0.0, Some(4), false);
    let (nada_com, _) = orla(0.5, Some(4), false);
    eprintln!(
        "[heavy edge] meia, água 1: Cold {hoje:.2} → {com:.2} · sem papel {nada:.2} → {nada_com:.2} (Rebelle 1,6)"
    );
    assert!(
        !(1.3..=1.6).contains(&hoje),
        "CONTROLO: o Cold sem a opção já dá {hoje:.2} — a barra não discrimina"
    );
    assert!(
        (1.3..=1.6).contains(&com),
        "a Heavy Edge a 0,5 dá a orla {com:.2} a meia no Cold (barra 1,3–1,6, Rebelle 1,6)"
    );
    assert!(
        nada_com > nada + 0.2,
        "sem papel a Heavy Edge não pesa a orla ({nada:.2} → {nada_com:.2})"
    );
    let (_, direita) = orla(1.0, None, false);
    let (_, direita_sem) = orla(0.0, None, false);
    assert!(
        direita == direita_sem,
        "a Heavy Edge mudou a tela DIREITA (sem inclinação não há para onde descer)"
    );
    for anel in [4u8, 8] {
        let v = Variante {
            heavy_edge: Some(0.5),
            ..Variante::default()
        };
        let l = super::achatamento_tests::le(&cena(&v, Some((3, anel)), 30, &mut |_, _| {}).0);
        eprintln!("[heavy edge] S anel {anel}: recto {l:?}");
        if let Err(e) = super::achatamento_tests::barra(l) {
            panic!("a Heavy Edge corta o S em rectângulos no anel {anel}: {e}");
        }
    }
}

/// **Live Water: a água anda com o traço ABERTO.** Pelo produto (a opção autorada), o motor dá passos
/// enquanto o ponteiro está em baixo; o controlo é a fábrica, que não dá nenhum (a W1f).
#[test]
fn com_a_live_water_a_agua_corre_com_o_traco() {
    use crate::tool::PainterTool;
    use ph2d_editor_core::Tool;
    use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
    let passos = |viva: bool| {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; 300 * 200 * 4], 300, 200);
        t.set_paint_tool_mode("wetpaint");
        t.set_wet_relogio_fixo(true);
        t.set_wet_agua_viva(viva);
        let p = |x: f32, phase| CanvasPointer {
            pos: [x, 100.0],
            pressure: 1.0,
            tilt: [0.0, 0.0],
            phase,
        };
        t.on_canvas_pointer(p(40.0, PointerPhase::Down));
        for k in 1..=60 {
            t.on_canvas_pointer(p(40.0 + 3.0 * k as f32, PointerPhase::Move));
            t.on_tick(1000.0 / 60.0);
        }
        let s = t.paint.wetpaint.session.as_mut().expect("sessão");
        s.bring_home();
        s.engine.sim.frame
    };
    let (fabrica, viva) = (passos(false), passos(true));
    eprintln!(
        "[live water] passos da água com o traço aberto: fábrica {fabrica} · Live Water {viva}"
    );
    assert_eq!(
        fabrica, 0,
        "CONTROLO: a fábrica já corre a água no traço (a W1f desligou-a)"
    );
    assert!(viva > 0, "a Live Water ligada e a água parada no traço");
}

/// **Sem papel escolhido o traço não tem o DESENHO de um papel** (smoke do dono 10/10: «mesmo com Paper:
/// None e Relief 0, o papel exerce efeito sobre o traço»; a escolha dele: o grão fino sem desenho,
/// `prancha_w1g_sem_papel.png`). Com Kind None o papel do motor são as alturas do papel de fábrica ao
/// acaso: a correlação entre células vizinhas é ~0 (o Cold, o controlo, passa de 0,5); o Tooth não muda
/// um byte do traço (com o Cold muda); e a tinta total é a do Cold (±3 %). Mutação que sangra: semear o
/// Cold de fábrica sem Kind (o produto até 10/10).
#[test]
fn sem_papel_o_traco_nao_tem_o_desenho_de_um_papel() {
    use crate::tool::PainterTool;
    use crate::tool::paint::media::PaintMedia;
    use ph2d_editor_core::Tool;
    use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
    use ph2d_painter_brush::TextureKind;
    let traco = |papel: Option<TextureKind>, tooth: f32| {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; 200 * 120 * 4], 200, 120);
        t.set_paint_media(PaintMedia::WetPaint);
        t.set_wet_relogio_fixo(true);
        if let Some(k) = papel {
            t.set_brush_paper_kind(k.to_u8());
        }
        t.set_brush_paper_depth(tooth);
        let p = |x: f32, phase| CanvasPointer {
            pos: [x, 60.0],
            pressure: 1.0,
            tilt: [0.0, 0.0],
            phase,
        };
        t.on_canvas_pointer(p(30.0, PointerPhase::Down));
        for k in 1..=45 {
            t.on_canvas_pointer(p(30.0 + 3.0 * k as f32, PointerPhase::Move));
            t.on_tick(8.0);
        }
        t.on_canvas_pointer(p(170.0, PointerPhase::Up));
        for _ in 0..90 {
            t.on_tick(1000.0 / 60.0);
        }
        let s = t.paint.wetpaint.session.as_mut().expect("sessão");
        s.bring_home();
        let g = s.engine.active_grid();
        // A correlação do papel com o vizinho da direita, no miolo da folha.
        let (mut a, mut b): (Vec<f64>, Vec<f64>) = (Vec::new(), Vec::new());
        for y in 10..g.h - 10 {
            for x in 10..g.w - 10 {
                let i = x + y * g.s;
                a.push(f64::from(g.paper[i]));
                b.push(f64::from(g.paper[i + 1]));
            }
        }
        let n = a.len() as f64;
        let (ma, mb) = (a.iter().sum::<f64>() / n, b.iter().sum::<f64>() / n);
        let cov: f64 = a
            .iter()
            .zip(&b)
            .map(|(x, y)| (x - ma) * (y - mb))
            .sum::<f64>()
            / n;
        let va: f64 = a.iter().map(|x| (x - ma).powi(2)).sum::<f64>() / n;
        let vb: f64 = b.iter().map(|y| (y - mb).powi(2)).sum::<f64>() / n;
        (t.run_full().0, cov / (va * vb).sqrt().max(1e-12))
    };
    let tinta = |px: &[u8]| -> f64 {
        px.chunks(4)
            .map(|p| f64::from(255 - p[..3].iter().min().copied().unwrap_or(255)))
            .sum()
    };
    let (cold0, _) = traco(Some(TextureKind::PaperCold), 0.0);
    let (cold1, r_cold) = traco(Some(TextureKind::PaperCold), 1.0);
    let (cold2, _) = traco(Some(TextureKind::PaperCold), 2.0);
    let (nada0, _) = traco(None, 0.0);
    let (nada1, r_nada) = traco(None, 1.0);
    let (nada2, _) = traco(None, 2.0);
    let razao = tinta(&nada1) / tinta(&cold1);
    eprintln!(
        "[papel] correlação com o vizinho: Cold {r_cold:.3} · sem papel {r_nada:.3} · tinta sem ÷ com {razao:.3}"
    );
    assert!(
        r_cold > 0.5,
        "CONTROLO: o Cold sem desenho ({r_cold:.3}) — a régua não discrimina"
    );
    assert!(
        cold0 != cold2,
        "CONTROLO: com o Cold o Tooth não mudou o traço"
    );
    assert!(
        r_nada.abs() < 0.05,
        "sem papel o papel do motor tem desenho (correlação {r_nada:.3})"
    );
    assert!(
        nada0 == nada1 && nada1 == nada2,
        "sem papel escolhido o Tooth mudou o traço"
    );
    assert!(
        (razao - 1.0).abs() <= 0.03,
        "sem papel a tinta total é {razao:.3} da do Cold"
    );
}

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

/// (cena, inclinação, hash de `fcc685e43`).
const ESPERADO: [(&str, Option<u8>, u64); 6] = [
    ("S", None, 0x5e34_16a6_2bd7_a539),
    ("S", Some(4), 0x115d_4dcf_a1e1_f5ac),
    ("S", Some(8), 0xe3b3_1067_8581_314d),
    ("bancada", None, 0x71bd_2a66_ca11_8504),
    ("bancada", Some(4), 0x96a7_8ff2_afb0_1f85),
    ("bancada", Some(8), 0x7fe7_f482_a360_1053),
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
/// 1; Rebelle 1,6). O controlo é o produto de hoje (0,77) a reprovar a mesma barra. Com a opção ligada a
/// régua do recto continua verde no S (meia e máxima) e a tela DIREITA não muda um byte (sem `|g|` a lei
/// não age). Pelo caminho do produto: o autorado → o reconcile → o motor.
#[test]
fn a_heavy_edge_a_meio_pesa_na_orla_de_baixo_como_no_rebelle() {
    let orla = |v: f64, anel: Option<u8>| {
        let px = super::sonda_gota::bancada_toda(0.005, 0.0, &[], anel, AGUAS, &|_| {}, &|t| {
            t.set_wet_borda_que_pesa(v)
        });
        (super::gota_tests::le(&px, 0).0, px)
    };
    let (hoje, _) = orla(0.0, Some(4));
    let (com, _) = orla(0.5, Some(4));
    eprintln!(
        "[heavy edge] meia, água 1: orla de baixo ÷ de cima {hoje:.2} → {com:.2} (Rebelle 1,6)"
    );
    assert!(
        !(1.3..=1.6).contains(&hoje),
        "CONTROLO: o produto sem a opção já dá {hoje:.2} — a barra não discrimina"
    );
    assert!(
        (1.3..=1.6).contains(&com),
        "a Heavy Edge a 0,5 dá a orla {com:.2} a meia (barra 1,3–1,6, Rebelle 1,6)"
    );
    let (_, direita) = orla(1.0, None);
    assert_eq!(
        fnv1a(&direita),
        ESPERADO[3].2,
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

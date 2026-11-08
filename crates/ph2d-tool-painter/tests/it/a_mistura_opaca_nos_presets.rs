//! A mistura opaca do Digital (Blend · Running color) pintada pela porta do ponteiro com os PRESETS de fábrica
//! (smoke do dono 07/11: com o Sketch Pencil o azul sobre o vermelho saía pálido ou azul puro — nada levado).
//! A cena do dono: uma mancha vermelha num traço, depois arrastos azuis por cima dela.

use crate::exame_bancada::pinceis;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_tool_painter::{ModoMisturaOpaca, PainterTool};

const W: usize = 400;
const H: usize = 240;
const Y: f32 = 120.0;

fn arrasta(t: &mut PainterTool, a: [f32; 2], b: [f32; 2]) {
    let cp = |pos: [f32; 2], phase| CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(cp(a, PointerPhase::Down));
    let n = 60;
    for k in 1..=n {
        let u = k as f32 / n as f32;
        t.on_canvas_pointer(cp(
            [a[0] + (b[0] - a[0]) * u, a[1] + (b[1] - a[1]) * u],
            PointerPhase::Move,
        ));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(cp(b, PointerPhase::Up));
}

/// O pincel `nome` (`None` = o redondo de sempre) sobre o branco, com a mancha vermelha de `x 140..300` pintada em
/// traços próprios (mistura desligada).
fn com_a_mancha(nome: Option<&str>) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
    if let Some(n) = nome {
        let p = pinceis(n).remove(0);
        p.aplica(&mut t, true);
    }
    t.set_brush_size_px(40.0);
    t.set_brush_mistura_modo(ModoMisturaOpaca::Off);
    t.set_brush_color_srgb8([200, 0, 0]);
    // A mancha: passadas sobrepostas até a tinta cobrir (o lápis tem grão: várias passadas).
    for _ in 0..4 {
        for k in 0..9 {
            let y = Y - 40.0 + 10.0 * k as f32;
            arrasta(&mut t, [140.0, y], [300.0, y]);
        }
    }
    t
}

/// A cor média de uma caixa 9×5 em `x`, à altura do arrasto.
fn media(t: &mut PainterTool, x: usize) -> [f32; 3] {
    let (px, w, h) = t.run_full();
    assert_eq!((w as usize, h as usize), (W, H), "a tela do run_full");
    let mut s = [0.0f32; 3];
    let mut n = 0.0;
    for y in Y as usize - 2..=Y as usize + 2 {
        for xx in x - 4..=x + 4 {
            let i = (y * W + xx) * 4;
            for c in 0..3 {
                s[c] += f32::from(px[i + c]);
            }
            n += 1.0;
        }
    }
    s.map(|v| v / n)
}

/// O arrasto azul do dono por cima da mancha (`modo`, amount 0,5 · density 1 · stretch 0,5); a cor dentro do vermelho.
fn arrasto_azul(nome: Option<&str>, modo: ModoMisturaOpaca) -> ([f32; 3], [f32; 3]) {
    // O `run_full` leva a tela: a mancha mede-se numa ferramenta à parte (a mesma, determinística).
    let vermelho = media(&mut com_a_mancha(nome), 260);
    let mut t = com_a_mancha(nome);
    t.set_brush_mistura_modo(modo);
    t.set_brush_mistura_numero(0, 0.5);
    t.set_brush_mistura_numero(1, 1.0);
    t.set_brush_mistura_numero(2, 0.5);
    t.set_brush_color_srgb8([0, 0, 220]);
    arrasta(&mut t, [60.0, Y], [340.0, Y]);
    (vermelho, media(&mut t, 260))
}

/// O «vermelho» de uma cor: `R − B` (o arrasto azul sobre a mancha vermelha).
fn vermelhidao(c: [f32; 3]) -> f32 {
    c[0] - c[2]
}

/// A margem (sRGB8, em `R − B`) que separa «a mistura levou o vermelho» do Off do MESMO pincel na MESMA cena.
const MARGEM_SOBRE_O_OFF: f32 = 40.0;

/// Dentro do vermelho o arrasto misturado leva o vermelho: é medida e claramente mais vermelho que o Off do mesmo
/// pincel (o critério é RELATIVO: com grão, o Off já deixa o vermelho aparecer nos furos — Charcoal, 07/11) e não
/// desbota para o branco.
fn leva_o_vermelho(nome: Option<&str>) {
    let rotulo = nome.unwrap_or("redondo");
    // Controlo: desligada, o azul passa por ali (o arrasto passou mesmo pela mancha).
    let (vermelho, off) = arrasto_azul(nome, ModoMisturaOpaca::Off);
    assert!(
        vermelho[0] > 120.0 && vermelho[2] < 80.0,
        "{rotulo}: a mancha não ficou vermelha: {vermelho:?}"
    );
    assert!(off[2] > off[0], "{rotulo} Off: o azul não passou: {off:?}");
    {
        let modo = ModoMisturaOpaca::WetBlend;
        let (_, c) = arrasto_azul(nome, modo);
        eprintln!("[mistura] {rotulo}: mancha {vermelho:?} · Off {off:?} · {modo:?} {c:?}");
        assert!(
            vermelhidao(c) - vermelhidao(off) >= MARGEM_SOBRE_O_OFF,
            "{rotulo} {modo:?}: a mistura não levou o vermelho além do Off (R−B {:.0} vs Off {:.0}): {c:?}",
            vermelhidao(c),
            vermelhidao(off)
        );
        // Pigmento: o azul sobre o vermelho ESCURECE-o (roxo-escuro/vinho), não o pinta de azul.
        assert!(
            c[0] <= vermelho[0] - 30.0,
            "{rotulo} {modo:?}: o azul não entrou: {c:?} sobre {vermelho:?}"
        );
        assert!(
            c[1] < 70.0,
            "{rotulo} {modo:?}: desbotou para o branco: {c:?}"
        );
    }
}

/// ⭐ O caso do dono: o Sketch Pencil.
#[test]
fn a_mistura_opaca_do_sketch_pencil_leva_o_vermelho() {
    leva_o_vermelho(Some("Sketch Pencil"));
}

/// Os outros novos pincéis do Digital e o redondo de sempre.
#[test]
fn a_mistura_opaca_age_nos_novos_pinceis_do_digital() {
    for nome in [
        None,
        Some("Charcoal Stick"),
        Some("Soft Airbrush"),
        Some("G-Pen"),
        Some("Turnip Pen"),
    ] {
        leva_o_vermelho(nome);
    }
}

#[test]
#[ignore = "sonda"]
fn sonda_mistura_variantes() {
    use ph2d_tool_painter::LinhaDaPonta;
    for (rotulo, f) in [
        (
            "tal qual",
            (|_: &mut PainterTool| {}) as fn(&mut PainterTool),
        ),
        ("sem dual", |t: &mut PainterTool| {
            t.set_ponta(LinhaDaPonta::DualMode, 0.0)
        }),
        ("sem grao", |t: &mut PainterTool| {
            t.set_brush_texture_kind(0)
        }),
        ("sem os dois", |t: &mut PainterTool| {
            t.set_ponta(LinhaDaPonta::DualMode, 0.0);
            t.set_brush_texture_kind(0);
        }),
    ] {
        {
            let modo = ModoMisturaOpaca::WetBlend;
            let mut t = com_a_mancha(Some("Sketch Pencil"));
            f(&mut t);
            t.set_brush_mistura_modo(modo);
            t.set_brush_color_srgb8([0, 0, 220]);
            arrasta(&mut t, [60.0, Y], [340.0, Y]);
            eprintln!("[sonda] {rotulo} {modo:?}: {:?}", media(&mut t, 260));
        }
    }
}

/// A foto (`PH2D_MISTURA_FOTO=<pasta>`): por pincel, a mancha vermelha e três arrastos azuis — Off (cima), Blend
/// (meio), Running (baixo); RGBA cru `400×240` em `<pasta>/<slug>.rgba`.
#[test]
#[ignore = "foto: corre à mão"]
fn foto_mistura_nos_presets() {
    let pasta =
        std::path::PathBuf::from(std::env::var("PH2D_MISTURA_FOTO").expect("PH2D_MISTURA_FOTO"));
    for (nome, slug) in [("Sketch Pencil", "sketch_pencil"), ("G-Pen", "gpen")] {
        let mut t = com_a_mancha(Some(nome));
        t.set_brush_color_srgb8([0, 0, 220]);
        for (modo, y) in [
            (ModoMisturaOpaca::Off, Y - 40.0),
            (ModoMisturaOpaca::WetBlend, Y),
            (ModoMisturaOpaca::WetBlend, Y + 40.0),
        ] {
            t.set_brush_mistura_modo(modo);
            arrasta(&mut t, [60.0, y], [340.0, y]);
        }
        std::fs::write(pasta.join(format!("{slug}.rgba")), t.run_full().0).expect("escreve");
    }
}

/// Desvio-padrão da luminância no miolo do arrasto sobre o branco (x 80..120, y ±6): o grão do papel.
fn grao(px: &[u8]) -> f32 {
    let mut v = Vec::new();
    for y in Y as usize - 6..=Y as usize + 6 {
        for x in 80..=120 {
            let i = (y * W + x) * 4;
            v.push(
                0.299 * f32::from(px[i])
                    + 0.587 * f32::from(px[i + 1])
                    + 0.114 * f32::from(px[i + 2]),
            );
        }
    }
    let m = v.iter().sum::<f32>() / v.len() as f32;
    (v.iter().map(|a| (a - m) * (a - m)).sum::<f32>() / v.len() as f32).sqrt()
}

/// X1 (07/11): o Grain e a Shape moldam o depósito da mistura — o Sketch Pencil guarda o grão do papel (≥ 80 % do
/// Off; antes 42 %: o depósito só via o falloff e o traço saía liso como aerógrafo).
#[test]
fn a_mistura_opaca_do_sketch_pencil_guarda_o_grao() {
    let mede = |modo: ModoMisturaOpaca| {
        let mut t = com_a_mancha(Some("Sketch Pencil"));
        t.set_brush_mistura_modo(modo);
        t.set_brush_color_srgb8([0, 0, 220]);
        arrasta(&mut t, [60.0, Y], [340.0, Y]);
        grao(&t.run_full().0)
    };
    let off = mede(ModoMisturaOpaca::Off);
    {
        let modo = ModoMisturaOpaca::WetBlend;
        let g = mede(modo);
        eprintln!("[grao] Off {off:.1} · {modo:?} {g:.1}");
        assert!(
            g >= 0.8 * off,
            "{modo:?}: grão {g:.1} < 80 % do Off {off:.1}"
        );
    }
}

/// X2 (07/11): sobre a camada VAZIA (alfa 0, o branco é o papel) o azul sai cheio, como o Off.
#[test]
fn a_mistura_opaca_sobre_a_camada_vazia_nao_desbota() {
    for nome in [None, Some("G-Pen")] {
        {
            let modo = ModoMisturaOpaca::WetBlend;
            let mut t = PainterTool::default();
            t.set_source(vec![0u8; W * H * 4], W as u32, H as u32);
            if let Some(n) = nome {
                pinceis(n).remove(0).aplica(&mut t, true);
            }
            t.set_brush_size_px(40.0);
            t.set_brush_mistura_modo(modo);
            t.set_brush_color_srgb8([0, 0, 220]);
            arrasta(&mut t, [60.0, Y], [340.0, Y]);
            let px = t.run_full().0;
            let i = ((Y as usize) * W + 200) * 4;
            let p = [px[i], px[i + 1], px[i + 2], px[i + 3]];
            assert!(
                p[3] >= 250 && p[2] >= 215,
                "{nome:?} {modo:?}: sobre o vazio desbotou: {p:?}"
            );
        }
    }
}

/// Quanto a luminância da linha central ESCURECE ao andar para a ponta da cauda (`x 340..W`, além da mancha e da borda dela): a cauda
/// clareia sem voltar atrás; anéis e contas soltas são vales de luminância (somas das descidas, sRGB8).
fn descidas_na_cauda(px: &[u8]) -> f32 {
    let l = |x: usize| {
        let mut s = 0.0;
        for y in Y as usize - 1..=Y as usize + 1 {
            let i = (y * W + x) * 4;
            s += 0.299 * f32::from(px[i])
                + 0.587 * f32::from(px[i + 1])
                + 0.114 * f32::from(px[i + 2]);
        }
        s / 3.0
    };
    // Desde 340, onde a tela do pen-down já é só papel: em 324..340 está a borda macia da mancha, e o pigmento ali
    // CLAREIA a mistura (o rosa da borda) antes do pincel sujo pousar no papel — conteúdo, não anel (medido 07/11:
    // 333 → 62,31,68 · 339 → 16,6,36 · daí monótona até ao branco).
    (340..W - 1).map(|x| (l(x) - l(x + 1)).max(0.0)).sum()
}

/// X3 (07/11): a cauda da mistura clareia monótona — sem os anéis do redondo nem as contas soltas do G-Pen (o Off
/// do mesmo pincel é o controlo).
#[test]
fn a_cauda_da_mistura_opaca_nao_tem_aneis_nem_contas() {
    for nome in [None, Some("G-Pen")] {
        let mede = |modo: ModoMisturaOpaca| {
            let mut t = com_a_mancha(nome);
            t.set_brush_mistura_modo(modo);
            t.set_brush_color_srgb8([0, 0, 220]);
            arrasta(&mut t, [60.0, Y], [340.0, Y]);
            let px = t.run_full().0;
            // Nem pontos pretos (o perfil da cor com a cobertura reconstruída do `u8`, 07/11): vermelho, azul e branco
            // não fazem preto.
            let pretos = px
                .chunks(4)
                .filter(|p| u32::from(p[0]) + u32::from(p[1]) + u32::from(p[2]) < 40)
                .count();
            assert_eq!(pretos, 0, "{nome:?} {modo:?}: {pretos} pontos pretos");
            descidas_na_cauda(&px)
        };
        let off = mede(ModoMisturaOpaca::Off);
        {
            let modo = ModoMisturaOpaca::WetBlend;
            let d = mede(modo);
            eprintln!("[cauda] {nome:?} Off {off:.1} · {modo:?} {d:.1}");
            assert!(
                d <= off + 3.0,
                "{nome:?} {modo:?}: a cauda tem vales (anéis/contas): {d:.1} vs Off {off:.1}"
            );
        }
    }
}

/// Sonda: com amount 1 a mistura é o azul puro — a diferença para o Off é só a COBERTURA (o tecto/pegada).
#[test]
#[ignore = "sonda"]
fn sonda_cobertura_amount_1() {
    for nome in [
        None,
        Some("Charcoal Stick"),
        Some("Sketch Pencil"),
        Some("Soft Airbrush"),
    ] {
        let mut v = Vec::new();
        for modo in [ModoMisturaOpaca::Off, ModoMisturaOpaca::WetBlend] {
            let mut t = com_a_mancha(nome);
            if std::env::var("SEM").as_deref() == Ok("dual") {
                t.set_ponta(ph2d_tool_painter::LinhaDaPonta::DualMode, 0.0);
            }
            if std::env::var("SEM").as_deref() == Ok("grao") {
                t.set_brush_texture_kind(0);
            }
            t.set_brush_mistura_modo(modo);
            t.set_brush_mistura_numero(0, 1.0);
            t.set_brush_mistura_numero(1, 1.0);
            t.set_brush_mistura_numero(2, 0.5);
            t.set_brush_color_srgb8([0, 0, 220]);
            arrasta(&mut t, [60.0, Y], [340.0, Y]);
            v.push(media(&mut t, 260));
        }
        eprintln!("[cob] {nome:?} Off {:?} · Blend a1 {:?}", v[0], v[1]);
    }
}

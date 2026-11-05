//! **O Smooth Edges é do TRAÇO** (dono 04/11: «por traço mesmo e cada pincel deve ter o seu. assim não perde detalhes»):
//! o valor pousa com a tinta. Cada portão compara cenas com a MESMA linha do tempo (a tinta é igual, só o valor muda),
//! por isso a igualdade ao byte diz «esta borda só lê o dela».

use super::*;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PanelEvent, PointerPhase};

const W: usize = 360;
const H: usize = 200;

fn tool(oleo: bool) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![240u8; W * H * 4], W as u32, H as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    t.set_wet_oleo_real_ligado(oleo);
    t.set_brush_size_px(14.0);
    t
}

fn traco(t: &mut PainterTool, a: [f32; 2], b: [f32; 2], valor: f32) {
    t.set_wet_smooth_edges(valor);
    let p = |k: f32, phase| CanvasPointer {
        pos: [a[0] + (b[0] - a[0]) * k, a[1] + (b[1] - a[1]) * k],
        pressure: 0.9,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(p(0.0, PointerPhase::Down));
    for i in 1..=24 {
        t.on_canvas_pointer(p(i as f32 / 24.0, PointerPhase::Move));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(p(1.0, PointerPhase::Up));
    t.on_tick(0.0);
}

/// Dois traços paralelos, o de cima a `y0` com `v0`, o de baixo a `y1` com `v1`; a tela recomposta inteira.
fn cena(oleo: bool, (y0, v0): (f32, f32), (y1, v1): (f32, f32)) -> PainterTool {
    let mut t = tool(oleo);
    traco(&mut t, [30.0, y0], [330.0, y0], v0);
    traco(&mut t, [30.0, y1], [330.0, y1], v1);
    t.wet_recomposite_full();
    t
}

fn faixa(t: &PainterTool, y0: usize, y1: usize) -> &[u8] {
    &t.canvas_rgba[y0 * W * 4..y1 * W * 4]
}

/// Quantos bytes diferem.
fn difs(a: &[u8], b: &[u8]) -> usize {
    a.iter().zip(b).filter(|(x, y)| x != y).count()
}

/// Portão 1 (e o 5, com o Óleo Real: a cor E o relevo): **dois traços com valores diferentes na mesma tela: cada borda
/// lê o seu**, ao byte do mesmo traço numa tela onde todos têm esse valor.
fn dois_tracos(oleo: bool) {
    let (cima, baixo) = (60.0, 140.0);
    let mista = cena(oleo, (cima, 0.5), (baixo, 0.12));
    let so_05 = cena(oleo, (cima, 0.5), (baixo, 0.5));
    let so_012 = cena(oleo, (cima, 0.12), (baixo, 0.12));
    let zero = cena(oleo, (cima, 0.0), (baixo, 0.0));
    let (d_cima, d_baixo) = (
        difs(faixa(&mista, 0, H / 2), faixa(&so_05, 0, H / 2)),
        difs(faixa(&mista, H / 2, H), faixa(&so_012, H / 2, H)),
    );
    let (efeito_cima, efeito_baixo) = (
        difs(faixa(&so_05, 0, H / 2), faixa(&zero, 0, H / 2)),
        difs(faixa(&so_012, H / 2, H), faixa(&zero, H / 2, H)),
    );
    eprintln!(
        "[suave por traço] óleo {oleo}: cima {d_cima} bytes, baixo {d_baixo}; o efeito 0,5 = {efeito_cima}, 0,12 = {efeito_baixo}"
    );
    assert!(
        efeito_cima > 100 && efeito_baixo > 100,
        "o Smooth Edges não mudou a tela: {efeito_cima} / {efeito_baixo}"
    );
    assert_eq!(d_cima, 0, "a borda a 0,5 não lê o seu");
    assert_eq!(d_baixo, 0, "a borda a 0,12 não lê o seu");
}

#[test]
fn dois_tracos_com_valores_diferentes_cada_borda_le_o_seu() {
    dois_tracos(false);
}

#[test]
fn com_o_oleo_o_relevo_acompanha_o_valor_de_cada_traco() {
    dois_tracos(true);
}

/// Portão 2: **pintar com 0 ao lado de um traço a 0,5 não muda um byte do 0,5**, longe e com 3 px de papel entre as
/// bordas; e o traço a 0 fica ao byte o de uma tela só a 0, o detalhe que se perdia com o 0,12 na tela inteira. Com
/// menos de 3 px (medido: 2 px, 24 bytes) as duas tintas já se vêem na mancha: é a orla delas, não a de um traço só.
#[test]
fn pintar_com_zero_ao_lado_nao_muda_um_byte_do_0_5() {
    for (nome, y1) in [("longe", 140.0f32), ("a 3 px de papel", 92.0)] {
        let y0 = 60.0f32;
        let meio = ((y0 + y1) * 0.5) as usize;
        let mista = cena(false, (y0, 0.5), (y1, 0.0));
        let so_05 = cena(false, (y0, 0.5), (y1, 0.5));
        let zero = cena(false, (y0, 0.0), (y1, 0.0));
        let d05 = difs(faixa(&mista, 0, meio), faixa(&so_05, 0, meio));
        let d0 = difs(faixa(&mista, meio, H), faixa(&zero, meio, H));
        eprintln!("[suave por traço] {nome}: o 0,5 mudou {d05} bytes; o 0 mudou {d0}");
        assert_eq!(d05, 0, "{nome}: o traço a 0 mudou o 0,5");
        assert_eq!(d0, 0, "{nome}: o traço a 0 não ficou o desligado");
    }
}

/// Portão 3: **mudar o valor corrente não recompõe o já pintado**, pela porta do painel e pela da ferramenta, com a tela
/// inteira recomposta depois.
#[test]
fn mudar_o_valor_nao_recompoe_o_ja_pintado() {
    for oleo in [false, true] {
        let mut t = cena(oleo, (60.0, 0.5), (140.0, 0.5));
        let antes = t.canvas_rgba.to_vec();
        assert!(t.route_brush_wetpaint_event(&PanelEvent::SetValue(
            crate::ids::PAINTER_WETPAINT_SMOOTH_EDGES,
            0.0
        )));
        for _ in 0..200 {
            t.on_tick(0.0);
        }
        assert_eq!(
            difs(&t.canvas_rgba, &antes),
            0,
            "óleo {oleo}: o painel mudou"
        );
        t.set_wet_smooth_edges(1.0);
        t.wet_recomposite_full();
        assert_eq!(
            difs(&t.canvas_rgba, &antes),
            0,
            "óleo {oleo}: a recomposição leu o valor de agora"
        );
    }
}

/// Portão 4: **desfazer devolve as células com o valor antigo** (o histórico da ferramenta): um traço a 0 por cima do
/// 0,5, desfeito, deixa o campo e a tela como estavam; e refazer a tinta a outro valor lê o novo.
#[test]
fn desfazer_devolve_as_celulas_com_o_valor_antigo() {
    let mut t = tool(false);
    traco(&mut t, [30.0, 100.0], [330.0, 100.0], 0.5);
    t.wet_recomposite_full();
    let tela = t.canvas_rgba.to_vec();
    let campo = |t: &PainterTool| {
        t.paint
            .wetpaint
            .session
            .as_ref()
            .expect("uma sessão viva")
            .engine
            .active_grid()
            .aparencia
            .iter()
            .map(|a| a.suave)
            .collect::<Vec<f32>>()
    };
    let v0 = campo(&t);
    assert!(v0.contains(&0.5), "a tinta não pousou o valor");
    traco(&mut t, [180.0, 40.0], [180.0, 160.0], 0.0);
    t.wet_recomposite_full();
    assert_ne!(campo(&t), v0, "a tinta por cima não misturou o valor");
    assert!(t.undo_last(), "não havia o que desfazer");
    t.on_tick(0.0);
    assert_eq!(campo(&t), v0, "o desfazer não devolveu o valor da célula");
    t.wet_recomposite_full();
    assert_eq!(
        difs(&t.canvas_rgba, &tela),
        0,
        "a tela desfeita não é a de antes"
    );
}

/// **Uma tinta toda a um valor é a lei escalar de antes, ao byte**: o halo no papel, a mancha, a orla. A referência é a
/// mesma tinta pousada a 0 com um só `Suave` desse raio (`raio_escalar`), no mesmo processo.
#[test]
fn um_valor_so_e_a_lei_de_um_raio_ao_byte() {
    for (valor, raio) in [(0.12f32, 0.12f32 * 6.0), (0.52, 0.52 * 6.0), (1.0, 6.0)] {
        let a = cena(false, (60.0, valor), (140.0, valor));
        let mut b = cena(false, (60.0, 0.0), (140.0, 0.0));
        super::smooth_edges::raio_escalar::set(Some(raio));
        b.wet_recomposite_full();
        super::smooth_edges::raio_escalar::set(None);
        let d = difs(&a.canvas_rgba, &b.canvas_rgba);
        eprintln!("[suave por traço] {valor}: {d} bytes contra um raio só");
        assert_eq!(d, 0, "o valor {valor} não é a lei de um raio");
    }
}

/// A mesma tinta por caixas e pela tela inteira: o byte de um pixel não depende da região nem dos outros valores dela.
#[test]
fn a_caixa_e_a_tela_inteira_dao_o_mesmo_byte_com_varios_valores() {
    let mut t = tool(false);
    traco(&mut t, [30.0, 60.0], [330.0, 60.0], 0.12);
    traco(&mut t, [30.0, 110.0], [330.0, 110.0], 0.7);
    traco(&mut t, [60.0, 20.0], [300.0, 180.0], 0.3);
    t.wet_recomposite_full();
    let inteira = t.canvas_rgba.to_vec();
    let sess = t.paint.wetpaint.session.as_mut().expect("uma sessão viva");
    sess.bring_home();
    for (x0, y0, x1, y1) in [(40, 50, 60, 70), (100, 100, 180, 120), (10, 10, 340, 30)] {
        sess.engine.merge_dirty(x0, y0, x1, y1);
    }
    composite_for_measure(&mut t);
    assert_eq!(
        difs(&t.canvas_rgba, &inteira),
        0,
        "recompor por caixa deu outro byte"
    );
}

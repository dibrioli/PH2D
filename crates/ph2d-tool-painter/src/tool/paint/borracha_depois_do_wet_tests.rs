//! **A Borracha depois do Wet Paint** (smoke do dono 05/11: «não apaga os traços dos outros modos já pintados»).

use super::*;
use ph2d_editor_core::tool::{PanelEvent, Tool};

const SIZE: u32 = 128;
const VELHO: (u32, u32, u32, u32) = (58, 71, 34, 47);
const VELHO_FORA: (u32, u32, u32, u32) = (30, 43, 34, 47);
const WET: (u32, u32, u32, u32) = (58, 71, 84, 97);

fn tinta_em(c: &[u8], (x0, x1, y0, y1): (u32, u32, u32, u32)) -> f64 {
    let mut s = 0.0;
    for y in y0..y1 {
        for x in x0..x1 {
            let i = ((y * SIZE + x) * 4) as usize;
            let luz = (f64::from(c[i]) + f64::from(c[i + 1]) + f64::from(c[i + 2])) / 765.0;
            s += f64::from(c[i + 3]) / 255.0 * (1.0 - luz);
        }
    }
    s
}

struct Mesa {
    t: PainterTool,
    tela: Option<Vec<u8>>,
}

impl Mesa {
    fn nova() -> Self {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (SIZE * SIZE * 4) as usize], SIZE, SIZE);
        let mut m = Self { t, tela: None };
        m.quadros(2);
        m
    }
    fn quadros(&mut self, n: usize) {
        for _ in 0..n {
            self.t.on_tick(1000.0 / 60.0);
            if let Some((px, _, _)) = self.t.take_preview_arc() {
                self.tela = Some(px.as_ref().clone());
            }
        }
    }
    fn meio(&mut self, m: PaintMedia) {
        self.t.handle_panel_event(PanelEvent::SelectOption(
            crate::ids::PAINTER_BRUSH_MEDIA,
            m.to_u8().to_string(),
        ));
        self.quadros(1);
    }
    fn trilho(&mut self, f: &str) {
        self.t.handle_panel_event(PanelEvent::SelectOption(
            ph2d_editor_core::ids::PAINTER_PAINT_MODE,
            f.to_string(),
        ));
        self.quadros(1);
    }
    fn risca(&mut self, de: [f32; 2], ate: [f32; 2], passos: usize) {
        self.t.on_canvas_pointer(cp(de, PointerPhase::Down));
        for k in 1..=passos {
            let f = k as f32 / passos as f32;
            self.t.on_canvas_pointer(cp(
                [de[0] + (ate[0] - de[0]) * f, de[1] + (ate[1] - de[1]) * f],
                PointerPhase::Move,
            ));
            self.quadros(1);
        }
        self.t.on_canvas_pointer(cp(ate, PointerPhase::Up));
        self.quadros(2);
    }
    fn pinta(&mut self, y: f32) {
        self.t.paint.brush.radius_px = 8.0;
        self.t.paint.brush.color = [0.05, 0.05, 0.1];
        self.risca([20.0, y], [108.0, y], 44);
    }
    fn apaga(&mut self) {
        self.t.paint.brush.radius_px = 10.0;
        for _ in 0..3 {
            self.risca([64.0, 20.0], [64.0, 108.0], 44);
        }
    }
    fn relata(&mut self, x: PaintMedia, fluxo: &str, antes: f64) {
        let ([cv, cf], [vl, wt]) = (
            [
                tinta_em(&self.t.canvas_rgba, VELHO),
                tinta_em(&self.t.canvas_rgba, VELHO_FORA),
            ],
            self.tela
                .as_ref()
                .map_or([-1.0, -1.0], |p| [tinta_em(p, VELHO), tinta_em(p, WET)]),
        );
        eprintln!(
            "BDW\t{x:?}\t{fluxo}\tmodo {:?} eraser {} menu {:?} sessão {}\tvelho antes {antes:.1}\tcanvas: velho {cv:.1} fora {cf:.1} wet {:.1}\ttela: velho {vl:.1} wet {wt:.1}",
            self.t.paint.paint_mode,
            self.t.paint.eraser,
            self.t.paint_media(),
            self.t.paint.wetpaint.session.is_some(),
            tinta_em(&self.t.canvas_rgba, WET),
        );
    }
}

/// Bug #70, decisão do dono: no Wet Paint a Borracha apaga tudo o que está sob o círculo, a tinta molhada e a antiga.
#[test]
fn a_borracha_no_wet_apaga_a_tinta_antiga_de_cada_meio() {
    for x in [
        PaintMedia::Digital,
        PaintMedia::Watercolor,
        PaintMedia::Impasto,
    ] {
        let mut m = Mesa::nova();
        m.meio(x);
        m.pinta(40.0);
        let velho = tinta_em(&m.t.canvas_rgba, VELHO);
        let fora = tinta_em(&m.t.canvas_rgba, VELHO_FORA);
        m.meio(PaintMedia::WetPaint);
        m.pinta(90.0);
        m.quadros(20);
        assert!(m.t.paint.wetpaint.session.is_some(), "{x:?}: sessão viva");
        m.trilho("eraser");
        m.t.paint.brush.radius_px = 10.0;
        m.risca([64.0, 20.0], [64.0, 108.0], 44);
        let uma = tinta_em(&m.t.canvas_rgba, VELHO);
        assert!(
            uma > velho * 0.4,
            "{x:?}: uma passada, {velho:.1} → {uma:.1}"
        );
        // Relato do dono 06/11: a antiga sai aos poucos, e várias passadas chegam ao papel.
        for _ in 0..4 {
            m.apaga();
        }
        assert!(
            m.t.paint.wetpaint.session.is_some(),
            "{x:?}: a água continua"
        );
        let (depois, fora2, wet) = (
            tinta_em(&m.t.canvas_rgba, VELHO),
            tinta_em(&m.t.canvas_rgba, VELHO_FORA),
            tinta_em(&m.t.canvas_rgba, WET),
        );
        assert!(
            depois < velho * 0.02,
            "{x:?}: a antiga {velho:.1} → {depois:.1}"
        );
        assert!(wet < 3.0, "{x:?}: a molhada ficou {wet:.1}");
        assert!(
            (fora2 - fora).abs() < 1e-9,
            "{x:?}: fora do círculo {fora:.1} → {fora2:.1}"
        );
    }
}

/// O Ctrl+Z devolve a tinta antiga com a água viva, e o refazer apaga-a outra vez.
#[test]
fn desfazer_a_borracha_no_wet_devolve_a_tinta_antiga() {
    let mut m = Mesa::nova();
    m.meio(PaintMedia::Digital);
    m.pinta(40.0);
    let velho = tinta_em(&m.t.canvas_rgba, VELHO);
    m.meio(PaintMedia::WetPaint);
    m.pinta(90.0);
    m.quadros(20);
    m.trilho("eraser");
    // Um traço só, para o Ctrl+Z desfazer a Borracha inteira.
    m.t.paint.brush.radius_px = 10.0;
    m.t.paint.brush.strength = 1.0;
    m.risca([64.0, 20.0], [64.0, 108.0], 44);
    let apagada = tinta_em(&m.t.canvas_rgba, VELHO);
    assert!(
        apagada < velho * 0.8,
        "a Borracha agiu: {velho:.1} → {apagada:.1}"
    );
    assert!(m.t.undo_last(), "desfez");
    m.quadros(2);
    assert!(m.t.paint.wetpaint.session.is_some(), "a água continua viva");
    let voltou = tinta_em(&m.t.canvas_rgba, VELHO);
    assert!(
        (voltou - velho).abs() < 1e-9,
        "voltou {voltou:.1} de {velho:.1}"
    );
    assert!(m.t.redo_last(), "refez");
    m.quadros(2);
    let de_novo = tinta_em(&m.t.canvas_rgba, VELHO);
    assert!(
        (de_novo - apagada).abs() < 1e-9,
        "refez {de_novo:.1} de {apagada:.1}"
    );
}

fn base_de(m: &Mesa) -> Vec<u8> {
    let s =
        m.t.paint
            .wetpaint
            .session
            .as_ref()
            .expect("a água continua");
    s.base.to_vec()
}

/// Recompõe a tela inteira a partir da base: é o que mostra uma base errada, que o journal do undo esconde.
fn recomposta(m: &mut Mesa) -> Vec<u8> {
    let s =
        m.t.paint
            .wetpaint
            .session
            .as_mut()
            .expect("a água continua");
    s.bring_home();
    s.engine.mark_dirty_full();
    m.t.wet_compoe_ou_adia();
    m.t.wet_compoe_o_pendente();
    m.t.canvas_rgba.to_vec()
}

fn regiao(c: &[u8], (x0, x1, y0, y1): (u32, u32, u32, u32)) -> Vec<u8> {
    (y0..y1)
        .flat_map(|y| {
            let i = ((y * SIZE + x0) * 4) as usize;
            c[i..i + ((x1 - x0) * 4) as usize].to_vec()
        })
        .collect()
}

/// O undo e o redo devolvem a própria base, passo a passo, com um traço de tinta entre duas Borrachas.
#[test]
fn desfazer_e_refazer_devolvem_a_base_de_cada_passo() {
    let mut m = Mesa::nova();
    m.meio(PaintMedia::Digital);
    m.pinta(40.0);
    m.meio(PaintMedia::WetPaint);
    m.pinta(90.0);
    m.quadros(20);
    let inicial = base_de(&m);
    let borracha = |m: &mut Mesa, x: f32| {
        m.trilho("eraser");
        m.t.paint.brush.radius_px = 10.0;
        m.t.paint.brush.strength = 1.0;
        m.risca([x, 20.0], [x, 60.0], 20);
    };
    borracha(&mut m, 64.0);
    let uma = base_de(&m);
    assert!(uma != inicial, "a 1.ª Borracha apagou a base");
    m.trilho("brush");
    m.pinta(70.0);
    assert!(base_de(&m) == uma, "o pincel não mexe na base");
    borracha(&mut m, 36.0);
    let duas = base_de(&m);
    assert!(duas != uma, "a 2.ª Borracha apagou a base");

    // A tinta antiga na tela recomposta é a da base: nem o que o journal devolveu a tapa.
    let confere = |m: &mut Mesa, esperada: &[u8], nome: &str| {
        assert!(base_de(m) == esperada, "{nome}: a base");
        let tela = recomposta(m);
        for r in [VELHO, VELHO_FORA] {
            assert!(
                regiao(&tela, r) == regiao(esperada, r),
                "{nome}: a tela recomposta em {r:?}"
            );
        }
    };
    assert!(m.t.undo_last(), "desfez a 2.ª Borracha");
    confere(&mut m, &uma, "desfez a 2.ª");
    assert!(m.t.undo_last(), "desfez o traço");
    confere(&mut m, &uma, "desfez o traço");
    assert!(m.t.undo_last(), "desfez a 1.ª Borracha");
    confere(&mut m, &inicial, "desfez a 1.ª");
    assert!(m.t.redo_last(), "refez a 1.ª Borracha");
    confere(&mut m, &uma, "refez a 1.ª");
    assert!(m.t.redo_last(), "refez o traço");
    confere(&mut m, &uma, "refez o traço");
    assert!(m.t.redo_last(), "refez a 2.ª Borracha");
    confere(&mut m, &duas, "refez a 2.ª");
}

/// O ecrã, e não só a tela: a tinta antiga sem água por cima some do que `take_preview_arc` entrega, à razão de grade e à
/// velocidade do traço que forem, e a orla inteira entra na composição.
#[test]
fn o_ecra_mostra_a_tinta_antiga_apagada_onde_nao_ha_agua() {
    for (razao, passos) in [(1, 16), (1, 2), (2, 16), (2, 2), (4, 16), (4, 2)] {
        let nome = format!("razão {razao}, {passos} passos");
        let mut m = Mesa::nova();
        m.meio(PaintMedia::Digital);
        m.pinta(40.0);
        m.t.paint.wetpaint.grid_ratio = razao;
        m.meio(PaintMedia::WetPaint);
        m.pinta(90.0);
        m.quadros(20);
        let velho = tinta_em(m.tela.as_ref().expect("ecrã"), VELHO);
        assert!(velho > 10.0, "{nome}: a tinta antiga no ecrã: {velho:.1}");
        m.trilho("eraser");
        m.t.paint.brush.radius_px = 10.0;
        m.t.paint.brush.strength = 1.0;
        m.tela = None;
        m.risca([64.0, 25.0], [64.0, 56.0], passos);
        let ecra = m.tela.clone().expect("o ecrã mudou");
        let depois = tinta_em(&ecra, VELHO);
        assert!(
            depois < velho * 0.8,
            "{nome}: ecrã {velho:.1} → {depois:.1}"
        );
        assert!(
            tinta_em(&ecra, VELHO_FORA) > 10.0,
            "{nome}: fora do círculo"
        );
        let cheia = recomposta(&mut m);
        let diferem = ecra
            .as_chunks::<4>()
            .0
            .iter()
            .zip(cheia.as_chunks::<4>().0)
            .filter(|(a, b)| a != b)
            .count();
        assert_eq!(diferem, 0, "{nome}: píxeis por recompor no ecrã");
    }
}

/// A base com este alfa, e `n` cliques da Borracha no mesmo sítio: os alfas que ficam.
fn cliques_sobre_alfa(alfa: u8, n: usize) -> Vec<u8> {
    let mut m = Mesa::nova();
    m.meio(PaintMedia::WetPaint);
    m.pinta(90.0);
    m.quadros(2);
    {
        let s = m.t.paint.wetpaint.session.as_mut().expect("sessão viva");
        for px in std::sync::Arc::make_mut(&mut s.base).as_chunks_mut::<4>().0 {
            px[3] = alfa;
        }
    }
    m.trilho("eraser");
    m.t.paint.brush.radius_px = 10.0;
    m.t.paint.brush.strength = 1.0;
    m.t.paint.brush.hardness = 0.1;
    for _ in 0..n {
        m.risca([64.0, 40.0], [64.0, 40.0], 1);
    }
    base_de(&m)
        .as_chunks::<4>()
        .0
        .iter()
        .map(|p| p[3])
        .collect()
}

/// Alfa baixo sob cobertura macia chega a 0: arredondar deixava-o em 1, com o filtro `> 0` a pedir trabalho para sempre.
#[test]
fn a_borracha_fraca_leva_o_alfa_baixo_a_zero() {
    // O `k` de cada píxel, medido sobre alfa cheio. A orla com `k` em (0,5; 1) é o caso.
    let k = cliques_sobre_alfa(255, 1);
    let orla = k.iter().filter(|&&a| a > 128 && a < 255).count();
    assert!(orla > 10, "a prova precisa de orla macia: {orla} píxeis");
    let baixo = cliques_sobre_alfa(1, 1);
    for (i, (&ka, &a)) in k.iter().zip(&baixo).enumerate() {
        match ka {
            255 => assert_eq!(a, 1, "píxel {i}: fora da borracha"),
            _ => assert_eq!(a, 0, "píxel {i}: k {ka}/255"),
        }
    }
}

/// A selecção protege a tinta antiga como protege a molhada.
#[test]
fn a_selecao_protege_a_tinta_antiga_da_borracha() {
    let mut m = Mesa::nova();
    m.meio(PaintMedia::Digital);
    m.pinta(40.0);
    let velho = tinta_em(&m.t.canvas_rgba, VELHO);
    m.meio(PaintMedia::WetPaint);
    m.pinta(90.0);
    m.quadros(20);
    m.t.set_rect_selection(0, 60, 128, 68);
    m.trilho("eraser");
    m.apaga();
    let depois = tinta_em(&m.t.canvas_rgba, VELHO);
    assert!((depois - velho).abs() < 1e-9, "{velho:.1} → {depois:.1}");
}

/// O corpo (altura e cobertura) da caixa `VELHO`, somado sobre as camadas.
fn corpo_em(t: &PainterTool) -> (f64, f64) {
    let (x0, x1, y0, y1) = VELHO;
    let caixa = || (y0..y1).flat_map(move |y| (x0..x1).map(move |x| (y * SIZE + x) as usize));
    let h = t
        .heights
        .values()
        .map(|p| caixa().map(|i| f64::from(p[i])).sum::<f64>());
    let c = t
        .covers
        .values()
        .map(|p| caixa().map(|i| f64::from(p[i])).sum::<f64>());
    (h.sum(), c.sum())
}

/// B06.1 (BUGS #70 «ABERTO novo»): a Borracha do trilho tem `impasto` off e o relevo ficava, com ou
/// sem Wet (105,97 → 105,97 sob tinta 0,0). O relevo segue a CAMADA, não o pincel.
#[test]
fn a_borracha_do_trilho_apaga_o_corpo_do_impasto() {
    corpo_sob_a_borracha(Wet::Sem, 1);
}

/// B06.1 no Wet: a Borracha apaga a `base` (viva) ou a tela (morta) pela lei gradual, e o relevo segue o alfa
/// (`relevo_da_borracha`). Red visto: altura 105,97 → 105,97 e cobertura 42 760 → 42 760 sob a tinta apagada.
#[test]
fn a_borracha_do_wet_apaga_o_corpo_do_impasto() {
    corpo_sob_a_borracha(Wet::Viva, 5);
}

/// O mesmo na tinta morta (Wet sem sessão, `borracha_morta`).
#[test]
fn a_borracha_da_tinta_morta_apaga_o_corpo_do_impasto() {
    corpo_sob_a_borracha(Wet::Morta, 5);
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Wet {
    Sem,
    Viva,
    Morta,
}

fn corpo_sob_a_borracha(wet: Wet, vezes: usize) {
    {
        let mut m = Mesa::nova();
        m.meio(PaintMedia::Impasto);
        m.pinta(40.0);
        let (h0, c0) = corpo_em(&m.t);
        assert!(h0 > 10.0 && c0 > 100.0, "fixture: corpo ({h0:.2}, {c0:.0})");
        if wet != Wet::Sem {
            m.meio(PaintMedia::WetPaint);
        }
        if wet == Wet::Viva {
            m.pinta(90.0);
            m.quadros(20);
        }
        m.trilho("eraser");
        if wet == Wet::Morta {
            assert!(
                m.t.borracha_morta_assume(),
                "fixture: a Borracha da tinta morta"
            );
        }
        let com_wet = wet;
        assert!(
            !m.t.paint.brush.impasto,
            "fixture: a Borracha do trilho sem Impasto"
        );
        for _ in 0..vezes {
            m.apaga();
        }
        let tinta = tinta_em(&m.t.canvas_rgba, VELHO);
        assert!(
            tinta < 1.0,
            "Wet {com_wet:?}: CONTROLO, a cor saiu ({tinta:.1})"
        );
        let (h1, c1) = corpo_em(&m.t);
        assert!(
            h1 <= h0 * 0.01 && c1 <= c0 * 0.01,
            "Wet {com_wet:?}: a tinta saiu e o corpo ficou: altura {h0:.2} → {h1:.2}, cobertura {c0:.0} → {c1:.0}"
        );
    }
}

/// Os planos de relevo da camada activa, inteiros.
fn relevo_de(t: &PainterTool) -> (Vec<f32>, Vec<u8>) {
    let l = t.layers.active().expect("camada activa");
    (
        t.heights.get(&l).map(|p| p.to_vec()).unwrap_or_default(),
        t.covers.get(&l).map(|p| p.to_vec()).unwrap_or_default(),
    )
}

/// O Ctrl+Z do passo vivo (e da tinta morta) devolve o relevo que a Borracha do Wet levou, e o refazer leva-o outra vez.
#[test]
fn desfazer_a_borracha_do_wet_devolve_o_corpo() {
    for wet in [Wet::Viva, Wet::Morta] {
        let mut m = Mesa::nova();
        m.meio(PaintMedia::Impasto);
        m.pinta(40.0);
        m.meio(PaintMedia::WetPaint);
        if wet == Wet::Viva {
            m.pinta(90.0);
            m.quadros(20);
        }
        let (antes, h0) = (relevo_de(&m.t), corpo_em(&m.t).0);
        m.trilho("eraser");
        m.t.paint.brush.radius_px = 10.0;
        m.t.paint.brush.strength = 1.0;
        m.risca([64.0, 20.0], [64.0, 108.0], 44);
        let (apagado, h1) = (relevo_de(&m.t), corpo_em(&m.t).0);
        assert!(
            h1 < h0 * 0.9,
            "{wet:?}: a Borracha levou o corpo ({h0:.2} → {h1:.2})"
        );
        assert!(m.t.undo_last(), "{wet:?}: desfez");
        m.quadros(2);
        assert!(
            relevo_de(&m.t) == antes,
            "{wet:?}: o Ctrl+Z devolve o corpo"
        );
        assert!(m.t.redo_last(), "{wet:?}: refez");
        m.quadros(2);
        assert!(
            relevo_de(&m.t) == apagado,
            "{wet:?}: o refazer leva-o outra vez"
        );
    }
}

#[test]
#[ignore = "sonda: a Borracha sobre os traços dos outros meios, depois do Wet Paint"]
fn sonda_borracha_depois_do_wet() {
    for x in [
        PaintMedia::Digital,
        PaintMedia::Watercolor,
        PaintMedia::Impasto,
    ] {
        let inicio = |com_wet: bool| {
            let mut m = Mesa::nova();
            m.meio(x);
            m.pinta(40.0);
            let antes = tinta_em(&m.t.canvas_rgba, VELHO);
            if com_wet {
                m.meio(PaintMedia::WetPaint);
                m.pinta(90.0);
                m.quadros(20);
            }
            (m, antes)
        };
        // O controlo: sem Wet nenhum.
        let (mut m, a) = inicio(false);
        m.trilho("eraser");
        m.apaga();
        m.relata(x, "controlo sem Wet", a);
        // X → Wet → X → Borracha.
        let (mut m, a) = inicio(true);
        m.meio(x);
        m.trilho("eraser");
        m.apaga();
        m.relata(x, "X Wet X Borracha", a);
        // X → Wet → Borracha → meio X (o menu da Borracha).
        let (mut m, a) = inicio(true);
        m.trilho("eraser");
        m.meio(x);
        m.apaga();
        m.relata(x, "X Wet Borracha X", a);
        // X → Wet → Borracha, sem tocar no menu.
        let (mut m, a) = inicio(true);
        let viva = tinta_em(&m.t.canvas_rgba, WET);
        m.trilho("eraser");
        m.apaga();
        m.relata(
            x,
            &format!("viva (wet antes {viva:.1}) Borracha(menu Wet)"),
            a,
        );
        // X → Wet → Borracha(Wet) → pincel → X → Borracha.
        let (mut m, a) = inicio(true);
        m.trilho("eraser");
        m.quadros(2);
        m.trilho("brush");
        m.meio(x);
        m.trilho("eraser");
        m.apaga();
        m.relata(x, "X Wet Borracha pincel X Borracha", a);
        // X → Wet → X → pinta outro traço em X → Borracha.
        let (mut m, a) = inicio(true);
        m.meio(x);
        m.pinta(66.0);
        m.trilho("eraser");
        m.apaga();
        m.relata(x, "X Wet X pinta Borracha", a);
        // A Borracha no menu Wet ANTES de pintar no Wet: sem sessão, cai na borracha normal.
        let (mut m, a) = inicio(false);
        m.meio(PaintMedia::WetPaint);
        m.trilho("eraser");
        m.apaga();
        m.relata(x, "X Borracha(menu Wet) sem sessão", a);
        // O mesmo, depois de a tinta Wet assentar.
        let (mut m, a) = inicio(true);
        for _ in 0..140 {
            m.t.paint_tick(0.5);
        }
        m.quadros(4);
        let assentada = tinta_em(&m.t.canvas_rgba, WET);
        m.trilho("eraser");
        m.apaga();
        m.relata(
            x,
            &format!("assentada (wet antes {assentada:.1}) Borracha(menu Wet)"),
            a,
        );
        // Outra ferramenta encerra a sessão; o menu continua Wet e a Borracha volta a apagar tudo.
        let (mut m, a) = inicio(true);
        m.trilho("smear");
        m.trilho("eraser");
        m.apaga();
        m.relata(x, "X Wet Smear Borracha(menu Wet)", a);
    }
}

/// B06.6: o botão direito do Grid Stamp apaga pela porta `stroke_erases`, e o relevo perguntava só
/// `paint.eraser`: com o filme do Paper, o gesto de apagar POUSAVA corpo onde tirava a cor.
#[test]
fn o_grid_stamp_a_apagar_tira_o_corpo_em_vez_de_o_pousar() {
    use super::super::GridAxis;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (SIZE * SIZE * 4) as usize], SIZE, SIZE);
    t.set_brush_stroke_method(ph2d_painter_brush::StrokeMethod::GridStamp.to_u8());
    t.set_grid_cell_px(GridAxis::X, 32.0);
    t.set_grid_cell_px(GridAxis::Y, 32.0);
    t.paint.brush.color = [0.0, 0.0, 0.0];
    t.paint.brush.film_depth = 1.0;
    let toque = |t: &mut PainterTool| {
        t.on_canvas_pointer(cp([48.0, 48.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([48.0, 48.0], PointerPhase::Up));
    };
    let corpo = |t: &PainterTool| -> (f64, f64) {
        let h = t
            .heights
            .values()
            .flat_map(|p| p.iter())
            .map(|&v| f64::from(v));
        let c = t
            .covers
            .values()
            .flat_map(|p| p.iter())
            .map(|&v| f64::from(v));
        (h.sum(), c.sum())
    };
    toque(&mut t);
    let (h0, c0) = corpo(&t);
    assert!(
        c0 > 100.0,
        "fixture: o filme pousou corpo ({h0:.2}, {c0:.0})"
    );
    let antes: Vec<f32> = t.heights.values().next().expect("plano").to_vec();
    let alfa: Vec<u8> = t.canvas_rgba.iter().skip(3).step_by(4).copied().collect();
    t.set_grid_stamp_erase(true);
    toque(&mut t);
    t.set_grid_stamp_erase(false);
    let (h1, c1) = corpo(&t);
    // O suporte: o corpo não fica em proporção maior que a cor, píxel a píxel somado.
    let depois = t.heights.values().next().expect("plano");
    let soma = |f: &dyn Fn(usize) -> f64| {
        (0..antes.len())
            .filter(|&i| antes[i] > 0.0)
            .map(f)
            .sum::<f64>()
    };
    let kh = soma(&|i| f64::from(depois[i])) / soma(&|i| f64::from(antes[i]));
    let ka = soma(&|i| f64::from(t.canvas_rgba[i * 4 + 3])) / soma(&|i| f64::from(alfa[i]));
    assert!(ka < 0.9, "CONTROLO: o gesto apagou a cor (fica {ka:.3})");
    assert!(
        h1 < h0 && c1 < c0 && kh <= ka + 0.05,
        "o corpo ficou mais que a cor: cor fica {ka:.3}, altura fica {kh:.3} ({h0:.2} → {h1:.2}), \
         cobertura {c0:.0} → {c1:.0}"
    );
}

/// B06.4: a máscara é um canal cinzento sem corpo; o slot do Mask nasce com o Plow 1,0 da faca, e o
/// Smear da máscara arrastava o relevo REAL da camada enquanto editava só o rascunho dela.
#[test]
fn o_esfregao_na_mascara_nao_toca_o_corpo() {
    let mut m = Mesa::nova();
    m.meio(PaintMedia::Impasto);
    m.pinta(40.0);
    let antes: Vec<Vec<f32>> = m.t.heights.values().map(|p| p.to_vec()).collect();
    assert!(
        antes.iter().flatten().any(|&h| h > 0.0),
        "fixture: há corpo"
    );
    m.trilho("mask");
    m.t.set_mask_brush(3);
    assert_eq!(m.t.mask_brush(), 3, "fixture: o Smear da máscara");
    m.t.paint.brush.radius_px = 12.0;
    m.risca([64.0, 20.0], [64.0, 70.0], 30);
    let depois: Vec<Vec<f32>> = m.t.heights.values().map(|p| p.to_vec()).collect();
    let mudou = antes
        .iter()
        .flatten()
        .zip(depois.iter().flatten())
        .filter(|(a, b)| a != b)
        .count();
    assert_eq!(
        mudou, 0,
        "o Smear da máscara moveu {mudou} texels do relevo da camada"
    );
}

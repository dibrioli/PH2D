//! **O material do PINCEL fica na tinta** (auditoria W3b C3 · C4 · U2 · U4, `docs/Painter/49_realshader/AUDITORIA_W3b.md`):
//! a Transmission, a Transparency e o Paper Relief são só do pincel (saíram do cartão Real Oil, dono 27/10), a tinta
//! pousada guarda-os, a orla virgem veste-os, mordem no estado de fábrica e o preview mostra-os antes de pintar.

use super::*;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};
use ph2d_wet_paint::material::MaterialCelula;
use ph2d_wet_paint::tuning::{KNOB_DEFS, Knob, KnobGroup};

const W: usize = 200;
const H: usize = 150;

fn pinta(t: &mut PainterTool, y: f32) {
    let p = |x: f32, phase| CanvasPointer {
        pos: [x, y],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(p(30.0, PointerPhase::Down));
    for k in 1..=40 {
        t.on_canvas_pointer(p(30.0 + 3.5 * k as f32, PointerPhase::Move));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(p(170.0, PointerPhase::Up));
    for _ in 0..20 {
        t.on_tick(1000.0 / 60.0);
    }
}

/// A tela de FÁBRICA (o artista acabou de abrir: sem papel escolhido, Relief do papel 0) com o Óleo Real ligado e o
/// pincel com os knobs `k`; `papel` = o Cold com Relief 0,6 (a cena das calibrações).
fn tela(k: &[(Knob, f64)], papel: bool) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    t.set_brush_size_px(14.0);
    if papel {
        t.set_brush_paper_kind(ph2d_painter_brush::TextureKind::PaperCold.to_u8());
        t.set_substrate_depth(0.6);
    }
    t.paint.wetpaint.knobs.set(Knob::PigmentPerDab, 3000.0);
    t.set_wet_oleo_real_ligado(true);
    for &(kn, v) in k {
        t.paint.wetpaint.knobs.set(kn, v);
    }
    t.set_brush_color_srgb8([30, 60, 200]);
    pinta(&mut t, 60.0);
    pinta(&mut t, 72.0);
    t
}

fn recompoe(t: &mut PainterTool) -> Vec<u8> {
    t.wet_recomposite_full();
    let mut px = t.canvas_rgba.to_vec();
    let (w, h) = (W as u32, H as u32);
    t.apply_impasto_light(&mut px, crate::Region { x: 0, y: 0, w, h });
    px
}

const PESOS: [(Knob, f64); 3] = [
    (Knob::Transmission, 0.8),
    (Knob::Transparency, 0.4),
    (Knob::PaperRelief, 1.0),
];

/// ⭐ **C3: a tinta pousada GUARDA o material do pincel** — pintada com a Transmission, a Transparency e o Paper Relief
/// ≠ 0 (só no pincel), o pincel volta a 0 e a tela recompõe-se com o Óleo Real e o papel: nem um byte muda. O CONTROLO:
/// a mesma tela pintada com o pincel a 0 é outra (os três morderam — o gate do painel pintava com o material NEUTRO).
#[test]
fn o_material_do_pincel_fica_na_tinta_pousada() {
    let mut t = tela(&PESOS, true);
    let pintada = recompoe(&mut t);
    let neutra = recompoe(&mut tela(&[], true));
    let difs = pintada.iter().zip(&neutra).filter(|(a, b)| a != b).count();
    assert!(difs > 1000, "controlo: os três morderam ({difs} bytes)");
    // E cada um sozinho (um peso morto — a tinta que não o lê — passava escondido atrás dos outros).
    for p in PESOS {
        let um = recompoe(&mut tela(&[p], true));
        let d = um.iter().zip(&neutra).filter(|(a, b)| a != b).count();
        assert!(d > 1000, "controlo: {:?} sozinho mordeu ({d} bytes)", p.0);
    }
    for (kn, _) in PESOS {
        t.paint.wetpaint.knobs.set(kn, 0.0);
    }
    let depois = recompoe(&mut t);
    let mudou = pintada.iter().zip(&depois).filter(|(a, b)| a != b).count();
    assert_eq!(
        mudou, 0,
        "o pincel a 0 mudou {mudou} bytes da tinta pousada"
    );
}

/// ⭐ **O MODO da Transmission fica na tinta pousada** (dono 28/10: o seletor Blender · Clear glass é do pincel): pintada
/// com Transmission 0,8 em Clear glass, o seletor do pincel passa a Blender e a tela recompõe-se — nem um byte muda.
/// O CONTROLO: a mesma tela pintada em Blender é outra (o modo mordeu a tinta).
#[test]
fn o_modo_da_transmissao_fica_na_tinta_pousada() {
    let tr = (Knob::Transmission, 0.8);
    let mut t = tela(&[tr, (Knob::TransmissionMode, 1.0)], true);
    let vidro = recompoe(&mut t);
    let blender = recompoe(&mut tela(&[tr], true));
    let difs = vidro.iter().zip(&blender).filter(|(a, b)| a != b).count();
    assert!(difs > 1000, "controlo: o modo mordeu ({difs} bytes)");
    t.paint.wetpaint.knobs.set(Knob::TransmissionMode, 0.0);
    let depois = recompoe(&mut t);
    let mudou = vidro.iter().zip(&depois).filter(|(a, b)| a != b).count();
    assert_eq!(
        mudou, 0,
        "o seletor do pincel mudou {mudou} bytes da tinta pousada"
    );
}

/// ⭐ **C4: o material veste a orla VIRGEM** (o par de `aparencia::veste_virgens`): pintado com a Transmission, o censo
/// das células com tinta (`sett + susp > 0`) e o material NEUTRO é ~0 — sem a orla, o pigmento que a água leva a uma
/// célula virgem mostrava a tinta opaca de sempre (um aro).
#[test]
fn o_material_veste_a_orla_virgem() {
    let mut t = tela(&[(Knob::Transmission, 0.8)], false);
    let s = t.paint.wetpaint.session.as_mut().expect("a sessão");
    s.bring_home();
    let g = &s.engine.layers[0].grid;
    let com_tinta: Vec<usize> = (0..g.sett.len())
        .filter(|&i| g.sett[i] + g.susp[i] > 0.0)
        .collect();
    let neutras = com_tinta
        .iter()
        .filter(|&&i| g.material_da_celula(i) == MaterialCelula::NEUTRO)
        .count();
    eprintln!(
        "ORLA: {neutras} das {} células com tinta têm o material NEUTRO",
        com_tinta.len()
    );
    assert!(com_tinta.len() > 1000, "controlo: há tinta");
    assert!(
        neutras * 1000 <= com_tinta.len(),
        "{neutras} das {} células com tinta ficaram sem o material do pincel",
        com_tinta.len()
    );
}

/// O DENTE entre duas telas nas células pintadas: o desvio-padrão da diferença de luminância.
fn dente(a: &[u8], b: &[u8]) -> f64 {
    let l =
        |p: &[u8]| 0.2126 * f64::from(p[0]) + 0.7152 * f64::from(p[1]) + 0.0722 * f64::from(p[2]);
    let d: Vec<f64> = a
        .chunks(4)
        .zip(b.chunks(4))
        .filter(|(p, _)| p[..3] != [255, 255, 255])
        .map(|(p, q)| l(p) - l(q))
        .collect();
    let mu = d.iter().sum::<f64>() / d.len() as f64;
    (d.iter().map(|x| (x - mu).powi(2)).sum::<f64>() / d.len() as f64).sqrt()
}

/// ⭐ **U2: o Paper Relief e o grão do vidro mordem no estado de FÁBRICA** (papel não escolhido, Relief do papel 0):
/// o Paper Relief 1 mexe a tinta pelo dente do papel (a textura com o piso `RELIEF_NA_TINTA_MIN`) e a Transmission 1
/// mostra o grão assado — cada um ≥ metade do que mostram na cena calibrada (Cold, Relief 0,6). Antes: 0 nos dois.
#[test]
fn o_paper_relief_e_o_grao_do_vidro_mordem_na_fabrica() {
    let medir = |papel: bool| {
        let rp = dente(
            &recompoe(&mut tela(&[(Knob::PaperRelief, 1.0)], papel)),
            &recompoe(&mut tela(&[], papel)),
        );
        let mut t = tela(&[(Knob::Transmission, 1.0)], papel);
        let com = recompoe(&mut t);
        super::grao_no_filme::sem_grao::set(true);
        let sem = recompoe(&mut t);
        super::grao_no_filme::sem_grao::set(false);
        (rp, dente(&com, &sem))
    };
    let (rp0, gr0) = medir(false);
    let (rp6, gr6) = medir(true);
    eprintln!(
        "FÁBRICA: Paper Relief {rp0:.2} (Cold 0,6: {rp6:.2}) · grão do vidro {gr0:.2} (Cold 0,6: {gr6:.2})"
    );
    assert!(
        rp6 > 0.3 && gr6 > 0.3,
        "controlo: a cena calibrada mostra o grão"
    );
    assert!(rp0 >= 0.5 * rp6, "o Paper Relief na fábrica: {rp0} × {rp6}");
    assert!(
        gr0 >= 0.5 * gr6,
        "o grão do vidro na fábrica: {gr0} × {gr6}"
    );
}

/// ⭐ **U4: cada knob do Material do pincel muda o PREVIEW** (o retorno antes de pintar) — derivado da tabela (todo
/// knob do grupo Material; um knob novo entra sozinho), com o Óleo Real ligado e uma base onde cada um morde (os pesos
/// a meio: o Coat Roughness só se vê com Coat, o raio só com Subsurface). O «Lock material» não muda um traço sobre o
/// papel limpo: muda a TELA (um traço por cima de tinta com outro material). Imprime o custo do preview.
#[test]
fn cada_knob_do_material_muda_o_preview() {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; 64 * 64 * 4], 64, 64);
    t.set_paint_media(crate::PaintMedia::WetPaint);
    t.set_brush_size_px(14.0);
    t.set_brush_color_srgb8([30, 60, 200]);
    t.paint.wetpaint.knobs.set(Knob::PigmentPerDab, 3000.0);
    t.set_wet_oleo_real_ligado(true);
    let defs: Vec<_> = KNOB_DEFS
        .iter()
        .filter(|d| d.group == KnobGroup::Material && d.knob != Knob::LockMaterial)
        .collect();
    for d in &defs {
        t.paint
            .wetpaint
            .knobs
            .set(d.knob, d.min + 0.5 * (d.max - d.min));
    }
    let preview = |t: &PainterTool| {
        let mut ultimo = Vec::new();
        let t0 = std::time::Instant::now();
        assert!(crate::tool::paint::wetpaint_preset::pintar_preview(
            &t.wet_preview_cena(),
            &|| false,
            &mut |px, _, _| ultimo = px.to_vec()
        ));
        (ultimo, t0.elapsed())
    };
    let (base, custo_oleo) = preview(&t);
    let mut mortos = Vec::new();
    for d in &defs {
        let v = t.paint.wetpaint.knobs.get(d.knob);
        let chave = t.wet_preview_chave();
        t.paint
            .wetpaint
            .knobs
            .set(d.knob, if v > d.min { d.min } else { d.max });
        assert_ne!(chave, t.wet_preview_chave(), "{}: a chave não viu", d.key);
        if preview(&t).0 == base {
            mortos.push(d.key);
        }
        t.paint.wetpaint.knobs.set(d.knob, v);
    }
    t.set_wet_oleo_real_ligado(false);
    let (_, custo_sem) = preview(&t);
    eprintln!(
        "PREVIEW: {} knobs do Material; custo com o Óleo Real {custo_oleo:?} · sem {custo_sem:?}",
        defs.len()
    );
    assert!(
        defs.len() >= 11,
        "controlo: a tabela do Material ({})",
        defs.len()
    );
    assert!(
        mortos.is_empty(),
        "knobs do Material sem retorno no preview: {mortos:?}"
    );
}

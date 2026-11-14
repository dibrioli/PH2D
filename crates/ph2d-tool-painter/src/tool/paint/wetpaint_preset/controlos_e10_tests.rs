//! E10-B1 — a migração dos presets para os controlos da E10 por FUNÇÃO (`F3_controlos.md` §3, §9) e a Dry speed
//! da tela. Números: `docs/Painter/Arquitetura/E10b1_ferramenta_e_presets.md`.

use super::controlos_e10::{
    BODY_DO_LEVELING, CHAVE_CONTROLOS, familia_do_preset, meio_da_familia, migra_a_receita,
    migra_para_controlos,
};
use super::*;
use ph2d_wet_paint::painter::receita::{Controlos, H_REF, K_W, receita, veiculo};

const TOL: f64 = 1e-6;
const MORTOS: [&str; 5] = [
    "extRakeLight",
    "extValleyGran",
    "extWetSheen",
    "extGrainDither",
    "extEdgeTint",
];

fn biblioteca() -> ph2d_brush_presets::Biblioteca {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    ph2d_brush_presets::Biblioteca::carregar(&raiz, None)
}

fn wet() -> PainterTool {
    let mut t = PainterTool::default();
    t.set_paint_media(PaintMedia::WetPaint);
    t
}

/// O preset `nome` da fábrica, com a pasta da família (`library_folder`) como a biblioteca a diz.
fn de_fabrica(nome: &str) -> Preset {
    let bib = biblioteca();
    let e = bib
        .entradas
        .iter()
        .find(|e| e.preset.parametros.name == nome)
        .unwrap_or_else(|| panic!("{nome} na fábrica"));
    let mut p = e.preset.clone();
    p.parametros.library_folder = Some(e.partes()[1].to_owned());
    p
}

fn rel(a: f64, b: f64) -> f64 {
    if b == 0.0 {
        a.abs()
    } else {
        (a / b - 1.0).abs()
    }
}

const CONTROLOS: [Knob; 4] = [Knob::Dilution, Knob::Volume, Knob::Medium, Knob::Body];

#[test]
fn cada_preset_de_fabrica_mapeia_por_funcao() {
    let bib = biblioteca();
    let (mut n, mut n_tinta) = (0, 0);
    let mut por_meio = std::collections::BTreeMap::<String, usize>::new();
    let (mut falhas, mut presos) = (Vec::new(), Vec::new());
    for e in bib
        .entradas
        .iter()
        .filter(|e| e.preset.parametros.media == MEIO_DO_PRESET)
    {
        n += 1;
        let familia = e.partes()[1];
        let mut prm = e.preset.parametros.clone();
        // A família pelos grupos de fábrica (quando a ponte não diz a pasta) dá o mesmo meio que a pasta.
        assert_eq!(
            meio_da_familia(familia_do_preset(&prm), &prm.group),
            meio_da_familia(familia, &prm.group),
            "{}: o meio pelo grupo ≠ pela pasta",
            prm.name
        );
        prm.library_folder = Some(familia.to_owned());

        // A referência da E4, sobre os números crus do ficheiro (a Water e a Opacity de agora = as de fábrica).
        let d = WetKnobs::DEFAULT;
        let lei = |kn: Knob| {
            let def = &KNOB_DEFS[kn as usize];
            prm.engine
                .get(def.key)
                .copied()
                .unwrap_or(d.get(kn))
                .clamp(def.min, def.max)
        };
        let (ppd, wpd) = (lei(Knob::PigmentPerDab), lei(Knob::WaterPerDab));
        let agua = prm.volumes.water.unwrap_or(d.water).clamp(0.0, 1.0);
        let op = prm
            .volumes
            .opacity
            .unwrap_or(f64::from(OPACIDADE_DE_FABRICA));
        let (p4, carga) = ph2d_wet_paint::painter::receita::migra_da_lei_velha(ppd, wpd, agua, op);
        let phi_e4 = p4 / (p4 + K_W * veiculo(agua, wpd));
        let h_e4 = op * carga * H_REF;

        // A migração pela função, como o carregador a corre.
        let mut k = d;
        for def in KNOB_DEFS.iter().filter(|d| d.knob != Knob::Evaporation) {
            if let Some(&v) = prm.engine.get(def.key) {
                k.set(def.knob, v);
            }
        }
        migra_a_receita(&mut k, &prm, d.water, f64::from(OPACIDADE_DE_FABRICA));
        let (dil_cru, vol_cru) = ph2d_wet_paint::painter::receita::migra_para_controlos(
            k.get(Knob::PigmentPerDab),
            k.get(Knob::WaterPerDab),
            agua,
            k.get(Knob::Intensity),
            op,
        );
        migra_para_controlos(&mut k, &prm, d.water, f64::from(OPACIDADE_DE_FABRICA));
        // (c) nenhum valor preso pela faixa.
        if k.dilution() != dil_cru || k.volume_um() != vol_cru {
            presos.push(format!(
                "{} (Dilution {dil_cru} → {}, Volume {vol_cru} → {})",
                prm.name,
                k.dilution(),
                k.volume_um()
            ));
        }
        // (a) φ e h por dab = os da E4.
        let r = receita(&Controlos {
            diluicao: k.dilution(),
            volume_um: k.volume_um(),
            meio: k.medium(),
        });
        let (dphi, dh) = (rel(r.phi, phi_e4), rel(r.h, h_e4));
        if dphi > TOL || dh > TOL {
            falhas.push(format!("{} (φ {dphi:.2e}, h {dh:.2e})", prm.name));
        }
        // (b) o meio pela tabela da família (§3.2), só Water (0) ou Oil (1) — a Acrylic é Water com Re-wet 0 (E10f).
        let esperado = match (familia, prm.group.as_str()) {
            ("Oil" | "ExpressOil", _) | ("Pastel", "Oil" | "Scrape") => 1.0,
            _ => 0.0,
        };
        assert_eq!(k.medium(), esperado, "{familia}/{}: Medium", prm.name);
        if familia == "Acrylic" {
            assert_eq!(k.get(Knob::Rewet), 0.0, "{}: Acrylic = Re-wet 0", prm.name);
        }
        let lev = prm.engine.get("levelingOwnStroke").copied().unwrap_or(0.0);
        assert_eq!(
            k.body(),
            if lev >= 0.5 && esperado == 0.0 {
                BODY_DO_LEVELING
            } else {
                0.0
            },
            "{}: Body",
            prm.name
        );
        *por_meio.entry(format!("{:.1}", k.medium())).or_default() += 1;
        eprintln!(
            "{familia:<11} {:<22} φ {:.6} h {:>9.3} · Dilution {:.6} Volume {:>8.4} µm · Medium {} · Body {} (lev {lev})",
            prm.name,
            r.phi,
            r.h,
            k.dilution(),
            k.volume_um(),
            k.medium(),
            k.body()
        );

        // A ferramenta da água e a Borracha só escrevem o corpo delas: a ida e volta é a dos pincéis de tinta.
        let ferramenta = ferramenta_do_preset(&prm);
        if ferramenta.ferramenta_da_agua().is_some()
            || ferramenta == crate::FerramentaDoPincel::Borracha
        {
            continue;
        }
        n_tinta += 1;
        let preset = Preset {
            parametros: prm.clone(),
            ..e.preset.clone()
        };
        let mut t = wet();
        t.wet_preset_aplicar(&preset);
        for kn in CONTROLOS {
            assert_eq!(
                t.paint.wetpaint.knobs.get(kn),
                k.get(kn),
                "{}: {kn:?} no carregador",
                prm.name
            );
        }
        // (d) gravar → recarregar → gravar é o mesmo.
        let (g1, tx) = t.wet_preset_capturar(&prm);
        assert_eq!(g1.engine.get(CHAVE_CONTROLOS), Some(&1.0));
        let mut u = wet();
        u.wet_preset_aplicar(&Preset {
            parametros: g1.clone(),
            texturas: tx,
            ..e.preset.clone()
        });
        for kn in CONTROLOS {
            assert_eq!(
                u.paint.wetpaint.knobs.get(kn),
                k.get(kn),
                "{}: {kn:?} recarregado",
                prm.name
            );
        }
        let (g2, _) = u.wet_preset_capturar(&g1);
        assert_eq!(g1.engine, g2.engine, "{}: gravar duas vezes", prm.name);
    }
    eprintln!(
        "{n} presets Wet Paint ({n_tinta} de tinta) · por Medium {por_meio:?} · presos {presos:?}"
    );
    assert_eq!(
        (n, n_tinta),
        (57, 46),
        "contados: 57 Wet Paint, 46 de tinta (B36.1 somou o Oil Wash)"
    );
    assert!(falhas.is_empty(), "{} mudaram: {falhas:?}", falhas.len());
    assert!(presos.is_empty(), "presos pela faixa: {presos:?}");
}

/// A Dry speed é da TELA (dono 01/11 Q3): carregar um preset com outra não a muda e gravar não a escreve. A luz da
/// cena do Real Oil (`oleo_real_valores`) já era da tela e continua; o Tilt viaja no preset por decisão (W1e).
#[test]
fn um_controlo_da_tela_nao_viaja_no_preset() {
    let ink = de_fabrica("Ink Brush");
    assert_eq!(
        ink.parametros.engine.get("evaporation"),
        Some(&3.0),
        "a fixtura muda a Dry speed"
    );
    let mut t = wet();
    t.paint.wetpaint.knobs.set(Knob::Evaporation, 0.4);
    t.paint.wetpaint.oleo_real_valores.sombra = 0.37;
    let luz = t.paint.wetpaint.oleo_real_valores;
    t.wet_preset_aplicar(&ink);
    assert_eq!(t.paint.wetpaint.knobs.dry_speed(), 0.4);
    assert_eq!(t.paint.wetpaint.oleo_real_valores, luz);
    t.wet_preset_aplicar(&de_fabrica("Filbert"));
    assert_eq!(t.paint.wetpaint.knobs.dry_speed(), 0.4);
    assert_eq!(t.paint.wetpaint.oleo_real_valores, luz);
    let (g, _) = t.wet_preset_capturar(&ink.parametros);
    assert!(
        !g.engine.contains_key("evaporation"),
        "gravar não escreve a Dry speed"
    );
}

/// ⭐ **O Paint type só tem dois pontos** (E10f, dono 01/11): o seletor escreve `0`/`1`, um `SetValue` contínuo cai no
/// degrau, e um preset gravado com o `0,5` de antes (o Fan Brush Acrylic) carrega como Water — o `0,51` como Oil.
#[test]
fn o_paint_type_escreve_agua_ou_oleo_e_o_meio_antigo_carrega_como_agua() {
    use crate::ids::{PAINTER_WETPAINT_MEDIUM, painter_wetpaint_medium_option_id};
    use ph2d_editor_core::tool::PanelEvent;
    let mut t = wet();
    for (i, m) in [(1u8, 1.0), (0, 0.0)] {
        assert!(t.route_brush_wetpaint_event(&PanelEvent::SelectOption(
            PAINTER_WETPAINT_MEDIUM,
            i.to_string()
        )));
        assert_eq!(t.paint.wetpaint.knobs.medium(), m, "opção {i}");
    }
    assert!(!t.route_brush_wetpaint_event(&PanelEvent::SelectOption(
        PAINTER_WETPAINT_MEDIUM,
        "2".to_owned()
    )));
    assert_ne!(
        painter_wetpaint_medium_option_id(0),
        painter_wetpaint_medium_option_id(1)
    );
    for (v, m) in [(0.5, 0.0), (0.3, 0.0), (0.51, 1.0), (0.9, 1.0)] {
        assert!(t.route_brush_wetpaint_event(&PanelEvent::SetValue(PAINTER_WETPAINT_MEDIUM, v)));
        assert_eq!(t.paint.wetpaint.knobs.medium(), m, "SetValue {v}");
    }
    let mut fan = de_fabrica("Wet Round");
    for (v, m) in [(0.5, 0.0), (0.51, 1.0)] {
        fan.parametros
            .engine
            .insert(CHAVE_CONTROLOS.to_owned(), 1.0);
        fan.parametros.engine.insert("medium".to_owned(), v);
        let mut u = wet();
        u.wet_preset_aplicar(&fan);
        assert_eq!(u.paint.wetpaint.knobs.medium(), m, "preset gravado com {v}");
        assert_eq!(u.paint.wetpaint.knobs.e_oleo(), m == 1.0);
    }
}

#[test]
fn os_knobs_mortos_do_preset_sao_ignorados() {
    let base = de_fabrica("Wet Round");
    let mut com = base.clone();
    for k in MORTOS {
        assert!(KNOB_DEFS.iter().all(|d| d.key != k), "{k} ainda no registo");
        com.parametros.engine.insert(k.to_owned(), 0.77);
    }
    let (mut a, mut b) = (wet(), wet());
    a.wet_preset_aplicar(&base);
    b.wet_preset_aplicar(&com);
    assert_eq!(a.paint.wetpaint.knobs, b.paint.wetpaint.knobs);
    let (g, _) = b.wet_preset_capturar(&com.parametros);
    let lidos: Vec<_> = MORTOS
        .iter()
        .filter(|k| g.engine.contains_key(**k))
        .collect();
    // A gravação parte do preset-base (os campos de ondas futuras sobrevivem): as chaves mortas viajam
    // intactas e continuam sem leitor — o que importa é que nenhuma muda um knob.
    eprintln!("chaves mortas regravadas intactas: {lidos:?}");
    let mut c = wet();
    c.wet_preset_aplicar(&Preset {
        parametros: g,
        ..com.clone()
    });
    assert_eq!(c.paint.wetpaint.knobs, a.paint.wetpaint.knobs);
}

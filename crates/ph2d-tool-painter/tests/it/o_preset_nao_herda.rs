//! **Escolher um preset DEPOIS de mexer no pincel dá o preset, não o preset com o lixo do anterior** (a família
//! do estado escondido, 22/10 — `ph2d-panel-painter-layers::seam_estado_escondido`). Cada sujeira que a família
//! conhece — o grão com Grain Colors, o dual com imagem, a ponta de N imagens, a pilha do Composite, a Shape Color —
//! é feita pela porta do painel; depois o preset de fábrica de cada meio pela porta do navegador
//! (`preset_aplicar`). O traço e o preview têm de ser os do mesmo preset escolhido num pincel limpo.

use ph2d_brush_presets::{Biblioteca, Preset};
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{
    CanvasPaintTool, CanvasPointer, PanelEvent, PointerPhase, RasterEditTool,
};
use ph2d_tool_painter::{
    LINHAS_DA_PONTA, LinhaDaPonta, PaintMedia, PainterTool, ids, meio_do_preset,
};
use std::path::PathBuf;

const W: usize = 200;
const H: usize = 120;

fn img(c: u8) -> Vec<ph2d_tool_painter::SrgbRgba> {
    let px: Vec<u8> = (0..16 * 16)
        .flat_map(|i| {
            let v = if (i % 16 + i / 16) % 3 == 0 { c } else { 255 };
            [v, v, v, 255]
        })
        .collect();
    ph2d_tool_painter::SrgbRgba::iter_byte_slice(&px).collect()
}

fn ponta(l: LinhaDaPonta, k: u8) -> PanelEvent {
    let i = LINHAS_DA_PONTA
        .iter()
        .position(|x| *x == l)
        .expect("a linha da ponta");
    PanelEvent::SelectOption(ids::PAINTER_PONTA_FIELDS[i], k.to_string())
}

/// Uma sujeira da família, pela porta do painel (e o seletor da shell para as imagens).
struct Sujeira {
    nome: &'static str,
    faz: fn(&mut PainterTool),
}

const SUJEIRAS: &[Sujeira] = &[
    Sujeira {
        nome: "Grain Colors",
        faz: |t| {
            t.handle_panel_event(PanelEvent::SelectOption(
                ids::PAINTER_BRUSH_TEXTURE_KIND,
                "3".into(),
            ));
            t.handle_panel_event(PanelEvent::Click(ids::PAINTER_BRUSH_TEXTURE_RAMP_ENABLE));
        },
    },
    Sujeira {
        nome: "Dual brush",
        faz: |t| {
            t.handle_panel_event(ponta(LinhaDaPonta::DualMode, 4));
            t.set_brush_dual_image_rgba(&img(30), 16, 16);
        },
    },
    Sujeira {
        nome: "N imagens",
        faz: |t| {
            for k in 0..3u8 {
                t.add_brush_shape_hose_image_rgba(&img(40 + 60 * k), 16, 16);
            }
        },
    },
    Sujeira {
        nome: "Composite",
        faz: |t| {
            t.handle_panel_event(PanelEvent::Click(ids::PAINTER_BRUSH_COMPOSITE_ENABLE));
            t.handle_panel_event(PanelEvent::Click(
                ids::PAINTER_BRUSH_COMPOSITE_ADD_OPTION[2],
            ));
            t.handle_panel_event(PanelEvent::Click(ids::PAINTER_BRUSH_COMPOSITE_ADD));
        },
    },
    Sujeira {
        nome: "Shape Color",
        faz: |t| t.handle_panel_event(PanelEvent::Click(ids::PAINTER_SHAPE_RAMP_ENABLE)),
    },
];

fn tool(m: PaintMedia) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_media(m);
    t.set_brush_color_srgb8([40, 110, 220]);
    t.set_wet_relogio_fixo(true);
    t
}

/// Um traço ondulado e a tela depois de a água assentar um pouco.
fn traco(t: &mut PainterTool) -> Vec<u8> {
    let p = |pos, phase| CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    let mut ultima = Vec::new();
    let mut tique = |t: &mut PainterTool| {
        t.on_tick(1000.0 / 60.0);
        if let Some((px, _, _)) = t.take_preview_arc() {
            ultima = px.as_ref().clone();
        }
    };
    t.on_canvas_pointer(p([30.0, 60.0], PointerPhase::Down));
    let mut pos = [30.0, 60.0];
    for k in 1..=40 {
        let f = k as f32 / 40.0;
        pos = [
            30.0 + 140.0 * f,
            60.0 + 18.0 * (f * std::f32::consts::TAU).sin(),
        ];
        t.on_canvas_pointer(p(pos, PointerPhase::Move));
        tique(t);
    }
    t.on_canvas_pointer(p(pos, PointerPhase::Up));
    for _ in 0..20 {
        tique(t);
    }
    ultima
}

fn preview(t: &PainterTool) -> Vec<u8> {
    let mut v = Vec::new();
    ph2d_tool_painter::pintar_preview(&t.wet_preview_cena(), &|| false, &mut |px, _, _| {
        v = px.to_vec()
    });
    v
}

fn difere(a: &[u8], b: &[u8]) -> usize {
    a.chunks(4).zip(b.chunks(4)).filter(|(x, y)| x != y).count() + a.len().abs_diff(b.len())
}

/// Uma escolha de preset: um ficheiro de fábrica (o navegador, `preset_aplicar`) ou o «Basic» do menu do pincel
/// (`PAINTER_BRUSH_PRESET`: 0 = Digital, 1 = Watercolor).
enum Escolha {
    Ficheiro(PaintMedia, Box<Preset>),
    Basico(u8),
}

impl Escolha {
    fn meio(&self) -> PaintMedia {
        match self {
            Escolha::Ficheiro(m, _) => *m,
            Escolha::Basico(1) => PaintMedia::Watercolor,
            Escolha::Basico(_) => PaintMedia::Digital,
        }
    }

    fn nome(&self) -> String {
        match self {
            Escolha::Ficheiro(_, p) => p.parametros.name.clone(),
            Escolha::Basico(k) => format!("Basic {k}"),
        }
    }

    fn aplica(&self, t: &mut PainterTool) {
        match self {
            Escolha::Ficheiro(_, p) => {
                t.preset_aplicar(p).expect("o preset aplica-se");
            }
            Escolha::Basico(k) => {
                t.handle_panel_event(PanelEvent::SelectOption(
                    ids::PAINTER_BRUSH_PRESET,
                    k.to_string(),
                ));
            }
        }
    }
}

/// Um preset de fábrica por meio (o primeiro da biblioteca) e os dois «Basic».
fn escolhas() -> Vec<Escolha> {
    let raiz = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let bib = Biblioteca::carregar(&raiz, None);
    let mut v: Vec<Escolha> = Vec::new();
    for e in bib.entradas {
        if let Some(m) = meio_do_preset(&e.preset.parametros.media)
            && !v.iter().any(|x| x.meio() == m)
        {
            v.push(Escolha::Ficheiro(m, Box::new(e.preset)));
        }
    }
    assert!(
        v.len() >= 2,
        "a fábrica tem presets de pelo menos dois meios"
    );
    v.extend([Escolha::Basico(0), Escolha::Basico(1)]);
    v
}

/// ⭐ Cada sujeira × cada preset: sujo + preset == limpo + preset (o traço e o preview), e o CONTROLO — a sujeira
/// muda o traço do pincel sem preset (senão a igualdade é vácua).
#[test]
fn o_preset_escolhido_depois_de_sujar_o_pincel_e_o_preset() {
    let mut falhas = Vec::new();
    let mut vivas = vec![false; SUJEIRAS.len()];
    for e in escolhas() {
        let m = e.meio();
        let limpo = {
            let mut t = tool(m);
            e.aplica(&mut t);
            (traco(&mut t), preview(&t))
        };
        for (k, s) in SUJEIRAS.iter().enumerate() {
            let mut t = tool_com(m, s);
            let vivo = difere(&traco(&mut tool_com(m, s)), &traco(&mut tool(m)));
            vivas[k] |= vivo > 0;
            e.aplica(&mut t);
            let (tela, pv) = (traco(&mut t), preview(&t));
            let (n, np) = (difere(&tela, &limpo.0), difere(&pv, &limpo.1));
            eprintln!(
                "PRESET\t{m:?}\t{}\t{}\ttela={n}\tpreview={np}\tvivo={vivo}",
                e.nome(),
                s.nome
            );
            if n > 0 || np > 0 {
                falhas.push(format!(
                    "{m:?} {} depois de {}: tela {n} · preview {np}",
                    e.nome(),
                    s.nome
                ));
            }
        }
    }
    for (s, viva) in SUJEIRAS.iter().zip(vivas) {
        assert!(
            viva,
            "CONTROLO: «{}» não muda o traço em meio nenhum (a igualdade seria vácua)",
            s.nome
        );
    }
    assert!(
        falhas.is_empty(),
        "o preset herdou o lixo do pincel anterior: {falhas:#?}"
    );
}

fn tool_com(m: PaintMedia, s: &Sujeira) -> PainterTool {
    let mut t = tool(m);
    (s.faz)(&mut t);
    t
}

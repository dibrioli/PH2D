//! **Camadas sem perda de dados** (missão Painter 4K, bloco B01 do
//! [BACKLOG](../../../../../docs/Painter/missao_4k/BACKLOG.md)): toda operação estrutural deixa os mapas por
//! camada (`images`, `heights`, `covers`, `mats`, `vidros`, `tintas_molhadas`) a seguir a pilha, a activa
//! pintável e fora de `images`, e a tela dela com os píxeis DELA.

use crate::layers::{LayerId as RtLayerId, LayerKind, LayerStack};
use crate::tool::PainterTool;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{
    CanvasPaintTool, CanvasPointer, PanelEvent, PointerPhase, RasterEditTool,
};
use ph2d_painter_brush::{BrushSpec, Falloff, StrokeMethod};
use std::sync::Arc;

const LADO: u32 = 64;
const N: usize = (LADO as usize) * (LADO as usize);

fn doc() -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; N * 4], LADO, LADO);
    let b = BrushSpec {
        radius_px: 2.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.0, 0.0, 0.0],
        space_attenuation: false,
        ..Default::default()
    };
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    t
}

fn cheia(c: [u8; 3]) -> Vec<u8> {
    (0..N).flat_map(|_| [c[0], c[1], c[2], 255]).collect()
}

const VERMELHO: [u8; 3] = [200, 20, 20];
const AZUL: [u8; 3] = [20, 20, 200];
const VERDE: [u8; 3] = [20, 200, 20];

fn camada(t: &mut PainterTool, nome: &str, c: [u8; 3]) -> RtLayerId {
    t.add_raster_layer_with_pixels(nome, cheia(c))
        .expect("a camada")
}

fn pixels_de(t: &PainterTool, id: RtLayerId) -> Vec<u8> {
    if t.layers.active() == Some(id) {
        t.canvas_rgba.as_ref().clone()
    } else {
        t.images
            .get(&id)
            .map_or_else(|| vec![0u8; N * 4], |i| i.rgba8.clone())
    }
}

fn cp(x: f32, y: f32, phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos: [x, y],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// ⭐ O invariante do bloco: os mapas por camada seguem a pilha.
fn mapas_seguem(t: &PainterTool) -> Result<(), String> {
    let vivas: std::collections::BTreeSet<RtLayerId> = t.layers.all_ids().collect();
    let a = t.layers.active().ok_or("sem activa")?;
    if !vivas.contains(&a) {
        return Err(format!("a activa {a:?} não existe"));
    }
    if matches!(
        t.layers.get(a).map(|l| &l.kind),
        Some(LayerKind::Group(_) | LayerKind::Adjustment(_))
    ) {
        return Err(format!("a activa {a:?} não é pintável"));
    }
    if t.images.contains_key(&a) {
        return Err(format!("a activa {a:?} está em images"));
    }
    let orfas = |nome: &str, ids: Vec<RtLayerId>| -> Result<(), String> {
        match ids.iter().find(|id| !vivas.contains(id)) {
            Some(id) => Err(format!("{nome} guarda a camada morta {id:?}")),
            None => Ok(()),
        }
    };
    orfas("images", t.images.keys().copied().collect())?;
    orfas("heights", t.heights.keys().copied().collect())?;
    orfas("covers", t.covers.keys().copied().collect())?;
    orfas("mats", t.mats.keys().copied().collect())?;
    orfas("vidros", t.vidros.keys().copied().collect())?;
    orfas(
        "tintas_molhadas",
        t.tintas_molhadas.iter().copied().collect(),
    )?;
    orfas("adormecidas", t.camadas_adormecidas())?; // B02.1
    Ok(())
}

fn com_corpo(t: &mut PainterTool, id: RtLayerId, h: f32) {
    t.heights.insert(id, Arc::new(vec![h; N]));
    t.covers.insert(id, Arc::new(vec![(h * 255.0) as u8; N]));
    t.mats.insert(id, Arc::new(vec![[7, 6, 5, 4, 3, 2, 1]; N]));
}

/// B01.1 — apagar o grupo que contém a activa: a camada que fica mostra os píxeis DELA.
#[test]
fn apagar_o_grupo_da_activa_nao_mistura_pixels() {
    let mut t = doc();
    let a = camada(&mut t, "A", VERMELHO);
    let b = camada(&mut t, "B", AZUL);
    com_corpo(&mut t, b, 0.5);
    let g = t.group_active().expect("o grupo");
    assert_eq!(t.layers.active(), Some(b));
    assert!(t.delete_layer(g));
    let nova = t.layers.active().expect("activa");
    assert_eq!(nova, a, "a nova activa é a do topo");
    assert!(
        pixels_de(&t, a) == cheia(VERMELHO),
        "A ficou com os píxeis de B"
    );
    mapas_seguem(&t).unwrap();
}

/// B01.2 — apagar a camada do topo com um grupo à frente: a activa nunca é o grupo.
#[test]
fn a_activa_nunca_e_um_grupo() {
    // Cena P: o grupo tem uma camada dentro ⇒ é ela a activa.
    let mut t = doc();
    let c = camada(&mut t, "C", VERDE);
    t.group_active().expect("o grupo");
    let l = camada(&mut t, "L", VERMELHO);
    assert!(t.delete_layer(l));
    assert_eq!(t.layers.active(), Some(c), "cena P");
    assert!(pixels_de(&t, c) == cheia(VERDE), "cena P: os píxeis de C");
    mapas_seguem(&t).unwrap();
    // Cena Q: o grupo está vazio ⇒ a base.
    let mut t = doc();
    let base = t.layers.active().expect("a base");
    t.add_group().expect("o grupo vazio");
    let l = camada(&mut t, "L", VERMELHO);
    assert!(t.delete_layer(l));
    assert_eq!(t.layers.active(), Some(base), "cena Q");
    mapas_seguem(&t).unwrap();
}

/// B01.3 — duplicar uma camada com relevo: a cópia leva o corpo inteiro, ao byte.
#[test]
fn duplicar_a_camada_leva_o_corpo() {
    let mut t = doc();
    let a = camada(&mut t, "A", VERMELHO);
    com_corpo(&mut t, a, 0.75);
    let copia = t.duplicate_layer(a).expect("a cópia");
    assert!(t.heights.get(&copia) == t.heights.get(&a), "heights");
    assert!(t.covers.get(&copia) == t.covers.get(&a), "covers");
    assert!(t.mats.get(&copia) == t.mats.get(&a), "mats");
    assert!(pixels_de(&t, copia) == cheia(VERMELHO));
    mapas_seguem(&t).unwrap();
}

/// B01.4 — a camada que nasce com píxeis (a fusão em camadas): desfazer e refazer devolve os píxeis.
#[test]
fn refazer_a_camada_com_pixels_devolve_os_pixels() {
    let mut t = doc();
    let m = camada(&mut t, "M", VERMELHO);
    assert!(t.undo_last());
    assert!(t.layers.get(m).is_none(), "o desfazer tira a camada");
    assert!(t.redo_last());
    assert_eq!(t.layers.active(), Some(m));
    assert!(
        pixels_de(&t, m) == cheia(VERMELHO),
        "o refazer devolveu-a vazia"
    );
    mapas_seguem(&t).unwrap();
}

fn elipse_aberta(t: &mut PainterTool) {
    t.paint.brush.stroke_method = StrokeMethod::Ellipse;
    t.on_canvas_pointer(cp(32.0, 32.0, PointerPhase::Down));
    t.on_canvas_pointer(cp(44.0, 32.0, PointerPhase::Move));
    t.on_canvas_pointer(cp(44.0, 32.0, PointerPhase::Up));
    assert!(t.paint.ellipse.is_some(), "controlo: a elipse está aberta");
}

/// B01.5 — toda porta de camada fecha a forma aberta (Apply na camada onde ela foi desenhada) antes de
/// trocar a tela; um Esc depois não cola na camada nova a tela da antiga.
#[test]
fn toda_porta_de_camada_fecha_a_forma_aberta() {
    type Porta = fn(&mut PainterTool, RtLayerId);
    let portas: [(&str, Porta); 5] = [
        ("select_layer", |t, a| t.select_layer(a)),
        ("delete_layer", |t, _| {
            let b = t.layers.active().expect("b");
            assert!(t.delete_layer(b));
        }),
        ("duplicate_layer", |t, _| {
            let b = t.layers.active().expect("b");
            t.duplicate_layer(b).expect("a cópia");
        }),
        ("add_raster_layer", |t, _| {
            t.add_raster_layer("X").expect("a nova");
        }),
        ("add_mask_to_active", |t, _| {
            t.add_mask_to_active().expect("a máscara");
        }),
    ];
    for (nome, porta) in portas {
        let mut t = doc();
        let a = camada(&mut t, "A", VERMELHO);
        let b = camada(&mut t, "B", AZUL);
        elipse_aberta(&mut t);
        let com_elipse = pixels_de(&t, b);
        porta(&mut t, a);
        assert!(
            !t.is_editing_shape() && !t.has_parked_shapes() && t.paint.drag_preview.is_none(),
            "{nome}: a forma ficou aberta"
        );
        t.cancel_open_shape();
        assert!(pixels_de(&t, a) == cheia(VERMELHO), "{nome}: A mudou");
        if t.layers.get(b).is_some() {
            assert!(
                pixels_de(&t, b) == com_elipse,
                "{nome}: a elipse não ficou em B"
            );
        }
        mapas_seguem(&t).unwrap_or_else(|e| panic!("{nome}: {e}"));
    }
}

fn wet_vivo() -> PainterTool {
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255u8; N * 4], LADO, LADO);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    t
}

fn traco(t: &mut PainterTool, y: f32) {
    t.set_brush_size_px(8.0);
    t.set_brush_color_srgb8([40, 90, 220]);
    t.on_canvas_pointer(cp(8.0, y, PointerPhase::Down));
    for k in 1..=12 {
        t.on_canvas_pointer(cp(8.0 + 4.0 * k as f32, y, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(cp(56.0, y, PointerPhase::Up));
    t.on_tick(1000.0 / 60.0);
}

/// B01.6 — a tinta viva do Wet não morre quando uma porta estrutural guarda e recarrega a mesma activa.
#[test]
fn apply_mask_e_delete_com_tinta_viva_mantem_a_sessao() {
    let mut t = wet_vivo();
    let base = t.layers.active().expect("a base");
    t.add_raster_layer("O").expect("O");
    let o = t.layers.active().expect("o");
    let m = t.add_mask_to_active().expect("a máscara");
    let lixo = t.add_raster_layer("lixo").expect("lixo");
    t.select_layer(base);
    t.set_paint_tool_mode("wetpaint");
    traco(&mut t, 32.0);
    assert!(
        t.paint.wetpaint.session.is_some(),
        "controlo: a sessão viva"
    );
    assert!(t.apply_mask(m));
    assert_eq!(t.layers.active(), Some(base));
    assert!(
        t.paint.wetpaint.session.is_some(),
        "apply_mask matou a tinta viva"
    );
    assert!(t.delete_layer(lixo));
    assert!(
        t.paint.wetpaint.session.is_some(),
        "delete_layer matou a tinta viva"
    );
    assert!(t.layers.get(o).is_some());
    mapas_seguem(&t).unwrap();
}

/// B01.7 — uma máscara que falha não deixa a activa duplicada em `images`.
#[test]
fn a_mascara_que_falha_nao_duplica_a_activa() {
    let mut t = doc();
    let a = camada(&mut t, "A", VERMELHO);
    let m = t.add_mask_to_active().expect("a máscara");
    t.select_layer(a);
    t.images.insert(
        m,
        Arc::new(crate::compositor::LayerImage {
            width: LADO,
            height: LADO,
            rgba8: Vec::new(),
        }),
    );
    assert!(!t.apply_mask(m), "a máscara curta é recusada");
    mapas_seguem(&t).unwrap();
    assert!(pixels_de(&t, a) == cheia(VERMELHO));
}

/// B01.8 — duplicar um GRUPO clona a subárvore com ids novos; a base nunca entra num grupo.
#[test]
fn duplicar_um_grupo_nao_partilha_os_filhos() {
    let mut s = LayerStack::new();
    let base = s.add_raster("base", 4, 4).expect("base");
    let l = s.add_raster("L", 4, 4).expect("L");
    let g = s.add_group("G").expect("G");
    assert!(s.move_into_group(l, g));
    let g2 = s.duplicate(g).expect("a cópia do grupo");
    let filhos = |s: &LayerStack, g| match s.get(g).map(|x| &x.kind) {
        Some(LayerKind::Group(gr)) => gr.children.clone(),
        _ => Vec::new(),
    };
    let (f1, f2) = (filhos(&s, g), filhos(&s, g2));
    assert_eq!(f2.len(), 1);
    assert!(
        f1.iter().all(|x| !f2.contains(x)),
        "a cópia partilha os filhos"
    );
    assert!(s.get(f2[0]).is_some(), "o filho da cópia existe");
    s.remove(g2);
    assert!(
        s.get(l).is_some(),
        "apagar a cópia levou o filho do original"
    );
    assert!(!s.move_into_group(base, g), "a base entrou num grupo");
}

fn aguada(t: &mut PainterTool, w: RtLayerId) {
    t.vidros.insert(w, Arc::new(Vec::new()));
    t.handle_panel_event(PanelEvent::SelectOption(
        crate::ids::PAINTER_WATERCOLOR_PAPER_COLOR_THUMB,
        "153,121,91".to_owned(),
    ));
}

/// B01.9 — apagar a camada da aguada (sozinha ou dentro de um grupo) leva o vidro dela: o papel de cor
/// volta a não atravessar vidro nenhum.
#[test]
fn papel_atravessa_vidro_falso_depois_de_apagar_a_aguada() {
    for no_grupo in [false, true] {
        let mut t = doc();
        let w = t.add_raster_layer("aguada").expect("a aguada");
        aguada(&mut t, w);
        assert!(t.papel_atravessa_vidro(), "controlo: a aguada deixou vidro");
        let alvo = if no_grupo {
            t.select_layer(w);
            t.group_active().expect("o grupo")
        } else {
            w
        };
        assert!(t.delete_layer(alvo));
        assert!(
            !t.papel_atravessa_vidro(),
            "grupo={no_grupo}: o vidro ficou"
        );
        mapas_seguem(&t).unwrap();
    }
}

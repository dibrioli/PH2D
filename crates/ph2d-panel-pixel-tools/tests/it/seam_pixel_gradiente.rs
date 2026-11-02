//! P8: as opções do Gradient no painel PIXEL — cada uma carregada no MEIO, pelo rato de verdade,
//! chega às opções (o app lê-as no arrasto: `ph2d-app-pixel/tests/it/degrade.rs`), e na largura
//! MÍNIMA da doca nenhum rótulo delas é cortado.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::pixel_panel::{PixelOptions, PixelPanelInfo, PixelRequest, PixelTool};
use ph2d_editor_core::text_elide::elisao;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_panel_pixel_tools::{PixelToolsPanel, PixelToolsPanelState, ids, set_current_pixel};
use ph2d_ui_testkit::MockPanelHost;

fn viewport(w: f32) -> Rect {
    Rect {
        x: 0.0,
        y: 0.0,
        w,
        h: 4000.0, // LITERAL-PX-OK: alto o bastante para o painel inteiro caber sem rolar
    }
}

fn info(o: PixelOptions) -> PixelPanelInfo {
    PixelPanelInfo {
        editing: true,
        tool: PixelTool::Gradient,
        options: o,
        ..Default::default()
    }
}

/// Pinta o painel com as opções `o` na largura `w` e carrega no meio de `id`.
fn carrega(o: PixelOptions, w: f32, id: ph2d_a11y::NodeId) -> Vec<EditorAction> {
    let mut host = MockPanelHost::with_panel::<PixelToolsPanel>();
    let mut st = PixelToolsPanelState::default();
    set_current_pixel(info(o));
    let rects = host.paint::<PixelToolsPanel>(&mut st, viewport(w));
    let r = rects
        .iter()
        .find(|(n, _)| *n == id)
        .map(|(_, r)| *r)
        .unwrap_or_else(|| panic!("o painel nunca pintou {id:?}"));
    let ev = host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5);
    assert!(
        ev.iter()
            .any(|e| matches!(e, WidgetEvent::Click(c) | WidgetEvent::Toggled(c) if *c == id)),
        "carregar no meio de {id:?} deu {ev:?}: MORTO sob o dedo"
    );
    for e in ev {
        let _ = host.apply_panel_event::<PixelToolsPanel>(&mut st, e);
    }
    host.drained_actions()
}

fn so(a: &[EditorAction], o: PixelOptions) {
    assert_eq!(a, [EditorAction::Pixel(PixelRequest::Options(Box::new(o)))]);
}

#[test]
fn each_gradient_option_clicked_in_the_panel_reaches_the_options() {
    let w = WidgetStore::DOCK_W_MIN;
    let base = PixelOptions::default();
    so(
        &carrega(base, w, ids::GRADIENT_RADIAL),
        PixelOptions {
            gradient_radial: true,
            ..base
        },
    );
    let radial = PixelOptions {
        gradient_radial: true,
        ..base
    };
    so(&carrega(radial, w, ids::GRADIENT_LINEAR), base);
    for (d, id, _) in ids::GRADIENT_DITHERS {
        // Parte de outra matriz, para o clique MUDAR alguma coisa.
        let outra = ids::GRADIENT_DITHERS
            .iter()
            .map(|(m, ..)| *m)
            .find(|m| *m != d)
            .expect("quatro matrizes");
        let de = PixelOptions {
            gradient_dither: outra,
            ..base
        };
        let quer = PixelOptions {
            gradient_dither: d,
            ..base
        };
        so(&carrega(de, w, id), quer);
        assert_eq!(
            quer.dynamics, base.dynamics,
            "a matriz do Gradient não toca a da dinâmica"
        );
    }
    so(
        &carrega(base, w, ids::GRADIENT_RAMP),
        PixelOptions {
            gradient_ramp: true,
            ..base
        },
    );
}

#[test]
fn no_gradient_option_label_is_cut_at_the_minimum_dock_width() {
    let mut host = MockPanelHost::with_panel::<PixelToolsPanel>();
    let mut st = PixelToolsPanelState::default();
    set_current_pixel(info(PixelOptions::default()));
    let (rects, medidos) = elisao::medindo(|| {
        host.paint::<PixelToolsPanel>(&mut st, viewport(WidgetStore::DOCK_W_MIN))
    });
    let nossos: Vec<&str> = ids::GRADIENT_DITHERS
        .iter()
        .map(|(_, _, k)| tr(k))
        .chain(
            [
                "panel.pixel.gradient",
                "panel.pixel.gradient.shape",
                "panel.pixel.gradient.linear",
                "panel.pixel.gradient.radial",
                "panel.pixel.gradient.dither",
                "panel.pixel.gradient.ramp",
            ]
            .map(tr),
        )
        .collect();
    let vistos: Vec<_> = medidos
        .iter()
        .filter(|m| nossos.contains(&m.texto.as_str()))
        .collect();
    // ⛔ Controlo de vacuidade: o censo tem de ter VISTO os rótulos (as 4 matrizes pelo menos).
    assert!(
        vistos.len() >= ids::GRADIENT_DITHERS.len(),
        "o censo só viu {:?}",
        vistos.iter().map(|m| &m.texto).collect::<Vec<_>>()
    );
    let cortados: Vec<_> = vistos
        .iter()
        .filter(|m| !m.coube())
        .map(|m| (&m.texto, &m.pintado))
        .collect();
    assert!(
        cortados.is_empty(),
        "cortados na largura mínima: {cortados:?}"
    );
    // Os quatro botões da matriz ficam na coluna do valor: entre a esquerda do Linear e a direita
    // do Radial (o painel tem largura própria, maior que a doca mínima; a régua é a coluna dele).
    let r_de = |id| {
        rects
            .iter()
            .find(|(n, _)| *n == id)
            .map(|(_, r)| *r)
            .expect("pintado")
    };
    let (lin, rad) = (r_de(ids::GRADIENT_LINEAR), r_de(ids::GRADIENT_RADIAL));
    let (esq, dir) = (lin.x, rad.x + rad.w);
    for (_, id, _) in ids::GRADIENT_DITHERS {
        let r = r_de(id);
        assert!(
            r.x >= esq && r.x + r.w <= dir + 1.0,
            "{id:?} fora de [{esq}, {dir}]: {r:?}"
        );
    }
}

/// A matriz 2 + 2 sem vão: a 2.ª fila desce do fim da 1.ª só o degrau entre controlos (o mesmo que
/// separa Linear|Radial da linha seguinte), nunca um cabeçalho vazio.
#[test]
fn the_dither_rows_stack_without_an_empty_band() {
    let mut host = MockPanelHost::with_panel::<PixelToolsPanel>();
    let mut st = PixelToolsPanelState::default();
    set_current_pixel(info(PixelOptions::default()));
    let rects = host.paint::<PixelToolsPanel>(&mut st, viewport(WidgetStore::DOCK_W_MIN));
    let r = |id| {
        rects
            .iter()
            .find(|(n, _)| *n == id)
            .map(|(_, r)| *r)
            .expect("pintado")
    };
    let (a, c) = (r(ids::GRADIENT_DITHERS[0].1), r(ids::GRADIENT_DITHERS[2].1));
    let vao = c.y - (a.y + a.h);
    assert!(
        (vao - ph2d_tokens::control_gap_px()).abs() < 0.5,
        "vão entre as filas = {vao} px (o degrau é {})",
        ph2d_tokens::control_gap_px()
    );
}

/// Ordem do dono (smoke P8, 02/11): o Gradient pinta a selecção (ou o cel) inteira, sem
/// contiguidade — as opções do balde saem do painel dele. Controlo: o balde ainda as pinta.
#[test]
fn the_gradient_panel_has_no_bucket_options() {
    let balde = [
        ids::TOLERANCE,
        ids::CONTIGUOUS,
        ids::CONNECT_4,
        ids::CONNECT_8,
        ids::SAMPLE_LAYER,
        ids::SAMPLE_VISIBLE,
    ];
    let pintados = |tool| {
        let mut host = MockPanelHost::with_panel::<PixelToolsPanel>();
        let mut st = PixelToolsPanelState::default();
        set_current_pixel(PixelPanelInfo {
            tool,
            ..info(PixelOptions::default())
        });
        host.paint::<PixelToolsPanel>(&mut st, viewport(WidgetStore::DOCK_W_MIN))
            .into_iter()
            .map(|(n, _)| n)
            .collect::<Vec<_>>()
    };
    let no_balde = pintados(PixelTool::Bucket);
    for id in balde {
        assert!(no_balde.contains(&id), "controlo: o balde pinta {id:?}");
    }
    let no_gradiente = pintados(PixelTool::Gradient);
    assert!(
        no_gradiente.contains(&ids::GRADIENT_RADIAL),
        "o painel do Gradient pintou"
    );
    for id in balde {
        assert!(
            !no_gradiente.contains(&id),
            "o Gradient ainda mostra {id:?}"
        );
    }
}

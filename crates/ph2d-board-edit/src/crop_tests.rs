//! W16 — o recorte pelo gesto contra os números do Excalidraw 0.18.1 (`saidas/w16/w16_sondas.json`:
//! uma imagem natural 400 × 300 a zoom 1, o canto superior esquerdo em `(420, 180)`).

use super::*;
use crate::{Metrics, Mods};
use ph2d_board_model::{Files, Image, ImageFile, Rgba, Style};
use ph2d_text::TextSystem;

const VIEW: [f64; 4] = [-10_000.0, -10_000.0, 10_000.0, 10_000.0];
const NONE: Mods = Mods {
    shift: false,
    ctrl: false,
    alt: false,
};
const SHIFT: Mods = Mods {
    shift: true,
    ctrl: false,
    alt: false,
};

struct W {
    ed: Editor,
    doc: BoardDoc,
    h: History,
    id: ElementId,
}

/// A imagem 400 × 300 numa caixa `w × h` em `(420, 180)`, com `edit` (espelho, ângulo), seleccionada
/// e em recorte.
fn world(w: f64, h: f64, edit: impl Fn(&mut Element)) -> W {
    let ink = Rgba([20, 20, 20, 255]);
    let mut ed = Editor::new(
        Style::new(None, Some(ink), ink),
        Metrics {
            handle: 6.0,
            rotate_offset: 24.0,
            snap: 8.0,
            drag: 3.0,
            click_size: [160.0, 100.0],
            paste_offset: 10.0,
            bind: 8.0,
            dot: 40.0,
            eraser: 10.0,
            frame_title: 20.0,
        },
    );
    let file = ImageFile::new(vec![1; 8], [400, 300]);
    let mut files = Files::new();
    files.insert(file.id(), file.clone());
    ed.know_files(&files);
    let mut doc = BoardDoc::default();
    let img = Image::new(file.id(), Style::new(None, None, ink));
    let mut el = Element::new_image(doc.mint_id(), doc.z_on_top(), img, [420.0, 180.0, w, h]);
    edit(&mut el);
    let id = el.id;
    BoardOp::Put(el).apply(&mut doc);
    let mut h = History::default();
    ed.selection = std::collections::BTreeSet::from([id]);
    assert!(ed.begin_crop(&mut doc, &mut h));
    W { ed, doc, h, id }
}

impl W {
    fn drag(&mut self, from: [f64; 2], by: [f64; 2], mods: Mods) {
        let mut ts = TextSystem::without_system_fonts();
        let p = |world| Pointer {
            world,
            px: 1.0,
            mods,
        };
        let to = [from[0] + by[0], from[1] + by[1]];
        self.ed
            .pointer_down(&mut self.doc, &mut self.h, &mut ts, p(from), VIEW);
        self.ed.pointer_move(
            &mut self.doc,
            &mut ts,
            p([from[0] + by[0] / 2.0, from[1] + by[1] / 2.0]),
        );
        self.ed.pointer_move(&mut self.doc, &mut ts, p(to));
        self.ed.pointer_up(&mut self.doc, &mut self.h, p(to));
    }

    fn el(&self) -> &Element {
        self.doc.get(self.id).unwrap()
    }

    fn boxed(&self) -> [f64; 4] {
        let e = self.el();
        [e.x, e.y, e.w, e.h].map(|v| (v * 1e6).round() / 1e6)
    }

    fn crop(&self) -> Option<[f64; 4]> {
        self.el()
            .image()
            .unwrap()
            .crop
            .map(|c| [c.x, c.y, c.w, c.h].map(|v| (v * 1e6).round() / 1e6))
    }
}

const SE: [f64; 2] = [820.0, 480.0];
const NW: [f64; 2] = [420.0, 180.0];

/// ⭐ A2/A3 — o canto SE: o canto oposto fica, a caixa encolhe, o recorte é a janela em píxeis
/// naturais; o NW move o elemento e a imagem fica parada no ecrã. A proporção é livre; `Shift` dá a
/// da imagem natural (a maior das duas caixas). A2d: a meio tamanho o recorte conta-se em naturais.
#[test]
fn a_corner_moves_its_side_of_the_window_and_the_picture_stays_put() {
    let mut w = world(400.0, 300.0, |_| {});
    w.drag(SE, [-80.0, -60.0], NONE);
    assert_eq!(w.boxed(), [420.0, 180.0, 320.0, 240.0]);
    assert_eq!(w.crop(), Some([0.0, 0.0, 320.0, 240.0]));

    let mut w = world(400.0, 300.0, |_| {});
    w.drag(SE, [-80.0, -30.0], NONE);
    assert_eq!(w.boxed()[2..], [320.0, 270.0], "livre");
    let mut w = world(400.0, 300.0, |_| {});
    w.drag(SE, [-80.0, -30.0], SHIFT);
    assert_eq!(w.boxed()[2..], [360.0, 270.0], "Shift: a proporção natural");

    let mut w = world(400.0, 300.0, |_| {});
    w.drag(NW, [80.0, 60.0], NONE);
    assert_eq!(w.boxed(), [500.0, 240.0, 320.0, 240.0]);
    assert_eq!(w.crop(), Some([80.0, 60.0, 320.0, 240.0]));

    let mut w = world(200.0, 150.0, |_| {});
    w.drag([620.0, 330.0], [-40.0, -30.0], NONE);
    assert_eq!(w.boxed()[2..], [160.0, 120.0]);
    assert_eq!(w.crop(), Some([0.0, 0.0, 320.0, 240.0]));
}

/// A4 — as pegas dos MEIOS dos lados (sem desenho) movem só o lado delas.
#[test]
fn a_side_middle_moves_only_its_side() {
    let mut w = world(400.0, 300.0, |_| {});
    w.drag([820.0, 330.0], [-80.0, 0.0], NONE);
    assert_eq!(w.boxed(), [420.0, 180.0, 320.0, 300.0]);
    w.drag([580.0, 180.0], [0.0, 60.0], NONE);
    assert_eq!(w.boxed(), [420.0, 240.0, 320.0, 240.0]);
    assert_eq!(w.crop(), Some([0.0, 60.0, 320.0, 240.0]));
}

/// ⭐ A5/A6 — arrastar DENTRO desliza a imagem sob a janela (aqui 1:1 com o dedo — o Excalidraw anda
/// o dobro, um defeito dele), a caixa não se mexe, e pára nas bordas naturais; uma pega puxada para
/// fora pára na borda e cobrir tudo volta a «sem recorte»; o mínimo é 10 × 10 do mundo e não vira.
#[test]
fn dragging_inside_slides_the_picture_and_everything_stops_at_its_edges() {
    let mut w = world(400.0, 300.0, |_| {});
    w.drag(SE, [-160.0, -120.0], NONE);
    let boxed = w.boxed();
    w.drag([500.0, 250.0], [-50.0, -40.0], NONE);
    assert_eq!(w.boxed(), boxed, "a caixa fica");
    assert_eq!(
        w.crop(),
        Some([50.0, 40.0, 240.0, 180.0]),
        "a imagem segue o dedo"
    );
    w.drag([500.0, 250.0], [300.0, 300.0], NONE);
    assert_eq!(w.crop(), Some([0.0, 0.0, 240.0, 180.0]), "pára na borda");
    w.drag([500.0, 250.0], [-600.0, -600.0], NONE);
    assert_eq!(w.crop(), Some([160.0, 120.0, 240.0, 180.0]));

    let mut w = world(400.0, 300.0, |_| {});
    w.drag(SE, [100.0, 80.0], NONE);
    assert_eq!(w.crop(), None, "puxada para fora: a imagem inteira");
    w.drag(SE, [-395.0, -295.0], NONE);
    assert_eq!(w.boxed()[2..], [10.0, 10.0], "o mínimo");
    let mut w = world(400.0, 300.0, |_| {});
    w.drag(SE, [-500.0, -400.0], NONE);
    assert_eq!(
        w.boxed(),
        [420.0, 180.0, 10.0, 10.0],
        "para lá do canto oposto: não vira"
    );
}

/// A10/A11 — espelhada, o recorte conta-se na imagem natural NÃO espelhada (o SE do ecrã é o lado
/// esquerdo dela); rodada 30°, as pegas seguem a rotação.
#[test]
fn a_mirrored_or_turned_picture_crops_in_its_own_frame() {
    let mut w = world(400.0, 300.0, |e| {
        e.image_mut().unwrap().flip = [true, false]
    });
    w.drag(SE, [-80.0, -60.0], NONE);
    assert_eq!(w.crop(), Some([80.0, 0.0, 320.0, 240.0]));

    let a = 30f64.to_radians();
    let mut w = world(400.0, 300.0, |e| e.angle = a);
    let (c, s) = (a.cos(), a.sin());
    // O canto SE rodado à volta do centro (620, 330), e o arrasto pelos eixos dele.
    let se = [620.0 + 200.0 * c - 150.0 * s, 330.0 + 200.0 * s + 150.0 * c];
    let by = [-80.0 * c + 60.0 * s, -80.0 * s - 60.0 * c];
    w.drag(se, by, NONE);
    let got = w.crop().unwrap();
    assert!(
        (got[2] - 320.0).abs() < 1e-6 && (got[3] - 240.0).abs() < 1e-6,
        "{got:?}"
    );
    assert!((w.el().angle - a).abs() < 1e-12);
}

/// A7/A8 — cada arrasto é UM passo de desfazer; `Enter`/`Escape` (o [`Editor::end_crop`]) e o clique
/// fora saem e o recorte fica; o clique fora larga a selecção; um duplo-clique numa imagem entra.
#[test]
fn each_drag_is_one_undo_step_and_leaving_keeps_the_crop() {
    let mut w = world(400.0, 300.0, |_| {});
    w.drag(SE, [-80.0, -60.0], NONE);
    w.drag([740.0, 420.0], [-80.0, -60.0], NONE);
    assert_eq!(w.crop(), Some([0.0, 0.0, 240.0, 180.0]));
    assert!(w.h.undo(&mut w.doc));
    assert_eq!(w.crop(), Some([0.0, 0.0, 320.0, 240.0]), "o 2.º arrasto");
    w.drag([100.0, 100.0], [0.0, 0.0], NONE);
    assert_eq!(w.ed.cropping(), None, "o clique fora sai");
    assert!(w.ed.selection.is_empty(), "e larga a selecção");
    assert_eq!(w.crop(), Some([0.0, 0.0, 320.0, 240.0]), "o recorte fica");

    let mut ts = TextSystem::without_system_fonts();
    let p = Pointer {
        world: [500.0, 250.0],
        px: 1.0,
        mods: NONE,
    };
    assert!(w.ed.double_click(&mut w.doc, &mut w.h, &mut ts, p));
    assert_eq!(
        w.ed.cropping(),
        Some(w.id),
        "o duplo-clique numa imagem recorta"
    );
    assert!(w.ed.double_click(&mut w.doc, &mut w.h, &mut ts, p));
    assert_eq!(w.ed.cropping(), Some(w.id), "e dentro do recorte não sai");
    w.ed.end_crop();
    assert_eq!(w.ed.cropping(), None);
}

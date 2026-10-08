//! ⭐ A cena 7 ENSINA o que acontece (lente de costura da W5b, 08/10: a 1.ª versão acendia primeiro
//! o centro, guias direitas e o canto de um vizinho). Cada «arraste-me» é conduzido pelo editor real,
//! com as medidas da interface (`board_view::metrics`, 8 px de encaixe), a zoom 1 e ao do enquadrar
//! (~0,4): o arrasto pára 3 antes do fim, a 25 do fim NADA o fez saltar (fora da tolerância nos dois zooms, e já depois do limiar
//! de arrasto, 10 a 2,5) e no fim fica o alinhamento que o texto promete.

use super::*;
use ph2d_board_edit::{Editor, Frame, Guide, Handle, Mods, Pointer};
use ph2d_board_model::{Element, History};
use ph2d_editor_core::screens::hero::board_view::metrics;
use ph2d_text::TextSystem;

const VIEW: [f64; 4] = [-1e4, -1e4, 1e4, 1e4];
/// Px de ecrã por unidade do mundo: zoom 1 e o do enquadrar.
const PXS: [f64; 2] = [1.0, 2.5];
/// A distância ao fim onde o arrasto ainda não pode acender nada (> 8 px × 2,5).
const QUIET: f64 = 25.0;
const R: f64 = 1e-6;
/// O arrasto PÁRA 3 antes do alinhamento: só o encaixe o leva lá (largar no sítio exacto não prova
/// nada — a prova de mutação apanhou-o: sem o espaçamento igual a lição continuava verde).
const SHORT: f64 = OFF - 3.0;

struct Run {
    ed: Editor,
    doc: BoardDoc,
    h: History,
    ts: TextSystem,
    l: Lessons,
    px: f64,
}

fn run(px: f64) -> Run {
    let mut doc = BoardDoc::default();
    let l = build(&mut doc);
    Run {
        ed: Editor::new(default_style(), metrics()),
        doc,
        h: History::default(),
        ts: TextSystem::without_system_fonts(),
        l,
        px,
    }
}

impl Run {
    fn at(&self, p: [f64; 2]) -> Pointer {
        Pointer {
            world: p,
            px: self.px,
            mods: Mods::default(),
        }
    }

    fn el(&self, id: ElementId) -> &Element {
        self.doc.get(id).expect("existe")
    }

    fn guides(&mut self) -> Vec<Guide> {
        self.ed.overlay(&mut self.doc, &mut self.ts).guides
    }

    /// Arrasta `id` de `from` por `d` em passos de 1; devolve quanto ele SALTOU (encaixou) a `QUIET`
    /// do fim e as guias no fim.
    fn drag(&mut self, id: ElementId, from: [f64; 2], d: [f64; 2]) -> (f64, Vec<Guide>) {
        let start = [self.el(id).x, self.el(id).y];
        let n = d[0].hypot(d[1]).round() as usize;
        let at_step = |k: usize| {
            let t = k as f64 / n as f64;
            [from[0] + d[0] * t, from[1] + d[1] * t]
        };
        let p0 = self.at(from);
        self.ed
            .pointer_down(&mut self.doc, &mut self.h, &mut self.ts, p0, VIEW);
        let mut jump = f64::NAN;
        for k in 1..=n {
            let p = self.at(at_step(k));
            self.ed.pointer_move(&mut self.doc, &mut self.ts, p);
            if k == n - QUIET as usize {
                let t = k as f64 / n as f64;
                let el = self.el(id);
                jump = (el.x - start[0] - d[0] * t).hypot(el.y - start[1] - d[1] * t);
            }
        }
        let end = self.guides();
        let p = self.at(at_step(n));
        self.ed.pointer_up(&mut self.doc, &mut self.h, p);
        (jump, end)
    }

    /// Roda `id` com o ponteiro no raio da pega até `deg` graus a partir dela.
    fn rotate(&mut self, id: ElementId, deg: f64) {
        self.ed.select(&self.doc, [id]);
        let c = self.el(id).center();
        let knob = Frame::of(self.el(id)).handle(Handle::Rotate, self.px, &metrics());
        let r = (knob[0] - c[0]).hypot(knob[1] - c[1]);
        let a0 = (knob[1] - c[1]).atan2(knob[0] - c[0]);
        let p = |d: f64| {
            let a = a0 + d.to_radians();
            [c[0] + r * a.cos(), c[1] + r * a.sin()]
        };
        let p0 = self.at(knob);
        self.ed
            .pointer_down(&mut self.doc, &mut self.h, &mut self.ts, p0, VIEW);
        for k in 1..=10 {
            let q = self.at(p(deg * f64::from(k) / 10.0));
            self.ed.pointer_move(&mut self.doc, &mut self.ts, q);
        }
        let q = self.at(p(deg));
        self.ed.pointer_up(&mut self.doc, &mut self.h, q);
    }
}

/// A caixa de `el` nos eixos de `deg` graus: `[u0, v0, u1, v1]`.
fn along(el: &Element, deg: f64) -> [f64; 4] {
    let (s, k) = deg.to_radians().sin_cos();
    el.corners().iter().fold(
        [
            f64::INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NEG_INFINITY,
        ],
        |b, p| {
            let (u, v) = (p[0] * k + p[1] * s, p[1] * k - p[0] * s);
            [b[0].min(u), b[1].min(v), b[2].max(u), b[3].max(v)]
        },
    )
}

fn runs_along(g: &Guide, deg: f64) -> bool {
    let d = [g.b[0] - g.a[0], g.b[1] - g.a[1]];
    let (s, k) = deg.to_radians().sin_cos();
    (d[0] * s - d[1] * k).abs() < 1e-6 * d[0].hypot(d[1]).max(1.0)
}

#[test]
fn every_lesson_teaches_its_own_alignment() {
    for px in PXS {
        // 1 · B desliza ao longo dos 30°: a borda esquerda dele na linha inclinada da de A.
        let mut w = run(px);
        let (a, b) = (w.l.tilt_a, w.l.tilt_b);
        let back = local([0.0, 0.0], TILT, -SHORT, 0.0);
        let (jump, end) = w.drag(b, w.el(b).center(), back);
        assert!(jump.abs() < R, "px {px}: saltou {jump} antes da lição");
        assert!(
            (along(w.el(b), TILT)[0] - along(w.el(a), TILT)[0]).abs() < R,
            "px {px}"
        );
        assert!(
            end.iter().any(|g| runs_along(g, TILT + 90.0)),
            "px {px}: guia inclinada"
        );

        // 3a · O 3.º da fila do frame rodado: o vão 80 vai a 40, nos eixos do frame.
        let mut w = run(px);
        let [_, k2, k3] = w.l.row;
        let back = local([0.0, 0.0], FRAME_TILT, -SHORT, 0.0);
        let (jump, _) = w.drag(k3, w.el(k3).center(), back);
        assert!(jump.abs() < R, "px {px}: saltou {jump} antes da lição");
        let gap = along(w.el(k3), FRAME_TILT)[0] - along(w.el(k2), FRAME_TILT)[2];
        assert!((gap - 40.0).abs() < R, "px {px}: vão {gap}");

        // 3b · O de baixo: a esquerda vai à do 1.º, nos eixos do frame.
        let mut w = run(px);
        let [k1, k4] = w.l.align;
        let fwd = local([0.0, 0.0], FRAME_TILT, SHORT, 0.0);
        let (jump, _) = w.drag(k4, w.el(k4).center(), fwd);
        assert!(jump.abs() < R, "px {px}: saltou {jump} antes da lição");
        let d = along(w.el(k4), FRAME_TILT)[0] - along(w.el(k1), FRAME_TILT)[0];
        assert!(d.abs() < R, "px {px}: {d}");

        // 4 · O 3.º da fila direita: o vão 100 vai a 60.
        let mut w = run(px);
        let c = w.l.spaced[2];
        let (jump, _) = w.drag(c, w.el(c).center(), [-SHORT, 0.0]);
        assert!(jump.abs() < R, "px {px}: saltou {jump} antes da lição");
        assert!((w.el(c).x - 1910.0).abs() < R, "px {px}: {}", w.el(c).x);

        // 5 · O Frame B, agarrado pelo miolo vazio, sobe até o topo ser o do A.
        let mut w = run(px);
        let [fa, fb] = w.l.frames;
        let grab = [w.el(fb).x + 50.0, w.el(fb).y + 400.0];
        let (jump, _) = w.drag(fb, grab, [0.0, -SHORT]);
        assert!(jump.abs() < R, "px {px}: saltou {jump} antes da lição");
        assert!(
            (w.el(fb).y - w.el(fa).y).abs() < R,
            "px {px}: {}",
            w.el(fb).y
        );

        // 2 · Rodar: 28° cola em 30°, 43° em 45°, 2° volta a direito (o arco do canto: 163 × 2° = 5,7 px).
        for (deg, want) in [(28.0, 30.0), (43.0, 45.0), (2.0, 0.0)] {
            let mut w = run(px);
            let id = w.l.rotate;
            w.rotate(id, deg);
            let got = w.el(id).angle.to_degrees();
            assert!((got - want).abs() < R, "px {px}: {deg}° foi a {got}°");
        }
    }
}

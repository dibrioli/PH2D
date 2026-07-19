//! A cena de smoke do **Expand** (Outline Stroke + Offset Path) — `PH2D_BUILD_SMOKE=17`.
//! Módulo irmão de `build_smoke` (teto de 600 LOC).
//!
//! A cena responde às quatro perguntas que o Expand faz, uma forma para cada:
//!
//! 1. **um traço puro** (sem preenchimento) — Outline Stroke o consome e devolve a forma;
//! 2. **traço + preenchimento** — Outline Stroke tem de deixar DOIS objetos, e o miolo fica
//!    com a cor dele;
//! 3. **um donut** (compound) — Offset Path tem de crescer a borda e ENCOLHER o furo, que é
//!    a metade que uma implementação ingênua erra em silêncio;
//! 4. **um arco aberto** — Power Stroke, a largura variando ao longo dele (a caligrafia).

use crate::build_smoke::shape;
use ph2d_vec_scene::{Contour, Rgba8, ShapeKind, StrokeSpec, VecVertex};

/// Um quadrado de lado `s` centrado em `c`, em sentido CCW.
fn square_at(c: [f64; 2], s: f64) -> Vec<VecVertex> {
    let h = s * 0.5;
    [
        [c[0] - h, c[1] - h],
        [c[0] + h, c[1] - h],
        [c[0] + h, c[1] + h],
        [c[0] - h, c[1] + h],
    ]
    .map(VecVertex::corner)
    .to_vec()
}

impl crate::App {
    /// Frame 3: monta as quatro formas e entra no modo Select (é nele que o artista escolhe o
    /// que vai converter — os comandos agem sobre a SELEÇÃO).
    pub(crate) fn smoke_expand_build(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("vector"));
        let scene = &mut gfx.vec_scene;

        // (1) Um ZIG-ZAG aberto, SÓ traço, grosso o bastante para o contorno ser visível.
        let zig = scene.push_path(ph2d_vec_scene::VecPath {
            verts: [
                [-4.2, -1.2],
                [-3.4, 1.2],
                [-2.6, -1.2],
                [-1.8, 1.2],
                [-1.0, -1.2],
            ]
            .map(VecVertex::corner)
            .to_vec(),
            closed: false,
            ..ph2d_vec_scene::VecPath::default()
        });
        if let Some(p) = scene.path_mut(zig) {
            let mut s = StrokeSpec::new(Rgba8::new(230, 90, 60, 255), 0.3);
            s.cap = ph2d_vec_scene::LineCap::Round;
            s.join = ph2d_vec_scene::LineJoin::Round;
            p.stroke = Some(s);
        }

        // (2) Uma estrela com preenchimento E traço — o caso dos DOIS objetos.
        let star = scene.push_path(shape(
            ShapeKind::Star,
            [-0.4, -1.2],
            [2.0, 1.2],
            &[5.0, 0.45, 0.0],
            [80, 140, 210],
        ));
        if let Some(p) = scene.path_mut(star) {
            p.stroke = Some(StrokeSpec::new(Rgba8::new(240, 200, 60, 255), 0.22));
        }

        // (3) Um DONUT (compound, parede fina) — o Offset tem de crescer a borda e encolher
        // o furo ao mesmo tempo.
        let donut = scene.push_path(shape(
            ShapeKind::Rectangle,
            [2.8, -1.2],
            [5.2, 1.2],
            &[],
            [120, 190, 120],
        ));
        if let Some(p) = scene.path_mut(donut) {
            p.subpaths = vec![Contour::new_closed(square_at([4.0, 0.0], 1.4))];
            p.fill_rule = ph2d_vec_scene::FillRule::EvenOdd;
        }

        // (4) Um arco aberto, só traço — o caso do Power Stroke (a caligrafia).
        let arc = scene.push_path(ph2d_vec_scene::VecPath {
            verts: [
                ([-4.0, -2.6], [-4.0, -2.6], [-2.4, -3.6]),
                ([0.0, -2.4], [-1.6, -3.4], [1.6, -3.4]),
                ([4.0, -2.6], [2.4, -3.6], [4.0, -2.6]),
            ]
            .map(|(a, i, o)| ph2d_vec_scene::VecVertex {
                anchor: a,
                in_handle: i,
                out_handle: o,
                ..ph2d_vec_scene::VecVertex::corner(a)
            })
            .to_vec(),
            closed: false,
            ..ph2d_vec_scene::VecPath::default()
        });
        if let Some(p) = scene.path_mut(arc) {
            p.stroke = Some(StrokeSpec::new(Rgba8::new(150, 110, 220, 255), 0.35));
        }

        self.vec_set_draw_mode(ph2d_tool_vector::DrawMode::Select);
    }

    /// Frame 4 (pós-`settle`): seleciona o zig-zag e imprime o roteiro.
    ///
    /// Seleciona UMA forma, não as três: o 1º gesto do smoke é o Outline Stroke, e ele sobre
    /// o donut (que não tem traço) não faria nada — o artista leria "não funciona".
    pub(crate) fn smoke_expand_select(&mut self) {
        let first = self
            .gfx
            .as_ref()
            .expect("gfx")
            .vec_scene
            .paths()
            .first()
            .map(|p| p.id);
        if let Some(id) = first {
            self.vec_pen.select(Some(id));
        }
        eprintln!(
            "[smoke] EXPAND — a seção **Expand** no painel (abaixo de Boolean).\n\
             \x20 1) O zig-zag laranja (já selecionado) é SÓ traço: clique **Outline Stroke**.\n\
             \x20    Ele vira uma FORMA preenchida — com as pontas redondas assadas. Entre no\n\
             \x20    modo Node e veja as âncoras: é geometria, não mais estilo.\n\
             \x20 2) Selecione a ESTRELA (tem traço amarelo + miolo azul) e clique Outline\n\
             \x20    Stroke: têm de sobrar DOIS objetos — o miolo azul e o anel amarelo.\n\
             \x20 3) Selecione o DONUT verde, ponha **Offset** em ~+0.4 e clique **Offset\n\
             \x20    Path**: a borda cresce e o FURO encolhe. Depois experimente negativo, e\n\
             \x20    troque **Join** para Round (a quina vira arco) e Bevel (corte reto).\n\
             \x20 4) O ARCO roxo embaixo é para o **Power Stroke**: selecione-o e clique.\n\
             \x20    A linha afina nas pontas e engrossa no meio (o perfil default). Mexa em\n\
             \x20    **W Start / W Mid / W End** e refaça — `W Pos` move onde o grosso senta.\n\
             \x20    Com os três em 1.00 o botão não faz nada de propósito: aí é Outline Stroke.\n\
             \x20 5) Ctrl+Z uma vez desfaz o comando INTEIRO."
        );
    }
}

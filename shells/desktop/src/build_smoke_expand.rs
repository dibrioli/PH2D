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
use std::cell::Cell;

thread_local! {
    /// Onde o nível 18 agarrou o slider (tela) — o arrasto move a partir daqui.
    static GRAB: Cell<(f32, f32)> = const { Cell::new((0.0, 0.0)) };
    /// O instante do frame anterior — mede o CUSTO de frame do roteiro (a queda de FPS).
    static LAST_T: Cell<Option<std::time::Instant>> = const { Cell::new(None) };
}

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
             \x20 3) OFFSET AO VIVO: selecione o DONUT verde e ARRASTE o slider **Offset** —\n\
             \x20    a forma muda em TEMPO REAL; ao soltar, o slider volta ao centro (0).\n\
             \x20    - **Side**: Both expande a borda E o furo; Outer só a borda; Inner só o\n\
             \x20      furo. Com **Inner** + **Join Round**, arraste POSITIVO: as quinas do\n\
             \x20      FURO arredondam (o bug do smoke passado). Negativo encolhe.\n\
             \x20    - O botão **Offset Path** ainda aplica o valor digitado no chip.\n\
             \x20 4) O ARCO roxo embaixo é o **Power Stroke** (agora LISO, sem rugosidade):\n\
             \x20    selecione-o e clique. Afina nas pontas e engrossa no meio. Mexa em\n\
             \x20    **W Start / W Mid / W End** e refaça — `W Pos` move onde o grosso senta.\n\
             \x20    Com os três em 1.00 o botão não faz nada de propósito: aí é Outline Stroke.\n\
             \x20 5) Ctrl+Z uma vez desfaz o comando INTEIRO."
        );
    }

    /// Nível 18 — **o roteiro do RETUNE, auto-dirigido pelo input real** (a ferramenta que
    /// decodificou o report de 2026-07-20: "queda de FPS + não muda para Miter/Bevel").
    /// Na ORDEM do report: arrasta com o Miter default (segura ~2 s por fase, pra dar
    /// tempo de screenshot), solta, e retuna Round → Bevel → Miter com cliques de timing
    /// REAL (Down e Up em frames separados). Loga por frame: custo, profundidade do undo,
    /// janela viva, join do painel, VERTS (o oráculo dos retunes) e a LARGURA do bbox
    /// (o oráculo do arrasto — verts é CEGO ao Miter/Bevel, cuja topologia não muda
    /// com `d`).
    pub(crate) fn smoke_expand_retune_drive(&mut self, f: u32) {
        // Telemetria de TODO frame do roteiro (o FPS é o 1º sintoma).
        let dt_ms = LAST_T.with(|c| {
            let now = std::time::Instant::now();
            let dt = c.get().map_or(0.0, |t| t.elapsed().as_secs_f64() * 1e3);
            c.set(Some(now));
            dt
        });
        if (9..=520).contains(&f) && (f.is_multiple_of(10) || f <= 30) {
            // VERTS, não área: a área a `Both` é CEGA por construção (o arredondamento
            // perde (4−π)d² na borda e ganha o MESMO no furo — cancela exato).
            // ⚠️ E BBOX, porque verts é CEGO ao Miter/Bevel: a topologia deles não muda
            // com `d` (um quadrado mitrado tem 4 quinas em qualquer offset), então só o
            // Round "mexe" na contagem — um preview CONGELADO em Miter seria verde de
            // verts. A largura do bbox cresce com `d` em qualquer join vivo.
            let (paths, verts, bw) = self.gfx.as_ref().map_or((0, 0, 0.0), |g| {
                let v: usize = g
                    .vec_scene
                    .paths()
                    .iter()
                    .map(|p| {
                        p.verts.len() + p.subpaths.iter().map(|c| c.verts.len()).sum::<usize>()
                    })
                    .sum();
                let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
                for p in g.vec_scene.paths() {
                    for vx in &p.verts {
                        lo = lo.min(vx.anchor[0]);
                        hi = hi.max(vx.anchor[0]);
                    }
                }
                (g.vec_scene.paths().len(), v, (hi - lo).max(0.0))
            });
            let d = self.gfx.as_ref().and_then(|g| {
                let hero = g.hero_screen.as_ref()?;
                let (_, v) = hero.store.slider(ph2d_editor::ids::VECTOR_EXPAND_OFFSET)?;
                Some(ph2d_tool_vector::params::slider_to_offset(v))
            });
            let active = self
                .gfx
                .as_ref()
                .and_then(|g| g.hero_screen.as_ref())
                .and_then(|h| h.store.active_id())
                .map_or("-".into(), |id| format!("{id:?}"));
            eprintln!(
                "[retune-smoke] f={f} dt={dt_ms:.1}ms undo={} win={} join={} d={} paths={paths} verts={verts} bw={bw:.3} active={active}",
                self.undo.depth(),
                u8::from(self.vec_offset_retune.is_some()),
                ph2d_panel_vector::expand_join(),
                d.map_or("?".into(), |d| format!("{d:.3}")),
            );
        }
        match f {
            // A seção Expand mora abaixo da dobra: ROLA o painel (roda, caminho real) até o
            // slider entrar no hit-index. O cursor precisa estar SOBRE o painel para a roda
            // ser dele — o painel docado fica na borda direita.
            5..=7
                if self
                    .smoke_find_widget(ph2d_editor::ids::VECTOR_EXPAND_OFFSET)
                    .is_none() =>
            {
                let win = self.gfx.as_ref().map(|g| g.surface.size());
                if let Some(w) = win {
                    let (px, py) = (w.width as f32 - 120.0, w.height as f32 * 0.5);
                    self.smoke_pointer_move(px, py);
                    self.on_mouse_wheel(winit::event::MouseScrollDelta::LineDelta(0.0, -24.0));
                }
            }
            // SEM pré-clique de join — o arrasto sai com o MITER default, que é o fluxo do
            // report ("muda em tempo real para round mas não muda para Miter e Bevel"): o
            // Round dele é o 1º RETUNE, não o join do arrasto.
            // Agarra o slider de Offset no centro (d=0) e segura.
            10 => match self.smoke_find_widget(ph2d_editor::ids::VECTOR_EXPAND_OFFSET) {
                Some((x, y)) => {
                    GRAB.with(|c| c.set((x, y)));
                    eprintln!("[retune-smoke] slider em ({x}, {y}) — DOWN");
                    self.smoke_pointer_down(x, y);
                }
                None => eprintln!("[retune-smoke] slider FORA do hit-index — roteiro morto"),
            },
            // Arrasta 68 px para a direita, 4 px/frame — o grab caiu na BORDA esquerda do
            // track (d≈−3.9), então 68 px param em d≈+1.3: um offset MODERADO, com as
            // quinas do resultado DENTRO do viewport. ⚠️ De propósito: a d≈4 a forma
            // estoura a tela e as quinas — onde o join mora — saem de vista, então NENHUM
            // retune muda um pixel visível (foi metade do report de 2026-07-20); o regime
            // extremo é coberto pelo gate determinístico do motor
            // (`an_offset_past_the_shapes_death_leaves_no_phantom`), não por este smoke.
            11..=27 => {
                let (x, y) = GRAB.with(Cell::get);
                self.smoke_pointer_move(x + ((f - 10) * 4) as f32, y);
            }
            // Segura o arrasto parado até f=115 (~2 s) — janela para o screenshot do
            // preview VIVO; solta em f=116. ⚠️ A posição é REAFIRMADA todo frame: o
            // desktop é vivo e o cursor FÍSICO também fala — o KWin reposiciona a janela
            // recém-aberta sob o cursor parado e isso emite `CursorMoved` REAIS, que o
            // slider ativo obedece (um hold sem re-assert já foi teleportado a d=−4 pelo
            // cursor físico em x≈400, e a investigação perseguiu um fantasma de app que
            // era do AMBIENTE).
            28..=115 => {
                let (x, y) = GRAB.with(Cell::get);
                self.smoke_pointer_move(x + 68.0, y);
            }
            116 => {
                eprintln!("[retune-smoke] UP (release — a janela de retune abre aqui)");
                self.smoke_pointer_up();
            }
            // Retunes 1..3 na ordem do report: Miter→ROUND (o único que o report diz
            // FUNCIONAR — verts sobem, quinas → arcos), Round→BEVEL (o report diz que NÃO
            // muda — verts têm de despencar) e Bevel→MITER (o outro que "não muda").
            //
            // ⚠️ Cada clique é Down num frame e Up TRÊS frames depois — o timing de um
            // mouse REAL. `smoke_click_screen` (down+up no MESMO frame) não contém a
            // corrida que um clique humano contém: entre o Down e o Up o `held_button`
            // suprime o diff do undo, e a janela de retune aprende a profundidade por
            // frame — um passo que registre um frame "tarde" a mataria em silêncio.
            170 | 290 | 410 => {
                let (id, name) = match f {
                    170 => (ph2d_editor::ids::VECTOR_EXPAND_JOIN_ROUND, "ROUND"),
                    290 => (ph2d_editor::ids::VECTOR_EXPAND_JOIN_BEVEL, "BEVEL"),
                    _ => (ph2d_editor::ids::VECTOR_EXPAND_JOIN_MITER, "MITER"),
                };
                match self.smoke_find_widget(id) {
                    Some((x, y)) => {
                        eprintln!("[retune-smoke] DOWN {name} em ({x}, {y})");
                        self.smoke_pointer_down(x, y);
                    }
                    None => eprintln!("[retune-smoke] chip {name} fora do hit-index"),
                }
            }
            173 | 293 | 413 => {
                eprintln!("[retune-smoke] UP do chip");
                self.smoke_pointer_up();
            }
            520 => eprintln!("[retune-smoke] fim do roteiro — feche a janela"),
            _ => {}
        }
    }
}

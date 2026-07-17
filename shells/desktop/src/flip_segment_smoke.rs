//! **A cena pronta para o smoke do domínio SEGMENT** (`PH2D_FLIP_SEGMENT_SMOKE=1`, §4.B).
//!
//! O Enio não monta cena ([[feedback_ready_to_smoke_example]]): o app abre com a tool Flip
//! no modo **Edit**, domínio **Segment** já armado, e quatro alvos — um para cada coisa que
//! o modo promete e que os gates não conseguem mostrar na tela.
//!
//! Roteiro (clicar, em cada um):
//!
//! 1. **O X (duas linhas que se cruzam)** — clicar num braço acende SÓ aquele braço, do
//!    cruzamento até a ponta. É a promessa central: o traço vizinho é a tesoura.
//! 2. **O triângulo intacto** — nada o cruza, então clicar em qualquer aresta acende a
//!    forma INTEIRA (o *fallback* do `§11`; é o caso comum do balde, que produz traço
//!    fechado). Sem ele, um clique aqui não acenderia nada.
//! 3. **O quadrado cortado por uma linha de OUTRA CAMADA** — duas coisas de uma vez:
//!    (a) o corte é do **QUADRO**, não do desenho ativo (a linha vermelha vive na camada
//!    "Cutter" e mesmo assim corta); (b) o pedaço da ESQUERDA **enrola na costura** — ele
//!    é um pedaço só, mas atravessa o ponto onde a polilinha fecha (os *"dois ranges"* do
//!    `§11`). Clicar na aresta esquerda tem de acender a quina de baixo E a de cima.
//! 4. **A curva cortada duas vezes** — o caso real (a caneta produz polilinha densa, não
//!    polígono): três pedaços, e clicar no do meio acende só o meio.
//!
//! Confira também: **arrastar** um pedaço o move (o gesto é o do domínio Point — o dado é o
//! mesmo `point_sel`); **Shift+clique** soma pedaços; a **caixa de seleção** acende o pedaço
//! INTEIRO que ela tocou (não recorta na borda dela); trocar para **Point** ou **Stroke** e
//! voltar — Point↔Segment preserva a seleção (mesmo dado), Stroke a promove/limpa.

use ph2d_core::Vec2;
use ph2d_flip::{FlipStroke, Hold, KeyKind, Point, Rgba};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering};

static FRAME: AtomicU32 = AtomicU32::new(0);

fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("PH2D_FLIP_SEGMENT_SMOKE").is_some())
}

/// Uma polilinha pelos vértices dados, largura de tela grossa (fácil de acertar).
fn stroke(verts: &[Vec2], color: Rgba, closed: bool) -> FlipStroke {
    let mut s = FlipStroke::new();
    for &p in verts {
        s.push_point(Point {
            pos: p,
            width: 6.0,
            opacity: 1.0,
            color,
        });
    }
    s.closed = closed;
    s
}

/// Uma cúbica amostrada em 17 pontos — a polilinha DENSA que a caneta produz (de Casteljau
/// à mão; sem transcendental, HR-5).
fn curve(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2) -> Vec<Vec2> {
    (0..17)
        .map(|i| {
            let t = i as f32 / 16.0;
            let u = 1.0 - t;
            let (a, b, c, d) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
            Vec2::new(
                a * p0.x + b * p1.x + c * p2.x + d * p3.x,
                a * p0.y + b * p1.y + c * p2.y + d * p3.y,
            )
        })
        .collect()
}

const INK: Rgba = Rgba::new(0.9, 0.9, 0.95, 1.0);
const CUT: Rgba = Rgba::new(0.9, 0.25, 0.2, 1.0);

impl crate::App {
    /// Roda no prólogo do frame (ao lado dos outros smokes). No-op sem a env.
    pub(crate) fn flip_segment_smoke(&mut self) {
        if !enabled() || self.gfx.is_none() {
            return;
        }
        match FRAME.fetch_add(1, Ordering::Relaxed) {
            3 => {
                let gfx = self.gfx.as_mut().expect("gfx");
                let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("flip"));
                let oid = gfx.flip.push_object("Segment Smoke");
                let obj = gfx.flip.object_mut(oid).expect("objeto recém-criado");
                obj.fps = 12.0;

                // ── A camada da ARTE (a ativa: é nela que se clica). ──
                let art = obj.add_layer("Art");
                if let Some(d) = obj.insert_frame(art, 0, Hold::Implicit, KeyKind::Keyframe) {
                    let dr = obj.drawing_mut(d).expect("desenho");
                    // (1) O X: duas linhas que se cruzam em (-2.5, 0.95).
                    dr.strokes.push(stroke(
                        &[Vec2::new(-3.5, 0.3), Vec2::new(-1.5, 1.6)],
                        INK,
                        false,
                    ));
                    dr.strokes.push(stroke(
                        &[Vec2::new(-3.5, 1.6), Vec2::new(-1.5, 0.3)],
                        INK,
                        false,
                    ));
                    // (2) O triângulo INTACTO — nada o cruza (o fallback).
                    dr.strokes.push(stroke(
                        &[
                            Vec2::new(1.5, 0.3),
                            Vec2::new(3.2, 0.3),
                            Vec2::new(2.35, 1.7),
                        ],
                        INK,
                        true,
                    ));
                    // (3) O quadrado — cortado por uma linha de OUTRA camada (abaixo). A
                    //     costura é a aresta ESQUERDA (o último ponto liga ao primeiro).
                    dr.strokes.push(stroke(
                        &[
                            Vec2::new(-3.5, -1.7),
                            Vec2::new(-1.5, -1.7),
                            Vec2::new(-1.5, -0.4),
                            Vec2::new(-3.5, -0.4),
                        ],
                        INK,
                        true,
                    ));
                    // (4) A CURVA densa, cortada 2× por duas verticais (na mesma camada).
                    dr.strokes.push(stroke(
                        &curve(
                            Vec2::new(1.3, -1.8),
                            Vec2::new(2.6, -1.9),
                            Vec2::new(1.9, -0.5),
                            Vec2::new(3.3, -0.5),
                        ),
                        INK,
                        false,
                    ));
                    dr.strokes.push(stroke(
                        &[Vec2::new(1.8, -2.0), Vec2::new(1.8, -0.2)],
                        CUT,
                        false,
                    ));
                    dr.strokes.push(stroke(
                        &[Vec2::new(2.8, -2.0), Vec2::new(2.8, -0.2)],
                        CUT,
                        false,
                    ));
                }

                // ── A camada CUTTER: a tesoura do quadrado mora FORA do desenho ativo. ──
                let cutter = obj.add_layer("Cutter");
                if let Some(d) = obj.insert_frame(cutter, 0, Hold::Implicit, KeyKind::Keyframe) {
                    let dr = obj.drawing_mut(d).expect("desenho");
                    // Vertical que cruza a base E o topo do quadrado ⇒ 2 cortes ⇒ o pedaço
                    // da esquerda ENROLA na costura.
                    dr.strokes.push(stroke(
                        &[Vec2::new(-2.5, -2.0), Vec2::new(-2.5, -0.1)],
                        CUT,
                        false,
                    ));
                }
                // A camada ATIVA é a da arte (a `Cutter` é só tesoura).
                self.flip_active_layer = Some(art);
                self.playhead.pause();
            }
            // Entra no Edit e arma o domínio Segment pelas portas REAIS (os mesmos eventos
            // dos pills do painel) — um smoke que semeasse o estado à mão não provaria que
            // o pill está fiado.
            8 => {
                if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
                    for id in [
                        ph2d_editor::ids::FLIP_MODE_EDIT,
                        ph2d_editor::ids::FLIP_EDIT_DOM_SEGMENT,
                    ] {
                        hero.bus
                            .push(ph2d_editor::action_bus::EditorAction::ToolPanelEvent(
                                ph2d_editor::tool::PanelEvent::Click(id),
                            ));
                    }
                }
                eprintln!(
                    "[segment-smoke] modo Edit, dominio SEGMENT armado. Quatro alvos:\n  \
                     (1) o X (cima-esq): clicar num braco acende SO aquele braco, do \
                     cruzamento ate a ponta.\n  \
                     (2) o triangulo (cima-dir): NADA o cruza, entao clicar em qualquer \
                     aresta acende a forma INTEIRA (o fallback -- o caso do balde).\n  \
                     (3) o quadrado (baixo-esq): a linha VERMELHA que o corta esta em OUTRA \
                     CAMADA (Cutter) e mesmo assim corta -- o corte e do QUADRO. E o pedaco \
                     da ESQUERDA ENROLA na costura: clicar na aresta esquerda acende a quina \
                     de baixo E a de cima (um pedaco so, dois trechos).\n  \
                     (4) a CURVA (baixo-dir): densa, cortada 2x -- tres pedacos; clicar no \
                     do meio acende so o meio.\n  \
                     Confira ainda: arrastar um pedaco o MOVE; Shift+clique SOMA pedacos; a \
                     caixa de selecao acende o pedaco INTEIRO que tocou (nao recorta na \
                     borda dela); Point<->Segment preserva a selecao (mesmo dado), Stroke \
                     promove/limpa."
                );
            }
            9 => self.any_input_this_frame = true, // arma o baseline do undo
            // O pill REALMENTE armou? Despachar o evento prova que o bus o aceitou; o que
            // interessa é o que a tool ficou sendo. Um pill pintado e inerte é o bug nº 1
            // do projeto, e ele passa por todo gate de compilação.
            12 => eprintln!(
                "[segment-smoke] dominio resolvido na tool: {:?} (tem de ser Segment)",
                self.flip_edit_domain_now()
            ),
            _ => {}
        }
    }
}

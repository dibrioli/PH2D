//! **A metade que precisa da `App`** do `segment_smoke` (W2/L5, 2026-09-11).
//!
//! O resto do módulo vive em [`ph2d_app_flip::segment_smoke`] — as leis e a cena, que não
//! precisam da shell. Aqui fica só o que toca os agregados DELA: `AppGfx` (o `FlipDoc`
//! vivo, o registo de ferramentas), o `HeroScreen` (o barramento dos painéis) e o
//! relógio. É a lista que o substrato da `ph2d-app-host` tem de cobrir para isto
//! também sair (W2 Fase B).

#[allow(unused_imports)]
use ph2d_app_flip::segment_smoke::*;
use ph2d_core::Vec2;
use ph2d_flip::{Hold, KeyKind};
use std::sync::atomic::Ordering;

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
                self.flip_state.active_layer = Some(art);
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
                     HOVER (§4.C): passe o mouse SEM clicar -- o pedaco sob o cursor acende \
                     em ambar FRACO (a promessa do clique); ao clicar, ele fica ambar SOLIDO \
                     (SO o pedaco, nunca o traco inteiro). Mova entre os pedacos do quadrado \
                     e da curva e veja o preview seguir o cursor.\n  \
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

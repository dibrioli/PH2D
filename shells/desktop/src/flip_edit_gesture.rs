//! ADR-0114 W6.1 — **os gestos do Edit Mode**: o marquee e o mover.
//!
//! O clique já selecionava (`flip_select`). Faltavam os dois gestos sem os quais um Edit
//! Mode é meio modo:
//!
//! - **Marquee** (box-select): arrastar no vazio pega tudo que a caixa toca. Num desenho
//!   cheio, selecionar traço a traço é penoso — e é o gesto que todo editor 2D tem.
//! - **Mover**: arrastar um traço SELECIONADO translada a seleção inteira.
//!
//! ## Duas decisões que moram aqui
//!
//! **1. Arrastar um traço NÃO-selecionado seleciona-o e já o move** (Illustrator, Blender
//! Edit Mode). O contrário — exigir clicar, soltar, e clicar de novo para arrastar — é a
//! ergonomia que faz o usuário achar que a ferramenta não responde.
//!
//! **2. Mover translada os PONTOS e os BURACOS.** Um preenchimento carrega os furos dele
//! em anéis próprios (`FlipStroke::holes`, o "O"): mover só os pontos deixaria os furos
//! para trás, e a forma se quebraria. É a MESMA regra que o Sculpt já obedece (a lição do
//! Suzanne: *a cor anda com a linha*), e é o tipo de esquecimento que só aparece no desenho
//! do usuário, meses depois.

use ph2d_core::Vec2;
use ph2d_flip::FlipDrawing;
use ph2d_vec_scene::Xform;

/// O gesto em curso no modo Edit.
#[derive(Clone, Copy, Debug)]
pub(crate) enum EditGesture {
    /// Caixa de seleção: `start`/`cur` em px de TELA (é onde ela é desenhada e onde o
    /// usuário a enxerga; a conversão para local só acontece no fim, no teste de hit).
    Marquee {
        start: (f32, f32),
        cur: (f32, f32),
        /// Shift no início: a caixa SOMA à seleção em vez de substituí-la.
        additive: bool,
    },
    /// Translação da seleção. `last` é o cursor no espaço LOCAL do objeto (a geometria é
    /// local — ADR-0111), e o delta de cada quadro é `agora − last`.
    ///
    /// `collapse_to` é o **colapso ADIADO**: o traço em que a seleção deve virar se o
    /// usuário soltar **sem arrastar** (`flip_select::plan_down`). Ele nasce preenchido
    /// quando o clique caiu num traço que JÁ estava selecionado — porque nesse caso o
    /// gesto é ambíguo (colapsar a seleção × arrastar o grupo), e só o que vem DEPOIS do
    /// down desempata. O 1º arrasto que passa do slop o zera: virou arrasto de grupo.
    ///
    /// Sem isto, pegar um traço de uma multisseleção para arrastá-la **destruía a
    /// multisseleção no instante do toque**, e o arrasto levava um traço só (smoke do
    /// Enio, 2026-07-13).
    Move {
        last: Vec2,
        down: (f32, f32),
        collapse_to: Option<usize>,
    },
    /// **Um clique que já se resolveu no Down** (o Shift+clique, que alterna e não
    /// arrasta). Existe só para o pen-UP ter o que CONSUMIR.
    ///
    /// Sem ele, o UP do Shift+clique não era consumido e caía no **picker de OBJETO** do
    /// editor — que, com Shift, alterna o objeto Flip na multisseleção de objetos. No modo
    /// Edit o canvas é da ferramenta: nenhum clique dele pode chegar lá (é a mesma razão
    /// pela qual o Down consome até quando erra o traço).
    Click,
}

/// A caixa do marquee, em px de tela (normalizada: min/max).
#[must_use]
pub(crate) fn marquee_rect(start: (f32, f32), cur: (f32, f32)) -> (f32, f32, f32, f32) {
    (
        start.0.min(cur.0),
        start.1.min(cur.1),
        start.0.max(cur.0),
        start.1.max(cur.1),
    )
}

/// Um arrasto curto demais é um CLIQUE, não um marquee (o dedo treme). Em px de tela.
const DRAG_SLOP_PX: f32 = 3.0; // LITERAL-PX-OK: slop de gesto, nao metrica de design

/// O arrasto andou o bastante para ser um gesto (e não um clique trêmulo)?
#[must_use]
pub(crate) fn passed_slop(start: (f32, f32), cur: (f32, f32)) -> bool {
    let (dx, dy) = (cur.0 - start.0, cur.1 - start.1);
    (dx * dx + dy * dy).sqrt() > DRAG_SLOP_PX
}

/// O traço toca o retângulo? (Tudo no MESMO espaço — o chamador converte.)
///
/// Não basta "algum ponto dentro": uma reta longa pode ATRAVESSAR a caixa sem ter um
/// vértice nela, e o usuário que desenhou a caixa em cima dela espera pegá-la. Então o
/// teste é ponto-dentro **ou** segmento-cruza. (Um traço de 1 ponto cai no primeiro.)
#[must_use]
pub(crate) fn stroke_touches_rect(pts: &[Vec2], min: Vec2, max: Vec2) -> bool {
    let inside = |p: Vec2| p.x >= min.x && p.x <= max.x && p.y >= min.y && p.y <= max.y;
    if pts.iter().any(|p| inside(*p)) {
        return true;
    }
    // Segmento × as quatro bordas da caixa.
    let corners = [
        Vec2::new(min.x, min.y),
        Vec2::new(max.x, min.y),
        Vec2::new(max.x, max.y),
        Vec2::new(min.x, max.y),
    ];
    pts.windows(2)
        .any(|w| (0..4).any(|i| segments_cross(w[0], w[1], corners[i], corners[(i + 1) % 4])))
}

/// Dois segmentos se cruzam? (Orientação — sem transcendental, HR-5.)
fn segments_cross(a: Vec2, b: Vec2, c: Vec2, d: Vec2) -> bool {
    let side = |p: Vec2, q: Vec2, r: Vec2| -> f32 {
        (q.x - p.x) * (r.y - p.y) - (q.y - p.y) * (r.x - p.x)
    };
    let (d1, d2) = (side(c, d, a), side(c, d, b));
    let (d3, d4) = (side(a, b, c), side(a, b, d));
    ((d1 > 0.0) != (d2 > 0.0)) && ((d3 > 0.0) != (d4 > 0.0))
}

/// Aplica o marquee: seleciona o que a caixa (em LOCAL) toca. Devolve `true` se mudou.
pub(crate) fn apply_marquee(
    drawing: &mut FlipDrawing,
    min: Vec2,
    max: Vec2,
    additive: bool,
) -> bool {
    let mut changed = false;
    if !additive {
        changed |= drawing.clear_selection();
    }
    for s in &mut drawing.strokes {
        if stroke_touches_rect(s.positions(), min, max) {
            changed |= !std::mem::replace(&mut s.selected, true);
        }
    }
    changed
}

/// **Mover um desenho: a arte ou a POSE?** (W7.2 — a regra que dá sentido à instância.)
///
/// - **Arte compartilhada** (o desenho é instanciado por 2+ chaves): o arrasto move a
///   **pose DESTA chave** (`FlipFrame::offset`) — o desenho inteiro anda, e só neste
///   quadro. É a razão de a instância existir: *a arte é uma só, o lugar é de cada
///   quadro* (é assim que um ciclo reusa desenho e ainda assim ANDA). Mover a geometria
///   arrastaria o gêmeo junto, e as duas chaves ficariam eternamente uma sobre a outra —
///   uma instância indistinguível de um hold.
/// - **Arte exclusiva** (o caminho comum): o arrasto move a **geometria** dos traços
///   selecionados, exatamente como antes. Aqui pose e geometria são observacionalmente
///   idênticas (só esta chave usa o desenho), e mexer na geometria mantém o documento
///   simples: um desenho não-posado é geometria em coords do objeto.
///
/// Numa arte compartilhada o arrasto **nunca deforma**, qualquer que seja a seleção — a
/// arte é dos dois quadros e um deles não pode reescrevê-la por baixo do outro. Quem quer
/// divergir a arte de um quadro **quebra o vínculo** (`Unlink` na tira,
/// [`ph2d_flip::FlipObject::make_single_user`]) e volta ao caminho comum.
fn move_drawing(
    flip: &mut ph2d_flip::FlipDoc,
    oid: ph2d_flip::FlipObjectId,
    lid: ph2d_flip::LayerId,
    key: ph2d_flip::Frame,
    did: ph2d_flip::DrawingId,
    delta: Vec2,
) -> bool {
    let Some(obj) = flip.object_mut(oid) else {
        return false;
    };
    if obj
        .drawing(did)
        .is_some_and(ph2d_flip::FlipDrawing::is_instanced)
    {
        return obj.translate_frame(lid, key, delta);
    }
    obj.drawing_mut(did)
        .is_some_and(|dr| translate_selection(dr, delta))
}

/// Translada os traços selecionados — **pontos E buracos** (ver o cabeçalho).
pub(crate) fn translate_selection(drawing: &mut FlipDrawing, delta: Vec2) -> bool {
    if delta.x == 0.0 && delta.y == 0.0 {
        return false;
    }
    let mut moved = false;
    for s in drawing.strokes.iter_mut().filter(|s| s.selected) {
        for p in s.positions_mut() {
            *p += delta;
        }
        for h in &mut s.holes {
            for p in h.iter_mut() {
                *p += delta;
            }
        }
        moved = true;
    }
    moved
}

impl crate::App {
    /// Converte um ponto de TELA no espaço LOCAL do objeto Flip ativo.
    fn flip_screen_to_local(&self, x: f32, y: f32, w2l: &Xform) -> Option<Vec2> {
        let gfx = self.gfx.as_ref()?;
        let win = gfx.surface.size();
        let world = gfx.camera.screen_to_world((x, y), win);
        let l = w2l.apply([f64::from(world[0]), f64::from(world[1])]);
        Some(Vec2::new(l[0] as f32, l[1] as f32))
    }

    /// Pen-move no modo Edit: arrasta a caixa, ou move a seleção. `true` = gesto vivo.
    pub(crate) fn flip_edit_canvas_move(&mut self, x: f32, y: f32) -> bool {
        let Some(gesture) = self.flip_edit_gesture else {
            return false;
        };
        match gesture {
            EditGesture::Marquee {
                start, additive, ..
            } => {
                self.flip_edit_gesture = Some(EditGesture::Marquee {
                    start,
                    cur: (x, y),
                    additive,
                });
                self.title_dirty = true;
                true
            }
            EditGesture::Click => true, // resolvido no down; nada a arrastar
            EditGesture::Move {
                last,
                down,
                collapse_to,
            } => {
                let w2l = self.flip_active_world_to_local();
                let Some(now) = self.flip_screen_to_local(x, y, &w2l) else {
                    return true;
                };
                let delta = now - last;
                // Passou do slop ⇒ é um ARRASTO de grupo, não um clique: o colapso morre.
                let collapse_to = if passed_slop(down, (x, y)) {
                    None
                } else {
                    collapse_to
                };
                self.flip_edit_gesture = Some(EditGesture::Move {
                    last: now,
                    down,
                    collapse_to,
                });
                let active_layer = self.flip_active_layer;
                let playhead = self.playhead;
                if let Some(gfx) = self.gfx.as_mut()
                    && let Some((oid, lid, key, did)) =
                        crate::flip_select::visible_key(&gfx.flip, &playhead, active_layer)
                    && move_drawing(&mut gfx.flip, oid, lid, key, did, delta)
                {
                    self.title_dirty = true;
                }
                true
            }
        }
    }

    /// Pen-up no modo Edit: fecha o marquee (aplicando a seleção) ou o move.
    ///
    /// O passo de undo sai do **diff pós-frame** (como todo o resto do Flip) — um arrasto
    /// inteiro vira UM passo porque o diff só roda quando não há gesto em curso.
    pub(crate) fn flip_edit_canvas_up(&mut self) -> bool {
        let Some(gesture) = self.flip_edit_gesture.take() else {
            return false;
        };
        let EditGesture::Marquee {
            start,
            cur,
            additive,
        } = gesture
        else {
            // `Click` (resolvido no down) ou `Move`. No `Move`, os pontos já foram
            // translados a cada quadro — mas resta o **colapso adiado**: se o usuário
            // soltou SEM arrastar, o clique num traço já selecionado significava "agora só
            // este". Em qualquer caso o UP é CONSUMIDO (no Edit o canvas é da ferramenta).
            if let EditGesture::Move {
                collapse_to: Some(i),
                ..
            } = gesture
            {
                let active_layer = self.flip_active_layer;
                let playhead = self.playhead;
                if let Some(gfx) = self.gfx.as_mut()
                    && let Some((oid, _l, did)) =
                        crate::flip_select::visible_drawing(&gfx.flip, &playhead, active_layer)
                    && let Some(dr) = gfx.flip.object_mut(oid).and_then(|o| o.drawing_mut(did))
                    && crate::flip_select::apply_pick(
                        dr,
                        Some(i),
                        crate::flip_select::Pick::Replace,
                    )
                {
                    self.title_dirty = true;
                }
            }
            return true;
        };
        // Um marquee que não passou do slop foi um CLIQUE no vazio: desmarcar (o clique é
        // tratado no down, então aqui não há o que fazer).
        if !passed_slop(start, cur) {
            return true;
        }
        let w2l = self.flip_active_world_to_local();
        let (x0, y0, x1, y1) = marquee_rect(start, cur);
        let (Some(a), Some(b)) = (
            self.flip_screen_to_local(x0, y0, &w2l),
            self.flip_screen_to_local(x1, y1, &w2l),
        ) else {
            return true;
        };
        // A caixa é de TELA; em LOCAL ela pode chegar invertida no eixo Y (a câmera olha
        // com y para cima e a tela com y para baixo) — normaliza depois de converter, não
        // antes. (Um min/max feito só na tela produziria uma caixa vazia em local.)
        let min = Vec2::new(a.x.min(b.x), a.y.min(b.y));
        let max = Vec2::new(a.x.max(b.x), a.y.max(b.y));

        let active_layer = self.flip_active_layer;
        let playhead = self.playhead;
        if let Some(gfx) = self.gfx.as_mut()
            && let Some((oid, _l, did)) =
                crate::flip_select::visible_drawing(&gfx.flip, &playhead, active_layer)
            && let Some(dr) = gfx.flip.object_mut(oid).and_then(|o| o.drawing_mut(did))
            && apply_marquee(dr, min, max, additive)
        {
            self.title_dirty = true;
        }
        true
    }
}

#[cfg(test)]
#[path = "flip_edit_gesture_tests.rs"]
mod tests;

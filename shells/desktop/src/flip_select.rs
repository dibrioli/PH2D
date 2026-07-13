//! ADR-0114 W6 — **o Edit Mode**: seleção de TRAÇOS, do lado do shell.
//!
//! Três decisões moram aqui, e as três têm um porquê que custou caro em outro lugar:
//!
//! 1. **A seleção é um ATRIBUTO do traço** (`FlipStroke::selected`, domínio Curve do GP),
//!    e não uma lista de índices no shell. A identidade de um traço é a posição dele na
//!    `Vec` — e o balde **insere no meio da lista** (`flip_fill`), a borracha a reescreve,
//!    o undo a restaura. Índices apodreceriam em silêncio contra as três, e o sintoma
//!    seria o painel recolorindo o traço errado.
//!
//! 2. **Selecionar NUNCA cria um quadro.** O resto dos gestos passa pelo
//!    `flip_autokey::target_drawing`, que materializa a chave (em branco ou duplicata)
//!    conforme a política da ferramenta. Selecionar não é autorar arte: ele lê o desenho
//!    que está NA TELA. Por isso o [`visible_drawing`] recebe um `&FlipDoc` **imutável** —
//!    a proibição é do TIPO, não de um comentário que a próxima wave esquece.
//!
//! 3. **O pick pega o traço de CIMA** (o que o usuário vê), e uma REGIÃO pega pelo
//!    INTERIOR: um preenchimento não tem linha (`hide_stroke`), então exigir proximidade
//!    da borda tornaria a cor do balde inselecionável — clicar no meio dela é o gesto
//!    óbvio, e é o que o GP faz.

use ph2d_core::{Playhead, Vec2};
use ph2d_flip::{DrawingId, FlipDoc, FlipDrawing, FlipObjectId, FlipStroke, LayerId};
use ph2d_vec_scene::Xform;

/// Raio mínimo de pick, em px de TELA. Uma linha de 1 px tem de ser clicável sem que o
/// usuário mire no pixel — é a mesma folga que o gizmo usa para pegar a arte.
const MIN_PICK_PX: f32 = 5.0; // LITERAL-PX-OK: folga de pick, nao metrica de design

/// O desenho que está **na tela** para a camada ativa — sem criar chave nenhuma.
///
/// Espelha a resolução do [`crate::flip_autokey::target_drawing`] (objeto → camada ativa
/// → quadro de AUTORIA, com a camada travada recusando), mas recebe o doc **imutável**:
/// selecionar lê, nunca materializa. Camada travada = `None` (a regra do GP: uma camada
/// travada não entrega os traços dela nem para seleção).
#[must_use]
pub(crate) fn visible_drawing(
    flip: &FlipDoc,
    playhead: &Playhead,
    active_layer: Option<LayerId>,
) -> Option<(FlipObjectId, LayerId, DrawingId)> {
    let oid = flip.objects().first().map(|o| o.id)?;
    let obj = flip.object(oid)?;
    let lid = active_layer
        .filter(|id| obj.layer(*id).is_some())
        .or_else(|| obj.layers().last().map(|l| l.id))?;
    let layer = obj.layer(lid)?;
    if layer.locked {
        return None;
    }
    let frame = layer.authoring_frame(obj.frame_at(playhead));
    let did = layer.drawing_at(frame)?;
    Some((oid, lid, did))
}

/// O traço sob o ponto `local`, se houver — **o de cima primeiro** (a ordem de z é a
/// ordem da lista, fundo → topo; então a varredura é de trás para a frente).
///
/// `px_to_world` converte px de tela em unidades de MUNDO e `w2l` desce de mundo para o
/// espaço LOCAL do objeto — a mesma conversão que o balde faz (`flip_fill::boundaries`):
/// a espessura do traço é absoluta em px de tela (brush absoluto, Enio 2026-07-11)
/// enquanto os pontos são unidades de documento, e é por isso que o raio de pick
/// **acompanha o zoom**: aproximar a câmera não pode exigir mira mais fina.
#[must_use]
pub(crate) fn stroke_at(
    drawing: &FlipDrawing,
    local: Vec2,
    px_to_world: f32,
    w2l: &Xform,
) -> Option<usize> {
    // px de TELA → unidade LOCAL (o `mean_scale` do objeto é o último degrau).
    let px_to_local = px_to_world * w2l.mean_scale() as f32;
    drawing
        .strokes
        .iter()
        .enumerate()
        .rev() // o de CIMA primeiro
        .find(|(_, s)| hits(s, local, px_to_local))
        .map(|(i, _)| i)
}

/// O ponto `local` pega este traço? (Tinta OU preenchimento.)
fn hits(s: &FlipStroke, p: Vec2, px_to_local: f32) -> bool {
    // (a) O INTERIOR do preenchimento — inclusive o de uma região (`hide_stroke`), que
    //     não tem linha nenhuma para se aproximar. Os buracos não pegam: clicar no furo
    //     de um "O" é clicar no que está ATRÁS dele.
    if s.fill.is_some()
        && crate::flip_fill::ring_contains(s.positions(), p)
        && !s
            .holes
            .iter()
            .any(|h| crate::flip_fill::ring_contains(h, p))
    {
        return true;
    }
    // (b) A TINTA: a meia-espessura do traço (px de tela → local, como o balde), com um
    //     piso para que uma linha fina não exija mira de pixel. Uma região não tem tinta.
    if s.hide_stroke {
        return false;
    }
    let pos = s.positions();
    let widths = s.widths();
    let reach = |i: usize| -> f32 {
        let half = widths.get(i).copied().unwrap_or(0.0) * 0.5;
        (half.max(MIN_PICK_PX) * px_to_local).max(f32::EPSILON)
    };
    if pos.len() == 1 {
        let d = p - pos[0];
        return d.x * d.x + d.y * d.y <= reach(0) * reach(0);
    }
    pos.windows(2).enumerate().any(|(i, w)| {
        let r = reach(i);
        seg_dist2(p, w[0], w[1]) <= r * r
    })
}

/// Distância² de `p` ao segmento `a`→`b`.
fn seg_dist2(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let len2 = ab.x * ab.x + ab.y * ab.y;
    let t = if len2 > 0.0 {
        (((p - a).x * ab.x + (p - a).y * ab.y) / len2).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let c = a + ab * t;
    let d = p - c;
    d.x * d.x + d.y * d.y
}

/// O que um clique faz com a seleção.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Pick {
    /// Clique simples: o traço vira a seleção INTEIRA (o resto sai).
    Replace,
    /// Shift+clique: ALTERNA este traço, preservando o resto.
    Toggle,
}

/// Aplica o clique. Devolve `true` se o documento mudou (o passo de undo sai do diff
/// pós-frame, como todo o resto do Flip).
pub(crate) fn apply_pick(drawing: &mut FlipDrawing, hit: Option<usize>, pick: Pick) -> bool {
    match (hit, pick) {
        // Clique no VAZIO (sem Shift) = limpar a seleção. Com Shift, o vazio não faz
        // nada: um shift-clique que errou o traço por 2 px não pode apagar a seleção
        // que o usuário levou meia dúzia de cliques para montar.
        (None, Pick::Replace) => drawing.clear_selection(),
        (None, Pick::Toggle) => false,
        (Some(i), Pick::Replace) => {
            // Já era a seleção inteira? Então nada muda (e nada vira passo de undo).
            let already = drawing.strokes[i].selected
                && drawing.strokes.iter().filter(|s| s.selected).count() == 1;
            if already {
                return false;
            }
            drawing.clear_selection();
            drawing.strokes[i].selected = true;
            true
        }
        (Some(i), Pick::Toggle) => {
            drawing.strokes[i].selected = !drawing.strokes[i].selected;
            true
        }
    }
}

impl crate::App {
    /// A tool Flip quer o canvas para SELECIONAR agora? (ativa + modo Edit.)
    #[must_use]
    pub(crate) fn flip_wants_edit(&self) -> bool {
        self.flip_active
            && matches!(
                self.flip_style.map(|s| s.mode),
                Some(ph2d_tool_flip::FlipMode::Edit)
            )
    }

    /// O clique de seleção. `true` = consumido (o gizmo/pick de objeto não o vê).
    ///
    /// **Consome mesmo quando erra o traço**: no modo Edit, um clique no vazio é
    /// "desmarcar", não "selecionar o objeto com o gizmo". Deixá-lo cair no gizmo faria o
    /// arrasto seguinte MOVER o objeto inteiro — que é justamente o que o Edit Mode
    /// existe para separar do Object Mode.
    pub(crate) fn flip_edit_canvas_down(&mut self, x: f32, y: f32) -> bool {
        if !self.flip_wants_edit() {
            return false;
        }
        let pick = if self.modifiers.shift_key() {
            Pick::Toggle
        } else {
            Pick::Replace
        };
        let active_layer = self.flip_active_layer;
        let w2l = self.flip_active_world_to_local();
        let playhead = self.playhead;

        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let win = gfx.surface.size();
        let world = gfx.camera.screen_to_world((x, y), win);
        let px_to_world = gfx.camera.height_world.max(f32::EPSILON) / win.height.max(1) as f32;
        let local = w2l.apply([f64::from(world[0]), f64::from(world[1])]);
        let local = Vec2::new(local[0] as f32, local[1] as f32);

        let Some((oid, _lid, did)) = visible_drawing(&gfx.flip, &playhead, active_layer) else {
            // Camada travada, ou quadro sem desenho: DIZ, em vez de engolir o clique em
            // silêncio (o mesmo princípio dos erros do balde).
            gfx.toasts.push(ph2d_editor::Toast::warning(
                "Edit: the layer is locked, or has no drawing on this frame",
            ));
            self.title_dirty = true;
            return true;
        };
        let Some(drawing) = gfx.flip.object_mut(oid).and_then(|o| o.drawing_mut(did)) else {
            return true;
        };
        let hit = stroke_at(drawing, local, px_to_world, &w2l);
        // **Arrastar um traço já o move** (W6.1). Se o clique pegou traço, o gesto que
        // começa é o de MOVER — inclusive quando o traço ainda não estava selecionado
        // (aí o pick o seleciona primeiro, e o arrasto o leva junto). Exigir clicar,
        // soltar e clicar de novo para arrastar é a ergonomia que faz o usuário concluir
        // que a ferramenta não responde. No VAZIO, o gesto é o marquee.
        //
        // Shift+arrasto num traço já selecionado seria ambíguo (alternar ou mover?): o
        // Shift manda, e o gesto vira alternar — o arrasto não pega.
        let shift = pick == Pick::Toggle;
        if apply_pick(drawing, hit, pick) {
            self.title_dirty = true;
        }
        self.flip_edit_gesture = match (hit, shift) {
            (Some(_), false) => Some(crate::flip_edit_gesture::EditGesture::Move { last: local }),
            (Some(_), true) => None,
            (None, _) => Some(crate::flip_edit_gesture::EditGesture::Marquee {
                start: (x, y),
                cur: (x, y),
                additive: shift,
            }),
        };
        // A seleção vira o alvo dos ajustes do painel — o "alvo vivo" (a última coisa
        // criada) sai de cena enquanto houver seleção.
        self.flip_live_clear();
        true
    }
}

impl crate::App {
    /// Apaga os traços selecionados do desenho visível. `true` = apagou algo (e a tecla
    /// foi consumida — ver o chamador em `input_dispatch::keyboard`).
    pub(crate) fn flip_delete_selected(&mut self) -> bool {
        let active_layer = self.flip_active_layer;
        let playhead = self.playhead;
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let Some((oid, _lid, did)) = visible_drawing(&gfx.flip, &playhead, active_layer) else {
            return false;
        };
        let Some(drawing) = gfx.flip.object_mut(oid).and_then(|o| o.drawing_mut(did)) else {
            return false;
        };
        let n = drawing.delete_selected();
        if n > 0 {
            self.title_dirty = true;
            self.flip_live_clear(); // o alvo vivo pode ter sido um dos apagados
        }
        n > 0
    }
}

/// **Os ajustes do painel miram a SELEÇÃO** — o passe por-frame que aposenta o "alvo
/// vivo" enquanto há traços selecionados.
///
/// Roda 1×/frame (ao lado do `flip_live_refresh`) e obedece a **duas** regras que não são
/// negociáveis:
///
/// 1. **Só a MUDANÇA age.** Se o passe reaplicasse o estilo a cada frame, selecionar um
///    traço vermelho com o painel em azul o pintaria de azul **no ato do clique** — o
///    usuário perderia a arte só por olhar para ela. Por isso o estilo do frame anterior é
///    guardado (`flip_edit_style`) e só os campos que **de fato mudaram** são escritos.
///    (É o mesmo princípio do `same_stroke_style` do alvo vivo, com a diferença de que
///    aqui não há insumo cru para refazer o traço.)
///
/// 2. **Nada de GEOMETRIA.** O passe escreve só atributos (cor, opacidade, dureza,
///    espessura). Reescrever posições a partir de uma cópia "pristina" desfaria uma
///    escultura feita depois da seleção — o slider de cor apagaria o trabalho do Sculpt.
///
/// A espessura é o único campo com forma: ela vira `size × perfil`, onde o perfil é a
/// razão `w_i / w_max` lida do PRÓPRIO traço, agora. Escalar preserva razões, então
/// re-derivar do estado atual é **idempotente** (arrastar o slider ida-e-volta devolve o
/// traço original) e preserva o desenho de pressão da caneta — que um `w_i := size` chapado
/// destruiria em silêncio.
pub(crate) fn flip_edit_style_refresh(app: &mut crate::App) {
    let editing = app.flip_wants_edit();
    let Some(style) = app.flip_style.filter(|_| editing) else {
        app.flip_edit_style = None;
        return;
    };
    let active_layer = app.flip_active_layer;
    let playhead = app.playhead;
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let Some((oid, _lid, did)) = visible_drawing(&gfx.flip, &playhead, active_layer) else {
        app.flip_edit_style = None;
        return;
    };
    let Some(drawing) = gfx.flip.object_mut(oid).and_then(|o| o.drawing_mut(did)) else {
        return;
    };
    if !drawing.any_selected() {
        app.flip_edit_style = None;
        return;
    }
    // A 1ª volta com seleção só MEMORIZA — não escreve nada (ver a regra 1).
    let Some(prev) = app.flip_edit_style else {
        app.flip_edit_style = Some(style);
        return;
    };
    app.flip_edit_style = Some(style);
    if apply_style_delta(drawing, &prev, &style) {
        app.title_dirty = true;
    }
}

/// Escreve nos traços selecionados **só os campos que mudaram** entre `prev` e `now`.
/// Devolve `true` se escreveu algo. É a parte pura do [`flip_edit_style_refresh`] — e é
/// pura de propósito: a regra "só a mudança age" é a que, se quebrar, destrói arte, então
/// ela tem de ser testável sem um `App` inteiro em volta.
pub(crate) fn apply_style_delta(
    drawing: &mut FlipDrawing,
    prev: &ph2d_tool_flip::FlipStyleSnapshot,
    now: &ph2d_tool_flip::FlipStyleSnapshot,
) -> bool {
    if prev == now {
        return false;
    }
    let color = crate::flip_draw::srgb8_to_linear(now.stroke);
    let mut changed = false;
    let fill_color = crate::flip_draw::srgb8_to_linear(now.fill_color);
    for s in drawing.strokes.iter_mut().filter(|s| s.selected) {
        // **A cor da LINHA e a cor do MIOLO são dois atributos, com dois controles.**
        //
        // O 1º corte fazia o swatch do traço recolorir o miolo junto ("um traço com fill é
        // uma coisa só") — e o Enio derrubou isso no smoke: são duas decisões de arte, e
        // fundi-las tira do usuário a única forma de mudar uma sem a outra. O painel ganhou
        // o swatch de Fill no modo Edit; aqui, cada um escreve no SEU campo.
        //
        // Uma REGIÃO (`hide_stroke`, a cor do balde) não tem linha visível: o swatch do
        // traço não a alcança — só o de Fill.
        if now.stroke != prev.stroke && !s.hide_stroke {
            for c in s.colors_mut() {
                *c = color;
            }
            changed = true;
        }
        // O miolo — de uma forma preenchida (Shape: Filled) OU de uma região do balde.
        // Um traço SEM miolo não ganha um: o swatch recolore, não cria.
        if now.fill_color != prev.fill_color
            && let Some(f) = s.fill.as_mut()
        {
            f.color = fill_color;
            changed = true;
        }
        if (now.opacity - prev.opacity).abs() > f32::EPSILON {
            for o in s.opacities_mut() {
                *o = now.opacity;
            }
            changed = true;
        }
        if (now.hardness - prev.hardness).abs() > f32::EPSILON {
            s.hardness = now.hardness;
            changed = true;
        }
        if (now.width_px - prev.width_px).abs() > f64::EPSILON {
            // O PERFIL da pressão (`w_i / w_max`) é lido do traço AGORA e re-imposto sobre
            // a espessura nova. Escalar preserva razões ⇒ idempotente (ida-e-volta no
            // slider devolve o traço) e a caneta não perde o desenho de pressão — que um
            // `w_i := size` chapado destruiria em silêncio.
            let max = s.widths().iter().copied().fold(0.0f32, f32::max);
            if max > 0.0 {
                let size = now.width_px as f32;
                for w in s.widths_mut() {
                    *w = size * (*w / max);
                }
                changed = true;
            }
        }
    }
    changed
}

#[cfg(test)]
#[path = "flip_select_tests.rs"]
mod tests;

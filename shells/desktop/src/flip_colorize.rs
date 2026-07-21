//! ADR-0114 **C2 (Colorize)** — rabiscar cores sobre a line-art (`docs/Flip/09`).
//!
//! Cada rabisco é uma polilinha colorida: uma SEMENTE do corte LazyBrush, não arte. Eles
//! acumulam num buffer transiente; **Apply** roda o motor `ph2d-flip-colorize` sobre TODOS
//! os rabiscos + a line-art e materializa cada região como um traço preenchido — pelo MESMO
//! `fill_stroke` do balde, então a borda de uma cor colorida e a de um balde não divergem;
//! **Clear** descarta os rabiscos.
//!
//! O gesto é irmão do `flip_draw` (down/move/up → polilinha); o commit é irmão do
//! `flip_fill` (autokey `Modify`, insere acima dos fills existentes); e o **overlay ao vivo**
//! (`flip_colorize_preview_data`) usa o MESMO slot de preview do traço do Draw — sem ele o
//! artista rabiscava às cegas, e um gesto que não deixa marca não se aprende.
//!
//! **O fluxo:** modo Colorize → escolha a cor na swatch **Color** → rabisque DENTRO de uma
//! região → troque a cor → rabisque noutra → **Apply**. **Clear** joga os rabiscos fora.

use crate::flip_fill_dilate::{boundaries, fill_stroke};
use ph2d_core::Vec2;
use ph2d_flip::{DrawingId, FlipDrawing, FlipObjectId, FlipStroke, Point};
use ph2d_flip_colorize::{ColorRegion, Scribble};
use ph2d_flip_render::{FlipGpuData, pack_drawing};
use ph2d_tool_flip::{FlipMode, FlipStyleSnapshot};
use ph2d_vec_scene::Xform;

/// **A última aplicação do Colorize, VIVA** — o "ajustar a última operação" do Blender (o
/// painel F6/redo): enquanto ela existe, mexer no **Trap** ou no **Bleed** re-roda o corte
/// **em tempo real**, sem clicar Apply de novo (o pedido do Enio, 6º smoke). Ela morre no
/// primeiro gesto que NÃO seja esse ajuste — novo rabisco, novo Apply, Clear, sair do modo,
/// undo, ou o artista editar o próprio desenho (o guard de comprimento).
///
/// O re-Apply é *restaurar a base congelada + reinserir* — posição-independente e sem
/// precisar identificar "os meus strokes" (o `FlipStroke` não tem id). A base é o desenho
/// como ele estava ANTES de a 1ª aplicação inserir uma região; reinserir sobre ela reproduz
/// o Apply com os parâmetros novos, sem empilhar.
struct LiveApply {
    /// Rótulo → cor (a paleta que o Apply montou dos rabiscos).
    palette: Vec<[u8; 4]>,
    /// As sementes já em LOCAL do desenho, congeladas no Apply — uma função pura de
    /// `(sementes, trap, bleed)`, então o re-Apply é determinístico.
    seeds: Vec<Scribble>,
    /// O alvo: o objeto e o desenho onde as regiões foram escritas.
    oid: FlipObjectId,
    did: DrawingId,
    /// Os strokes do desenho ANTES da 1ª inserção — a base congelada.
    base: Vec<FlipStroke>,
    /// Quantos strokes a última (re)inserção produziu. O **guard de segurança**: se o
    /// desenho não é mais `base.len() + produced`, o artista editou outra coisa (nova
    /// linha, balde, borracha) e a sessão MORRE — re-Aplicar apagaria a edição dele.
    produced: usize,
    /// Os últimos `(trap, bleed)` rodados — o re-Apply só dispara quando MUDAM.
    trap: f64,
    bleed: f64,
}

/// `(precision, trap_px)` a partir do estilo + da vista — a MESMA conversão do balde, numa
/// porta só, para o Apply e o re-Apply ao vivo **nunca** divergirem (duas portas divergem).
///
/// O `trap_px` devolvido é o EFETIVO: o `max` do slider **Trap** com o raio de **selagem** do
/// `Bleed 0` (6º smoke, "Bleed 0 SELA o vão"). Os dois são força de trapped-ball; o `Bleed`
/// alimenta o raio pelo `seal_from_bleed` (o pedágio satura e nunca fecha a lente — só a bola
/// que não passa pelo vão o faz). `Bleed` acima do joelho ⇒ selo 0 ⇒ o Trap sozinho, e a
/// seepage fica com o `squeeze` — o 5º smoke intacto.
fn precision_and_trap(style: &FlipStyleSnapshot, px_to_world: f32, obj_scale: f32) -> (f32, f32) {
    let doc_per_px = px_to_world * obj_scale;
    // px de tela → px de buffer por unidade de documento (a precisão do balde).
    let precision = 1.6 / doc_per_px.max(1e-6);
    // O Trap chega em px de TELA e atravessa as duas conversões (BUGS #11: subir a
    // Precision encolheria a bola em silêncio se ele não cruzasse `precision`).
    let trap_px = (style.trap as f32) * doc_per_px * precision;
    // O selo do Bleed 0, em DOC units → px de buffer pela MESMA precisão. Combinado por `max`.
    let seal_px = ph2d_flip_colorize::seal_from_bleed(style.colorize_bleed as f32) * precision;
    (precision, trap_px.max(seal_px))
}

/// **A porta única que produz e insere as regiões** — o Apply e o re-Apply ao vivo chamam
/// esta MESMA função, então a borda de uma cor colorida nunca depende de por qual caminho
/// ela foi (re)gerada. Assume `drawing` já na base (line-art + fills pré-Colorize) e devolve
/// quantos strokes foram inseridos.
fn insert_regions(
    drawing: &mut FlipDrawing,
    palette: &[[u8; 4]],
    seeds: &[Scribble],
    precision: f32,
    trap_px: f32,
    squeeze: u32,
) -> usize {
    let lines = boundaries(drawing);
    if lines.is_empty() {
        return 0;
    }
    let regions: Vec<ColorRegion> =
        ph2d_flip_colorize::colorize_with(&lines, seeds, precision, trap_px, squeeze);
    // Cada região entra ACIMA dos fills existentes (a cor nova cobre a velha, abaixo da
    // linha — o `Paint` do balde), com a MESMA dilatação (contour_widths + fill_stroke).
    let is_fill = |s: &FlipStroke| s.hide_stroke && s.fill.is_some();
    let mut produced = 0;
    for region in regions {
        let color = crate::flip_draw::srgb8_to_linear(palette[region.label as usize]);
        let widths = ph2d_flip_fill::contour_widths(&lines, &region.fill.outer);
        let stroke = fill_stroke(&region.fill.outer, region.fill.holes, color, 1.0, &widths);
        let at = drawing
            .strokes
            .iter()
            .rposition(is_fill)
            .map_or(0, |i| i + 1);
        drawing.strokes.insert(at, stroke);
        produced += 1;
    }
    produced
}

/// Distância mínima entre amostras de um rabisco (px de tela) — igual ao `flip_draw`.
const MIN_SAMPLE_PX: f32 = 2.0;

/// A espessura do rabisco, em unidades LOCAIS do desenho.
///
/// ⚠️ **Pela MESMA porta do traço do Draw** (`size_to_world(Size) × escala do objeto`,
/// `flip_draw::build_stroke`): o `Point.width` do Flip é MUNDO, não px de tela, e cravar um
/// número de tela ali pinta um borrão maior que o desenho. E é o **Size do pincel** que manda
/// — o Colorize não ganha um 2º slider para a mesma grandeza (a regra do Erase/Sculpt).
///
/// O MESMO número governa o overlay e a SEMENTE: o que o artista pinta é o que semeia.
fn scribble_width(style: &FlipStyleSnapshot, w2l: &Xform) -> f32 {
    ph2d_tool_flip::size_to_world(style.width_px) * w2l.mean_scale() as f32
}

/// Os rabiscos coloridos acumulados + o rabisco em curso. Transientes: não viajam no
/// documento (são sementes), e o Apply/Clear os consomem.
#[derive(Default)]
pub(crate) struct FlipColorize {
    /// (cor sRGB8, pontos em MUNDO). MUNDO porque a pose do objeto pode mudar entre desenhar
    /// e aplicar; a conversão para LOCAL acontece no Apply.
    scribbles: Vec<([u8; 4], Vec<Vec2>)>,
    /// Os rabiscos REMOVIDOS pelo Ctrl+Z (o redo local do Colorize — "undo/redo ruim", 7º
    /// smoke): rabisco é semente transiente, fora do `ProjectState`, então o Ctrl+Z dele é
    /// deste buffer, nunca da fila global (`undo_route::UndoOwner::Colorize`). Um rabisco
    /// NOVO descarta os removidos (a lei de toda fila de redo); Apply e Clear também.
    popped: Vec<([u8; 4], Vec<Vec2>)>,
    /// O rabisco em curso (MUNDO) + a cor fixada no pen-down.
    current: Vec<Vec2>,
    current_color: [u8; 4],
    active: bool,
    /// A última aplicação viva (Trap/Bleed em tempo real). `None` = nada a re-ajustar.
    live: Option<LiveApply>,
}

impl FlipColorize {
    pub(crate) fn clear(&mut self) {
        self.scribbles.clear();
        self.popped.clear();
        self.current.clear();
        self.active = false;
        self.live = None;
    }

    /// Encerra o ajuste ao vivo — o desenho mudou por fora do Trap/Bleed (undo, troca de
    /// modo, edição do artista), então a base congelada não descreve mais a realidade.
    pub(crate) fn end_live(&mut self) {
        self.live = None;
    }

    /// Semeia um rabisco pronto (pontos em MUNDO) — usado pelo smoke para demonstrar o
    /// Apply sem o gesto interativo.
    pub(crate) fn push_scribble(&mut self, color: [u8; 4], world_points: Vec<Vec2>) {
        if world_points.len() >= 2 {
            self.scribbles.push((color, world_points));
            self.popped.clear();
            // Uma semente nova torna o resultado aplicado obsoleto: o próximo Apply é uma
            // operação NOVA, e mexer no Trap antes dele re-rodaria sementes desatualizadas.
            self.live = None;
        }
    }

    /// Há rabisco pendente para o Ctrl+Z remover?
    #[must_use]
    pub(crate) fn can_undo_scribble(&self) -> bool {
        !self.scribbles.is_empty()
    }

    /// Há rabisco removido para o Ctrl+Shift+Z devolver?
    #[must_use]
    pub(crate) fn can_redo_scribble(&self) -> bool {
        !self.popped.is_empty()
    }

    /// Remove o ÚLTIMO rabisco (Ctrl+Z no modo Colorize). O overlay ao vivo é quem mostra
    /// o efeito — a marca some da tela no mesmo frame.
    pub(crate) fn undo_scribble(&mut self) {
        if let Some(s) = self.scribbles.pop() {
            self.popped.push(s);
        }
    }

    /// Devolve o último rabisco removido (Ctrl+Shift+Z no modo Colorize).
    pub(crate) fn redo_scribble(&mut self) {
        if let Some(s) = self.popped.pop() {
            self.scribbles.push(s);
        }
    }
}

impl crate::App {
    /// A tool Flip quer o canvas para RABISCAR agora? (ativa + modo Colorize.)
    #[must_use]
    pub(crate) fn flip_wants_colorize(&self) -> bool {
        self.flip_active && matches!(self.flip_style.map(|s| s.mode), Some(FlipMode::Colorize))
    }

    /// Tela → mundo (o rabisco é capturado em MUNDO, como o `flip_draw`).
    fn flip_colorize_world(&self, x: f32, y: f32) -> Option<(Vec2, f32)> {
        let gfx = self.gfx.as_ref()?;
        let win = gfx.surface.size();
        let w = gfx.camera.screen_to_world((x, y), win);
        let px_to_world = gfx.camera.height_world.max(f32::EPSILON) / win.height.max(1) as f32;
        Some((Vec2::new(w[0], w[1]), px_to_world))
    }

    /// Pen-down: começa um rabisco novo com a cor atual do Colorize.
    pub(crate) fn flip_colorize_canvas_down(&mut self, x: f32, y: f32) -> bool {
        if !self.flip_wants_colorize() {
            return false;
        }
        let Some(style) = self.flip_style else {
            return false;
        };
        let Some((w, _)) = self.flip_colorize_world(x, y) else {
            return false;
        };
        self.flip_colorize.current.clear();
        self.flip_colorize.current.push(w);
        self.flip_colorize.current_color = style.colorize_color;
        self.flip_colorize.active = true;
        true
    }

    /// Pen-move: acumula amostras (só as que andaram ≥ `MIN_SAMPLE_PX`).
    pub(crate) fn flip_colorize_canvas_move(&mut self, x: f32, y: f32) -> bool {
        if !self.flip_colorize.active {
            return false;
        }
        let Some((w, px_to_world)) = self.flip_colorize_world(x, y) else {
            return false;
        };
        let min = MIN_SAMPLE_PX * px_to_world;
        let moved = self
            .flip_colorize
            .current
            .last()
            .is_none_or(|p| (w - *p).length() >= min);
        if moved {
            self.flip_colorize.current.push(w);
        }
        true
    }

    /// Pen-up: fecha o rabisco em curso e o acumula (≥ 2 pontos).
    pub(crate) fn flip_colorize_canvas_up(&mut self) -> bool {
        if !self.flip_colorize.active {
            return false;
        }
        self.flip_colorize.active = false;
        let color = self.flip_colorize.current_color;
        let pts = std::mem::take(&mut self.flip_colorize.current);
        if pts.len() >= 2 {
            // Pela porta única: um rabisco novo também descarta os removidos (redo local).
            self.flip_colorize.push_scribble(color, pts);
        }
        true
    }

    /// **Clear** — descarta os rabiscos acumulados.
    pub(crate) fn flip_colorize_clear(&mut self) {
        self.flip_colorize.clear();
    }

    /// GPU-data dos rabiscos acumulados (+ o em curso) pro **overlay ao vivo**.
    ///
    /// Sem ele o artista rabisca ÀS CEGAS — os rabiscos só existiriam no resultado do
    /// Apply, e um gesto que não deixa marca não se aprende. Viaja pelo MESMO slot de
    /// preview do traço do Draw (`flip_draw::flip_preview_data`): os dois nunca coexistem,
    /// porque são MODOS diferentes — um slot, uma resposta a *"o que está em curso?"*.
    #[must_use]
    pub(crate) fn flip_colorize_preview_data(&self) -> Option<FlipGpuData> {
        if !self.flip_wants_colorize() {
            return None;
        }
        let live = self.flip_colorize.active && self.flip_colorize.current.len() >= 2;
        if self.flip_colorize.scribbles.is_empty() && !live {
            return None;
        }
        let style = self.flip_style?;
        // MUNDO → LOCAL da camada ativa (a mesma conversão do preview do Draw; o Apply
        // usa a MESMA `w2l` e a MESMA largura, então o que se vê é o que semeia).
        let w2l = self.flip_active_world_to_local();
        let width = scribble_width(&style, &w2l);
        let mut d = FlipDrawing::default();
        let committed = self.flip_colorize.scribbles.iter().map(|(c, p)| (*c, p));
        let in_flight = live.then_some({
            (
                self.flip_colorize.current_color,
                &self.flip_colorize.current,
            )
        });
        for (color, pts) in committed.chain(in_flight) {
            if pts.len() < 2 {
                continue;
            }
            let c = crate::flip_draw::srgb8_to_linear(color);
            let mut s = FlipStroke::new();
            for p in pts {
                let l = w2l.apply([f64::from(p.x), f64::from(p.y)]);
                s.push_point(Point {
                    pos: Vec2::new(l[0] as f32, l[1] as f32),
                    width,
                    opacity: 1.0,
                    color: c,
                });
            }
            d.strokes.push(s);
        }
        if d.strokes.is_empty() {
            return None;
        }
        Some(pack_drawing(&d))
    }

    /// **Apply** — roda o corte LazyBrush sobre TODOS os rabiscos + a line-art e materializa
    /// cada região como um traço preenchido, no desenho-alvo (autokey `Modify`, como o
    /// balde). Consome os rabiscos.
    pub(crate) fn flip_colorize_apply(&mut self) {
        if self.flip_colorize.scribbles.is_empty() {
            return;
        }
        let Some(style) = self.flip_style else {
            return;
        };
        let active_layer = self.flip_active_layer;
        let w2l = self.flip_active_world_to_local();
        // A MESMA largura que o overlay desenhou — o que o artista pinta é o que semeia.
        let seed_width = scribble_width(&style, &w2l);
        let playhead = self.playhead;

        // Rabiscos MUNDO → LOCAL, agrupados por cor: cada cor distinta é um rótulo, e o
        // mapa rótulo→cor devolve a cor de cada região.
        //
        // ⚠️ **Feito ANTES de tocar o `gfx`, e as sementes NÃO são consumidas aqui.** Abaixo há
        // CINCO saídas que recusam o Apply, e três delas mandam o artista *corrigir e tentar de
        // novo* ("desenhe a line-art primeiro", "rabisque dentro das formas fechadas", "a
        // camada está travada") — o que era impossível, porque um `mem::take` no topo já tinha
        // levado os rabiscos embora, e o **Ctrl+Z não os trazia de volta** (a fila de removidos
        // era limpa na linha seguinte, então o `undo_route` deixava de ser dono do atalho).
        // Uma recusa não pode custar o trabalho do artista: só o SUCESSO consome (no fim).
        let mut palette: Vec<[u8; 4]> = Vec::new();
        let mut seeds: Vec<Scribble> = Vec::new();
        for (color, world_pts) in &self.flip_colorize.scribbles {
            let label = palette.iter().position(|c| c == color).unwrap_or_else(|| {
                palette.push(*color);
                palette.len() - 1
            }) as u16;
            let points: Vec<Vec2> = world_pts
                .iter()
                .map(|p| {
                    let l = w2l.apply([f64::from(p.x), f64::from(p.y)]);
                    Vec2::new(l[0] as f32, l[1] as f32)
                })
                .collect();
            seeds.push(Scribble {
                label,
                points,
                width: seed_width,
            });
        }

        let strip = &mut self.flip_strip;
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let win = gfx.surface.size();
        let px_to_world = gfx.camera.height_world.max(f32::EPSILON) / win.height.max(1) as f32;

        let Some((oid, _lid, did)) = crate::flip_autokey::target_drawing(
            &mut gfx.flip,
            &playhead,
            active_layer,
            strip,
            crate::flip_autokey::FlipEdit::Modify,
        ) else {
            gfx.toasts.push(ph2d_editor::Toast::warning(
                "Colorize: the layer is locked, or has no drawing on this frame",
            ));
            self.title_dirty = true;
            return;
        };

        let Some(drawing) = gfx.flip.object_mut(oid).and_then(|o| o.drawing_mut(did)) else {
            return;
        };
        if boundaries(drawing).is_empty() {
            gfx.toasts.push(ph2d_editor::Toast::warning(
                "Colorize: draw the line-art first",
            ));
            self.title_dirty = true;
            return;
        }
        let obj_scale = w2l.mean_scale() as f32;

        // O **Trap** é o raio da bola, e o **Bleed** governa o vazamento pelo vão em duas
        // metades: o pedágio de aperto (contínuo) e, no extremo baixo, o RAIO de selagem (que
        // entra no `max` com o Trap). `precision_and_trap`/`squeeze_from_bleed` são as portas
        // compartilhadas com o re-Apply ao vivo, senão os dois caminhos divergiriam.
        //
        // ⚠️ **Correção (auditoria 2026-07-20): o comentário que morava aqui MENTIA.** Ele
        // dizia *"o Trap é um PISO — o motor cresce a bola até os rabiscos caírem em regiões
        // distintas"*. O motor **não faz isso**: `trap_px` vai direto para `segment(grid,
        // trap_px)` e a única adaptação é o *fallback* para raio 0 quando NENHUM pixel comporta
        // a bola. Não há busca, não há crescimento — o número que entra é o que vale.
        let (precision, trap_px) = precision_and_trap(&style, px_to_world, obj_scale);
        let squeeze = ph2d_flip_colorize::squeeze_from_bleed(style.colorize_bleed as f32);

        // A **base congelada** — o desenho ANTES de a 1ª região entrar. É o que o re-Apply ao
        // vivo restaura para reinserir sem empilhar (Trap/Bleed em tempo real).
        let base = drawing.strokes.clone();
        let produced = insert_regions(drawing, &palette, &seeds, precision, trap_px, squeeze);
        if produced == 0 {
            drawing.strokes = base; // nada saiu — devolve o desenho intocado
            gfx.toasts.push(ph2d_editor::Toast::warning(
                "Colorize: no regions — scribble inside the closed shapes",
            ));
            self.title_dirty = true;
            return;
        }

        // ✅ SÓ AGORA as sementes foram consumidas — o Apply teve sucesso. Um redo de rabisco
        // pós-Apply devolveria uma semente sem o contexto que a criou, então a fila de
        // removidos morre junto.
        self.flip_colorize.scribbles.clear();
        self.flip_colorize.popped.clear();

        // A operação fica VIVA: mexer no Trap/Bleed agora re-roda o corte em tempo real
        // (`flip_colorize_live_adjust`), sem clicar Apply de novo.
        self.flip_colorize.live = Some(LiveApply {
            palette,
            seeds,
            oid,
            did,
            base,
            produced,
            trap: style.trap,
            bleed: style.colorize_bleed,
        });
        self.title_dirty = true;
    }

    /// **Trap/Bleed em tempo real depois do Apply** (6º smoke, pedido do Enio: *"trap e bleed
    /// não estão em tempo real após apply. faça ficar em tempo real para ajustes"*).
    ///
    /// Roda no prólogo do frame (ao lado do drain do Apply, com `self` livre). Quando o Trap
    /// ou o Bleed mudou desde a última rodada, restaura a base congelada e reinsere as regiões
    /// com os parâmetros novos — o "ajustar a última operação" do Blender.
    ///
    /// **Undo sai de graça:** um arrasto de slider mantém `held_button` preso, então o
    /// `post_frame_undo` suprime os frames intermediários e o gesto inteiro vira UM passo (o
    /// mesmo mecanismo do `envelope_gesture`). Desfazer devolve ao resultado do Trap anterior;
    /// de novo, ao pré-Apply.
    pub(crate) fn flip_colorize_live_adjust(&mut self) {
        if self.flip_colorize.live.is_none() {
            return;
        }
        // Sair do modo Colorize encerra a adjustabilidade (o painel some, a base congelada
        // deixa de descrever o que está na tela).
        let Some(style) = self.flip_style.filter(|_| self.flip_wants_colorize()) else {
            self.flip_colorize.end_live();
            return;
        };
        let live = self.flip_colorize.live.as_ref().expect("live is Some");
        // Só re-roda quando um dos dois de fato MUDOU (senão todo frame pagaria o corte).
        if style.trap == live.trap && style.colorize_bleed == live.bleed {
            return;
        }
        let (oid, did) = (live.oid, live.did);
        let w2l = self.flip_active_world_to_local();
        let obj_scale = w2l.mean_scale() as f32;

        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let win = gfx.surface.size();
        let px_to_world = gfx.camera.height_world.max(f32::EPSILON) / win.height.max(1) as f32;
        let (precision, trap_px) = precision_and_trap(&style, px_to_world, obj_scale);
        let squeeze = ph2d_flip_colorize::squeeze_from_bleed(style.colorize_bleed as f32);

        let Some(drawing) = gfx.flip.object_mut(oid).and_then(|o| o.drawing_mut(did)) else {
            self.flip_colorize.end_live(); // o desenho-alvo sumiu (undo/delete)
            return;
        };
        // **O guard de segurança:** o desenho ainda é `base + as MINHAS regiões`? Se o artista
        // desenhou uma linha nova, encheu com o balde ou apagou algo, o comprimento muda —
        // restaurar a base apagaria o trabalho dele. A sessão morre e o Trap novo simplesmente
        // não retro-aplica (ele clica Apply de novo).
        let live = self.flip_colorize.live.as_mut().expect("live is Some");
        if drawing.strokes.len() != live.base.len() + live.produced {
            self.flip_colorize.end_live();
            return;
        }
        drawing.strokes.clone_from(&live.base);
        live.produced = insert_regions(
            drawing,
            &live.palette,
            &live.seeds,
            precision,
            trap_px,
            squeeze,
        );
        live.trap = style.trap;
        live.bleed = style.colorize_bleed;
        // Como o Apply deferido: sem isto o `post_frame_undo` pularia o diff num frame sem
        // outro input, e o ajuste ficaria fora do passo. Preso o `held_button` do arrasto, os
        // frames intermediários seguem suprimidos — um passo por gesto.
        self.any_input_this_frame = true;
        self.title_dirty = true;
    }
}

#[cfg(test)]
#[path = "flip_colorize_tests.rs"]
mod tests;

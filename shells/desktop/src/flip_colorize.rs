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
use ph2d_editor::Job;
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

    /// **Há um ajuste ao vivo pendente?** — um corte em voo, ou um `(trap, bleed)` pedido
    /// que ainda não foi honrado.
    ///
    /// O `post_frame_undo` a consulta pelo MESMO motivo que consulta o `held_button`: *"um
    /// gesto em andamento muta a cada Move; espera o fim"*. Um recálculo pendente **é** o
    /// gesto não ter terminado — sem isto, soltar o slider registraria um passo com o
    /// resultado ANTIGO e a chegada do worker registraria um segundo, dando dois Ctrl+Z para
    /// um arrasto.
    #[must_use]
    pub(crate) fn live_busy(&self, style: Option<&FlipStyleSnapshot>) -> bool {
        let Some(live) = self.live.as_ref() else {
            return false;
        };
        live.job.is_some()
            || style.is_some_and(|s| s.trap != live.trap || s.colorize_bleed != live.bleed)
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

        let Some((oid, lid, did)) = crate::flip_autokey::target_drawing(
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
        // vivo restaura para reinserir sem empilhar (Trap/Bleed em tempo real); as `lines`
        // vêm junto porque o worker do ajuste ao vivo não pode ver o documento.
        let base = drawing.strokes.clone();
        let lines = boundaries(drawing);
        let regions = colorize_regions(&lines, &seeds, precision, trap_px, squeeze);
        let produced = install_regions(drawing, &lines, &palette, regions);
        if produced == 0 {
            drawing.strokes = base; // nada saiu — devolve o desenho intocado
            gfx.toasts.push(ph2d_editor::Toast::warning(
                "Colorize: no regions — scribble inside the closed shapes",
            ));
            self.title_dirty = true;
            return;
        }
        let mut frames = vec![LiveFrame {
            did,
            lines,
            base,
            produced,
        }];

        // **O ONION FILL** (fatia C3, `09 §5.2`): com chaves selecionadas na tira, o MESMO
        // rabisco colore todas. O que a C3 acrescenta ao multiframe do balde **não é o
        // range** (esse já existia) — é a **SEMENTE**: o balde replica um PONTO, e aqui o
        // artista rabisca por cima das poses EMPILHADAS e cada quadro é semeado pelo traço
        // inteiro. As sementes já estão em LOCAL e servem todos os quadros (a `w2l` é do
        // OBJETO); o que muda de quadro para quadro é a LINHA, então cada um resolve
        // sozinho — `09 §5.2` e o comentário gêmeo do `flip_fill`: *"N solves independentes;
        // a região pode mudar de forma, não há contorno a reaproveitar"*.
        //
        // **Os vizinhos falham em SILÊNCIO** (a política herdada do balde, `09 §5.2`): um
        // quadro em que a arte não fecha não pode derrubar o gesto nos outros, e o toast
        // fala pelo quadro ATIVO — que é onde o artista está olhando. A recusa acima já
        // aconteceu: se o ATIVO não produziu nada, nem chegamos aqui.
        //
        // **`falloff = false`**: colorir é op discreta, como o balde. Meia-cor não existe.
        let extra: Vec<DrawingId> = crate::flip_multiframe::targets(
            &gfx.flip,
            oid,
            lid,
            &playhead,
            strip.selected_keys(),
            (
                did,
                gfx.flip.object(oid).map_or(0, |o| o.frame_at(&playhead)),
            ),
            false,
        )
        .into_iter()
        .map(|t| t.did)
        .filter(|d| *d != did)
        .collect();
        frames.extend(colorize_frames(
            &mut gfx.flip,
            oid,
            &extra,
            &palette,
            &seeds,
            precision,
            trap_px,
            squeeze,
        ));

        // ✅ SÓ AGORA as sementes foram consumidas — o Apply teve sucesso. Um redo de rabisco
        // pós-Apply devolveria uma semente sem o contexto que a criou, então a fila de
        // removidos morre junto.
        self.flip_colorize.scribbles.clear();
        self.flip_colorize.popped.clear();

        // A operação fica VIVA: mexer no Trap/Bleed agora re-roda o corte em tempo real
        // (`flip_colorize_live_adjust`), sem clicar Apply de novo — em TODOS os quadros que
        // o gesto escreveu, senão os vizinhos ficariam presos no Trap da 1ª rodada e a tira
        // mostraria dois ajustes diferentes para uma operação só.
        self.flip_colorize.live = Some(LiveApply {
            palette,
            seeds,
            oid,
            frames,
            trap: style.trap,
            bleed: style.colorize_bleed,
            job: None,
        });
        self.title_dirty = true;
    }
}

/// A sessão viva (o ajuste Trap/Bleed pós-Apply + o worker) — irmão pelo teto de LOC.
#[path = "flip_colorize_live.rs"]
mod live;
use live::{LiveApply, LiveFrame};

/// O motor (conversões + a porta única de inserção) — irmão pelo teto de LOC do shell.
#[path = "flip_colorize_engine.rs"]
mod engine;
use engine::{colorize_frames, colorize_regions, install_regions, precision_and_trap};

#[cfg(test)]
#[path = "flip_colorize_tests.rs"]
mod tests;

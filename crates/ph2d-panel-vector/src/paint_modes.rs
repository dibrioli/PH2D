//! O seletor de MODO + os PARÂMETROS da forma em foco + as seções de TEXTO.
//!
//! Módulo irmão de `paint_sections` (teto de 600 LOC por arquivo de painel). O CATÁLOGO
//! de formas (categoria + grade de thumbnails) mora em `paint_catalog`.
//!
//! **Seis modos** (ADR-0112 + o 5º pill da reforma + o **Connect**): **Select** (o gizmo
//! manda) · **Node** (edita âncoras) · **Pen** (cria) · **Shape** (desenha a forma ATIVA
//! do catálogo) · **Text** · **Connect** (liga duas formas com uma linha que as segue).
//! As FORMAS não são modos — são um **catálogo**; escolher uma põe a tool em `Shape`, e
//! por isso o modo Shape precisa de um pill: sem ele a fileira de modos ficava TODA
//! apagada justamente enquanto se desenhava uma forma. O **conector**, ao contrário, É um
//! modo: a geometria dele não é autorada, é derivada da relação entre duas formas.
//!
//! Os campos de parâmetro vivem numa seção IRMÃ cujo **título é o NOME da forma em foco**
//! (`STAR`, `GEAR`, `SPEECH`…) — é assim que o usuário descobre a quem os campos
//! pertencem; soltos, como estavam, não diziam nada.

/// **As seções de TEXTO** (`TEXT` · `FONT` · `PARAGRAPH` · `AXES`) — irmão pelo teto de 600 LOC do
/// painel, e o corte é na fronteira que o cabeçalho deste arquivo já nomeava: aqui mora *que
/// ferramenta está na mão e com que parâmetros*, lá *como um texto se compõe*. As quatro partilham
/// o mesmo gate (`state::text_visible()`), então são um grupo e não quatro vizinhos.
#[path = "paint_text_sections.rs"]
mod text_sections;

use crate::ids;
use crate::paint_sections::BodyCtx;
use crate::paint_sections::LABEL_COL_W;
use crate::state;
use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::widget::panel_chrome::paint_segmented_button;
use ph2d_editor_core::widget::showcase::read_number_input;
use ph2d_editor_core::widget::{NumberInput, paint_number_input_with_buffer};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::{ColorToken, Spacing, TypeToken};
use ph2d_tool_vector::VectorStyleSnapshot;
use ph2d_tool_vector::params::{DrawMode, MarqueeShape};
use ph2d_tool_vector::shapes;

impl BodyCtx<'_> {
    /// Seção **TOOL** — os seis modos, numa grade de 3 colunas
    /// (`Select | Node | Pen` · `Shape | Text | Connect`).
    pub(crate) fn tool_section(&mut self, snap: &VectorStyleSnapshot, y: f32) -> f32 {
        let (y, collapsed) =
            self.section_header(ids::VECTOR_SECTION_TOOL, tr("panel.vector.section.tool"), y);
        if collapsed {
            return y;
        }
        let modes = [
            (
                ids::VECTOR_MODE_SELECT,
                tr("panel.vector.mode.select"),
                DrawMode::Select,
            ),
            (
                ids::VECTOR_MODE_NODE,
                tr("panel.vector.mode.node"),
                DrawMode::Node,
            ),
            (
                ids::VECTOR_MODE_PEN,
                tr("panel.vector.mode.pen"),
                DrawMode::Pen,
            ),
            // **Lápis** — ao lado da caneta de propósito: são os dois gestos de PRODUZIR
            // geometria à mão, e a vizinhança é o que faz o artista achar o novo.
            (
                ids::VECTOR_MODE_PENCIL,
                tr("panel.vector.mode.pencil"),
                DrawMode::Pencil,
            ),
            (
                ids::VECTOR_MODE_SHAPE,
                tr("panel.vector.mode.shape"),
                DrawMode::Shape,
            ),
            (
                ids::VECTOR_MODE_TEXT,
                tr("panel.vector.mode.text"),
                DrawMode::Text,
            ),
            (
                ids::VECTOR_MODE_CONNECT,
                tr("panel.vector.mode.connect"),
                DrawMode::Connect,
            ),
            (
                ids::VECTOR_MODE_BUILD,
                tr("panel.vector.mode.build"),
                DrawMode::Build,
            ),
            // **Fillet / Chamfer** — o 9º e 10º pills: arredondar / chanfrar quina por
            // clicar-e-arrastar. Consolidam a alça de raio (antes escondida no Node) e o toggle
            // Chamfer (antes na seção Vertex) numa dupla de ferramentas explícitas.
            (
                ids::VECTOR_MODE_FILLET,
                tr("panel.vector.mode.fillet"),
                DrawMode::Fillet,
            ),
            (
                ids::VECTOR_MODE_CHAMFER,
                tr("panel.vector.mode.chamfer"),
                DrawMode::Chamfer,
            ),
            // **Width** — ao lado dos pills de quina de propósito: os três editam um atributo
            // VIVO de uma forma que já existe, apontando-a no canvas.
            (
                ids::VECTOR_MODE_WIDTH,
                tr("panel.vector.mode.width"),
                DrawMode::Width,
            ),
            // **Corte** — o 13º pill, e fica no fim desta vizinhança de propósito: os quatro
            // (Fillet · Chamfer · Width · Cut) editam uma forma que JÁ existe apontando-a no
            // canvas, em vez de produzir geometria nova.
            //
            // ⚠️ Ele é UM pill onde havia dois (Tesoura + Faca). As duas produziam peças ABERTAS,
            // e a lei do produto é que uma forma fechada cortada dá formas FECHADAS — então não
            // eram duas ferramentas, eram duas metades de uma que não existia ainda.
            (
                ids::VECTOR_MODE_CUT,
                tr("panel.vector.mode.cut"),
                DrawMode::Cut,
            ),
            // **Moldura** — o 14º pill, e fica no FIM porque é o único que produz um CONTÊINER:
            // os anteriores desenham ou editam uma forma, este cria o lugar onde as formas moram.
            (
                ids::VECTOR_MODE_FRAME,
                tr("panel.vector.mode.frame"),
                DrawMode::Frame,
            ),
            // NOTA: o **Pick Shapes** (`VECTOR_MODE_PICKBLEND`) NÃO fica aqui — ele é uma etapa do
            // Blend (escolher as formas na ordem), e mora na seção BLEND, ao lado do botão que as
            // liga (ADR-0128 C2b). É um modo de tool, mas seu botão vive lá, não nesta fileira.
        ];
        let cols = 3usize;
        let y = self.button_grid(y, cols, modes.len(), |i| {
            let (id, label, m) = modes[i];
            (id, label, snap.mode == m)
        });
        let y = self.cut_line_row(snap, y);
        self.marquee_row(snap, y)
    }

    /// **A FORMA do marquee** — `Box | Lasso`, a região que o arrasto no vazio desenha.
    ///
    /// ⚠️ **Só no modo Node, e colado na fileira TOOL** — o mesmo desenho (e a mesma razão) da
    /// linha do CORTE logo acima: o gesto COMEÇA no pill (estar em Node) e este par diz o que ele
    /// faz; separar as duas metades por meia tela é como o artista arrasta um retângulo e nunca
    /// descobre que havia um laço.
    ///
    /// ⚠️ **Aqui e não na seção VERTEX**, que seria o vizinho temático óbvio: aquela seção só
    /// existe **com um vértice já selecionado** (`current_vertex_type()`), então um controle de
    /// *como selecionar* moraria lá exatamente onde não se precisa dele — invisível no estado em
    /// que o artista o procura, que é antes de ter selecionado o que quer que seja.
    ///
    /// O chip é a metade PEGAJOSA; o **Ctrl** troca a de um gesto só (`MarqueeShape::for_gesture`).
    fn marquee_row(&mut self, snap: &VectorStyleSnapshot, y: f32) -> f32 {
        if snap.mode != DrawMode::Node {
            return y;
        }
        let m = snap.marquee;
        self.segmented(
            tr("panel.vector.marquee"),
            &[
                (
                    ids::VECTOR_MARQUEE_BOX,
                    tr("panel.vector.marquee.box"),
                    m == MarqueeShape::Box,
                ),
                (
                    ids::VECTOR_MARQUEE_LASSO,
                    tr("panel.vector.marquee.lasso"),
                    m == MarqueeShape::Lasso,
                ),
            ],
            y,
        )
    }

    /// **Os dois botões da LINHA DE CORTE** — executar e descartar.
    ///
    /// Ficam aqui, colados na fileira TOOL, e não numa seção própria lá em baixo: o gesto do
    /// corte COMEÇA no pill (escolher `Cut`) e termina num botão, e separar as duas metades por
    /// meia tela é como o artista desenha a lâmina e não descobre o que fazer com ela.
    ///
    /// ⚠️ **Só no modo Cut, e só com lâmina desenhada** — as duas condições (Enio, 2026-07-31:
    /// *"as opções Cut e Discard Cut Line só aparecem se o botão Cut estiver checado"*).
    ///
    /// Um `Cut` sem linha não tem com que cortar e um `Discard` sem linha não tem o que descartar;
    /// e fora do modo Cut os dois seriam controles de uma ferramenta que não está na mão. Pintados
    /// assim seriam botões mudos no estado mais comum de todos — o defeito que o Average e os dois
    /// pills desta wave já pagaram no mesmo dia.
    fn cut_line_row(&mut self, snap: &VectorStyleSnapshot, y: f32) -> f32 {
        if snap.mode != DrawMode::Cut || !state::cut_line_exists() {
            return y;
        }
        let gap = Spacing::Sm.px();
        let w = ((self.inner_w - gap) / 2.0).max(1.0);
        self.row2(
            w,
            gap,
            [
                (ids::VECTOR_CUT_APPLY, tr("panel.vector.cut.apply")),
                (ids::VECTOR_CUT_DISCARD, tr("panel.vector.cut.discard")),
            ],
            y,
        )
    }

    pub(crate) fn shape_params_section(&mut self, snap: &VectorStyleSnapshot, y: f32) -> f32 {
        let Some(focus) = crate::shape_focus::resolved(snap) else {
            return y;
        };
        let desc = shapes::desc(focus);
        let (mut y, collapsed) =
            self.section_header(ids::VECTOR_SECTION_SHAPE_PARAMS, desc.label, y);
        if collapsed {
            return y;
        }
        if desc.fields.is_empty() {
            paint_text(
                self.text_system,
                self.scene,
                tr("panel.vector.shape.no_params"),
                self.inner_x,
                y + (self.row_h - TypeToken::Sm.px()) * 0.5,
                TypeToken::Sm.px(),
                self.inner_w,
                resolve(ColorToken::Text3, self.theme),
            );
            return y + self.row_h + self.row_gap;
        }
        for (i, f) in desc.fields.iter().enumerate() {
            let id = ph2d_editor_core::ids::vector_shape_field_id(i);
            y = match f.unit {
                // Uma ESCOLHA não é um número. Um campo mostrando "Viewed: 1" não diz nada;
                // um botão dizendo "From below" diz tudo — e clicar nele cicla.
                shapes::FieldUnit::Choice(_) => {
                    let cur = self.store.number_value(id).unwrap_or(0.0);
                    let text = shapes::choice_label(focus, i, cur).unwrap_or("");
                    // O HIT vai no gêmeo botão; o valor continua morando no slot numérico.
                    let btn = ph2d_editor_core::ids::vector_shape_choice_id(i);
                    self.labeled_choice_button(f.label, btn, text, y)
                }
                _ => self.labeled_number_field(f.label, id, f.step, y),
            };
        }
        y
    }

    /// Um campo de ESCOLHA: rótulo + um botão que mostra a opção corrente e CICLA ao clique.
    /// Ocupa a mesma coluna da caixa numérica, então a seção continua alinhada.
    ///
    /// `pub(crate)`: a seção do CONECTOR (`paint_connector`) pinta o campo **Route** com este
    /// mesmo desenho — ela é irmã da seção de parâmetros de forma, não um enxerto.
    pub(crate) fn labeled_choice_button(
        &mut self,
        label: &str,
        id: ph2d_a11y::NodeId,
        current: &str,
        y: f32,
    ) -> f32 {
        let gap = Spacing::Sm.px();
        paint_text(
            self.text_system,
            self.scene,
            label,
            self.inner_x,
            y + (self.row_h - TypeToken::Sm.px()) * 0.5,
            TypeToken::Sm.px(),
            LABEL_COL_W,
            resolve(ColorToken::Text2, self.theme),
        );
        let field_x = self.inner_x + LABEL_COL_W + gap;
        let field_w = (self.inner_w - LABEL_COL_W - gap).max(1.0);
        let rect = Rect::new(field_x, y, field_w, self.row_h);
        self.hit_index.register(id, rect);
        paint_segmented_button(
            rect,
            current,
            false,
            self.store.button_visual(id),
            self.scene,
            self.text_system,
            self.theme,
        );
        y + self.row_h + self.row_gap
    }

    /// Uma grade de botões segmentados de `cols` colunas — o desenho compartilhado do
    /// seletor de modo e da grade de tipos do catálogo (que empilha o thumbnail por cima
    /// da mesma chrome).
    pub(crate) fn button_grid(
        &mut self,
        y: f32,
        cols: usize,
        n: usize,
        item: impl Fn(usize) -> (ph2d_a11y::NodeId, &'static str, bool),
    ) -> f32 {
        if n == 0 {
            return y;
        }
        let gap = Spacing::Sm.px();
        let w = ((self.inner_w - gap * (cols as f32 - 1.0)) / cols as f32).max(1.0);
        for i in 0..n {
            let (id, label, active) = item(i);
            let rx = self.inner_x + (i % cols) as f32 * (w + gap);
            let ry = y + (i / cols) as f32 * (self.row_h + gap);
            let rect = Rect::new(rx, ry, w, self.row_h);
            let st = self.store.button_visual(id);
            paint_segmented_button(
                rect,
                label,
                active,
                st,
                self.scene,
                self.text_system,
                self.theme,
            );
            self.hit_index.register(id, rect);
        }
        let rows = n.div_ceil(cols) as f32;
        y + rows * self.row_h + (rows - 1.0) * gap + self.row_gap
    }

    /// Um campo numérico rotulado (`<rótulo> [ valor ]`), largura cheia — os parâmetros
    /// de forma, os eixos de variação da fonte e os campos do CONECTOR (Jetty / Spread)
    /// compartilham este desenho. **Caixa, não slider:** as faixas variam demais entre
    /// formas (3 lados · 500 px de raio · 360°) para um knob servir a todas.
    pub(crate) fn labeled_number_field(
        &mut self,
        label: &str,
        id: ph2d_a11y::NodeId,
        step: f64,
        y: f32,
    ) -> f32 {
        let gap = Spacing::Sm.px();
        paint_text(
            self.text_system,
            self.scene,
            label,
            self.inner_x,
            y + (self.row_h - TypeToken::Sm.px()) * 0.5,
            TypeToken::Sm.px(),
            LABEL_COL_W,
            resolve(ColorToken::Text2, self.theme),
        );
        let field_x = self.inner_x + LABEL_COL_W + gap;
        let field_w = (self.inner_w - LABEL_COL_W - gap).max(1.0);
        let rect = Rect::new(field_x, y, field_w, self.row_h);
        self.hit_index.register(id, rect);
        let (st, value, buffer, caret, anchor) = read_number_input(self.store, id);
        let input = NumberInput::new(id, "", value)
            .step(step)
            .visual((st, self.store.hover_live(id)));
        paint_number_input_with_buffer(
            &input,
            Some(buffer),
            caret,
            anchor,
            rect,
            self.scene,
            self.text_system,
            self.theme,
        );
        y + self.row_h + self.row_gap
    }
}

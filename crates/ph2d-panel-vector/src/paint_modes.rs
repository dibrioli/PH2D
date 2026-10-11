//! O seletor de MODO + as linhas partilhadas (escolha, número, grelha) + as seções de TEXTO.
//!
//! Módulo irmão de `paint_sections` (teto de 600 LOC por arquivo de painel). O CATÁLOGO
//! de formas mora no flyout da coluna ① (D10; os contornos: `shape_outlines`).
//!
//! **Seis modos** (ADR-0112 + o 5º pill da reforma + o **Connect**): **Select** (o gizmo
//! manda) · **Node** (edita âncoras) · **Pen** (cria) · **Shape** (desenha a forma ATIVA
//! do catálogo) · **Text** · **Connect** (liga duas formas com uma linha que as segue).
//! As FORMAS não são modos — são um **catálogo**; escolher uma põe a tool em `Shape`, e
//! por isso o modo Shape precisa de um pill: sem ele a fileira de modos ficava TODA
//! apagada justamente enquanto se desenhava uma forma. O **conector**, ao contrário, É um
//! modo: a geometria dele não é autorada, é derivada da relação entre duas formas.
//!
//! ⛔ F9 (11/10): os PARÂMETROS da forma em foco saíram do painel — alças no canvas e o número
//! exacto na fila ② (`bars_row::shape_chips`), como a Context Toolbar do Affinity.

/// **As seções de TEXTO** (`TEXT` · `FONT` · `PARAGRAPH` · `AXES`) — irmão pelo teto de 600 LOC do
/// painel, e o corte é na fronteira que o cabeçalho deste arquivo já nomeava: aqui mora *que
/// ferramenta está na mão e com que parâmetros*, lá *como um texto se compõe*. As quatro partilham
/// o mesmo gate (`state::text_visible()`), então são um grupo e não quatro vizinhos.
#[path = "paint_text_sections.rs"]
mod text_sections;

use crate::paint_sections::BodyCtx;
use crate::paint_sections::label_col_w;
use ph2d_editor_core::paint::resolve;
use ph2d_editor_core::widget::panel_chrome::{
    paint_segmented_button, paint_segmented_button_in_group,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, TypeToken};

impl BodyCtx<'_> {
    /// Um campo de ESCOLHA: rótulo + um botão que mostra a opção corrente e CICLA ao clique.
    /// Ocupa a mesma coluna da caixa numérica, então a seção continua alinhada.
    ///
    /// `pub(crate)`: a seção do CONECTOR (`paint_connector`) pinta o campo **Route** com este
    /// desenho.
    pub(crate) fn labeled_choice_button(
        &mut self,
        label: &str,
        id: ph2d_a11y::NodeId,
        current: &str,
        y: f32,
    ) -> f32 {
        ph2d_editor_core::widget::paint_property_label(
            self.text_system,
            self.scene,
            label,
            self.inner_x,
            y + (self.row_h - TypeToken::Sm.px()) * 0.5,
            TypeToken::Sm.px(),
            label_col_w(self.inner_x, self.inner_w),
            resolve(ColorToken::Text2, self.theme),
        );
        let rect = ph2d_editor_core::panel::value_col(self.inner_x, self.inner_w, y, self.row_h);
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
        // ⭐⭐ **A grelha é UM corpo** — a lei do Blender que o dono apontou (foto do `TOOL` do
        //    Vector, 2026-09-06): as peças encostam e só os quatro cantos do BLOCO arredondam.
        //
        // ⛔⛔ **E a QUEBRA sai das PALAVRAS, com `cols` a ser só o TECTO** (2026-09-23). A grelha
        //    repartia sempre em `cols` partes iguais, e no degrau estreito (o dock no mínimo, onde o
        //    dono trabalha) `Bucket`, `Chamfer` e `Connect` saíam `Buc…`/`Cha…`/`Con…` numa célula de
        //    `55 px` com a fileira de dois a caber. A porta que o resto da casa já usa para isto é a
        //    `wrapped_cells_for` — a mesma lei, com o `max_cols` como tecto de PRODUTO — e a doc
        //    dela descreve à letra a 1.ª redacção desta: *«um `cols` literal … repartir bem uma
        //    fileira mal formada troca de vítima»*.
        let items: Vec<_> = (0..n).map(&item).collect();
        let labels: Vec<&str> = items.iter().map(|t| t.1).collect();
        let block = ph2d_editor_core::widget::wrapped_cells_for(
            Rect::new(self.inner_x, y, self.inner_w, 0.0),
            &labels,
            self.row_h,
            cols,
            self.text_system,
        );
        let cells: Vec<_> = block.iter().flatten().copied().collect();
        for (i, &(id, label, active)) in items.iter().enumerate() {
            let (rect, cell) = cells[i];
            let st = self.store.button_visual(id);
            paint_segmented_button_in_group(
                rect,
                label,
                active,
                st,
                self.scene,
                self.text_system,
                self.theme,
                cell,
            );
            self.hit_index.register(id, rect);
        }
        // ⭐ **A grelha é UMA escolha** (*uma de N*, com a acesa a dizer qual) — os três
        //    chamadores (os modos da ferramenta, o *Pick Shapes* do blend, os tamanhos da moldura)
        //    passam todos a bandeira `active`. Declará-la é o que faz o censo das entradas ler UM
        //    controlo e não `15` comandos soltos (medido 2026-09-24: o `vector` lia `45` comandos, e
        //    `15` eram estas peças).
        ph2d_editor_core::widget::composto::grupo(items.iter().map(|t| t.0));
        // ⚠️ **A altura sai da MESMA porta que dispôs as peças** — a conta à mão que estava aqui
        // somava o vão antigo, e um contentor medido por uma regra e preenchido por outra escreve
        // a fileira seguinte por cima desta.
        y + ph2d_editor_core::widget::grid_height(block.len(), self.row_h) + self.row_gap
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
        self.with_rows(|r| r.labeled_number_field(label, id, step, y))
    }
}

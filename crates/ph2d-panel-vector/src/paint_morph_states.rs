//! ⭐ **A seção MORPH STATES** (plano 32 W4/W7) — *entre que formas este objecto transita, e o que
//! dispara cada passagem*.
//!
//! # Seção PRÓPRIA — e a W4 tinha-a posto na de outra pessoa
//!
//! ⛔⛔ Até 2026-08-25 estas linhas eram uma **sub-lista dentro da seção `States`**, a das poses de
//! UI e do Smart Animate. O argumento era o ADR-0166 (*o Inspector mostra o que o objecto TEM*) mais
//! *"um objecto raramente é as duas coisas"*. Os dois são verdadeiros e **nenhum deles era a
//! pergunta**: o efeito prático foi o cabeçalho de uma feature **já entregue** passar a aparecer por
//! causa de outra — Enio leu como contaminação, e era.
//!
//! ⚠️ *A lei diz o que MOSTRAR, nunca ONDE.* Duas features com donos, histórias e gates diferentes
//! debaixo de um cabeçalho só é uma porta a mais na seção de quem chegou primeiro; e quem chegou
//! primeiro é quem paga a regressão.
//!
//! # UM BOTÃO faz o conjunto, e as setas são GERADAS (W8)
//!
//! Enio, 2026-08-25: *"o usuário seleciona todas as peças... com o clique de um único botão um
//! objeto novo surge na hierarquia tendo como filhos as shapes escolhidas. Todas as setas são
//! atribuídas automaticamente cobrindo todas as morphs possíveis... As setas são virtuais e ninguém
//! jamais vê."*
//!
//! ⇒ esta seção tem **duas faces**, e nunca uma terceira:
//!
//! - **sem máquina**, com 2+ formas escolhidas: o botão que as transforma no conjunto;
//! - **com máquina**: as `n(n-1)` transições, cada uma com a acção que a dispara.
//!
//! # ⭐⭐⭐ E a tecla pertence à FORMA (W10)
//!
//! Enio, no smoke seguinte: *"em vez de um evento para cada transição, melhor seria um evento por
//! shape (…) se a seta para cima leva ao retângulo azul, independente de que forma estiver ativa no
//! momento, a seta para cima vai levar ao retângulo azul (…) assim reduzimos o número de transições
//! no painel para o número de formas envolvidas."*
//!
//! ⇒ a lista tem **uma linha por FORMA**, e não uma por passagem: de `n(n-1)` para `n`. Com 9
//! formas, de **72** linhas para **9**.
//!
//! ⛔ **Não há gesto de acrescentar linha, e não há lixeira.** A lista **É** o conjunto de formas do
//! objecto — tirar uma seria tirar uma forma do conjunto, que é outro gesto e ainda não existe.
//! *Desligar uma forma é tirar-lhe a tecla*, e uma forma sem tecla existe e nunca é alcançada.
//!
//! # Duas linhas por forma, e a segunda é a que evita a mentira
//!
//! A primeira diz **que forma é**; a segunda traz a **tecla que leva até ela**. É o mesmo corte do
//! `paint_signals`, e pela mesma razão: espremer as duas numa linha só daria um chip de meia dúzia
//! de caracteres, e o artista não conseguiria ler a acção que escolheu.
//!
//! ⚠️ **A condição é um MENU das acções do Input Map, nunca um campo de texto.** Um nome digitado à
//! mão pode não existir, e uma seta que espera uma acção inexistente **nunca dispara** — sem uma
//! palavra na tela a dizer porquê. *Um modelo que aceita o que o painel não mostra produz estado
//! inalcançável.*

use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::widget::{
    Button, ButtonKind, Dropdown, DropdownOption, paint_button, paint_dropdown_chip,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::Spacing;

use crate::ids;
use crate::paint_sections::{BodyCtx, LABEL_COL_W};
use crate::state::{self, MorphStatesState};

impl BodyCtx<'_> {
    /// **A seção MORPH STATES** — o cabeçalho próprio, e some inteira quando a seleção não tem
    /// máquina nenhuma.
    ///
    /// ⚠️ **Ela pergunta só ao `morph_states_state`**, e é isso que a mantém fora do caminho da
    /// seção de poses: as duas nunca se consultam, então nenhuma pode fazer a outra aparecer.
    pub(crate) fn morph_states_section(&mut self, y: f32) -> f32 {
        let Some(s) = state::morph_states_state() else {
            return y;
        };
        let (y, collapsed) = self.section_header(
            ids::VECTOR_SECTION_MORPH_STATES,
            tr("panel.vector.section.morph_states"),
            y,
        );
        if collapsed {
            return y;
        }
        self.morph_shape_rows(&s, y)
    }

    /// **As formas desta máquina** — ou a face que oferece o botão que a cria.
    fn morph_shape_rows(&mut self, s: &MorphStatesState, y: f32) -> f32 {
        // ⭐ **A FACE DE CRIAÇÃO vem PRIMEIRO quando não há máquina**, e ela é a seção inteira: sem
        // conjunto não há transição nenhuma de que falar, e um cabeçalho «Transitions» por cima de
        // nada leria como avaria.
        if s.rows.is_empty() {
            return self.morph_make_face(s, y);
        }

        // ⭐⭐ **O INTERRUPTOR DA PRÉ-VISUALIZAÇÃO vem PRIMEIRO**, e a posição é o argumento: ele
        // é o que decide de quem é o teclado, e essa pergunta antecede qualquer transição da lista.
        let mut y = self.morph_preview_row(s.preview, y);
        y = self.label_line(tr("panel.vector.morph.arrows"), y);

        // ⭐ **O READOUT: em que forma a máquina está AGORA.**
        //
        // ⚠️ Ele sai da MESMA máquina que escreve o mundo — um readout derivado noutro sítio diria
        // uma forma e a cena mostraria outra. É o argumento, palavra por palavra, do `live` da
        // seção de poses.
        if let Some(cur) = s.current.as_ref() {
            y = self.label_line(&format!("{} {cur}", tr("panel.vector.morph.current")), y);
        }

        let shown = s.rows.len().min(ids::MAX_MORPH_STATES);
        for (i, row) in s.rows.iter().enumerate().take(shown) {
            y = self.shape_name_row(row, y);
            y = self.shape_key_row(i, row, &s.actions, y);
        }
        // ⚠️ **Acima do tecto a lista DIZ o que ficou de fora**, em vez de o esconder: o grafo
        // continua a funcionar (a máquina alcança todas as formas), e é o painel que não as
        // mostra. Um corte silencioso far-lhe-ia procurar uma forma que ele tem a certeza de ter
        // escolhido.
        if s.rows.len() > shown {
            y = self.label_line(
                &format!(
                    "{} {}",
                    s.rows.len() - shown,
                    tr("panel.vector.morph.beyond")
                ),
                y,
            );
        }
        y
    }

    /// ⭐⭐ **O interruptor da PRÉ-VISUALIZAÇÃO**, e — quando ligada — a linha que diz como sair.
    ///
    /// ⚠️ **O botão troca de ESTADO, nunca de rótulo** (a mesma escolha do irmão das poses): um
    /// botão cujo texto alterna entre *"Preview"* e *"Exit"* obriga a ler para saber onde se está,
    /// enquanto um aceso se lê de relance.
    ///
    /// ⚠️ **A porta de saída é ANUNCIADA, e aqui ela é obrigatória:** este modo toma o **teclado**,
    /// então o artista que não soubesse sair tentaria carregar em teclas — que é exactamente o que
    /// o modo consome. Um modo que come a própria tentativa de sair lê-se como travado.
    fn morph_preview_row(&mut self, on: bool, y: f32) -> f32 {
        let rect = Rect::new(self.inner_x, y, self.inner_w, self.row_h);
        // ⚠️ O *ligado* é o **KIND**, não o `ButtonState`: aquele descreve o rato (hover, press) e
        // o kind descreve o que o botão É. Escrever *ligado* no `ButtonState` faria o aceso
        // desaparecer no instante em que o cursor passasse por cima dele.
        let btn = Button::new(ids::VECTOR_MORPH_PREVIEW, tr("panel.vector.morph.preview"))
            .kind(if on {
                ButtonKind::Accent
            } else {
                ButtonKind::Default
            })
            .visual(self.store.button_visual(ids::VECTOR_MORPH_PREVIEW));
        paint_button(&btn, rect, self.scene, self.text_system, self.theme);
        self.hit_index.register(ids::VECTOR_MORPH_PREVIEW, rect);
        let y = y + self.row_h + Spacing::Xs.px();
        if on {
            return self.label_line(tr("panel.vector.morph.preview.on"), y);
        }
        y
    }

    /// ⭐ **A face de CRIAÇÃO** — o botão que transforma a seleção num conjunto de estados, ou a
    /// frase que diz o que falta para ele existir.
    ///
    /// ⚠️ **Três respostas, e as três são diferentes de propósito:** *escolha mais formas* ·
    /// *escolheu formas a mais* · *aqui está o botão*. Colapsar as duas primeiras numa só faria o
    /// artista que escolheu doze formas ler *"escolha duas ou mais"* e concluir que o app está
    /// partido.
    fn morph_make_face(&mut self, s: &MorphStatesState, y: f32) -> f32 {
        if s.can_make < 2 {
            return self.label_line(tr("panel.vector.morph.need_shapes"), y);
        }
        if s.can_make > ids::MAX_MORPH_STATES {
            // ⚠️ **A frase traz o TETO**, e o teto vem da constante — nunca de um número escrito na
            // tabela de i18n, que envelheceria no dia em que a medição mudasse.
            return self.label_line(
                &format!(
                    "{} {}.",
                    tr("panel.vector.morph.too_many"),
                    ids::MAX_MORPH_STATES
                ),
                y,
            );
        }
        // ⚠️ **A promessa é a MESMA lei do produto, dita antes do clique** — e desde a W10 ela é
        // *«uma tecla por forma»*, não uma contagem de passagens. O botão promete `n` linhas, e é
        // `n` linhas que a lista mostra. ⛔ A conta `n(n-1)` que vivia aqui morreu com o modelo.
        let n = s.can_make;
        let y = self.label_line(&format!("{n} {}", tr("panel.vector.morph.make.shapes")), y);
        self.action_button(
            ids::VECTOR_MORPH_STATES_MAKE,
            tr("panel.vector.morph.make"),
            y,
        )
    }

    /// A linha de CIMA: **o nome da forma**, e o realce de quem está na tela AGORA.
    ///
    /// ⭐⭐ **Era `de -> para` até 2026-08-25** (W10): a tecla pertence ao DESTINO, então a linha
    /// deixou de precisar da origem — ela vale de qualquer forma. É isso que encolhe a lista de
    /// `n(n-1)` para `n`.
    ///
    /// ⛔ **Não há lixeira, e a ausência é a lei:** a lista É o conjunto de formas do objecto.
    /// Apagar uma linha seria tirar uma forma do conjunto, que é outro gesto (e ele ainda não
    /// existe). *Desligar uma forma é tirar-lhe a tecla* — o «—» do menu abaixo —, e uma forma sem
    /// tecla existe e nunca é alcançada.
    fn shape_name_row(&mut self, row: &state::MorphShapeRow, y: f32) -> f32 {
        let gap = Spacing::Xs.px();
        // ⭐ **A forma VIVA diz-se pelo texto**, porque é a única linha da lista que descreve o
        // presente e não uma regra. Sem marca nenhuma, a lista não responde à pergunta que o
        // artista faz enquanto toca: *em qual delas estou?*
        //
        // ⛔ **ASCII**, e não uma seta tipográfica: a fonte da casa não cobre o bloco de setas do
        // Unicode (U+2190..U+21FF) e o glifo sairia como uma caixa vazia — há gate a varrer isto
        // (`no_tofu_glyphs`), e ele já mordeu três vezes neste repo.
        let shown = if row.live {
            format!("{} {}", tr("panel.vector.morph.live_mark"), row.to)
        } else {
            row.to.clone()
        };
        self.label_line_in(&shown, Rect::new(self.inner_x, y, self.inner_w, self.row_h));
        y + self.row_h + gap
    }

    /// A linha de BAIXO: a CONDIÇÃO, num menu das acções do Input Map.
    fn shape_key_row(
        &mut self,
        i: usize,
        row: &state::MorphShapeRow,
        actions: &[String],
        y: f32,
    ) -> f32 {
        let gap = Spacing::Xs.px();
        self.label_line_in(
            tr("panel.vector.morph.reached_by"),
            Rect::new(self.inner_x, y, LABEL_COL_W, self.row_h),
        );
        let chip = Rect::new(
            self.inner_x + LABEL_COL_W + gap,
            y,
            (self.inner_w - LABEL_COL_W - gap).max(1.0),
            self.row_h,
        );
        let id = ids::morph_shape_key_id(i);
        let open = matches!(
            self.store.get(id),
            Some(InteractiveState::Dropdown { open: true, .. })
        );
        // ⚠️ **Vazio mostra o traço, e não uma string vazia.** Uma célula em branco lê-se como um
        // controlo por carregar; o traço diz *"sem condição, de propósito"*.
        let shown = if row.when.is_empty() {
            tr("panel.vector.morph.when.none")
        } else {
            row.when.as_str()
        };
        let dd = Dropdown::new(id, "", vec![DropdownOption::new(id, (), shown)])
            .selected(())
            .open(open)
            .visual(self.store.dropdown_visual(id));
        paint_dropdown_chip(&dd, chip, self.scene, self.text_system, self.theme);
        self.hit_index.register(id, chip);
        if open {
            let _ = actions;
            state::set_pending_morph_key_dd(Some((i, chip)));
        }
        y + self.row_h + gap
    }
}

/// **O menu das acções do Input Map** para a tecla da forma `row` — pintado no passe DIFERIDO,
/// por cima de todas as seções. Espelho exacto do `paint_filters_blend::paint_blend_popover`.
///
/// ⚠️ **A opção `0` é o «—»** (sem tecla): tirá-la tem de ser um gesto, e desde a W10 é **a única
/// maneira de desligar uma forma** — a lista não tem lixeira, porque ela É o conjunto de formas.
///
/// ⚠️ **As acções saem da lista PUBLICADA**, nunca de uma leitura do painel: elas são conteúdo
/// autorado do projecto, e uma segunda leitura aqui envelheceria no dia em que ele criasse uma.
pub(crate) fn paint_when_popover(
    ctx: &mut ph2d_editor_core::panel::PaintCtx,
    row: usize,
    chip: Rect,
    theme: ph2d_tokens::Theme,
) {
    use ph2d_editor_core::widget::{
        DROPDOWN_SCROLLBAR_ID, paint_dropdown_popover_scrolled, scrollbar_is_needed,
        scrollbar_track_rect,
    };
    let Some(s) = state::morph_states_state() else {
        return;
    };
    let Some(shape_row) = s.rows.get(row) else {
        return;
    };
    let id = ids::morph_shape_key_id(row);
    // O «—» à frente, e depois as acções — ATÉ ao pool de ids que o `populate` registou.
    let mut labels: Vec<&str> = vec![tr("panel.vector.morph.when.none")];
    labels.extend(
        s.actions
            .iter()
            .map(String::as_str)
            .take(ids::MAX_MORPH_ACTIONS - 1),
    );
    let sel = labels
        .iter()
        .position(|l| *l == shape_row.when)
        .unwrap_or(0);
    let options: Vec<DropdownOption<usize>> = labels
        .iter()
        .enumerate()
        .map(|(i, l)| DropdownOption::new(ids::morph_shape_key_option_id(row, i), i, *l))
        .collect();
    let dd = Dropdown::new(id, "", options).selected(sel).open(true);

    let panel = dd.popover_rect_clamped(chip, ctx.layout.popover_region());
    let content_h = dd.content_height(chip.h);
    let visible_h = panel.h;
    let max_scroll = (content_h - visible_h).max(0.0);
    {
        let store = ctx.host.store_mut();
        store.set_dropdown_popover(id, panel);
        store.set_panel_content_h(id, content_h);
        store.set_panel_visible_h(id, visible_h);
        if store.panel_scroll(id) > max_scroll {
            store.set_panel_scroll(id, max_scroll);
        }
    }
    let scroll = ctx.host.store().panel_scroll(id).clamp(0.0, max_scroll); // CLAMP-OK: 0.0 literal; max_scroll is a non-negative px extent
    paint_dropdown_popover_scrolled(
        &dd,
        chip,
        panel,
        scroll,
        ctx.host
            .store()
            .scrollbar_visual_for(DROPDOWN_SCROLLBAR_ID, Some(id)),
        ctx.scene,
        ctx.text_system,
        theme,
    );
    // Hit-register só a parte VISÍVEL de cada linha — a barra de rolagem é o alvo do arrasto.
    let hit_index = ctx.host.hit_index_mut();
    for i in 0..labels.len() {
        let r = dd.option_rect_in_scrolled(chip, panel, i, scroll);
        let top = r.y.max(panel.y);
        let bot = (r.y + r.h).min(panel.y + panel.h);
        if bot - top >= 1.0 {
            hit_index.register(
                ids::morph_shape_key_option_id(row, i),
                Rect::new(r.x, top, r.w, bot - top),
            );
        }
    }
    if scrollbar_is_needed(content_h, visible_h) {
        ctx.host
            .hit_index_mut()
            .register(DROPDOWN_SCROLLBAR_ID, scrollbar_track_rect(panel));
    }
}

//! O que a família diz para fora sem mexer em nada: a barra do assistente e as SONDAS dos testes e
//! da auditoria (o que cada aba mostra e pintou no último quadro).

use crate::docs::{CodeDocs, GrammarProbe};
use ph2d_code_syntax::HighlightKind;
use ph2d_script::ScriptId;

impl CodeDocs {
    /// O que a barra diz do assistente de código no script aberto (`None` = pronto ou desligado).
    #[must_use]
    pub fn assist_text(&self) -> Option<String> {
        self.intel
            .assist()
            .map(|a| ph2d_panel_code::assist_line(a).0)
    }

    /// ⚠️ SONDA: a razão crua do assistente não estar pronto.
    #[must_use]
    pub fn assist_reason(&self) -> Option<String> {
        self.intel.reason().map(str::to_owned)
    }

    /// ⚠️ SONDA: os diagnósticos do assistente para o texto que ele tem AGORA do script `id`, e a
    /// versão desse texto.
    #[must_use]
    pub fn assist_diagnostics(
        &self,
        id: ScriptId,
    ) -> (Option<u64>, Option<Vec<ph2d_code_intel::Diag>>) {
        (
            self.intel.version(id),
            self.intel.diagnostics(id).map(<[_]>::to_vec),
        )
    }

    /// O texto que o editor mostra do script `id` quando a aba está fechada com ● (`None` = não
    /// estacionado).
    #[must_use]
    pub fn parked_text(&self, id: ScriptId) -> Option<String> {
        self.parked.get(&id).map(|p| p.buffer.text().to_string())
    }

    /// ⚠️ SONDA: os sublinhados de problema que a aba do script `id` pintou no último quadro.
    #[must_use]
    pub fn underlines_of(&self, id: ScriptId) -> Vec<ph2d_panel_code::Underline> {
        self.docs
            .iter()
            .find(|d| d.script == id)
            .map(|d| d.view.last_underlines().to_vec())
            .unwrap_or_default()
    }

    /// ⚠️ SONDA: a lista de completação que a aba do script `id` mostra agora (os rótulos, pela
    /// ordem, e o índice da seleccionada); `None` = fechada.
    #[must_use]
    pub fn completion_of(&mut self, id: ScriptId) -> Option<(Vec<String>, usize)> {
        let d = self.docs.iter_mut().find(|d| d.script == id)?;
        let head = d.buffer.selection().primary().head;
        let list = d.complete.shown(d.buffer.text(), head)?;
        let s = d.complete.session.as_ref()?;
        let items = s.items.as_ref()?;
        Some((
            list.iter().map(|(i, _)| items[*i].label.clone()).collect(),
            s.selected,
        ))
    }

    /// ⚠️ SONDA: as acções de contexto (I7) que a aba do script `id` mostra agora — os títulos, o
    /// índice da seleccionada e a prévia dela; `None` = fechadas ou à espera do assistente.
    #[must_use]
    pub fn actions_of(&self, id: ScriptId) -> Option<(Vec<String>, usize, Vec<String>)> {
        let d = self.docs.iter().find(|d| d.script == id)?;
        let s = d.actions_shown()?;
        Some((
            s.rows.into_iter().map(|r| r.label).collect(),
            s.selected,
            s.preview.into_iter().map(|l| l.text).collect(),
        ))
    }

    /// ⚠️ SONDA: as linhas que a lista da aba `id` pintou no último quadro (índice, rótulo,
    /// seleccionada) e o centro de cada uma (onde o clique a aceita).
    #[must_use]
    pub fn completion_painted(
        &self,
        id: ScriptId,
    ) -> Option<Vec<ph2d_panel_code::CompletionPaintedRow>> {
        let d = self.docs.iter().find(|d| d.script == id)?;
        d.view.completion_painted()
    }

    /// ⚠️ SONDA: o texto do balão do rato que a aba do script `id` tem para mostrar.
    #[must_use]
    pub fn hover_of(&self, id: ScriptId) -> Option<String> {
        let d = self.docs.iter().find(|d| d.script == id)?;
        d.hints
            .hover
            .as_ref()?
            .shown
            .as_ref()
            .map(|h| h.text.clone())
    }

    /// ⚠️ SONDA: a assinatura que a aba do script `id` mostra: o rótulo e o parâmetro marcado.
    #[must_use]
    pub fn signature_of(&self, id: ScriptId) -> Option<(String, Option<String>)> {
        let d = self.docs.iter().find(|d| d.script == id)?;
        let s = d.hints.sig.as_ref()?.shown.as_ref()?;
        Some((
            s.label.clone(),
            s.active.clone().map(|r| s.label[r].to_owned()),
        ))
    }

    /// ⚠️ SONDA: o balão do rato e a assinatura que a aba do script `id` pintou no último quadro.
    #[must_use]
    pub fn hints_painted(
        &self,
        id: ScriptId,
    ) -> (
        Option<ph2d_panel_code::HintProbe>,
        Option<ph2d_panel_code::HintProbe>,
    ) {
        self.docs
            .iter()
            .find(|d| d.script == id)
            .map(|d| d.view.hints_painted())
            .unwrap_or_default()
    }

    /// ⚠️ SONDA: o centro do ponto da calha da linha `line` (0-based) da aba do script `id`.
    #[must_use]
    pub fn problem_dot(&self, id: ScriptId, line: usize) -> Option<(f32, f32)> {
        let d = self.docs.iter().find(|d| d.script == id)?;
        d.view.problem_dot(line)
    }

    /// ⚠️ SONDA: o centro do carácter no byte `at` da aba do script `id` (onde o rato o aponta), se
    /// está à vista.
    #[must_use]
    pub fn glyph_center(&self, id: ScriptId, at: usize) -> Option<(f32, f32)> {
        let d = self.docs.iter().find(|d| d.script == id)?;
        let t = d.buffer.text();
        let a = d.view.caret_rect(&d.buffer, at)?;
        let b = d.view.caret_rect(&d.buffer, t.next_grapheme(at))?;
        Some(((a.x + b.x) * 0.5, a.y + a.h * 0.5))
    }

    /// ⚠️ SONDA: a gramática da aba do script `id`.
    #[must_use]
    pub fn grammar_of(&self, id: ScriptId) -> Option<GrammarProbe> {
        let d = self.docs.iter().find(|d| d.script == id)?;
        Some(GrammarProbe {
            sync_reparses: d.syntax.sync_reparses(),
            stale: d.syntax.is_stale(),
        })
    }

    /// ⚠️ SONDA: o centro do marcador de dobra da linha `line` na calha (onde o clique abre/fecha a
    /// dobra), se a linha está à vista. O clique só é dobra se o último quadro pintou ali uma.
    #[must_use]
    pub fn fold_marker(&self, id: ScriptId, line: usize) -> Option<(f32, f32)> {
        let d = self.docs.iter().find(|d| d.script == id)?;
        let r = d
            .view
            .caret_rect(&d.buffer, d.buffer.text().byte_of_line(line))?;
        let x = d.view.text_rect().x - d.view.metrics().fold_w * 0.5;
        Some((x, r.y + r.h * 0.5))
    }

    /// ⚠️ SONDA: a cor com que o último quadro pintou o byte `at` da aba do script `id` (`Plain` =
    /// texto sem cor, ou não pintado).
    #[must_use]
    pub fn ink_painted(&self, id: ScriptId, at: usize) -> ph2d_panel_code::Ink {
        self.docs
            .iter()
            .find(|d| d.script == id)
            .and_then(|d| d.painted.iter().rev().find(|(r, _)| r.contains(&at)))
            .map_or(ph2d_panel_code::Ink::Plain, |(_, k)| *k)
    }

    /// ⚠️ SONDA: quantas cores do assistente a aba do script `id` segura.
    #[must_use]
    pub fn semantic_count(&self, id: ScriptId) -> usize {
        self.docs
            .iter()
            .find(|d| d.script == id)
            .map_or(0, |d| d.sem.list.len())
    }

    /// ⚠️ SONDA: o realce que a aba do script `id` pinta no byte `at` (`None` = texto sem cor).
    #[must_use]
    pub fn highlight_at(&self, id: ScriptId, at: usize) -> Option<HighlightKind> {
        let d = self.docs.iter().find(|d| d.script == id)?;
        d.syntax
            .highlights(d.buffer.text(), at..at + 1)
            .into_iter()
            .filter(|h| h.range.contains(&at))
            .map(|h| h.kind)
            .next_back()
    }

    /// ⚠️ SONDA: as ocorrências realçadas na aba do script `id` (I10).
    #[must_use]
    pub fn occurrences_of(&self, id: ScriptId) -> Vec<std::ops::Range<usize>> {
        self.docs
            .iter()
            .find(|d| d.script == id)
            .map(|d| d.occ.ranges.clone())
            .unwrap_or_default()
    }

    /// ⚠️ SONDA: a caixinha do rename (I10): o nome escrito (`None` = só a razão), a ajuda e a razão.
    #[must_use]
    pub fn rename_of(&self, id: ScriptId) -> Option<(Option<String>, String, Option<String>)> {
        let d = self.docs.iter().find(|d| d.script == id)?;
        let r = d.rename.as_ref()?;
        let (hint, why) = d.rename_note();
        Some((
            r.kind.as_ref().map(|_| r.field.text().to_string()),
            hint,
            why,
        ))
    }

    /// ⚠️ SONDA: quantas vezes o assistente arrancou (um que morre e volta conta outra vez).
    #[must_use]
    pub fn assist_boots(&self) -> u64 {
        self.intel.boots()
    }

    /// ⚠️ SONDA: a lista do Shift+F12 (I10): as linhas e a seleccionada.
    #[must_use]
    pub fn references_of(&self, id: ScriptId) -> Option<(Vec<String>, usize)> {
        let d = self.docs.iter().find(|d| d.script == id)?;
        let r = d.refs.as_ref()?;
        Some((
            d.refs_rows(r).into_iter().map(|x| x.label).collect(),
            r.selected,
        ))
    }

    /// ⚠️ SONDA: a caixa da caixinha do rename no último quadro pintado.
    #[must_use]
    pub fn rename_rect_of(&self, id: ScriptId) -> Option<ph2d_editor_core::zones::Rect> {
        self.docs
            .iter()
            .find(|d| d.script == id)?
            .view
            .rename_rect()
    }

    /// ⚠️ SONDA: as faixas da selecção do script `id` (`início..fim`, a primária primeiro).
    #[must_use]
    pub fn selection_of(&self, id: ScriptId) -> Vec<std::ops::Range<usize>> {
        let Some(d) = self.docs.iter().find(|d| d.script == id) else {
            return Vec::new();
        };
        let sel = d.buffer.selection();
        let p = sel.primary();
        std::iter::once(p)
            .chain(sel.ranges().iter().copied().filter(|r| *r != p))
            .map(|r| r.from()..r.to())
            .collect()
    }

    /// ⚠️ SONDA: o modelo com buracos (I12): `None` = sem sessão; senão os buracos NÃO activos.
    #[must_use]
    pub fn snippet_of(&self, id: ScriptId) -> Option<Vec<std::ops::Range<usize>>> {
        let d = self.docs.iter().find(|d| d.script == id)?;
        d.snippet.as_ref().map(crate::snippet::Tabstops::marks)
    }

    /// ⚠️ SONDA: as dicas de tipo que a aba segura (I9): `(byte, rótulo)`.
    #[must_use]
    pub fn inlays_of(&self, id: ScriptId) -> Vec<(usize, String)> {
        self.docs
            .iter()
            .find(|d| d.script == id)
            .map(|d| {
                d.inlay
                    .list
                    .iter()
                    .map(|h| (h.at, h.label.clone()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// ⚠️ SONDA: onde o cursor se pintaria no byte `at` do script `id` (`None` = fora da vista).
    #[must_use]
    pub fn caret_rect_at(&self, id: ScriptId, at: usize) -> Option<ph2d_editor_core::zones::Rect> {
        let d = self.docs.iter().find(|d| d.script == id)?;
        d.view.caret_rect(&d.buffer, at)
    }

    /// ⚠️ SONDA: os retângulos que o último quadro pinta para os bytes `s..e` do script `id` (a
    /// selecção e as marcas pintam-se por eles).
    #[must_use]
    pub fn segments_of(
        &self,
        id: ScriptId,
        s: usize,
        e: usize,
    ) -> Vec<ph2d_editor_core::zones::Rect> {
        self.docs
            .iter()
            .find(|d| d.script == id)
            .map(|d| d.view.probe_segments(&d.buffer, s, e))
            .unwrap_or_default()
    }

    /// ⚠️ SONDA: o elo do grupo que o próximo desfazer (`redo = false`) ou refazer da aba tiraria.
    #[must_use]
    pub fn top_link_of(&self, id: ScriptId, redo: bool) -> Option<u64> {
        let d = self.docs.iter().find(|d| d.script == id)?;
        d.buffer.history().top_link(redo)
    }

    /// ⚠️ SONDA: espera as respostas do formatador em voo (o quadro seguinte aplica-as).
    pub fn wait_format(&mut self) {
        self.format.wait(std::time::Duration::from_secs(30));
    }

    /// ⚠️ SONDA: quantos pedidos ao formatador ainda não responderam.
    #[must_use]
    pub fn format_in_flight(&self) -> usize {
        self.format.in_flight()
    }
}

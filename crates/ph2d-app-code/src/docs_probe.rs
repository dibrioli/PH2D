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
}

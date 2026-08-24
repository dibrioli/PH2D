//! **A DOBRA de uma seção** — o lado do `WidgetStore` que responde *esta gaveta está
//! aberta?*, *quanto dela está aberto agora?* e *que altura o corpo dela tinha?*.
//!
//! Separado do [`super::chrome_ops`] pelo teto de LOC (HR-18, 700 para `crates/`), e o
//! corte é por RESPONSABILIDADE: lá ficam os pares chave/valor soltos do chrome (área de
//! transferência, nome de cena, tooltips, flyouts), aqui fica a única família daquele
//! arquivo que tem uma LEI própria — o binário, o `t` que o anima e a altura que o `t`
//! multiplica são três leituras da MESMA gaveta, e ler uma sem as outras é o defeito que
//! o doc do `section_open_live` descreve.

use super::WidgetStore;
use ph2d_a11y::NodeId;

impl WidgetStore {
    /// `true` iff the section/panel at `id` is currently collapsed.
    /// Missing entries default to expanded — newly-registered
    /// sections start open without any setup.
    pub fn is_collapsed(&self, id: NodeId) -> bool {
        self.collapsed.get(&id).copied().unwrap_or(false)
    }

    /// **O que o ARTISTA escolheu para esta seção** — `None` quando ele ainda não
    /// escolheu nada.
    ///
    /// ⚠️ **Existe para separar «não escolhido» de «escolhido aberto»**, que o
    /// [`Self::is_collapsed`] colapsa no mesmo `false`. Quem quer semear um estado
    /// inicial (uma seção que nasce fechada) precisa da diferença: semear por cima de
    /// uma escolha do artista reabriria a gaveta que ele fechou, a cada quadro.
    ///
    /// ⚠️ **O default NÃO mora aqui.** O store lembra escolhas; quem declara como uma
    /// seção começa é quem a desenha — senão o desenho de um nó ficaria em dois sítios.
    #[must_use]
    pub fn collapsed_choice(&self, id: NodeId) -> Option<bool> {
        self.collapsed.get(&id).copied()
    }

    /// Set the collapsed state for a section/panel. `true` collapses,
    /// `false` expands.
    pub fn set_collapsed(&mut self, id: NodeId, collapsed: bool) {
        self.collapsed.insert(id, collapsed);
    }

    /// Flip the collapsed state for `id`. Convenience for click
    /// handlers — equivalent to
    /// `set_collapsed(id, !is_collapsed(id))`.
    pub fn toggle_collapsed(&mut self, id: NodeId) {
        let was = self.is_collapsed(id);
        // ⚠️ **A PARTIDA é gravada ANTES de o estado virar, e é isso que faz a PRIMEIRA dobra de
        //    cada secção animar.** A lei do substrato é que a *primeira vista de um id CHEGA ao
        //    alvo*; sem esta linha o relógio vê a secção pela primeira vez já no destino e a
        //    estreia de cada dobra SALTA — todas as seguintes animariam, e o defeito seria uma
        //    vez por secção por sessão, que é a forma mais fácil de ninguém reproduzir.
        //
        // ⚠️ `or_insert` e não `insert`: com uma dobra EM VOO o valor publicado é a verdade, e
        //    re-clicar a meio tem de RETOMAR de onde ela está — é a interruptibilidade que o
        //    substrato promete, e escrever por cima dela devolveria a secção ao extremo.
        self.fold_live
            .entry(id)
            .or_insert(if was { 0.0 } else { 1.0 });
        self.collapsed.insert(id, !was);
    }

    /// **Quanto desta secção está ABERTO** (`0.0` fechada … `1.0` aberta) — o número que o
    /// chevron veste.
    ///
    /// ⚠️ **O neutro é o BINÁRIO de hoje**, não zero: uma secção que o relógio nunca viu devolve
    /// exatamente `1.0`/`0.0` conforme [`Self::is_collapsed`], então um store não-tiquado — e um
    /// pintor ainda não migrado — desenha **byte a byte** o que desenhava antes desta wave. É a
    /// mesma neutralidade do `hover_live`, e é ela que torna a migração dos ~34 sítios segura de
    /// fazer aos poucos: um sítio esquecido fica com a troca binária, nunca meio-animado.
    #[must_use]
    pub fn section_open_live(&self, id: NodeId) -> f32 {
        self.fold_live
            .get(&id)
            .copied()
            .unwrap_or(if self.is_collapsed(id) { 0.0 } else { 1.0 })
    }

    /// **Toda secção que o despacho sabe dobrar** — o conjunto que o `populate` semeou.
    ///
    /// ⚠️ Existe para o TIQUE e para o harness: os dois precisam de percorrer *quem tem dobra*,
    /// e a alternativa (iterar o `fold_live`) veria só as que já se mexeram — a secção que o
    /// artista acabou de fechar e que nunca animou ficaria de fora, que é exactamente a que
    /// interessa.
    pub fn collapsible_ids(&self) -> Vec<NodeId> {
        self.collapsible_sections.iter().copied().collect()
    }

    /// **O tique publica aqui.** Único escritor — o gêmeo exacto do
    /// [`super::WidgetStore::set_hover_live`] e do `set_panel_scroll_live`.
    pub fn set_section_open_live(&mut self, id: NodeId, t: f32) {
        self.fold_live.insert(id, t);
    }

    /// **Quão alto o CORPO desta secção mediu da última vez que foi pintado**, ou `None` se ela
    /// nunca foi pintada aberta. Alimenta o RECORTE da dobra e mais nada — ver
    /// [`super::WidgetStore::fold_body_h`] para o porquê de ser um valor lembrado e não medido
    /// no próprio quadro.
    #[must_use]
    pub fn section_body_h(&self, id: NodeId) -> Option<f32> {
        self.fold_body_h.borrow().get(&id).copied()
    }

    /// **O pintor publica o que acabou de medir.** ⚠️ `&self` de propósito — ver
    /// [`super::WidgetStore::fold_body_h`].
    pub fn remember_section_body_h(&self, id: NodeId, h: f32) {
        self.fold_body_h.borrow_mut().insert(id, h.max(0.0));
    }

    /// As secções que o tique tem de animar: **as que o utilizador tocou**.
    ///
    /// ⚠️ Uma secção nunca tocada não está no mapa e **não custa nada** — ela está aberta, o
    /// neutro devolve `1.0`, e não há nada a integrar. Uma que nasce fechada no `populate`
    /// (`set_collapsed(id, true)` no arranque) entra no mapa **sem partida gravada**, então o
    /// alvo e o neutro coincidem e ela assenta sem animar: *nascer fechada não é dobrar-se*.
    pub fn collapse_states(&self) -> impl Iterator<Item = (NodeId, bool)> + '_ {
        self.collapsed.iter().map(|(k, v)| (*k, *v))
    }
}

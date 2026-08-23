//! **O PIE MENU no store** — irmão do [`super::chrome_ops`] pelo teto de 700 LOC, e o corte é por
//! ASSUNTO: ali ficam os diálogos, os modais e a paleta (chrome que o rato abre e fecha); aqui, o
//! único chrome deste app que **um gesto SEGURA** — a tecla fica em baixo, e soltá-la escolhe.
//!
//! ⚠️ **Abrir, acender e escolher são três verbos de um gesto só**, e é por isso que eles moram
//! juntos: separá-los pelos ficheiros do chrome faria a próxima pessoa procurar o cancelar num
//! sítio e o escolher noutro.

use super::WidgetStore;

impl WidgetStore {
    /// **ABRE O PIE MENU** no ponto `center`, com `items` — a vista de OITO direcções da mesma
    /// lista que a paleta oferece.
    ///
    /// ⚠️ **Sem itens ele não abre**, e a recusa é a mesma lei do modo de preview: um menu que não
    /// oferece nada é indistinguível de um atalho partido, e o artista não teria como saber que o
    /// que falta é o modo em que ele está.
    pub fn open_radial(&mut self, center: [f32; 2], items: Vec<crate::widget::RadialItem>) -> bool {
        if items.is_empty() {
            return false;
        }
        self.radial = Some(crate::widget::RadialOpen {
            center,
            items,
            hot: None,
        });
        true
    }

    /// Fecha o pie menu (Escape, ou a soltura da tecla). Devolve o sector escolhido, se houve um.
    ///
    /// ⚠️ **Fechar e ESCOLHER são a mesma operação**, de propósito: soltar a tecla é o gesto que
    /// faz as duas, e separá-las daria ao chamador a chance de fechar sem ler — que é como um menu
    /// perde a escolha do artista em silêncio.
    pub fn close_radial(&mut self) -> Option<crate::widget::RadialItem> {
        let open = self.radial.take()?;
        let i = open.hot?;
        open.items.get(i).cloned()
    }

    /// O ponteiro mexeu-se: recalcula que sector está aceso. No-op sem menu aberto.
    pub fn radial_point(&mut self, pointer: [f32; 2]) {
        if let Some(r) = self.radial.as_mut() {
            r.hot = crate::widget::radial_sector_at(r.center, pointer, r.items.len());
        }
    }

    /// O pie menu aberto, para o pintor.
    #[must_use]
    pub fn radial(&self) -> Option<&crate::widget::RadialOpen> {
        self.radial.as_ref()
    }
}

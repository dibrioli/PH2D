//! **OS PLANOS OPCIONAIS por-vértice** — cor, máscara e o AO assado: o que eles
//! são, como se materializam, o que o undo empresta deles, e a validade do
//! único que envelhece.
//!
//! Filho do [`crate::mesh`], não irmão, pelo mesmo motivo do [`super::splice`]:
//! ele escreve os campos PRIVADOS da malha, e privacidade em Rust alcança os
//! descendentes do módulo que declara.
//!
//! ⚠️ **O corte é por RESPONSABILIDADE.** O pai diz *o que uma malha é* e
//! carrega os planos que TODO vértice tem (posição, normal, curvatura). Aqui
//! ficam os que podem **não existir** — e é por isso que os quatro `take`/`put`
//! moram juntos: eles são um padrão só (*emprestar um plano a quem precisa ler
//! a adjacência ao mesmo tempo, e devolvê-lo, inclusive devolvendo a AUSÊNCIA*),
//! e separá-los faria a próxima pessoa perguntar por que metade está noutro
//! arquivo.
//!
//! ⚠️ **E um deles não é como os outros dois.** Cor e máscara são AUTORADAS:
//! continuam verdadeiras seja qual for a forma. O AO é **MEDIDO DA FORMA**, então
//! mexer na forma não o apaga nem o mantém — deixa-o **ERRADO**, e errado de um
//! jeito que ninguém reporta. É o único plano com data de validade, e a regra
//! que a denuncia mora aqui.

use super::{DEFAULT_COLOR, DEFAULT_MASK, Mesh};

impl Mesh {
    /// A cor por vértice, se a malha já foi pintada. `None` = `DEFAULT_COLOR`
    /// em todo vértice.
    #[must_use]
    pub fn colors(&self) -> Option<&[[f32; 3]]> {
        self.colors.as_deref()
    }

    /// A máscara por vértice, se alguém mascarou. `None` = `DEFAULT_MASK`.
    #[must_use]
    pub fn masks(&self) -> Option<&[f32]> {
        self.masks.as_deref()
    }

    /// Materializa o plano de cor (aloca no primeiro uso — ver o doc do módulo).
    pub fn colors_mut(&mut self) -> &mut [[f32; 3]] {
        self.colors
            .get_or_insert_with(|| vec![DEFAULT_COLOR; self.positions.len()])
    }

    /// Materializa o plano de máscara.
    pub fn masks_mut(&mut self) -> &mut [f32] {
        self.masks
            .get_or_insert_with(|| vec![DEFAULT_MASK; self.positions.len()])
    }

    /// **Tira o plano de máscara da malha**, se houver — o gesto de *limpar*, e
    /// a metade que empresta o plano a quem precisa ler a adjacência ao mesmo
    /// tempo.
    ///
    /// ⚠️ **A segunda razão é o borrow, e ela decide o desenho das operações de
    /// máscara:** borrar uma máscara é `m ← média do anel`, o que exige a
    /// adjacência (imutável) e o plano (mutável) no MESMO escopo. Clonar a
    /// adjacência custaria milhões de `u32` por passo; tirar o plano custa mover
    /// um `Vec`. Quem tira devolve com [`Self::put_masks`].
    ///
    /// Devolve `None` se ninguém mascarou — e é isso que faz *limpar uma malha
    /// limpa* não alocar nada.
    pub fn take_masks(&mut self) -> Option<Vec<f32>> {
        self.masks.take()
    }

    /// **Tira o plano de COR da malha**, se houver — o espelho de
    /// [`Self::take_masks`], e ele existe porque *restaurar* um estado tem de
    /// poder devolver a malha ao que ela era, inclusive a **não ter** o plano.
    /// Sem ele, desfazer uma operação que criou a cor deixaria a malha pagando
    /// 12 B/vértice por um canal que ninguém pediu.
    pub fn take_colors(&mut self) -> Option<Vec<[f32; 3]>> {
        self.colors.take()
    }
    /// Devolve o plano tirado por [`Self::take_masks`].
    ///
    /// ⚠️ **Recusa em silêncio um plano do tamanho errado** — não: ele PANICA,
    /// porque um plano curto seria lido como *"os últimos vértices estão
    /// livres"*, que é uma máscara diferente da que o artista pintou, e nada na
    /// tela diria por quê.
    pub fn put_masks(&mut self, masks: Vec<f32>) {
        assert_eq!(
            masks.len(),
            self.positions.len(),
            "o plano de máscara tem de medir a malha"
        );
        self.masks = Some(masks);
    }

    /// O AO assado por vértice, se alguém já assou. `None` = `DEFAULT_AO`
    /// (céu aberto — a leitura honesta de *"ninguém mediu"*, e a que não
    /// escurece a peça por omissão).
    #[must_use]
    pub fn ao(&self) -> Option<&[f32]> {
        self.ao.as_deref()
    }

    /// **O AO descreve uma forma que não existe mais.**
    ///
    /// ⚠️ Só é verdade quando há AO: sem bake não há o que envelhecer, e
    /// devolver `true` numa malha nunca assada faria a UI anunciar um problema
    /// que ninguém tem.
    #[must_use]
    pub fn ao_is_stale(&self) -> bool {
        self.ao.is_some() && self.baked_stale
    }

    /// Instala o resultado de um bake — e é o **único** jeito de o AO deixar de
    /// estar velho.
    ///
    /// # Panics
    /// Se `ao` não tiver um valor por vértice. É a mesma validação do
    /// [`Self::put_masks`], e pelo mesmo motivo: um plano por-vértice de
    /// comprimento errado não falha, ele lê o vizinho.
    pub fn set_ao(&mut self, ao: Vec<f32>) {
        assert_eq!(
            ao.len(),
            self.positions.len(),
            "o AO tem de ter um valor por vertice"
        );
        self.ao = Some(ao);
        self.baked_stale = false;
    }

    /// Joga o bake fora — o gesto de *"não quero mais este canal"*, e o que
    /// devolve os 4 B/vértice.
    pub fn clear_ao(&mut self) {
        self.ao = None;
        self.baked_stale = false;
    }

    /// **Tira o plano de AO da malha**, se houver — o espelho do
    /// [`Self::take_masks`], e ele existe pelo mesmo motivo: restaurar um estado
    /// tem de poder devolver a malha a **não ter** o canal.
    ///
    /// ⚠️ Devolve o par com a validade junto: separar os dois deixaria quem
    /// restaura reinstalar um AO velho dizendo que é fresco.
    pub fn take_ao(&mut self) -> Option<(Vec<f32>, bool)> {
        let stale = self.baked_stale;
        self.baked_stale = false;
        self.ao.take().map(|a| (a, stale))
    }

    /// Devolve o que o [`Self::take_ao`] tirou, **com a validade que ele tinha**.
    ///
    /// # Panics
    /// Se o comprimento não bater com a contagem de vértices.
    pub fn put_ao(&mut self, ao: Vec<f32>, stale: bool) {
        assert_eq!(
            ao.len(),
            self.positions.len(),
            "o AO tem de ter um valor por vertice"
        );
        self.ao = Some(ao);
        self.baked_stale = stale;
    }

    /// A **espessura** assada por vértice, se alguém já assou. `None` = nunca
    /// medida — e o consumidor lê isso como *opaca*, que é a leitura que não
    /// acende nada por omissão.
    #[must_use]
    pub fn thickness(&self) -> Option<&[f32]> {
        self.thickness.as_deref()
    }

    /// **A espessura descreve uma forma que não existe mais.**
    ///
    /// ⚠️ Lê a MESMA validade do [`Self::ao_is_stale`] — ver o campo em
    /// [`crate::Mesh`]: os dois planos são medidos pelo mesmo gesto contra a
    /// mesma malha, então envelhecem no mesmo instante. O que difere entre as
    /// duas portas é só **de qual plano se está perguntando**, e é por isso que
    /// cada uma é guardada pela própria presença.
    #[must_use]
    pub fn thickness_is_stale(&self) -> bool {
        self.thickness.is_some() && self.baked_stale
    }

    /// Instala o resultado de um bake de espessura.
    ///
    /// ⚠️ **Ele NÃO zera a validade sozinho** — quem a zera é o [`Self::set_ao`],
    /// e a razão é a mesma que junta os dois campos: assar um plano e não o
    /// outro deixaria metade do que a tela lê descrevendo a forma de ontem, com
    /// a UI anunciando que está tudo fresco. Os dois entram pelo mesmo gesto.
    ///
    /// # Panics
    /// Se `thickness` não tiver um valor por vértice — a validação do
    /// [`Self::put_masks`], pelo mesmo motivo.
    pub fn set_thickness(&mut self, thickness: Vec<f32>) {
        assert_eq!(
            thickness.len(),
            self.positions.len(),
            "a espessura tem de ter um valor por vertice"
        );
        self.thickness = Some(thickness);
    }

    /// Joga o bake de espessura fora — o gesto de *"não quero mais este canal"*.
    pub fn clear_thickness(&mut self) {
        self.thickness = None;
    }

    /// **Tira o plano de espessura da malha**, se houver — o espelho do
    /// [`Self::take_ao`], e ele existe pelo mesmo motivo: restaurar um estado
    /// tem de poder devolver a malha a **não ter** o canal.
    pub fn take_thickness(&mut self) -> Option<Vec<f32>> {
        self.thickness.take()
    }

    /// Devolve o que o [`Self::take_thickness`] tirou.
    ///
    /// # Panics
    /// Se o comprimento não bater com a contagem de vértices.
    pub fn put_thickness(&mut self, thickness: Vec<f32>) {
        assert_eq!(
            thickness.len(),
            self.positions.len(),
            "a espessura tem de ter um valor por vertice"
        );
        self.thickness = Some(thickness);
    }
}

#[cfg(test)]
#[path = "mesh_planes_tests.rs"]
mod tests;

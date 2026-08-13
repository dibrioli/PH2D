//! **AS JANELAS QUE UM TRAÇO PUBLICA** — filho (`#[path]`) de [`super`], e o
//! corte é de ASSUNTO: lá mora *o que um traço FAZ* (a lei, o dab, a captura),
//! aqui *o que se PERGUNTA a ele*.
//!
//! ⚠️ **Filho e não irmão, pela mesma conta do [`super::apply`]:** as sete
//! respostas leem os planos privados (`touched`, `base_pos`, `call_moved`,
//! `region`, …), e um módulo irmão os obrigaria a virar `pub(crate)` — a
//! visibilidade viraria função do TAMANHO do arquivo, que é o oposto do que o
//! teto de LOC existe para fazer.
//!
//! ⚠️ **As três janelas NÃO são a mesma, e a diferença de cada par já custou um
//! defeito visível** (os docs de cada uma trazem qual): *o que o gesto tocou*
//! (o undo) · *o que o último dab moveu* · *o que a GPU precisa RE-LER*.

use super::*;

impl SculptStroke {
    /// Os vértices que este traço tocou — **a janela do undo**.
    #[must_use]
    pub fn touched(&self) -> &[u32] {
        &self.touched
    }

    /// As posições de antes do traço, na ordem de [`Self::touched`] — **o
    /// estado anterior do undo**. Não há um segundo sistema a construir: o
    /// congelamento que a lei exige já É a entrada de undo.
    #[must_use]
    pub fn base_positions(&self) -> &[[f32; 3]] {
        &self.base_pos
    }

    /// As máscaras de antes do traço, na ordem de [`Self::touched`].
    #[must_use]
    pub fn base_masks(&self) -> &[f32] {
        &self.base_mask
    }

    /// Os vértices que o ÚLTIMO [`Self::dab`] de fato moveu — **todas as cópias
    /// de espelho dele**, e não a última.
    ///
    /// ⚠️ *"O último dab"* é a CHAMADA, e a diferença já custou um smoke: com o
    /// espelho armado a chamada aplica de duas a oito cópias, e publicar a
    /// última é publicar o reflexo do que o artista fez. O tamanho desta lista é
    /// exatamente o número que [`Self::dab`] devolve — se os dois divergirem,
    /// dois chamadores do mesmo dab discordam sobre ele.
    #[must_use]
    pub fn last_moved(&self) -> &[u32] {
        &self.call_moved
    }

    /// Os vértices que o último dab deixou **obsoletos na GPU** — a janela do
    /// upload incremental.
    ///
    /// ⚠️ **É um SUPERCONJUNTO de [`Self::last_moved`], e confundir os dois é um
    /// defeito visível.** Mover um vértice muda a normal de todo vizinho que
    /// compartilha uma face com ele, mesmo que o vizinho não tenha andado —
    /// `refresh_region` já os conserta na CPU, e subir só os movidos deixa a
    /// malha iluminada por normais velhas numa faixa de um anel de largura, bem
    /// na BORDA do pincel. Um gate de GPU pegou isto comparando o quadro
    /// incremental com o quadro do upload cheio.
    #[must_use]
    pub fn last_refreshed(&self) -> &[u32] {
        &self.call_refreshed
    }

    /// Os vértices que a GPU precisa **RE-LER** depois do último dab, em
    /// QUALQUER canal — a janela do upload incremental.
    ///
    /// ⚠️ **Não é o mesmo que [`Self::last_refreshed`], e a diferença é uma
    /// feature inteira.** Aquele responde *de quem eu recomputei a NORMAL*, e um
    /// traço de máscara não move geometria: ele escreve o canal de máscara e
    /// **esquece a região de propósito**. Um chamador que subisse `refreshed`
    /// não subiria byte nenhum de um traço de Mask — a máscara ficaria invisível
    /// na GPU, agora por um segundo motivo, com todos os gates de CPU verdes.
    ///
    /// Os dois casos são exclusivos por construção (o dab ou pinta máscara, ou
    /// move geometria), então a resposta é uma escolha e nunca uma união.
    #[must_use]
    pub fn last_gpu_dirty(&self) -> &[u32] {
        if self.last_paints_mask {
            &self.call_moved
        } else {
            &self.call_refreshed
        }
    }

    /// Bytes segurados. A sonda de memória o soma: o custo do GESTO não pode
    /// ficar fora da conta só por ser transitório.
    #[must_use]
    pub fn capacity_bytes(&self) -> usize {
        let v3 = size_of::<[f32; 3]>();
        (self.slot.capacity() + self.stamp.capacity()) * size_of::<u32>()
            + (self.touched.capacity()
                + self.footprint.capacity()
                + self.moved.capacity()
                + self.call_moved.capacity()
                + self.call_refreshed.capacity())
                * size_of::<u32>()
            + (self.base_pos.capacity() + self.base_nrm.capacity() + self.target.capacity()) * v3
            + (self.base_mask.capacity() + self.accum.capacity()) * size_of::<f32>()
            + self.query.capacity_bytes()
            + self.region.capacity_bytes()
    }
}

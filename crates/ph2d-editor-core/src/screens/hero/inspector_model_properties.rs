//! ⭐⭐⭐ **O CARTÃO DE PROPRIEDADES do Inspector** — *«o que este objecto DIZ que é»*.
//!
//! # ⛔⛔ O buraco que ele fecha (report do Enio, 2026-08-31)
//!
//! *«quando mudo o conteúdo entre `{}` o inspector não muda»*. As chaves do nome
//! (`Casa {Size=Small, State=Idle}`) tinham **dois** leitores em todo o app — o selo `*²` da
//! Hierarquia e a fileira de troca do cartão de instância — e o segundo só existe sobre uma família
//! de **duas ou mais receitas**. Num objecto solto, ou numa cópia de um mestre único, reescrever as
//! chaves não mudava um pixel do Inspector.
//!
//! ⚠️ *Uma declaração sem leitor é decoração* — e esta tinha um selo na Hierarquia a prometer que
//! alguém a lia. O cartão é o leitor.
//!
//! # ⚠️ Porque é um cartão SEPARADO do de instância
//!
//! O cartão de instância responde *«de que receita sou cópia, e o que tenho de diferente dela»*, e
//! **não existe** sobre um objecto que não é cópia de nada. As propriedades são um facto do próprio
//! objecto: elas existem sobre qualquer nome que as declare. ⇒ dois sujeitos, dois cartões.
//!
//! ⛔ **E as fileiras MUDARAM de dono, não nasceram aqui:** elas viviam dentro do cartão de
//! instância. Deixá-las lá e acrescentar uma cópia aqui poria os **mesmos ids** registados por dois
//! pintores no mesmo quadro — o segundo `register` ganha, e o artista clicaria num chip para ver
//! outro acender.

/// Snapshot das propriedades da entidade selecionada.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct InspectorPropertiesInfo {
    pub entity_bits: u64,
    /// A RAIZ da instância — quem recebe a troca de variante. `0` quando isto não é cópia de nada,
    /// e aí nenhuma fileira tem mais de um valor.
    pub root_bits: u64,
    /// Uma fileira por pergunta ou por propriedade declarada. Ver
    /// [`super::variant_axes::rows_for`].
    pub rows: Vec<super::variant_axes::VariantAxis>,
    /// O que a tabela de ids não endereça — **escrito**, nunca truncado em silêncio.
    pub beyond: usize,
}

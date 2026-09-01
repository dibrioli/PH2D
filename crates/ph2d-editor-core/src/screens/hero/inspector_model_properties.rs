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
    /// ⭐⭐⭐ **Quantas modificações a cópia escolhida tem por gravar** — `0` esconde o botão
    /// *Salvar Variação…*.
    ///
    /// ⚠️ **É a pergunta que decide se o botão EXISTE**, e não uma decoração: sem modificação não
    /// há versão a criar, e um botão que não faz nada é a espécie que a caça aos knobs mortos
    /// nomeia. *A decisão de MOSTRAR e a de GRAVAR leem a mesma pergunta.*
    pub pending: usize,
    /// ⭐⭐ **As propriedades que a família JÁ declara** — o selector do formulário.
    ///
    /// ⛔ **Não é `rows.iter().map(name)`**: uma propriedade em que todas as versões concordam não
    /// é uma FILEIRA (um chip único não escolhe nada) mas continua a ser uma propriedade — e é
    /// justamente a ela que o artista quer acrescentar o segundo valor.
    pub declared: Vec<String>,
    /// ⭐⭐⭐ **O NOME DO OBJECTO SELECIONADO**, como a Hierarquia o mostra — o título do cartão.
    ///
    /// # ⚠️ Este campo já significou o CONTRÁRIO, e a decisão é do dono
    ///
    /// A 1.ª versão punha aqui o nome do **componente** — a fonte das propriedades — para explicar
    /// por que o cartão diz `Small` sobre uma cópia que o artista renomeou para `Big`. Enio
    /// (2026-08-31): *«Properties of "Nome do objeto na Hierarquia"»*.
    ///
    /// ⛔ **A pergunta que aquela versão respondia continua sem resposta na tela**, e ficou
    /// nomeada: uma cópia cujo nome declara `{Size=Big}` mostra as propriedades do componente
    /// (`Small`), porque uma propriedade é do COMPONENTE. ⚠️ Na prática ela quase não se atinge
    /// desde que uma variante nasce com valor próprio (`variant_axes::variant_name`) — o artista
    /// deixa de precisar de escrever chaves numa cópia para criar uma versão.
    ///
    /// ⚠️ **É o nome CURTO** (`display_name`): um título com as chaves lá dentro é a frase
    /// comprida que o report de 2026-08-30 recusou.
    pub source_name: Option<String>,
}

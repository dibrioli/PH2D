//! ⭐⭐ **PARA ONDE O ESFREGÃO EMPURRA O DESLOCAMENTO** — a direcção `d̂` do
//! [`crate::Verb::SmearMultires`] (`SPEC_unblocked_brushes.md` §5.3).
//!
//! ⚠️⚠️ **Ele NÃO move o vértice para onde a mão vai.** O que este selector
//! escolhe é a direcção contra a qual o **peso de cada vizinho** é medido — e
//! como esse peso é a parte **negativa** do cosseno, quem contribui é sempre
//! quem está **a montante** de `d̂`. É isso que faz o deslocamento *viajar*
//! sobre a superfície em vez de borrar por igual, e é por isso que o mesmo
//! motor dá um arrasto, um adensamento e um espalhamento com três direcções
//! diferentes e **uma** lei.
//!
//! ⚠️ **A ORDEM dos chips é a da tabela da espec §5.3** e não uma escolha
//! nossa. ⛔ Quem a mudar tem de olhar se algum ficheiro guarda o ÍNDICE em vez
//! do nome (a fileira do painel guarda a POSIÇÃO).

/// **A direcção do esfregão** — espec §5.3.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SmearMode {
    /// **O movimento do cursor entre dabs.** O detalhe acompanha a mão.
    ///
    /// ⚠️⚠️ **Com o cursor PARADO ele é DEGENERADO, e isso é a lei e não um
    /// defeito:** `d̂` vale zero, nenhum vizinho passa o teste do cosseno, a
    /// média colapsa em `D[v]` e o alvo é a posição viva ⇒ **o pincel fica
    /// inerte**. A espec mede-o: `1,3e-03` de deslocamento máximo contra
    /// `2,4e-02` no traço que anda. *Os outros dois NÃO têm esta
    /// degenerescência* — a direcção deles nasce da geometria.
    #[default]
    Drag,
    /// **Do vértice PARA o centro do dab.** O detalhe adensa-se no meio; ele
    /// endurece feições **sem** apertar a malha.
    Pinch,
    /// **Do centro PARA o vértice.** O detalhe espalha-se para fora; alisa.
    Expand,
}

impl SmearMode {
    /// Os três, na ordem da tabela da espec §5.3.
    pub const ALL: [Self; 3] = [Self::Drag, Self::Pinch, Self::Expand];

    /// O rótulo que aparece no chip (a UI da casa é inglesa — HR/memória).
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Drag => "Drag",
            Self::Pinch => "Pinch",
            Self::Expand => "Expand",
        }
    }

    /// **A DIRECÇÃO `d̂` DESTE VÉRTICE, NÃO normalizada** — a porta ÚNICA da
    /// pergunta que a espec §5.3 responde com uma tabela de três linhas.
    ///
    /// ⚠️ **Ela recebe o caminho do gesto E a geometria porque as três linhas
    /// da tabela leem coisas diferentes**, e é isso que a torna a resposta
    /// inteira: o arrasto lê o gesto e ignora onde o vértice está; os outros
    /// dois leem a geometria e ignoram o gesto. ⛔ *Uma porta que só soubesse a
    /// metade geométrica devolveria um sentinela para o arrasto, e o chamador
    /// teria de o substituir — que é a segunda resposta à mesma pergunta.*
    ///
    /// ⚠️ **Ela devolve o vector CRU e quem normaliza é o chamador**, por uma
    /// razão de degenerescência: o [`Self::Drag`] parado devolve `[0,0,0]` e um
    /// vértice **no centro** do dab devolve o mesmo nos outros dois.
    /// Normalizar aqui obrigaria a inventar uma direcção para o caso nulo — e a
    /// resposta certa é *não há direcção*, que a lei lê como *nenhum vizinho
    /// contribui* (espec §5.3: com o cursor parado o pincel fica **inerte**).
    ///
    /// ⚠️ **O ponto do vértice é o da SUPERFÍCIE DE REFERÊNCIA**, como todo o
    /// resto desta lei: a vizinhança é medida lá, e medir `d̂` nas posições
    /// deslocadas faria a direcção mudar com o que o próprio traço já esfregou.
    #[must_use]
    pub fn direction(self, path: [f32; 3], centre: [f32; 3], ponto: [f32; 3]) -> [f32; 3] {
        let para_fora = [
            ponto[0] - centre[0],
            ponto[1] - centre[1],
            ponto[2] - centre[2],
        ];
        match self {
            Self::Drag => path,
            Self::Pinch => [-para_fora[0], -para_fora[1], -para_fora[2]],
            Self::Expand => para_fora,
        }
    }
}

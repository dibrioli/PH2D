//! **A LIGAÇÃO sinal → papel** — o que faz um nome gritado noutro lugar mover esta forma.
//!
//! # Por que ela mora aqui, e não numa tabela própria
//!
//! O `ph2d-runtime` (R0) deu ao app uma saída de sinais com três produtores — a timeline quando o
//! play cruza um marker, a física quando duas coisas se tocam, e um **controle autorado** quando o
//! artista aperta um botão que ele próprio desenhou. Os três publicam um NOME, e até aqui o único
//! consumidor era um toast: *o nome aparecia e nada acontecia*.
//!
//! O que faltava era a ligação, e ela é **conteúdo autorado** — não motor. A tentação é uma tabela
//! global `nome → ações`, e ela está errada por duas razões que se medem:
//!
//! 1. **O ciclo de vida.** Uma tabela própria precisaria da sua própria varredura de hospedeiros
//!    mortos, e [`crate::StateSets::retain_hosts`] já corre por frame. Ligada ao hospedeiro, uma
//!    forma apagada leva as ligações dela **sem uma linha a mais** — a mesma porta, o mesmo frame,
//!    o mesmo passo de undo.
//! 2. **A superfície de autoria.** O artista seleciona a forma que RESPONDE e diz a que nome ela
//!    responde; ele não abre uma tabela e procura um id. As ligações moram ao lado da duração e da
//!    curva porque são a mesma pergunta: *como este hospedeiro se comporta*.
//!
//! ⚠️ **E a tabela continua a ser por NOME, que é o contrato do ADR-0143.** Nada aqui olha para
//! quem gritou: [`crate::StateSets::targets`] casa numa string, então uma colisão de física e um
//! botão com o mesmo nome movem a mesma cena — que é o desacoplamento inteiro do ADR-0075.
//!
//! # O que uma ação PODE ser, e a resposta é derivada
//!
//! Uma só: **ir para um papel**. Não é modéstia — é o que o modo de preview consegue DESFAZER.
//! Ele captura, ao ligar, a pose de todo id mencionado em qualquer estado autorado, e devolve-a ao
//! sair; uma ação que mudasse qualquer outra coisa **não seria restaurada**, e o documento do
//! artista mudaria por ele ter olhado. É a mesma lei que fez a preview restaurar o MUNDO em vez de
//! ir para o `Default`.
//!
//! ⇒ enquanto a restauração for a pose, a ação é a pose. Uma ação nova traz consigo a metade que a
//! desfaz, ou não entra.

use crate::role::StateRole;
use serde::{Deserialize, Serialize};

/// **Quando `name` for gritado, este hospedeiro vai para `role`.**
///
/// ⚠️ O nome é uma `String` LIVRE, e é essa a diferença entre isto e o gatilho do rato. O papel do
/// rato é DERIVADO (entrou = hover, apertou = pressed) e por isso não precisa de tabela — a
/// [`crate::StateRole`] escreve essa cerca. Um sinal não tem de onde ser derivado: ele vem de um
/// marker que alguém nomeou, de um contato que alguém nomeou, ou de um botão cujo nome é o `Name`
/// da entidade. **Ligar exige uma tabela porque o nome é autorado nos dois lados.**
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalBinding {
    /// O nome do sinal. **Vazio nunca casa** — ver [`SignalBinding::matches`].
    pub name: String,
    /// Para onde ir quando ele chegar.
    pub role: StateRole,
}

impl SignalBinding {
    /// Uma ligação nova, ainda sem nome — o que o botão *Add* cria.
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// **Esta ligação responde a `signal`?**
    ///
    /// ⚠️ **Uma ligação SEM NOME não casa com nada, e a guarda é load-bearing.** O artista aperta
    /// *Add* e a linha nasce vazia; sem esta cláusula ela ficaria à espera de um sinal cujo nome é
    /// a string vazia — e o dia em que um produtor publicasse um nome vazio (um marker que alguém
    /// criou e não nomeou) *toda ligação recém-criada do documento dispararia de uma vez*. O
    /// modo de falha não é um erro: é a cena inteira a saltar de pose sem ninguém ter pedido.
    #[must_use]
    pub fn matches(&self, signal: &str) -> bool {
        !self.name.is_empty() && self.name == signal
    }
}

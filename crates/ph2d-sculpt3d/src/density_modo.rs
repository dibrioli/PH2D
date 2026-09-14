//! **O QUE O PINCEL DE DENSIDADE FAZ COM A MALHA** — as duas direcções.
//!
//! ⚠️⚠️ **Ele nasceu de um report do dono — *«por que não pode aumentar a
//! densidade também?»* (2026-09-14) — e a resposta é que ELE PODE, e a primeira
//! redacção deste módulo é que tinha lido mal a espec.**
//!
//! # ⛔ A leitura errada, e onde ela estava escrita
//!
//! O [`crate::Verb::refina_no_dyntopo`] afirmava, com o comentário ao lado, que
//! *«a densidade nunca acrescenta superfície, e é lei e não omissão»*. A espec
//! diz outra coisa, e diz-a numa tabela-verdade de três linhas (§3.2): o pincel
//! **acrescenta a bandeira de colapso** ao modo do passe — ele não **retira** a
//! de partir. Quem decide o partir é o ajuste de *método de refino*:
//!
//! | ajuste de refino pede… | colapsar corre? | partir corre? |
//! |---|---|---|
//! | partir | **SIM** (é este pincel a pedi-lo) | sim |
//! | colapsar | sim | não |
//! | partir + colapsar | sim | sim |
//!
//! ⭐ **E a espec mede as duas células na MESMA malha grossa:** com o refino em
//! «só colapsar» ela fica em `81 → 81` (nada, porque não há aresta curta a
//! comer); com «partir + colapsar» ela vai a `81 → 101` — **cresce**. *As duas
//! leituras são do mesmo pincel; o que muda é o ajuste.*
//!
//! ⚠️ A recusa medida da espec — *«fazer o `Density` também subdividir»* — é
//! sobre o pincel **FORÇAR** o partir, como ele força o colapso. Essa continua
//! de pé: o partir é do ajuste, nunca do pincel.
//!
//! # ⭐ Porque o ajuste vive no PINCEL e não na cena
//!
//! O alvo guarda-o na cena, e a §9.8 da espec regista que **existe um pedido
//! público aberto para o tirar de lá e o pôr no pincel**. ⛔ *Não copiámos o
//! modelo de cena dele: ele está a caminho de o abandonar.* Aqui o modo é do
//! pincel que está na mão, que é onde o artista o procura.

/// **As duas direcções do pincel de densidade.**
///
/// ⚠️ **O colapso corre nas duas** — é a lei do pincel (espec §3.2) e não um
/// ajuste: ele acrescenta a bandeira de colapso aconteça o que acontecer. O que
/// este enum escolhe é **se o partir também corre**.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum DensityModo {
    /// **Leva a malha À densidade pedida, nos dois sentidos** — colapsa onde ela
    /// está fina demais e parte onde está grossa demais.
    ///
    /// ⭐ **É o de omissão, e a escolha tem duas razões que apontam ao mesmo
    /// lado.** A primeira é a referência: o ajuste de refino dela ship em
    /// *partir + colapsar*, que é a linha de baixo da tabela do cabeçalho. A
    /// segunda é o report que o fez nascer — com o pincel só a afinar, **ele só
    /// tem o que fazer quando o detalhe pedido está mais grosso que a malha**, e
    /// em toda a outra metade do curso do slider ele é indistinguível de uma
    /// ferramenta partida. *Um pincel que só age em metade dos ajustes é um
    /// pincel que o artista conclui que não funciona.*
    #[default]
    Igualar,
    /// **Só AFINA** — colapsa arestas curtas e nunca acrescenta superfície.
    ///
    /// É o ajuste de refino em *só colapsar*, e serve o gesto oposto: baixar a
    /// contagem de uma zona sem que a mesma passagem volte a adensar o que o
    /// artista acabou de aliviar.
    Afinar,
}

impl DensityModo {
    /// Os dois, na ordem em que a fileira os desenha.
    pub const ALL: [Self; 2] = [Self::Igualar, Self::Afinar];

    /// O rótulo que aparece no chip (a UI da casa é inglesa — HR/memória).
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Igualar => "Equalise",
            Self::Afinar => "Thin Only",
        }
    }

    /// **O passe também PARTE arestas longas neste modo?** — a porta única, e a
    /// razão de ela existir é a de sempre: o predicado do verbo pergunta para
    /// armar o motor e o painel perguntaria para explicar o chip. Duas cópias
    /// divergiriam no dia do terceiro modo.
    #[must_use]
    pub fn parte_arestas_longas(self) -> bool {
        matches!(self, Self::Igualar)
    }
}

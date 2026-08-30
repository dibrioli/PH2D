//! ⭐⭐⭐ **O CARÁCTER de uma mistura** — a **forma** da transição, e o número que diz o tamanho dela.
//!
//! ⚠️ **Arquivo irmão por LOC** (HR-18, W99): o `lib.rs` desta crate passou das 700 linhas quando o
//! chanfro entrou. ⛔ *Split, nunca allowlist* — e o corte é por assunto: aqui está tudo o que
//! responde *«que forma tem esta junta, e de que tamanho»*, e nada do que responde *«quem se junta a
//! quem»* (esse é o [`crate::Op`] e o [`crate::fold_verb`], que ficaram onde estavam).
//!
//! # ⚠️ As DUAS RÉGUAS, e confundi-las custou uma nota errada por meses
//!
//! | régua | o que mede | quem concorda com o filete |
//! |---|---|---|
//! | **recuo** | até onde a mistura sobe a parede | o chanfro (`1,02×`) · o orgânico **não** (`1,16×`) |
//! | **mordida** | onde fica a silhueta do canto | o orgânico (`1,00×`) · o chanfro **não** (`1,71×`) |
//!
//! **Nenhum carácter bate as duas**, e escolher qual calibrar é decisão de produto. Os números saem
//! de `ph2d-field-eval/tests/the_four_characters.rs`, que é o oráculo delas.

use serde::{Deserialize, Serialize};

/// O **caráter** do arredondamento de uma operação.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Blend {
    /// Aresta viva.
    Sharp,
    /// Raio **constante de verdade** — o *look* de produto, e o default do módulo.
    /// Medido: entrega o raio pedido com **0,00 %** de erro (ADR-0161 §3).
    Exact { radius: f32 },
    /// ⭐⭐⭐ **Corte RETO a 45º** (W99) — o chanfro.
    ///
    /// ⚠️ **`radius` é o alcance do corte ao longo de cada face**, e não a largura da faceta: num
    /// canto de 90º o plano do chanfro é `a + b = radius`, logo ele começa a `radius` de distância
    /// da quina em cada uma das duas superfícies. É a mesma convenção do filete, onde o arco também
    /// arranca a `radius` da quina — *dois caracteres, uma régua*.
    ///
    /// ⭐ No CAD, filete e chanfro são duas máquinas com modos de falha diferentes. Aqui são a mesma
    /// conta com um termo trocado (`ph2d_field_eval::ops::union_chamfer`), e **nenhuma pode falhar**.
    Chamfer { radius: f32 },
    /// Transição contínua ("derretida").
    ///
    /// ⚠️ **`radius` é o RAIO ENTREGUE**, calibrado — ver [`Blend::ORGANIC_REACH`]. A forma crua
    /// deste operador tem um alcance `k` que **não** é um raio, e mostrá-lo na UI ao lado dos outros
    /// três mentiria uma fracção fixa, sempre. *Quatro caracteres numa fileira têm de medir a mesma
    /// coisa, senão trocar de carácter muda o tamanho da peça.*
    Organic { radius: f32 },
}

impl Blend {
    /// ⭐⭐ **O ALCANCE CRU do smooth-min por raio entregue** (W99).
    ///
    /// ⚠️ **MEDIDO, não escolhido** — ver o gate `the_four_characters_measure_the_same_radius`, que
    /// o deriva de onde a superfície cruza a diagonal de um canto de 90º e o compara com o filete
    /// exacto no mesmo sítio. O número vive aqui, ao lado da variante que o consome, porque é o
    /// único sítio em que ele significa alguma coisa.
    ///
    /// ⭐⭐⭐ **E ela é ANALÍTICA, não um decimal ajustado:** `4 − 2√2`. O smooth-min polinomial vale
    /// `d − k/4` onde as duas superfícies estão à mesma distância `d`, e a mordida do filete exacto
    /// põe a silhueta em `d/√2` — igualar as duas dá `k = 4(1 − 1/√2)·d`. A medição confirma-a a
    /// `1,0000` (`the_four_characters`), e o gate `the_organic_blend_falls_short_by_exactly_k_over_four`
    /// prende a forma fechada.
    ///
    /// ⛔ **A nota antiga dizia «3/4» e mandava calibrar ×4/3, e as duas coisas eram sobre uma
    /// TERCEIRA grandeza** — o **valor do campo** no cotovelo, que não é nem o recuo nem a mordida.
    /// *Três réguas, e a que decide é a que o artista vê.*
    pub const ORGANIC_REACH: f32 = 4.0 - 2.0 * std::f32::consts::SQRT_2;

    /// ⭐⭐⭐ **O MESMO CARÁCTER, OUTRO NÚMERO** — a lei que todo gesto de raio partilha.
    ///
    /// ⚠️ **Uma porta, e não a mesma escada escrita em cada sítio.** Ela vive em dois caminhos (o
    /// filete de um grupo e o raio de junção de uma forma), e enquanto foi copiada os dois
    /// discordavam sobre o que um zero faz a um chanfro.
    ///
    /// ⚠️ **Zero não apaga o carácter dos que o têm por escolha.** Um `Chamfer { radius: 0 }` avalia
    /// exactamente como uma quina viva — e guarda a escolha, para que subir o número de volta não
    /// devolva um filete que ninguém pediu. O `Exact` colapsa em [`Blend::Sharp`] porque *ele* é o
    /// carácter que um raio positivo acorda: ali o zero não perde informação nenhuma.
    #[must_use]
    pub fn with_amount(self, amount: f32) -> Self {
        match self {
            Blend::Organic { .. } => Blend::Organic { radius: amount },
            Blend::Chamfer { .. } => Blend::Chamfer { radius: amount },
            Blend::Sharp | Blend::Exact { .. } if amount <= 0.0 => Blend::Sharp,
            Blend::Sharp | Blend::Exact { .. } => Blend::Exact { radius: amount },
        }
    }

    /// O raio desta mistura, ou `0.0` se for viva. ⭐ **Os quatro medem a MESMA coisa** — é o que
    /// torna a fileira de caracteres honesta.
    #[must_use]
    pub fn amount(self) -> f32 {
        match self {
            Blend::Sharp => 0.0,
            Blend::Exact { radius } | Blend::Chamfer { radius } | Blend::Organic { radius } => {
                radius
            }
        }
    }
}

/// ⭐⭐⭐ **O CARÁCTER de uma mistura** — a **forma** da transição, ao lado do número que diz o
/// tamanho dela.
///
/// ⚠️ **Três e não quatro:** a aresta **viva** não é um carácter, é o **raio zero**. Um quarto chip
/// «Sharp» seria uma segunda porta para o que o slider já faz, e as duas podiam discordar.
///
/// # ⚠️ O que os três partilham, e o que cada um NÃO partilha (medido, `the_four_characters`)
///
/// | | recuo na parede | mordida no canto |
/// |---|---|---|
/// | `Fillet` (o arco) | `1,00×` | `1,00×` |
/// | `Chamfer` (o corte reto) | **`1,00×`** | `1,71×` — é a FORMA dele |
/// | `Organic` (o derretido) | `1,16×` — divergência declarada | **`1,00×`** |
///
/// ⭐ A calibração do orgânico é feita pela **mordida** ([`Blend::ORGANIC_REACH`]) porque é a
/// silhueta que o artista vê: trocar de carácter com o mesmo número deixa o canto onde está.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Character {
    Fillet,
    Chamfer,
    Organic,
}

impl Character {
    /// ⚠️ **A fonte da contagem** — quem pinta a fileira deriva dela, e um carácter novo aparece na
    /// UI sem uma linha de mudança.
    pub const ALL: [Character; 3] = [Character::Fillet, Character::Chamfer, Character::Organic];

    /// O carácter desta mistura. ⚠️ **Uma aresta viva lê `Fillet`**, e é honesto: é o carácter que um
    /// raio positivo acorda ([`Blend::with_amount`]).
    #[must_use]
    pub fn of(blend: Blend) -> Self {
        match blend {
            Blend::Sharp | Blend::Exact { .. } => Character::Fillet,
            Blend::Chamfer { .. } => Character::Chamfer,
            Blend::Organic { .. } => Character::Organic,
        }
    }

    /// Esta mistura, com o carácter trocado e o **número mantido**.
    ///
    /// ⚠️ Trocar de carácter não é mexer num raio: quem carrega no chip escolheu a **forma**, e ver
    /// o número saltar junto seria o painel a decidir por ele.
    #[must_use]
    pub fn apply(self, blend: Blend) -> Blend {
        let amount = blend.amount();
        match self {
            Character::Fillet => Blend::Sharp.with_amount(amount),
            Character::Chamfer => Blend::Chamfer { radius: amount },
            Character::Organic => Blend::Organic { radius: amount },
        }
    }
}

/// ⭐⭐⭐ **UMA JUNTA — o chanfro E o filete, nesta ordem** (Enio, 2026-08-30).
///
/// > *«Poderíamos ter os 2, com chamfer antes de fillet para a possibilidade de arredondar as
/// > bordas geradas por chamfer»*
///
/// # Por que DOIS números, e não o carácter que já existe
///
/// A fileira de chips ([`Character`]) escolhe **um** carácter de cada vez, e é a forma certa para a
/// junta de um grupo: ali a pergunta é *«que forma tem esta mistura?»*. Aqui a pergunta é outra —
/// *«corta a quina, e depois arredonda o que o corte deixou?»* —, e ela **não se exprime** com um
/// carácter só: um chanfro seguido de filete tem três superfícies onde havia uma aresta.
///
/// ⭐ E os dois números medem a **mesma coisa que os chips medem**: o recuo ao longo de cada face
/// (ver [`Blend::Chamfer`]). Trocar um pelo outro não muda o tamanho da peça.
///
/// # ⚠️ A ORDEM é a do CAD, e é a que o pedido nomeia
///
/// O chanfro corta primeiro. As duas arestas que ele cria (face↔chanfro, dos dois lados) é que o
/// filete arredonda. ⛔ Ao contrário — filetar e depois chanfrar — o corte comeria o arco, e o
/// segundo número apagaria o primeiro.
///
/// # ⚠️ Zero é o estado de nascimento, e ele tem de ser BYTE-IDÊNTICO
///
/// `Joint::SHARP` avalia pelo caminho de sempre — nem um nó a mais na árvore. É o que permite dar
/// esta junta a modificadores que já existem sem mexer numa peça já autorada.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Joint {
    /// O recuo do corte reto a 45°, ao longo de cada uma das duas faces.
    pub chamfer: f32,
    /// O raio do arco que arredonda o que sobrou — as arestas do chanfro, ou a quina viva se
    /// `chamfer` for zero.
    pub fillet: f32,
}

impl Joint {
    /// A aresta viva: os dois números a zero.
    pub const SHARP: Self = Self {
        chamfer: 0.0,
        fillet: 0.0,
    };

    /// **Esta junta faz alguma coisa?** — a pergunta que escolhe entre o caminho de sempre e o novo.
    ///
    /// ⚠️ Um número **negativo** conta como viva: ele não é alcançável pelo painel (a faixa é
    /// positiva) e um ficheiro corrompido não pode fazer a árvore crescer.
    #[must_use]
    pub fn is_sharp(self) -> bool {
        !(self.chamfer > 0.0 || self.fillet > 0.0)
    }

    /// Até onde esta junta ACRESCENTA material, medido da quina para fora.
    ///
    /// ⚠️ **É o que o bordo da peça tem de crescer** — uma junta enche o vinco entre duas cópias, e
    /// um bordo que não a conte recorta a peça na marcha e na exportação. Os dois recuos somam-se
    /// porque o filete age sobre o que o chanfro deixou.
    #[must_use]
    pub fn reach(self) -> f32 {
        self.chamfer.max(0.0) + self.fillet.max(0.0)
    }
}

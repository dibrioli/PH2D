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
    // ─────────────────── W145 (pedido do Enio, 2026-09-09) ───────────────────
    //
    // ⚠️⚠️ **AS CINCO SEGUINTES SÃO APENDIDAS, e isso é load-bearing.** O `postcard` numera as
    // variantes pela ORDEM de declaração, e uma `Blend` viaja posicionalmente dentro do blob do
    // componente `ph2d_field_ecs::FieldVerb`. Acrescentar no FIM não move índice nenhum ⇒ toda
    // peça já gravada lê-se ao bit e o `PROJECT_SCHEMA` **não sobe**. ⛔ Pôr uma delas no meio, ou
    // acrescentar um campo a uma das quatro de cima, mudaria os bytes de valores que já existem —
    // e aí o degrau de schema é obrigatório (é o que os degraus 109 e 110 do `project_schema`
    // registam).
    /// ⭐⭐⭐ **A transição LISA de segunda ordem** (W145) — a mesma família do [`Blend::Organic`],
    /// um grau acima.
    ///
    /// O `Organic` é um polinómio de grau 2 e é `G1`: a **curvatura** dá um degrau onde a mistura
    /// começa, e é esse degrau que uma luz de estúdio desenha como uma banda na peça. Este é de
    /// grau 3 e é `G2` — o brilho corre pela junta sem a marca.
    ///
    /// **Medido** (`probe_the_other_junctions`, canto de 90°, todos à MESMA mordida): salto de
    /// curvatura `0,391` contra `1,290` do `Organic` e `2,880` do `Fillet` — `7,4×` mais liso que o
    /// filete —, com `16` nós contra `20` e `18`. *É mais liso E mais barato que os dois.*
    ///
    /// ⚠️ **`radius` é o RAIO ENTREGUE**, como no `Organic`; a calibração é a [`Blend::SOFT_REACH`].
    ///
    /// ⛔⛔ **E é ele que fecha a «fileira do cheio» numa entrada só.** A família da norma-`p`
    /// (`p = 4`, o «arco apertado») foi construída e medida: **à mesma mordida ela entrega a mesma
    /// peça** que este (recuo `0,3812` contra `0,3907`, `2,4 %`). *Dois chips para um look é ruído
    /// numa fileira*, e a medição escolheu o mais barato dos dois.
    Soft { radius: f32 },
    /// ⭐⭐⭐ **O CORDÃO sobre a costura** (W145) — um tubo de raio `radius` a correr por cima do
    /// encontro, como um cordão de solda, de cola ou de vedante.
    ///
    /// ⚠️ **Ele ACRESCENTA matéria onde as outras misturas só suavizam**, e é o único carácter que
    /// dá um vinco vivo de propósito: um cordão real encontra a chapa com uma quina, e é isso que o
    /// faz ler como cordão.
    ///
    /// **Medido:** `13` nós e `1,17` ns/ponto — **a junta mais barata de todas**, mais barata que o
    /// `Fillet` que já shipa.
    Bead { radius: f32 },
    /// ⭐⭐⭐ **O SULCO ao longo da costura** (W145) — um canal escavado sobre o encontro, que é a
    /// «linha de painel» com que uma peça passa a ler como montada de partes.
    ///
    /// `radius` é o **raio do canal**, medido a partir do vinco.
    ///
    /// ⛔⛔ **UM número, e o segundo foi RETIRADO por medição** (report do Enio, 09/09). A 1.ª
    /// versão tinha profundidade e meia-largura e localizava o canal em `|a − b| < largura` — o
    /// **conjunto medial**, que numa peça a sério é uma REGIÃO e não uma curva: ela esvaziava a base
    /// de um saliente e soltava-o da chapa. A cura localiza pelo **vinco** (`‖(a,b)‖`), e um tubo
    /// à volta de uma curva tem **um** raio. *O segundo número não foi perdido — ele nunca descreveu
    /// nada que existisse.* Ver `ops_bool::SO_QUEM_ESCAVA_PRECISA_DA_GUARDA`.
    Groove { radius: f32 },
    /// ⭐⭐⭐ **O FRISO ao longo da costura** (W145) — o oposto do [`Blend::Groove`]: uma nervura
    /// levantada sobre o encontro.
    ///
    /// `radius` é a **altura** e `width` a meia-largura. ⚠️ A altura é o que o bordo da peça tem de
    /// crescer, e é por isso que ela — e não a largura — é o [`Blend::amount`].
    Ridge { radius: f32, width: f32 },
    /// ⭐⭐ **O CHANFRO DE DOIS RECUOS** (W145) — o *two-distance chamfer* do CAD.
    ///
    /// `radius` é o recuo do lado do que já estava, e `bias` multiplica-o do lado da forma que
    /// chega: `1,0` é o chanfro simétrico.
    ///
    /// ⛔⛔ **Ele não tem chip próprio, e a ausência é a decisão.** O [`Character::of`] manda-o para
    /// o `Chamfer`, porque a pergunta *«que forma tem esta junta?»* tem **uma** resposta — «um corte
    /// reto» — e o desequilíbrio é um número dela, não outra forma. Dois chips fariam o artista
    /// escolher entre duas palavras para a mesma coisa.
    ///
    /// ⚠️ **`bias == 1,0` volta a ser [`Blend::Chamfer`]** ([`Blend::with_second`]): um valor tem
    /// **uma** representação, senão dois documentos idênticos na tela diferem nos bytes.
    Bevel { radius: f32, bias: f32 },
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

    /// ⭐⭐⭐ **O mesmo alcance cru, para o polinómio de GRAU 3** (W145) — `6 − 3√2`.
    ///
    /// ⭐⭐ **E os dois números são a MESMA lei, com o grau dentro.** Onde as duas superfícies estão
    /// à distância `d`, o polinómio de grau `n` desce `k/(2n − 2)`: o de grau 2 desce `k/4` e o de
    /// grau 3 desce `k/6`. Igualar isso à mordida do filete exacto (`d/√2`) dá
    /// `k = 2(n − 1)(1 − 1/√2)·d` — que é `4 − 2√2` num e `6 − 3√2` no outro.
    ///
    /// ⚠️ **MEDIDO antes de escrito:** a sonda correu o cúbico a `k = 1,6 r` e leu mordida
    /// `0,0943`; a forma fechada prevê `1,6 × 0,25 × √2/6 = 0,09428`. *A previsão e a leitura
    /// batem nos cinco algarismos, e é isso que autoriza a constante analítica.*
    pub const SOFT_REACH: f32 = 6.0 - 3.0 * std::f32::consts::SQRT_2;

    /// ⭐ **A meia-largura com que um sulco ou um friso NASCEM**, em fracção do tamanho deles.
    ///
    /// ⚠️ **Um número de nascimento, nunca uma amarra:** o segundo número é editável e sobrevive a
    /// mexer no primeiro ([`Blend::with_amount`]). Ele existe porque um carácter escolhido no chip
    /// tem de mostrar **alguma coisa** na hora — um sulco de largura zero é uma junta viva com um
    /// nome bonito.
    ///
    /// O valor é o da bancada (`probe_the_other_junctions`, `0,35 / 0,5 = 0,7`), que é onde as
    /// secções foram lidas como sulco e como friso.
    pub const SEAM_WIDTH_RATIO: f32 = 0.7;

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
            // ⚠️ **O segundo número SOBREVIVE ao primeiro** — mexer na profundidade de um sulco não
            // pode apagar a largura que o artista escolheu. É a mesma lei que faz o carácter
            // sobreviver a um raio novo, um nível abaixo.
            Blend::Soft { .. } => Blend::Soft { radius: amount },
            Blend::Bead { .. } => Blend::Bead { radius: amount },
            Blend::Groove { .. } => Blend::Groove { radius: amount },
            Blend::Ridge { width, .. } => Blend::Ridge {
                radius: amount,
                width,
            },
            Blend::Bevel { bias, .. } => Blend::Bevel {
                radius: amount,
                bias,
            },
            Blend::Sharp | Blend::Exact { .. } if amount <= 0.0 => Blend::Sharp,
            Blend::Sharp | Blend::Exact { .. } => Blend::Exact { radius: amount },
        }
    }

    /// O raio desta mistura, ou `0.0` se for viva. ⭐ **Todos medem a MESMA coisa** — é o que
    /// torna a fileira de caracteres honesta.
    ///
    /// ⚠️ **E é também o que o BORDO da peça tem de crescer** (`ph2d_field_eval::bounds`), e é por
    /// isso que nas duas juntas de dois números o `radius` é a grandeza que sai para FORA (a altura
    /// de um friso, a profundidade de um sulco) e nunca a largura.
    #[must_use]
    pub fn amount(self) -> f32 {
        match self {
            Blend::Sharp => 0.0,
            Blend::Exact { radius }
            | Blend::Chamfer { radius }
            | Blend::Organic { radius }
            | Blend::Soft { radius }
            | Blend::Bead { radius }
            | Blend::Groove { radius }
            | Blend::Ridge { radius, .. }
            | Blend::Bevel { radius, .. } => radius,
        }
    }

    /// ⭐⭐⭐ **O SEGUNDO NÚMERO desta mistura, se ela tiver um** — com o rótulo e a faixa ao lado
    /// dele (W145).
    ///
    /// ⚠️ **A linha do painel DERIVA daqui**, e é por isso que a chave e a faixa vivem coladas à
    /// variante que as possui: um carácter novo com segundo número aparece no painel sem uma linha
    /// de mudança lá, e um sem ele não oferece controle nenhum. *É a mesma lei da fileira de chips,
    /// que já é derivada do [`Character::ALL`].*
    ///
    /// ⚠️ **A faixa da largura é [`Span::Positive`] e não uma parede:** uma meia-largura maior do
    /// que a profundidade dá um sulco raso e largo, que é uma peça legítima. O que a fecha é a
    /// vista, como em toda largura deste módulo.
    #[must_use]
    pub fn second(self) -> Option<crate::Dim> {
        match self {
            Blend::Ridge { width, .. } => Some(crate::Dim {
                key: "field.dim.seam_width",
                value: width,
                span: crate::Span::Positive,
            }),
            Blend::Bevel { bias, .. } => Some(crate::Dim {
                key: "field.dim.bevel_bias",
                value: bias,
                // ⚠️⚠️ **SEM PAREDE, e a ausência é medida (§0):** o campo continua a ser uma
                // distância com qualquer desequilíbrio — o plano é normalizado e entra por um `min`
                // com a união, logo é sempre minorante. O que existe é ESCALA (um corte muito
                // desigual come a peça), e essa é a mesma resposta que o `radius_bound` já dá a
                // toda mistura: `Soft`, fechada pela vista. *Um tecto que só dissesse «por
                // segurança» seria um palpite à espera de um smoke.*
                span: crate::Span::Positive,
            }),
            // ⚠️⚠️ **O chanfro SIMÉTRICO oferece a linha, com `1,0` dentro** — sem isto o
            // desequilíbrio seria **inalcançável**: o painel só desenha o que o `second` devolve, e
            // um `Bevel` só nasce de alguém escrever nesta linha. *Um controlo que só aparece
            // depois de já ter sido usado não existe.*
            Blend::Chamfer { .. } => Some(crate::Dim {
                key: "field.dim.bevel_bias",
                value: 1.0,
                span: crate::Span::Positive,
            }),
            Blend::Sharp
            | Blend::Exact { .. }
            | Blend::Organic { .. }
            | Blend::Soft { .. }
            | Blend::Bead { .. }
            | Blend::Groove { .. } => None,
        }
    }

    /// A mesma mistura, com o **segundo** número trocado. Devolve-se a si própria quando não tem um.
    ///
    /// ⭐⭐ **O `bias` de volta a `1,0` VOLTA A SER um [`Blend::Chamfer`]**, e não um `Bevel` com um
    /// nesse campo: *um valor tem uma representação*. Sem isto, dois documentos que a tela mostra
    /// iguais teriam bytes diferentes, e o caminho rápido do chanfro simétrico (que é o de sempre,
    /// ao bit) deixaria de ser tomado por quem lá voltou.
    #[must_use]
    pub fn with_second(self, second: f32) -> Self {
        match self {
            Blend::Ridge { radius, .. } => Blend::Ridge {
                radius,
                width: second,
            },
            Blend::Chamfer { radius } | Blend::Bevel { radius, .. } => {
                if (second - 1.0).abs() < f32::EPSILON {
                    Blend::Chamfer { radius }
                } else {
                    Blend::Bevel {
                        radius,
                        bias: second,
                    }
                }
            }
            Blend::Sharp
            | Blend::Exact { .. }
            | Blend::Organic { .. }
            | Blend::Soft { .. }
            | Blend::Bead { .. }
            | Blend::Groove { .. } => self,
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
    /// ⭐ O [`Blend::Soft`] — a transição lisa de segunda ordem (W145).
    Soft,
    /// ⭐ O [`Blend::Bead`] — o cordão sobre a costura (W145).
    Bead,
    /// ⭐ O [`Blend::Groove`] — o sulco sobre a costura (W145).
    Groove,
    /// ⭐ O [`Blend::Ridge`] — o friso sobre a costura (W145).
    Ridge,
}

impl Character {
    /// ⚠️ **A fonte da contagem** — quem pinta a fileira deriva dela, e um carácter novo aparece na
    /// UI sem uma linha de mudança.
    ///
    /// ⚠️ **A ORDEM é a da leitura, e não a da implementação:** primeiro as quatro que **suavizam**
    /// a quina, da mais dura para a mais macia; depois as três que põem uma **feição** sobre a
    /// costura. Um artista que percorre a fileira da esquerda para a direita vê a transição a
    /// derreter e depois a ganhar relevo.
    ///
    /// ⚠️ O tecto do painel é o `MAX_MODES` (`16`), e a fileira **envolve linha** — o gate
    /// `the_panel_has_a_slot_for_every_character` mantém a folga do lado seguro.
    pub const ALL: [Character; 7] = [
        Character::Fillet,
        Character::Chamfer,
        Character::Organic,
        Character::Soft,
        Character::Bead,
        Character::Groove,
        Character::Ridge,
    ];

    /// O carácter desta mistura. ⚠️ **Uma aresta viva lê `Fillet`**, e é honesto: é o carácter que um
    /// raio positivo acorda ([`Blend::with_amount`]).
    #[must_use]
    pub fn of(blend: Blend) -> Self {
        match blend {
            Blend::Sharp | Blend::Exact { .. } => Character::Fillet,
            // ⭐⭐ **O `Bevel` lê-se CHANFRO**, e é a linha que impede um segundo chip para a mesma
            // forma — ver o doc do [`Blend::Bevel`].
            Blend::Chamfer { .. } | Blend::Bevel { .. } => Character::Chamfer,
            Blend::Organic { .. } => Character::Organic,
            Blend::Soft { .. } => Character::Soft,
            Blend::Bead { .. } => Character::Bead,
            Blend::Groove { .. } => Character::Groove,
            Blend::Ridge { .. } => Character::Ridge,
        }
    }

    /// Esta mistura, com o carácter trocado e o **número mantido**.
    ///
    /// ⚠️ Trocar de carácter não é mexer num raio: quem carrega no chip escolheu a **forma**, e ver
    /// o número saltar junto seria o painel a decidir por ele.
    #[must_use]
    pub fn apply(self, blend: Blend) -> Blend {
        let amount = blend.amount();
        // ⭐⭐ **A meia-largura SOBREVIVE à troca de carácter quando o destino também a tem** — um
        // sulco que vira friso mantém a largura que o artista escolheu. Quem não a tem semeia-a do
        // tamanho ([`Blend::SEAM_WIDTH_RATIO`]), porque um segundo número a zero seria um carácter
        // escolhido que não mostra nada.
        let width = match blend {
            Blend::Ridge { width, .. } => width,
            _ => amount * Blend::SEAM_WIDTH_RATIO,
        };
        match self {
            Character::Fillet => Blend::Sharp.with_amount(amount),
            // ⚠️ **O desequilíbrio sobrevive a sair e voltar ao chanfro** — sair para outro carácter
            // e voltar não pode apagar um número que o artista escreveu, e é a mesma lei do
            // [`Blend::with_amount`].
            Character::Chamfer => match blend {
                Blend::Bevel { bias, .. } => Blend::Bevel {
                    radius: amount,
                    bias,
                },
                _ => Blend::Chamfer { radius: amount },
            },
            Character::Organic => Blend::Organic { radius: amount },
            Character::Soft => Blend::Soft { radius: amount },
            Character::Bead => Blend::Bead { radius: amount },
            Character::Groove => Blend::Groove { radius: amount },
            Character::Ridge => Blend::Ridge {
                radius: amount,
                width,
            },
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

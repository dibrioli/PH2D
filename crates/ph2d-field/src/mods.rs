//! ⭐ **Os modificadores de um nó** — a casca e o afastamento.
//!
//! # Por que estes dois, e por que aqui
//!
//! São a **tese do módulo** dita numa linha de aritmética, como a booleana e o filete:
//!
//! | verbo | a conta | por que ela não pode falhar |
//! |---|---|---|
//! | **casca** | `\|f\| − t` | o valor absoluto de uma distância **é** a distância à mesma superfície, vista dos dois lados. Não há costura a fechar, não há auto-intersecção a resolver, não há espessura mínima |
//! | **afastamento** | `f − d` | deslocar a superfície por uma distância é o que uma distância assinada **é**. Cresce com `d > 0`, encolhe com `d < 0` |
//!
//! ⚠️ Numa malha, **a casca é a operação que falha**: ela pede um offset da superfície, e um offset
//! de malha auto-intersecta em toda concavidade mais apertada do que a espessura. É por isso que
//! todo modelador de malha tem um botão de casca com uma lista de exceções ao lado. Aqui a lista
//! não existe, e é essa a razão de o módulo ser um campo.
//!
//! # ⚠️ O afastamento ARREDONDA a quina convexa, e é de propósito
//!
//! `f − d` com `d > 0` transforma cada aresta convexa num arco de raio exatamente `d`, e deixa a
//! côncava viva — é o mesmo operador que a [`crate::Primitive`] usa para o `round` dela
//! ([`ph2d_field_eval::ops::offset`]). Quem quiser crescer **sem** arredondar quer outra operação,
//! e ela não existe neste módulo: é a receita canônica que o campo entrega, não um defeito.
//!
//! # A PILHA, e de onde ela vem
//!
//! Os modificadores de um nó são uma **lista ordenada**, e não um grafo: encascar-e-afastar não é
//! o mesmo que afastar-e-encascar, e a ordem tem de ser dita. É a mesma forma que os *Live Path
//! Effects* do vetorial escolheram e mediram ([ADR-0132]: *"uma pilha por path, não um grafo de
//! nós"*) — e pela mesma razão: um grafo paga um editor de grafo para exprimir uma sequência.
//!
//! # ⚠️ Os números são LOCAIS, como as dimensões
//!
//! A pilha corre **antes** da pose ([`ph2d_field_eval`] aplica `place` por cima), então uma
//! espessura de `0,02` num nó escalado 2× dá parede de `0,04` no mundo — exatamente como a largura
//! de uma caixa dentro de um grupo escalado. *Uma regra para todo número deste módulo*, em vez de
//! uma exceção que só aparece quando alguém agrupa.
//!
//! [ADR-0132]: ../../../docs/architecture/decisions/0132-vector-live-path-effects-are-a-per-path-stack-not-a-node-graph.md

use crate::{FieldError, Span};
use serde::{Deserialize, Serialize};

/// Um modificador aplicado ao campo de um nó, **depois** do que ele é e **antes** da pose dele.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Unary {
    /// **Casca**: esvazia o sólido e deixa uma parede de espessura `thickness`, centrada na
    /// superfície que lá estava.
    ///
    /// ⚠️ A parede é **centrada**, e é o que `|f| − t` entrega: metade para dentro, metade para
    /// fora. Uma casca *"só para dentro"* é `|f + t| − t`, e é outra decisão — de produto, com o
    /// número na mão de quem a pedir.
    Shell { thickness: f32 },
    /// **Afastamento**: move a superfície por `distance`. Positivo cresce (e arredonda a quina
    /// convexa); negativo encolhe.
    Offset { distance: f32 },
    /// **Espelho** no plano `x = 0` do nó: o que existe de um lado passa a existir dos dois.
    ///
    /// ⚠️ **Sem número nenhum** — e o eixo é o **X local**. Os outros dois têm variante própria
    /// ([`Unary::MirrorY`], [`Unary::MirrorZ`]); ver lá **por que a cerca que dizia «roda o nó»
    /// caiu**.
    Mirror,
    /// **Matriz linear**: `count` cópias espaçadas de `spacing` ao longo do **X local**.
    ///
    /// ⚠️ Mesmo eixo, mesma razão do [`Unary::Mirror`].
    Array { count: u32, spacing: f32 },
    /// **Inclinação (draft/taper)**: a secção transversal cresce ou encolhe ao longo do **Y local**,
    /// à razão de `slope` por unidade de altura.
    ///
    /// ⚠️ **É o primeiro modificador deste módulo que NÃO devolve uma distância exata.** Ver o doc
    /// do módulo e [`ph2d_field_eval`], onde o preço está medido.
    Taper { slope: f32 },
    /// **Matriz radial**: `count` cópias em círculo, em torno do **Z local**.
    ///
    /// ⚠️ **Z, e não X** — e o critério não é a coerência com os irmãos acima, é a **coerência com
    /// a peça**: o [`crate::Primitive::Cylinder`] aponta em Z, e uma coroa de parafusos à volta de
    /// um flange gira em torno do eixo dele. Cada modificador nomeia o seu eixo e diz porquê, que é
    /// o que as primitivas já fazem (o cilindro é Z, o torno é Y, e cada um tem a razão escrita).
    Radial { count: u32 },
    /// **Espelho** no plano `y = 0` do nó. Ver [`Unary::MirrorZ`] para a razão de existir.
    MirrorY,
    /// **Espelho** no plano `z = 0` do nó.
    ///
    /// # ⛔ A cerca que estava escrita, e por que ela caiu (2026-08-26)
    ///
    /// O doc do [`Unary::Mirror`] dizia: *«sem escolha de eixo — quem quer outro **roda o nó**, e uma
    /// escolha de eixo por modificador seria um terceiro vocabulário de orientação no mesmo
    /// módulo»*. ⚠️ **A analogia com o `Cylinder` não se aplica**, e é aí que ela falha: rodar o nó
    /// roda a **peça**, e o espelho age no espaço **local**, *antes* da pose do nó — para espelhar em
    /// Y por rotação seria preciso um nó intermédio só para rodar, espelhar e desrodar. *Uma
    /// equivalência que exige uma terceira entidade não é uma equivalência: é um contorno.*
    ///
    /// ⇒ decisão do Enio, 2026-08-26, depois de o ver a funcionar em X: **três botões**.
    ///
    /// ⚠️ **Variantes NOVAS, e no fim da lista** — e as duas coisas são a mesma razão: o documento
    /// serializa por **posição**, então um campo `axis` dentro do `Mirror` (ou uma variante no meio)
    /// mudaria o significado dos bytes de **toda peça já gravada**. *Append-only é o que faz uma
    /// extensão não ser uma migração.*
    ///
    /// ⚠️ E os irmãos de eixo — [`Unary::Array`] (X) e [`Unary::Radial`] (Z) — **ficam como estão**:
    /// eles têm número, e uma matriz por eixo é outra pergunta (três botões × dois números). O que
    /// o Enio pediu foi o espelho.
    MirrorZ,
}

/// Quantas cópias uma matriz consegue ter.
///
/// ⚠️ **É um limite de CUSTO, e ele está medido**: a repetição é feita por dobra do domínio, então
/// N cópias custam o mesmo que uma — o que cresce é o número de **células vizinhas** que a lei tem
/// de olhar para o campo continuar uma distância honesta (ver `ph2d_field_eval`), e isso é 2 por
/// eixo, constante em N. O teto existe pelo outro lado: uma matriz de 4096 cópias sai do
/// enquadramento e do orçamento de traçado muito antes de custar alguma coisa.
///
/// 64 é o que a peça inteira comporta a `half_extent` normal com espaçamento visível; acima disso a
/// matriz já não cabe no quadro, e o que o artista vê é uma parede.
pub const MAX_ARRAY_COUNT: u32 = 64;

impl Unary {
    /// A chave i18n do nome. ⚠️ Uma **chave**, nunca um rótulo pronto (HR-15).
    #[must_use]
    pub fn key(self) -> &'static str {
        self.kind().key()
    }

    /// ⭐ **Os números deste modificador**, na ordem em que o painel os mostra.
    ///
    /// ⚠️ **Vários, e não um** — foi o que a matriz forçou, e é a forma certa: uma matriz tem
    /// quantas cópias **e** que espaçamento, e enfiá-las em dois modificadores separados seria
    /// partir uma coisa em duas para caber num campo. É a mesma forma que [`crate::dims`] já usa
    /// para uma primitiva — *um vocabulário, não dois*.
    ///
    /// Um modificador **sem números** (o espelho) devolve vazio, e o painel não pinta linha nenhuma
    /// para ele: o chip aceso já diz tudo o que há para dizer.
    #[must_use]
    pub fn dims(self) -> Vec<crate::Dim> {
        match self {
            Unary::Shell { thickness } => vec![crate::Dim {
                key: "field.mod.thickness",
                value: thickness,
                // Sem parede: uma casca mais grossa do que a peça deixa de ser oca, o que é uma
                // forma legítima e não um erro. O alcance útil é o da vista.
                span: Span::Positive,
            }],
            // ⚠️ **Simétrica**, e é metade da razão de o afastamento existir: encolher é o gesto de
            // folga de encaixe. Uma faixa só positiva mataria metade da ferramenta.
            Unary::Offset { distance } => vec![crate::Dim {
                key: "field.mod.distance",
                value: distance,
                span: Span::Free,
            }],
            Unary::Mirror | Unary::MirrorY | Unary::MirrorZ => Vec::new(),
            Unary::Array { count, spacing } => vec![
                crate::Dim {
                    key: "field.mod.count",
                    value: count as f32,
                    span: Span::Count {
                        min: 1,
                        max: MAX_ARRAY_COUNT,
                    },
                },
                crate::Dim {
                    key: "field.mod.spacing",
                    value: spacing,
                    span: Span::Positive,
                },
            ],
            // ⚠️ **Sem espaçamento**: numa coroa o espaçamento é o próprio ângulo, e ele já está
            // dito pela contagem (`2π/n`). Um segundo número aqui seria uma forma de pedir uma
            // coroa incompleta — que é outra feature, com outro nome.
            Unary::Taper { slope } => vec![crate::Dim {
                key: "field.mod.slope",
                value: slope,
                // ⚠️ Uma parede dos **dois** lados: inclinar para dentro e para fora são os dois
                // gestos, e o teto é do CUSTO da marcha (ver `MAX_TAPER_SLOPE`).
                span: Span::Walls(MAX_TAPER_SLOPE),
            }],
            Unary::Radial { count } => vec![crate::Dim {
                key: "field.mod.count",
                value: count as f32,
                span: Span::Count {
                    min: 1,
                    max: MAX_ARRAY_COUNT,
                },
            }],
        }
    }

    /// ⭐ **Escreve um dos números**, ou recusa — a porta única.
    ///
    /// # Errors
    /// [`FieldError::NonPositive`] para um valor não-finito, para um índice que não é deste
    /// modificador, e para os números cujo zero não quer dizer nada (uma casca sem parede não é uma
    /// casca; uma matriz de espaçamento zero é N cópias no mesmo sítio).
    pub fn set_dim(&mut self, node: u32, field: u8, value: f32) -> Result<(), FieldError> {
        let bad = |what: &'static str| FieldError::NonPositive { node, what };
        if !value.is_finite() {
            return Err(bad("mod"));
        }
        match (&mut *self, field) {
            (Unary::Shell { thickness }, 0) => {
                if value <= 0.0 {
                    return Err(bad("thickness"));
                }
                *thickness = value;
            }
            // ⚠️ **Zero é legítimo aqui**: um afastamento de zero é o campo intacto, e é o ponto por
            // onde o número passa ao ir de encolher para crescer. Recusá-lo faria o slider ter um
            // buraco no meio.
            (Unary::Offset { distance }, 0) => *distance = value,
            (Unary::Array { count, .. }, 0) => {
                // ⚠️ **O documento é quem arredonda.** O painel mostra um inteiro porque a faixa diz
                // que ele é um (`Span::Count`), mas quem garante é esta linha — um valor fracionário
                // que chegasse por outra porta viraria `count` na mesma, e não meia cópia.
                if value < 1.0 {
                    return Err(bad("count"));
                }
                *count = (value.round() as u32).min(MAX_ARRAY_COUNT);
            }
            (Unary::Array { spacing, .. }, 1) => {
                if value <= 0.0 {
                    return Err(bad("spacing"));
                }
                *spacing = value;
            }
            (Unary::Taper { slope }, 0) => {
                *slope = value.clamp(-MAX_TAPER_SLOPE, MAX_TAPER_SLOPE);
            }
            (Unary::Radial { count }, 0) => {
                if value < 1.0 {
                    return Err(bad("count"));
                }
                *count = (value.round() as u32).min(MAX_ARRAY_COUNT);
            }
            _ => return Err(bad("mod")),
        }
        Ok(())
    }

    /// **Um modificador novo, no ponto NEUTRO da sua natureza.**
    ///
    /// ⚠️ Neutro quer dizer coisas diferentes nos dois, e é por isso que não há um default só: um
    /// afastamento de zero é literalmente nada a acontecer, e é o sítio certo para começar a
    /// arrastar. Uma casca de zero seria **recusada** pela própria porta acima — então ela nasce
    /// numa fração da peça, e o número vem de fora ([`crate::characteristic_size`]), porque só quem
    /// vê a peça sabe o que é fino nela.
    #[must_use]
    pub fn born(kind: UnaryKind, scale: f32) -> Unary {
        let fraction = |f: f32| (scale * f).max(f32::MIN_POSITIVE);
        match kind {
            UnaryKind::Shell => Unary::Shell {
                thickness: fraction(SHELL_BIRTH_FRACTION),
            },
            UnaryKind::Offset => Unary::Offset { distance: 0.0 },
            UnaryKind::Mirror => Unary::Mirror,
            UnaryKind::MirrorY => Unary::MirrorY,
            UnaryKind::MirrorZ => Unary::MirrorZ,
            // ⚠️ **Duas cópias, e o espaçamento é a própria peça.** Uma matriz nasce com o número
            // mínimo que se **vê** ser uma matriz (uma cópia é a peça intacta), e com as duas
            // encostadas — que é onde o artista começa a afastá-las. Um espaçamento menor do que a
            // peça faria as cópias nascerem sobrepostas, e a lei do campo tem um bound aí (ver
            // `ph2d_field_eval`).
            UnaryKind::Array => Unary::Array {
                count: 2,
                spacing: fraction(ARRAY_BIRTH_SPAN),
            },
            // ⚠️ **Seis, e não dois.** Numa coroa, duas cópias a 180° leem-se como um espelho e não
            // como uma coroa — o gesto não se explica sozinho. Seis é o menor número em que a
            // circularidade é imediata, e é o que uma flange de verdade costuma ter.
            // Zero é o ponto neutro: a peça intacta, e o sítio de onde se começa a arrastar.
            UnaryKind::Taper => Unary::Taper { slope: 0.0 },
            UnaryKind::Radial => Unary::Radial { count: 6 },
        }
    }

    /// De que **natureza** este modificador é — o que o botão do painel escolhe.
    #[must_use]
    pub fn kind(self) -> UnaryKind {
        match self {
            Unary::Shell { .. } => UnaryKind::Shell,
            Unary::Offset { .. } => UnaryKind::Offset,
            Unary::Mirror => UnaryKind::Mirror,
            Unary::MirrorY => UnaryKind::MirrorY,
            Unary::MirrorZ => UnaryKind::MirrorZ,
            Unary::Array { .. } => UnaryKind::Array,
            Unary::Taper { .. } => UnaryKind::Taper,
            Unary::Radial { .. } => UnaryKind::Radial,
        }
    }
}

/// Que fração da menor peça uma casca nova mede.
///
/// ⚠️ **Um décimo, e o recurso é a VISIBILIDADE**: a parede tem de se ver no primeiro quadro (senão
/// o botão parece não ter feito nada) e tem de deixar buraco (senão a peça continua a parecer
/// maciça). Entre `1/20` — invisível a 480 px numa peça que ocupa meio quadro — e `1/4`, que quase
/// não deixa vazio, um décimo é o degrau que cumpre as duas.
const SHELL_BIRTH_FRACTION: f32 = 0.1;

/// Que fração da menor peça uma matriz nova usa de espaçamento.
///
/// ⚠️ **Duas vezes a peça**, e o recurso é a **lei do campo**: a repetição por dobra do domínio é
/// uma distância exata enquanto a forma cabe na célula, e nascer com o dobro põe a matriz nesse
/// regime de origem — o artista vê duas cópias separadas e limpas, e é ele que decide apertá-las.
const ARRAY_BIRTH_SPAN: f32 = 2.0;

/// **Até onde a inclinação vai**, e o recurso é o **custo da marcha de raios**.
///
/// ⚠️ Escrito por **MEDIÇÃO**, e as duas tabelas estão ao lado dos números que as produziram.
/// A inclinação deforma o domínio, e o campo que sai é um **bound** conservador: para nunca
/// superestimar (a condição de a marcha não atravessar a peça) ele divide por `1 + 2·|declive|`, e
/// a marcha paga isso em passos.
///
/// **O custo REAL, medido** (`measure_taper_frame_cost`, quadro de 320×240):
///
/// | declive | ms/quadro | razão |
/// |---|---|---|
/// | 0,00 | 9,89 | 1,00× |
/// | 0,25 | 12,22 | 1,24× |
/// | 0,50 | 15,09 | 1,53× |
/// | **1,00** | **20,00** | **2,02×** |
/// | 1,50 | 24,77 | 2,51× |
///
/// ⭐ **E a primeira medição enganava.** A sonda do `‖∇f‖` diz que o **pior passo** no declive 1 é
/// 1/300 de um passo cheio — o que sugeriria um teto muito mais baixo. O quadro custa **2,02×**:
/// pouquíssimos pixels pagam o pior passo. *O pior caso não é o custo; o quadro é.*
///
/// Não há joelho — o custo sobe liso. Então o teto é uma escolha de **orçamento**, e o número
/// escrito é o que ele compra: **no teto, o traçado custa o dobro**. Declive 1 é 45°, generoso para
/// o que um draft de moldagem pede (1° a 5°) e suficiente para dar forma.
pub const MAX_TAPER_SLOPE: f32 = 1.0;

/// A **natureza** de um modificador, sem o número dele — o que um botão nomeia.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnaryKind {
    Shell,
    Offset,
    Mirror,
    Array,
    Radial,
    Taper,
    MirrorY,
    MirrorZ,
}

impl UnaryKind {
    /// ⭐ **A fonte da contagem.** O painel deriva os botões daqui, como já faz com `Mode::ALL` — um
    /// modificador novo acrescenta-se aqui e o painel segue sem uma linha de mudança.
    pub const ALL: [UnaryKind; 8] = [
        UnaryKind::Shell,
        UnaryKind::Offset,
        UnaryKind::Mirror,
        UnaryKind::MirrorY,
        UnaryKind::MirrorZ,
        UnaryKind::Array,
        UnaryKind::Radial,
        UnaryKind::Taper,
    ];

    /// A chave i18n do botão que o acrescenta.
    #[must_use]
    pub fn key(self) -> &'static str {
        match self {
            UnaryKind::Shell => "panel.model3d.mod.shell",
            UnaryKind::Offset => "panel.model3d.mod.offset",
            UnaryKind::Mirror => "panel.model3d.mod.mirror",
            UnaryKind::MirrorY => "panel.model3d.mod.mirror_y",
            UnaryKind::MirrorZ => "panel.model3d.mod.mirror_z",
            UnaryKind::Array => "panel.model3d.mod.array",
            UnaryKind::Radial => "panel.model3d.mod.radial",
            UnaryKind::Taper => "panel.model3d.mod.taper",
        }
    }
}

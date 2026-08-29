//! O **estilo do traço** — irmão de `lib.rs` pelo teto de 700 LOC daquele arquivo.
//!
//! A caneta: largura, ponta, junção, tracejado e as PONTAS (arrowheads). O que ela
//! DESENHA com isso é outra pergunta, e mora em [`crate::stroke_plan`].

use serde::{Deserialize, Serialize};

use crate::{Marker, Rgba8};

/// Ponta do traço (mapeia p/ `kurbo::Cap` no render). Default = `Butt`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LineCap {
    #[default]
    Butt,
    Round,
    Square,
}

/// Junção entre segmentos (mapeia p/ `kurbo::Join`). Default = `Miter`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LineJoin {
    #[default]
    Miter,
    Round,
    Bevel,
}

/// **Qual contorno o Offset Path move** — num compound (uma forma com furos), a borda de fora
/// e as bordas dos furos são coisas separadas, e o artista pode querer mover só uma.
///
/// O offset é **por contorno**: `d > 0` empurra o contorno escolhido para FORA (ao longo da
/// sua normal externa, para longe do miolo que ele fecha); `d < 0` para dentro. Assim a quina
/// (Round/Bevel) aparece em QUALQUER contorno que se expanda — não só no de fora, que era a
/// queixa do smoke (`2026-07-20`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum OffsetSide {
    /// Só o contorno de fora. `d > 0` cresce a forma.
    Outer,
    /// Só os contornos de furo. `d > 0` cresce os furos (para dentro da tinta).
    Inner,
    /// Todos os contornos. `d > 0` expande cada um pela sua normal externa.
    #[default]
    Both,
}

impl OffsetSide {
    /// Este modo move o contorno de FORA?
    #[must_use]
    pub fn hits_outer(self) -> bool {
        matches!(self, OffsetSide::Outer | OffsetSide::Both)
    }

    /// Este modo move os contornos de FURO?
    #[must_use]
    pub fn hits_inner(self) -> bool {
        matches!(self, OffsetSide::Inner | OffsetSide::Both)
    }
}

/// **De que lado da linha a tinta do traço cai** — o *Align Stroke* do Illustrator / o
/// *stroke-alignment* do Figma.
///
/// Um traço centrado gasta metade da largura para dentro da forma e metade para fora, então
/// **engrossá-lo muda a silhueta**: um botão de 100 px com borda de 20 mede 120. Inner o prende à
/// borda (a silhueta não se mexe, que é o que um contorno de UI quer) e Outer o deita todo por
/// fora (a moldura, o *sticker*, o realce que não pode comer o desenho).
///
/// ⚠️ **É uma pergunta sobre REGIÃO, e uma linha aberta não tem uma.** *Dentro* e *fora* só
/// existem onde há interior; num caminho aberto os dois nomes não significam nada, e é por isso
/// que quem oferece a escolha (o painel) e quem a executa (a booleana) perguntam ambos a
/// [`Self::needs_a_region`] — a mesma função, para o botão nunca prometer o que a geometria
/// recusa.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum StrokeAlign {
    /// Metade para cada lado. O default, e o que todo desenho já feito tem.
    #[default]
    Centre,
    /// Toda a largura para DENTRO. A silhueta da forma não se move ao engrossar.
    Inner,
    /// Toda a largura para FORA. O miolo desenhado não é comido pelo contorno.
    Outer,
}

impl StrokeAlign {
    /// **Este alinhamento precisa de um interior para significar alguma coisa?**
    ///
    /// Porta única da pergunta *"posso oferecer/executar isto?"*: o painel a usa para decidir se
    /// pinta as opções, e a `ph2d-vec-boolean` para decidir se recorta. Duas cópias divergiriam no
    /// dia em que um quarto modo entrasse, e a divergência apareceria como um botão que desenha e
    /// não faz nada.
    #[must_use]
    pub fn needs_a_region(self) -> bool {
        !matches!(self, StrokeAlign::Centre)
    }
}

/// ⭐⭐⭐ **A ARTE QUE PERCORRE O CONTORNO** — o *Pattern Brush* (plano 36, W1).
///
/// # Porque é uma variante e não um modo do padrão
///
/// São **dois modelos**, e o estado da arte entrega os dois (plano 36 §1):
///
/// - o [`StrokePaint::Pattern`] é uma **TINTA** que o contorno REVELA. É **normativo em SVG 2**:
///   *"the stroke operation must be identical to converting the stroke geometry to a path and
///   filling it with the paint server"* ⇒ um tracejado são **buracos** no papel de parede, e a arte
///   não os conhece.
/// - isto é um **PINCEL**: a arte **percorre** o caminho, roda com a tangente e (como no
///   Illustrator) **reinicia em cada traço** do tracejado.
///
/// Um modo do padrão faria a mesma tinta responder às duas leis, e metade dos knobs de cada uma
/// ficaria morta na outra.
///
/// # ⛔ A arte é uma FORMA, e não um [`crate::PatternSource`]
///
/// O motor (`pattern_along`, plano 23) copia **GEOMETRIA**: ele recebe um `VecPath` e devolve
/// `Vec<VecPath>`. Um `PatternSource` também aceita **imagem**, e `Brush(Image(..))` seria estado
/// **gravável e indesenhável** — a mesma lei que recusou reusar o [`crate::Paint`] aqui.
///
/// # ⚠️ E ele ESCALA com a largura do traço, ao contrário do padrão
///
/// O plano 35 §2.3 fixou que uma TINTA **não** escala com a largura (*"a largura decide a faixa; o
/// padrão decide o que a preenche"*). Um pincel é o oposto, e é o que o Illustrator faz: **o pincel
/// É a faixa**, então a arte nasce com a altura da largura do traço, e o [`Self::scale`] multiplica
/// isso. *A mesma pergunta tem respostas contrárias nos dois modelos, e é por isso que são dois.*
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BrushStroke {
    /// A **forma** do documento que se repete ao longo do contorno. `None` = ainda não escolhida.
    ///
    /// ⭐⭐ **`Option`, e um gate achou porquê:** `VecPathId::default()` é um id **VÁLIDO** — a
    /// primeira forma de uma cena pode tê-lo. Com um id cru, *"sem arte"* e *"a arte é aquela
    /// forma"* seriam **os mesmos bytes**, e a porta que escreve a arte recusava-a em silêncio por
    /// «já é esse valor». É a lei que esta casa já pagou noutro sítio: *um zero de «não medido» e um
    /// de «perfeito» são o mesmo byte.*
    ///
    /// ⚠️ E `Some(id)` **não** garante que a forma existe — ela pode ter sido apagada. *"Tem arte?"*
    /// é uma pergunta à CENA; este campo só diz o que foi **autorado**.
    pub art: Option<crate::VecPathId>,
    /// A cor que a linha pinta enquanto a arte não resolve (apagada, ou um id que não existe).
    ///
    /// ⚠️ **Desenhar NADA seria pior** — uma linha invisível não se distingue de uma forma sem
    /// contorno. É a mesma lei da `fallback` do padrão.
    pub fallback: Rgba8,
    /// Multiplica a **largura do motivo** para dar o avanço por cópia: `1.0` encaixa borda-a-borda,
    /// `<1` sobrepõe, `>1` deixa vão. É o `spacing` do motor.
    pub spacing: f64,
    /// Desvio da guia ao longo da **normal**, positivo para a ESQUERDA do sentido de marcha — a
    /// convenção (e o sinal) do `dy` do texto em caminho.
    pub offset: f64,
    /// A arte do **outro lado** da curva, a percorrê-la ao contrário.
    pub flip: bool,
    /// **Orientação do motivo sobre a curva, em GRAUS**, dentro do referencial da cópia.
    ///
    /// ⚠️ A unidade está no NOME de propósito: um ângulo sem unidade declarada é o defeito que não
    /// dá erro em sítio nenhum, e o motor já paga essa lição.
    pub rotation_deg: f64,
    /// Multiplica a altura **DERIVADA da largura do traço**. `1.0` = a arte tem exactamente a
    /// altura da faixa.
    pub scale: f64,
}

impl Default for BrushStroke {
    fn default() -> Self {
        Self {
            art: None,
            fallback: Rgba8::new(0, 0, 0, 255),
            spacing: 1.0,
            offset: 0.0,
            flip: false,
            rotation_deg: 0.0,
            scale: 1.0,
        }
    }
}

/// ⭐⭐ **COM QUE TINTA um traço desenha** (plano 35) — a lista FECHADA, e não o [`crate::Paint`] do
/// preenchimento.
///
/// ⛔ **Reusar o `Paint` está RECUSADO por medição de risco, não por gosto:** ele representa
/// `Linear`/`Radial`/`MultiPoint`, e o renderer de traço **não os desenha**. Um modelo que
/// representa o que o desenho não faz produz um documento que se grava, recarrega e pinta errado —
/// *estado inalcançável, gravado*. Quando um gradiente no traço for pedido, este enum **ganha uma
/// variante** (append-only, um degrau da escada).
///
/// ⛔ E a saída barata — manter `color` e apendar um `pattern: Option<..>` ao lado — daria ao traço
/// **duas fontes de tinta que podem discordar**, e o sintoma seria a swatch a mostrar uma cor
/// enquanto a linha desenha outra coisa.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum StrokePaint {
    /// Uma cor chapada — o caminho de 99 % dos traços.
    Solid(Rgba8),
    /// ⭐ **Um PADRÃO de textura**, a mesma lei do preenchimento (plano 33).
    ///
    /// ⚠️ **`Box`, e o número decidiu-o:** em linha o `StrokeSpec` ia de `64` para `176` bytes e o
    /// `VecPath` engordava **54 %** — pago por **toda** forma em **toda** fotografia de undo,
    /// inclusive as que não têm padrão nenhum. Atrás do `Box` são `+4 %`. O preço é o `StrokeSpec`
    /// deixar de ser `Copy`, e o compilador contou esse preço em ~15–30 sítios mecânicos.
    Pattern(Box<crate::PatternFill>),
    /// ⭐⭐⭐ **Um PINCEL** — a arte PERCORRE o contorno (plano 36). Ver [`BrushStroke`] para os dois
    /// modelos e por que são dois.
    ///
    /// ⚠️ `Box` pela MESMA conta do padrão: em linha, toda forma da cena pagaria o tamanho em toda
    /// fotografia de undo, inclusive as que não têm pincel nenhum.
    Brush(Box<BrushStroke>),
}

impl StrokePaint {
    /// **A COR que representa esta tinta** — a sólida, ou a `fallback` do padrão.
    ///
    /// ⭐ É a porta que mantém honesta toda superfície que só sabe perguntar *"de que cor é este
    /// traço?"* — a swatch do painel, o token de cor, o `StrokeStyle` da shell. ⚠️ A `fallback` de um
    /// padrão **não é uma aproximação**: é literalmente a cor que ele pinta enquanto o ladrilho não
    /// resolve.
    #[must_use]
    pub fn color(&self) -> Rgba8 {
        match self {
            Self::Solid(c) => *c,
            Self::Pattern(p) => p.fallback,
            Self::Brush(b) => b.fallback,
        }
    }

    /// O **pincel** desta tinta (`None` se ela não é um).
    #[must_use]
    pub fn brush(&self) -> Option<&BrushStroke> {
        match self {
            Self::Brush(b) => Some(b),
            _ => None,
        }
    }
}

/// Estilo do traço de um path: tinta + largura (world-units) + ponta/junção +
/// tracejado opcional. Substitui a tupla `(Rgba8, f64)` da Fase 0.
///
/// ⚠️ **Deixou de ser `Copy` em 2026-08-27** (plano 35): a tinta pode ser um padrão, e ele vive
/// atrás de um `Box` porque em linha o `VecPath` engordaria 54 % (§0.1 do plano).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StrokeSpec {
    /// ⚠️ **Era `color: Rgba8`.** Quem só quer uma cor pergunta por [`Self::color`].
    pub paint: StrokePaint,
    pub width: f64,
    pub cap: LineCap,
    pub join: LineJoin,
    /// Tracejado como **múltiplos da largura**: `Some((dash, gap))` ⇒ traço de
    /// `dash·width` e vão de `gap·width`; `None` (ou `dash ≤ 0`) = contínuo.
    /// Width-aware: engrossar o traço alonga dash e vão na proporção, então a
    /// projeção da ponta nunca engole o vão.
    pub dash: Option<(f64, f64)>,
    /// **Ponta no COMEÇO** do caminho (arrowhead). Só vale em caminho aberto.
    ///
    /// ⚠️ **Corrigido em 2026-08-01 — a frase anterior era falsa.** Ela dizia que *"o postcard é
    /// posicional, então um save anterior a este campo segue legível e lê `Marker::None`"*: as duas
    /// metades não se seguem. Posicional é justamente o que **impede** a leitura — não há marca de
    /// ausência, o leitor chega ao fim dos bytes e falha (`Hit the end of buffer`, medido). O
    /// `#[serde(default)]` serve a formatos auto-descritivos e à construção em código; quem protege
    /// o arquivo é o `VEC_SCENE_SCHEMA_VERSION`, que recusa em vez de ler torto.
    #[serde(default)]
    pub marker_start: Marker,
    /// **Ponta no FIM** do caminho. Idem.
    #[serde(default)]
    pub marker_end: Marker,
    /// **Tamanho da ponta**, como múltiplo do tamanho que a largura do traço já dita.
    /// `1.0` = o default.
    ///
    /// Por que um MÚLTIPLO e não um tamanho absoluto: a ponta tem de crescer com o traço,
    /// senão engrossar a linha faz a seta encolher visualmente até virar um cotoco. O
    /// multiplicador dá o ajuste fino sem quebrar essa proporção.
    #[serde(default = "unit_scale")]
    pub marker_scale: f64,
    /// **Arredondamento das quinas da ponta**: `0` = afiada, `1` = o máximo que a geometria
    /// da ponta comporta sem se descaracterizar.
    #[serde(default)]
    pub marker_round: f64,
    /// **De que lado da linha a faixa cai** — ver [`StrokeAlign`]. Default `Centre`, que é o que
    /// todo desenho já feito tem.
    ///
    /// ⚠️ **Apendar aqui BUMPA o `VEC_SCENE_SCHEMA_VERSION`** (13→14), e o `#[serde(default)]` das
    /// linhas acima **não** salva um save anterior: o postcard é posicional e **não sinaliza
    /// ausência** — ele chega ao fim dos bytes e falha, em vez de cair no default. O atributo
    /// serve para construir o struct em código e para formatos auto-descritivos; a compatibilidade
    /// de leitura vem do NÚMERO, e é ele que recusa o arquivo velho em vez de o ler torto.
    #[serde(default)]
    pub align: StrokeAlign,
}

/// O default do [`StrokeSpec::marker_scale`]. Precisa ser uma função porque o default de
/// `serde` para um `f64` é **zero** — e uma ponta de tamanho zero é uma ponta invisível.
fn unit_scale() -> f64 {
    1.0
}

impl StrokeSpec {
    /// Traço sólido, ponta/junção default (Butt/Miter), sem tracejado, sem setas.
    #[must_use]
    pub fn new(color: Rgba8, width: f64) -> Self {
        Self {
            paint: StrokePaint::Solid(color),
            width,
            cap: LineCap::Butt,
            join: LineJoin::Miter,
            dash: None,
            marker_start: Marker::None,
            marker_end: Marker::None,
            marker_scale: 1.0,
            marker_round: 0.0,
            align: StrokeAlign::Centre,
        }
    }

    /// **A COR que representa este traço** — a sólida, ou a `fallback` do padrão.
    ///
    /// ⭐ Ela existe para que a troca de `color: Rgba8` por [`StrokePaint`] custe **um caractere**
    /// em cada leitor que só quer uma cor (`s.color` -> `s.color()`), em vez de um `match` espalhado
    /// por quinze ficheiros. *Uma pergunta que quinze sítios fazem tem de ter uma porta.*
    #[must_use]
    pub fn color(&self) -> Rgba8 {
        self.paint.color()
    }

    /// O padrão desta tinta, se ela for um. `None` num traço sólido.
    #[must_use]
    pub fn pattern(&self) -> Option<&crate::PatternFill> {
        match &self.paint {
            StrokePaint::Solid(_) | StrokePaint::Brush(_) => None,
            StrokePaint::Pattern(p) => Some(p),
        }
    }

    /// O **pincel** desta tinta, se ela for um (plano 36). `None` num traço sólido ou de padrão.
    #[must_use]
    pub fn brush(&self) -> Option<&BrushStroke> {
        self.paint.brush()
    }

    /// **Esta faixa precisa ser RECORTADA contra o interior da forma?**
    ///
    /// Porta única do *"tem alinhamento a executar aqui?"*: a booleana pergunta antes de montar a
    /// banda dupla, o desenho pergunta antes de trocar o traço por um preenchimento, e o painel
    /// pergunta antes de anunciar o modo. Um traço de largura zero responde `false` pelo mesmo
    /// motivo que [`Self::lays_a_band`] existe — recortar o nada é trabalho sobre o vazio.
    #[must_use]
    pub fn is_aligned(&self) -> bool {
        self.align.needs_a_region() && self.lays_a_band()
    }

    /// `true` se o traço tem alguma ponta — o render então precisa recuar a linha.
    #[must_use]
    pub fn has_markers(&self) -> bool {
        self.marker_start != Marker::None || self.marker_end != Marker::None
    }

    /// **Este traço deita uma FAIXA de tinta?** `false` em largura zero — e `0` significa *sem
    /// traço*, não *o traço mais fino que der* (a lei que o slider de Width promete desde
    /// 2026-07-16; ver `stroke_zero_tests` na `ph2d-vec-render`).
    ///
    /// ⚠️ **A pergunta é sobre a FAIXA, não sobre a tinta toda.** Uma ponta (seta, losango) é
    /// desenhada à parte, e o tamanho dela também escala com a largura — mas quem responde por ela
    /// é o [`crate::stroke_plan`], não isto. Um caminho FECHADO nunca tem pontas (não há extremo
    /// onde pô-las), e é por isso que quem pergunta pela SILHUETA de uma forma preenchida pode
    /// parar aqui.
    ///
    /// Existe porque a pergunta tinha **três** respostas espalhadas: o desenho confiava no Vello
    /// (que não encoda nada com largura 0), a booleana devolvia tinta vazia, e o campo de distância
    /// perguntava `stroke.is_some()` — que continua **verdadeiro** com largura zero. A terceira
    /// discordava das outras duas, e o preço foi o pente do bevel voltar numa forma sem contorno
    /// nenhum (BUGS #24, 2ª rodada).
    #[must_use]
    pub fn lays_a_band(&self) -> bool {
        self.width > 0.0
    }

    /// **O tracejado em COMPRIMENTO** — `[traço, vão]` no mesmo espaço da geometria, ou
    /// `None` para linha contínua.
    ///
    /// O campo [`Self::dash`] guarda MÚLTIPLOS da largura (engrossar o traço alonga os dois
    /// na proporção, então a projeção da ponta nunca engole o vão); quem desenha ou assa
    /// precisa dos comprimentos. A conversão é UMA — o renderer e o Outline Stroke falam com
    /// versões diferentes da kurbo e cada um constrói o próprio `Stroke`, mas os dois têm de
    /// concordar sobre **quanto mede um traço**, senão a forma assada sai com o tracejado
    /// noutra cadência que a desenhada.
    ///
    /// O vão é afastado do zero: um vão de comprimento nulo é um elemento degenerado para a
    /// kurbo, e o que o usuário quis dizer com ele é "sólido".
    #[must_use]
    pub fn dash_lengths(&self) -> Option<[f64; 2]> {
        match self.dash {
            Some((d, g)) if d > 0.0 => Some([d * self.width, (g * self.width).max(f64::EPSILON)]),
            _ => None,
        }
    }
}

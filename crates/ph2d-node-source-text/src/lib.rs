#![forbid(unsafe_code)]
//! `source.text` — **texto como geometria vetorial viva, UM GLIFO POR INSTÂNCIA**
//! (doc 89, folha 14 §3 item 1, o P0 mais caro da conferência inteira).
//!
//! A folha SOURCE mediu a ausência assim: *"não há nó de texto, e embora
//! `ph2d-vector-font` exista como foundational, **nada no grafo o alcança**"* —
//! e fechou com *"metade do mograph do mundo é texto animado por caractere"*.
//!
//! ## O achado que decidiu o desenho: a geometria já estava paga
//!
//! O módulo Vector construiu texto vetorial inteiro (glifo → `VecPath`, eixos
//! variáveis, refluxo, texto em caminho). Este nó **não traz geometria nova**: é
//! a MESMA fiação que a wave do `source.shape` descreveu no seu próprio doc —
//! *"35 formas que o editor já desenhava eram inalcançáveis de um grafo: a
//! geometria nunca faltou, a fiação faltava"* —, uma família adiante.
//!
//! ## Por que UM GLIFO POR INSTÂNCIA, e não um bloco só
//!
//! ⚠️ **É a wave inteira numa linha.** Um bloco de texto como UMA instância
//! desenharia a mesma imagem e não seria a feature: o que a referência entrega
//! (Cavalry *Sub-Mesh behaviour* · AE *Text Animators* · Blender *String to
//! Curves*, que devolve **uma instância de curva por caractere**) é a letra como
//! ELEMENTO. Emitindo uma linha por glifo, **toda a biblioteca `motion.*` passa a
//! agir por caractere de graça** — stagger, wave, jitter, campos, `drive`,
//! `falloff`, `duplicator` — sem um único nó novo a jusante.
//!
//! ## O pivô, e a propriedade que o torna invisível em repouso
//!
//! Uma letra que gira tem de girar **em torno de si mesma**. O pivô é o meio do
//! avanço, sobre a baseline ([`Pivot::Center`], o default) — um número que o laço
//! de layout já tem na mão, e que é uniforme na vertical para a palavra rodar como
//! palavra em vez de esfarelar. [`Pivot::Pen`] é a origem crua do glifo, para o
//! efeito de máquina de escrever (a letra cresce a partir da esquerda).
//!
//! ⚠️ **Os dois desenham a MESMA imagem em repouso** — a geometria anda `−c` e o
//! `P` anda `+c`, então em tamanho unitário e rotação zero a soma é a mesma. É o
//! que faz a escolha do default ser invisível até alguém animar, e é um gate.
//!
//! ## A porta é a do `source.shape`, e ela não é escolha
//!
//! Um nó recebe params, entradas e o playhead — nada mais (a propriedade que deixa
//! o cook memoizar e reproduzir bit-exato), logo **não alcança a biblioteca de
//! vetor nem a fonte**. O shell faz o layout, interna cada glifo no `VecPathStore`
//! e publica um stream de N linhas `(P, geometry_id)` no canal externo sob a
//! **chave de conteúdo** ([`text_key`]); este nó lê essa chave. Chave sem
//! publicação (um cook adiantado) é externo vazio ⇒ stream vazio: nada desenhado,
//! nunca um pânico.

use ph2d_node_registry::{
    NodeRegistry, ParamUiHint, ParamUnit, ParamUnitDecl, ParamWidget, RegistryError,
};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// **O texto e a fonte são TEXT PARAMS** (doc 32), não `ParamSpec`s: eles moram no
/// `Graph`, ao lado do manifesto e nunca dentro dele, que é o que deixa um param
/// não-`f32` existir sem tocar o `NodeManifest` congelado (§6).
pub const TEXT_KEY: &str = "text";
/// A família de fonte. Vazio ⇒ a fonte embutida — e o vazio **não é um fallback de
/// emergência**: é o caminho que não varre as fontes do sistema (50–200 ms).
pub const FONT_KEY: &str = "font";

/// O texto que um nó recém-SOLTO carrega. ⚠️ **Um nó que nasce mudo lê como
/// quebrado**: a 4ª condição de UI é *a sequência leva a algum lugar*, e o gesto
/// aqui é soltar o nó. É o default do *String to Curves* do Blender, pela mesma
/// razão.
///
/// ⚠️ **Ele é SEMEADO pelo editor, nunca resolvido no `eval`** — o nó declara,
/// `register_text_defaults` publica, e o gesto de soltar escreve o override. Um
/// fallback aqui dentro faria o canvas desenhar "Text" enquanto o painel — que lê
/// o override e não acha nenhum — mostra o campo VAZIO: duas respostas à mesma
/// pergunta, visíveis lado a lado no primeiro quadro.
pub const DEFAULT_TEXT: &str = "Text";

/// Os params `f32`. ⚠️ **Esta lista é a CHAVE** — [`text_key`] varre-a, então um
/// param acrescentado aqui entra na chave e no manifesto de uma vez. Uma chave que
/// enumera as entradas de um valor é como a próxima é esquecida, e o sintoma é o
/// controle **inerte depois da primeira vez** (o defeito do *Pattern Offset*,
/// 2026-08-09, e o que a `shape_key` documenta ao lado da dela).
pub mod param {
    pub const SIZE: &str = "size";
    pub const TRACKING: &str = "tracking";
    pub const LINE_HEIGHT: &str = "line_height";
    pub const ALIGN: &str = "align";
    pub const WEIGHT: &str = "weight";
    pub const PIVOT: &str = "pivot";

    /// Toda a lista, na ordem da chave.
    pub const ALL: &[&str] = &[SIZE, TRACKING, LINE_HEIGHT, ALIGN, WEIGHT, PIVOT];
}

/// Onde o referencial de cada letra assenta.
///
/// ⚠️ Os dois produzem a **mesma imagem em repouso** (ver o doc do módulo); a
/// diferença só existe quando alguma coisa a jusante gira, escala ou empurra a
/// letra — que é precisamente para o que este nó existe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pivot {
    /// A origem crua do glifo: canto esquerdo, sobre a baseline. O ponto onde um
    /// caret assenta. A letra cresce/gira a partir da esquerda.
    Pen,
    /// O meio do AVANÇO, sobre a baseline — o default. Horizontalmente centrado
    /// em si mesma, verticalmente ancorada na linha, que é o que faz uma palavra
    /// inteira rodar como palavra.
    Center,
}

impl Pivot {
    /// O índice que o param guarda ⇒ **APPEND ONLY** (um grafo salvo guarda o
    /// número, não o nome).
    #[must_use]
    pub fn from_index(v: f32) -> Pivot {
        match v.round() as i32 {
            0 => Pivot::Pen,
            _ => Pivot::Center,
        }
    }
}

/// Como as linhas se alinham entre si. Índice guardado ⇒ **APPEND ONLY**.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Align {
    Left,
    Center,
    Right,
}

impl Align {
    #[must_use]
    pub fn from_index(v: f32) -> Align {
        match v.round() as i32 {
            1 => Align::Center,
            2 => Align::Right,
            _ => Align::Left,
        }
    }
}

/// O descritor do bloco — tudo o que decide **onde cada letra cai**, menos as duas
/// strings.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextParams {
    pub size: f32,
    pub tracking: f32,
    pub line_height: f32,
    pub align: Align,
    pub weight: f32,
    pub pivot: Pivot,
}

impl TextParams {
    /// Lê o descritor de QUALQUER fonte de params — o nó passa `|n| ctx.param(n)`,
    /// o shell passa um closure sobre os overrides do grafo mais os defaults do
    /// manifesto. **Um leitor, dois chamadores** ⇒ a chave do nó e a do shell são
    /// os mesmos bits pelo mesmo código.
    #[must_use]
    pub fn read(get: impl Fn(&str) -> f32) -> TextParams {
        TextParams {
            size: get(param::SIZE),
            tracking: get(param::TRACKING),
            line_height: get(param::LINE_HEIGHT),
            align: Align::from_index(get(param::ALIGN)),
            weight: get(param::WEIGHT),
            pivot: Pivot::from_index(get(param::PIVOT)),
        }
    }
}

/// A string que este nó desenha, dado o override que o grafo guarda (ou a falta
/// dele). **Porta única** — o nó e o shell perguntam aqui, senão um deles
/// desenharia uma coisa enquanto o outro mintava a chave de outra.
///
/// ⚠️ **Ausente é VAZIO**, e não [`DEFAULT_TEXT`]: o texto de fábrica é escrito
/// no grafo pelo editor ao soltar o nó, então quando ele existe **está no
/// override** e o painel o vê. Um nó montado por `add_node` num gate ou numa
/// demo não passou por gesto nenhum e desenha nada — o que é honesto.
#[must_use]
pub fn text_of(override_: Option<&str>) -> &str {
    override_.unwrap_or("")
}

/// A família de fonte, `""` para a embutida. Porta única, pela mesma razão.
#[must_use]
pub fn font_of(override_: Option<&str>) -> &str {
    override_.unwrap_or("")
}

/// **A chave de conteúdo** — o nome do externo que o nó lê e o shell publica.
/// Endereçada por conteúdo: dois nós com o mesmo bloco partilham o mesmo trabalho.
///
/// ⚠️ **A fonte leva o comprimento à frente e o texto vem por ÚLTIMO**, e não é
/// arrumação: sem isso `font="a:b", text="c"` e `font="a", text="b:c"` mintam a
/// MESMA chave — dois blocos diferentes a partilhar uma publicação, que é um
/// desenho errado que ninguém consegue reproduzir de propósito.
#[must_use]
pub fn text_key(get: impl Fn(&str) -> f32, font: &str, text: &str) -> String {
    // ⛔⛔⛔ **O `$` NÃO É DECORAÇÃO — ele é a cerca que mantém isto FORA do selector.**
    //
    // Auditoria de seis lentes, doc 96 §5.5. Esta chave é publicada na MESMA tabela de externos
    // de que o picker de objectos tira as opções (`source_options`), e o filtro dele é
    // `!is_reserved(k)`. Sem o prefixo, cada planta/forma/texto/tabela derivada aparecia ao
    // artista como uma *"Drawn shape"* escolhível — na cena `=108` são **cinco chips de lixo**,
    // com a gramática crua lá dentro, e clicar num planta a PRÓPRIA planta como folha dela.
    //
    // ⚠️ O doc do `RESERVED_PREFIX` já dizia as duas metades: *«o editor publica DENTRO do
    // namespace, e recusa publicar um nome do artista que já esteja nele»*. A primeira metade é
    // que não estava a ser cumprida por quem cunha chaves de CONTEÚDO.
    //
    // ⚠️ **Mudar o prefixo é seguro porque a chave é opaca:** quem a cunha e quem a lê chamam
    // esta mesma função, e ela não é persistida em lado nenhum (é derivada a cada quadro).
    let mut k = String::from("$text");
    for name in param::ALL {
        k.push(':');
        k.push_str(&get(name).to_bits().to_string());
    }
    k.push(':');
    k.push_str(&font.len().to_string());
    k.push(':');
    k.push_str(font);
    k.push(':');
    k.push_str(text);
    k
}

/// O contrato estático (ADR-0031). ⚠️ `NodeManifest` congelado (8 campos)
/// **intocado** — o texto e a fonte são text params, que moram no `Graph`.
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("source.text"),
    name: "source.text",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: param::SIZE,
            default: 1.0,
        },
        ParamSpec {
            name: param::TRACKING,
            default: 0.0,
        },
        ParamSpec {
            name: param::LINE_HEIGHT,
            default: 1.2,
        },
        ParamSpec {
            name: param::ALIGN,
            default: 0.0,
        },
        ParamSpec {
            name: param::WEIGHT,
            default: 400.0,
        },
        // O default é `Center` (índice 1) porque a razão de este nó existir é a
        // animação por caractere, e é o pivô sob o qual ela lê certo. Em repouso
        // os dois desenham o mesmo (gate), então o default não muda imagem
        // nenhuma — só o que acontece quando alguém a anima.
        ParamSpec {
            name: param::PIVOT,
            default: 1.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

struct SourceText;

impl NodeOp for SourceText {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        // ⚠️ O nó lê o texto e a fonte pelas MESMAS portas que o shell — e a chave
        // pela mesma função —, que é o que faz as duas pontas serem os mesmos bits.
        let text = text_of(ctx.text_param(TEXT_KEY)).to_string();
        let font = font_of(ctx.text_param(FONT_KEY)).to_string();
        let key = text_key(|n| ctx.param(n), &font, &text);
        // O shell montou o bloco, internou um `VecPath` por glifo e publicou um
        // stream de N linhas `(P, geometry_id)` sob esta chave. O clone é
        // refcount (colunas `Arc`); chave sem publicação = externo vazio.
        let stream = ctx.external(&key).clone();
        ctx.emit(stream);
    }
}

/// Registra o nó no registry de runtime. Chamado (via codegen) do
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(SourceText))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Text",
            category: ph2d_node_registry::NodeUiCategory::Source,
            silhouette: ph2d_node_registry::NodeSilhouette::TrapezoidDown,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    // O texto de fábrica: o nó DECLARA, o editor APLICA ao soltar.
    reg.register_text_defaults(MANIFEST.id, FACTORY_TEXT);
    // ⚠️ **Sem isto as letras saem como quadrados brancos.** A saída carrega
    // `geometry_id`, que o cook residente na GPU não tem rota para desenhar (o
    // lowering do device crava `texture_id`), então um documento que traz um
    // deles recusa o cook GPU — a lei do ADR-0154/0155, e o irmão exato do
    // `source.shape`.
    reg.register_live_vector_source(MANIFEST.id);
    Ok(())
}

/// **O que cada número É** (doc 88, wave A) — nunca como é mostrado. Só o tamanho
/// é uma distância de mundo; `tracking` e `line_height` são FRAÇÕES do tamanho (um
/// tracking em unidades de mundo deixaria de acompanhar o corpo da letra, que é o
/// oposto do que a tipografia faz), e o peso é a coordenada de um eixo de fonte.
static PARAM_UNITS: &[ParamUnitDecl] = &[ParamUnitDecl {
    param: param::SIZE,
    unit: ParamUnit::Length,
}];

/// O que um `source.text` recém-solto já carrega. A fonte fica FORA: vazio já é
/// a embutida, e semear um nome de família seria escolher uma tipografia pelo
/// artista.
static FACTORY_TEXT: &[(&str, &str)] = &[(TEXT_KEY, DEFAULT_TEXT)];

static ALIGN_LABELS: &[&str] = &["Left", "Center", "Right"];
static PIVOT_LABELS: &[&str] = &["Pen", "Center"];

static PARAM_HINTS: &[ParamUiHint] = &[
    // As duas strings primeiro: é o que o artista muda antes de tudo.
    ParamUiHint {
        param: TEXT_KEY,
        label: "Text",
        min: 0.0,
        max: 0.0,
        step: 0.0,
        widget: ParamWidget::Text,
    },
    ParamUiHint {
        param: FONT_KEY,
        label: "Font",
        min: 0.0,
        max: 0.0,
        step: 0.0,
        widget: ParamWidget::Text,
    },
    ParamUiHint {
        param: param::SIZE,
        label: "Size",
        min: 0.05,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::TRACKING,
        label: "Tracking",
        min: -0.3,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::LINE_HEIGHT,
        label: "Line Height",
        min: 0.5,
        max: 3.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::ALIGN,
        label: "Align",
        min: 0.0,
        max: 0.0,
        step: 0.0,
        widget: ParamWidget::Enum {
            labels: ALIGN_LABELS,
        },
    },
    // A faixa é a do eixo `wght` do OpenType (100..900). Uma fonte que não o
    // exponha ignora o valor — o eixo é pedido por TAG, e uma tag ausente não
    // move o contorno.
    ParamUiHint {
        param: param::WEIGHT,
        label: "Weight",
        min: 100.0,
        max: 900.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::PIVOT,
        label: "Pivot",
        min: 0.0,
        max: 0.0,
        step: 0.0,
        widget: ParamWidget::Enum {
            labels: PIVOT_LABELS,
        },
    },
];

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

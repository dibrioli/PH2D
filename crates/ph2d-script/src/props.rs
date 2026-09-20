//! ⭐⭐⭐ **OS NÚMEROS DE UM SCRIPT, POR OBJECTO** — a lei pura do TOP-20 #16 (`ScriptProperties`).
//!
//! Um script declara no topo os números que oferece (`ph2d.property("speed", 2)`); cada objecto que
//! o carrega guarda **só os que o artista PÔS**; e esta função diz, para cada declaração, que valor
//! o objecto usa e **de onde** ele vem. Plano: `docs/Components/13_plano_script_properties.md`.
//!
//! # A lei é a do ORÁCULO, menos três perdas silenciosas
//!
//! O Godot 4.7.2 (MIT) foi **corrido** sobre cenas nossas
//! (`docs/Components/ferramentas/godot_export_probe.gd`, controlo C0 verde). Portado dele:
//!
//! - **Q1** um objecto sem valor próprio **segue** o default quando o script muda;
//! - **Q2** um objecto com valor próprio **guarda-o**;
//! - **Q6** a faixa é **pista de edição** — o valor gravado fora dela é lido como está;
//! - **Q7** a ordem é **a da declaração**;
//! - **Q9** duas instâncias são **independentes**;
//! - **Q10** com o script **desconhecido** (ficheiro sumido) os valores **ficam**.
//!
//! ⛔ **E três divergências DECLARADAS**, cada uma contra uma perda que o alvo comete em silêncio:
//!
//! - **D1 (Q3)** — no alvo, *«próprio»* é *«difere do default no instante de gravar»*: um `4`
//!   escrito à mão igual ao default antigo passa a `7` quando o default muda. ⇒ aqui *próprio* é o
//!   que o artista **PÔS** (a lei da casa: *o discriminador é quem pôs, nunca um limiar*), e o
//!   [`forget`] é a única porta que o larga.
//! - **D2 (Q4b)** — no alvo, um valor cuja propriedade **saiu** do script perde-se na gravação
//!   seguinte, e **volta ao default** quando a propriedade regressa (renomear e desfazer o nome
//!   apaga o trabalho). ⇒ aqui ele fica, **nomeado** como órfão ([`OrphanWhy::Missing`]).
//! - **D3 (Q5b)** — no alvo, `"rapido"` numa propriedade que passou a número lê-se **`0`**. ⇒ aqui
//!   um valor do tipo errado **não se aplica nem se converte** ([`OrphanWhy::WrongKind`]).
//!
//! # ⚠️ «Não sei o que o script declara» NÃO é «o script não declara nada»
//!
//! Com o ficheiro sumido ou com um erro de sintaxe, as declarações são **desconhecidas** — e tratar
//! isso como uma lista vazia faria **todo** valor próprio virar órfão, com um botão *Remove* ao lado
//! de cada um. É o convite exacto à perda que o D2 existe para impedir (o alvo apaga tudo ao gravar
//! nesse instante). ⇒ [`resolve`] recebe um `Option`, e com `None` os valores vão para
//! [`Resolution::kept`], que o painel mostra **sem** verbo de apagar.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// **Um valor que um script oferece ao painel.** O Luau tem um tipo numérico só, então o par
/// int/float do alvo (Q8) não tem onde acontecer aqui.
///
/// ⚠️ **A ORDEM das variantes é o fio** (postcard é posicional): acrescentar no fim, nunca no meio.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ScriptValue {
    /// Um `number` do Luau.
    Number(f64),
    /// Um `boolean`.
    Bool(bool),
    /// Uma `string`.
    Text(String),
    /// ⭐⭐⭐ **Uma POSIÇÃO** — `ph2d.vec2(x, y)`. Uma fileira com dois campos, em vez de duas
    /// propriedades que o artista tem de lembrar-se de manter juntas.
    Vec2([f64; 2]),
    /// ⭐⭐⭐ **Uma COR** — `ph2d.color(r, g, b, a?)`, cada canal em `0..=1`. Uma amostra que abre
    /// o selector da casa, em vez de três ou quatro campos numéricos.
    ///
    /// ⚠️ **O domínio `0..=1` é conferido na DECLARAÇÃO e NÃO no valor gravado**, e a assimetria
    /// com o enum tem mecanismo: um valor fora da lista de um enum existe porque o artista o pode
    /// **ESCREVER** (o campo é livre); um canal fora de `0..=1` não, porque a única superfície que
    /// escreve uma cor é o selector, que é limitado por construção. *Inventar uma quarta razão de
    /// orfandade para um estado que nada produz é construir para um fantasma.*
    Color([f64; 4]),
}

/// **De que tipo um valor é** — a pergunta do D3, com uma resposta só.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScriptValueKind {
    /// Um número.
    Number,
    /// Um sim/não.
    Bool,
    /// Um texto.
    Text,
    /// Uma posição.
    Vec2,
    /// Uma cor.
    Color,
}

impl ScriptValue {
    /// O tipo deste valor.
    #[must_use]
    pub fn kind(&self) -> ScriptValueKind {
        match self {
            Self::Number(_) => ScriptValueKind::Number,
            Self::Bool(_) => ScriptValueKind::Bool,
            Self::Text(_) => ScriptValueKind::Text,
            Self::Vec2(_) => ScriptValueKind::Vec2,
            Self::Color(_) => ScriptValueKind::Color,
        }
    }

    /// ⭐⭐ **Os NÚMEROS que este valor carrega** — a porta única da conferência de finitude.
    ///
    /// ⚠️ **Ela existe porque a lei era escrita à mão para UM tipo:** o `check_decl` tinha
    /// `if let ScriptValue::Number(v) = … && !v.is_finite()`, e um tipo novo com números lá dentro
    /// passaria por ele **calado** — um `nan` numa componente chega ao script e o painel pinta
    /// `NaN`. *Uma conferência indexada pela variante esquece a variante seguinte.*
    #[must_use]
    pub fn componentes(&self) -> &[f64] {
        match self {
            Self::Number(n) => std::slice::from_ref(n),
            Self::Bool(_) | Self::Text(_) => &[],
            Self::Vec2(v) => v.as_slice(),
            Self::Color(c) => c.as_slice(),
        }
    }
}

impl ScriptValueKind {
    /// O nome que o painel e as mensagens de erro usam — o da **linguagem**, que é o que o
    /// artista escreveu.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Number => "number",
            Self::Bool => "boolean",
            Self::Text => "string",
            // ⭐ O nome é o do CONSTRUTOR que o artista escreveu (`ph2d.vec2`), pela mesma lei das
            // três de cima: a mensagem devolve-lhe a palavra do ficheiro dele.
            Self::Vec2 => "vec2",
            Self::Color => "color",
        }
    }
}

/// **As pistas de edição de um número** (`{ min = 0, max = 10, step = 0.5 }`).
///
/// ⚠️ **Pistas, não leis** (Q6): elas mandam no que o painel deixa escrever e arrastar; o valor
/// gravado fora delas é lido como está. Um script que estreite a faixa não reescreve o documento.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PropHint {
    /// O menor valor que o painel aceita.
    pub min: Option<f64>,
    /// O maior valor que o painel aceita.
    pub max: Option<f64>,
    /// O passo de arrasto.
    pub step: Option<f64>,
    /// ⭐⭐⭐ **AS OPÇÕES de um texto** — `ph2d.property("mode", "fast", {options = {…}})`.
    ///
    /// Vazia = texto livre, que é o de sempre **ao bit**. Com opções, o painel pinta um chip e a
    /// lei abaixo deixa de aceitar o que não está nelas.
    ///
    /// ⚠️ **O enum NÃO é uma variante do [`ScriptValue`]**, e a medição é que o decidiu: ele é um
    /// `Text` com a lista na PISTA. Uma variante nova custaria um degrau no fio (o postcard é
    /// posicional) e não compraria nada — *o valor de um enum é o texto que o script compara*.
    pub options: Vec<String>,
}

/// **Uma declaração** — `ph2d.property(name, default, hint)` no topo de um script.
#[derive(Clone, Debug, PartialEq)]
pub struct PropDecl {
    /// O nome, que é também a chave em `self`.
    pub name: String,
    /// O valor de quem não pôs nenhum — e o TIPO da propriedade.
    pub default: ScriptValue,
    /// As pistas de edição (só números as têm).
    pub hint: PropHint,
}

/// **De onde vem o valor que o objecto usa.**
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Origin {
    /// Do script — o objecto não pôs nenhum.
    Default,
    /// Do objecto — o artista pôs este (D1: mesmo que seja igual ao default).
    Own,
}

/// Uma propriedade resolvida, na ordem da declaração.
#[derive(Clone, Debug, PartialEq)]
pub struct Resolved {
    /// O nome declarado.
    pub name: String,
    /// O valor que o objecto usa.
    pub value: ScriptValue,
    /// De onde ele veio — a cor da linha e o botão *Revert*.
    pub origin: Origin,
    /// As pistas da declaração.
    pub hint: PropHint,
}

/// **Porque um valor próprio não tem onde ser aplicado.**
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrphanWhy {
    /// O script já não declara este nome (D2).
    Missing,
    /// O script declara-o com OUTRO tipo (D3) — o objecto lê o default.
    WrongKind {
        /// O tipo que o script declara agora.
        declared: ScriptValueKind,
    },
    /// ⭐⭐⭐ **O script declara-o com uma LISTA, e este valor não está nela** — o objecto lê o
    /// default, e o artista vê o valor gravado com um *Remove* ao lado.
    ///
    /// ⚠️ **É a MESMA lei do [`Self::WrongKind`] uma casa abaixo**, e a razão é a mesma: um valor
    /// que a declaração não admite **não se aplica nem se converte**. ⛔ A alternativa — encostá-lo
    /// à opção mais parecida, ou à primeira — é o *«aceita e mente»* que esta casa já pagou três
    /// vezes; e deixá-lo passar é o que ele fazia até hoje, com o script a comparar `"fst"` com
    /// `"fast"` e a cair no ramo errado **em silêncio**.
    NotAnOption,
}

/// Um valor próprio sem onde ser aplicado.
#[derive(Clone, Debug, PartialEq)]
pub struct Orphan {
    /// O nome gravado.
    pub name: String,
    /// O valor gravado — intacto.
    pub value: ScriptValue,
    /// Porquê.
    pub why: OrphanWhy,
}

/// **O que um objecto usa, e o que ficou de fora.**
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Resolution {
    /// Uma por declaração, na ordem da declaração (Q7).
    pub values: Vec<Resolved>,
    /// Os valores próprios sem onde ser aplicados, por ordem de NOME (a do mapa gravado).
    pub orphans: Vec<Orphan>,
    /// ⚠️ **Com as declarações DESCONHECIDAS, todo valor próprio vem para aqui** — ver o cabeçalho
    /// do módulo. Por ordem de nome.
    pub kept: Vec<(String, ScriptValue)>,
}

/// ⭐⭐⭐ **A lei.** `decls = None` quer dizer *«não sei o que o script declara»* (ficheiro sumido,
/// erro de sintaxe) — e é diferente de `Some(&[])`.
#[must_use]
pub fn resolve(decls: Option<&[PropDecl]>, own: &BTreeMap<String, ScriptValue>) -> Resolution {
    let Some(decls) = decls else {
        return Resolution {
            kept: own.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
            ..Resolution::default()
        };
    };
    let mut out = Resolution::default();
    for d in decls {
        let (value, origin) = match own.get(&d.name) {
            Some(v) if v.kind() == d.default.kind() && aceita(d, v) => (v.clone(), Origin::Own),
            // D3: o do tipo errado — e o de fora da LISTA — não se aplicam: o objecto lê o default.
            _ => (d.default.clone(), Origin::Default),
        };
        out.values.push(Resolved {
            name: d.name.clone(),
            value,
            origin,
            hint: d.hint.clone(),
        });
    }
    for (name, value) in own {
        let why = match decls.iter().find(|d| d.name == *name) {
            None => OrphanWhy::Missing,
            Some(d) if d.default.kind() != value.kind() => OrphanWhy::WrongKind {
                declared: d.default.kind(),
            },
            // ⚠️ **Depois do tipo, nunca antes:** um número gravado numa propriedade que passou a
            // enum é `WrongKind`, que diz ao artista a coisa mais útil das duas.
            Some(d) if !aceita(d, value) => OrphanWhy::NotAnOption,
            Some(_) => continue,
        };
        out.orphans.push(Orphan {
            name: name.clone(),
            value: value.clone(),
            why,
        });
    }
    out
}

/// **A declaração admite este valor?** — a porta, lida pelas DUAS metades do [`resolve`].
///
/// ⚠️ **Escrita uma vez de propósito:** as duas perguntas (*«aplica-se?»* e *«é órfão?»*) são a
/// mesma, e duas cópias divergiriam no dia em que uma delas ganhasse um caso — que é como um valor
/// passaria a ser recusado **e** a não ser nomeado, ou o contrário.
///
/// ⭐ Sem opções ela é `true` **sempre**: o caminho de omissão é o de sempre, ao bit.
fn aceita(d: &PropDecl, v: &ScriptValue) -> bool {
    match v {
        ScriptValue::Text(t) if !d.hint.options.is_empty() => d.hint.options.iter().any(|o| o == t),
        _ => true,
    }
}

/// **O artista PÔS este valor** (D1: fica próprio mesmo igual ao default).
pub fn put(own: &mut BTreeMap<String, ScriptValue>, name: &str, value: ScriptValue) {
    own.insert(name.to_owned(), value);
}

/// **Larga o valor próprio** — o *Revert* de uma linha e o *Remove* de um órfão são esta porta.
/// Devolve se havia alguma coisa a largar.
pub fn forget(own: &mut BTreeMap<String, ScriptValue>, name: &str) -> bool {
    own.remove(name).is_some()
}

/// **Porque uma declaração é recusada** — a mensagem que o painel mostra.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeclError {
    /// O nome não é um identificador (`[A-Za-z_][A-Za-z0-9_]*`).
    BadName(String),
    /// O nome é da casa (`id`).
    Reserved(String),
    /// Declarado duas vezes.
    Twice(String),
    /// O default não é número finito, sim/não, texto, `vec2` nem `color`.
    BadDefault(String),
    /// ⭐ Um canal de cor fora de `0..=1` — ver a recusa no [`check_decl`].
    ColorOutOfRange(String),
    /// Uma opção que a casa não conhece (`mni` em vez de `min`).
    UnknownOption {
        /// A propriedade.
        name: String,
        /// A opção desconhecida.
        option: String,
    },
    /// `min`/`max`/`step` num valor que não é número, ou `step <= 0`, ou `min > max`.
    BadHint(String),
    /// `ph2d.property` chamada fora do topo do script.
    NotAtTop(String),
    /// Mais declarações do que o painel sabe pintar ([`PROPS_MAX`]).
    TooMany(String),
}

impl std::fmt::Display for DeclError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadName(n) => write!(f, "'{n}' is not a valid property name"),
            Self::Reserved(n) => write!(f, "'{n}' is reserved (self.{n} belongs to the engine)"),
            Self::Twice(n) => write!(f, "property '{n}' is declared twice"),
            Self::BadDefault(n) => {
                write!(
                    f,
                    "property '{n}' needs a number, boolean, string, ph2d.vec2 or ph2d.color \
                     default"
                )
            }
            // ⭐ A frase diz a SAÍDA e não só a recusa: quem quer um valor fora de `0..=1` tem a
            // composição de três números, que é a que sempre existiu.
            Self::ColorOutOfRange(n) => write!(
                f,
                "property '{n}': every ph2d.color channel must be between 0 and 1 (for values \
                 outside it, declare plain numbers)"
            ),
            // ⚠️ A lista de chaves é a que o `read_decl` de facto aceita — ela ficou a dizer três
            // no dia em que o enum trouxe a quarta, e uma mensagem que nomeia menos do que o
            // parser aceita manda o artista apagar uma chave que funciona.
            Self::UnknownOption { name, option } => write!(
                f,
                "property '{name}': unknown option '{option}' (min, max, step, options)"
            ),
            Self::BadHint(n) => write!(
                f,
                "property '{n}': min/max/step need a number or ph2d.vec2 default, step > 0 and \
                 min <= max"
            ),
            Self::TooMany(n) => write!(
                f,
                "property '{n}': a script can offer at most {PROPS_MAX} properties"
            ),
            Self::NotAtTop(n) => write!(
                f,
                "ph2d.property('{n}') must be called at the top of the script, not inside a function"
            ),
        }
    }
}

/// **Quantas propriedades um script pode oferecer.**
///
/// ⚠️ **De que recurso ele é:** das LINHAS que a secção do Inspector sabe pintar — os ids dela são
/// uma tabela deste tamanho, e *um modelo que aceita o que o painel não mostra produz estado
/// inalcançável* (a lei do `ANIM_TAGS_MAX`). Os dois números são o MESMO facto, e um gate na shell,
/// que vê as duas crates, afirma-o.
pub const PROPS_MAX: usize = 32;

/// **Quantas OPÇÕES um enum pode oferecer.**
///
/// ⚠️ **De que recurso ele é, e ele NÃO é a altura do popover:** o popover **rola** (há gate,
/// `a_long_popover_scrolls`), logo o ecrã não o limita. O recurso é a **tabela de ids** do
/// selector — um custo pago em tempo de compilação —, e os dois números são o MESMO facto, com o
/// mesmo gate na shell a afirmá-lo que o [`PROPS_MAX`] tem.
///
/// ⭐ Ele é `32` por ser o mesmo tamanho do irmão, e isso é uma escolha de CONVENIÊNCIA declarada:
/// as duas tabelas nascem do mesmo construtor, e um número diferente não compraria nada.
pub const OPCOES_MAX: usize = 32;

/// Os nomes que o `self` já usa — um script não os pode declarar.
pub const RESERVED_NAMES: &[&str] = &["id"];

/// **Valida uma declaração nova contra as anteriores** — a porta única da recusa.
///
/// # Errors
/// A [`DeclError`] que o painel mostra.
pub fn check_decl(previous: &[PropDecl], decl: &PropDecl) -> Result<(), DeclError> {
    let n = &decl.name;
    let mut chars = n.chars();
    let first_ok = chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_');
    if !first_ok || !chars.all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(DeclError::BadName(n.clone()));
    }
    if RESERVED_NAMES.contains(&n.as_str()) {
        return Err(DeclError::Reserved(n.clone()));
    }
    if previous.iter().any(|p| p.name == *n) {
        return Err(DeclError::Twice(n.clone()));
    }
    if previous.len() >= PROPS_MAX {
        return Err(DeclError::TooMany(n.clone()));
    }
    // ⚠️ **Pela PORTA, e não por variante** — ver [`ScriptValue::componentes`].
    if decl.default.componentes().iter().any(|v| !v.is_finite()) {
        return Err(DeclError::BadDefault(n.clone()));
    }
    // ⭐⭐⭐ **O domínio de uma COR, e a recusa NÃO retira capacidade nenhuma.**
    //
    // A única superfície que edita uma cor é a AMOSTRA, e o selector da casa é `0..=1` por
    // construção. Um canal declarado a `2` seria pintado como `1` e **reescrito em silêncio** no
    // primeiro toque — o *«aceita e mente»* que esta casa já pagou no `lattice`, no `kaleidoscope`
    // e no `iterations` do colisor.
    //
    // ⭐ **E o que torna a recusa barata é a COMPOSIÇÃO ficar intacta:** quem quiser uma cor fora
    // de `0..=1` (um emissivo) declara **três números**, exactamente como fazia antes desta wave.
    // *Uma recusa que não fecha caminho nenhum é só um silêncio que passou a falar.*
    if let ScriptValue::Color(c) = &decl.default
        && c.iter().any(|v| !(0.0..=1.0).contains(v))
    {
        return Err(DeclError::ColorOutOfRange(n.clone()));
    }
    let h = &decl.hint;
    let has_hint = h.min.is_some() || h.max.is_some() || h.step.is_some();
    // ⭐ **A faixa vale para o `Vec2`, e as duas componentes partilham-na** — é o que o
    // `@export_range` do oráculo faz sobre um `Vector2`, e uma faixa por eixo seria uma segunda
    // resposta a *«até onde este ponto pode ir»*.
    //
    // ⛔ **Numa COR ela é RECUSADA:** o domínio dela é `0..=1` por natureza, e um `min`/`max` ali
    // seria a segunda resposta à mesma pergunta — com as duas a divergirem no dia em que uma
    // mudasse.
    let aceita_faixa = matches!(
        decl.default.kind(),
        ScriptValueKind::Number | ScriptValueKind::Vec2
    );
    if has_hint && !aceita_faixa {
        return Err(DeclError::BadHint(n.clone()));
    }
    let finite = |x: Option<f64>| x.is_none_or(f64::is_finite);
    if !finite(h.min) || !finite(h.max) || !finite(h.step) {
        return Err(DeclError::BadHint(n.clone()));
    }
    if h.step.is_some_and(|s| s <= 0.0) {
        return Err(DeclError::BadHint(n.clone()));
    }
    if let (Some(lo), Some(hi)) = (h.min, h.max)
        && lo > hi
    {
        return Err(DeclError::BadHint(n.clone()));
    }
    opcoes_sao_validas(decl, n.as_str())?;
    Ok(())
}

/// ⭐⭐⭐ **A recusa de uma LISTA malformada — e ela falha FECHADA.**
///
/// ⚠️⚠️ **A lei que esta casa pagou no L-System:** *o parser falhava ABERTO em dois dos três
/// sub-campos de uma regra — uma condição que não compila EVAPORAVA e ia desenhar.* Aqui, uma
/// lista que não se pode honrar **recusa a declaração inteira**, com a queixa a chegar ao painel
/// pelo caminho que o `DeclError` já tem.
///
/// As quatro recusas, e cada uma impede um estado que o produto não sabe pintar:
///
/// 1. **opções num default que não é TEXTO** — o chip escolheria uma string para um número;
/// 2. **uma opção VAZIA** — um chip sem rótulo é um controlo que o artista não sabe que existe;
/// 3. **opções REPETIDAS** — duas entradas iguais dão dois chips indistinguíveis, e escolher um
///    deles é indistinguível de escolher o outro;
/// 4. ⭐ **o default FORA da lista** — a mais importante das quatro: sem ela, um objecto que não
///    pôs nada leria um valor que a própria declaração recusa, e o [`resolve`] cairia num default
///    que ele próprio nomearia órfão se alguém o tivesse posto à mão.
fn opcoes_sao_validas(decl: &PropDecl, n: &str) -> Result<(), DeclError> {
    let opts = &decl.hint.options;
    if opts.is_empty() {
        return Ok(());
    }
    let ScriptValue::Text(padrao) = &decl.default else {
        return Err(DeclError::BadHint(n.to_owned()));
    };
    if opts.iter().any(String::is_empty) {
        return Err(DeclError::BadHint(n.to_owned()));
    }
    let unicas: std::collections::BTreeSet<&String> = opts.iter().collect();
    if unicas.len() != opts.len() {
        return Err(DeclError::BadHint(n.to_owned()));
    }
    if !opts.contains(padrao) {
        return Err(DeclError::BadHint(n.to_owned()));
    }
    if opts.len() > OPCOES_MAX {
        return Err(DeclError::BadHint(n.to_owned()));
    }
    Ok(())
}

#[cfg(test)]
#[path = "props_tests.rs"]
mod tests;

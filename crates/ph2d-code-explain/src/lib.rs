//! **A explicação escrita à mão** de cada mensagem do Luau (CodeEditor Onda 2, I8).
//!
//! O editor mostra-a na calha e o Inspector no cartão; a mensagem crua fica no balão do rato e no
//! console. Uma [`Kind`] por classe do corpus do I0c (`docs/CodeEditor/ferramentas/corpus_erros/`),
//! juntando as variantes que pedem o mesmo conserto; o texto mora em `ph2d-i18n`
//! (`code.explain.<id>` e `.why`). Medição e lei: `docs/CodeEditor/ferramentas/medicoes_onda2/
//! I8_explicacoes.md`. O censo (`tests/it/censo.rs`) passa o corpus inteiro por [`explain`]: uma
//! mensagem sem explicação, ou uma classe sem texto, é vermelho.

mod read;

use ph2d_i18n::tr_with;
use read::{
    after, first_quoted, got, lua_type, normalize, number_after, opener, quoted_after, short_type,
    word_after,
};
use std::fmt::Display;

macro_rules! kinds {
    ($($k:ident => $id:literal),* $(,)?) => {
        /// Uma classe de mensagem com explicação própria.
        #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub enum Kind { $($k),* }

        impl Kind {
            /// Todas, pela ordem da tabela.
            pub const ALL: &'static [Kind] = &[$(Kind::$k),*];

            /// O nome estável (o registo local das mensagens vistas grava-o).
            #[must_use]
            pub const fn id(self) -> &'static str {
                match self { $(Kind::$k => $id),* }
            }

            /// A chave do título (uma linha, com os nomes reais).
            #[must_use]
            pub const fn title_key(self) -> &'static str {
                match self { $(Kind::$k => concat!("code.explain.", $id)),* }
            }

            /// A chave do porquê e do que fazer.
            #[must_use]
            pub const fn why_key(self) -> &'static str {
                match self { $(Kind::$k => concat!("code.explain.", $id, ".why")),* }
            }
        }
    };
}

kinds! {
    IncompleteStatement => "incomplete_statement",
    ExpectedExpression => "expected_expression",
    ExpectedExpressionEof => "expected_expression_eof",
    VariableName => "variable_name",
    Unclosed => "unclosed",
    UnclosedHere => "unclosed_here",
    ExpectedThen => "expected_then",
    ExpectedDo => "expected_do",
    UnclosedBlock => "unclosed_block",
    ExtraToken => "extra_token",
    MalformedString => "malformed_string",
    TableComma => "table_comma",
    CallArgs => "call_args",
    COperator => "c_operator",
    FunctionParen => "function_paren",
    ExpectedEquals => "expected_equals",
    UnknownGlobal => "unknown_global",
    TypeMismatch => "type_mismatch",
    UnknownKey => "unknown_key",
    UnknownKeyHere => "unknown_key_here",
    TooFewArgs => "too_few_args",
    TooManyArgs => "too_many_args",
    TooFewArgsAnon => "too_few_args_anon",
    CompareTypes => "compare_types",
    MissingReturn => "missing_return",
    CallNilType => "call_nil_type",
    NeedsColon => "needs_colon",
    BadCast => "bad_cast",
    SameLine => "same_line",
    ShadowGlobal => "shadow_global",
    ShadowLocal => "shadow_local",
    LocalUnused => "local_unused",
    FunctionUnused => "function_unused",
    GlobalAsLocal => "global_as_local",
    Uninitialized => "uninitialized",
    ImplicitReturn => "implicit_return",
    Unreachable => "unreachable",
    TooManyValues => "too_many_values",
    TooManyVars => "too_many_vars",
    Multiline => "multiline",
    IndexNil => "index_nil",
    IndexNilNumber => "index_nil_number",
    Arith => "arith",
    CallNilValue => "call_nil_value",
    ConvertArgument => "convert_argument",
    MissingArgument => "missing_argument",
    InvalidArgument => "invalid_argument",
    TableIndexNil => "table_index_nil",
    Compare => "compare",
    Concat => "concat",
    BadDefault => "bad_default",
    BadName => "bad_name",
    BadHint => "bad_hint",
    UnknownField => "unknown_field",
    UnknownFieldNear => "unknown_field_near",
    UnknownObject => "unknown_object",
    UnknownObjectNear => "unknown_object_near",
    ObjectTwice => "object_twice",
    UnknownMaster => "unknown_master",
    UnknownMasterNear => "unknown_master_near",
    UnknownProperty => "unknown_property",
    UnknownPropertyNear => "unknown_property_near",
}

impl Kind {
    /// Uma classe NOSSA (o lint do `self`, I7): não vem do Luau, logo não está no corpus.
    #[must_use]
    pub const fn ours(self) -> bool {
        matches!(
            self,
            Kind::UnknownField
                | Kind::UnknownFieldNear
                | Kind::UnknownObject
                | Kind::UnknownObjectNear
                | Kind::ObjectTwice
                | Kind::UnknownMaster
                | Kind::UnknownMasterNear
                | Kind::UnknownProperty
                | Kind::UnknownPropertyNear
        )
    }
}

/// Uma mensagem explicada: a classe e os nomes que o texto leva.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Explained {
    pub kind: Kind,
    args: Vec<(&'static str, String)>,
}

impl Explained {
    fn new(kind: Kind, args: &[(&'static str, &str)]) -> Self {
        let args = args.iter().map(|(k, v)| (*k, (*v).to_string())).collect();
        Self { kind, args }
    }

    /// O aviso nosso do campo do `self` que ninguém dá (I7), com o vizinho mais perto se houver.
    #[must_use]
    pub fn unknown_field(name: &str, near: Option<&str>) -> Self {
        match near {
            Some(near) => Self::new(Kind::UnknownFieldNear, &[("name", name), ("near", near)]),
            None => Self::new(Kind::UnknownField, &[("name", name)]),
        }
    }

    /// O aviso nosso do nome de objecto que a cena não tem (`ph2d.object`, item 8), com o mais perto.
    #[must_use]
    pub fn unknown_object(name: &str, near: Option<&str>) -> Self {
        match near {
            Some(near) => Self::new(Kind::UnknownObjectNear, &[("name", name), ("near", near)]),
            None => Self::new(Kind::UnknownObject, &[("name", name)]),
        }
    }

    /// O aviso nosso do nome que `count` objectos da cena partilham (item 8).
    #[must_use]
    pub fn object_twice(name: &str, count: usize) -> Self {
        Self::new(
            Kind::ObjectTwice,
            &[("name", name), ("count", &count.to_string())],
        )
    }

    /// O aviso nosso do mestre que a cena não tem (`ph2d.spawn`, item 8), com o mais perto.
    #[must_use]
    pub fn unknown_master(name: &str, near: Option<&str>) -> Self {
        match near {
            Some(near) => Self::new(Kind::UnknownMasterNear, &[("name", name), ("near", near)]),
            None => Self::new(Kind::UnknownMaster, &[("name", name)]),
        }
    }

    /// O aviso nosso do nome de propriedade que nenhum componente tem (`ph2d.get/set`, doc
    /// CodeEditor 08), com o mais perto.
    #[must_use]
    pub fn unknown_property(name: &str, near: Option<&str>) -> Self {
        match near {
            Some(near) => Self::new(Kind::UnknownPropertyNear, &[("name", name), ("near", near)]),
            None => Self::new(Kind::UnknownProperty, &[("name", name)]),
        }
    }

    /// O título, uma linha.
    #[must_use]
    pub fn title(&self) -> String {
        self.fill(self.kind.title_key())
    }

    /// O porquê e o que fazer.
    #[must_use]
    pub fn why(&self) -> String {
        self.fill(self.kind.why_key())
    }

    /// Um dos nomes lidos da mensagem (`"name"`, `"key"`…).
    #[must_use]
    pub fn arg(&self, name: &str) -> Option<&str> {
        self.args
            .iter()
            .find(|(k, _)| *k == name)
            .map(|(_, v)| v.as_str())
    }

    fn fill(&self, key: &str) -> String {
        let args: Vec<(&str, &dyn Display)> = self
            .args
            .iter()
            .map(|(k, v)| (*k, v as &dyn Display))
            .collect();
        tr_with(key, &args)
    }
}

/// A explicação de uma mensagem crua do `luau-lsp` ou do motor, ou `None` se a classe é nova.
#[must_use]
pub fn explain(raw: &str) -> Option<Explained> {
    let all = normalize(raw);
    let m = all
        .strip_prefix("TypeError: ")
        .or_else(|| all.strip_prefix("SyntaxError: "))
        .unwrap_or(&all);
    syntax(m)
        .or_else(|| types(m))
        .or_else(|| lint(m))
        .or_else(|| runtime(m))
        .or_else(|| declaration(m))
}

fn syntax(m: &str) -> Option<Explained> {
    use Kind::*;
    let e = |k| Some(Explained::new(k, &[]));
    if m.starts_with("Incomplete statement") {
        return e(IncompleteStatement);
    }
    if m.starts_with("Expected identifier when parsing expression") {
        return match got(m) {
            Some(t) => Some(Explained::new(ExpectedExpression, &[("got", t)])),
            None => e(ExpectedExpressionEof),
        };
    }
    if m.starts_with("Expected identifier when parsing variable name") {
        return e(VariableName);
    }
    for close in [")", "]", "}"] {
        if m.starts_with(&format!("Expected '{close}' (to close")) {
            let open = opener(close);
            return Some(match number_after(m, "at line ") {
                Some(line) => Explained::new(
                    Unclosed,
                    &[("open", open), ("close", close), ("line", &line)],
                ),
                None => Explained::new(UnclosedHere, &[("close", close)]),
            });
        }
    }
    if m.starts_with("Expected 'then'") {
        return e(ExpectedThen);
    }
    if m.starts_with("Expected 'do'") {
        return e(ExpectedDo);
    }
    if m.starts_with("Expected 'end' (to close") {
        let kw = quoted_after(m, "to close ")?;
        let line = number_after(m, "at line ")?;
        return Some(Explained::new(
            UnclosedBlock,
            &[("kw", kw), ("line", &line)],
        ));
    }
    if m.starts_with("Expected <eof>, got") {
        return Some(Explained::new(ExtraToken, &[("got", got(m)?)]));
    }
    if m.starts_with("Malformed string") {
        return e(MalformedString);
    }
    if m.starts_with("Expected ',' after table constructor element") {
        return e(TableComma);
    }
    if m.starts_with("Expected '(', '{' or <string> when parsing function call")
        || m.starts_with("Expected function call arguments after '('")
    {
        return e(CallArgs);
    }
    if m.starts_with("Unexpected '") && m.contains("did you mean '") {
        let bad = first_quoted(m)?;
        let good = quoted_after(m, "did you mean ")?;
        return Some(Explained::new(COperator, &[("bad", bad), ("good", good)]));
    }
    if m.starts_with("Expected '(' when parsing function") {
        return e(FunctionParen);
    }
    if m.starts_with("Expected '=' when parsing assignment") {
        return e(ExpectedEquals);
    }
    None
}

fn types(m: &str) -> Option<Explained> {
    use Kind::*;
    let e = |k| Some(Explained::new(k, &[]));
    if m.starts_with("Unknown global ") {
        let name = quoted_after(m, "Unknown global ")?;
        return Some(Explained::new(UnknownGlobal, &[("name", name)]));
    }
    if m.starts_with("Expected this to be ") {
        let expected = short_type(quoted_after(m, "Expected this to be ")?);
        let got = short_type(quoted_after(m, "but got ")?);
        return Some(Explained::new(
            TypeMismatch,
            &[("expected", &expected), ("got", &got)],
        ));
    }
    if m.starts_with("Key '") && m.contains(" not found in table '") {
        let key = quoted_after(m, "Key ")?;
        let table = after(m, " not found in table '")?;
        let table = table.strip_suffix('\'').unwrap_or(table);
        let table = table
            .strip_prefix("typeof(")
            .and_then(|t| t.strip_suffix(')'))
            .unwrap_or(table);
        if table.starts_with('{') || table.contains("*TRUNCATED*") || table.chars().count() > 40 {
            return Some(Explained::new(UnknownKeyHere, &[("key", key)]));
        }
        return Some(Explained::new(
            UnknownKey,
            &[("key", key), ("table", table)],
        ));
    }
    if m.starts_with("Argument count mismatch.") {
        let expects = after(m, "expects ")?.split(" argument").next()?;
        let few = m.contains("but only ") || m.contains("but none");
        let given = if m.contains("but none") {
            "0".to_string()
        } else {
            number_after(m, if few { "but only " } else { "but " })?
        };
        let counts = [("expects", expects), ("given", given.as_str())];
        return match (quoted_after(m, "Function "), few) {
            (Some(name), true) => Some(Explained::new(
                TooFewArgs,
                &[("name", name), counts[0], counts[1]],
            )),
            (Some(name), false) => Some(Explained::new(
                TooManyArgs,
                &[("name", name), counts[0], counts[1]],
            )),
            (None, true) => Some(Explained::new(TooFewArgsAnon, &counts)),
            (None, false) => None,
        };
    }
    if m.starts_with("Type ") && m.contains(" cannot be compared with ") {
        let a = after(m, "Type ")?.split(" cannot be compared").next()?;
        let b = after(m, " cannot be compared with ")?;
        let unq = |t: &str| short_type(t.trim_matches('\''));
        return Some(Explained::new(
            CompareTypes,
            &[("a", &unq(a)), ("b", &unq(b))],
        ));
    }
    if m.starts_with("Not all codepaths in this function return") {
        return e(MissingReturn);
    }
    if m.starts_with("Cannot call a value of type nil") {
        return e(CallNilType);
    }
    if m.starts_with("This function must be called with self") {
        return e(NeedsColon);
    }
    if m.starts_with("Cannot cast '") {
        let from = short_type(quoted_after(m, "Cannot cast ")?);
        let to = short_type(quoted_after(m, " into ")?);
        return Some(Explained::new(BadCast, &[("from", &from), ("to", &to)]));
    }
    None
}

fn lint(m: &str) -> Option<Explained> {
    use Kind::*;
    let (tag, _) = m.split_once(": ")?;
    let named = |k| Some(Explained::new(k, &[("name", first_quoted(m)?)]));
    match tag {
        "SameLineStatement" => Some(Explained::new(SameLine, &[])),
        "LocalShadow" if m.contains("shadows a global") => named(ShadowGlobal),
        "LocalShadow" => named(ShadowLocal),
        "LocalUnused" => named(LocalUnused),
        "FunctionUnused" => named(FunctionUnused),
        "GlobalUsedAsLocal" => named(GlobalAsLocal),
        "UninitializedLocal" => named(Uninitialized),
        "ImplicitReturn" => named(ImplicitReturn),
        "UnreachableCode" => Some(Explained::new(Unreachable, &[])),
        "UnbalancedAssignment" => {
            let values = number_after(m, "Assigning ")?;
            let vars = number_after(m, "values to ")?;
            let k = if m.contains("leaves some values unused") {
                TooManyValues
            } else {
                TooManyVars
            };
            Some(Explained::new(k, &[("values", &values), ("vars", &vars)]))
        }
        "MultiLineStatement" => Some(Explained::new(Multiline, &[])),
        _ => None,
    }
}

fn runtime(m: &str) -> Option<Explained> {
    use Kind::*;
    if m.starts_with("attempt to index nil with ") {
        return Some(match quoted_after(m, "with ") {
            Some(key) => Explained::new(IndexNil, &[("key", key)]),
            None => Explained::new(IndexNilNumber, &[]),
        });
    }
    if m.starts_with("attempt to perform arithmetic") {
        let (a, b) = after(m, ") on ")?.split_once(" and ")?;
        let what = if a == "number" { b } else { a };
        return Some(Explained::new(Arith, &[("what", what)]));
    }
    if m.starts_with("attempt to call a nil value") {
        return Some(Explained::new(CallNilValue, &[]));
    }
    if m.starts_with("bad argument #") && m.contains("error converting Lua ") {
        let n = number_after(m, "#")?;
        let got = word_after(m, "error converting Lua ")?;
        let want = lua_type(&word_after(m, &format!("Lua {got} to "))?);
        return Some(Explained::new(
            ConvertArgument,
            &[("n", &n), ("got", &got), ("want", &want)],
        ));
    }
    if m.starts_with("missing argument #") {
        let n = number_after(m, "#")?;
        let name = first_quoted(m)?;
        let want = word_after(m, "(")?;
        return Some(Explained::new(
            MissingArgument,
            &[("n", &n), ("name", name), ("want", &want)],
        ));
    }
    if m.starts_with("invalid argument #") {
        let n = number_after(m, "#")?;
        let name = first_quoted(m)?;
        let want = word_after(m, "(")?;
        let got = word_after(m, "got ")?;
        return Some(Explained::new(
            InvalidArgument,
            &[("n", &n), ("name", name), ("want", &want), ("got", &got)],
        ));
    }
    if m.starts_with("table index is nil") {
        return Some(Explained::new(TableIndexNil, &[]));
    }
    if let Some(rest) = m.strip_prefix("attempt to compare ") {
        let mut w = rest.split(' ');
        let (a, _, b) = (w.next()?, w.next()?, w.next()?);
        return Some(Explained::new(Compare, &[("a", a), ("b", b)]));
    }
    if m.starts_with("attempt to concatenate ") {
        let a = word_after(m, "concatenate ")?;
        let b = word_after(m, " with ")?;
        return Some(Explained::new(Concat, &[("a", &a), ("b", &b)]));
    }
    None
}

fn declaration(m: &str) -> Option<Explained> {
    use Kind::*;
    let name = first_quoted(m)?;
    if m.starts_with("property '") && m.contains("needs a number, boolean, string") {
        return Some(Explained::new(BadDefault, &[("name", name)]));
    }
    if m.ends_with("is not a valid property name") {
        return Some(Explained::new(BadName, &[("name", name)]));
    }
    if m.starts_with("property '") && m.contains("min/max/step") {
        return Some(Explained::new(BadHint, &[("name", name)]));
    }
    None
}

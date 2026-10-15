//! ⭐⭐⭐ **O VOCABULÁRIO DO SCRIPT DO ARTISTA** (TOP-20 #16) — o instantâneo e a edição, num módulo
//! abaixo do [`crate::action_bus`], como os irmãos [`crate::statemachine_edits`] e companhia.
//!
//! # ⚠️ O painel não conhece o script — conhece o que a shell RESOLVEU
//!
//! As linhas de números nascem das DECLARAÇÕES do ficheiro (`ph2d.property`), que só a VM conhece,
//! e a origem de cada valor (do script ou do objecto) é a resposta da lei `ph2d_script::resolve`.
//! Esta crate não depende da VM ⇒ o instantâneo traz a resposta pronta, e o painel **pinta**. ⛔ Uma
//! segunda resolução aqui seria a segunda resposta a *«que valor este objecto usa?»*.
//!
//! # ⭐⭐ O que o painel DIZ que os campos sozinhos não diriam
//!
//! | aviso | o que se passa |
//! |---|---|
//! | `No script file yet` | o componente não aponta script nenhum, e não corre |
//! | `Script file not found: …` | um caminho antigo que o load não importou — nunca corre vazio |
//! | `That script is gone` | o projeto já não tem o script apontado — os valores próprios ficam |
//! | `The script has an error` | o Luau recusou-o; a mensagem é a do Luau, com a LINHA |
//! | `Stopped: …` | o objecto parou de correr o script (erro num gancho, laço sem fim) |
//! | `… not in the script` | um valor próprio cuja propriedade saiu do ficheiro (D2) |
//! | `The clock is stopped` | um script só corre com a cena a tocar |
//!
//! # ⚠️ As edições levam o NOME da propriedade, nunca a linha
//!
//! A ordem das linhas é a das declarações, e um ficheiro gravado a meio de uma edição pode trocá-la.
//! Um índice mudaria de propriedade debaixo da mão do artista; o nome é o contrato.

/// Um valor, como o painel o edita. Espelho do `ph2d_script::ScriptValue` (esta crate não depende
/// da VM).
#[derive(Clone, Debug, PartialEq)]
pub enum InspectorScriptValue {
    /// Um número — campo numérico.
    Number(f64),
    /// Um sim/não — caixa.
    Bool(bool),
    /// Um texto — campo de texto.
    Text(String),
    /// ⭐ Uma posição — **dois campos numa fileira só**.
    Vec2([f64; 2]),
    /// ⭐ Uma cor — **uma amostra** que abre o selector da casa.
    Color([f64; 4]),
}

/// Uma linha de propriedade, na ordem da declaração.
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorScriptProp {
    /// O nome declarado.
    pub name: String,
    /// O valor que o objecto usa.
    pub value: InspectorScriptValue,
    /// ⭐ **O artista PÔS este valor?** — a cor da linha e o botão *Reset*.
    pub own: bool,
    /// ⭐⭐⭐ **As OPÇÕES que o script declarou** — vazia = texto livre, que é o de sempre.
    ///
    /// ⚠️ Ela é do PAINEL e não do documento: o que o objecto guarda é o texto escolhido, e a
    /// lista vive na declaração — *renomear uma opção no script não reescreve o que os objectos
    /// têm*, e é isso que faz um valor que saiu da lista ser NOMEADO em vez de apagado.
    pub options: Vec<String>,
    /// As pistas de edição de um número (a faixa do campo).
    pub min: Option<f64>,
    /// Ver [`Self::min`].
    pub max: Option<f64>,
    /// O passo de arrasto.
    pub step: Option<f64>,
}

/// Um valor próprio sem onde ser aplicado (a divergência D2/D3 do oráculo).
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorScriptOrphan {
    /// O nome gravado.
    pub name: String,
    /// O valor gravado, pronto a ler.
    pub value: InspectorScriptValue,
    /// Porque ele não se aplica — ver [`PorqueOrfao`].
    pub wants: PorqueOrfao,
}

/// **Porque um valor próprio não tem onde ser aplicado** — a frase que o painel mostra.
///
/// ⚠️⚠️ **É um ENUM e não um `Option` com um `bool` ao lado**, e a razão é a lei da casa: dois
/// campos que têm de concordar divergem, e o terceiro estado nasceu exactamente assim (ele foi um
/// `Option<&str>` de dois estados até o enum de script chegar). *Um estado a mais num tipo é um
/// braço a mais num `match`; um estado a mais em dois campos é uma combinação impossível.*
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PorqueOrfao {
    /// O script já não declara este nome (D2).
    NaoDeclarado,
    /// O script declara-o com OUTRO tipo (D3) — o nome do tipo que ele pede agora.
    OutroTipo(&'static str),
    /// ⭐ O script declara-o com uma LISTA, e este valor não está nela.
    ForaDaLista,
}

/// **Um erro do Luau, como o cartão o pinta** — a 1.ª linha da mensagem e a linha DESTE script.
/// Quem compõe a frase é o pintor (a chave `panel.inspector.script.at_line`).
///
/// ⭐ (CodeEditor E7, retorno do dono: *«o `=` cru entre aspas confunde»*) o símbolo culpado com que
/// o Luau ACABA a mensagem (`…, got '='`) sai dela para `symbol`, e o cartão pinta-o destacado.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InspectorScriptError {
    pub message: String,
    pub line: Option<u32>,
    pub symbol: Option<String>,
}

impl InspectorScriptError {
    /// Lê a 1.ª linha de uma mensagem do Luau: um `'símbolo'` no FIM vai para `symbol` e sai da
    /// frase; aspas a meio da frase ficam (são palavras, não o culpado).
    #[must_use]
    pub fn from_luau(message: &str, line: Option<u32>) -> Self {
        let first = message.lines().next().unwrap_or_default().trim_end();
        let split = first
            .strip_suffix('\'')
            .and_then(|s| s.rfind('\'').map(|i| (&s[..i], &s[i + 1..])))
            .filter(|(_, sym)| !sym.is_empty());
        match split {
            Some((head, sym)) => Self {
                message: head.trim_end().to_owned(),
                line,
                symbol: Some(sym.to_owned()),
            },
            None => Self {
                message: first.to_owned(),
                line,
                symbol: None,
            },
        }
    }
}

/// **Em que estado está o script.**
#[derive(Clone, Debug, PartialEq)]
pub enum InspectorScriptStatus {
    /// O componente não aponta script nenhum.
    NoFile,
    /// A VM não arrancou nesta sessão — o app corre sem scripts.
    Unavailable,
    /// O script ainda não foi lido (o primeiro quadro depois de o escolher).
    Loading,
    /// O projeto não tem o script apontado.
    Missing,
    /// O Luau recusou-o e NADA corre (nunca houve uma versão boa) — a mensagem é a dele.
    Broken(InspectorScriptError),
    /// ⭐ (CodeEditor E6/E7) O texto novo foi recusado, e os objectos CONTINUAM na versão anterior.
    BrokenRunsPrevious(InspectorScriptError),
    /// Carregou.
    Ready,
    /// ⛔ Um caminho de ficheiro que o load não importou (ADR-0188) — RECUSADO, nunca corre vazio.
    Unimported(String),
}

/// Snapshot da secção SCRIPT da entidade selecionada.
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorScriptInfo {
    pub entity_bits: u64,
    /// O nome do script na biblioteca do projeto — ou o caminho por importar.
    pub source: String,
    pub status: InspectorScriptStatus,
    /// Uma por declaração, na ordem da declaração.
    pub props: Vec<InspectorScriptProp>,
    pub orphans: Vec<InspectorScriptOrphan>,
    /// ⚠️ **Valores próprios GUARDADOS enquanto as declarações são desconhecidas** (ficheiro sumido
    /// ou partido) — contados e nunca oferecidos para apagar: *não sei* não é *nada*.
    pub kept: usize,
    /// Porque o objecto parou de correr o script, se parou.
    pub failure: Option<InspectorScriptError>,
    /// O relógio está a andar?
    pub clock_playing: bool,
    /// ⚠️ O objecto é também um corpo da física — as duas mãos brigam pelo `Transform`.
    pub also_physics: bool,
    pub selected_count: usize,
}

/// Uma edição da secção SCRIPT.
#[derive(Clone, Debug, PartialEq)]
pub enum ScriptFieldEdit {
    /// Escrito à mão: o NOME de um script do projeto, ou o caminho de um ficheiro a importar.
    Source(String),
    /// `Browse` — a shell abre o diálogo (é ela que tem a janela) e o ficheiro é IMPORTADO.
    Browse,
    /// ⭐ `Edit script` — abre o script do objecto numa aba do editor (CodeEditor E3); o documento da
    /// cena não muda.
    Edit,
    /// O artista PÔS este número.
    SetNumber(String, f64),
    /// O artista PÔS este sim/não.
    SetBool(String, bool),
    /// O artista PÔS este texto.
    SetText(String, String),
    /// ⭐ O artista PÔS esta posição.
    ///
    /// ⚠️ **Ela leva as DUAS componentes**, e não *«a componente `i`»*: o documento guarda um
    /// valor só, e uma edição por eixo obrigaria quem aplica a ler o valor de antes para compor o
    /// novo — a segunda resposta a *«qual é a posição?»*, com a corrida entre os dois campos por
    /// prémio. O painel lê o par vivo e manda-o inteiro.
    SetVec2(String, [f64; 2]),
    /// ⭐ O artista PÔS esta cor.
    SetColor(String, [f64; 4]),
    /// `Reset` numa linha, e `Remove` num órfão — a MESMA porta (`ph2d_script::props::forget`).
    Forget(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐ (E7) O símbolo do FIM sai da frase sem aspas; aspas a meio ficam; sem aspas, nada.
    #[test]
    fn o_simbolo_do_fim_sai_da_frase() {
        let e = InspectorScriptError::from_luau(
            "Expected identifier when parsing expression, got '='",
            Some(3),
        );
        assert_eq!(
            (e.message.as_str(), e.symbol.as_deref(), e.line),
            (
                "Expected identifier when parsing expression, got",
                Some("="),
                Some(3)
            )
        );
        let e = InspectorScriptError::from_luau("attempt to index nil with 'x'\nstack", None);
        assert_eq!(
            (e.message.as_str(), e.symbol.as_deref()),
            ("attempt to index nil with", Some("x"))
        );
        let e = InspectorScriptError::from_luau(
            "Expected 'end' (to close 'function' at line 1), got <eof>",
            None,
        );
        assert_eq!(e.symbol, None);
        assert_eq!(
            e.message,
            "Expected 'end' (to close 'function' at line 1), got <eof>"
        );
        assert_eq!(
            InspectorScriptError::from_luau("got ''", None).symbol,
            None,
            "aspas vazias não são símbolo"
        );
    }

    /// ⚠️ **Todo tipo de valor tem a sua edição** — um tipo sem variante é uma linha que o painel
    /// pinta e ninguém pode mexer.
    #[test]
    fn todo_tipo_de_valor_tem_uma_edicao() {
        for v in [
            InspectorScriptValue::Number(1.0),
            InspectorScriptValue::Bool(true),
            InspectorScriptValue::Text(String::new()),
            InspectorScriptValue::Vec2([0.0, 0.0]),
            InspectorScriptValue::Color([1.0, 1.0, 1.0, 1.0]),
        ] {
            let e = match v {
                InspectorScriptValue::Number(n) => ScriptFieldEdit::SetNumber("a".into(), n),
                InspectorScriptValue::Bool(b) => ScriptFieldEdit::SetBool("a".into(), b),
                InspectorScriptValue::Text(t) => ScriptFieldEdit::SetText("a".into(), t),
                InspectorScriptValue::Vec2(p) => ScriptFieldEdit::SetVec2("a".into(), p),
                InspectorScriptValue::Color(c) => ScriptFieldEdit::SetColor("a".into(), c),
            };
            assert!(!matches!(e, ScriptFieldEdit::Forget(_)));
        }
        let _ = ScriptFieldEdit::Forget(String::new());
        let _ = ScriptFieldEdit::Source(String::new());
        let _ = ScriptFieldEdit::Browse;
        let _ = ScriptFieldEdit::Edit;
    }
}

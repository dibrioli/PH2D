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
//! | `No script file yet` | o componente não nomeia ficheiro nenhum, e não corre |
//! | `That file is gone` | o caminho gravado não se deixa ler — os valores próprios ficam |
//! | `The script has an error` | o Luau recusou-o; a mensagem é a do Luau |
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

/// **Em que estado está o ficheiro.**
#[derive(Clone, Debug, PartialEq)]
pub enum InspectorScriptStatus {
    /// O componente não nomeia ficheiro nenhum.
    NoFile,
    /// A VM não arrancou nesta sessão — o app corre sem scripts.
    Unavailable,
    /// O ficheiro ainda não foi lido (o primeiro quadro depois de o nomear).
    Loading,
    /// O ficheiro não se deixa ler.
    Missing,
    /// O Luau recusou-o — a mensagem é a dele.
    Broken(String),
    /// Carregou.
    Ready,
}

/// Snapshot da secção SCRIPT da entidade selecionada.
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorScriptInfo {
    pub entity_bits: u64,
    /// O caminho, como o componente o guarda.
    pub source: String,
    pub status: InspectorScriptStatus,
    /// Uma por declaração, na ordem da declaração.
    pub props: Vec<InspectorScriptProp>,
    pub orphans: Vec<InspectorScriptOrphan>,
    /// ⚠️ **Valores próprios GUARDADOS enquanto as declarações são desconhecidas** (ficheiro sumido
    /// ou partido) — contados e nunca oferecidos para apagar: *não sei* não é *nada*.
    pub kept: usize,
    /// Porque o objecto parou de correr o script, se parou.
    pub failure: Option<String>,
    /// O relógio está a andar?
    pub clock_playing: bool,
    /// ⚠️ O objecto é também um corpo da física — as duas mãos brigam pelo `Transform`.
    pub also_physics: bool,
    pub selected_count: usize,
}

/// Uma edição da secção SCRIPT.
#[derive(Clone, Debug, PartialEq)]
pub enum ScriptFieldEdit {
    /// O caminho, escrito à mão.
    Source(String),
    /// `Browse` — a shell abre o diálogo (é ela que tem a janela).
    Browse,
    /// O artista PÔS este número.
    SetNumber(String, f64),
    /// O artista PÔS este sim/não.
    SetBool(String, bool),
    /// O artista PÔS este texto.
    SetText(String, String),
    /// `Reset` numa linha, e `Remove` num órfão — a MESMA porta (`ph2d_script::props::forget`).
    Forget(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⚠️ **Todo tipo de valor tem a sua edição** — um tipo sem variante é uma linha que o painel
    /// pinta e ninguém pode mexer.
    #[test]
    fn todo_tipo_de_valor_tem_uma_edicao() {
        for v in [
            InspectorScriptValue::Number(1.0),
            InspectorScriptValue::Bool(true),
            InspectorScriptValue::Text(String::new()),
        ] {
            let e = match v {
                InspectorScriptValue::Number(n) => ScriptFieldEdit::SetNumber("a".into(), n),
                InspectorScriptValue::Bool(b) => ScriptFieldEdit::SetBool("a".into(), b),
                InspectorScriptValue::Text(t) => ScriptFieldEdit::SetText("a".into(), t),
            };
            assert!(!matches!(e, ScriptFieldEdit::Forget(_)));
        }
        let _ = ScriptFieldEdit::Forget(String::new());
        let _ = ScriptFieldEdit::Source(String::new());
        let _ = ScriptFieldEdit::Browse;
    }
}

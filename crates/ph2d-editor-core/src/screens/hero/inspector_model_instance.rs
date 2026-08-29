//! ⭐⭐⭐ **A seção COMPONENT do Inspector** (ADR-0164 / F5) — *«o que esta cópia tem de diferente
//! da receita»*.
//!
//! # O buraco que ela fecha, e por que ele durou
//!
//! O modelo de override existe desde a F4.4 e é **inteiramente invisível**: o artista sabia que
//! uma peça estava overridada pelo COMPORTAMENTO (a receita deixou de a alcançar) e pelos verbos
//! do menu de linha. ⚠️ *Um estado que só se lê pelo que ele impede não é um estado que o artista
//! possa gerir* — e a lista de abertos deste módulo carregava a frase «nada na tela MOSTRA que
//! campo está overridado» desde 26/08.
//!
//! A F5.3 juntou-lhe um segundo estado, os **órfãos** (uma excepção cuja peça o mestre apagou), e
//! aí a ausência de superfície deixou de ser só desconforto: o plano manda-os **nunca apagar
//! sozinhos**, então sem um gesto eles acumulavam para sempre num sítio que ninguém via.
//!
//! # ⚠️ O que ela mostra é DERIVADO, e é por isso que não há estado aqui
//!
//! Os nomes vêm do `ph2d-component-desc` (o mesmo catálogo de que o `+` deriva a paleta) e o
//! conjunto vem do `ObjectInstance` da RAIZ da instância. Guardar rótulos aqui seria uma segunda
//! resposta a *«como se chama este componente?»*, e ela envelheceria no dia em que o catálogo
//! mudasse um nome.

/// Snapshot do estado de instância da entidade selecionada.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct InspectorInstanceInfo {
    pub entity_bits: u64,
    /// O nome da RECEITA de que esta cópia nasceu — o que o artista lê na Hierarquia.
    pub master_name: String,
    /// Os componentes **desta peça** que a cópia possui contra a receita, por nome de exibição.
    ///
    /// ⚠️ **Desta peça, e não da instância inteira.** O `ObjectInstance` mora na raiz e chaveia por
    /// `(peça, tipo)`; mostrar o conjunto todo numa peça diria ao artista que ele mexeu em coisas
    /// que estão noutro sítio da cópia. *A lista é do que está selecionado.*
    pub overridden: Vec<String>,
    /// Quantas excepções a instância inteira tem **sem alvo** (F5.3) — a peça que as tinha já não
    /// existe no mestre.
    ///
    /// ⚠️ **Da instância INTEIRA**, ao contrário do campo acima, e a diferença é o sujeito: um
    /// órfão não tem peça, então não há peça em que ele pudesse ser listado. É o mesmo sítio onde
    /// o Unity os põe (a lista do `PrefabInstance`, não a do objeto).
    pub orphans: usize,
    /// A entidade da RAIZ da instância — quem recebe o gesto de limpar os órfãos.
    pub root_bits: u64,
}

impl InspectorInstanceInfo {
    /// A linha que resume o estado, para quem não quer ler a lista.
    ///
    /// ⚠️ **Sem excepção nenhuma ela diz «segue a receita»**, e isso é informação: é a diferença
    /// entre *«não mexi nesta»* e *«mexi e não vejo onde»*, que era exactamente o que faltava.
    #[must_use]
    pub fn summary(&self) -> String {
        match (self.overridden.len(), self.orphans) {
            (0, 0) => "Follows the component".to_string(),
            (0, n) => format!("Follows the component \u{b7} {n} unused"),
            (k, 0) => format!("{k} override(s) on this piece"),
            (k, n) => format!("{k} override(s) on this piece \u{b7} {n} unused"),
        }
    }
}

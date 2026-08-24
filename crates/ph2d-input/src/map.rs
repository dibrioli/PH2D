//! **O MAPA** — a colecção de acções, e o contador que torna os ids estáveis.

use serde::{Deserialize, Serialize};

use crate::action::{ActionId, InputAction};

/// **O INPUT MAP**: as acções que este projecto conhece.
///
/// ⚠️ **Uma `Vec` ordenada, e nunca um `HashMap`.** Este mapa alimenta a fita determinística
/// (`InputTape`), e a ordem de iteração de um `HashMap` não é uma promessa — é a mesma espinha de
/// determinismo que faz o módulo de física proibir `HashMap` por lint estrutural. A `Vec` também dá
/// de graça a ordem que o painel mostra, que é a ordem em que o autor as criou.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct InputMap {
    actions: Vec<InputAction>,
    /// O próximo id a atribuir. ⛔ **Viaja com o mapa** — ver [`ActionId`].
    next_id: u32,
}

impl InputMap {
    /// Um mapa vazio.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// **Cria uma acção com `name` e devolve o id estável dela.**
    ///
    /// ⚠️ **Nome repetido devolve a acção que já existe**, em vez de criar uma segunda. Duas acções
    /// com o mesmo nome tornariam [`InputMap::id`] uma pergunta sem resposta única, e o painel
    /// mostraria duas linhas que o código não sabe distinguir — a forma de estado inalcançável que
    /// este repo evita impondo o invariante na porta.
    pub fn create(&mut self, name: impl AsRef<str>) -> ActionId {
        let name = name.as_ref();
        if let Some(a) = self.actions.iter().find(|a| a.name == name) {
            return a.id;
        }
        let id = ActionId(self.next_id);
        self.next_id += 1;
        self.actions.push(InputAction::new(id, name));
        id
    }

    /// Acrescenta uma acção já montada, **adoptando o contador**.
    ///
    /// ⚠️ **O `next_id` sobe para além do id adoptado.** Sem isto, um mapa montado à mão (uma
    /// fixtura, um ficheiro de outra versão) faria a próxima [`InputMap::create`] devolver um id
    /// **já em uso** — a armadilha do contador que este repo já pagou noutro domínio.
    pub fn insert(&mut self, action: InputAction) {
        self.next_id = self.next_id.max(action.id.0.saturating_add(1));
        match self.actions.iter_mut().find(|a| a.id == action.id) {
            Some(slot) => *slot = action,
            None => self.actions.push(action),
        }
    }

    /// O id da acção chamada `name`, se ela existir.
    #[must_use]
    pub fn id(&self, name: &str) -> Option<ActionId> {
        self.actions.iter().find(|a| a.name == name).map(|a| a.id)
    }

    /// A acção com este id.
    #[must_use]
    pub fn get(&self, id: ActionId) -> Option<&InputAction> {
        self.actions.iter().find(|a| a.id == id)
    }

    /// A acção com este id, para editar (o painel).
    pub fn get_mut(&mut self, id: ActionId) -> Option<&mut InputAction> {
        self.actions.iter_mut().find(|a| a.id == id)
    }

    /// **Apaga a acção**, e o id dela **não volta a ser atribuído**.
    ///
    /// ⚠️ É o `next_id` que o garante: ele nunca desce. Uma fita gravada que refira o id apagado
    /// passa a referir **nada** — que é a resposta certa — em vez de referir a acção seguinte que
    /// alguém criasse.
    pub fn remove(&mut self, id: ActionId) -> Option<InputAction> {
        let at = self.actions.iter().position(|a| a.id == id)?;
        Some(self.actions.remove(at))
    }

    /// As acções, na ordem em que o painel as mostra.
    #[must_use]
    pub fn actions(&self) -> &[InputAction] {
        &self.actions
    }

    /// Quantas acções o mapa tem.
    #[must_use]
    pub fn len(&self) -> usize {
        self.actions.len()
    }

    /// O mapa não tem acção nenhuma.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }
}

#[cfg(test)]
#[path = "map_tests.rs"]
mod tests;

//! **O MAPA** — a colecção de acções, e o contador que torna os ids estáveis.

use serde::{Deserialize, Serialize};

use crate::action::{ActionId, InputAction};

/// **OS NOMES DAS ACÇÕES DO JOGADOR** — o contrato entre o mapa e quem o lê.
///
/// ⚠️ Eles vivem aqui, e não em cada leitor, porque uma string escrita à mão em dois sítios é uma
/// que diverge no primeiro erro de dedo — e o modo de falha é o pior que há: `pressed("jamp")`
/// devolve `false` para sempre, em silêncio, com todos os gates verdes.
pub const PLAYER_MOVE_LEFT: &str = "move_left";
/// Ver [`PLAYER_MOVE_LEFT`].
pub const PLAYER_MOVE_RIGHT: &str = "move_right";
/// Ver [`PLAYER_MOVE_LEFT`].
pub const PLAYER_JUMP: &str = "jump";
/// Ver [`PLAYER_MOVE_LEFT`].
pub const PLAYER_DOWN: &str = "down";
/// Ver [`PLAYER_MOVE_LEFT`].
pub const PLAYER_DASH: &str = "dash";
/// Ver [`PLAYER_MOVE_LEFT`].
pub const PLAYER_GRAB: &str = "grab";

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

    /// **O mapa de um projecto NOVO** — os seis verbos que o controlador de plataforma já declara,
    /// ligados às teclas que a shell tinha **cravadas** antes desta wave.
    ///
    /// ⚠️ **Um mapa vazio num projecto novo seria uma regressão**, não um começo limpo: o jogador
    /// deixaria de andar no dia em que o mapa passasse a mandar, e o artista teria de descobrir
    /// sozinho que precisa de declarar seis acções antes de a seta funcionar. O Godot faz o mesmo —
    /// as acções `ui_*` vêm de fábrica.
    ///
    /// ⚠️ **Os nomes são o contrato com quem lê**, e é por isso que estão aqui e não num sítio que
    /// o runtime não alcance. Trocá-los parte todo projecto salvo; acrescentar é livre.
    ///
    /// ⛔ **As teclas são as de HOJE, ao bit**: `←/A` · `→/D` · `↑/Z` · `↓/S` · `Q` · `R`. Um
    /// default "melhor" aqui seria uma mudança de produto escondida numa refactoração.
    #[must_use]
    pub fn with_player_defaults() -> Self {
        /// Os keycodes normalizados que a shell já produzia (ASCII maiúsculo + faixa das setas).
        const LEFT: u32 = 0xF702;
        const RIGHT: u32 = 0xF703;
        const UP: u32 = 0xF700;
        const DOWN: u32 = 0xF701;
        let mut m = Self::new();
        for (name, keys) in [
            (PLAYER_MOVE_LEFT, [LEFT, 0x41].as_slice()),
            (PLAYER_MOVE_RIGHT, [RIGHT, 0x44].as_slice()),
            (PLAYER_JUMP, [UP, 0x5A].as_slice()),
            (PLAYER_DOWN, [DOWN, 0x53].as_slice()),
            (PLAYER_DASH, [0x51].as_slice()),
            (PLAYER_GRAB, [0x52].as_slice()),
        ] {
            let id = m.create(name);
            let a = m.get_mut(id).expect("acabou de nascer");
            for k in keys {
                a.bindings.push(crate::action::Binding::Key(crate::keyboard::Key(*k)));
            }
        }
        m
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

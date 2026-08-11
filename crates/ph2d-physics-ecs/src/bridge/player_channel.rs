//! **O CANAL do player com o mundo de fora** — o que a shell DIZ a ele e o que
//! ela PERGUNTA sobre ele.
//!
//! ⚠️ **Corte por RESPONSABILIDADE** (cap de LOC, `W-Probes`): o pai responde
//! *"o que a lei decidiu neste tique?"* — o cast, a chamada da lei, o motor, a
//! reação — e este responde a pergunta de fora do tique: *o que entra* (a
//! entrada do jogador) e *o que sai* (a descida em curso, a leitura dos
//! sensores). Nenhum dos dois dá um passo.
//!
//! Módulo FILHO por `#[path]`, então `super::*` continua a alcançar o que o pai
//! não exporta.

use super::*;

impl PhysicsBridge {
    /// **A entrada deste player.** Chamada pela shell a cada frame.
    ///
    /// Escrever `drive = 0` é uma instrução (*"pare"*), não a ausência de uma —
    /// e é por isso que a tabela guarda a entrada em vez de a consumir: um
    /// dispatch que deve vários ticks aplica a MESMA entrada a todos eles, que é
    /// o que um jogador segurando uma tecla quer dizer.
    pub fn set_player_input(&mut self, entity: Entity, input: PlayerInput) {
        self.player_input.insert(entity, input);
    }

    /// O que este player está recebendo agora (`default` = parado).
    #[must_use]
    pub fn player_input(&self, entity: Entity) -> PlayerInput {
        self.player_input.get(&entity).copied().unwrap_or_default()
    }

    /// **O que os sensores olharam no último tique** (`W-Probes`) — a leitura
    /// que o overlay desenha.
    ///
    /// Vazia sem player nenhum, e vazia enquanto a física está em `hold` (sem
    /// passo não há sensor). A ordem é a do `BTreeMap` de corpos, logo
    /// determinística; para vários players as marcas vêm concatenadas, e nenhum
    /// consumidor precisa saber de quem é qual — o que se desenha é geometria de
    /// mundo.
    #[must_use]
    pub fn player_probe_marks(&self) -> &[ProbeMark] {
        &self.player_probes
    }

    /// **Este player está ATRAVESSANDO uma plataforma jump-through agora?** (W20)
    ///
    /// ⚠️ **O bit viaja no CORPO e vale para TODA plataforma one-way da cena**
    /// (`oneway::modify_solver_contacts`), então a resposta não é *"aquela
    /// prancha"* — é *"nenhuma prancha é sólida para ele neste instante"*. É
    /// exatamente isso que o contorno desenha, e é o que torna VISÍVEL uma
    /// descida que não se aposenta: hoje ela é silenciosa.
    #[must_use]
    pub fn player_is_dropping(&self, entity: Entity) -> bool {
        self.player_drop.contains_key(&entity)
    }

    /// **Algum player está atravessando alguma coisa?** — a pergunta que o
    /// overlay faz uma vez por quadro, em vez de uma por plataforma.
    #[must_use]
    pub fn any_player_is_dropping(&self) -> bool {
        !self.player_drop.is_empty()
    }

    /// Esquece toda entrada de player.
    ///
    /// Chamada por quem derruba o mundo derivado (`rebuild`): os bits de
    /// entidade são reciclados ali, então uma entrada guardada passaria a
    /// dirigir **outro** objeto — a mesma armadilha que fez as âncoras de joint
    /// viajarem por NOME em vez de por bits.
    pub fn clear_player_input(&mut self) {
        self.player_input.clear();
        // ⚠️ E o estado de PULO junto, pelo mesmo motivo e mais um: além dos bits
        // reciclados, um `airborne` sobrevivente calaria a perna de um corpo que
        // nunca pulou — o personagem cairia através do mundo sem nada na tela a
        // dizer por quê.
        self.player_state.clear();
        // ⚠️ E a DESCIDA (W12), pela razão mais forte das três: ela guarda um
        // `ColliderHandle`, e handles são reciclados junto com os corpos — uma
        // descida sobrevivente apontaria para uma forma que hoje é outra coisa,
        // e o sintoma seria uma plataforma qualquer que deixa de ser sólida sem
        // ninguém ter pedido.
        self.player_drop.clear();
    }
}

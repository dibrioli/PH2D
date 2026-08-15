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

/// **Um empurrão que o MUNDO deu a um player** (`W-Launch`) — o
/// `LaunchCharacter` do Unreal, que existe precisamente porque o controlador
/// comeria a velocidade.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Launch {
    /// A velocidade a SOMAR, em m/s e em eixos de mundo.
    ///
    /// ⚠️ **Soma, não substitui**, no primeiro corte — o `bXYOverride` do Unreal
    /// é uma segunda pergunta, e entra quando houver quem a peça.
    pub velocity: [f32; 2],
    /// **Quantos segundos o empurrão é dono do personagem** — a janela em que a
    /// caminhada fica calada (`PlayerState::push_lock`).
    ///
    /// ⚠️ **Sem ela o empurrão é apagado em 0,15 s**, e isso é medido, não
    /// temido: uma explosão ao lado entrega `13,92 m/s` no primeiro tique e a
    /// caminhada os leva a `0,000` no **décimo**, com o jogador a não tocar em
    /// nada — quem come é o FREIO. Uma porta que entregasse velocidade sem esta
    /// janela seria uma porta que não faz nada.
    ///
    /// ⚠️ **É do CHAMADOR, e não o `WallConfig::jump_lockout`.** O primitivo
    /// reusado é o mecanismo (um relógio que cala o controlo, lido por um `if`
    /// só); o NÚMERO é de quem empurra — uma explosão e uma almofada de salto
    /// não são donas do personagem pelo mesmo tempo, e ler o número da parede
    /// faria um knob significar duas coisas.
    pub lock: f32,
}

impl PhysicsBridge {
    /// **Empurra este player** — a porta que os três modos honram.
    ///
    /// ⚠️ **Ela existe porque um impulso NÃO chega a dois deles:** medido, uma
    /// explosão ao lado de um personagem alcança **1** corpo sob Spring e
    /// **ZERO** sob Snap e Pure — quem possui a velocidade ali é o
    /// `KinematicState`, e o rapier não tem a quem entregar o impulso.
    ///
    /// ⚠️ **E ela DESCARTA o ring de checkpoints**, pela razão exacta do
    /// [`Self::explode`]: um empurrão é uma descontinuidade que a fita não
    /// grava, então todo checkpoint anterior descreve uma corrida que deixou de
    /// existir. O preço é o mesmo que uma explosão já paga — um scrub para trás
    /// replaya sem o empurrão —, e está nomeado no handoff.
    ///
    /// Um segundo empurrão no mesmo tique **substitui** a velocidade e fica com
    /// a janela MAIOR: encurtar uma janela viva devolveria o personagem à
    /// caminhada no meio de um empurrão que ainda está a acontecer.
    pub fn launch_player(&mut self, entity: Entity, velocity: [f32; 2], lock: f32) {
        let lock = lock.max(0.0);
        let e = self.player_launch.entry(entity).or_default();
        e.velocity = velocity;
        e.lock = e.lock.max(lock);
        self.ring.clear();
    }

    /// O empurrão ainda não entregue a este player (`None` = nenhum).
    #[must_use]
    pub fn pending_launch(&self, entity: Entity) -> Option<Launch> {
        self.player_launch.get(&entity).copied()
    }

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
        // ⚠️ E o EMPURRÃO por entregar (`W-Launch`), pelo primeiro dos motivos
        // acima: os bits são reciclados, então um empurrão guardado passaria a
        // atirar **outro** objeto.
        self.player_launch.clear();
    }
}

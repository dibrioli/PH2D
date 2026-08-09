//! **§14 Platform Player** — o modelo da seção de COMPORTAMENTO (W5).
//!
//! Irmão do `inspector_model_physics.rs` pelo mesmo corte que separa as duas
//! seções: a §11 responde *"que corpo é este?"* e a §14 responde *"que
//! comportamento este corpo tem?"*.

/// O que a §14 mostra sobre a entidade selecionada.
///
/// ⚠️ **`Some` para todo corpo Dynamic, com ou sem o componente** — e a face
/// VAZIA é a metade importante. A §11 do W2a já pagou esta lição: uma seção que
/// só existe onde o comportamento já existe é uma seção que ninguém alcança
/// para criá-lo.
///
/// ⚠️ **E `None` para o que não é Dynamic**, por FÍSICA e não por gosto: a mola
/// é um impulso, e um impulso não move massa infinita. Oferecer o botão num
/// corpo Static seria oferecer um gesto que a ponte recusa em silêncio.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InspectorPlayerInfo {
    pub entity_bits: u64,
    /// Esta entidade carrega `PlatformPlayer` agora?
    pub has_player: bool,

    /// **Como este player é MOVIDO** — o tag do `PlayerMode` (0 = Dynamic).
    ///
    /// ⚠️ Um `u8` e não um enum, porque é a fronteira da UI: o mapeamento
    /// tag↔variante é feito UMA vez, no `PlayerMode::tag`/`from_tag`, e um
    /// segundo `match` aqui é o que faz um chip selecionar outra coisa no dia em
    /// que a terceira opção existir (a cicatriz do `BodyKind::tag`).
    pub mode_tag: u8,
    /// **O mundo OUVE este personagem?** — a resposta de `PlayerMode::transmits`,
    /// resolvida na shell (W-KinPure).
    ///
    /// ⚠️ **Um bool, e não um `mode_tag == 2` no painel**, pela razão exata da
    /// `mass_is_read` da W-KinWeight: o painel não sabe o que é um `PlayerMode`,
    /// e um literal aqui seria a **segunda cópia** do mapeamento que o
    /// `tag()`/`from_tag()` existe para manter única — a que ninguém lembra de
    /// atualizar quando o quarto modo chegar.
    ///
    /// É ele que decide se o card REACTION é PINTADO: sob o *puro sangue* os
    /// três escalares não são lidos por ninguém, e um card inteiro de controles
    /// mortos é pior que a ausência dele.
    pub reaction_is_live: bool,

    /// **E o `Push on Bodies` é MAIS estreito que o card que o contém** — só o
    /// modo cinemático o lê (report do Enio, 2026-08-09).
    ///
    /// ⚠️ **Um segundo booleano, e não uma condição composta no pintor:** os
    /// dois vizinhos do card (*Weight on Ground* e *Push on Ground*) são VIVOS
    /// em Dynamic — quem os aplica é a mola —, então o card fica e só esta row
    /// sai. Esconder o card inteiro tiraria dois controles que funcionam.
    ///
    /// A pergunta é resolvida na shell, pela porta `PlayerMode::pushes_bodies`;
    /// o painel recebe a resposta e nunca o mapeamento.
    pub push_is_live: bool,

    /// A altura a que ele paira, metros (do centro do corpo para baixo).
    pub float_height: f32,
    /// A altura MÍNIMA que a forma deste corpo exige para de fato flutuar —
    /// `half_height + radius / cos(max_slope)`, com a tabela em
    /// `RideConfig::min_float_height`.
    ///
    /// ⚠️ **Readout, e é o que torna o defeito VISÍVEL.** O ponto de partida
    /// (`0,5`) deixa a cápsula canônica tangente ao chão — ela não paira, e só
    /// uma rampa revela. Um número que o app sabe e não mostra é um número que o
    /// artista descobre por acidente.
    pub min_float_height: f32,
    /// A forma deste corpo tem um mínimo computável? (Cápsula e bola têm; uma
    /// caixa tem OUTRA fórmula, e inventar uma para ela seria mentir.)
    pub min_float_known: bool,

    pub cling_distance: f32,
    pub spring_strength: f32,
    pub spring_damping: f32,

    pub speed: f32,
    pub acceleration: f32,
    pub air_acceleration: f32,
    pub max_slope_deg: f32,

    /// A altura de um pulo COMPLETO, metros — com gravidade neutra (W4).
    pub jump_height: f32,
    /// Multiplicador de gravidade na saída, acima de [`Self::takeoff_speed`].
    pub takeoff_gravity: f32,
    /// A velocidade acima da qual a gravidade de saída age, m/s.
    pub takeoff_speed: f32,
    /// Multiplicador perto do ápice — ⚠️ abaixo de 1 ALONGA.
    pub peak_gravity: f32,
    /// A janela do ápice, m/s.
    pub peak_speed: f32,
    /// Multiplicador na queda.
    pub fall_gravity: f32,
    /// Multiplicador enquanto sobe com o botão solto — a altura variável.
    pub cut_gravity: f32,
    /// **Coyote Time** (W8), segundos.
    pub coyote_time: f32,
    /// **Jump Buffer** (W8), segundos.
    pub jump_buffer: f32,
    /// **Corner Reach** (W10) — em METROS, ao contrário dos dois acima.
    pub corner_reach: f32,
    /// **Lift Momentum** (W10) — segundos.
    pub lift_momentum: f32,
    /// **Wall Slide** (W13) — m/s. `0` desliga a capacidade.
    pub wall_slide_speed: f32,
    /// **Wall Jump** (W13) — metros. `0` desliga.
    pub wall_jump_height: f32,
    /// **Wall Push** (W13) — m/s para LONGE da parede.
    pub wall_jump_push: f32,
    /// **Wall Lockout** (W13) — segundos de silêncio do controle aéreo.
    pub wall_jump_lockout: f32,
    /// **Wall Reach** (W13) — metros além da própria largura.
    pub wall_reach: f32,
    /// **WALL GRAB** (W23) — por quantos segundos ele segura a parede de vez.
    pub wall_grab_stamina: f32,
    /// **Dash Speed** (W14) — m/s. `0` desliga a capacidade.
    pub dash_speed: f32,
    /// **Dash Time** (W14) — segundos.
    pub dash_time: f32,
    /// **Dash Cooldown** (W14) — segundos depois do FIM.
    pub dash_cooldown: f32,
    /// **Crouch Height** (W15) — a altura de flutuação agachado, m. `0` desliga.
    pub crouch_height: f32,
    /// **Crouch Speed** (W15) — m/s agachado. ⚠️ `0` aqui NÃO desliga nada.
    pub crouch_speed: f32,
    /// Quanto do peso volta ao chao (W6).
    pub reaction_support: f32,
    /// Quanto da caminhada volta ao chao (W6).
    pub reaction_movement: f32,
    /// **Quanto de um bloqueio LATERAL volta** (W-KinPush).
    ///
    /// ⚠️ **Inerte sob Spring, e a row o DIZ:** um corpo dinâmico já empurra o
    /// que esbarra, pelo solver. É a mesma classe do `lift_momentum` (inerte em
    /// chão estático) e do `crouch_speed` (inerte sem `crouch_height`) — a §14
    /// escreve o escopo no rótulo em vez de esconder o número, porque um
    /// controle que APARECE e some é mais difícil de aprender que um que
    /// explica onde age.
    pub reaction_push: f32,

    /// **Quantos segundos de CORRIDA GRAVADA o documento carrega** (W17).
    ///
    /// ⚠️ **Zero é a resposta *"ninguém correu"*, e é ela que apaga o botão** —
    /// a ausência do controle é o outro readout. Ver
    /// [`crate::ids::INSP_PLAYER_CLEAR_RUN`].
    ///
    /// ⚠️ **É um fato do DOCUMENTO, não desta entidade**, e por isso não sai do
    /// `PlatformPlayer` como os vinte e quatro acima: a fita é uma só. Quem o
    /// preenche é a shell, que é quem a tem.
    pub recorded_run_seconds: f32,
    /// **Quantos segundos de corrida DESCARTADA esperam por um desfazer** (W24).
    ///
    /// ⚠️ **Sessão apenas, e nunca no arquivo:** uma corrida descartada foi
    /// descartada, e um arquivo que a carregasse ressuscitaria o que o artista
    /// apagou. Isto é o desfazer de um CLIQUE, não um segundo documento.
    ///
    /// ⚠️ **Ele só é OFERECIDO com a fita viva vazia**, e essa é a regra inteira
    /// do ciclo de vida — derivada, não mantida: gravar de novo enche a fita, o
    /// botão de devolver some, e o próximo descarte sobrescreve o guardado. Não
    /// existe caminho em que ele ressuscite uma corrida velha.
    pub discarded_run_seconds: f32,
}

/// Uma edição na §14 — o vocabulário que o painel emite e a shell honra.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PlayerFieldEdit {
    /// Anexa `PlatformPlayer` com o ponto de partida — **já ajustado à forma**
    /// do collider, para o personagem nascer pairando e não tangente.
    Add,
    /// Remove o componente: o corpo volta a ser um corpo comum.
    Remove,
    /// Semeia a `float_height` a partir da forma do collider (o botão
    /// **Fit to Collider**).
    FitFloatHeight,
    /// Semeia a `crouch_height` pelo MESMO piso, uma perna abaixo (o botão
    /// **Fit Crouch to Collider**, W18) — *o agachar mais fundo que este corpo
    /// consegue de fato fazer*.
    FitCrouchHeight,
    /// **Descarta a corrida gravada** (W17) — a fita de entrada do jogador.
    ///
    /// ⚠️ **O único verbo desta lista que NÃO é uma escrita de componente**, e
    /// por isso ele não passa pelo `apply_player_edit`: a fita mora na shell. Ele
    /// é interceptado no laço de ações, onde o `self` é mutável — o lugar e a
    /// razão exatos do `Join` da §11 e do eyedropper da §12.
    ClearRun,
    /// **Devolve a corrida que o `ClearRun` descartou** (W24).
    ///
    /// ⚠️ Ele existe porque descartar era **um clique irreversível**: a fita não
    /// é `ProjectState` (de propósito — um Ctrl+Z do canvas não deve rebobinar
    /// uma gravação), então sem isto o único caminho de volta era reabrir o
    /// arquivo.
    RestoreRun,

    /// **Troca o modo** (W-KinMove) — escreve o `PlayerMode` E o `RigidBody.kind`
    /// por UMA porta; ver `ids::INSP_PLAYER_MODE`.
    Mode(u8),

    FloatHeight(f32),
    ClingDistance(f32),
    SpringStrength(f32),
    SpringDamping(f32),

    Speed(f32),
    Acceleration(f32),
    AirAcceleration(f32),
    MaxSlopeDeg(f32),

    /// A altura de um pulo COMPLETO, metros (W4).
    JumpHeight(f32),
    /// Multiplicador de gravidade na saída.
    TakeoffGravity(f32),
    /// A velocidade acima da qual a gravidade de saída age.
    TakeoffSpeed(f32),
    /// Multiplicador perto do ápice — abaixo de 1 ALONGA.
    PeakGravity(f32),
    /// A janela do ápice.
    PeakSpeed(f32),
    /// Multiplicador na queda.
    FallGravity(f32),
    /// Multiplicador enquanto sobe com o botão solto — a altura variável.
    CutGravity(f32),
    /// A janela de coyote, em segundos (W8).
    CoyoteTime(f32),
    /// A janela de buffer, em segundos (W8).
    JumpBuffer(f32),
    /// **Corner Reach** (W10) — metros de escape lateral sob uma beirada.
    CornerReach(f32),
    /// **Lift Momentum** (W10) — segundos de memória do referencial do chão.
    LiftMomentum(f32),
    /// AS PAREDES (W13).
    WallSlideSpeed(f32),
    WallJumpHeight(f32),
    WallJumpPush(f32),
    WallJumpLockout(f32),
    WallReach(f32),
    /// Quantos segundos ele segura a parede de vez (W23). `0` desliga.
    WallGrabStamina(f32),
    /// O ARRANQUE (W14).
    DashSpeed(f32),
    DashTime(f32),
    DashCooldown(f32),
    /// O AGACHAR (W15).
    CrouchHeight(f32),
    CrouchSpeed(f32),
    /// Quanto do peso volta ao chao (W6).
    ReactionSupport(f32),
    /// Quanto da caminhada volta ao chao (W6).
    ReactionMovement(f32),
    ReactionPush(f32),
}

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
    /// **Descarta a corrida gravada** (W17) — a fita de entrada do jogador.
    ///
    /// ⚠️ **O único verbo desta lista que NÃO é uma escrita de componente**, e
    /// por isso ele não passa pelo `apply_player_edit`: a fita mora na shell. Ele
    /// é interceptado no laço de ações, onde o `self` é mutável — o lugar e a
    /// razão exatos do `Join` da §11 e do eyedropper da §12.
    ClearRun,

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
}

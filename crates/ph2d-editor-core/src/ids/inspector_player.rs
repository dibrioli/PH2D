//! **§14 Platform Player** — os ids da seção de COMPORTAMENTO (W5).
//!
//! ⚠️ **Seção própria, e sem SELETOR** (D9 do plano). Um seletor de um item é um
//! controle morto: ele pede uma escolha onde não há escolha, e o custo dele é
//! permanente (uma row para sempre) enquanto o benefício é hipotético. O ponto
//! de extensão de *"que comportamentos este corpo tem?"* é o **componente** —
//! um comportamento novo é uma seção nova, exatamente como um joint novo é um
//! `JointKind` novo e não um dropdown dentro do Pin.
//!
//! ⚠️ **E ela é irmã da §11, não parte dela.** A §11 responde *"que corpo é
//! este?"* (massa, forma, material) e a §14 responde *"que comportamento este
//! corpo tem?"*. Colapsá-las daria a um estado de colapso dois assuntos, e o
//! artista que fecha "Physics Body" para ver a lista de objetos perderia os
//! controles do personagem junto.

use super::hash_node_id;
use ph2d_a11y::NodeId;

/// **O cabeçalho da §14 — Platform Player** (dono do estado de colapso) e o
/// círculo de cor dele.
pub const INSP_LIVE_PLAYER_SECTION: NodeId = hash_node_id("insp_live_player_section");
pub const INSP_LIVE_PLAYER_COLOR: NodeId = hash_node_id("insp_live_player_color");

/// **O gesto que CRIA um player** — o botão da face vazia.
///
/// ⚠️ A face vazia é a metade importante da seção, e é a lição que a §11 do W2a
/// já tinha pago: sem ela o comportamento é alcançável só onde já existe, ou
/// seja em lugar nenhum. Ele aparece para qualquer corpo **Dynamic** sem o
/// componente.
pub const INSP_PLAYER_ADD: NodeId = hash_node_id("insp_player_add");
/// O gesto oposto — devolve o corpo a um corpo comum.
pub const INSP_PLAYER_REMOVE: NodeId = hash_node_id("insp_player_remove");

/// **A altura a que o personagem PAIRA**, metros, medida do centro do corpo.
pub const INSP_PLAYER_FLOAT: NodeId = hash_node_id("insp_player_float");
/// **Fit to Collider** — semeia a altura de flutuação a partir da forma.
///
/// ⚠️ Existe porque o número tem um PISO GEOMÉTRICO que ninguém adivinha: o
/// sensor mede na vertical e quem encosta na rampa é a cápsula ao longo da
/// normal dela, então flutuar exige
/// `float_height > half_height + radius / cos(max_slope)`
/// (`ph2d_platformer::RideConfig::min_float_height`, com a tabela medida). Com o
/// ponto de partida `0,5` e a cápsula canônica o personagem fica **TANGENTE** ao
/// chão — ele não paira, e a primeira rampa o revela. O botão é o mesmo idioma
/// do collider que nasce da caixa do sprite: o app sabe a resposta, então ele a
/// oferece em vez de deixar o artista descobrir por acidente.
pub const INSP_PLAYER_FIT: NodeId = hash_node_id("insp_player_fit");
/// Quanto ACIMA da altura de repouso a mola ainda age — o que separa *"subi um
/// degrau"* de *"pulei"*.
pub const INSP_PLAYER_CLING: NodeId = hash_node_id("insp_player_cling");
/// Rigidez da perna, em aceleração-por-metro.
pub const INSP_PLAYER_STIFFNESS: NodeId = hash_node_id("insp_player_stiffness");
/// Amortecimento da perna — fração da velocidade relativa removida por tick.
///
/// ⚠️ Tem TETO MEDIDO (`RideConfig::MAX_DAMPING`): acima dele o boost inverte a
/// velocidade em vez de matá-la, e o personagem pipoca.
pub const INSP_PLAYER_DAMPING: NodeId = hash_node_id("insp_player_damping");

/// Velocidade de cruzeiro, m/s — **relativa ao chão**.
pub const INSP_PLAYER_SPEED: NodeId = hash_node_id("insp_player_speed");
/// Aceleração no chão, m/s².
pub const INSP_PLAYER_ACCEL: NodeId = hash_node_id("insp_player_accel");
/// Aceleração no ar — o controle aéreo. `0` conserva o arco do salto.
pub const INSP_PLAYER_AIR_ACCEL: NodeId = hash_node_id("insp_player_air_accel");
/// A inclinação máxima em que o personagem fica de pé, em GRAUS.
///
/// Graus na fronteira, cosseno no motor — a convenção do ângulo de joint.
pub const INSP_PLAYER_MAX_SLOPE: NodeId = hash_node_id("insp_player_max_slope");

/// **O PULO** (W4) — sete números, e o único que o artista pensa é o primeiro.
///
/// ⚠️ Ids por HASH DE STRING, como os irmãos acima — um literal numérico neste
/// arquivo seria a segunda convenção do mesmo bloco.
///
/// A altura de um pulo COMPLETO, metros acima da decolagem, com gravidade
/// neutra.
pub const INSP_PLAYER_JUMP_HEIGHT: NodeId = hash_node_id("insp_player_jump_height");

/// Quanto do PESO volta para o chao (W6) -- ver `PlatformPlayer::reaction_support`.
pub const INSP_PLAYER_REACT_SUPPORT: NodeId = hash_node_id("insp_player_react_support");

/// Quanto da CAMINHADA volta para o chao (W6) -- o tapete, que nasce em zero.
pub const INSP_PLAYER_REACT_MOVEMENT: NodeId = hash_node_id("insp_player_react_movement");
/// Multiplicador de gravidade na SAÍDA, acima de [`INSP_PLAYER_TAKEOFF_SPEED`].
pub const INSP_PLAYER_TAKEOFF_G: NodeId = hash_node_id("insp_player_takeoff_g");
/// A velocidade acima da qual a gravidade de saída age, m/s.
pub const INSP_PLAYER_TAKEOFF_SPEED: NodeId = hash_node_id("insp_player_takeoff_speed");
/// Multiplicador perto do ÁPICE — ⚠️ **abaixo de 1 ALONGA** (a decisão do
/// módulo: o *forgiveness* do Celeste), acima de 1 encurta.
pub const INSP_PLAYER_PEAK_G: NodeId = hash_node_id("insp_player_peak_g");
/// A janela do ápice, m/s.
pub const INSP_PLAYER_PEAK_SPEED: NodeId = hash_node_id("insp_player_peak_speed");
/// Multiplicador na QUEDA — acima de 1 desce mais rápido do que sobe.
pub const INSP_PLAYER_FALL_G: NodeId = hash_node_id("insp_player_fall_g");
/// Multiplicador enquanto sobe com o botão SOLTO — a altura variável.
pub const INSP_PLAYER_CUT_G: NodeId = hash_node_id("insp_player_cut_g");

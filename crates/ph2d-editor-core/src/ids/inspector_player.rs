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
/// **Fit Crouch to Collider** (W18) — o espelho exato do [`INSP_PLAYER_FIT`], uma
/// perna mais curta abaixo.
///
/// ⚠️ **O defeito que ele resolve foi MEDIDO e não é o que a nota da W15 dizia.**
/// Ela previa *"o corpo enterrado"*; o corpo **não enterra — ele SATURA**. O
/// solver segura a cápsula tangente com 1 mm de folga (o
/// `normalized_allowed_linear_error`), a pose fica **perfeitamente estável**, e o
/// que acontece de verdade é o slider ficar **MORTO**: numa rampa de 45° (piso
/// `0,583`) autorar `0,50` dá folga `0,059` e autorar `0,30` dá `0,058` — duzentos
/// milímetros de curso, **um milímetro** de resposta, e nada na tela a dizer por
/// quê.
///
/// ⚠️ **Oferecido só com o agachar ARMADO** (`crouch_height > 0`): em zero a
/// capacidade está desligada e não há defeito nenhum, e um botão que a ligasse
/// pelas costas conflataria dois gestos.
pub const INSP_PLAYER_FIT_CROUCH: NodeId = hash_node_id("insp_player_fit_crouch");
/// **Descartar a CORRIDA GRAVADA** (W17) — a fita de entrada do jogador.
///
/// ⚠️ **Ele é a metade visível inteira da persistência, e a AUSÊNCIA dele é o
/// outro readout:** o botão só é oferecido quando existe corrida, e o rótulo
/// carrega quantos segundos ela tem. É o precedente do `Fit to Collider (needs >
/// 0.50 m)`, escrito uma seção acima — *o aviso mora no rótulo do próprio
/// controle que o resolve*; um readout separado seria uma segunda superfície
/// dizendo o mesmo fato.
///
/// ⚠️ E ele é GLOBAL numa seção por-entidade, porque a fita é uma só — o mesmo
/// desenho do `hand_input_to_players`, que entrega UM dedo a todos os players.
/// Quando houver um segundo dedo, os dois se movem juntos.
pub const INSP_PLAYER_CLEAR_RUN: NodeId = hash_node_id("insp_player_clear_run");
/// **Devolve a corrida DESCARTADA** (W24) — o desfazer do botão acima.
pub const INSP_PLAYER_RESTORE_RUN: NodeId = hash_node_id("insp.player.restore.run");

/// **COMO ele é movido** — o chip `Dynamic | Kinematic` (W-KinMove).
///
/// ⚠️ **Um gesto, DOIS campos.** Clicar aqui escreve o `PlayerMode` **e** o
/// `RigidBody.kind`, porque um player cinemático precisa de um corpo cinemático
/// e pedir ao artista que ponha o corpo em Kinematic noutra seção seria a falha
/// de duas-portas que este módulo já pagou. O `PlayerMode` continua a ser o
/// discriminador (um player ASSADO também tem corpo cinemático, e a cena é dona
/// da pose dele) — ver `ph2d_physics_ecs::PlayerMode`.
///
/// ⚠️ **Ele mora na §14 e não na §11**, e a distinção não é arrumação: a §11
/// responde *"que corpo é este?"* para QUALQUER objeto; esta pergunta só existe
/// para um personagem, e a resposta dela cresce (o *"puro sangue"* é o terceiro).
pub const INSP_PLAYER_MODE: NodeId = hash_node_id("insp_player_mode");
/// As opções do chip. ⚠️ **Uma fatia, nunca um par** — a `W-KinPure` trouxe a
/// terceira, e ela custou exatamente uma linha nesta tabela, como prometido.
pub const INSP_PLAYER_MODE_IDS: [NodeId; 3] = [
    hash_node_id("insp_player_mode_dynamic"),
    hash_node_id("insp_player_mode_kinematic"),
    hash_node_id("insp_player_mode_pure"),
];

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

/// Quanto de um bloqueio LATERAL volta (W-KinPush) -- o empurrao, so' sob Snap.
pub const INSP_PLAYER_REACT_PUSH: NodeId = hash_node_id("insp_player_react_push");
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
///
/// ⚠️ Este doc-comment estava **órfão** desde a W8: os dois ids do perdão foram
/// inseridos entre ele e a const que ele descreve, então ele passou a documentar
/// o *Coyote Time* (que não é multiplicador nenhum) e o `CUT_G` ficou sem doc.
/// Corrigido na W9 — *um comentário que descreve outra coisa é pior que
/// comentário nenhum*.
pub const INSP_PLAYER_CUT_G: NodeId = hash_node_id("insp_player_cut_g");

/// **Coyote Time** (W8) — segundos de perdão depois de sair do chão.
pub const INSP_PLAYER_COYOTE: NodeId = hash_node_id("insp.player.coyote");
/// **Jump Buffer** (W8) — segundos que um aperto cedo demais sobrevive.
pub const INSP_PLAYER_BUFFER: NodeId = hash_node_id("insp.player.buffer");

/// **Corner Reach** (W10) — METROS de deslocamento lateral que a assistência de
/// quina pode dar.
///
/// ⚠️ Mora no card do PERDÃO ao lado dos dois de cima, e a família é a mesma
/// (*o jogo perdoa um erro do jogador*) — mas a grandeza não: os dois primeiros
/// perdoam erros de **quando**, este perdoa um erro de **onde**. Daí a unidade
/// no rótulo.
pub const INSP_PLAYER_CORNER: NodeId = hash_node_id("insp.player.corner");
/// **Lift Momentum** (W10) — segundos em que o controle aéreo continua medindo
/// no referencial da plataforma que se deixou.
pub const INSP_PLAYER_LIFT: NodeId = hash_node_id("insp.player.lift");

/// AS PAREDES (W13) — o card próprio, ver `sections::player`.
/// **Wall Slide (m/s)** (W13).
pub const INSP_PLAYER_WALL_SLIDE: NodeId = hash_node_id("insp.player.wall.slide");
/// **Wall Jump (m)** (W13).
pub const INSP_PLAYER_WALL_JUMP: NodeId = hash_node_id("insp.player.wall.jump");
/// **Wall Push (m/s)** (W13).
pub const INSP_PLAYER_WALL_PUSH: NodeId = hash_node_id("insp.player.wall.push");
/// **Wall Lockout (s)** (W13).
pub const INSP_PLAYER_WALL_LOCK: NodeId = hash_node_id("insp.player.wall.lock");
/// **Wall Reach (m)** (W13).
pub const INSP_PLAYER_WALL_REACH: NodeId = hash_node_id("insp.player.wall.reach");
/// **WALL GRAB** (W23) — por quantos segundos ele segura a parede de vez.
pub const INSP_PLAYER_WALL_GRAB: NodeId = hash_node_id("insp.player.wall.grab");

/// **Os CARDS da §14** (W9) — os títulos que agrupam os dezenove números.
///
/// ⚠️ **Só moldura, nenhum estado.** Eles não são colapsáveis e não guardam
/// nada: o estado de colapso é da SEÇÃO (`INSP_LIVE_PLAYER_SECTION`), e dar a
/// cada card o seu seria oferecer cinco lugares onde um controle pode
/// desaparecer sem que a seção diga por quê. O id existe porque `Card` pede um —
/// e porque um dia a a11y vai querer nomear o grupo.
///
/// ⚠️ **Por que agrupar:** dezenove caixas numéricas em fila são uma lista que
/// não se lê (Enio, 2026-08-04: *"esse tanto de parâmetros juntos não fica bem"*).
/// Os cinco títulos são as cinco perguntas que a lei de fato responde — a perna,
/// a caminhada, o pulo, o perdão e o que volta ao chão —, e cada uma é um módulo
/// do `ph2d-platformer`. O agrupamento não é decoração: é o desenho do motor
/// dito na tela.
/// **O ARRANQUE** (W14) — a velocidade, a duração e a recuperação.
///
/// ⚠️ Os dois primeiros são o par que decide a **DISTÂNCIA**, que é o número
/// que o artista de facto julga; o terceiro é o espaçamento entre dois
/// arranques no chão. O que impede voar **não está aqui** — é a carga, reposta
/// pelo pé no chão, e ela não é um knob de propósito.
pub const INSP_PLAYER_DASH_SPEED: NodeId = hash_node_id("insp.player.dash.speed");
/// Quanto tempo o arranque dura, segundos.
pub const INSP_PLAYER_DASH_TIME: NodeId = hash_node_id("insp.player.dash.time");
/// Quanto tempo depois do FIM até poder de novo, segundos.
pub const INSP_PLAYER_DASH_COOL: NodeId = hash_node_id("insp.player.dash.cool");

/// **O AGACHAR** (W15) — a altura agachado e a velocidade agachado.
///
/// ⚠️ Dois números com significados DIFERENTES para o zero: zero na altura
/// desliga a capacidade; zero na velocidade é um agachar em que não se anda,
/// que é uma escolha legítima. Ver [`INSP_PLAYER_CROUCH_SPEED`].
pub const INSP_PLAYER_CROUCH_HEIGHT: NodeId = hash_node_id("insp.player.crouch.height");
/// A velocidade de cruzeiro agachado, m/s. ⚠️ Zero aqui NÃO desliga nada.
pub const INSP_PLAYER_CROUCH_SPEED: NodeId = hash_node_id("insp.player.crouch.speed");

/// **O NADO** (W-Swim) — a velocidade, a autoridade e o LIMIAR de entrada.
///
/// ⚠️ **O terceiro não é uma altura**, e é a distinção que decide se o artista
/// entende o card: ele conta **PESOS carregados pelo fluido**, então `1` diz *a
/// água sozinha me sustenta* em qualquer poça — enquanto uma altura diria coisas
/// diferentes em cada uma. Ver [`INSP_PLAYER_SWIM_ENTER`].
pub const INSP_PLAYER_SWIM_SPEED: NodeId = hash_node_id("insp.player.swim.speed");
/// Quão depressa se chega à velocidade de nado, m/s². ⚠️ É autoridade CONTRA o
/// empuxo: pouca, e o corpo boia sozinho; muita, e ele treda água parado.
pub const INSP_PLAYER_SWIM_ACCEL: NodeId = hash_node_id("insp.player.swim.accel");
/// **Quantos pesos o fluido tem de carregar para ele começar a nadar.**
///
/// ⚠️ **Só a ENTRADA usa este número** — sair é uma trava (o chão, ou estar
/// completamente fora da água), porque um limiar só faria o nadador oscilar em
/// torno dele exatamente onde o jogador tenta emergir.
pub const INSP_PLAYER_SWIM_ENTER: NodeId = hash_node_id("insp.player.swim.enter");

pub const INSP_PLAYER_CARD_LEG: NodeId = hash_node_id("insp_player_card_leg");
pub const INSP_PLAYER_CARD_WALK: NodeId = hash_node_id("insp_player_card_walk");
pub const INSP_PLAYER_CARD_JUMP: NodeId = hash_node_id("insp_player_card_jump");
pub const INSP_PLAYER_CARD_FORGIVE: NodeId = hash_node_id("insp_player_card_forgive");
pub const INSP_PLAYER_CARD_REACT: NodeId = hash_node_id("insp_player_card_react");
pub const INSP_PLAYER_CARD_WALL: NodeId = hash_node_id("insp_player_card_wall");
pub const INSP_PLAYER_CARD_DASH: NodeId = hash_node_id("insp_player_card_dash");
pub const INSP_PLAYER_CARD_CROUCH: NodeId = hash_node_id("insp_player_card_crouch");
pub const INSP_PLAYER_CARD_SWIM: NodeId = hash_node_id("insp_player_card_swim");

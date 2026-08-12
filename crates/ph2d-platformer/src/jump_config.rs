//! **OS KNOBS DO PULO** — o que o artista autora, e por quê cada número é o que é.
//!
//! ⚠️ **Módulo irmão do `jump.rs` por RESPONSABILIDADE:** ali mora *o que
//! acontece num tique* e aqui *o que o artista pode dizer sobre isso*. Os dois
//! crescem por motivos diferentes — a lei por wave, esta tabela por knob, e cada
//! knob traz a MEDIÇÃO que o escolheu —, e foi o pulo do ar (`W-MultiJump`) que
//! cruzou o teto de 700 LOC.
//!
//! ⚠️ **A LEI continua sendo lida em `jump.rs`**, e é lá que está o aviso do
//! módulo (a altura autorada, as duas escolas do ápice, a mola que cala). Este
//! arquivo é o vocabulário, não a semântica; re-exportado pelo pai, então nenhum
//! caminho de chamador muda.

/// Como o personagem PULA.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct JumpConfig {
    /// A altura de um pulo COMPLETO, metros acima do ponto de decolagem, **com
    /// gravidade neutra** (ver o aviso do módulo).
    pub jump_height: f32,

    /// **QUANTOS pulos o personagem tem DEPOIS de sair do chão** (`W-MultiJump`)
    /// — o *air actions counter* do `bevy-tnua`. `0` desliga, e desligado esta
    /// wave é **byte-idêntica** ao mundo anterior.
    ///
    /// ⚠️ **É a contagem que é o interruptor**, e não uma altura em zero: com
    /// `air_jumps = 1` e altura zero o artista teria um controle que dispara e
    /// não levanta ninguém — o knob morto que este módulo recusa em toda wave.
    ///
    /// ⚠️ **Recarrega no CHÃO, pela porta única [`JumpState::on_ground`]** — a
    /// mesma que o coyote, a memória do chão e o ARRANQUE já perguntam. Uma
    /// cópia do predicado aqui daria *"às vezes o duplo pulo não recarrega"*, o
    /// sintoma que o doc do arranque já descreve para si.
    ///
    /// **Sem teto, e o §0 é o motivo:** não há recurso — o contador é um `u32` e
    /// custa uma comparação por tique. O que a faixa do painel diz é conforto de
    /// arrasto, não limite físico.
    pub air_jumps: u32,

    /// **A altura de um pulo do AR**, metros, mesma régua do
    /// [`jump_height`](Self::jump_height).
    ///
    /// ⚠️ **METROS, não uma fração do primeiro pulo** — e a razão é que este
    /// módulo tem **três** pulos (chão, parede, ar): o da parede já é uma altura
    /// absoluta ([`crate::WallConfig::jump_height`]), então uma escala aqui faria
    /// dois falarem metros e um falar multiplicador, na mesma seção do painel.
    /// *Um artista que lê três linhas seguidas não deve ter de lembrar qual
    /// delas mudou de unidade.*
    ///
    /// O valor de partida é o MESMO do primeiro pulo — a escolha do Celeste
    /// (segundo pulo cheio); o Hollow Knight põe menos. É um fato AUTORADO
    /// separado, não uma cópia que precise seguir o de cima.
    pub air_jump_height: f32,

    /// Multiplicador de gravidade na saída, enquanto a velocidade de subida
    /// passa de [`takeoff_speed`](Self::takeoff_speed).
    ///
    /// Acima de `1` o personagem sai rápido e desacelera rápido — o *snappy* que
    /// o `bevy-tnua` chama de cura do *"painfully slow"*. `1.0` é inerte.
    pub takeoff_gravity: f32,
    /// A velocidade acima da qual a gravidade de saída age.
    pub takeoff_speed: f32,

    /// Multiplicador perto do ápice — ⚠️ **abaixo de `1` ALONGA** (a decisão do
    /// módulo), acima de `1` encurta.
    pub peak_gravity: f32,
    /// A janela do ápice: `|v_subida| ≤ isto` conta como topo.
    pub peak_speed: f32,

    /// Multiplicador na QUEDA. Acima de `1` é o padrão de todo platformer —
    /// descer mais rápido do que se sobe.
    pub fall_gravity: f32,
    /// Multiplicador enquanto SOBE com o botão já SOLTO — a altura variável.
    ///
    /// É o que faz um toque curto dar um pulo curto. `1.0` desliga a altura
    /// variável (todo pulo vai à altura cheia).
    pub cut_gravity: f32,

    /// **COYOTE TIME** (W8) — por quantos segundos depois de sair do chão o pulo
    /// ainda sai.
    ///
    /// ⚠️ **É perdão para um erro de TEMPO, não uma segunda chance:** ele é
    /// CONSUMIDO pela decolagem e só volta a encher com o pé no chão, então nada
    /// aqui dá um pulo duplo. `0.0` desliga.
    pub coyote_time: f32,

    /// **JUMP BUFFER** (W8) — por quantos segundos um aperto *cedo demais*
    /// sobrevive, esperando o pé tocar o chão.
    ///
    /// O erro simétrico do coyote: um apertou tarde, o outro cedo. `0.0`
    /// desliga.
    pub jump_buffer: f32,

    /// **CORNER CORRECTION** (W10) — quantos METROS de lado o personagem pode ser
    /// deslocado para passar raspando por baixo de uma beirada.
    ///
    /// ⚠️ É uma DISTÂNCIA, não um tempo, e é o que separa esta assistência das
    /// duas de cima: coyote e buffer perdoam erros de *quando*, esta perdoa um
    /// erro de *onde*. `0.0` desliga — e desliga também o sensor
    /// ([`crate::corner_probe_wanted`]), então não custa um raio sequer.
    pub corner_reach: f32,

    /// **QUANTAS amostras o perfil do teto varre.** Ímpar (ver
    /// [`crate::odd_samples`]); teto e preço em [`crate::MAX_CORNER_SAMPLES`].
    ///
    /// ⚠️ É a RESOLUÇÃO da beirada: o perfil erra por meia célula, e o passo vale
    /// `2·(meia_largura + alcance)/(N−1)`. O primeiro corte usava 25 e o passo
    /// saía 2,7 cm num corpo de 40 cm — um encosto de 10 cm **não era salvo** com
    /// o alcance em 12 cm. Com 65 o passo cai a 1,0 cm.
    pub corner_samples: usize,

    /// **Quantos tiques de antecedência o perfil olha.**
    ///
    /// O leque mede `rel_up · dt · lookahead`, então a quina é vista ANTES do
    /// tique em que a cabeça a alcançaria — é isto, e só isto, que torna a
    /// assistência preditiva. Ele **se escala sozinho com a velocidade**: um
    /// comprimento fixo em metros seria curto num pulo rápido e longo demais
    /// perto do ápice.
    ///
    /// ⚠️ **`0.0` é legítimo e significa *sem antecedência*** — a quina passa a
    /// ser vista no tique do contato. Não é um desligar: o vão LATERAL continua
    /// a ser varrido, e é ele que a lei usa para escolher o escape.
    pub corner_lookahead: f32,

    /// **LIFT MOMENTUM** (W10) — por quantos segundos, depois de sair do chão, o
    /// controle aéreo continua medindo a velocidade no referencial do chão que
    /// se DEIXOU.
    ///
    /// ⚠️ **Sem ele, sair de uma plataforma móvel APAGA a velocidade dela.** A
    /// caminhada mira `drive × speed` **relativo ao chão** ([`crate::walk`]), e
    /// no ar o chão vale zero — então no tique em que o pé sai de um vagão a
    /// 5 m/s o alvo salta para o referencial do MUNDO e o controle aéreo começa
    /// a frear os 5 m/s que a física dera de graça. O corpo mantém a velocidade
    /// (isso é o solver), mas a assistência trabalha contra ela.
    ///
    /// ⚠️ **A memória SEGURA o valor cheio e depois solta — ela NÃO desvanece, e
    /// a primeira versão desvanecia.** A medição derrubou o desvanecimento: com
    /// ele, um pulo de um vagão a 4 m/s avançava **1,03 m** contra os 2,67 m do
    /// voo balístico, porque o alvo caía continuamente e o controle aéreo freava
    /// o tempo todo — a assistência entregava *metade* do que o nome promete.
    /// Segurando, o mesmo pulo avança **2,67 m**, ou seja 100%.
    ///
    /// O degrau no fim da janela não é um solavanco: o que muda ali é o ALVO, e
    /// o controle aéreo é uma aceleração limitada — a velocidade converge, não
    /// salta.
    ///
    /// `0.0` é o comportamento de antes desta wave, AO BIT. E em chão ESTÁTICO a
    /// memória é `[0, 0]`, então o default ligado não move nada até existir uma
    /// plataforma que se mova.
    pub lift_momentum: f32,
}

impl JumpConfig {
    /// Um perfil de partida — ⚠️ **NÃO são defaults de produto** (a mesma nota
    /// dos irmãos [`crate::RideConfig::STARTING_POINT`] e
    /// [`crate::WalkConfig::STARTING_POINT`]).
    pub const STARTING_POINT: Self = Self {
        jump_height: 2.0,
        // ⚠️ **A capacidade nasce DESLIGADA** — o precedente do wall slide e do
        // wall jump (`WallConfig::STARTING_POINT`, os dois em zero): ligar um
        // pulo duplo por default mudaria todo player já autorado.
        air_jumps: 0,
        // ...mas a altura nasce ÚTIL, senão pôr a contagem em 1 daria um pulo de
        // zero metro e o artista concluiria que o controle não funciona.
        air_jump_height: 2.0,
        // Inerte: a saída é a da gravidade do mundo até alguém decidir o
        // contrário. Duas rows neutras, não duas rows mortas — mexer nelas faz
        // coisa, e é essa a diferença.
        takeoff_gravity: 1.0,
        takeoff_speed: 0.0,
        // ⚠️ ABAIXO de 1: o topo ALONGA. Ver o aviso do módulo.
        peak_gravity: 0.5,
        peak_speed: 1.5,
        fall_gravity: 2.0,
        cut_gravity: 4.0,
        // ⚠️ **0,1 s é a janela do Celeste** (5-6 quadros a 60 Hz), e o número
        // que a torna julgável é a DISTÂNCIA: a 5 m/s de caminhada são **0,5 m
        // além da beirada**, e a queda dentro dela é `½·g·t² = 4,9 cm` — menos
        // que um passo, e um vigésimo da altura do personagem. É por isso que
        // ela lê como *"eu ainda estava na borda"* em vez de *"pulei do ar"*.
        coyote_time: 0.1,
        jump_buffer: 0.1,
        // ⚠️ **0,12 m, e a tabela é do `measure_corner`** (2026-08-04, cápsula
        // da fixture: 0,4 m de largura, beirada 1,7 m acima da cabeça). A coluna
        // que decide é o PICO do pulo — a assistência é boa quando ele volta ao
        // que seria sem obstáculo nenhum (0,833 m):
        //
        // | encosto | pico SEM | pico COM | desvio lateral |
        // |---|---|---|---|
        // | 0,04 m | 0,784 | **0,833** | −0,052 |
        // | 0,08 m | 0,741 | **0,833** | −0,090 |
        // | 0,10 m | 0,727 | **0,833** | −0,112 |
        // | 0,12 m | 0,716 | 0,716 | 0,000 |
        // | 0,20 m (cabeça inteira) | 0,702 | 0,702 | 0,000 |
        //
        // ⚠️ **O que ele salva é `corner_reach − passo/2`**, não o alcance
        // cheio: uma amostra do perfil fala por uma célula, e meia célula é o que
        // a lei não pode afirmar (ver [`crate::CORNER_SAMPLES`]). Aqui isso é
        // 0,115 m, e a tabela concorda — 0,10 passa, 0,12 não.
        //
        // ⚠️ **E a linha que importa é a última:** com a cabeça inteira tapada o
        // pico é IDÊNTICO com e sem a assistência. Um teto continua um teto.
        corner_reach: 0.12,
        // ⚠️ Os defaults SÃO as consts de sempre — o mundo já autorado fica
        // byte-idêntico, e o que muda é só quem pode mexer neles.
        corner_samples: crate::CORNER_SAMPLES,
        corner_lookahead: crate::CORNER_LOOKAHEAD,
        // ⚠️ **1,5 s é MEDIDO contra o pulo, não estimado:** um pulo default de
        // altura cheia fica **1,45 s no ar** (pico 2,101 m,
        // `measure_how_long_a_default_jump_lasts`), então a janela cobre o pulo
        // mais longo que a config de partida produz — e nada além dele.
        //
        // A tabela que a escolheu (vagão a 4 m/s, voo de 0,67 s,
        // `measure_what_the_window_delivers`):
        //
        // | janela | avanço | fração do balístico |
        // |---|---|---|
        // | 0,00 s | 0,291 m | 11% |
        // | 0,25 s | 1,358 m | 51% |
        // | 0,50 s | 2,291 m | 86% |
        // | **0,75 s** | **2,667 m** | **100%** |
        //
        // ⚠️ **A linha de cima é o defeito que esta wave conserta:** sem memória
        // o personagem chega a 11% do que a física lhe deu — o controle aéreo
        // come o resto. E a janela acabar em vez de durar para sempre é o que
        // impede uma queda de dez segundos de guardar a velocidade de um
        // elevador que ficou lá em cima.
        lift_momentum: 1.5,
    };
}

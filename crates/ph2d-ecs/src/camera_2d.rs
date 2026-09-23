//! ⭐⭐⭐ **A CÂMERA DE JOGO — a lei pura** (TOP-20 #7, W1).
//!
//! # Porque este ficheiro existe
//!
//! Até 2026-09-09 a câmera deste app era um **recurso do editor** (`ph2d_render::Camera2d`, um
//! campo da shell): ela sabia enquadrar o que o artista está a EDITAR e não havia maneira de um
//! objecto da cena dizer *«a câmera segue-me»*. O levantamento chama a isto **«a maior lacuna do
//! PH2D e de metade da indústria»** — o Godot tem limites e suavização, o Cinemachine tem o pacote
//! inteiro, e o Cocos e o Bevy **não têm nada**.
//!
//! # ⭐⭐⭐ A lei NÃO foi inventada: ela foi MEDIDA, e a porta estava aberta
//!
//! O **Godot 4.7.2 é MIT** e está instalado nesta máquina (`/usr/bin/godot`, pacote `godot`), o que
//! pela triagem do [`CLAUDE.md §0.9`] dispensa clean-room: **porta-se, com atribuição**. E porta-se
//! **CORRENDO-O**, nunca lendo o fonte — ele foi posto a mover uma `Camera2D` sem interface
//! (`godot --headless --script`) sobre trajetórias **nossas**, e cada corrida virou uma fixtura com
//! cabeçalho de proveniência em `tests/fixtures/camera2d_godot/` (**45** delas, 9 configurações ×
//! 5 trajetórias). *Cada fixtura é um gate que ninguém teve de inventar.*
//!
//! ⚠️ **A primeira medição deu um número plausível e ERRADO** — `75,33` de trilho onde a verdade é
//! `115,2` — porque o `make_current()` corria em `_init`, antes de a árvore existir, e falhava; a
//! sonda lia então uma câmera que não era a activa. *Um arnês de oráculo tem de provar que está a
//! medir o sujeito* (a corrida boa imprime `current=true` no cabeçalho).
//!
//! # As três leis, cada uma exacta ao sexto decimal
//!
//! ```text
//!   1) ZONA MORTA   alvo_perseguido = clamp(centro, mira − dz·meia, mira + dz·meia)
//!   2) AMORTECIMENTO  centro += (alvo_perseguido − centro) · speed · dt
//!   3) LIMITES      centro = clamp(centro, min + meia, max − meia)
//! ```
//!
//! ⚠️ **A ORDEM é load-bearing e foi medida, não deduzida.** Com `dz = 0,2` (meia-janela `576 px` ⇒
//! margem `115,2`), `speed = 5`, `dt = 1/60` e um degrau de `0 → 300`, o oráculo devolve
//! **`15,400024`** no primeiro quadro. A ordem *zona morta → amortecimento* prevê `184,8/12 =
//! 15,4`; a ordem trocada preveria `184,8` — um salto. *As duas ordens compilam e só uma é a lei.*
//!
//! # ⛔⛔ E o oráculo DIVERGE — a falha está medida e NÃO é portada
//!
//! O amortecimento do Godot é um `lerp` cru cujo parâmetro é `speed · dt`, **sem cerca**. Medido
//! nesta máquina, com o degrau de `0 → 300`:
//!
//! | `speed` | `speed·dt` | o que o ORÁCULO faz |
//! |---:|---:|---|
//! | 5 | 0,083 | converge (a lei) |
//! | 60 | 1,000 | chega ao alvo num quadro |
//! | 120 | 2,000 | ⛔ **oscila para sempre**: `600 → 0 → 600 → 0` |
//! | 200 | 3,333 | ⛔ **diverge**: `1000,000061 → −1333,333740` |
//!
//! ⇒ **o factor é cravado em `1`** ([`damp_axis`]), e `1` não é um número escolhido: é o topo do
//! domínio de uma **interpolação**. Acima dele aquilo deixa de interpolar e passa a extrapolar, o
//! que é outra operação com o nome da primeira. ⚠️ **As 45 fixturas correm todas com `speed·dt < 1`**,
//! onde as duas leis concordam exactamente — a cerca não afrouxa gate nenhum, e há um gate próprio
//! a afirmar que **nós** não divergimos onde ele diverge, com os números dele lá dentro.
//!
//! # ⚠️ O que é CONFIG e o que é estado VIVO
//!
//! É a lei do [`crate::timer`], e aqui ela vale com o mesmo peso: [`GameCamera`], [`CameraFollow`] e
//! [`CameraLimits`] são **configuração** e registam-se; o centro que a câmera ocupa AGORA é
//! [`CameraRuntime`], que **não** deriva `Serialize` e por isso **não se pode registar** — a porta
//! fica fechada pelo TIPO, e não por um gate que alguém tem de lembrar. ⛔ Sem essa separação, uma
//! câmera a seguir o jogador faria **cada quadro com clique virar um passo de undo**.
//!
//! # ⚠️ O alvo é o NOME
//!
//! Como em toda referência durável deste repo (`SignalActions`, `AnchorMount`): o undo respawna
//! tudo com bits novos, então guardar um `Entity` daria uma câmera que segue o vazio depois do
//! primeiro Ctrl+Z.

use bevy_ecs::prelude::Component;
use serde::{Deserialize, Serialize};

use crate::{Entity, World};

/// A altura visível mínima, em metros. ⚠️ **Escrita e não importada** — o
/// `ph2d_render::Camera2d::ZOOM_MIN_HEIGHT_WORLD` é a mesma lei do outro lado da fronteira, e esta
/// crate não depende daquela. Há gate na shell a prender os dois números.
pub const CAMERA_MIN_HEIGHT_WORLD: f32 = 0.5; // LITERAL-PX-OK: metros, não pixels
/// A altura visível máxima, em metros. Ver [`CAMERA_MIN_HEIGHT_WORLD`].
pub const CAMERA_MAX_HEIGHT_WORLD: f32 = 100.0; // LITERAL-PX-OK: metros, não pixels

/// ⭐ **A CÂMERA como componente de cena** — o que a janela do jogo mostra.
///
/// ⚠️ **Ela não guarda posição**: a posição de uma câmera é o `Transform` da entidade que a carrega,
/// como em todo o resto da casa. *Um segundo sítio para «onde ela está» seria a segunda resposta à
/// mesma pergunta.*
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GameCamera {
    /// Altura do mundo visível, em metros. A largura sai da proporção da janela.
    pub height_world: f32,
    /// Deslocamento fixo aplicado depois de tudo — o enquadramento que o artista escolhe.
    pub offset: [f32; 2],
    /// ⭐ **Quem manda quando há várias.** A activa é a de MAIOR prioridade; empate desempata pelo
    /// [`crate::StableId`], que é a lei de ordenação determinística desta casa.
    pub priority: i32,
    /// Desligada, ela não concorre a activa. ⚠️ **Distinto de não existir**: é assim que uma cena
    /// guarda uma câmera de cutscene sem que ela roube o enquadramento.
    pub active: bool,
    /// Máscara de camadas de visibilidade (`VisibilityLayer`). `u32::MAX` = mostra tudo.
    pub cull_mask: u32,
    /// ⭐⭐⭐ **O DOLLY** (plano 24, W5) — a câmera anda em PROFUNDIDADE, em fracções da distância
    /// focal. **Nenhum motor 2D tem isto**; é a câmera multiplano da Disney, de 1937.
    ///
    /// ⚠️ **Adimensional de propósito, e é isso que fecha o bloqueador §6.1 do plano:** a lei
    /// depende só de `k` e de `d/z₀`, logo não há um `z₀` para medir nem um default para escolher.
    ///
    /// ⚠️ **O campo é o ÚLTIMO da struct** — o postcard é posicional, e acrescentar no fim é a
    /// única forma aditiva (e mesmo assim o `PROJECT_SCHEMA` sobe: sem ele um ficheiro velho seria
    /// lido errado **em silêncio**).
    ///
    /// ⛔ **`0` é a omissão e a saída é byte-idêntica** — ver [`crate::ScrollFactor::escala_do_dolly`].
    pub dolly: f32,
}

impl Default for GameCamera {
    fn default() -> Self {
        Self {
            height_world: 10.0, // LITERAL-PX-OK: metros — o mesmo enquadramento do editor
            offset: [0.0, 0.0],
            priority: 0,
            active: true,
            cull_mask: u32::MAX,
            dolly: 0.0,
        }
    }
}

/// ⭐⭐ **Quem a câmera segue** — o script que todo jogo 2D reescreve, morto por UI.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CameraFollow {
    /// O NOME do objecto seguido. **Vazio = não segue ninguém** (a câmera fica onde está).
    pub target: String,
    /// Velocidade do amortecimento por eixo, em `1/s`. ⚠️ **`0` = instantâneo**, e é o valor que
    /// reproduz o `position_smoothing_enabled = false` do oráculo.
    pub damping: [f32; 2],
    /// ⭐ **A JANELA MORTA** por eixo, em fracção da meia-janela (`0..=1`): o alvo anda aqui dentro
    /// sem mover a câmera. `0` = a câmera cola no alvo; `1` = o alvo pode ir à borda do ecrã.
    pub dead_zone: [f32; 2],
    /// ⭐ **Antecipação, em SEGUNDOS** — a câmera mira onde o alvo vai estar daqui a tanto tempo.
    ///
    /// ⚠️ **A unidade é a decisão de desenho, e ela é o que evita uma constante mágica:** o
    /// Cinemachine (que é a referência desta linha e é proprietário, logo **não portável**) exprime
    /// isto como um comprimento, e um comprimento obriga a escolher um número que só serve à
    /// velocidade em que foi afinado. *Segundos são o mesmo número em qualquer velocidade.*
    pub lookahead: [f32; 2],
    /// Deslocamento sobre o alvo, em metros — a câmera acima do carro, à frente do corredor.
    pub offset: [f32; 2],
}

impl Default for CameraFollow {
    fn default() -> Self {
        Self {
            target: String::new(),
            // ⚠️ **O default é o do oráculo** (`position_smoothing_speed = 5.0`), e não `0`: uma
            // câmera que nasce colada ao alvo lê-se como trepidação, não como precisão.
            damping: [5.0, 5.0], // LITERAL-PX-OK: 1/s, o default medido do oráculo
            dead_zone: [0.0, 0.0],
            lookahead: [0.0, 0.0],
            offset: [0.0, 0.0],
        }
    }
}

/// ⭐⭐ **Os limites da fase** — e o que se prende é a **JANELA VISÍVEL**, nunca só o centro.
///
/// ⚠️ É a diferença que o levantamento nomeia como a entrega do P0: prender o centro deixa meio ecrã
/// para lá do fim do nível.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CameraLimits {
    pub min: [f32; 2],
    pub max: [f32; 2],
}

impl Default for CameraLimits {
    fn default() -> Self {
        // ⚠️ **Uma caixa de 20×20 m, não o infinito.** Um default infinito é indistinguível de
        // «não anexei», e o artista que anexa o componente quer VER a cerca para a arrastar.
        Self {
            min: [-10.0, -10.0], // LITERAL-PX-OK: metros
            max: [10.0, 10.0],   // LITERAL-PX-OK: metros
        }
    }
}

/// ⛔ **O estado VIVO da câmera — NÃO é documento e NÃO se pode registar.**
///
/// ⚠️ **A ausência de `Serialize` é a cerca, e ela é do TIPO** — a mesma decisão, escrita pela mesma
/// razão, que o [`crate::TimerRuntime`] tomou. O `register_default` do registo exige `Serialize`,
/// então esta porta não se abre por distracção: só derivando-a de propósito. *Derivá-la por
/// conveniência faria cada quadro com clique virar um passo de undo, e nada no ecrã diria porquê.*
#[derive(Component, Clone, Debug, Default, PartialEq)]
pub struct CameraRuntime {
    /// Onde a câmera está AGORA, em metros.
    pub center: [f32; 2],
    /// ⭐⭐ **O acumulador da ZONA MORTA** — para onde o centro está a ser puxado.
    ///
    /// ⚠️ **É estado SEPARADO do centro, e a separação foi medida** (ver [`follow_step`]): fundir os
    /// dois deixa o corpus do degrau e da rampa verde e erra `1,07 px` em toda inversão do alvo.
    pub anchor: [f32; 2],
    /// Onde o alvo estava na AMOSTRA anterior — é daqui que sai a velocidade da antecipação.
    ///
    /// ⚠️ **`None` no primeiro passo, e isso é informação:** sem passo anterior não há velocidade, e
    /// inventar `0` seria dizer que o alvo está parado quando ele pode estar a 30 m/s.
    pub last_target: Option<[f32; 2]>,
    /// ⭐⭐ **A velocidade SUAVIZADA do alvo** — o estado que a antecipação exige.
    ///
    /// ⚠️ **Ela é ESTADO e não uma derivação por quadro**, e é isso que impede o solavanco: uma
    /// velocidade crua colapsa a zero no instante em que o alvo pára, e a mira recua de
    /// `v · lookahead` metros num quadro. Ver [`smooth_velocity`].
    pub velocity: [f32; 2],
    /// A câmera já foi assente uma vez? ⚠️ Sem isto, uma câmera que nasce em `(0,0)` **viaja** desde
    /// a origem até ao alvo no primeiro segundo de jogo — o `reset_smoothing()` do oráculo.
    pub settled: bool,
}

/// A meia-janela visível, em metros, dada a altura e a proporção `largura/altura` da janela.
///
/// ⚠️ **A proporção não é opcional**: a zona morta e os limites são os dois horizontais **e**
/// verticais, e num ecrã largo a meia-largura é quase o dobro da meia-altura.
#[must_use]
pub fn half_extent(height_world: f32, aspect: f32) -> [f32; 2] {
    let half_h = (height_world.max(0.0)) * 0.5;
    [half_h * aspect.max(0.0), half_h]
}

/// **A ZONA MORTA** — o ponto que a câmera de facto persegue.
///
/// A câmera só se mexe quando a mira sai da janela `±dz·meia` à volta dela. ⚠️ `dz` é **cravado em
/// `0..=1`**: acima de `1` a janela morta seria maior que o ecrã e o alvo poderia sair da vista sem
/// a câmera reagir — que é um estado que nenhum artista quer e todo painel deixa escrever.
#[must_use]
pub fn dead_zone_goal(center: [f32; 2], aim: [f32; 2], half: [f32; 2], dz: [f32; 2]) -> [f32; 2] {
    let mut out = center;
    for i in 0..2 {
        let m = half[i] * dz[i].clamp(0.0, 1.0); // CLAMP-OK: fracção da meia-janela
        // ⚠️ `m` pode ser `0` ⇒ `lo == hi` ⇒ a câmera cola no alvo. É o caso de omissão.
        out[i] = out[i].clamp(aim[i] - m, aim[i] + m); // CLAMP-OK: lo <= hi por construção
    }
    out
}

/// **O AMORTECIMENTO de um eixo** — a lei do oráculo, com o parâmetro dentro do domínio dele.
///
/// `speed <= 0` ⇒ instantâneo (o `position_smoothing_enabled = false` do oráculo).
///
/// ⚠️ **O `min(1.0)` é a divergência declarada** — ver a tabela no doc do módulo: sem ele, um
/// `speed` que o painel deixa escrever põe a câmera a oscilar para sempre ou a divergir.
#[must_use]
pub fn damp_axis(center: f32, goal: f32, speed: f32, dt: f32) -> f32 {
    if !speed.is_finite() || speed <= 0.0 || !dt.is_finite() || dt <= 0.0 {
        return goal;
    }
    let k = (speed * dt).min(1.0); // CLAMP-OK: o topo do domínio de uma interpolação
    center + (goal - center) * k
}

/// **OS LIMITES de um eixo** — prende a JANELA, e trata a caixa mais estreita que ela.
///
/// ⭐⭐ **A caixa estreita fixa a câmera no CENTRO da caixa**, e isto é MEDIDO, não escolhido: com
/// limites `[200, 500]` numa janela de meia-largura `576`, o oráculo devolve `350,000000` em todos
/// os quadros, com o alvo a passar de `80` a `780`.
///
/// ⛔ **É também a razão de este ramo existir em vez de um `clamp`**: com a caixa mais estreita que
/// a janela, `lo > hi`, e o `f32::clamp` da biblioteca padrão **entra em pânico**. *Uma sala pequena
/// derrubaria o jogo.*
#[must_use]
pub fn clamp_axis_to_limits(center: f32, half: f32, lo: f32, hi: f32) -> f32 {
    let (a, b) = (lo + half, hi - half);
    if a > b {
        // A janela não cabe entre os limites ⇒ o melhor enquadramento é o centro da caixa.
        return (lo + hi) * 0.5;
    }
    center.clamp(a, b) // CLAMP-OK: a <= b garantido pelo ramo acima
}

/// ⭐⭐⭐ **UM PASSO da câmera** — a composição das três leis, na ordem MEDIDA.
///
/// `aim` é onde a câmera quer olhar (o alvo já com `offset` e antecipação somados); `half` é a
/// meia-janela de [`half_extent`]. Devolve `(âncora, centro)`.
///
/// # ⭐⭐⭐ São DOIS estados, e isso foi medido — não deduzido
///
/// A zona morta **não** trabalha sobre o centro suavizado: ela tem **acumulador próprio** (a
/// *âncora*), e o amortecimento persegue a âncora. Com um estado só, o degrau e a rampa do corpus
/// passam — porque numa perseguição monótona a âncora fica exactamente em `mira − margem`, que é o
/// mesmo que cravar o centro — e o **vaivém** reprova em `1,066284 px`.
///
/// ⚠️ **A diferença aparece exactamente quando o alvo INVERTE**, que é o gesto mais comum de um
/// jogo de plataforma: aí o centro ainda vem a caminho, está DENTRO da janela morta, e um modelo
/// de estado único devolve *«não te mexas»* enquanto o oráculo continua a andar para a âncora que
/// ficou para trás. *Um erro de um pixel por quadro em toda mudança de direção sente-se como
/// câmera «pastosa» e não se explica olhando para o código.*
///
/// ⚠️ **Os limites entram DEPOIS do amortecimento**, e não antes: prender primeiro deixaria o
/// amortecimento a puxar para fora da cerca e a cerca a puxar de volta, todo quadro — o tremor
/// clássico na borda do nível.
#[must_use]
pub fn follow_step(
    anchor: [f32; 2],
    center: [f32; 2],
    aim: [f32; 2],
    half: [f32; 2],
    follow: &CameraFollow,
    limits: Option<&CameraLimits>,
    dt: f32,
) -> ([f32; 2], [f32; 2]) {
    let anchor = dead_zone_goal(anchor, aim, half, follow.dead_zone);
    let mut out = [
        damp_axis(center[0], anchor[0], follow.damping[0], dt),
        damp_axis(center[1], anchor[1], follow.damping[1], dt),
    ];
    if let Some(l) = limits {
        for i in 0..2 {
            out[i] = clamp_axis_to_limits(out[i], half[i], l.min[i], l.max[i]);
        }
    }
    (anchor, out)
}

/// **A velocidade CRUA do alvo**, de uma amostra à seguinte.
///
/// ⚠️⚠️ **`sample_dt` é o tempo entre as DUAS AMOSTRAS, e NÃO o passo da lei** — e a distinção é o
/// defeito que a auditoria de 2026-09-10 mediu. A ponte só vê o alvo **uma vez por quadro**, e um
/// quadro leva `ticks` passos: dividir por um passo quando passaram dois lê o **dobro** da
/// velocidade. Medido, com o herói a `8 m/s` e antecipação de `0,5 s`: a mira saltava de `+4,00 m`
/// para **`+8,00 m`** em toda moldura de dois tiques, e voltava na seguinte — `4 m` de ida e volta
/// numa vista de `17,8 m` de largura, a **22 % do ecrã**, várias vezes por segundo.
///
/// ⭐ **`None` no passo anterior devolve ZERO, e é o certo**: sem duas amostras não há velocidade,
/// e inventar uma faria a câmera antecipar no quadro em que nasce.
#[must_use]
pub fn sample_velocity(previous: Option<[f32; 2]>, target: [f32; 2], sample_dt: f32) -> [f32; 2] {
    let (Some(prev), true) = (previous, sample_dt > 0.0 && sample_dt.is_finite()) else {
        return [0.0, 0.0];
    };
    [
        (target[0] - prev[0]) / sample_dt,
        (target[1] - prev[1]) / sample_dt,
    ]
}

/// **A velocidade SUAVIZADA** — a que a antecipação de facto usa.
///
/// # ⛔⛔ Porque a crua não serve, medido
///
/// Uma velocidade tirada de duas amostras é um degrau: ela salta com o ritmo do quadro e, pior,
/// **colapsa a zero no instante em que o alvo pára**. Medido na mesma auditoria: ao largar a tecla,
/// a mira caía de `+4,00 m` para `+0,00 m` **num quadro** — a câmera dava um recuo de quatro metros
/// que ninguém pediu. *Antecipar sem suavizar é trocar um atraso por um solavanco.*
///
/// # ⭐ A constante de tempo é a PRÓPRIA antecipação, e por isso não há número novo
///
/// A pergunta que a antecipação faz é *«onde é que isto vai estar daqui a `L` segundos?»*. Estimar
/// a velocidade numa janela mais curta que `L` é medir ruído; numa mais longa é medir o passado.
/// ⇒ a taxa é `1/L`, que **degenera correctamente**: com `L = 0` não há antecipação nenhuma e a
/// suavização não tem o que fazer.
///
/// ⚠️ **Ela reusa o [`damp_axis`]** — a mesma lei do amortecimento da câmera, com o mesmo cravo no
/// topo do domínio. *Duas exponenciais escritas à mão no mesmo ficheiro divergiriam no dia em que
/// uma delas ganhasse uma cerca.*
#[must_use]
pub fn smooth_velocity(
    current: [f32; 2],
    sample: [f32; 2],
    lookahead: [f32; 2],
    sample_dt: f32,
) -> [f32; 2] {
    let mut out = current;
    for (i, o) in out.iter_mut().enumerate() {
        let l = lookahead[i];
        // ⚠️ Sem antecipação neste eixo a velocidade não é lida por ninguém — segui-la à letra
        // mantém o estado honesto sem custo, e evita um ramo que só existe para não dividir por
        // zero.
        *o = if l > 0.0 {
            damp_axis(*o, sample[i], 1.0 / l, sample_dt)
        } else {
            sample[i]
        };
    }
    out
}

/// **Onde a câmera quer olhar**, dado o alvo, a velocidade dele e a antecipação.
///
/// ⚠️⚠️ **Ela NÃO estima velocidade nenhuma, e a ausência é a cura estrutural.** A primeira redacção
/// fazia as duas coisas — estimar e aplicar — e o `dt` de que precisava para a primeira **não era**
/// o `dt` que o chamador tinha para a segunda. *Uma função que faz dois trabalhos com um argumento
/// só convida a chamada errada, e ela foi escrita.* Hoje a velocidade chega pronta
/// ([`sample_velocity`] + [`smooth_velocity`]) e este passo é uma soma.
#[must_use]
pub fn aim_at(target: [f32; 2], velocity: [f32; 2], follow: &CameraFollow) -> [f32; 2] {
    [
        target[0] + follow.offset[0] + velocity[0] * follow.lookahead[0],
        target[1] + follow.offset[1] + velocity[1] * follow.lookahead[1],
    ]
}

/// ⭐ **A câmera que MANDA** — maior prioridade, desempate pelo [`crate::StableId`].
///
/// ⚠️ **Ela não recusa várias**, pela mesma razão do [`crate::listener_of`]: recusar obrigaria a
/// cena a estar correcta para se ver alguma coisa, e uma cena a meio de ser montada não está. Quem
/// diz *«há N câmeras; esta é a que manda»* é o painel.
///
/// ⚠️ **`active == false` não concorre** — é assim que uma câmera de cutscene fica guardada na cena
/// sem roubar o enquadramento.
#[must_use]
pub fn active_camera_of(world: &mut World) -> Option<Entity> {
    world
        .query::<(Entity, &GameCamera, &crate::StableId)>()
        .iter(world)
        .filter(|(_, c, _)| c.active)
        // ⚠️ **`max_by_key` sobre `(prioridade, −id)`**: a maior prioridade ganha, e no empate ganha
        // o MENOR `StableId`. Negar o id dá o desempate certo com uma chave só — e o `StableId` é
        // `u64`, então a negação vai por `Reverse`, nunca por aritmética.
        .max_by_key(|(_, c, s)| (c.priority, std::cmp::Reverse(s.0)))
        .map(|(e, _, _)| e)
}

/// Quantas câmeras a cena tem — o número que o painel mostra quando não é `1`.
#[must_use]
pub fn camera_count(world: &mut World) -> usize {
    world.query::<&GameCamera>().iter(world).count()
}

#[cfg(test)]
#[path = "camera_2d_tests.rs"]
mod tests;

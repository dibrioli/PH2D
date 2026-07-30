//! **A RODA DA CORDA** — a roldana como ENTIDADE (ADR-0131, W-Pulley W1).
//!
//! # Por que uma entidade, e não dois campos no joint
//!
//! É o argumento do W3 desta mesma linha, palavra por palavra: *um joint guardado
//! NO corpo só pode ser um por corpo*. Uma roldana guardada **dentro** do
//! [`super::super::PhysicsJoint`] tem o teto **dois** — e quatro dos oito pedidos
//! do artista caem junto com esse teto:
//!
//! | pedido | sai de graça da entidade |
//! |---|---|
//! | nº de roldanas, em tempo real | é um `spawn` — Hierarquia, nome, delete, undo e save |
//! | selecionar e posicionar | ela **tem `Transform`**: a posição é o gizmo que já existe |
//! | motor por roldana (W2) | campo no componente dela, não um 9º campo do joint |
//! | break por centro (W2) | idem |
//!
//! ⚠️ **O `Transform.translation` É o centro** — não há um segundo campo de
//! posição para discordar dele. É a mesma escolha que o joint faz com a âncora, e
//! pelo mesmo motivo: um fato, um dono.
//!
//! ⚠️ **A corda é apontada por NOME** (`stable_name_id`), nunca por bits de
//! entidade — o undo respawna tudo com bits novos, e bits dentro dos *bytes de um
//! componente* envenenam o próprio undo (o `canonicalize` ordena pelas bytes e
//! remapeia só o `parent`). É a mesma chave que `PhysicsJoint::body_a` usa.

use ph2d_ecs::{Component, SimComponent};
use serde::{Deserialize, Serialize};

/// **De que lado a corda passa nesta roldana** — o item (7) do pedido do artista,
/// mais o escape manual ao lado dele.
///
/// O algoritmo acerta a esmagadora maioria (ponto fixo sobre a poligonal dos
/// centros; ver `ph2d_physics::world::rope_route::resolve_sides`), e a lição que a
/// linha do Flip pagou é que **um matcher automático precisa do escape manual ao
/// lado**: quando ele erra, o artista corrige em vez de reorganizar a cena até o
/// algoritmo concordar.
///
/// `Over`/`Under` são ditos como um humano olha para a tela — *a corda passa por
/// CIMA da roldana* significa que ela toca o topo dela, e portanto que o centro
/// fica ABAIXO da corda.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum WrapSide {
    /// Descoberto pela geometria. O default, e o que o artista quase nunca troca.
    #[default]
    Auto,
    /// A corda passa pelo TOPO: o centro da roldana fica abaixo dela.
    Over,
    /// A corda passa por BAIXO: o centro fica acima dela.
    Under,
}

impl WrapSide {
    /// A ordem dos chips da §13 — e a MESMA lista que o painel pinta.
    pub const ALL: [Self; 3] = [Self::Auto, Self::Over, Self::Under];

    /// O tag que atravessa a fronteira do painel, que não conhece este enum.
    #[must_use]
    pub fn tag(self) -> u8 {
        match self {
            Self::Auto => 0,
            Self::Over => 1,
            Self::Under => 2,
        }
    }

    /// A volta. ⚠️ **`None` para tag desconhecido, nunca um default silencioso** —
    /// dobrar o desconhecido no primeiro variant é o defeito que o `BodyKind` do
    /// W4 pagou: com dois variants é redundante, com o terceiro vira um chip que
    /// seleciona outra coisa.
    #[must_use]
    pub fn from_tag(tag: u8) -> Option<Self> {
        Self::ALL.get(tag as usize).copied()
    }
}

/// **Uma roldana de uma polia — componente de uma entidade própria.**
///
/// Presente numa entidade com `Transform`, ela entra na rota da corda cujo nome
/// bate com [`Self::rope`], na posição [`Self::order`].
///
/// ⚠️ **Componente novo NÃO custa bump de `PROJECT_SCHEMA`** — o blob é chaveado
/// pelo hash do nome do tipo, enquanto apendar campo a um componente existente é
/// postcard POSICIONAL. É a mesma razão pela qual a família das zonas tem sete
/// componentes em vez de uma struct com sete campos.
///
/// Config, nunca estado vivo de solver: o ângulo de giro da roda (que a wave do
/// desenho mostra) mora na PONTE, porque um ângulo serializado seria um passo de
/// undo por frame.
#[derive(Component, Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PulleyWheel {
    /// `stable_name_id` da CORDA (a entidade-joint [`JointKind::Pulley`]) a que
    /// esta roldana pertence.
    ///
    /// `0` (o default de um componente recém-anexado) não casa com nome nenhum, e
    /// isso é deliberado: uma roldana órfã é inerte, nunca uma roldana que se
    /// prende à primeira corda que aparecer.
    ///
    /// [`JointKind::Pulley`]: super::super::JointKind::Pulley
    pub rope: u64,
    /// A posição ao longo da corda, de A para B. Empates são desempatados pelo
    /// `stable_name_id` do nome — que é estável através de undo, ao contrário dos
    /// bits de entidade.
    pub order: u16,
    /// O RAIO, em metros — o pedido (3) do artista, e o número que substituiu o
    /// `ratio`.
    ///
    /// Ele muda três coisas de uma vez: por onde a corda passa (a superfície, não
    /// o centro), quanto de corda existe (o arco), e quanto a roda gira (a mesma
    /// corda por um raio maior gira menos).
    pub radius: f32,
    /// De que lado a corda passa aqui.
    pub wrap: WrapSide,
    /// **O MOTOR — esta roldana é um TAMBOR**, em radianos por segundo.
    ///
    /// Positivo **RECOLHE** (a corda encurta e o que estiver pendurado sobe);
    /// negativo **PAGA** corda. `0` é uma roldana livre, que é o que toda roldana
    /// é até alguém dizer o contrário — não há um segundo booleano *"tem motor?"*
    /// para discordar do número (a lei P7, a mesma do `∞ = off` da ruptura).
    ///
    /// ⚠️ **A grandeza é ANGULAR, e é isso que faz o diâmetro ser o CÂMBIO.** A
    /// corda anda `ω·r` — então o mesmo motor num tambor grande recolhe mais
    /// depressa, e é essa a vantagem que o `ratio` prometia e não entregava
    /// (§3 do plano). Com a corda a `v = ω·r` e a roda girando a `ω = v/r`, o
    /// desenho do giro que o W1 já faz mostra o tambor **na velocidade em que ele
    /// foi mandado girar**, sem uma segunda conta.
    pub motor_speed: f32,
    /// **Este EIXO pode ceder?** (W2.)
    ///
    /// O `∞ = off` do P7 na forma que o resto da física já usa (`break_enabled`
    /// do joint, `limits_enabled`, `motor_enabled`): um checkbox, com o número
    /// vivo embaixo dele mesmo enquanto está desmarcado. Guardar `f32::INFINITY`
    /// no número poria *"inf"* numa row numérica e obrigaria o artista a digitar
    /// um finito para ACHAR o controle.
    pub break_enabled: bool,
    /// **Em que CORPO esta roldana está montada** — o `stable_name_id` do nome
    /// dele; `0` é uma roldana pregada no cenário (W3).
    ///
    /// Montada num corpo que se move, ela é a **cadernal móvel** de uma talha, e o
    /// corpo passa a ser sustentado por DOIS ramos da mesma corda — a vantagem
    /// mecânica que o `ratio` prometia e não entregava (§3 do plano). Medido: um
    /// bloco de 2 kg equilibra com **1 kg** de contrapeso.
    ///
    /// Apontada pelo NOME, a mesma chave por que a corda aponta os corpos e por
    /// que esta roldana aponta a corda — bits de entidade são id de ALOCAÇÃO e o
    /// undo respawna tudo com bits novos.
    pub body: u64,
    /// Onde no corpo o EIXO está, no frame local dele. Ignorado com
    /// [`Self::body`] em `0`.
    ///
    /// ⚠️ **Local e nunca mundo, e o preço já foi pago uma vez:** a âncora do
    /// joint era um ponto de mundo re-derivado contra a pose viva, e mover a
    /// prancha 2 m **arrastava o pino 2 m** pelo corpo (W-AnchorFollow). Um eixo
    /// guardado em mundo caminharia pelo bloco do mesmo jeito.
    pub local: [f32; 2],
    /// **O [`Self::local`] já foi semeado?**
    ///
    /// O sentinela existe porque `[0, 0]` é um eixo legítimo (no centro do corpo)
    /// e não se distingue de *"nunca convertido"*. Uma roldana que chega com
    /// `mounted: false` tem o local derivado UMA vez, do `Transform` autorado
    /// contra a pose de REPOUSO do corpo; a partir daí um move de corpo lê o local
    /// **inalterado**, que é o fix inteiro.
    pub mounted: bool,
    /// **O que este eixo aguenta**, newtons. Ignorado com
    /// [`Self::break_enabled`] desmarcado.
    ///
    /// ⚠️ **A carga aqui NÃO é a tensão da corda.** O eixo carrega a RESULTANTE
    /// do desvio — `T·|u_saída − u_entrada|` —, então um enlace de 180° sente
    /// `2T` e uma roldana que quase não desvia a corda sente quase nada. É por
    /// isso que este número é POR RODA enquanto o da corda é um só: a tensão é
    /// uniforme, a resultante não.
    pub break_force: f32,
    /// **O SEGUNDO diâmetro deste eixo** — o raio por onde a corda SAI, em
    /// metros. `0` é uma roldana COMUM, que é o que toda roldana era até o W4.
    ///
    /// Uma roldana com dois raios é um **tambor diferencial**: a corda chega
    /// enrolando num diâmetro e sai enrolando noutro, presos ao mesmo eixo. Girar
    /// o eixo recolhe de um lado e paga do outro em proporções diferentes, e daí
    /// nasce a **vantagem mecânica contínua** — `r_entra / r_sai`.
    ///
    /// ⚠️ **É isto que substitui o `ratio` que o W1 aposentou.** Aquele campo era
    /// *"uma talha diferencial com o eixo invisível"* (§3 do plano): um número sem
    /// peça na cena. Aqui as duas circunferências são desenhadas, o artista as
    /// dimensiona, e o número **cai delas** em vez de ser digitado.
    ///
    /// ⚠️ **Não-positivo é *não é um diferencial*, nunca um erro** — a mesma regra
    /// que a geometria e a engrenagem do `RopeWheel` seguem, para que as três não
    /// possam discordar. Zero é o default de um componente recém-anexado, e é
    /// exatamente o que uma roldana comum quer dizer.
    pub radius_out: f32,
}

impl Default for PulleyWheel {
    fn default() -> Self {
        Self {
            rope: 0,
            order: 0,
            radius: Self::DEFAULT_RADIUS,
            // Uma roldana nasce COMUM: o segundo diâmetro é um segundo gesto.
            radius_out: 0.0,
            wrap: WrapSide::Auto,
            motor_speed: 0.0,
            body: 0,
            local: [0.0, 0.0],
            mounted: false,
            break_enabled: false,
            break_force: Self::DEFAULT_BREAK_FORCE,
        }
    }
}

impl PulleyWheel {
    /// **O raio de uma roldana que o gesto de criação não dimensionou**, metros.
    ///
    /// Só é usado por quem anexa o componente à mão; o gesto de criar uma polia
    /// deriva o raio da GEOMETRIA que o artista já montou (uma fração da altura
    /// que ele deu às roldanas), então ele escala com a cena. Este é o piso para
    /// quando não há geometria de onde derivar — um palmo, do tamanho dos corpos
    /// que as cenas de smoke usam.
    pub const DEFAULT_RADIUS: f32 = 0.25;

    /// **O menor raio aceito**, metros — e ele é ZERO, de propósito.
    ///
    /// Raio zero é uma roldana-PONTO, o modelo que a v1 desta wave shipou, e a
    /// rota o reproduz EXATAMENTE (é a âncora de regressão da wave), então ele não
    /// é um caso degenerado a proibir. O que este piso recusa é o raio
    /// **negativo**, que inverteria a tangente — a corda passaria a nascer do lado
    /// oposto da roda, sem nada na tela dizendo por quê.
    pub const MIN_RADIUS: f32 = 0.0;

    /// **O que um eixo aguenta quando o artista marca a caixa sem digitar nada**,
    /// newtons.
    ///
    /// Não é um teto nem um limite físico: é um ponto de partida VISÍVEL. 500 N é
    /// pouco mais que o peso de 50 kg, então a primeira coisa que o artista faz
    /// depois de marcar a caixa — pendurar algo — já tem chance de partir a
    /// roldana, que é como ele descobre que o controle funciona. O mesmo
    /// raciocínio do `DEFAULT_BREAK_FORCE` do joint.
    pub const DEFAULT_BREAK_FORCE: f32 = 500.0;

    /// Um componente que chegou de um arquivo (ou de um teste) com números que a
    /// rota não sabe usar.
    ///
    /// A porta de carga, como o `clamped()` do joint: um raio negativo inverteria
    /// a tangente e um `NaN` envenenaria a pose E o hash C9.
    #[must_use]
    pub fn clamped(self) -> Self {
        Self {
            radius: if self.radius.is_finite() {
                self.radius.max(Self::MIN_RADIUS)
            } else {
                Self::DEFAULT_RADIUS
            },
            // ⚠️ `NaN` cai em ZERO e não no default: zero quer dizer *roldana
            // comum*, que é a leitura segura de um número que não é número. Um
            // `NaN` aqui viraria uma engrenagem `NaN`, e daí o comprimento da
            // corda, a pose dos dois corpos e o hash C9.
            radius_out: if self.radius_out.is_finite() {
                self.radius_out.max(0.0)
            } else {
                0.0
            },
            // Um `NaN` no motor viraria um comprimento de corda `NaN` no primeiro
            // sub-passo, e daí a pose e o hash C9. Não há teto: um tambor rápido
            // demais é uma escolha do artista, e a corda que ele puxa é a mesma.
            motor_speed: if self.motor_speed.is_finite() {
                self.motor_speed
            } else {
                0.0
            },
            // Um limiar negativo partiria a roldana antes de a corda tocá-la, e
            // um `NaN` nunca compara verdadeiro — ou seja, seria um eixo
            // indestrutível com a caixa marcada.
            break_force: if self.break_force.is_finite() {
                self.break_force.max(0.0)
            } else {
                Self::DEFAULT_BREAK_FORCE
            },
            // Um `NaN` no eixo montado viraria um centro `NaN`, e daí uma rota
            // `NaN`, a pose de dois corpos e o hash C9. Sem clamp de faixa: um
            // eixo longe do centro do corpo é uma escolha do artista (é assim que
            // se pendura uma roldana num braço).
            local: if self.local[0].is_finite() && self.local[1].is_finite() {
                self.local
            } else {
                [0.0, 0.0]
            },
            ..self
        }
    }
}

impl SimComponent for PulleyWheel {}

/// **O artista RE-COLOCOU o eixo desta roldana** — desarma o sentinela para que o
/// próximo reconcile derive o `local` da pose que ela tem AGORA (W-Pulley W6).
///
/// ⚠️ **Sem isto o gesto é MORTO, e o modo de morrer é silencioso:** o centro de
/// uma roldana montada é **derivado** (`corpo · local`), e o `sync_mounted_wheels`
/// devolve esse número ao `Transform` a cada frame de repouso. Escrever o
/// `Transform` — pelo dot de canvas ou pela row Position — e não desarmar o
/// sentinela é escrever num campo que o frame seguinte reescreve: a alça anda com
/// o dedo e volta ao soltar, sem erro e sem aviso.
///
/// ⚠️ **É a lei do W-AnchorFollow, e a exceção que o JOINT tem aqui não vale.**
/// Lá o sentinela `anchored` re-deriva **DUAS** âncoras, então limpá-lo ao editar
/// a ponta A jogaria fora a ponta B que o artista acabou de posicionar — por isso
/// o joint passa pela porta de âncora em vez do sentinela. Uma roldana tem **UM**
/// eixo: não há segunda metade a perder, e o sentinela é a resposta certa.
///
/// Devolve `true` quando de fato desarmou. Roldana de cenário (`body == 0`) não
/// tem eixo derivado — o `Transform` dela É o centro — e a chamada é inerte, o que
/// deixa os chamadores livres de perguntar *"esta roldana está montada?"* antes.
pub fn reseat_mounted_axle(
    world: &mut bevy_ecs::world::World,
    wheel: bevy_ecs::entity::Entity,
) -> bool {
    let Some(mut w) = world.get_mut::<PulleyWheel>(wheel) else {
        return false;
    };
    if w.body == 0 || !w.mounted {
        return false;
    }
    w.mounted = false;
    true
}

/// **O artista autorou a GEOMETRIA desta roldana — re-derive o que depende dela.**
///
/// Duas coisas dependem, e as duas eram congeladas por um sentinela:
///
/// 1. **O eixo de uma roldana MONTADA** ([`reseat_mounted_axle`]) — o centro é
///    `corpo · local`, e o `sync_mounted_wheels` o reescreve todo frame.
/// 2. **O COMPRIMENTO da corda** (`PhysicsJoint::max_length`, o `L0`), semeado
///    UMA vez da rota que a montagem tem em repouso e depois congelado por
///    `anchored = true`.
///
/// ⚠️ **O (2) era o *salto explosivo*** (Enio, 2026-07-29: *"aumentar o diâmetro
/// da polia … na simulação ocorre aqueles saltos explosivos"*). A restrição é
/// `L(rota) ≤ L0`, e crescer o raio CRESCE a rota — o abraço é maior — então com
/// o `L0` parado ela nasce **violada**, e o solver come a diferença de uma vez.
/// **Medido** (`tests/measure_pulley_radius.rs`, elevador de 2 roldanas, `L0`
/// semeado em 11,9650 m):
///
/// | raio | rota | violação | maior salto num tick |
/// |---|---|---|---|
/// | 0,30 (controle) | 11,9650 | +0,0000 | 0,0817 m |
/// | 0,45 | 12,2184 | +0,2535 | — |
/// | 0,60 | 12,4851 | +0,5201 | 0,3097 m |
/// | 0,90 | 13,0581 | +1,0931 | **14,1247 m** |
/// | 1,50 | 14,3667 | +2,4018 | **50,4327 m** (a carga sai a +53 m) |
///
/// **É a MESMA doença que o W-AnchorFollow curou na âncora**, e a cura é a mesma
/// lei: *autorar re-deriva, o runtime congela*. Uma porta só, porque as duas
/// re-derivações nascem do MESMO gesto — quem chamasse metade deixaria o rig
/// estável e errado.
///
/// ⚠️ **Limpar `anchored` re-deriva TAMBÉM as âncoras**, e isso é idempotente em
/// repouso de propósito: o `sync_joint_pivots` mantém `Transform = bodyA·local_a`,
/// então a re-derivação devolve o MESMO local. Um flag próprio para o `L0` custaria
/// um campo no `PhysicsJoint` — postcard é posicional ⇒ bump de `PROJECT_SCHEMA`
/// ⇒ **todo projeto salvo recusado**, para separar dois fatos que hoje mudam
/// juntos.
///
/// Devolve `true` se algo foi re-aberto (para o chamador saber que houve efeito).
pub fn reseat_wheel_geometry(
    world: &mut bevy_ecs::world::World,
    wheel: bevy_ecs::entity::Entity,
) -> bool {
    let axle = reseat_mounted_axle(world, wheel);
    let Some(e) = rope_joint_of(world, wheel) else {
        return axle;
    };
    let Some(mut j) = world.get_mut::<crate::PhysicsJoint>(e) else {
        return axle;
    };
    if !j.anchored {
        return axle;
    }
    j.anchored = false;
    true
}

/// **De que CORDA é esta roldana?** — a resposta única.
///
/// Uma roldana aponta a corda pelo **NOME** (`stable_name_id`), a mesma chave por
/// que o reconcile a resolve — nunca por bits de entidade, que o undo troca. Dois
/// consumidores fazem esta pergunta ([`reseat_wheel_geometry`] e a §13 do
/// Inspector, que autora o raio pela fila de comandos), e duas buscas seriam duas
/// respostas capazes de discordar sobre qual joint re-abrir.
#[must_use]
pub fn rope_joint_of(
    world: &mut bevy_ecs::world::World,
    wheel: bevy_ecs::entity::Entity,
) -> Option<bevy_ecs::entity::Entity> {
    let rope = world.get::<PulleyWheel>(wheel).map(|w| w.rope)?;
    if rope == 0 {
        return None;
    }
    let mut q = world.query::<(
        bevy_ecs::entity::Entity,
        &ph2d_ecs::Name,
        &crate::PhysicsJoint,
    )>();
    q.iter(world)
        .find(|(_, n, _)| ph2d_ecs::stable_name_id(n.as_str()) == rope)
        .map(|(e, _, _)| e)
}

/// **Este eixo é uma talha de WESTON** — a corda sai pelo diâmetro grande, dá a
/// volta no que houver no meio, e **VOLTA pelo pequeno** (W-Weston).
///
/// Ausente (o caso de sempre) o eixo de dois diâmetros é um **TAMBOR**: a corda
/// troca de diâmetro no MESMO nó, os dois contatos são adjacentes, e a vantagem a
/// jusante é `R/r`. Presente, os dois contatos passam a **ABRAÇAR** o resto da
/// rota — e o quociente que sai da eliminação da rotação do eixo é outro:
/// **`R/(R−r)`**, que com uma cadernal móvel no meio dá a vantagem `2R/(R−r)`.
///
/// ⚠️ **A diferença não é o número, é de que CIRCUNFERÊNCIAS ele sai.** Vantagem
/// 32 pede `r_saída = 0,031` num tambor de `R = 0,5` (um tambor de espessura de
/// fio de cabelo) e `r = 0,469` numa Weston — quase do tamanho do irmão. É a
/// *diferença* de dois raios gordos que é pequena, e **é por isso que a máquina
/// foi inventada**: a vantagem sai de duas circunferências que o artista consegue
/// desenhar. Medições em [`ph2d_physics::world::rope_route::crossing_gear`].
///
/// ⚠️ **Sozinho não faz nada:** sem um segundo diâmetro (`radius_out > 0`) não há
/// par de contatos a formar, então o marcador é inerte — e a row não o oferece. A
/// mesma lei do [`crate::AreaFalloff`], que atenua o nada sem uma força ao lado.
///
/// ⚠️ **O RETORNO tem de ser pelo diâmetro MENOR.** Com `r = R` a máquina está
/// **travada** (a carga não anda por mais que se puxe) e com `r > R` ela
/// **inverte** — e nenhuma das duas é um orçamento `L ≤ L0` que `λ ≥ 0` saiba
/// segurar. A rota recusa o par (`axle_pair`) e os dois contatos voltam a ser
/// roldanas comuns; a **row** é quem impede o artista de chegar lá.
///
/// ⚠️ **Um MOTOR aqui é um sarilho DIFERENCIAL, e ele já funciona:** a taxa
/// `ω·R` entra no orçamento do lado do esforço, que é pesado por `1`, então a
/// carga sobe a `ω·R/(2·R/(R−r)) = ω(R−r)/2` — a cinemática exata do sarilho
/// diferencial, sem uma linha de código a mais. O contato de RETORNO não recolhe
/// (um eixo, uma rotação, um termo de recolhimento).
///
/// Vigésimo-terceiro componente registrado, e marcador em vez de campo pela
/// razão que esta linha já pagou oito vezes: o blob de um componente é postcard
/// POSICIONAL, então apendar campo na [`PulleyWheel`] seria bump de
/// `PROJECT_SCHEMA` — e um bump **recusa todo projeto já salvo**. Presença É o
/// booleano (o idioma do [`crate::Ccd`], do [`crate::LockRotation`] e do
/// [`crate::OneWayPlatform`]).
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct WestonAxle;

impl SimComponent for WestonAxle {}

impl PulleyWheel {
    /// **Esta edição muda a ROTA da corda?** — e portanto o `L0` dela.
    ///
    /// Compara só os campos que a rota consome: raio, 2º raio, ordem, lado do
    /// abraço e em que corpo o eixo está montado. `motor_speed`, a ruptura e o
    /// sentinela `mounted` ficam de fora **porque não movem a corda** — re-abrir o
    /// `L0` por causa deles seria re-derivar por um fato que não mudou nada.
    #[must_use]
    pub fn route_differs(&self, other: &Self) -> bool {
        self.radius != other.radius
            || self.radius_out != other.radius_out
            || self.order != other.order
            || self.wrap != other.wrap
            || self.body != other.body
            || self.local != other.local
    }
}

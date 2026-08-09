//! **A MOLA** — a perna da cápsula flutuante (W2).
//!
//! Módulo irmão do [`crate::walk`] pelo mesmo motivo que o resto deste repo
//! corta arquivos: **uma lei por módulo**. O pulo (W4) e a tolerância (W8) já
//! têm onde nascer sem que ninguém precise reabrir este.

use crate::{GroundSample, Motor, Vec2};

/// Os ganhos da perna.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RideConfig {
    /// A que altura o personagem paira, medida do ponto de origem do raio.
    pub float_height: f32,
    /// Quanto ACIMA da altura de repouso a mola ainda age.
    ///
    /// Dentro desta faixa a mola puxa o personagem de volta — inclusive para
    /// BAIXO. Fora dela ele está no ar, e a mola se cala: é esta distância que
    /// separa *"subi um degrau"* de *"pulei"*.
    pub cling_distance: f32,
    /// Rigidez, em aceleração-por-metro (não é N/m: a força sai na ponte).
    pub spring_strength: f32,
    /// Amortecimento, em **fração da velocidade relativa removida por tick**.
    ///
    /// ⚠️ `1.0` mata a velocidade relativa por completo num tick; acima disso
    /// ela é invertida. Ver o aviso no topo do `lib.rs` e o gate do teto.
    pub spring_damping: f32,
}

impl RideConfig {
    /// ⚠️ **O teto do amortecimento, MEDIDO** ([`tests::the_damping_ceiling_is_where_the_boost_inverts`]).
    ///
    /// Em `1.0` o boost remove exatamente a velocidade relativa. Em `2.0` ele a
    /// inverte inteira (`v → −v`), que é uma colisão perfeitamente elástica com
    /// um chão imaginário — o personagem pipoca. O teto que shipa é o ponto
    /// onde a mola ainda **converge**, e o `clamp` da porta o honra.
    pub const MAX_DAMPING: f32 = 1.0;

    /// ⚠️ **O teto da RIGIDEZ, e ele é um fato da DISCRETIZAÇÃO** — o irmão
    /// exato do [`Self::MAX_DAMPING`], medido pelo mesmo método
    /// (`measure_landing::measure_landing_against_stiffness`).
    ///
    /// Com o amortecimento no teto o boost apaga a velocidade relativa inteira a
    /// cada tique, então o que sobra do erro decai por **`1 − k·dt²`**. Em
    /// `k = 1/dt²` a razão é **zero** — a mola chega no alvo em UM passo, a
    /// resposta *deadbeat*. Acima disso ela fica **negativa**: a mola passa do
    /// alvo em vez de chegar nele.
    ///
    /// | `spring_strength` | pouso | afunda | `1 − k·dt²` |
    /// |---|---|---|---|
    /// | 3400 | 0,050 s | −0,0 cm | 0,056 |
    /// | **3600 (o teto)** | **0,033 s** | **−0,0 cm** | **−0,000** |
    /// | 4000 | 0,067 s | **2,5 cm** | −0,111 |
    /// | 5000 | 0,133 s | **8,9 cm** | −0,389 |
    ///
    /// ⚠️ **`3600` É `1/dt²` para o tique de 60 Hz deste módulo**, e o gate
    /// `the_stiffness_ceiling_is_where_the_spring_overshoots` o afirma contra a
    /// fórmula em vez de contra o literal — se o relógio mudar, ele sangra.
    pub const MAX_SPRING_STRENGTH: f32 = 3600.0;

    /// Um perfil de partida — ⚠️ **NÃO são defaults de produto.**
    ///
    /// Os números que shipam saem da varredura da wave (o molde das tabelas do
    /// `GRAB_STIFFNESS`).
    ///
    /// # ⚠️ A RIGIDEZ deixou de ser o ponto de partida (W-Landing, 2026-08-07)
    ///
    /// Ela era `400` — *"a rigidez que o `bevy-tnua` usa"* —, e o report do Enio
    /// (*"a desaceleração ao encostar no chão é muito lenta e fica
    /// artificial"*) tinha esse número como causa **inteira**: com o
    /// amortecimento no teto o pouso decai por `1 − k·dt²`, que em `400` vale
    /// `0,889` e dá **meio segundo** de escorregada.
    ///
    /// | `spring_strength` | pouso | deriva 30°/10 s | soco na jangada |
    /// |---|---|---|---|
    /// | 400 (o que shipava) | **0,500 s** | 0,0000 m | 19,6 cm |
    /// | 1200 | 0,217 s | 0,0000 m | 19,8 cm |
    /// | **2000 (o que shipa)** | **0,133 s** | **0,0000 m** | 19,9 cm |
    ///
    /// ⚠️ **As duas colunas da direita são o que torna esta a cura CERTA, e não
    /// a óbvia:** baixar o amortecimento dá o mesmo pouso e **gasta** a deriva
    /// que a W11c comprou (`0,0382 m` em `d = 0,50`); a rigidez não a toca —
    /// `0,0000` em toda a faixa, medido, e a lei publicada não tem termo em `k`.
    /// E o soco que a 3ª lei entrega a uma jangada **não muda** (+1,5%), porque
    /// o impulso total é o peso do personagem e não a pressa da perna.
    ///
    /// O amortecimento fica onde a W11c o pôs — no teto (a tabela abaixo).
    ///
    /// # ⚠️ O que a varredura de 2026-08-04 mediu sobre `spring_damping`
    ///
    /// Ela existe porque o eixo do amortecedor foi corrigido
    /// ([`super::damping_axis`]) — **antes dele, nenhum valor do knob removia a
    /// deriva de rampa** (medido: `1,0` dava 0,3276 contra 0,3295 do controle,
    /// ou seja nada). Com o eixo certo o knob passa a governá-la:
    ///
    /// | `spring_damping` | deriva parado (30°, 10 s) | quique ao pousar | peso transmitido |
    /// |---|---|---|---|
    /// | 0,25 | 0,0498 m | **199 mm** | 98% |
    /// | 0,50 | 0,0331 m | 24 mm | 95% |
    /// | 0,75 | 0,0165 m | 0 mm | 93% |
    /// | **1,00** (o que shipa) | **0,0000 m** | 0 mm | **91%** |
    ///
    /// ⚠️ **E a coluna da deriva é uma LEI, não quatro medições** — a varredura
    /// por inclinação de 05/08 (`measure_the_drift_against_the_slope`) dá
    ///
    /// ```text
    /// deriva(10 s) = 0,153 · sen θ · (1 − d)   metros
    /// ```
    ///
    /// que reproduz os 24 valores de `20°..45° × d ∈ {0,25 … 1,00}` ao quarto
    /// decimal. Duas consequências que a tabela sozinha não mostra: a deriva
    /// **cresce com a rampa** (a 40° ela é 1,29× a de 30°, que foi o que o smoke
    /// de 05/08 viu como *"um pouco mais rápido"*), e o zero do teto é
    /// **exacto em toda inclinação** — não um cruzamento calibrado a 30°, que
    /// era a suspeita que a varredura foi escrita para matar.
    ///
    /// ## ⚠️ E o eixo que ela NÃO tem é a rigidez (medido 2026-08-09)
    ///
    /// A lei acima nasceu de uma varredura do amortecimento, então *"sem termo
    /// em `k`"* era uma ausência e não um facto. Varrida a rigidez numa faixa
    /// de **64×** (`measure_the_drift_against_the_stiffness`, 100 a 6400), as
    /// colunas saem **planas a cinco decimais** (`d = 0,25` dá `0,05747` nas
    /// sete linhas; `d = 0,50` dá `0,03834`) e batem com a previsão a 0,2%.
    ///
    /// ⇒ **A resposta a *"o meu personagem deriva"* é UM knob.** Mexer na
    /// rigidez não move a deriva — ela move o *quique do pouso*
    /// (`measure_landing_against_stiffness`), que é a outra pergunta.
    ///
    /// ⚠️ Duas notas de borda, honestas: em `k = 100` o teto do amortecimento
    /// deixa **0,31 mm** em vez de zero exacto (a mola mais mole da faixa não
    /// cancela tudo), e a coluna `d = 0` tem **±1,4%** de dispersão sem
    /// tendência — é ruído do caso não-amortecido, não uma dependência.
    /// Gate: `the_drift_law_has_no_stiffness_term`.
    ///
    /// ## ⚠️ E a lei tem um TERCEIRO eixo, que esta tabela esconde (W26)
    ///
    /// A tabela é medida no `substeps = 4` do produto, e as duas colunas do meio
    /// **não escalam igual** com esse número:
    ///
    /// | | `spring_damping` | `substeps` |
    /// |---|---|---|
    /// | deriva de rampa | `∝ (1 − d)` | **`∝ 1/n`** |
    /// | quique do pouso | `∝ (1 − d)` | **INDEPENDENTE** |
    ///
    /// Medido (30°, 10 s parado; queda de 1,5 m no plano, `float_height = 0,9`):
    ///
    /// ```text
    ///   substeps   d=0.25  deriva / quique   d=0.50  deriva / quique
    ///          1       0.2299 m /  34.1 mm      0.1533 m /   4.6 mm
    ///          4       0.0575 m /  32.7 mm      0.0383 m /   1.2 mm
    ///         12       0.0194 m /  32.4 mm      0.0130 m /   0.4 mm
    /// ```
    ///
    /// ⇒ **A deriva e o quique NÃO estão soldados**, e quem os desprende é um
    /// knob que o artista já tem: baixar este número devolve o pouso, e subir os
    /// **Sub-steps** (painel de mundo, teto medido `12`) devolve a quietude. No
    /// par `d = 0,25 · n = 12` sobram **99% do quique com um terço da deriva** do
    /// default de sub-passos.
    ///
    /// ⚠️ **A tabela do `BUGS_physics.md` §7 que mede sub-passos é
    /// PRÉ-`gravity_hold`**: lá a deriva CRESCE com `n`, e ajustar a lei dela
    /// hoje leva à conclusão oposta. Gate:
    /// `platform_idle.rs::the_bounce_is_a_fact_of_the_knob_and_the_drift_is_a_fact_of_the_substeps`.
    ///
    /// ⚠️ **A tabela é a SEGUNDA, e a primeira fica registada porque a diferença
    /// entre as duas É a wave** ([`ph2d_platformer::PlayerStep::gravity_hold`]):
    /// o cancelamento da gravidade passou a ser **integrado como a gravidade**,
    /// em vez de agrupado no topo do tique. Antes disso a mesma tabela dizia
    /// `0,2476 / 0,1644 / 0,0819 / 0,0000` de deriva e `88 / 77 / 65 / **53**` %
    /// de peso — **cinco vezes mais deriva em todo valor do knob**, e o teto a
    /// custar quase metade do personagem.
    ///
    /// ⚠️ **O quique do pouso não se moveu** (196 → 199, 20 → 24 mm): a correção
    /// é de integração, não de lei, e é essa coluna que o prova — o que o artista
    /// aprovou no smoke da W6/W9 continua igual.
    ///
    /// ## Por que o peso é a coluna que decide, e como ela se calcula
    ///
    /// A perna paira **ACIMA** do pedido por um erro que cresce com o
    /// amortecimento, então o offset fica negativo e a mola empurra menos que o
    /// peso; o que ela deixa de empurrar é o que o chão deixa de sentir
    /// (`peso = (9,81 − k·erro)/9,81`, com `k` = [`Self::spring_strength`]). O
    /// erro no teto caiu de **11,50 para 2,30 mm**, e é só isso que muda a última
    /// coluna de 53% para 91%.
    ///
    /// ⚠️ **Subir a RIGIDEZ não recupera o peso** (medido, `k` de 400 a 6400): o
    /// erro de repouso cai `∝ 1/k` mas o produto `k · erro` — a força que falta —
    /// fica **constante**. Não há knob que pague as duas coisas; o que as pagou
    /// foi corrigir onde o impulso cai dentro do tique.
    ///
    /// ⚠️ **O default é o TETO, e quem o pôs lá foi o smoke de 2026-08-05** —
    /// depois de a wave `gravity_hold` tornar o teto barato. O Enio reportou a
    /// subida DUAS vezes (*"quase imperceptível"*, depois *"um pouco mais
    /// rápido"* numa rampa mais íngreme) e nunca reportou o quique; entre um
    /// personagem que anda sozinho e um pouso sem 24 mm de quique, o defeito é
    /// o primeiro e o segundo é estilo — e o knob do painel devolve o quique a
    /// quem o quiser.
    ///
    /// ⚠️ **O que tornou a escolha barata foi a wave, não o gosto:** o teto
    /// custava **metade do peso** do personagem e hoje custa **4 pontos** (95%
    /// → 91%); e o quique que ele leva já valia 24 mm, contra os 199 do
    /// `0,25` — ou seja o quique de verdade vive no fundo do knob, não aqui.
    pub const STARTING_POINT: Self = Self {
        float_height: 0.5,
        cling_distance: 0.25,
        spring_strength: 2000.0,
        spring_damping: 1.0,
    };

    /// ⚠️ **A `float_height` MÍNIMA de uma cápsula, e ela é GEOMETRIA, não
    /// gosto** — o número que a W3 mediu e que a config de partida não conhece.
    ///
    /// O sensor mede na **vertical**, mas quem encosta na rampa é a cápsula ao
    /// longo da **normal** dela. Numa inclinação `θ` a folga perpendicular vale
    /// `float_height · cos θ` e a cápsula ocupa `radius + half_height · cos θ`,
    /// então flutuar de verdade exige
    ///
    /// ```text
    ///     float_height  >  half_height + radius / cos(max_slope)
    /// ```
    ///
    /// ## A tabela MEDIDA (cápsula `half_height = 0,3`, `radius = 0,2`)
    ///
    /// | `float_height` | inclinação máxima em que ela ainda flutua |
    /// |---|---|
    /// | **0,5** (o ponto de partida) | **NENHUMA** — ela fica tangente já no plano |
    /// | 0,7 | 60,0° |
    /// | 0,9 | 70,5° |
    /// | 1,2 | 77,2° |
    ///
    /// ⚠️ **A primeira linha é a que importa:** com o `STARTING_POINT` e essa
    /// cápsula o personagem **não paira** — ele fica encostado, e a mola passa a
    /// disputar com o solver de contato uma rampa que ela deveria só sobrevoar.
    /// A caminhada num plano ainda funciona (foi o que o W2 mediu), e é por isso
    /// que o defeito só apareceu quando a rampa entrou.
    ///
    /// A cura de PRODUTO é semear a `float_height` a partir do collider, como o
    /// collider já nasce da caixa do sprite — item nomeado para a W5, onde a
    /// autoria mora. Esta função é a porta única daquele número.
    ///
    /// ⚠️ **É a fórmula de uma CÁPSULA.** Uma caixa tem a sua
    /// (`half_y + half_x · tan θ`), porque a extensão dela ao longo da normal
    /// não é isotrópica. Quando a segunda forma precisar do número, ela ganha o
    /// próprio braço aqui — nunca uma segunda cópia noutro arquivo.
    #[must_use]
    pub fn min_float_height(half_height: f32, radius: f32, max_slope_cos: f32) -> f32 {
        if max_slope_cos <= 0.0 {
            return f32::INFINITY;
        }
        half_height + radius / max_slope_cos
    }
}

/// **O EIXO do amortecedor: a NORMAL do chão**, com o `up` como recuo.
///
/// # ⚠️ Por que não é o `up`, e por que a premissa envelheceu
///
/// A perna segura uma folga **perpendicular à superfície**; o que ela precisa
/// amortecer é a taxa com que o corpo se APROXIMA dessa superfície, e essa taxa
/// mede-se ao longo da normal. Amortecer o `up` é correto **no plano, onde a
/// normal É o `up`** — e era essa a única fixture quando a lei nasceu.
///
/// Numa rampa as duas deixam de coincidir, e o preço é um **modo marginal**: a
/// caminhada remove só a componente ao longo da TANGENTE e o amortecedor só a
/// componente VERTICAL. Uma velocidade ao longo da normal tem projeção **zero**
/// na tangente (a caminhada é cega a ela) e a parte vertical dela é exatamente
/// o que a mola precisa para segurar a altura (o amortecedor não pode removê-la
/// sem largar o personagem). Ninguém a remove, e o personagem **desliza rampa
/// acima para sempre** — medido, 30°, parado: `v = (0,0332, −0,0575)`, que é
/// perpendicular à tangente **ao quarto decimal**, com o freio da caminhada a
/// calcular um empurrão de `−8,7e−8` porque não há nada que ele consiga ver.
///
/// Com a normal, `{tangente, normal}` é uma base **ORTONORMAL**: as duas leis
/// juntas cobrem o plano inteiro e não sobra direção em que uma velocidade se
/// esconda.
///
/// ⚠️ **No plano isto é byte-idêntico** e não por aproximação: a normal de uma
/// face horizontal é `[0, 1]` exata, `sqrt(1.0)` é `1.0` exato, e a divisão por
/// um é a identidade — o eixo devolvido É o `up`, bit a bit.
///
/// ⚠️ **Normal degenerada recua para o `up`**, a mesma política que a
/// [`crate::footing`] já toma (*"não sabemos a orientação; a suposição menos
/// daninha é chão plano"*). Um raio nascido dentro da geometria reporta `[0,0]`,
/// e normalizar isso daria `NaN` no eixo de um amortecedor.
#[must_use]
pub fn damping_axis(normal: Vec2, up: Vec2) -> Vec2 {
    let len2 = normal[0] * normal[0] + normal[1] * normal[1];
    if !len2.is_finite() || len2 <= 0.0 {
        return up;
    }
    let len = len2.sqrt();
    [normal[0] / len, normal[1] / len]
}

/// A metade DISTÂNCIA da pergunta *"estou no chão?"* — o sensor achou algo ao
/// alcance da perna?
///
/// ⚠️ **Não é a pergunta inteira**, e é por isso que ela mudou de nome nesta
/// wave: uma parede de 80° está ao alcance do raio e **não é chão**. A pergunta
/// completa é [`crate::footing`], que é quem os dois laws consomem — esta aqui
/// é a camada de dentro, e existe para que a mola nunca aja sobre uma amostra
/// longe, seja qual for o caminho pelo qual ela chegou.
#[must_use]
pub fn within_reach(cfg: &RideConfig, sample: Option<&GroundSample>) -> bool {
    match sample {
        Some(s) => s.distance <= cfg.float_height + cfg.cling_distance,
        None => false,
    }
}

/// **A MOLA.** O termo que faz o personagem pairar.
///
/// - `sample`: o chão que a lei ACEITA (o resultado de [`crate::footing`]).
/// - `body_velocity`: a velocidade do corpo, em mundo.
/// - `gravity`: a gravidade do mundo (m/s²), que a mola **compensa** enquanto
///   segura o personagem — sem isso ela teria de vencer o peso com o próprio
///   erro, e o personagem pairaria mais baixo do que a `float_height` pede,
///   por uma quantidade que depende da gravidade.
/// - `up`: a direção "para cima" (normalmente `[0, 1]`).
///
/// No ar devolve [`Motor::default`] — zerado. A gravidade extra de queda é do
/// pulo (W4), não da perna: são duas perguntas, e misturá-las faria a perna
/// mudar a altura do salto.
#[must_use]
pub fn ride_spring(
    cfg: &RideConfig,
    sample: Option<&GroundSample>,
    body_velocity: Vec2,
    gravity: Vec2,
    up: Vec2,
) -> Motor {
    let Some(s) = sample else {
        return Motor::default();
    };
    if !within_reach(cfg, Some(s)) {
        return Motor::default();
    }

    // Positivo = está BAIXO demais e a mola empurra para cima; negativo = está
    // alto demais dentro do `cling_distance` e ela puxa para baixo.
    let offset = cfg.float_height - s.distance;

    // ⚠️ Tudo RELATIVO ao chão: sobre uma plataforma que sobe, a velocidade do
    // corpo já contém a dela, e amortecer contra o referencial do mundo faria a
    // mola lutar contra a plataforma em vez de contra a oscilação.
    //
    let rel = [
        body_velocity[0] - s.ground_velocity[0],
        body_velocity[1] - s.ground_velocity[1],
    ];
    // ⚠️ **O eixo do amortecedor é a NORMAL, o da mola é o `up`, e são perguntas
    // DIFERENTES** — ver [`damping_axis`] para o eixo e o aviso abaixo para o
    // porquê de a mola ficar onde está.
    let axis = damping_axis(s.normal, up);
    let rel_along_axis = rel[0] * axis[0] + rel[1] * axis[1];

    let damping = cfg.spring_damping.clamp(0.0, RideConfig::MAX_DAMPING);
    let spring = offset * clamped_strength(cfg);

    Motor {
        // ⚠️ **A MOLA fica no `up`, e isso é decisão, não esquecimento.** Ela
        // responde *"a que altura eu pairo?"*, e a altura é medida por um raio
        // VERTICAL — a pergunta é vertical porque o sensor é. E o `− gravity`
        // é o que faz uma rampa CAMINHÁVEL não escorregar: empurrar ao longo da
        // normal cancelaria só a componente normal do peso e o personagem
        // escorregaria por toda ladeira, que é o oposto do que o `max_slope_deg`
        // significa (o `slopeLimit` da Unity, o `floor_max_angle` do Godot).
        accel: support_accel(spring, gravity, up),
        // O amortecimento é BOOST — ver o aviso do `lib.rs`.
        boost: [
            -axis[0] * rel_along_axis * damping,
            -axis[1] * rel_along_axis * damping,
        ],
    }
}

/// **A rigidez que a lei de facto usa** — o gêmeo do clamp do amortecimento.
///
/// ⚠️ **Uma porta, dois leitores:** o motor do personagem e o que o CHÃO sente
/// ([`ride_support_on_ground`]) têm de concordar sobre o número, senão a 3ª lei
/// devolve uma força que a perna não fez.
#[must_use]
fn clamped_strength(cfg: &RideConfig) -> f32 {
    cfg.spring_strength
        .clamp(0.0, RideConfig::MAX_SPRING_STRENGTH)
}

/// **A aceleração de uma perna** — a PORTA de um só termo, com dois leitores.
///
/// `push` é o termo da mola (`k·x`), e o `− gravity` é o peso que a perna
/// cancela. Os dois leitores diferem em **uma** coisa nomeada: o motor do
/// personagem passa o `push` como ele é, e o que o CHÃO sente passa-o
/// clampado ([`ride_support_on_ground`]).
#[must_use]
fn support_accel(push: f32, gravity: Vec2, up: Vec2) -> Vec2 {
    [up[0] * push - gravity[0], up[1] * push - gravity[1]]
}

/// **O QUE SEGURA O PERSONAGEM** — a escolha que separa os dois modos, lida
/// **uma vez, dentro da lei** (K3 do plano 07).
///
/// # ⚠️ Por que esta pergunta é da LEI e não da ponte
///
/// A alternativa seria a ponte SUBTRAIR do `accel` as parcelas que dependem de
/// o corpo ser dinâmico — uma **enumeração**, e enumeração apodrece no dia em
/// que uma força nova entra no fold. Aqui o valor já existe isolado e com nome
/// próprio (`let spring = ride_hold(…)`), então a escolha é uma linha.
///
/// ⚠️ **E ela governa DUAS respostas, não uma:** o que segura o personagem
/// ([`ride_hold`]) e o que o chão sente disso ([`ride_support_on_ground`]). As
/// duas perguntam ao MESMO valor — se divergissem, existiria um tique em que o
/// personagem é segurado por uma mola que o chão não sente, ou o contrário.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Support {
    /// A **cápsula flutuante**: uma mola segura o personagem, o solver resolve
    /// o contato, e o mundo pode empurrá-lo de volta. O que shipa.
    #[default]
    Spring,
    /// O personagem é **colocado** onde tem de estar, e a mola cala.
    ///
    /// ⚠️ **Não existe força vertical nenhuma neste modo** — o que mantém a
    /// altura é a translação escrita no corpo. É por isso que a `gravity_hold`
    /// não pode reclamar um cancelamento aqui: não há o que cancelar.
    Snap,
}

impl Support {
    /// **Uma MOLA segura este personagem?** — a porta única da pergunta.
    ///
    /// Ela tem três consumidores dentro da lei (o termo do motor, o que o chão
    /// sente, e a declaração da `gravity_hold`), e é por serem três que ela é
    /// uma função em vez de um `matches!` repetido.
    #[must_use]
    pub fn is_spring(self) -> bool {
        matches!(self, Self::Spring)
    }
}

/// **O que segura o personagem neste tique** — a porta única do §3.
///
/// Sob [`Support::Spring`] é o [`ride_spring`] verbatim; sob [`Support::Snap`]
/// é **nada**, porque a altura passa a ser escrita e não empurrada.
#[must_use]
pub fn ride_hold(
    support: Support,
    cfg: &RideConfig,
    sample: Option<&GroundSample>,
    body_velocity: Vec2,
    gravity: Vec2,
    up: Vec2,
) -> Motor {
    if !support.is_spring() {
        return Motor::default();
    }
    ride_spring(cfg, sample, body_velocity, gravity, up)
}

/// **O que o CHÃO sente da perna** — o peso, e só a metade que EMPURRA.
///
/// # ⚠️ Por que isto não é o [`ride_spring`] outra vez
///
/// A mola tem duas metades e só uma existe fora do modelo. Comprimida
/// (`offset > 0`) ela **empurra**, e o chão sente — é a 3ª lei, e é o que faz
/// uma jangada afundar. Esticada dentro do `cling_distance` (`offset < 0`) ela
/// **puxa o personagem para baixo**, que é o truque que o mantém colado ao
/// descer uma lomba — e transmitir a reação DISSO puxa o chão para cima.
///
/// Medido antes desta porta existir: um personagem a cair numa jangada a fazia
/// **subir 96,9 mm ao encontro dele** antes de a empurrar para baixo, porque na
/// faixa de cling o termo da mola chega a `0,25 × 400 = 100 m/s²` — dez vezes a
/// gravidade, e para o lado errado.
///
/// ⚠️ **O peso NÃO é clampado, e essa é a metade que decide a correção.** Em
/// repouso o personagem converge para a altura de flutuação **por cima**
/// (medido: `0,9023` contra `0,900`), ou seja o `offset` de repouso é levemente
/// NEGATIVO para sempre. Zerar o suporte inteiro quando ele é negativo mataria a
/// 3ª lei no caso mais comum que existe — alguém **parado** em cima de algo — e
/// o gate `a_raft_still_sinks_under_a_player_that_stands_on_it` é quem recusa
/// essa cura.
///
/// O `boost` sai zerado porque o [`crate::react::reaction`] já o ignora para o
/// suporte: um amortecedor devolvido ao chão é o mesmo laço com outro nome.
#[must_use]
pub fn ride_support_on_ground(
    support: Support,
    cfg: &RideConfig,
    sample: Option<&GroundSample>,
    gravity: Vec2,
    up: Vec2,
) -> Motor {
    let Some(s) = sample else {
        return Motor::default();
    };
    if !within_reach(cfg, Some(s)) {
        return Motor::default();
    }
    // ⚠️ **Sob Snap o chão sente o PESO, e só ele** (K6 do plano 07) — não há
    // mola a comprimir, então não há termo de mola a transmitir. E repare que a
    // fórmula não ganha um caso especial: `push = 0` já **é** o peso, porque o
    // `support_accel` sempre carregou o `− gravity` ao lado do empurrão. É isso
    // que faz a 3ª lei sobreviver ao modo sem uma segunda função.
    let push = if support.is_spring() {
        ((cfg.float_height - s.distance) * clamped_strength(cfg)).max(0.0)
    } else {
        0.0
    };
    Motor {
        accel: support_accel(push, gravity, up),
        boost: [0.0, 0.0],
    }
}

#[cfg(test)]
#[path = "ride_tests.rs"]
mod tests;

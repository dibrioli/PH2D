//! **A família das ZONAS** — o que uma ÁREA faz aos corpos que estão dentro dela.
//!
//! Irmão do [`super::overrides`] e o corte é o que os dois já vinham fazendo sozinhos: lá
//! mora *o que este CORPO é* (a gravidade dele, os eixos travados, a massa dele); aqui,
//! *o que esta ÁREA faz a OUTROS corpos* — um vento, um redemoinho, uma poça. Nasceu do
//! cap de 700 LOC quando o falloff (W-AreaFalloff) foi o sétimo componente da mesma área,
//! e a família continua crescendo: a próxima wave de zona chega aqui, não lá.
//!
//! Todos seguem o idioma do irmão — **ausente é o default do motor**, um marcador vale
//! pela PRESENÇA, um valuado DESTACA no neutro, e um componente novo **não custa bump de
//! `PROJECT_SCHEMA`** (é chaveado pelo hash do próprio nome de tipo, enquanto apendar
//! campo é postcard POSICIONAL). É essa última linha que explica por que a área tem sete
//! componentes em vez de uma struct com sete campos: cada campo teria RECUSADO todo
//! projeto já salvo.
//!
//! E todos compartilham um acoplamento: **só mordem num collider SENSOR**. A narrow phase
//! registra sobreposição apenas quando um dos lados é sensor, e um collider sólido empurra
//! os corpos para FORA em vez de deixá-los entrar — uma área em que não se entra não é uma
//! área. As rows da §11 são oferecidas sob exatamente essa condição.
//!
//! Config, nunca estado vivo de solver.

use ph2d_ecs::{Component, SimComponent};
use serde::{Deserialize, Serialize};

/// **Force zone — an optional presence-override component (W-Area).**
///
/// Absent (the common case) is an ordinary body that pushes nothing, exactly what
/// every body did before this existed. Present, `force` is a constant force in
/// **newtons** (world axes) applied every substep to every DYNAMIC body overlapping
/// this collider: wind, an updraft, a conveyor, a current (Unity's `AreaEffector2D`,
/// Godot's `Area2D` overrides).
///
/// **A force, not an acceleration** — the impulse is resisted by the body's mass, so
/// a leaf is carried by a wind a crate barely feels. That asymmetry is the feature,
/// and it is the half that cannot be authored per body: an *acceleration* zone would
/// be a second answer to what [`GravityScale`] already says about one body.
///
/// ⚠️ **It only bites when the collider is a SENSOR** (`Collider::is_sensor`): the
/// narrow phase records an overlap only when one side is a sensor, and a solid
/// collider pushes bodies OUT rather than letting them in — an area you cannot enter
/// is not an area. The §11 Force rows are offered only for a sensor for that reason,
/// which makes this the first control in the section gated on another CONTROL rather
/// than on the body kind.
///
/// Same idiom as [`GravityScale`]/[`DampingOverride`] — an override most bodies do not
/// carry — so it is a newly registered component keyed by its type-name hash (**no
/// `PROJECT_SCHEMA` bump**), and the Inspector DETACHES it at neutral
/// ([`Self::is_neutral`]). The bridge folds it into `BodyDesc.effector`, so it rides
/// the spawn recipe the world rebuilds from and a rewind re-arms it. Config, never
/// live solver state, like every component here.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AreaEffector {
    /// The force in newtons, world axes. `[0, 0]` is neutral (see [`Self::is_neutral`]).
    pub force: [f32; 2],
}

impl AreaEffector {
    /// Is this the neutral value — a zone that pushes nothing? A zero force on both
    /// axes. Used to DETACH so a body with no push carries no component.
    #[must_use]
    pub fn is_neutral(self) -> bool {
        self.force[0] == 0.0 && self.force[1] == 0.0
    }
}

impl SimComponent for AreaEffector {}

/// **Area drag — the medium half of a force zone (W-AreaDrag).**
///
/// Absent (the common case) is an area that offers no resistance. Present, its `f32`
/// is a drag coefficient applied every substep to every DYNAMIC body overlapping this
/// collider: the difference between **wind** and **water**.
///
/// It is the same law as the world's default drag and the per-body [`DampingOverride`]
/// (`v /= 1 + d·dt`), so "drag" means one thing everywhere — and it damps **both** the
/// linear and the angular velocity, because a medium resists a spin too.
///
/// ⚠️ **Its own component rather than a field on [`AreaEffector`], deliberately.** The
/// wrapper bundles the two into one `AreaEffect` (that side is not serialized), but a
/// component's blob is postcard, which is POSITIONAL: appending a field to
/// `AreaEffector` would be a `PROJECT_SCHEMA` bump, and a bump **refuses every project
/// file already saved at the old number** — throwing away real work to avoid a second
/// component. A newly registered component is keyed by its own type-name hash and is
/// purely additive, so every existing file still loads. The two are independently
/// meaningful anyway: a wind that does not slow you, a pool of syrup that does not push.
///
/// Same coupling as its sibling: it only bites when the collider is a **SENSOR**, and
/// the §11 row is offered under exactly that condition. It rides the `BodyDesc` the
/// world rebuilds from, so a rewind re-arms it. Config, never live solver state.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AreaDrag(pub f32);

impl AreaDrag {
    /// Is this neutral — an area that resists nothing? Used to DETACH so a zone with
    /// no medium carries no component.
    #[must_use]
    pub fn is_neutral(self) -> bool {
        self.0 <= 0.0
    }
}

impl SimComponent for AreaDrag {}

/// **Empuxo de área — a densidade do FLUIDO (W-Buoyancy).**
///
/// Ausente (o caso comum) é uma área sem empuxo. Presente, seu `f32` é a densidade do
/// fluido em kg/m² (2D), e cada corpo submerso recebe `ρ·|g|·A_submersa` para cima,
/// aplicada no centroide da parte submersa.
///
/// ⚠️ **Não é uma `AreaEffector` para cima, e a diferença é o motivo da wave.** Uma
/// força constante não sabe onde a superfície está (o corpo leve é arremessado para fora
/// da piscina em vez de parar na linha d'água), é vencida pela MASSA e não pela densidade
/// (o número certo muda para cada objeto, quando a intuição é *madeira boia, pedra
/// afunda* — propriedade do MATERIAL), e não endireita nada. Arquimedes resolve os três
/// com um número só, e ele é **comparável ao `density` do `Collider`**: menor que ele o
/// corpo afunda, maior ele boia.
///
/// Terceiro componente da mesma área, pela terceira vez pela mesma razão: um blob de
/// componente é postcard POSICIONAL, então apendar campo no [`AreaEffector`] seria bump
/// de `PROJECT_SCHEMA` — e um bump **recusa todo projeto já salvo**. Mesmo acoplamento
/// dos irmãos: só morde num collider **SENSOR**, e a row da §11 é oferecida sob
/// exatamente essa condição. Config, nunca estado vivo.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AreaBuoyancy(pub f32);

impl AreaBuoyancy {
    /// É neutro — uma área sem empuxo? Usado para DESTACAR, como os irmãos.
    #[must_use]
    pub fn is_neutral(self) -> bool {
        self.0 <= 0.0
    }
}

impl SimComponent for AreaBuoyancy {}

/// **Arrasto de FORMA — a resistência que sabe para onde o corpo aponta (W-FormDrag).**
///
/// Ausente é uma área sem resistência de forma. Presente, cada aresta do corpo virada
/// para o escoamento é empurrada ao longo da própria normal, o que dá duas coisas que o
/// [`AreaDrag`] uniforme não pode dar: **resistência por secção** (o mesmo tronco sofre
/// 4× mais de través que de proa) e **freio de rotação pela FORMA** (um tronco comprido
/// resiste a girar muito mais que uma bola de mesma área).
///
/// Coexiste com o `AreaDrag` porque são mecanismos diferentes e ambos existem na
/// natureza: *Drag* é viscosidade, *Shape Drag* é resistência de forma. Quarto
/// componente da mesma área, pela quarta vez pela mesma razão (campo novo = bump de
/// `PROJECT_SCHEMA`, e um bump recusa todo projeto salvo). Só morde num SENSOR.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AreaFormDrag(pub f32);

impl AreaFormDrag {
    /// É neutro? Usado para DESTACAR, como os irmãos.
    #[must_use]
    pub fn is_neutral(self) -> bool {
        self.0 <= 0.0
    }
}

impl SimComponent for AreaFormDrag {}

/// **Torque de área** — o análogo ROTACIONAL do [`AreaEffector`] (W-AreaTorque).
///
/// Ausente (o caso comum) é uma área que não gira nada. Presente, seu `f32` é um torque
/// em N·m aplicado a cada sub-passo a todo corpo DINÂMICO que sobrepõe este collider: um
/// redemoinho, uma mesa giratória, uma esteira que gira. Enquanto o `AreaEffector` empurra
/// pelo centro de massa (não gira nada), este é a metade que FAZ girar.
///
/// ⚠️ **Um torque, resistido pelo MOMENTO DE INÉRCIA** — não uma aceleração angular. Um
/// tronco comprido gira mais devagar que uma bola de mesma área, exatamente como a folha
/// voa e o caixote não sob a `force`: a forma resiste ao giro como a massa resiste à
/// translação. Uma zona de *aceleração* seria independente da forma e uma segunda porta
/// para o que a inércia já responde.
///
/// ⚠️ **O sinal é o SENTIDO** (`> 0` anti-horário, `< 0` horário), então o neutro é
/// `== 0.0` — diferente dos irmãos de arrasto, cujo neutro é `<= 0.0` (arrasto negativo
/// adicionaria energia). Um torque negativo é uma direção, não um valor inválido.
///
/// Quinto componente da mesma área, pela quinta vez pela mesma razão: um blob de
/// componente é postcard POSICIONAL, então apendar campo no [`AreaEffector`] seria bump
/// de `PROJECT_SCHEMA` — e um bump **recusa todo projeto já salvo**. Mesmo acoplamento dos
/// irmãos: só morde num collider **SENSOR**, e a row da §11 é oferecida sob exatamente essa
/// condição. Config, nunca estado vivo.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AreaTorque(pub f32);

impl AreaTorque {
    /// É neutro — uma área que não gira nada? Um torque de exatamente zero. Usado para
    /// DESTACAR, mas com `== 0.0`: um torque negativo é um SENTIDO (horário), não um
    /// valor a descartar como o arrasto negativo dos irmãos.
    #[must_use]
    pub fn is_neutral(self) -> bool {
        self.0 == 0.0
    }
}

impl SimComponent for AreaTorque {}

/// **Prender a força da área aos eixos de MUNDO** (W-AreaFrame).
///
/// Sua **presença É o booleano**, como [`Ccd`]/[`LockRotation`]/[`OneWayPlatform`]. Ausente
/// (o default), a [`AreaEffector`] é autorada no frame da ZONA e **girar o sensor gira o
/// vento** — uma esteira diagonal é uma esteira que você virou. Presente, a direção fica
/// presa ao mundo: a zona gira, o sopro não (o `useGlobalAngle` do `AreaEffector2D` da
/// Unity, que existe para o caso de girar a zona só para encaixá-la na geometria da cena).
///
/// ⚠️ **Governa a força e SÓ ela, e isso é geometria — não escopo escolhido.** O
/// [`AreaTorque`] 2D é um escalar sobre Z e girar a zona *dentro do plano* é uma rotação em
/// torno de Z, então `τ_local ≡ τ_mundo` e não há o que rodar; o [`AreaDrag`] é isotrópico;
/// o [`AreaBuoyancy`] mede a superfície pela GRAVIDADE (água é horizontal mesmo em poça
/// torta); e o [`AreaFormDrag`] empurra pela normal de cada aresta do CORPO. A força é a
/// única grandeza da área com direção própria.
///
/// ⚠️ **Um marcador, não um campo na [`AreaEffector`]** — pela sexta vez na família de
/// zonas, e pela mesma razão: o blob de um componente é postcard POSICIONAL, então apendar
/// campo seria bump de `PROJECT_SCHEMA`, e um bump **recusa todo projeto já salvo**. Um
/// componente novo é chaveado pelo hash do próprio nome de tipo e é puramente aditivo. E um
/// booleano não tem valor a carregar, que é o que faz do marcador a forma honesta.
///
/// A ponte dobra a presença em `AreaEffect.world_axes` (lado do wrapper, esse **não** é
/// serializado), então ele rida o `BodyDesc` que o mundo reconstrói e **um rewind o
/// re-arma**. Config, nunca estado vivo — como todo componente daqui.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AreaForceWorldAxes;

impl SimComponent for AreaForceWorldAxes {}

/// **O quanto o empurrão da área ENFRAQUECE do centro para a borda** (W-AreaFalloff).
///
/// Ausente (o caso comum) é um campo uniforme: todo corpo dentro da zona recebe o mesmo
/// empurrão, esteja no olho ou encostado na parede. Presente, seu `f32` em `0..=1` é o
/// quanto se perde no caminho até a borda — `1` desvanece até **zero exatamente na
/// borda**, `0.5` chega lá com metade, e a lei é `1 − falloff · t` com `t` a fração do
/// caminho do centro até a fronteira ([`ph2d_physics::ShapeDesc::radial_fraction`]).
///
/// É o redemoinho que perde força longe do olho e a corrente que afrouxa na margem — a
/// nota que o W-AreaTorque deixou aberta. E é o antídoto do DEGRAU: sem ele o corpo que
/// atravessa a fronteira passa de força cheia a nada dentro de um sub-passo.
///
/// ⚠️ **A régua é a silhueta da PRÓPRIA zona, nunca um raio à parte.** Um "raio de
/// falloff" seria um segundo número que o artista tem de manter de acordo com o tamanho
/// do collider — a falha de duas-portas que esta linha já pagou várias vezes —, e ele
/// discordaria da zona no instante em que ela fosse redimensionada. Como a régua é a
/// forma, a atenuação chega a zero em **toda** direção e acompanha a escala de graça
/// (W6). O preço honesto: uma zona pequena desvanece em pouco espaço, o que é
/// exatamente o que "fração do caminho até a borda" quer dizer.
///
/// ⚠️ **Pesa a [`AreaEffector`] e o [`AreaTorque`] — os dois EMPURRÕES — e mais nada.** O
/// [`AreaDrag`], o [`AreaBuoyancy`] e o [`AreaFormDrag`] descrevem um MEIO, e um meio não
/// fica mais ralo perto da própria margem: a água da beira da piscina molha igual. Há
/// gate nessa fronteira.
///
/// ⚠️ **Sozinho não faz uma zona:** um falloff sem força nem torque atenua o nada, então a
/// porta `zone_effect` continua recusando a área inerte — o componente é um MODIFICADOR,
/// não um efeito.
///
/// Sétimo componente da mesma área, pela sétima vez pela mesma razão: o blob de um
/// componente é postcard POSICIONAL, então apendar campo no [`AreaEffector`] seria bump de
/// `PROJECT_SCHEMA`, e um bump **recusa todo projeto já salvo**. A ponte o dobra em
/// `AreaEffect.falloff` (lado do wrapper, esse **não** é serializado), então ele rida o
/// `BodyDesc` que o mundo reconstrói e **um rewind o re-arma**. Config, nunca estado vivo.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AreaFalloff(pub f32);

impl AreaFalloff {
    /// É neutro — uma área que empurra igual em toda a sua extensão? Falloff zero (ou
    /// negativo, que não tem sentido: só se pode PERDER força pelo caminho). Usado para
    /// DESTACAR o componente, na mesma política dos irmãos de arrasto.
    #[must_use]
    pub fn is_neutral(self) -> bool {
        self.0 <= 0.0
    }
}

impl SimComponent for AreaFalloff {}

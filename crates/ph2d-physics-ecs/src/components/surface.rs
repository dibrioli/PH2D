//! **A SUPERFÍCIE** — o que este chão diz a quem ANDA sobre ele (`W-Surface`).
//!
//! Terceiro irmão de [`super::overrides`] e [`super::area`], e o corte segue a
//! mesma linha que os dois já traçavam: lá mora *o que este CORPO é*, ao lado *o
//! que esta ÁREA faz a outros*, e aqui *o que esta SUPERFÍCIE faz a quem a
//! pisa*. Ela não é nenhuma das duas — não é uma propriedade do corpo (a mesma
//! caixa é gelo por cima e madeira por baixo se o artista o quiser, uma peça de
//! cada vez) nem uma zona (não há nada em que entrar; há uma face em que se
//! encosta).
//!
//! # ⚠️ Por que NÃO é o `Collider::friction`, e a razão é estrutural
//!
//! A perna do personagem **FLUTUA** (`ph2d_platformer::ride`): ele paira a
//! `float_height` do chão e o que o segura é uma mola, não um contato. O atrito
//! de Coulomb do solver — que é o que aquele campo governa — **nunca se aplica a
//! este personagem, nem uma vez**. Acoplá-los faria *"esta rampa é escorregadia
//! para o personagem"* significar também *"e todo caixote desliza nela"*, e uma
//! esteira **não-escorregadia** ficaria inexprimível: uma correia não tem análogo
//! nenhum em `friction`.
//!
//! # ⚠️ Um componente, dois campos — e o dia em que vier um terceiro
//!
//! Os dois nascem na mesma wave e descrevem o mesmo assunto, então cabem juntos
//! sem custo. Um campo APENDADO amanhã seria postcard **posicional** ⇒ bump de
//! `PROJECT_SCHEMA` ⇒ recusa de todo projeto já salvo; a resposta, então, é um
//! componente IRMÃO — foi exactamente o caminho da família das zonas, que tem
//! sete em vez de uma struct com sete campos.
//!
//! Config, nunca estado vivo de solver.

use ph2d_ecs::{Component, SimComponent};
use serde::{Deserialize, Serialize};

/// **A superfície que o pé encontra — um componente opcional VALUADO
/// (`W-Surface`).**
///
/// Ausente (o caso comum) é o chão de sempre: `grip` neutro e correia parada,
/// exactamente o que toda cena fazia antes disto existir. Ele vive na entidade
/// que carrega o [`crate::Collider`] — o corpo, se ele tem uma forma só, ou a
/// **PEÇA** de um corpo composto (W-Compound), e é por isso que uma plataforma
/// pode ter uma face de gelo e outra de borracha.
///
/// ⚠️ **Quem responde é o MESMO raio que ganhou.** O leque de pés
/// (`bridge::player_leg`) toma o chão como o mais próximo de N raios, e a
/// superfície sai desse vencedor — senão o personagem andaria no gelo e
/// derraparia na madeira dentro do mesmo tique.
///
/// ⚠️ **Só o PLAYER a lê**, e isso é uma limitação nomeada, não um descuido: a
/// amostra de chão é o canal do personagem, então um caixote sobre uma esteira
/// **não** é levado por ela. O `SurfaceEffector2D` da Unity leva qualquer
/// rigidbody; aqui o alcance é o que o nome diz — a superfície como a lei da
/// CAMINHADA a vê.
///
/// Um componente novo é chaveado pelo hash do próprio nome de tipo, então **não
/// custa bump de `PROJECT_SCHEMA`**; e ele é lido do ECS a cada dispatch, o que
/// o torna vivo sob a mão do artista e imune a um rewind sem re-armar nada.
#[derive(Component, Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WalkSurface {
    /// **Quanto do orçamento de caminhada o pé aproveita aqui** — o
    /// multiplicador de `ph2d_platformer::GroundSample::grip`. Neutro `1.0`;
    /// gelo é baixo, borracha é alto, `0.0` é gelo perfeito (não arranca e não
    /// pára).
    ///
    /// ⚠️ **Ele multiplica arrancar E parar**, porque os dois gastam a mesma
    /// tração. Quem separa os dois é o `Brake` do personagem — a razão pela qual
    /// esta wave chega depois daquela.
    pub grip: f32,
    /// **A ESTEIRA** — a velocidade que a superfície apresenta a quem está sobre
    /// ela, em m/s, com **sinal**, medida ao longo da TANGENTE dela. Neutro
    /// `0.0`.
    ///
    /// ⚠️ **Escalar, e não um `Vec2` de mundo**, e a diferença é o que torna o
    /// erro inexprimível: um vetor autorado em eixos de mundo teria componente ao
    /// longo da NORMAL numa rampa, e uma superfície não empurra ninguém para
    /// dentro nem para fora de si mesma — quem faz isso é a perna. Como escalar
    /// sobre a tangente, uma esteira em rampa sobe sozinha e uma plataforma que
    /// gira leva a correia com ela.
    ///
    /// ⚠️ **É a `SurfaceEffector2D` da Unity** (*"forças tangentes ao longo da
    /// superfície para igualar uma velocidade"*), e não a plataforma a mover-se:
    /// hoje uma esteira exigiria que o corpo de facto andasse, o que ninguém
    /// constrói assim.
    ///
    /// ⚠️ **Ela leva por TRAÇÃO**, então uma correia sobre `grip = 0` não leva
    /// nada — o que é exactamente o que uma esteira sem atrito faz no mundo.
    pub belt: f32,
}

impl WalkSurface {
    /// A superfície que ninguém autorou — e o que a ausência do componente
    /// significa.
    pub const NEUTRAL: Self = Self {
        grip: 1.0,
        belt: 0.0,
    };

    /// **Este componente diz alguma coisa?** — o predicado do DETACH: no neutro
    /// o Inspector o remove, e um arquivo de projeto fica livre do no-op.
    #[must_use]
    pub fn is_neutral(self) -> bool {
        self == Self::NEUTRAL
    }
}

impl Default for WalkSurface {
    fn default() -> Self {
        Self::NEUTRAL
    }
}

impl SimComponent for WalkSurface {}

/// **Esta superfície NÃO é parede** — a mão não a agarra (`W-WallMaterial`).
///
/// O `platform_wall_layers` do Godot, e como o irmão dele
/// ([`crate::PlatformLift`] para o `platform_on_leave`) resolvido **por CORPO ou
/// por PEÇA**, não por camada: um bitmask obriga o artista a arrumar o nível em
/// camadas para exprimir uma propriedade de material, e a peça é a granularidade
/// que a família da superfície já tem.
///
/// Sua **presença É o booleano**, como [`crate::Ccd`]/[`crate::LockRotation`]/
/// [`crate::OneWayPlatform`]. Ausente — o caso comum — toda superfície íngreme
/// continua a ser parede, exactamente como antes disto existir.
///
/// # O que ele desliga, e é a família inteira de uma vez
///
/// A lei aceita uma parede por **inclinação** e só por ela ([`ph2d_platformer::cling`]:
/// *"a MESMA régua da perna: parede é o que ela recusou por inclinação"*), e a
/// amostra que sai dali alimenta os **três** verbos — deslizar, agarrar-se e o
/// pulo de parede. Marcar a superfície derruba o acerto **no SENSOR**, então os
/// três somem juntos: não é *"não pula daqui"*, é *"isto não é uma parede"*.
///
/// ⚠️ **Por que NÃO é o [`WalkSurface::grip`], que já existe e já significa
/// tração:** o doc dele diz o que ele é — *quanto do orçamento de CAMINHADA o pé
/// aproveita aqui* —, e esticá-lo até a mão faria a documentação mentir sobre o
/// próprio campo. E as duas propriedades são de facto independentes: um piso de
/// gelo escalável e uma parede de borracha inescalável são as duas coisas que um
/// nível quer dizer, e um só número não diz as duas.
///
/// ⚠️ **A lei do part-vs-corpo é a MESMA da [`WalkSurface`]** — a peça
/// sobrepõe-se ao corpo, wholesale — porque as duas viajam na mesma entrada da
/// tabela do bridge. Corolário honesto: **uma peça não consegue dizer *"eu SOU
/// parede"* contra um corpo marcado**, porque a presença de um marcador não tem
/// como exprimir o "sim"; quem precisar disso marca as peças, não o corpo.
///
/// Um componente novo é chaveado pelo hash do próprio nome de tipo, então **não
/// custa bump de `PROJECT_SCHEMA`** — a saída que o doc deste módulo já prescreve
/// para o dia em que um campo novo apareça.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NoWallCling;

impl SimComponent for NoWallCling {}

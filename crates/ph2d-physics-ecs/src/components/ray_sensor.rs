//! **O objecto passa a OLHAR** (suplente #21) — um raio persistente, autorado.
//!
//! # ⭐⭐⭐ O buraco tem TRÊS nomes, e nenhum se compõe do que existia
//!
//! A sonda [`mede_o_que_a_composicao_ja_da_ao_raio`] correu antes da primeira linha (`CLAUDE.md`
//! §5.0), com a melhor composição que a casa tem — um colisor `is_sensor` fino deitado ao longo da
//! linha, que é exactamente como o **#3 `SensorZone`** fechou **por composição** dois dias antes:
//!
//! | a pergunta | a barra sensor | o [`cast_ray`](ph2d_physics::PhysicsWorld::cast_ray) |
//! |---|---|---|
//! | **ORDEM** | `triggered_sensors()` devolve `["Barra"]` — **um** elemento, com as duas paredes lá dentro | acerta na de `x = 2`, a mais perto |
//! | **MÉTRICA** | **`0` contactos de pé** (um sensor atravessa) ⇒ `point`/`normal`/`impulse` vazios | `d = 1,7500` · ponto `(1,75 ; 0)` · normal `(−1 ; 0)` |
//! | **DIRECÇÃO** | **não distingue**: uma FORMA é simétrica, e a barra apanha a parede de `x = −3` | a de trás está no alcance e **não** volta |
//!
//! ⇒ *o motor está pago e é rico; o que faltava era um componente autorável que lhe chegue.*
//!
//! # ⚠️⚠️ A nota da porta do motor, RECONFERIDA (§0.0)
//!
//! O [`cast_ray`](ph2d_physics::PhysicsWorld::cast_ray) põe `EXCLUDE_SENSORS` **dentro dele**, e
//! justifica-o com *«os cinco consumidores querem matéria; um parâmetro seria uma escolha oferecida
//! a ninguém»*. Este componente é o **sexto** consumidor e o primeiro **autorável**, logo o §0.0
//! obriga a reconferir a nota em vez de a herdar.
//!
//! **Veredito: ela continua de pé, e por uma razão mais forte.** As três perguntas que este
//! componente serve — *há chão por baixo? · há parede à frente? · a arma aponta para quê?* — são
//! todas sobre **matéria**, e um volume de gatilho que bloqueasse a linha de visão seria um defeito
//! e não uma opção. ⇒ **nenhum parâmetro novo na porta.**
//!
//! # ⚠️ É CONFIG, nunca estado vivo
//!
//! A lei do módulo: o que o raio VÊ muda a cada tique e **não** vive aqui — ele vive no mapa da
//! ponte, que é onde o canal de triggers (W7) já guarda o dele. Um campo «o que estou a ver» dentro
//! de um componente registado faria o `canonicalize` do undo ver **cada quadro como um passo**.

use bevy_ecs::prelude::Component;
use ph2d_core::Vec2;
use ph2d_ecs::SimComponent;
use serde::{Deserialize, Serialize};

/// **Um raio persistente: o objecto «Ray Sensor»** (GEF 07, dono 15/11 — objecto filho, como a forma do ADR-0199).
///
/// A origem é a POSIÇÃO do objecto e o rumo é `angle` rodado pela pose dele (e a do pai): dois raios
/// são dois objectos. ⚠️ `angle` é LOCAL e em GRAUS (a unidade autorada, `FieldKind::Angle`): anima-se
/// sem passar por `0,707`, e o motor faz o vetor (`dir_local`).
#[derive(Component, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct RaySensor {
    /// Para onde aponta, em graus locais: `0` = direita, `90` = cima, `-90` = baixo.
    pub angle: f32,
    /// Até onde ele enxerga, em metros.
    pub reach: f32,
    /// A camada de colisão pela qual ele pergunta — a matriz do mundo decide o que ele vê.
    pub layer: u8,
    /// Espelha com o lado para onde o personagem dono olha (`PlayerState::facing`): a olhar para a
    /// esquerda, o ângulo e o desvio do raio trocam de lado. Opção do artista (dono 15/11), desligada.
    pub turns_with_facing: bool,
}

impl RaySensor {
    /// O rumo unitário local que `angle` diz.
    #[must_use]
    pub fn dir_local(&self) -> Vec2 {
        let r = self.angle.to_radians();
        Vec2::new(libm::cosf(r), libm::sinf(r))
    }
}

impl Default for RaySensor {
    /// O default OLHA PARA BAIXO: o sensor de chão é o primeiro raio de um jogo 2D.
    fn default() -> Self {
        Self {
            angle: -90.0,
            reach: 1.0,
            layer: 0,
            turns_with_facing: false,
        }
    }
}

impl SimComponent for RaySensor {}

/// **O que um raio publica quando PASSA A ver e quando DEIXA de ver.**
///
/// ⚠️ **Dois nomes, porque são duas perguntas** — a regra que o [`SignalOnHit`](super::SignalOnHit)
/// já escreve: emitir o mesmo nome nos dois extremos torna o sinal ambíguo, e quem escuta casa numa
/// string. ⚠️ **Vazio = calado**, palavra por palavra a regra dele.
///
/// ⛔ **Componente à parte e não campos do [`RaySensor`]:** ele é serializado POSICIONALMENTE pelo
/// postcard, e um objecto que só quer *«há chão?»* não paga dois nomes que nunca usa. É o mesmo
/// trade que o `AreaEffector`/`AreaDrag` pagou.
#[derive(Component, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RaySignals {
    /// Publicado no tique em que o raio passa a ver alguma coisa.
    pub on_enter: String,
    /// Publicado no tique em que ele deixa de ver.
    pub on_exit: String,
}

impl RaySignals {
    /// O nome de entrada, ou `None` se estiver em branco.
    #[must_use]
    pub fn enter(&self) -> Option<&str> {
        let t = self.on_enter.trim();
        (!t.is_empty()).then_some(t)
    }

    /// O nome de saída, ou `None` se estiver em branco.
    #[must_use]
    pub fn exit(&self) -> Option<&str> {
        let t = self.on_exit.trim();
        (!t.is_empty()).then_some(t)
    }
}

impl SimComponent for RaySignals {}

/// **O que um raio viu neste tique** — a leitura que a ponte publica.
///
/// ⛔ Ela **não** é um componente: o que nasce numa corrida não é documento (a lei do #11 e do #20),
/// e a memória de um tique cabe no mapa da ponte — o precedente é o canal de triggers.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RayHit {
    /// Quem ele acertou.
    pub body: ph2d_ecs::Entity,
    /// A que distância da origem, em metros.
    pub distance: f32,
    /// Onde, em mundo.
    pub point: [f32; 2],
    /// A normal da superfície ali — é ela que faz a REFLEXÃO do Construct ser exprimível.
    pub normal: [f32; 2],
}

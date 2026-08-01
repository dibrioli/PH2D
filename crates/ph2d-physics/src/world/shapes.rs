//! **As formas de um corpo** — a porta única para *"de que este corpo é feito?"*
//! (W-CompoundZone).
//!
//! # O vão, medido
//!
//! Cinco sítios do wrapper perguntavam `rb.colliders().first()`, o que é a frase
//! *"um corpo tem exatamente um collider"* escrita em código. Ela era verdade até
//! a **W-Compound**, e ninguém a reconferiu — o mesmo envelhecimento que deixou o
//! canal de trigger cego a uma peça-sensor (W-PartSensor).
//!
//! O preço está medido (`ph2d-physics-ecs/tests/measure_compound_zone.rs`): duas
//! jangadas de **silhueta e massa idênticas**, uma feita de UMA caixa e outra de
//! DUAS, largadas na MESMA poça de água plana. A única fica em `0,000°`; a
//! composta **CAPOTA para −90°**.
//!
//! ⚠️ **E o sintoma não é o que a intuição prevê.** Metade do empuxo faria
//! esperar *"afunda o dobro"*; o que acontece é pior e mais confuso de
//! diagnosticar: a força nasce **descentrada** (só a primeira forma a produz),
//! logo há torque, logo o barco tomba. *Meia-força e força-no-lugar-errado são
//! defeitos diferentes, e este é o segundo.*
//!
//! # A ordem é FIXA, e isso é HR-5
//!
//! Somar impulsos sobre uma lista é somar `f32`, e adição de ponto flutuante
//! **não é associativa** — a ordem em que as formas de um corpo aparecem em
//! `rb.colliders()` é a ordem de inserção, que é detalhe interno. Ordenar por
//! handle torna a soma reproduzível, o mesmo cuidado que a tabela `effectors`
//! já toma pelo mesmo motivo, e é o que mantém o `physics_ecs_c9` honesto.
//!
//! # ⚠️ O custo que este modelo ACEITA: a emenda interna
//!
//! Um corpo composto tem arestas **internas** — a face onde duas formas se
//! encostam — e um modelo por-forma as vê encarando o escoamento como qualquer
//! outra. O `form_drag` de uma jangada de duas metades resiste, portanto, **mais**
//! que o da caixa única de silhueta idêntica.
//!
//! Isto é over-count honesto, e a alternativa não é *"não mudar nada"*: ler só a
//! primeira forma erra nos **dois** eixos — resiste de menos **e** aplica a força
//! no lugar errado, que é um torque. Medido, uma jangada lançada de lado gira
//! **−12,995°** com `.first()` e **0,000°** somando todas. *Magnitude ligeiramente
//! alta num lugar certo é um trade; força num lugar errado é um defeito.*
//!
//! A cura exata seria a fronteira do casco convexo do corpo, que é outra
//! operação (e outra wave). Nomeado aqui em vez de escondido.
//!
//! # O que este módulo NÃO toca, de propósito
//!
//! Sobram dois `.colliders().first()` no wrapper — [`super::effector`] e
//! [`super::queries::PhysicsWorld::waterlines`] — e os dois perguntam pela
//! **ZONA**, não pelo corpo afetado. Uma zona é single-collider **por
//! construção**: a face de PEÇA do §11 não pinta as rows de área (W-PartFace), de
//! modo que a família `Area*` só é autorável num corpo. Trocá-los seria
//! generalidade sem caso de uso — e o dia em que uma peça puder carregar uma zona
//! é o dia em que os dois entram aqui.

use rapier2d::dynamics::RigidBody;
use rapier2d::geometry::{Collider, ColliderHandle};

/// As formas de `rb` em ordem **estável**, escritas em `out` (reusado pelo
/// chamador, então uma cena sem zonas não aloca depois do warm-up).
pub(crate) fn sorted_shapes(rb: &RigidBody, out: &mut Vec<ColliderHandle>) {
    out.clear();
    out.extend_from_slice(rb.colliders());
    out.sort_unstable_by_key(|h| h.into_raw_parts());
}

/// **Esta forma desloca fluido / resiste ao escoamento?**
///
/// Um **sensor** não: ele é um marcador, não matéria — atravessa tudo por
/// definição, e dar-lhe empuxo faria o pé-sensor de um personagem
/// (W-PartSensor) boiar num pedaço de nada.
///
/// ⚠️ **É uma pergunta NOVA, que o `.first()` escondia**, e ela muda o
/// comportamento de um caso que já existia: um corpo cujo collider PRÓPRIO é
/// sensor boiava, e agora não boia. É a resposta coerente com o resto do módulo
/// — a zona e o one-way já são mutuamente exclusivos pela mesma razão (um sensor
/// não gera contato) — e está gateada nos dois sentidos.
pub(crate) fn displaces(c: &Collider) -> bool {
    !c.is_sensor()
}

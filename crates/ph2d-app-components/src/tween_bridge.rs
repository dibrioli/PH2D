//! **A ponte do TWEEN** (suplente #22) — o que a lei pediu, escrito no mundo pelo LEDGER.
//!
//! Irmã do [`crate::hud_bridge`] e do [`crate::script_bridge`], e pela mesma lei: *o que um motor
//! escreve agora é **pré-visualização** — vê-se, não se guarda nem se desfaz.* Sem o ledger, um
//! fade de meio segundo seria **trinta passos de `Ctrl+Z`**.
//!
//! # ⭐⭐⭐ Três drivers para oito canais, e a conta não é arbitrária
//!
//! | o canal escreve em | o driver | porquê |
//! |---|---|---|
//! | `Sprite::tint[3]` | **`SpriteAlpha`**, que já existia | é o **mesmo facto vindo de outro motor** — a curva de `Opacity` da timeline escreve ali. Um driver novo sobre o mesmo campo seria duas entradas para um valor, e o `authored` de uma apagaria o da outra |
//! | `Sprite::self_tint` + `tint_fill` | `TweenTint` | ninguém os conduzia; e eles são **um** facto (*«a silhueta está acesa nesta cor»*) |
//! | `Transform` | `TweenPose` | a pose tem outros donos (o solver, a timeline, um script, o palco), e a chave do ledger é `(entidade, driver)` |
//!
//! # ⚠️ UM censo por entidade, e não um por escrita
//!
//! Dois tweens de pose no mesmo objecto (`PositionX` e `PositionY`) escrevem o **mesmo**
//! `Transform`. A fotografia do ANTES é tirada uma vez, as escritas todas acontecem, e só então o
//! facto é declarado — senão a segunda declaração leria a saída da primeira como *«outra mão
//! escreveu»* e promoveria a pré-visualização a documento. *É a mesma frase que o `fase_sequences`
//! escreve para duas cutscenes na mesma entidade.*

use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_preview_drive::{Driven, Driver, PreviewDrive};
use ph2d_render::Sprite;
use ph2d_tween::Canal;

/// O que uma entidade tinha ANTES deste passe — a fotografia de que o ledger precisa.
#[derive(Clone, Copy)]
struct Antes {
    pose: Option<Transform>,
    alfa: Option<f32>,
    tinta: Option<([f32; 4], bool)>,
}

/// ⭐⭐⭐ **Um quadro de tweens.** Devolve **quantas escritas** foram feitas — o número que um
/// diagnóstico imprime e que um gate lê.
///
/// ⚠️ **Zero custo quando não há nenhum:** a lista vem vazia e a função devolve `0` antes de tirar
/// censo nenhum, que é o caso de toda cena que já existe.
pub fn drive_tweens(
    sim: &mut SimWorld,
    drive: &mut PreviewDrive,
    caminhos: &[crate::path_follow_bridge::PoseDeCaminho],
) -> usize {
    let pedidos = ph2d_ecs::tween::a_escrever(sim.world_mut());

    // ⚠️ **As entidades ainda conduzidas entram no censo mesmo sem pedido neste quadro** — é a
    // linha do meio da tabela do ledger: um motor que continua a conduzir e cujo valor não mudou
    // não pode deixar a pré-visualização voltar a ser documento.
    let mut alvos: Vec<Entity> = pedidos
        .iter()
        .map(|p| p.entity)
        .chain(caminhos.iter().map(|c| c.entity))
        .collect();
    alvos.sort_unstable();
    alvos.dedup();
    if alvos.is_empty() {
        return 0;
    }

    let antes: Vec<(Entity, Antes)> = alvos
        .iter()
        .map(|&e| {
            let pose = sim.world().get::<Transform>(e).copied();
            let sprite = sim.world().get::<Sprite>(e);
            (
                e,
                Antes {
                    pose,
                    alfa: sprite.map(|s| s.tint[3]),
                    tinta: sprite.map(|s| (s.self_tint, s.tint_fill)),
                },
            )
        })
        .collect();

    let mut n = 0;
    for p in &pedidos {
        if escreve(sim, p) {
            n += 1;
        }
    }
    // ⭐⭐⭐ **O SEGUIDOR DE CAMINHO escreve AQUI, e não num passe próprio** (suplente #23): a
    // fotografia do ANTES já foi tirada acima, e a declaração ao ledger vem abaixo — *uma pose,
    // um censo*. Um segundo passe leria a saída do tween como se fosse o documento.
    for c in caminhos {
        if pousa_no_caminho(sim, c) {
            n += 1;
        }
    }

    for (e, era) in antes {
        declara(sim, drive, e, era);
    }
    n
}

/// Põe uma entidade **onde a curva manda**, convertendo o mundo para o referencial do pai dela.
///
/// ⚠️⚠️ **A conversão é obrigatória e não é cosmética:** a curva vive em MUNDO (a ponte já compôs
/// a cadeia de pais da forma) e o `Transform` de uma entidade é **LOCAL**. Escrever mundo num local
/// põe um seguidor com pai ao lado da pista, deslocado exactamente pela pose do pai.
///
/// ⚠️ **A ESCALA não se toca** — ela é do artista; o que este motor conduz é *onde* e *para onde*.
fn pousa_no_caminho(sim: &mut SimWorld, c: &crate::path_follow_bridge::PoseDeCaminho) -> bool {
    let pai = ph2d_ecs::parent_world_transform(sim.world(), c.entity);
    let Some(local) = sim.world().get::<Transform>(c.entity).copied() else {
        return false;
    };
    let mut alvo = Transform::compose(pai, local);
    alvo.translation.x = c.mundo[0];
    alvo.translation.y = c.mundo[1];
    if let Some(a) = c.angulo {
        alvo.rotation = a;
    }
    let Some(novo) = Transform::inverse_compose(pai, alvo) else {
        return false;
    };
    let Some(mut t) = sim.world_mut().get_mut::<Transform>(c.entity) else {
        return false;
    };
    t.translation = novo.translation;
    if c.angulo.is_some() {
        t.rotation = novo.rotation;
    }
    true
}

/// Escreve UMA pedida. `false` = a entidade não tem o componente que o canal endereça — e isso é um
/// no-op silencioso de propósito: *um tween de cor num objecto sem sprite é inerte, não um erro*.
fn escreve(sim: &mut SimWorld, p: &ph2d_ecs::Escrita) -> bool {
    let v = p.valor;
    match p.canal {
        Canal::Opacity => {
            let Some(mut s) = sim.world_mut().get_mut::<Sprite>(p.entity) else {
                return false;
            };
            if s.tint[3] != v[0] {
                s.tint[3] = v[0];
            }
        }
        Canal::Tint | Canal::Silhueta => {
            // ⭐ **A silhueta é a MESMA cor com o interruptor ligado** — e o `Tint` desliga-o de
            // propósito, senão um `Tint` depois de um `Silhueta` herdaria a silhueta em silêncio.
            let fill = matches!(p.canal, Canal::Silhueta);
            let Some(mut s) = sim.world_mut().get_mut::<Sprite>(p.entity) else {
                return false;
            };
            if s.self_tint != v || s.tint_fill != fill {
                s.self_tint = v;
                s.tint_fill = fill;
            }
        }
        _ => {
            let Some(mut t) = sim.world_mut().get_mut::<Transform>(p.entity) else {
                return false;
            };
            // ⚠️ **Um campo de cada vez**, e é o que faz `PositionX` e `PositionY` comporem em vez
            // de se apagarem: escrever o `Transform` inteiro faria o segundo repor o eixo do
            // primeiro pelo valor que ele tinha antes do passe.
            match p.canal {
                Canal::PositionX => t.translation.x = v[0],
                Canal::PositionY => t.translation.y = v[0],
                Canal::ScaleX => t.scale.x = v[0],
                Canal::ScaleY => t.scale.y = v[0],
                Canal::Rotation => t.rotation = v[0],
                // Os de aparência já saíram nos braços acima — este é inalcançável.
                Canal::Opacity | Canal::Tint | Canal::Silhueta => return false,
            }
        }
    }
    true
}

/// Declara ao ledger o que mudou nesta entidade, um driver de cada vez.
fn declara(sim: &SimWorld, drive: &mut PreviewDrive, e: Entity, era: Antes) {
    if let (Some(antes), Some(agora)) = (era.pose, sim.world().get::<Transform>(e).copied()) {
        conduz(
            drive,
            e,
            Driver::TweenPose,
            Driven::TweenPose(antes),
            Driven::TweenPose(agora),
        );
    }
    let Some(s) = sim.world().get::<Sprite>(e) else {
        return;
    };
    if let Some(antes) = era.alfa {
        conduz(
            drive,
            e,
            Driver::SpriteAlpha,
            Driven::SpriteAlpha(antes),
            Driven::SpriteAlpha(s.tint[3]),
        );
    }
    if let Some((self_tint, fill)) = era.tinta {
        conduz(
            drive,
            e,
            Driver::TweenTint,
            Driven::TweenTint { self_tint, fill },
            Driven::TweenTint {
                self_tint: s.self_tint,
                fill: s.tint_fill,
            },
        );
    }
}

/// ⚠️ **A linha do meio da tabela do ledger**, escrita uma vez: declarar quando mudou, e **voltar a
/// declarar** quando não mudou mas o motor continua a conduzir. Sem a segunda metade, um tween
/// parado num valor deixaria a `settle` promovê-lo a documento.
fn conduz(drive: &mut PreviewDrive, e: Entity, d: Driver, antes: Driven, agora: Driven) {
    if antes != agora {
        drive.driven(e, antes, agora);
    } else if drive.still_driving(e, d) {
        drive.driven(e, agora, agora);
    }
}

#[cfg(test)]
#[path = "tween_bridge_tests.rs"]
mod tests;

//! **Uma pilha que assenta tem de adormecer DEITADA, não a meio do caminho.**
//!
//! ⛔⛔ **Por que este ficheiro existe:** em 2026-08-29, na subida da `rapier2d` 0.28 → 0.35, o
//! `sleep_linear_threshold` foi **fixado** no `0,4` da 0.28 com a justificação de *«não mudar o
//! tato em silêncio»*. Mas o solver foi **reescrito** por baixo: um corpo passa a assentar por
//! outro caminho e cai abaixo de `0,4 m/s` **a meio do assentamento**, adormecendo torto. O dono
//! do produto viu-o num smoke — uma caixa parada inclinada, sem assentar no chão — e **nenhum dos
//! 981 testes da física media a POSE em repouso**. Mediam velocidade, penetração, energia, hashes:
//! nenhum perguntava *«em que ÂNGULO ele parou?»*.
//!
//! ⭐⭐ **A barra é um ORÁCULO medido na mesma corrida, não um número escolhido.** A pose certa é a
//! de uma simulação a que se PROÍBE dormir; o produto tem de chegar à mesma, e ainda assim dormir
//! de facto. Um número mágico envelheceria com o solver — este oráculo re-mede-se sozinho.
//!
//! ⚠️ **E a metade JUSTA:** sem a alínea (a), um futuro que curasse o ângulo **desligando o sono**
//! passaria o gate. Sem a (b), o valor de sempre passaria. As duas juntas é que fecham.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, PhysicsBridge, PhysicsSettings, RigidBody,
};

/// A cena 4 do smoke, replicada: chão + 12 corpos de três tamanhos em quatro colunas.
/// ⚠️ **Uma pilha, não um corpo só** — um corpo isolado assenta deitado mesmo com o limiar mau;
/// é o contato com os vizinhos que produz a velocidade lenta em que o defeito vive.
/// Os três tamanhos da cena de smoke `=4`. ⚠️ **Constante partilhada de propósito:** a âncora
/// absoluta lá em baixo precisa de saber qual é o MENOR, e escrevê-lo à mão foi exactamente o
/// erro que a 1.ª versão dela cometeu (usou o maior e reprovou sobre produto correcto).
const SIZES: [f32; 3] = [0.7, 0.45, 0.28];

fn pilha() -> (SimWorld, Vec<Entity>) {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Floor"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 4.0,
                half_y: 0.2,
            },
            density: 1.0,
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, -1.0)),
    ));
    let sizes = SIZES;
    let ids = (0..12u32)
        .map(|i| {
            let s = sizes[(i % 3) as usize];
            sim.world_mut()
                .spawn((
                    Name::new(format!("Body{i:02}")),
                    RigidBody {
                        kind: BodyKind::Dynamic,
                    },
                    Collider {
                        shape: ColliderShape::Cuboid {
                            half_x: s * 0.5,
                            half_y: s * 0.5,
                        },
                        density: 1.0,
                        ..Collider::default()
                    },
                    Transform::from_translation(Vec2::new(
                        (i % 4) as f32 * 0.9 - 1.35,
                        2.0 + (i / 4) as f32 * 1.1,
                    )),
                ))
                .id()
        })
        .collect();
    (sim, ids)
}

/// `(pior ângulo, quantos ficaram BIT-A-BIT parados entre os 10 s e os 20 s, a ALTURA de cada um)`.
fn assenta(sleep: Option<f32>) -> (f32, usize, Vec<f32>) {
    let (mut sim, ids) = pilha();
    let mut bridge = PhysicsBridge::default();
    // `None` = o oráculo: proibido dormir. O limiar `0` nunca é alcançado por um corpo em contato,
    // que treme sempre um pouco.
    bridge.set_settings(PhysicsSettings {
        sleep_linear_threshold: sleep.unwrap_or(0.0),
        ..bridge.settings()
    });
    let ler = |sim: &SimWorld| -> Vec<(f32, f32, f32)> {
        ids.iter()
            .map(|e| {
                let t = *sim
                    .world()
                    .entity(*e)
                    .get::<Transform>()
                    .expect("transform");
                (t.translation.x, t.translation.y, t.rotation)
            })
            .collect()
    };
    for t in 0..=600u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let dez = ler(&sim);
    for t in 601..=1200u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let vinte = ler(&sim);

    let parados = dez
        .iter()
        .zip(&vinte)
        .filter(|(a, b)| a.0 == b.0 && a.1 == b.1 && a.2 == b.2)
        .count();
    let pior = vinte
        .iter()
        .map(|(_, _, r)| r.to_degrees().abs())
        .fold(0.0f32, f32::max);
    // ⛔⛔ **A POSIÇÃO também sai daqui, e a 1.ª versão deitava-a fora.** A `ler()` sempre recolheu
    // `x` e `y`, e este `fold` só olhava para a rotação ⇒ uma pilha congelada **25 cm acima do
    // chão**, com rotação nula, passava as duas metades. E isso é a **metade literal** do report
    // que fez este ficheiro nascer: *«a caixa parou antes de assentar no chão»* — não «parou
    // torta». Achado por auditoria no mesmo dia, num gate escrito nesse dia.
    // ⚠️ *Um gate escrito a partir de metade de um report defende metade dele.*
    (pior, parados, vinte.iter().map(|(_, y, _)| *y).collect())
}

/// **Medido em 2026-08-29** (o defeito, a cura e o oráculo, na mesma corrida):
///
/// | limiar / atraso | pior ângulo | parados 10 s→20 s |
/// |---|---|---|
/// | ⛔ `0,4` / `2,0 s` — o valor fixado da rapier 0.28 | **`2,320°`** | 12/12 |
/// | ⭐ `0,05` / `2,0 s` — o que shipa | `0,04455°` | 12/12 |
/// | ⛔ `0,4` / `0,5 s` | **`7,621°`** | 12/12 |
/// | oráculo: proibido dormir | `0,04455°` | 0/12 |
///
/// ⚠️ **A folga de `0,05°` sobre o oráculo é `1000×` a diferença medida** (`1e-5°`) e ainda assim
/// **`46×` menor** que o defeito que ela apanha — ⛔ não é um número escolhido para caber: é a
/// distância entre duas medições, e a mutação que ela defende (repor o `0,4`) atravessa-a `46×`.
#[test]
fn a_settled_stack_falls_asleep_lying_flat_not_halfway_there() {
    let (oraculo, oraculo_parados, oraculo_y) = assenta(None);
    let (produto, produto_parados, produto_y) =
        assenta(Some(PhysicsSettings::default().sleep_linear_threshold));

    // A fixtura tem de CONTER o fenómeno: se o oráculo também congelasse, ele não seria um oráculo
    // de «nunca dorme» e a comparação abaixo não diria nada.
    assert_eq!(
        oraculo_parados, 0,
        "o oraculo tinha de ficar ACORDADO e treme-lhe a pose; congelaram {oraculo_parados}/12"
    );

    // (a) O produto tem MESMO de dormir — senao esta lei passa por nunca a testar.
    assert_eq!(
        produto_parados, 12,
        "com as definicoes do produto os 12 corpos tinham de ADORMECER (bit-a-bit parados entre \
         os 10 s e os 20 s); apenas {produto_parados} pararam"
    );

    // (b) E tem de dormir na MESMA pose de quem nunca dorme.
    // (c) ⛔ **E tem de adormecer NO SÍTIO, não só direito.** O oráculo dá a altura de graça — e a
    // 1.ª versão deste gate recolhia-a sem a comparar.
    let pior_desvio = produto_y
        .iter()
        .zip(&oraculo_y)
        .map(|(p, o)| (p - o).abs())
        .fold(0.0f32, f32::max);
    // ⛔⛔ **E uma ÂNCORA ABSOLUTA, porque a comparação com o oráculo é cega a um desvio GLOBAL.**
    // A mutação que a auditoria propôs — somar `+0,25 m` a toda pose lida no `readback` —
    // **SOBREVIVEU** ao par produto↔oráculo: os dois passam pelo mesmo caminho de leitura, logo os
    // dois sobem, e a diferença não se mexe. ⭐ *Um gate que só compara duas corridas nossas não vê
    // o que move as duas.*
    //
    // O chão tem o centro em `y = −1,0` e meia-altura `0,2` ⇒ topo em **`−0,8`**. O corpo mais
    // baixo da pilha tem de o tocar. A folga de `0,02 m` é `75×` a penetração medida em repouso
    // (`0,000264 m`) e `12×` menor que a mutação que apanha.
    // ⚠️ **DERIVADA, não escrita.** A 1.ª versão desta âncora usou a meia-altura do corpo MAIOR e
    // reprovou sobre produto correcto — quem fica por baixo numa pilha é o corpo **mais pequeno**.
    // *Uma âncora absoluta escrita à mão erra sobre o produto certo; derivada da fixtura, não.*
    const TOPO_DO_CHAO: f32 = -0.8; // centro −1,0 + meia-altura 0,2
    let menor_meia_altura = SIZES.iter().copied().fold(f32::INFINITY, f32::min) * 0.5;
    let esperado = TOPO_DO_CHAO + menor_meia_altura;
    let mais_baixo = produto_y.iter().copied().fold(f32::INFINITY, f32::min);
    assert!(
        (mais_baixo - esperado).abs() <= 0.02,
        "a pilha nao esta' ASSENTE NO CHAO: o corpo mais baixo tem o centro em {mais_baixo:.4} e \
         deitado no topo do chao teria {esperado:.4}. ⚠️ Uma pilha inteira DESLOCADA e' invisivel \
         a' comparacao com o oraculo — os dois lados passam pelo mesmo readback, e foi assim que a \
         mutacao `+0,25 m` no `readback.rs` sobreviveu ao par produto-oraculo."
    );

    assert!(
        pior_desvio <= 0.01,
        "a pilha adormeceu NO SITIO ERRADO: o corpo mais desviado parou {pior_desvio:.4} m do \
         lugar onde para quem nao dorme. E' a metade literal do report de 2026-08-29 — «a caixa \
         parou antes de assentar no chao» — e um gate que so' olhe para o ANGULO nao a ve^."
    );

    assert!(
        produto <= oraculo + 0.05,
        "a pilha adormeceu TORTA: pior angulo {produto:.5} graus contra {oraculo:.5} do oraculo \
         que nao dorme. E' o defeito de 2026-08-29 — um limiar de sono alto demais apanha o corpo \
         A MEIO do assentamento e congela-o ali."
    );
}

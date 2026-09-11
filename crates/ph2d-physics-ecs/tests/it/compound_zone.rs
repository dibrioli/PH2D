//! **Uma ZONA vê o corpo COMPOSTO inteiro** (W-CompoundZone).
//!
//! Cinco sítios do wrapper perguntavam `rb.colliders().first()` — a frase *"um
//! corpo tem exatamente um collider"* escrita em código, verdadeira até a
//! W-Compound. Estes gates dirigem a PONTE de verdade e comparam um corpo
//! composto contra o **CONTROLE**: o corpo de mesma silhueta e mesma massa feito
//! de UMA forma.
//!
//! ⚠️ Sem o controle nenhum destes números é atribuível — ele é o que separa *"a
//! segunda forma é ignorada"* de *"a física é assim mesmo"*.

use ph2d_core::Vec2;
use ph2d_ecs::{ChildOf, Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    AreaBuoyancy, AreaDrag, AreaFormDrag, BodyKind, Collider, ColliderShape, PhysicsBridge,
    RigidBody,
};

const HALF_X: f32 = 0.6;
const HALF_Y: f32 = 0.25;

fn cuboid(half_x: f32, half_y: f32) -> Collider {
    Collider {
        shape: ColliderShape::Cuboid { half_x, half_y },
        density: 1.0,
        ..Collider::default()
    }
}

/// Uma poça de água plana, 4× mais densa que os corpos.
///
/// ⚠️ **A densidade DIFERENTE é o que torna o gate legível:** com densidades
/// iguais o empuxo é neutro, as duas jangadas afundam juntas, e o oráculo
/// desaparece. Boiando, a linha d'água de equilíbrio é um número nítido.
///
/// ⚠️ **E o `AreaDrag` não é enfeite: empuxo sem resistência é uma MOLA SEM
/// AMORTECIMENTO.** A primeira versão deste gate não o tinha, e o controle foi
/// medido em `1,0795` — o equilíbrio existe, a jangada só nunca chega nele. Um
/// oráculo de repouso precisa que o repouso exista.
fn pool(sim: &mut SimWorld, form_drag: bool) {
    let mut e = sim.world_mut().spawn((
        Name::new("Pool"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            is_sensor: true,
            ..cuboid(10.0, 3.0)
        },
        AreaBuoyancy(4.0),
        AreaDrag(0.6),
        Transform::from_translation(Vec2::new(0.0, -3.0)),
    ));
    if form_drag {
        e.insert(AreaFormDrag(3.0));
    }
}

/// `compound = false` ⇒ UMA caixa larga (o CONTROLE).
/// `compound = true`  ⇒ duas metades, a segunda como PEÇA. Mesma silhueta
/// (`x ∈ [−0,6; 1,8]`), mesma massa (medida: 1,200000 nas duas).
fn raft(sim: &mut SimWorld, compound: bool, sensor_part: bool) -> Entity {
    if compound {
        let body = sim
            .world_mut()
            .spawn((
                Name::new("Raft"),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                cuboid(HALF_X, HALF_Y),
                Transform::from_translation(Vec2::new(0.0, 1.5)),
            ))
            .id();
        sim.world_mut().spawn((
            Name::new("Raft Deck"),
            Collider {
                is_sensor: sensor_part,
                ..cuboid(HALF_X, HALF_Y)
            },
            Transform::from_translation(Vec2::new(HALF_X * 2.0, 0.0)),
            ChildOf(body),
        ));
        body
    } else {
        sim.world_mut()
            .spawn((
                Name::new("Raft"),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                cuboid(HALF_X * 2.0, HALF_Y),
                Transform::from_translation(Vec2::new(HALF_X, 1.5)),
            ))
            .id()
    }
}

/// Assenta uma jangada e devolve `(altura do centro, inclinação em graus)`.
///
/// 2400 ticks: a sonda mostra o equilíbrio estável a partir de ~900.
fn settle(compound: bool, sensor_part: bool, form_drag: bool) -> (f32, f32) {
    settle_at(compound, sensor_part, form_drag, 2400)
}

fn settle_at(compound: bool, sensor_part: bool, form_drag: bool, ticks: u64) -> (f32, f32) {
    let mut sim = SimWorld::new();
    pool(&mut sim, form_drag);
    let raft = raft(&mut sim, compound, sensor_part);
    let mut bridge = PhysicsBridge::new();
    for t in 0..=ticks {
        bridge.dispatch(&mut sim, true, t);
    }
    let t = ph2d_ecs::world_transform(sim.world(), raft).expect("transform");
    (t.translation.y, t.rotation.to_degrees())
}

/// **A jangada composta NÃO CAPOTA** — o gate central da wave.
///
/// ⚠️ **O oráculo é a INCLINAÇÃO, e a intuição erra o sintoma.** Meia-força faria
/// esperar *"afunda o dobro"*; o que o empuxo lido de UMA forma produz é uma
/// força **descentrada**, logo torque, logo o barco tomba. Medido antes do
/// conserto: o controle em `0,000°` e a composta em **−90,007°** — de pé.
///
/// *Meia-força e força-no-lugar-errado são defeitos diferentes, e este é o
/// segundo.*
#[test]
fn a_compound_raft_floats_level_like_the_single_shaped_one() {
    let (y_one, tilt_one) = settle(false, false, false);
    let (y_two, tilt_two) = settle(true, false, false);
    assert!(
        tilt_one.abs() < 1.0,
        "o CONTROLE inclinou {tilt_one:.3}deg: nada neste gate e' atribuivel"
    );
    assert!(
        tilt_two.abs() < 1.0,
        "a jangada COMPOSTA inclinou {tilt_two:.3}deg -- o empuxo nasceu \
         descentrado, ou seja a zona so' enxergou uma das formas"
    );
    // E ela boia na MESMA linha d'água: mesma silhueta, mesma massa.
    assert!(
        (y_two - y_one).abs() < 0.01,
        "linha d'agua diferente: controle {y_one:.4}, composta {y_two:.4}. \
         Metade da area submersa (a zona ve' uma forma) ou o DOBRO da forca \
         (a zona aplica uma vez por PAR de colliders) dao os dois um delta aqui"
    );
}

/// **O empuxo é aplicado UMA vez por CORPO, não uma por par de colliders.**
///
/// ⚠️ Este gate existe porque o defeito era **invisível por compensação**: com o
/// empuxo lendo só a primeira forma, uma jangada de duas metades iguais recebia
/// `2 × meia-força` = a força certa, por acidente aritmético. Consertar só uma
/// das metades faz a jangada boiar com METADE da submersão — *meia correção é
/// pior que nenhuma*.
///
/// O oráculo é a submersão ABSOLUTA, prevista pela física e não copiada do
/// produto: `ρ_f · A_sub = ρ_b · A_total` com `ρ_f = 4·ρ_b` dá `A_sub = A/4`,
/// logo `0,5/4 = 0,125` de calado e o centro em `+0,125` (a superfície é `y=0`).
#[test]
fn the_waterline_is_the_one_archimedes_predicts() {
    for compound in [false, true] {
        let (y, _) = settle(compound, false, false);
        assert!(
            (y - 0.125).abs() < 0.01,
            "composta={compound}: centro em {y:.4}, Arquimedes diz 0,125. \
             ~0,1876 e' forca DOBRADA; ~0,0625 e' meia forca"
        );
    }
}

/// **Um SENSOR não desloca fluido** — a pergunta que o `.first()` escondia.
///
/// Um sensor é marcador, não matéria: ele atravessa tudo por definição, e o
/// pé-sensor de um personagem (W-PartSensor) daria empuxo a um pedaço de nada.
/// Com o convés marcado como sensor a jangada boia como se só tivesse a metade
/// sólida — o que é o modelo, não um defeito.
#[test]
fn a_sensor_part_displaces_no_fluid() {
    let (y_solid, _) = settle(true, false, false);
    let (y_sensor, _) = settle(true, true, false);
    assert!(
        y_sensor < y_solid - 0.05,
        "o convés SENSOR deu empuxo: solido {y_solid:.4}, sensor {y_sensor:.4} \
         (esperado bem mais fundo, porque so' metade do corpo desloca fluido)"
    );
}

/// **O shape drag age nos LUGARES certos** — a irmã do empuxo.
///
/// ⚠️ **O primeiro corte deste gate media o REPOUSO e a mutação SOBREVIVEU:** o
/// arrasto de forma só age enquanto o corpo se move, então um oráculo tirado
/// depois de a jangada assentar mede exatamente a janela em que a coisa vigiada
/// não acontece. Agora a jangada é LANÇADA e o gate lê a inclinação enquanto ela
/// atravessa a água.
///
/// ⚠️ **E a afirmação é sobre o LUGAR, não sobre a magnitude** — ver
/// `shapes::displaces` para o custo que este modelo aceita: um corpo composto tem
/// arestas INTERNAS (a emenda entre as duas metades) que encaram o escoamento
/// como qualquer outra, então ele resiste MAIS que o corpo de silhueta idêntica
/// feito de uma peça. É over-count honesto e nomeado; o que `.first()` fazia era
/// pior nos dois eixos (resistia de menos **e** no lugar errado).
#[test]
fn the_shape_drag_of_a_compound_body_pushes_at_the_right_places() {
    let tilt_one = launched_tilt(false);
    let tilt_two = launched_tilt(true);
    assert!(
        tilt_one.abs() < 1.0,
        "o CONTROLE girou {tilt_one:.3}deg atravessando a agua: nada e' atribuivel"
    );
    assert!(
        tilt_two.abs() < 1.0,
        "a jangada COMPOSTA girou {tilt_two:.3}deg -- o arrasto de forma agiu \
         sobre uma das metades sozinha, e uma forca descentrada e' um torque"
    );
}

/// Lança a jangada de lado dentro da poça e devolve a inclinação **enquanto ela
/// ainda se move** — a janela em que o arrasto de forma existe.
fn launched_tilt(compound: bool) -> f32 {
    let mut sim = SimWorld::new();
    pool(&mut sim, true);
    let raft = raft(&mut sim, compound, false);
    sim.world_mut()
        .entity_mut(raft)
        .insert(ph2d_physics_ecs::InitialVelocity {
            linvel: [6.0, 0.0],
            angvel: 0.0,
        });
    let mut bridge = PhysicsBridge::new();
    for t in 0..=45u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    ph2d_ecs::world_transform(sim.world(), raft)
        .expect("transform")
        .rotation
        .to_degrees()
}

/// **A secção do arrasto de AR é a da UNIÃO das formas** — o terceiro sítio da
/// mesma premissa, e o mais fácil de errar em silêncio porque não há zona
/// nenhuma envolvida: é um `PhysicsSettings` global.
///
/// ⚠️ **O oráculo é EXATO nesta fixture**, e é por isso que ela foi escolhida: a
/// caixa envolvente das duas metades é **idêntica** à da caixa única (ambas
/// `2,4 × 0,5`), logo a secção característica é o MESMO número e os dois corpos
/// têm de desacelerar igual. Ler só a primeira forma dá ao composto a secção de
/// um braço (`0,85` contra `1,45`), e ele atravessa o ar mais longe.
#[test]
fn the_air_drag_section_is_the_union_of_the_shapes() {
    use ph2d_physics_ecs::PhysicsSettings;

    let travel = |compound: bool| {
        let mut sim = SimWorld::new();
        // Sem chão e sem zona: só ar. A gravidade é zerada para o gate medir a
        // desaceleração HORIZONTAL e nada mais.
        let raft = raft(&mut sim, compound, false);
        sim.world_mut()
            .entity_mut(raft)
            .insert(ph2d_physics_ecs::InitialVelocity {
                linvel: [12.0, 0.0],
                angvel: 0.0,
            });
        let x0 = ph2d_ecs::world_transform(sim.world(), raft)
            .expect("transform")
            .translation
            .x;
        let mut bridge = PhysicsBridge::new();
        bridge.set_settings(PhysicsSettings {
            gravity_x: 0.0,
            gravity_y: 0.0,
            air_drag: 3.0,
            ..PhysicsSettings::default()
        });
        for t in 0..=120u64 {
            bridge.dispatch(&mut sim, true, t);
        }
        ph2d_ecs::world_transform(sim.world(), raft)
            .expect("transform")
            .translation
            .x
            - x0
    };

    let (one, two) = (travel(false), travel(true));
    assert!(
        (two - one).abs() < 0.02,
        "o composto viajou {two:.4} e o controle {one:.4}: a secção do ar não é \
         a da união (ler só a primeira forma dá 0,85 contra 1,45, e o composto \
         vai mais longe)"
    );
}

/// **A sonda que abriu a wave.** `cargo test -p ph2d-physics-ecs --release
/// --test compound_zone -- --ignored --nocapture`
///
/// ⚠️ Ela mora AQUI, sobre a fixture dos gates, e não num arquivo próprio: a
/// primeira versão era um `measure_*.rs` com uma cópia quase idêntica da mesma
/// jangada, e duas fixtures para um fenômeno divergem no dia em que alguém
/// afina uma delas.
///
/// O que ela mostra e os gates não: a evolução no TEMPO, que é o que separa
/// *"assentou aqui"* de *"ainda está indo"* — foi ela que provou que o
/// `0,1876` era equilíbrio estável e não transiente.
#[test]
#[ignore = "sonda de medição"]
fn probe_compound_zone() {
    println!("\n=== a zona vê o corpo composto inteiro? ===");
    println!("  (mesma silhueta 2,40 x 0,50, mesma massa 1,200000)");
    for ticks in [360u64, 900, 2400, 6000] {
        let one = settle_at(false, false, false, ticks);
        let two = settle_at(true, false, false, ticks);
        println!(
            "  t={ticks:>4}   UMA: y {:>7.4} inclinacao {:>8.3}deg   \
             DUAS: y {:>7.4} inclinacao {:>8.3}deg",
            one.0, one.1, two.0, two.1
        );
    }
    println!(
        "\n  Arquimedes prevê o centro em 0,1250. ~0,1876 e' forca DOBRADA\n  \
         (a zona aplica uma vez por PAR de colliders); ~0,0625 e' meia forca."
    );
}

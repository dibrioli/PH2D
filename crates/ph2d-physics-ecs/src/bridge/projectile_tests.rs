//! Gates da ponte do projéctil — a lei contra a `rapier` de verdade.

use super::*;
use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform};
use ph2d_projectile::ProjectileLaw;

const DT: f32 = 1.0 / 60.0;

fn cena(law: ProjectileLaw, alvo: u64, em: Vec2, angulo: f32) -> (SimWorld, PhysicsBridge, Entity) {
    let mut sim = SimWorld::new();
    let mut t = Transform::from_translation(em);
    t.rotation = angulo;
    let quem = sim
        .world_mut()
        .spawn((
            Name::new("Bala"),
            crate::RigidBody {
                kind: crate::BodyKind::Kinematic,
            },
            crate::Collider {
                shape: crate::ColliderShape::Ball { radius: 0.1 },
                ..crate::Collider::default()
            },
            ProjectileMotion::from_law(law, alvo),
            t,
        ))
        .id();
    (sim, PhysicsBridge::new(), quem)
}

fn parede(sim: &mut SimWorld, em: Vec2, meio: (f32, f32)) {
    sim.world_mut().spawn((
        Name::new("Parede"),
        crate::RigidBody {
            kind: crate::BodyKind::Static,
        },
        crate::Collider {
            shape: crate::ColliderShape::Cuboid {
                half_x: meio.0,
                half_y: meio.1,
            },
            ..crate::Collider::default()
        },
        Transform::from_translation(em),
    ));
}

fn pos(sim: &SimWorld, quem: Entity) -> Vec2 {
    sim.world()
        .get::<Transform>(quem)
        .expect("a bala")
        .translation
}

fn corre(sim: &mut SimWorld, bridge: &mut PhysicsBridge, tiques: u64) {
    for t in 1..=tiques {
        bridge.dispatch(sim, true, t);
    }
}

/// ⭐⭐ **Ele voa sozinho, para onde o corpo está virado.**
#[test]
fn ele_voa_sozinho_para_onde_o_corpo_esta_virado() {
    let (mut sim, mut bridge, quem) = cena(
        ProjectileLaw {
            initial_speed: 6.0,
            ..ProjectileLaw::default()
        },
        0,
        Vec2::new(0.0, 0.0),
        0.0,
    );
    corre(&mut sim, &mut bridge, 30);
    let p = pos(&sim, quem);
    assert!(
        (p.x - 3.0).abs() < 0.05,
        "meio segundo a 6 m/s ⇒ 3 m: {p:?}"
    );
    assert!(p.y.abs() < 1.0e-3, "sem gravidade ele nao cai: {p:?}");
}

/// ⭐⭐⭐ **Ele RICOCHETEIA numa parede, e o orçamento do tique atravessa o salto.**
#[test]
fn ele_ricocheteia_na_parede() {
    let (mut sim, mut bridge, quem) = cena(
        ProjectileLaw {
            initial_speed: 6.0,
            max_bounces: 4,
            bounciness: 1.0,
            ..ProjectileLaw::default()
        },
        0,
        Vec2::new(0.0, 0.0),
        0.0,
    );
    // Uma parede vertical com a face em x = 2.
    parede(&mut sim, Vec2::new(3.0, 0.0), (1.0, 20.0));
    corre(&mut sim, &mut bridge, 60);
    let p = pos(&sim, quem);
    assert!(
        p.x < 0.0,
        "ele tinha de voltar para tras depois do salto: x = {}",
        p.x
    );
}

/// ⭐⭐⭐ **O ALCANCE mata o voo, e conta METROS — não tempo.**
///
/// ⚠️ É a metade que separa este componente do `Lifetime` do #12. O gate corre **duas** balas com
/// rapidezes diferentes e exige que elas morram na mesma DISTÂNCIA, em tiques diferentes.
#[test]
fn o_alcance_conta_metros_e_nao_tempo() {
    let mut onde = Vec::new();
    for rapidez in [4.0_f32, 12.0] {
        let (mut sim, mut bridge, quem) = cena(
            ProjectileLaw {
                initial_speed: rapidez,
                range: 2.0,
                ..ProjectileLaw::default()
            },
            0,
            Vec2::new(0.0, 0.0),
            0.0,
        );
        // Corre até a ponte anunciar o fim, ou 300 tiques.
        let mut t = 0;
        while t < 300 && bridge.projectile_done().is_empty() {
            t += 1;
            bridge.dispatch(&mut sim, true, t);
        }
        assert!(
            !bridge.projectile_done().is_empty(),
            "a {rapidez} m/s ele nunca acabou o voo"
        );
        onde.push((rapidez, t, pos(&sim, quem).x));
    }
    for (rapidez, _, x) in &onde {
        assert!(
            (*x - 2.0).abs() < 0.25,
            "a {rapidez} m/s ele morreu em x = {x} e o alcance e' 2 m"
        );
    }
    assert!(
        onde[0].1 > onde[1].1 * 2,
        "a bala LENTA tem de demorar MUITO mais tiques a percorrer os mesmos metros: {onde:?}"
    );
}

/// ⚠️ **O que o mundo não deixou andar não conta para o alcance** — uma bala barrada não percorreu
/// nada, e cobrar-lhe alcance mata-a mais cedo por ter batido.
#[test]
fn uma_bala_barrada_nao_gasta_alcance() {
    let (mut sim, mut bridge, _) = cena(
        ProjectileLaw {
            initial_speed: 6.0,
            // ⚠️⚠️ **O alcance é CURTO de propósito, e a 1.ª redacção deste gate estava errada.**
            // Com `100 m` e 120 tiques, um produto que contasse o orçamento PEDIDO em vez do
            // ANDADO acumularia `0,1 m` por tique ⇒ `12 m`, muito abaixo do alcance — e o gate
            // passava sobre o defeito. *Uma fixtura que não consegue produzir o sujeito do próprio
            // teste passa sempre.* Com `4 m`: o produto honesto anda `~1,4 m` até à parede e nunca
            // lá chega; o defeito chega aos `4` em `40` tiques.
            range: 4.0,
            // ⚠️ Sem saltos ela ACABA ao bater — o que se mede aqui é o alcance, então o voo tem
            // de sobreviver ao toque: um tecto alto e uma parede à frente.
            max_bounces: 200,
            bounciness: 0.0,
            ..ProjectileLaw::default()
        },
        0,
        Vec2::new(0.0, 0.0),
        0.0,
    );
    parede(&mut sim, Vec2::new(1.5, 0.0), (1.0, 20.0));
    // ⚠️⚠️ **O canal é do DISPATCH, não do quadro** — ele é limpo a cada tique, e uma leitura no
    // FIM de 120 tiques mede o último instante, nunca a corrida. A 1.ª redacção deste gate lia-o
    // no fim e uma mutação sobreviveu: o defeito matava a bala ao tique `40` e o canal já estava
    // vazio ao `120`. *Um canal por-tique lê-se a cada tique.*
    let mut morreu = false;
    for t in 1..=120 {
        bridge.dispatch(&mut sim, true, t);
        morreu |= !bridge.projectile_done().is_empty();
    }
    assert!(
        !morreu,
        "ela morreu de alcance encostada a uma parede — o alcance esta' a contar o PEDIDO, nao o \
         ANDADO"
    );
}

/// ⭐⭐ **A flecha aponta para onde voa** — e num arco ela desce com a trajectória.
#[test]
fn a_flecha_aponta_para_onde_voa() {
    let (mut sim, mut bridge, quem) = cena(
        ProjectileLaw {
            initial_speed: 6.0,
            gravity: 20.0,
            face_velocity: true,
            ..ProjectileLaw::default()
        },
        0,
        Vec2::new(0.0, 0.0),
        0.0,
    );
    corre(&mut sim, &mut bridge, 30);
    let a = sim.world().get::<Transform>(quem).expect("a bala").rotation;
    assert!(
        a < -0.3,
        "no arco a flecha tem de apontar para BAIXO: {a} rad"
    );
}

/// ⭐⭐ **Ele persegue um alvo NOMEADO.**
///
/// ⚠️ O alvo é o `stable_name_id`, e é isso que o faz sobreviver a um `Ctrl+Z`.
///
/// ⚠️⚠️ **A régua é a APROXIMAÇÃO contra um CONTROLO, e não a posição num instante.** A 1.ª
/// redacção media `y > 1` ao fim de 40 tiques e lia `−0,017` sobre um homing que estava a
/// funcionar: com aceleração forte a bala **ultrapassa o alvo e volta**, então uma fotografia num
/// instante mede a fase da órbita, não a perseguição. *O que a perseguição É: chegar mais perto do
/// que a mesma bala sem ela.*
#[test]
fn ele_persegue_um_alvo_nomeado() {
    const ALVO: Vec2 = Vec2::new(3.0, 6.0);
    let perto_de = |accel: f32| {
        let (mut sim, mut bridge, quem) = cena(
            ProjectileLaw {
                initial_speed: 6.0,
                homing_accel: accel,
                ..ProjectileLaw::default()
            },
            ph2d_ecs::stable_name_id("Alvo"),
            Vec2::new(0.0, 0.0),
            0.0,
        );
        sim.world_mut()
            .spawn((Name::new("Alvo"), Transform::from_translation(ALVO)));
        let mut minima = f32::INFINITY;
        for t in 1..=60 {
            bridge.dispatch(&mut sim, true, t);
            let p = pos(&sim, quem);
            minima = minima.min(((p.x - ALVO.x).powi(2) + (p.y - ALVO.y).powi(2)).sqrt());
        }
        minima
    };
    let com = perto_de(300.0);
    let sem = perto_de(0.0);
    assert!(
        com < sem * 0.5,
        "a perseguicao tinha de o levar MUITO mais perto: {com:.3} m com, {sem:.3} m sem"
    );
    // ⚠️ **A barra é MEDIDA, não escolhida** (§0.0): nesta fixtura a aproximação mínima é
    // `0,635 m` — a bala chega ao alvo em arco e o ponto mais perto é onde ela vira. A folga até
    // `1,0` é o que separa *«ela chega lá»* de *«ela passa ao largo»* (sem perseguição são
    // `5,89 m`), e apertá-la abaixo do medido seria calibrar no ruído da fixtura.
    assert!(com < 1.0, "e perto de verdade: {com:.3} m (medido 0,635)");
}

/// ⚠️ **Um alvo que não existe não parte nada** — o campo fica escrito no ficheiro quando alguém
/// apaga o objecto, e um `unwrap` ali mataria o quadro.
#[test]
fn um_alvo_que_nao_existe_nao_parte_nada() {
    let (mut sim, mut bridge, quem) = cena(
        ProjectileLaw {
            initial_speed: 6.0,
            homing_accel: 400.0,
            ..ProjectileLaw::default()
        },
        ph2d_ecs::stable_name_id("NinguemComEsteNome"),
        Vec2::new(0.0, 0.0),
        0.0,
    );
    corre(&mut sim, &mut bridge, 30);
    let p = pos(&sim, quem);
    assert!(p.x.is_finite() && p.y.is_finite() && p.x > 2.0, "{p:?}");
}

/// ⭐⭐⭐ **A memória ATRAVESSA os tiques** — sem isso a bala re-nasce a cada quadro.
///
/// # ⚠️⚠️ A 1.ª redacção deste gate media a grandeza FRACA, e uma mutação sobreviveu
///
/// Ela pedia *«caiu mais de `1,5 m`»*, e a bala com a memória apagada cai **`1,93 m`** — passa. O
/// mecanismo que a salva é subtil e vale a pena escrever: com `face_velocity` ligado, a ponte
/// escreve o ÂNGULO do corpo a cada tique, e o re-nascimento do tique seguinte lê esse ângulo. ⇒
/// **a rotação é uma SEGUNDA memória de direcção**, e ela mascara a perda da primeira.
///
/// ⭐ A régua que separa os dois é uma **LEI**, não um limiar: a gravidade só toca em `y`, logo o
/// `x` de um voo com memória é **exactamente** `v·t`. Com a memória apagada o corpo roda para baixo
/// e o `x` fica para trás (`2,20` contra `3,00`). E a queda tem o valor discreto EXACTO da soma
/// `g·dt²·n(n+1)/2` — `2,5833`, e não o `½gt²` contínuo.
#[test]
fn a_memoria_do_voo_atravessa_os_tiques() {
    const G: f32 = 20.0;
    const V: f32 = 6.0;
    const N: u64 = 30;
    let (mut sim, mut bridge, quem) = cena(
        ProjectileLaw {
            initial_speed: V,
            gravity: G,
            ..ProjectileLaw::default()
        },
        0,
        Vec2::new(0.0, 0.0),
        0.0,
    );
    corre(&mut sim, &mut bridge, N);
    let p = pos(&sim, quem);

    // ⭐ **A gravidade não toca em `x`** — isto é lei, e o produto acerta-a ao milésimo.
    let esperado_x = V * (N as f32) * DT;
    assert!(
        (p.x - esperado_x).abs() < 1.0e-3,
        "o `x` de um voo com memoria e' EXACTAMENTE v·t: {:.4} contra {esperado_x:.4}.\n\
         ⚠️ Se ele ficou para tras, a memoria da velocidade nasce fresca e quem conduz a \
         direccao passou a ser a ROTACAO.",
        p.x
    );
    // ⭐ E a queda é a soma DISCRETA exacta, não `½gt²`: `g·dt²·(1+2+…+n)`.
    #[allow(clippy::cast_precision_loss)]
    let esperado_y = -G * DT * DT * (N * (N + 1) / 2) as f32;
    assert!(
        (p.y - esperado_y).abs() < 1.0e-3,
        "a queda tem de ser a soma discreta exacta: {:.4} contra {esperado_y:.4}",
        p.y
    );
}

/// ⭐⭐⭐ **Uma morte num tique do MEIO de uma moldura sobrevive até quem a lê.**
///
/// # ⛔⛔ O canal é do DISPATCH, e o código limpava-o por TIQUE
///
/// Uma moldura pode dever vários tiques (o relógio anda mais depressa que o ecrã, ou o artista
/// arrasta a régua). Com a limpeza dentro do `drive_projectiles` — uma chamada por **passo** — a
/// morte do tique `8` era apagada pelo tique `9`, e a shell, que lê o canal **uma vez por
/// moldura**, nunca a via: ⇒ **uma bala transitória ficava na cena para sempre**, com a lei, o
/// alcance e os 17 gates da wave todos certos.
///
/// ⚠️ É o irmão exacto do `accumulate_joint_breaks`, cujo doc já escreve a mesma frase sobre o
/// mesmo laço (*«o wrapper limpa a própria lista a cada `step`, então uma rotura num tique inicial
/// já desapareceu no último»*) — *a lei estava escrita ao lado, para o vizinho.*
///
/// **Mutação que deve sangrar:** devolver o `self.projectile_done.clear();` ao topo do
/// `drive_projectiles`.
#[test]
fn uma_morte_no_meio_da_moldura_chega_a_quem_a_le() {
    let (mut sim, mut bridge, quem) = cena(
        ProjectileLaw {
            initial_speed: 8.0,
            // 1 m a 8 m/s ⇒ o voo acaba ao tique 8 (`0,1333 s`), bem dentro da moldura de 60.
            range: 1.0,
            ..ProjectileLaw::default()
        },
        0,
        Vec2::new(0.0, 0.0),
        0.0,
    );
    // ⚠️ **UM dispatch que deve 60 tiques** — e é este o sujeito: com `corre()`, um tique por
    // moldura, o defeito é invisível porque a leitura acontece no tique em que a morte ocorre.
    bridge.dispatch(&mut sim, true, 60);

    let mortos: Vec<Entity> = bridge.projectile_done().iter().map(|(e, _)| *e).collect();
    assert!(
        mortos.contains(&quem),
        "a bala morreu ao tique ~8 de uma moldura de 60 e o anúncio nao chegou ao fim dela: \
         {mortos:?} — quem lê o canal apaga a copia da cena, e ela ficaria la' para sempre"
    );
    // E o controlo: ela de facto parou dentro do alcance, senao o gate mede outra coisa.
    let p = pos(&sim, quem);
    assert!(
        (p.x - 1.0).abs() < 0.15,
        "o alcance e' 1 m e ela parou em {p:?}"
    );
}

/// ⭐⭐ **A etiqueta «o voo acabou» é um FACTO, não um acontecimento.**
///
/// # ⚠️⚠️ Um evento lido como estado acerta pelo tempo que ninguém o apagar
///
/// O Inspector pinta *«The flight is over — rewind to launch it again»*, e lia o **canal de
/// morte**, que é de UM dispatch. ⇒ a etiqueta dependia de o relógio estar a andar: **parado** ela
/// ficava (nada limpava o canal) e **a andar** sumia no quadro seguinte — e depois de a limpeza do
/// canal passar a ser por dispatch (a cura da morte no meio da moldura), sumiria nos dois.
///
/// Este gate mede a diferença nos DOIS sentidos, e é a segunda metade que o torna honesto: sem
/// ela, um `projectiles_finished` que devolvesse toda a gente passaria.
///
/// **Mutação que deve sangrar:** `self.projectile_state.iter().map(|(&e, _)| e)` (sem o filtro) ⇒
/// a 2.ª asserção; devolver o canal do evento ⇒ a 1.ª.
#[test]
fn o_voo_acabado_le_se_igual_com_o_relogio_parado_ou_a_andar() {
    let (mut sim, mut bridge, quem) = cena(
        ProjectileLaw {
            initial_speed: 8.0,
            range: 1.0,
            ..ProjectileLaw::default()
        },
        0,
        Vec2::new(0.0, 0.0),
        0.0,
    );
    let acabados = |b: &PhysicsBridge| -> Vec<Entity> { b.projectiles_finished().collect() };

    corre(&mut sim, &mut bridge, 60);
    assert!(
        acabados(&bridge).contains(&quem),
        "ao tique 60 o voo de 1 m ja' acabou"
    );

    // ⚠️ **O relógio ANDA e a morte já é velha** — o canal do evento está vazio (ninguém morreu
    // neste dispatch) e o facto continua a ser verdade.
    bridge.dispatch(&mut sim, true, 90);
    assert!(
        bridge.projectile_done().is_empty(),
        "o canal do EVENTO nao anuncia uma morte velha: {:?}",
        bridge.projectile_done()
    );
    assert!(
        acabados(&bridge).contains(&quem),
        "com o relogio a andar a etiqueta desapareceu — era o evento a ser lido como estado"
    );

    // E PARADO, que era o único sítio onde a leitura antiga acertava.
    bridge.dispatch(&mut sim, false, 90);
    assert!(acabados(&bridge).contains(&quem), "parado tambem");

    // ⭐ E rebobinar apaga-a — o voo vai recomeçar, e a etiqueta que dizia o contrário mandaria o
    // artista rebobinar outra vez.
    bridge.dispatch(&mut sim, false, 0);
    assert!(
        acabados(&bridge).is_empty(),
        "depois do Reset nenhum voo acabou: {:?}",
        acabados(&bridge)
    );
}

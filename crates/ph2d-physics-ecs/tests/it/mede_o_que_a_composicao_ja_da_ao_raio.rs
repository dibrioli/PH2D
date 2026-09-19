//! ⭐⭐⭐ **A PERGUNTA DE ANTES DE QUALQUER LINHA do suplente #21 (`RaySensor`)**:
//! *um colisor SENSOR fino já é um raio?*
//!
//! `CLAUDE.md` §5.0: **antes de construir um item de lista aberta, MEÇA se a composição já o
//! exprime** — e esta linha tem o precedente fresco: o **#3 `SensorZone`** fechou **por
//! composição** em 17/09, e a leitura por `grep` **não** bastou; o veredito saiu de CORRER a cena.
//!
//! ⚠️ **Sonda, não gate.** Ela corre com `--ignored` e IMPRIME; o que ela decide é se o componente
//! tem razão de existir, e o QUÊ dele.
//!
//! ```text
//! cargo test -p ph2d-physics-ecs --test it mede_o_que_a_composicao_ja_da_ao_raio \
//!     -- --ignored --nocapture
//! ```
//!
//! # As perguntas, uma por bloco
//!
//! A) **ORDEM** — com DUAS paredes na linha, a composição diz qual está mais perto?
//! B) **MÉTRICA** — ela diz a QUE DISTÂNCIA, em que PONTO, com que NORMAL?
//! C) **DIRECÇÃO** — ela distingue uma parede À FRENTE de uma ATRÁS?
//! D) **O motor** — o que o `cast_ray` responde às mesmas três perguntas.
//! E) **O CENSO** — quem chega ao `cast_ray` hoje, contado do ficheiro e não de cabeça.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, PhysicsBridge, RigidBody};

/// Uma parede estática, com nome — é pelo nome que a impressão fica legível.
fn parede(sim: &mut SimWorld, nome: &str, em: Vec2, meio: (f32, f32)) -> Entity {
    sim.world_mut()
        .spawn((
            Name::new(nome),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: meio.0,
                    half_y: meio.1,
                },
                ..Collider::default()
            },
            Transform::from_translation(em),
        ))
        .id()
}

/// **A tentativa HONESTA de fazer um raio com o que a casa já tem:** uma barra fina marcada
/// `is_sensor`, deitada ao longo da linha que se quer sondar.
///
/// ⚠️ Ela é a melhor composição disponível, e não um espantalho: o #3 fechou exactamente assim
/// (sensor + `SignalOnHit` + `SignalTagFilter`), e a cena `PH2D_TAGS_SMOKE=2` prova-o.
fn barra_sensor(sim: &mut SimWorld, nome: &str, centro: Vec2, meio: (f32, f32)) -> Entity {
    sim.world_mut()
        .spawn((
            Name::new(nome),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: meio.0,
                    half_y: meio.1,
                },
                is_sensor: true,
                ..Collider::default()
            },
            Transform::from_translation(centro),
        ))
        .id()
}

fn nome_de(sim: &SimWorld, e: Entity) -> String {
    sim.world()
        .get::<Name>(e)
        .map(|n| n.as_str().to_string())
        .unwrap_or_else(|| format!("{e:?}"))
}

#[test]
#[ignore = "sonda de medição — corre à mão, imprime"]
fn mede_o_que_a_composicao_ja_da_ao_raio() {
    println!("\n════════ A) ORDEM — duas paredes na linha ════════");
    // A barra cobre `x ∈ [-0,25 ; 8,25]`, logo apanha as DUAS paredes de uma vez.
    let mut sim = SimWorld::new();
    let perto = parede(
        &mut sim,
        "Parede PERTO (x=2)",
        Vec2::new(2.0, 0.0),
        (0.25, 1.0),
    );
    let longe = parede(
        &mut sim,
        "Parede LONGE (x=5)",
        Vec2::new(5.0, 0.0),
        (0.25, 1.0),
    );
    let barra = barra_sensor(&mut sim, "Barra", Vec2::new(4.0, 0.0), (4.25, 0.05));
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, true, 30);

    let acesos = bridge.triggered_sensors();
    println!(
        "  sensores ACESOS: {:?}",
        acesos.iter().map(|e| nome_de(&sim, *e)).collect::<Vec<_>>()
    );
    println!(
        "  a barra acendeu? {}",
        if acesos.contains(&barra) {
            "SIM"
        } else {
            "NAO"
        }
    );
    println!(
        "  ⇒ a porta devolve ENTIDADES ACESAS e nada mais: {} elemento(s), e NENHUM diz\n     \
         qual das duas paredes esta' mais perto (as duas estao la' dentro).",
        acesos.len()
    );
    let _ = (perto, longe);

    println!("\n════════ B) MÉTRICA — distancia, ponto, normal ════════");
    let contactos = bridge.contacts();
    println!(
        "  contactos DE PE' neste tique: {} — e um SENSOR nao gera contacto (ele atravessa),\n     \
         logo o canal que traz `point`/`normal`/`impulse` esta' VAZIO para a barra.",
        contactos.len()
    );
    println!(
        "  {}",
        if contactos.is_empty() {
            "  ⇒ CONFIRMADO: zero contactos. A metrica nao existe por esta rota."
        } else {
            "  ⇒ ATENCAO: ha' contactos — reler antes de concluir."
        }
    );

    println!("\n════════ C) DIRECÇÃO — uma parede ATRÁS ════════");
    let mut sim2 = SimWorld::new();
    let _frente = parede(
        &mut sim2,
        "Parede A FRENTE (x=2)",
        Vec2::new(2.0, 0.0),
        (0.25, 1.0),
    );
    let _atras = parede(
        &mut sim2,
        "Parede ATRAS (x=-3)",
        Vec2::new(-3.0, 0.0),
        (0.25, 1.0),
    );
    // A MESMA barra centrada na origem: ela é simétrica, logo alcança os dois lados.
    let barra2 = barra_sensor(&mut sim2, "Barra", Vec2::new(0.0, 0.0), (4.0, 0.05));
    let mut b2 = PhysicsBridge::new();
    b2.dispatch(&mut sim2, true, 30);
    println!(
        "  a barra acendeu? {} — e ela alcanca `x ∈ [-4 ; 4]`, logo apanha a de TRAS tambem.",
        if b2.triggered_sensors().contains(&barra2) {
            "SIM"
        } else {
            "NAO"
        }
    );
    println!(
        "  ⇒ uma FORMA e' simetrica por construcao. Para so' olhar em frente e' preciso\n     \
         desloca-la meio comprimento — e ai' o ALCANCE deixa de ser um numero e passa a ser\n     \
         geometria autorada em DOIS sitios (meia-largura E deslocamento)."
    );

    println!("\n════════ D) O MOTOR — o que o `cast_ray` responde ════════");
    {
        use ph2d_physics::PhysicsWorld;
        let mut w = PhysicsWorld::new();
        let (a, _) = w.add_static_cuboid(2.0, 0.0, 0.25, 1.0);
        let (b, _) = w.add_static_cuboid(5.0, 0.0, 0.25, 1.0);
        let (_t, _) = w.add_static_cuboid(-3.0, 0.0, 0.25, 1.0);
        w.step();
        match w.cast_ray([0.0, 0.0], [1.0, 0.0], 20.0, None, 0) {
            Some(h) => println!(
                "  ORDEM  : acertou {} (o MAIS PERTO dos dois: perto={:?} longe={:?})\n  \
                 METRICA: distancia={:.4}  ponto=({:.4}, {:.4})  normal=({:.4}, {:.4})\n  \
                 DIRECCAO: a parede ATRAS (x=-3) esta' dentro do alcance e NAO foi devolvida.",
                if h.body == Some(a) {
                    "a PERTO"
                } else if h.body == Some(b) {
                    "a LONGE"
                } else {
                    "OUTRA"
                },
                a,
                b,
                h.distance,
                h.point[0],
                h.point[1],
                h.normal[0],
                h.normal[1]
            ),
            None => println!("  ⛔ o raio nao acertou nada — reler a fixtura antes de concluir."),
        }
    }

    println!("\n════════ E) O CENSO — quem chega ao `cast_ray` hoje ════════");
    {
        let raiz = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut consumidores: Vec<String> = Vec::new();
        let mut pilha = vec![raiz.join("crates"), raiz.join("shells")];
        while let Some(d) = pilha.pop() {
            let Ok(it) = std::fs::read_dir(&d) else {
                continue;
            };
            for e in it.flatten() {
                let p = e.path();
                if p.is_dir() {
                    pilha.push(p);
                    continue;
                }
                if p.extension().is_none_or(|x| x != "rs") {
                    continue;
                }
                let nome = p
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                // ⛔ Fora os TESTES e a própria porta: o que se conta é quem a CONSOME em produto.
                if nome.ends_with("_tests.rs")
                    || nome == "cast.rs"
                    || p.to_string_lossy().contains("/tests/")
                {
                    continue;
                }
                let Ok(s) = std::fs::read_to_string(&p) else {
                    continue;
                };
                let n = s.matches(".cast_ray(").count() + s.matches(".cast_ray_skipping(").count();
                if n > 0 {
                    consumidores.push(format!(
                        "{n}× {}",
                        p.strip_prefix(&raiz).unwrap_or(&p).display()
                    ));
                }
            }
        }
        consumidores.sort();
        println!(
            "  chamadas de produto ({} ficheiro(s)):",
            consumidores.len()
        );
        for c in &consumidores {
            println!("    {c}");
        }
        println!(
            "  ⇒ o doc da porta diz «o `cast_ray` tem exactamente CINCO consumidores no repo\n     \
             inteiro — o sensor de chao, os dois de teto, o de headroom e o de parede», e os\n     \
             cinco vivem DENTRO do platformer. Nenhum componente autoravel lhe chega."
        );
    }
    println!();
}

//! Os gates da cena do IMPACTO — ver o cabeçalho de [`super`].
//!
//! ⚠️ **Eles correm a LEI com os números DA CENA** — cada inimigo e a bala são a CÓPIA que a fábrica
//! faria do molde que o `montar` deixou (a porta de cópia do produto). *Escrever os números à mão
//! aqui mediria uma cena que o dono não vê.* ⛔ O clique e a fábrica são do dono
//! (`PH2D_VIDA_SMOKE=2`).

use super::*;
use crate::smoke_copia::copia;
use ph2d_ecs::SimWorld;
use ph2d_physics_ecs::{PhysicsBridge, ProjectileMotion};

fn mundo() -> SimWorld {
    let mut sim = SimWorld::new();
    let m = crate::vida_smoke::montar(sim.world_mut(), 2);
    assert_eq!(m.nivel, 2, "o roteador montou outra cena");
    sim
}

/// Os componentes da CÓPIA de `nome`, prontos a clonar para uma corrida da ponte.
fn inimigo(
    sim: &mut SimWorld,
    nome: &str,
) -> (RigidBody, Collider, GravityScale, DampingOverride, Health) {
    let c = copia(sim, nome);
    let w = sim.world();
    (
        *w.get::<RigidBody>(c).expect("sem corpo"),
        *w.get::<Collider>(c).expect("sem forma"),
        *w.get::<GravityScale>(c)
            .expect("sem a gravidade a zero — ele cairia"),
        *w.get::<DampingOverride>(c)
            .expect("sem arrasto — um empurrão levava-o para fora"),
        w.get::<Health>(c)
            .unwrap_or_else(|| panic!("a CÓPIA de «{nome}» nasceu SEM VIDA"))
            .clone(),
    )
}

/// **Quanto um tiro da bala DA CENA empurra a cópia de `nome`** — a distância que ela percorre
/// depois do golpe, até assentar.
fn quanto_voa(nome: &str) -> f32 {
    let mut cena = mundo();
    let alvo = inimigo(&mut cena, nome);
    let b = copia(&mut cena, "Bala");
    let bala = {
        let w = cena.world();
        (
            *w.get::<RigidBody>(b).unwrap(),
            *w.get::<Collider>(b).unwrap(),
            *w.get::<ProjectileMotion>(b).unwrap(),
            w.get::<Damage>(b).expect("a bala sem dano").clone(),
        )
    };
    let mut sim = SimWorld::new();
    let a = sim
        .world_mut()
        .spawn((
            alvo.0,
            alvo.1,
            alvo.2,
            alvo.3,
            alvo.4,
            Transform::from_translation(Vec2::new(X_INIMIGOS, 0.0)),
        ))
        .id();
    sim.world_mut().spawn((
        bala.0,
        bala.1,
        bala.2,
        bala.3,
        Transform::from_translation(Vec2::new(-2.0, 0.0)),
    ));
    let mut ponte = PhysicsBridge::new();
    for t in 0..=240 {
        ponte.dispatch(&mut sim, true, t);
    }
    sim.world().get::<Transform>(a).unwrap().translation.x - X_INIMIGOS
}

/// ⭐⭐⭐ **O leve VOA, o pesado mal se mexe, o CONTROLO não sai do sítio** — o passo (2)–(4) do
/// roteiro, com os números da cena. ⚠️ A razão pesado/leve é a do `Push Taken` deles (`¼`), e é
/// ela — não a massa — que decide.
///
/// **Mutações que devem sangrar:** o `knockback` da bala a `0` · o `knockback_taken` do controlo a
/// `1` · a cena sem o arrasto (o leve não assenta).
#[test]
fn o_leve_voa_o_pesado_mal_se_mexe_e_o_controlo_fica() {
    let leve = quanto_voa(INIMIGOS[0].nome);
    let pesado = quanto_voa(INIMIGOS[1].nome);
    let controlo = quanto_voa(INIMIGOS[2].nome);
    assert!(leve > 1.0, "o leve não voou à vista: {leve}");
    assert!(
        leve < 4.0,
        "o leve voou demais — sem arrasto ele sai do ecrã: {leve}"
    );
    let razao = pesado / leve;
    assert!(
        (0.2..=0.3).contains(&razao),
        "o pesado tinha de voar ¼ do leve: {pesado} contra {leve}"
    );
    assert!(
        controlo.abs() < 0.05,
        "o CONTROLO saiu do sítio: {controlo}"
    );
}

/// ⭐⭐ **O controlo é o CONTROLO: a mesma vida sem nada do impacto** — e os outros dois têm cada
/// peça que o roteiro nomeia. ⚠️ O piscar só existe com invencibilidade (ele dura a janela dela): um
/// `blink_s` sem ela seria um knob morto.
#[test]
fn cada_inimigo_tem_o_que_o_roteiro_diz() {
    let mut sim = mundo();
    let (.., leve) = inimigo(&mut sim, INIMIGOS[0].nome);
    let (.., pesado) = inimigo(&mut sim, INIMIGOS[1].nome);
    let (.., controlo) = inimigo(&mut sim, INIMIGOS[2].nome);
    for h in [&leve, &pesado] {
        assert!(h.numbers, "o roteiro manda ver o número");
        assert!(
            h.blink_s > 0.0 && h.invincible_s > 0.0,
            "sem janela o piscar é morto"
        );
    }
    assert_eq!(pesado.death_hitstop_s, PAUSA_DA_MORTE);
    assert!(
        pesado.death_hitstop_s > PAUSA,
        "a morte do pesado tem de pesar MAIS que um tiro"
    );
    assert_eq!(
        controlo.max, leve.max,
        "o controlo tem a MESMA vida do leve"
    );
    assert_eq!(controlo.knockback_taken, 0.0);
    assert!(!controlo.numbers);
    assert_eq!(controlo.blink_s, 0.0);
    assert_eq!(controlo.death_hitstop_s, 0.0);
}

/// ⭐⭐ **A bala pesa, o espinho empurra, e o herói pode piscar** — e as equipas fecham: a bala do
/// herói não o fere a ele, o espinho sim.
#[test]
fn a_bala_o_espinho_e_o_heroi_tem_o_impacto() {
    let mut sim = mundo();
    let b = copia(&mut sim, "Bala");
    let d = sim.world().get::<Damage>(b).unwrap().clone();
    assert_eq!((d.knockback, d.hitstop_s), (EMPURRAO, PAUSA));
    let w = sim.world_mut();
    let mut q = w.query::<(&Name, Option<&Health>, Option<&Damage>)>();
    let (mut heroi, mut espinho) = (None, None);
    for (n, h, d) in q.iter(w) {
        match n.as_str() {
            "Heroi" => heroi = h.cloned(),
            "Espinho" => espinho = d.cloned(),
            _ => {}
        }
    }
    let heroi = heroi.expect("o herói sem vida — o passo (5) não tem sujeito");
    let espinho = espinho.expect("o espinho sem dano");
    assert!(heroi.blink_s > 0.0 && heroi.invincible_s > 0.0 && heroi.numbers);
    assert!(espinho.knockback > 0.0);
    assert_ne!(espinho.team, heroi.team, "o espinho tem de ferir o herói");
    assert_eq!(d.team, heroi.team, "a bala do herói não o pode ferir a ele");
}

/// ⚠️ **A cena cabe na banda que a régua deixa visível** (`+4,09` / `−1,19` m, medida na foto da
/// cena da arma) — a coluna, a barra de cima, o herói e o espinho.
#[test]
fn a_cena_cabe_na_banda_visivel() {
    for Inimigo { y, .. } in INIMIGOS {
        assert!(
            y - LADO / 2.0 >= -1.19,
            "um inimigo cai por baixo da banda: {y}"
        );
        assert!(
            y + 0.6 + 0.05 <= 4.09,
            "a barra de um inimigo sai por cima: {y}"
        );
    }
    for y in [HEROI_XY[1], ESPINHO_XY[1]] {
        assert!(
            (-1.19 + 0.4..=4.09 - 0.4).contains(&y),
            "fora da banda: {y}"
        );
    }
    // ⚠️ E o `x` — a metade que a 1.ª redacção deste gate não media (a foto apanhou o herói e o
    // espinho cortados pela borda esquerda) — é ERRO DE COMPILAÇÃO ao lado da `BORDA_ESQUERDA`:
    // duas constantes comparadas não são um teste.
}

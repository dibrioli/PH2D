//! Os gates das duas cenas — o que o dono vai ver tem de estar lá.

use super::*;
use ph2d_ecs::SimWorld;

fn monta(nivel: u32) -> SimWorld {
    let mut sim = SimWorld::new();
    let devolveu = montar(sim.world_mut(), nivel);
    assert_eq!(
        devolveu,
        nivel.clamp(1, CENAS),
        "o roteador devolveu outra cena"
    );
    sim
}

fn nomes(sim: &SimWorld) -> Vec<String> {
    let mut v = Vec::new();
    let mut q = sim.world().try_query::<&Name>().unwrap();
    for n in q.iter(sim.world()) {
        v.push(n.as_str().to_string());
    }
    v
}

fn lei(sim: &SimWorld, nome: &str) -> ProjectileLaw {
    let mut q = sim
        .world()
        .try_query::<(&Name, &ProjectileMotion)>()
        .unwrap();
    for (n, c) in q.iter(sim.world()) {
        if n.as_str() == nome {
            return c.law();
        }
    }
    panic!("falta a bala `{nome}`");
}

/// ⭐⭐⭐ **UM KNOB por bala, e as outras colunas IGUAIS.**
///
/// ⚠️ É a coisa toda desta cena: quatro balas com quatro rapidezes não ensinariam nada — tudo seria
/// diferente de tudo. O gate exige que a rapidez seja a MESMA e que cada bala difira da recta em
/// **exactamente um** campo.
#[test]
fn cada_bala_da_galeria_difere_da_recta_em_um_knob_so() {
    let sim = monta(1);
    let recta = lei(&sim, "Straight");
    assert!((recta.initial_speed - RAPIDEZ).abs() < 1.0e-5);

    for (nome, quantos) in [("Bouncy", 2), ("Arc", 1), ("Short Range", 1)] {
        let l = lei(&sim, nome);
        assert!(
            (l.initial_speed - RAPIDEZ).abs() < 1.0e-5,
            "a `{nome}` tem de sair com a MESMA rapidez da recta"
        );
        let difere = usize::from(l.gravity != recta.gravity)
            + usize::from(l.range != recta.range)
            + usize::from(l.max_bounces != recta.max_bounces)
            + usize::from(l.bounciness != recta.bounciness)
            + usize::from(l.homing_accel != recta.homing_accel)
            + usize::from(l.acceleration != recta.acceleration)
            + usize::from(l.max_speed != recta.max_speed);
        // ⚠️ A `Bouncy` muda DOIS (o tecto e a perda) porque um tecto sem perda ricocheteia para
        // sempre — os dois são o MESMO knob do ponto de vista do artista, e o gate di-lo.
        assert_eq!(
            difere, quantos,
            "a `{nome}` difere da recta em {difere} campos, e devia diferir em {quantos}"
        );
    }
}

/// ⚠️ **A bala recta MORRE na parede** — é o que `Max Bounces = 0` quer dizer, e sem parede não há
/// o que demonstrar.
#[test]
fn a_galeria_tem_a_parede_contra_a_qual_se_atira() {
    let sim = monta(1);
    let n = nomes(&sim);
    assert!(n.iter().any(|s| s == "Wall"), "falta a parede: {n:?}");
    assert_eq!(lei(&sim, "Straight").max_bounces, 0);
    assert!(
        lei(&sim, "Bouncy").max_bounces >= 1,
        "a laranja tem de saltar"
    );
}

/// ⭐⭐ **O ARCO aponta para onde voa** — sem isso a bala azul é um rectângulo a deslizar de lado.
#[test]
fn a_bala_do_arco_aponta_para_onde_voa() {
    let sim = monta(1);
    let l = lei(&sim, "Arc");
    assert!(l.gravity > 0.0, "sem gravidade nao ha' arco");
    assert!(l.face_velocity, "e sem virar, o arco nao se le");
}

/// ⭐⭐⭐ **A cena `=2` tem o CONTROLO ao lado do míssil.**
///
/// ⚠️ Sem ele o artista não distingue *«o homing funciona»* de *«ele ia para lá de qualquer
/// maneira»*, e o gate exige que só a perseguição difira.
#[test]
fn o_missil_tem_um_controlo_com_tudo_igual_menos_a_perseguicao() {
    let sim = monta(2);
    let m = lei(&sim, "Missile");
    let c = lei(&sim, "Control (no homing)");
    assert!(m.homing_accel > 0.0, "o missil tem de perseguir");
    assert!(c.homing_accel == 0.0, "o controlo NAO persegue");
    assert_eq!(
        ProjectileLaw {
            homing_accel: 0.0,
            ..m
        },
        c,
        "so' a perseguicao pode diferir entre o missil e o controlo"
    );
}

/// ⭐⭐ **O alvo ANDA, e é o artista que o conduz** — um alvo parado não distingue *«ele vai até
/// lá»* de *«ele foi disparado para lá»*.
#[test]
fn o_alvo_da_cena_dois_anda_e_e_o_sujeito_da_perseguicao() {
    let sim = monta(2);
    let n = nomes(&sim);
    assert!(n.iter().any(|s| s == "Target"), "falta o alvo: {n:?}");
    let mut q = sim
        .world()
        .try_query::<(&Name, &TopDownPlayer)>()
        .expect("query");
    let mut conduzivel = false;
    for (nome, c) in q.iter(sim.world()) {
        if nome.as_str() == "Target" {
            conduzivel = c.law().default_controls;
        }
    }
    assert!(conduzivel, "o alvo tem de andar com as setas");

    // ⚠️ E as duas balas apontam ao alvo pelo NOME — os bits não sobrevivem a um Ctrl+Z.
    let mut q = sim
        .world()
        .try_query::<(&Name, &ProjectileMotion)>()
        .expect("query");
    for (nome, c) in q.iter(sim.world()) {
        if nome.as_str().starts_with("Missile") || nome.as_str().starts_with("Control") {
            assert_eq!(
                c.homing_target,
                ph2d_ecs::stable_name_id("Target"),
                "a `{}` aponta para outro sitio",
                nome.as_str()
            );
        }
    }
}

/// ⭐⭐⭐ **AS BALAS DESENHAM POR CIMA DO CHÃO** — a lição de 2026-09-15, aplicada a uma cena nova.
///
/// ⚠️ O chão é a primeira raiz criada, logo desenha atrás; se alguém trocar a ordem, as balas
/// somem e a cena ensina que o componente está partido.
#[test]
fn as_balas_desenham_por_cima_do_chao() {
    for nivel in 1..=CENAS {
        let mut sim = monta(nivel);
        ph2d_ecs::assign_missing_root_order(sim.world_mut());
        let mut chaves: Vec<(String, (u32, _))> = Vec::new();
        let mut q = sim
            .world()
            .try_query::<(ph2d_ecs::Entity, &Name)>()
            .expect("query");
        for (e, n) in q.iter(sim.world()) {
            chaves.push((n.as_str().to_string(), ph2d_ecs::root_key(sim.world(), e)));
        }
        let chao = chaves.iter().find(|(n, _)| n == "Floor").expect("o chao").1;
        let balas: Vec<&(String, (u32, _))> = chaves
            .iter()
            .filter(|(n, _)| n != "Floor" && n != "Wall")
            .collect();
        assert!(
            !balas.is_empty(),
            "a cena =${nivel} nao tem nada em cima do chao"
        );
        for (n, k) in balas {
            assert!(
                *k > chao,
                "na cena =${nivel} o `{n}` desenha ATRAS do chao ({k:?} contra {chao:?})"
            );
        }
    }
}

/// Um nível desconhecido cai na primeira cena.
#[test]
fn um_nivel_desconhecido_cai_na_primeira_cena() {
    let mut sim = SimWorld::new();
    assert_eq!(montar(sim.world_mut(), 99), 1);
}

//! Os gates das duas cenas — o que o dono vai ver tem de estar lá.

use super::*;
use ph2d_ecs::SimWorld;

fn monta(nivel: u32) -> SimWorld {
    let mut sim = SimWorld::new();
    let devolveu = montar(sim.world_mut(), nivel);
    assert_eq!(
        devolveu,
        nivel.min(CENAS).max(1),
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

#[test]
fn a_cena_um_tem_o_dente_contra_o_qual_se_encosta() {
    // ⚠️ **O gate nomeia o DENTE**, e não «há paredes»: o recinto sozinho põe o teste na borda do
    // ecrã, e a instrução manda encostar. *Um passo de smoke que aponta para uma coisa tem de
    // provar que a coisa está na cena* (a lei que esta linha já pagou num painel).
    let sim = monta(1);
    let n = nomes(&sim);
    assert!(n.iter().any(|s| s == "Wall Corner"), "falta o dente: {n:?}");
    assert!(n.iter().any(|s| s == "Hero"), "falta o boneco: {n:?}");
}

#[test]
fn o_boneco_da_cena_um_anda_em_oito_direccoes_e_e_cinematico() {
    let sim = monta(1);
    let mut q = sim
        .world()
        .try_query::<(&Name, &TopDownPlayer, &RigidBody)>()
        .unwrap();
    let mut achou = false;
    for (n, c, b) in q.iter(sim.world()) {
        if n.as_str() != "Hero" {
            continue;
        }
        achou = true;
        assert_eq!(c.law().direction, DirectionMode::EightWay);
        assert_eq!(c.law().speed, VELOCIDADE);
        // ⛔ Dinâmico seria do solver, e o componente ficaria sem pose para escrever.
        assert_eq!(
            b.kind,
            BodyKind::Kinematic,
            "o mover escreve a propria pose"
        );
    }
    assert!(achou);
}

#[test]
fn a_cena_dois_tem_o_CONTROLO_ao_lado_do_isometrico() {
    // ⭐ Sem o controlo o artista não distingue «o viewpoint funciona» de «ele anda assim de
    // qualquer maneira» — e o gate exige que os DOIS estejam lá, com viewpoints DIFERENTES.
    let sim = monta(2);
    let mut q = sim.world().try_query::<(&Name, &TopDownPlayer)>().unwrap();
    let mut iso = None;
    let mut ctl = None;
    for (n, c) in q.iter(sim.world()) {
        match n.as_str() {
            "Hero (Isometric)" => iso = Some(c.law()),
            "Control (Top-Down)" => ctl = Some(c.law()),
            _ => {}
        }
    }
    let iso = iso.expect("falta o isometrico");
    let ctl = ctl.expect("falta o CONTROLO");
    assert_eq!(iso.viewpoint, Viewpoint::Isometric2to1);
    assert_eq!(ctl.viewpoint, Viewpoint::TopDown);
    assert_eq!(iso.direction, ctl.direction, "so' o viewpoint pode diferir");
    assert_eq!(iso.speed, ctl.speed, "so' o viewpoint pode diferir");
}

#[test]
fn e_a_mesma_seta_leva_os_dois_a_sitios_DIFERENTES() {
    // ⚠️ **A prova é da LEI, não da cena**: o gate roda a mesma intenção pelos dois componentes e
    // exige que as direcções de mundo divirjam. Sem isto, a cena podia ter dois viewpoints
    // escritos e o movimento ser igual — que é exactamente o que o artista tem de poder descartar.
    let sim = monta(2);
    let mut q = sim.world().try_query::<(&Name, &TopDownPlayer)>().unwrap();
    let mut dirs = Vec::new();
    for (n, c) in q.iter(sim.world()) {
        if n.as_str().starts_with("Hero (") || n.as_str().starts_with("Control (") {
            dirs.push((
                n.as_str().to_string(),
                ph2d_topdown::world_direction([1.0, 0.0], &c.law()),
            ));
        }
    }
    assert_eq!(dirs.len(), 2, "{dirs:?}");
    let (a, b) = (dirs[0].1, dirs[1].1);
    let dist = ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt();
    assert!(
        dist > 0.3,
        "a mesma seta leva os dois ao mesmo sitio: {dirs:?}"
    );
}

#[test]
fn a_grelha_do_tabuleiro_esta_desenhada() {
    // ⛔ Sem ela a isometria é uma afirmação sobre números que ninguém vê.
    let sim = monta(2);
    let n = nomes(&sim);
    let a = n.iter().filter(|s| s.starts_with("Grid A")).count();
    let b = n.iter().filter(|s| s.starts_with("Grid B")).count();
    assert_eq!(a, 13, "as linhas de uma familia do losango");
    assert_eq!(b, 13, "e as da outra");
}

#[test]
fn um_nivel_desconhecido_cai_na_primeira_cena() {
    let mut sim = SimWorld::new();
    assert_eq!(montar(sim.world_mut(), 99), 1);
}

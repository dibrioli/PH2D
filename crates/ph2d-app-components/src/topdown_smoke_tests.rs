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
fn a_cena_dois_tem_o_controlo_ao_lado_do_isometrico() {
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
fn e_a_mesma_seta_leva_os_dois_a_sitios_diferentes() {
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
                ph2d_topdown::world_direction(
                    [1.0, 0.0],
                    &c.law(),
                    // ⚠️ Memória FRESCA: o que este gate mede é o viewpoint, e uma
                    // dominância herdada trocaria o eixo sob a medição.
                    &mut ph2d_topdown::direction::Dominance::default(),
                ),
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

/// ⭐⭐⭐ **O BONECO DESENHA POR CIMA DO CHÃO** (report do dono, 2026-09-15: *«para o Hero ser
/// visível deve ficar abaixo na Hierarchy»*).
///
/// ⚠️⚠️ **Nenhum gate desta cena media a ORDEM DE DESENHO**, e a cena montava-se perfeita: os
/// nomes lá, os números certos, os componentes certos — e um rectângulo cinzento por cima de tudo.
/// O chão é a primeira raiz criada, e a varredura foundational que numera as raízes
/// ([`ph2d_ecs::assign_missing_root_order`]) **invertia** a ordem de criação, logo ele recebia o
/// número mais alto e passava a desenhar à frente.
///
/// ⇒ o gate corre a varredura — que é o que o quadro faz — e lê a pilha pela **porta partilhada**
/// ([`ph2d_ecs::root_key`]), que é a mesma que a lista da Hierarquia e o `propagate_transforms`
/// leem. *Um gate que lesse a ordem de spawn em vez da porta ficaria verde sobre o defeito.*
#[test]
fn o_boneco_desenha_por_cima_do_chao() {
    for nivel in 1..=CENAS {
        let mut sim = monta(nivel);
        // O passe do quadro: é ele que dá número às raízes recém-nascidas.
        ph2d_ecs::assign_missing_root_order(sim.world_mut());

        let mut por_nome: Vec<(String, (u32, _))> = Vec::new();
        let mut q = sim
            .world()
            .try_query::<(ph2d_ecs::Entity, &Name)>()
            .expect("query");
        for (e, n) in q.iter(sim.world()) {
            por_nome.push((n.as_str().to_string(), ph2d_ecs::root_key(sim.world(), e)));
        }
        let chave = |alvo: &str| {
            por_nome
                .iter()
                .find(|(n, _)| n == alvo)
                .unwrap_or_else(|| panic!("falta `{alvo}` na cena =${nivel}"))
                .1
        };
        let chao = chave("Floor");
        // ⚠️ Os nomes dos bonecos MUDAM entre as cenas — lidos do que está lá, nunca escritos duas
        // vezes (a cena 1 tem `Hero`; a 2 tem `Hero (Isometric)` e `Control (Top-Down)`).
        let bonecos: Vec<String> = por_nome
            .iter()
            .map(|(n, _)| n.clone())
            .filter(|n| n.starts_with("Hero") || n.starts_with("Control"))
            .collect();
        assert!(
            !bonecos.is_empty(),
            "a cena =${nivel} nao tem boneco nenhum"
        );
        for b in &bonecos {
            assert!(
                chave(b) > chao,
                "na cena =${nivel} o `{b}` desenha ATRAS do chao ({:?} contra {:?}) — \
                 ele fica invisivel, e o dono ve um rectangulo cinzento",
                chave(b),
                chao
            );
        }
    }
}

/// ⭐⭐ **A cena `=2` é o sítio onde a ÚLTIMA SETA se testa — e o sujeito é o CONTROLO.**
///
/// ⚠️ *Um passo de smoke que aponta para uma coisa tem de provar que a coisa está na cena* (a lei
/// que esta linha já pagou num painel). A instrução manda segurar a `→` e carregar na `↑` **no
/// quadrado cinzento**, então o gate exige que ele exista, que esteja em **4 direcções** — que é o
/// único modo em que a regra vale — e que a cena `=1` **não** esteja, porque ali a diagonal é a
/// resposta certa.
#[test]
fn a_regra_da_ultima_seta_tem_sujeito_na_cena_dois_e_nao_na_um() {
    let sim = monta(2);
    let mut q = sim.world().try_query::<(&Name, &TopDownPlayer)>().unwrap();
    let mut achou = false;
    for (n, c) in q.iter(sim.world()) {
        if n.as_str() != "Control (Top-Down)" {
            continue;
        }
        achou = true;
        assert_eq!(
            c.law().direction,
            DirectionMode::FourWay,
            "o sujeito do passo tem de estar em 4 direccoes — e' o unico modo em que a regra vale"
        );
    }
    assert!(
        achou,
        "falta o CONTROLO, que e' o sujeito do passo do smoke"
    );

    let sim = monta(1);
    let mut q = sim.world().try_query::<&TopDownPlayer>().unwrap();
    for c in q.iter(sim.world()) {
        assert_ne!(
            c.law().direction,
            DirectionMode::FourWay,
            "⛔ a cena =1 e' a do DESLIZE e vive das diagonais — po-la em 4 direccoes apagaria \
             metade dos rumos do passo que o dono ja' aprovou"
        );
    }
}

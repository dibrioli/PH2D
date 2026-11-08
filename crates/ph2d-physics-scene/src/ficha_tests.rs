//! A cena 120 não pode ensinar o contrário do que acontece: cada frase da mensagem, pela ponte real.

use super::*;
use ph2d_ecs::SimWorld;
use ph2d_physics_ecs::{PhysicsBridge, PlayerInput};

struct Corrida {
    sim: SimWorld,
    bridge: PhysicsBridge,
    tick: u64,
}

impl Corrida {
    fn nova() -> Self {
        let mut sim = SimWorld::new();
        monta(sim.world_mut());
        ph2d_physics_ecs::resolve_body_names(sim.world_mut());
        let mut bridge = PhysicsBridge::new();
        bridge.set_settings(regras(PhysicsSettings::default()));
        Self {
            sim,
            bridge,
            tick: 0,
        }
    }

    fn por_nome(&self, nome: &str) -> Entity {
        let mut q = self.sim.world().try_query::<(Entity, &Name)>().unwrap();
        q.iter(self.sim.world())
            .find(|(_, n)| n.0 == nome)
            .map(|(e, _)| e)
            .expect(nome)
    }

    fn at(&self, nome: &str) -> [f32; 2] {
        let t = self
            .sim
            .world()
            .get::<Transform>(self.por_nome(nome))
            .expect("pose");
        [t.translation.x, t.translation.y]
    }

    fn corre(&mut self, quadros: u32) {
        for _ in 0..quadros {
            self.bridge.dispatch(&mut self.sim, true, self.tick);
            self.tick += 1;
        }
    }

    fn dedo(&mut self, drive: f32, jump: bool) {
        let e = self.por_nome("Player");
        self.bridge.set_player_input(
            e,
            PlayerInput {
                drive,
                jump,
                ..PlayerInput::default()
            },
        );
    }
}

/// «Nada segura» a forma solta; o Floor em Repeated segura, no tamanho do desenho.
#[test]
fn o_crate_atravessa_a_forma_solta_e_pousa_no_floor() {
    let mut c = Corrida::nova();
    c.corre(240);
    let y = c.at("Crate")[1];
    assert!(
        (y - 0.5).abs() < 0.05,
        "o Crate pousou no topo do Floor: {y}"
    );
}

/// Controlo da zona: sem ela a bola vai ao chão; com ela nunca lá chega, e sobe depois de entrar.
#[test]
fn a_zona_empurra_a_bola_para_cima() {
    let mut c = Corrida::nova();
    let mut sem = Corrida::nova();
    let z = sem.por_nome("Updraft Zone");
    let z = ph2d_physics_ecs::main_shape(sem.sim.world(), z).expect("a forma");
    sem.sim.world_mut().entity_mut(z).remove::<AreaEffector>();
    sem.corre(300);
    assert!(
        sem.at("Stretched Ball")[1] < 1.2,
        "controlo: sem a força, cai"
    );

    let (mut menor, mut anterior, mut subiu) = (f32::MAX, c.at("Stretched Ball")[1], false);
    for _ in 0..600 {
        c.corre(1);
        let [x, y] = c.at("Stretched Ball");
        assert!(
            (x - ZONE_AT[0]).abs() < ZONE_HALF[0] + 1.0,
            "fica na coluna: {x}"
        );
        menor = menor.min(y);
        subiu |= y > anterior + 1e-3 && y < ZONE_AT[1] + ZONE_HALF[1];
        anterior = y;
    }
    assert!(subiu, "dentro da zona a bola sobe");
    assert!(menor > 1.2, "a bola nunca chega ao Floor: {menor}");
}

/// O Player anda por baixo, salta, atravessa a One-way por baixo e pousa em cima.
#[test]
fn o_player_sobe_pela_one_way_por_baixo_e_pousa() {
    let mut c = Corrida::nova();
    c.corre(60);
    let chao = c.at("Player")[1];
    assert!(
        (chao - HERO_SHEET_SOLE_BELOW_CENTRE).abs() < 0.08,
        "de pé no Floor: {chao}"
    );
    let meio = ONE_WAY_AT[0];
    while c.at("Player")[0] < meio - 0.6 {
        c.dedo(1.0, false);
        c.corre(1);
        assert!(c.tick < 400, "anda até à plataforma");
    }
    c.dedo(0.2, true);
    c.corre(12);
    c.dedo(0.0, false);
    c.corre(120);
    let [x, y] = c.at("Player");
    let topo = ONE_WAY_AT[1] + 0.25 + HERO_SHEET_SOLE_BELOW_CENTRE;
    assert!(
        (y - topo).abs() < 0.1,
        "pousou em cima da One-way: ({x}, {y}), topo {topo}"
    );
}

/// Controlo: sem One-way a borda barra o Player, cuja cabeça passa acima do fundo da plataforma.
#[test]
fn sem_one_way_a_borda_barra_o_player() {
    let mut c = Corrida::nova();
    let p = c.por_nome("One-way Platform");
    let p = ph2d_physics_ecs::main_shape(c.sim.world(), p).expect("a forma");
    c.sim.world_mut().entity_mut(p).remove::<OneWayPlatform>();
    c.corre(60);
    c.dedo(1.0, false);
    c.corre(200);
    let x = c.at("Player")[0];
    assert!(x < ONE_WAY_AT[0] - 1.2, "barrado antes da plataforma: {x}");
}

/// O Player atravessa o Crate: «Hits» do Player só diz Ground, e cada camada tem o seu.
#[test]
fn cada_camada_tem_um_hits_diferente_e_a_matriz_e_simetrica() {
    let nomeadas = &MATRIZ[..CAMADAS.len()];
    for (a, &linha_a) in nomeadas.iter().enumerate() {
        for (b, &linha_b) in nomeadas.iter().enumerate() {
            assert!(a == b || linha_a != linha_b, "camadas {a} e {b}");
            assert_eq!(linha_a >> b & 1, linha_b >> a & 1, "assimétrica {a},{b}");
        }
    }
    let mut c = Corrida::nova();
    c.corre(180);
    let alvo = c.at("Crate")[0];
    let de = c.at("Player")[0];
    while c.at("Player")[0] < alvo + 1.0 {
        c.dedo(1.0, false);
        c.corre(1);
        assert!(
            c.tick < 800,
            "o Crate não pode barrar o Player: x {}",
            c.at("Player")[0]
        );
    }
    assert!(
        (c.at("Crate")[0] - alvo).abs() < 0.01,
        "ele nem o empurra (de {de})"
    );
}

/// A sonda da força da zona: `cargo-test-narrow.sh ph2d-app-physics probe_cena_120 -- --ignored --nocapture`.
#[test]
#[ignore = "sonda de medição"]
fn probe_cena_120() {
    let mut c = Corrida::nova();
    for q in 0..600 {
        c.corre(1);
        if q % 30 == 0 {
            println!(
                "{q} bola {:?} crate {:?}",
                c.at("Stretched Ball"),
                c.at("Crate")
            );
        }
    }
    let mut c = Corrida::nova();
    c.corre(60);
    while c.at("Player")[0] < ONE_WAY_AT[0] - 0.6 {
        c.dedo(1.0, false);
        c.corre(1);
    }
    println!("sob a plataforma {:?} t={}", c.at("Player"), c.tick);
    c.dedo(0.2, true);
    c.corre(12);
    c.dedo(0.2, false);
    for q in 0..80 {
        c.corre(2);
        println!("{q} player {:?}", c.at("Player"));
    }
}

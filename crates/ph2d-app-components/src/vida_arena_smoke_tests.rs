//! Os gates da ARENA (plano 28, W7) — pelas portas do PRODUTO, e com o QUADRO inteiro: os
//! relógios, as fábricas (`tick_factories` + `apply_births`), a tabela de acções
//! (`resolve_signal_actions` + `apply`), a ponte da física, os sinais que ela publica, o dreno das
//! mortes e o recomeço.
//!
//! ⚠️ **A régua que nenhuma cena irmã tem é a do laço INTEIRO**: aqui o jogo é jogado pela tabela
//! que o artista escreveu, e o recomeço é servido como a shell o serve (a `fase_fabrica_e_morte`).
//! *Um arnês que não corre o que o quadro corre não afirma nada sobre o quadro* (a lição do gate
//! das luzes do FIM DE JOGO).
//!
//! ⚠️ **Cada caso monta a cena de raiz e põe o herói onde o caso precisa ANTES do 1.º tique**: o
//! herói é um mover cinemático cuja pose a ponte conduz, e teletransportá-lo a meio mediria o arnês.

use super::*;
use crate::signal_actions_bridge::{Som, apply};
use ph2d_ecs::{Disparo, SignalEffect, SimWorld, Spawned, resolve_signal_actions};
use ph2d_physics_ecs::health::{HealthEvent, HealthEventKind};
use ph2d_physics_ecs::{HealthNow, PedidoDeVida, PhysicsBridge};
use ph2d_preview_drive::PreviewDrive;

const DT_US: u64 = 16_667;

/// **O jogo a correr** — tudo o que um quadro da shell toca, e nada mais.
struct Jogo {
    sim: SimWorld,
    ponte: PhysicsBridge,
    tags: ph2d_tags::TagTree,
    registo: ph2d_ecs::scene::ComponentRegistry,
    docs: (
        ph2d_vec_scene::VecScene,
        ph2d_vec_entities::entities::VecEntityMap,
    ),
    drive: PreviewDrive,
    t: u64,
    /// Os sinais do quadro anterior (a física publica-os DEPOIS do tique), com quem os gritou.
    fila: Vec<(String, Option<Entity>)>,
    recomecos: u32,
    heroi: Entity,
}

/// O que um quadro deixou.
#[derive(Default)]
struct Quadro {
    eventos: Vec<HealthEvent>,
    sinais: Vec<String>,
}

impl Jogo {
    /// Monta a cena e põe o herói em `xy` com o rumo `rumo`, antes do 1.º tique.
    fn novo(xy: [f32; 2], rumo: f32) -> Self {
        let mut sim = SimWorld::new();
        let heroi = cena_quatro(sim.world_mut());
        crate::vida_smoke::resolver_receitas(sim.world_mut());
        {
            let mut t = sim
                .world_mut()
                .get_mut::<Transform>(heroi)
                .expect("o herói tem pose");
            t.translation = Vec2::new(xy[0], xy[1]);
            t.rotation = rumo;
        }
        ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
        Self {
            sim,
            ponte: PhysicsBridge::new(),
            tags: ph2d_tags::TagTree::default(),
            registo: crate::component_registry_for_tests::registo(),
            docs: crate::instance_docs::empty_docs(),
            drive: PreviewDrive::default(),
            t: 0,
            fila: Vec::new(),
            recomecos: 0,
            heroi,
        }
    }

    /// O jogo tal como a cena o monta.
    fn da_cena() -> Self {
        Self::novo(HEROI_XY, 0.0)
    }

    /// **Um quadro**, com os sinais `extra` a soar (o gatilho de uma arma, um relógio forçado).
    fn quadro(&mut self, extra: &[&str]) -> Quadro {
        let mut q = Quadro::default();
        // (a) os relógios — o que o `timer_tick::tick_timers` da shell faz.
        ph2d_ecs::reconcile_timers(self.sim.world_mut());
        let mut soaram: Vec<(String, Option<Entity>)> = std::mem::take(&mut self.fila);
        {
            let mundo = self.sim.world_mut();
            let mut qt = mundo.query::<(Entity, &Timers, &mut ph2d_ecs::TimerRuntime)>();
            for (e, cfg, mut rt) in qt.iter_mut(mundo) {
                for (tm, st) in cfg.0.iter().zip(rt.0.iter_mut()) {
                    if ph2d_ecs::timer_advance(tm, st, DT_US).fires > 0 && !tm.signal.is_empty() {
                        soaram.push((tm.signal.clone(), Some(e)));
                    }
                }
            }
        }
        soaram.extend(extra.iter().map(|s| ((*s).to_owned(), Some(self.heroi))));
        q.sinais = soaram.iter().map(|(n, _)| n.clone()).collect();
        // (b) as fábricas.
        let nomes: Vec<&str> = soaram.iter().map(|(n, _)| n.as_str()).collect();
        let tick = ph2d_ecs::tick_factories(self.sim.world_mut(), &self.tags, &nomes);
        {
            let mut docs = crate::instance_docs::OwnedDocs {
                vec_scene: &mut self.docs.0,
                vec_entities: &mut self.docs.1,
            };
            crate::factory_bridge::apply_births(
                &mut self.sim,
                &self.registo,
                &mut docs,
                &tick.births,
                self.t,
            );
        }
        // (c) a tabela de acções.
        let disparos: Vec<Disparo<'_>> = soaram
            .iter()
            .map(|(n, quem)| Disparo {
                nome: n,
                quem: *quem,
                outro: None,
            })
            .collect();
        let efeitos: Vec<SignalEffect> =
            resolve_signal_actions(self.sim.world_mut(), &self.tags, &disparos);
        let mut recomecar = false;
        if !efeitos.is_empty() {
            let mut mudo = |_: &mut SimWorld, _: Som, _: Entity| false;
            let r = apply(&mut self.sim, &efeitos, &mut self.drive, &mut mudo);
            for (alvo, p) in r.pedidos_de_vida {
                self.ponte.pede_vida(alvo, p);
            }
            recomecar = r.recomecar;
        }
        // (d) o tique da física.
        ph2d_ecs::assign_missing_stable_ids(self.sim.world_mut());
        ph2d_ecs::assign_master_pieces(self.sim.world_mut());
        self.ponte.dispatch(&mut self.sim, true, self.t);
        self.t += 1;
        q.eventos = self.ponte.health_events().to_vec();
        self.fila = self
            .ponte
            .signal_events(&self.sim, &self.tags)
            .into_iter()
            .map(|s| (s.name, Some(s.source)))
            .collect();
        // (e) o dreno das mortes — o da `fase_fabrica_e_morte`.
        let mortes = self.ponte.mortes_anunciadas(self.sim.world());
        crate::factory_bridge::apply_deaths(&mut self.sim, &mortes);
        // (f) o recomeço, servido DEPOIS de tudo — como a shell o serve.
        if recomecar {
            crate::factory_bridge::sweep_spawned(&mut self.sim);
            ph2d_ecs::rewind_runtime::rewind_runtime_state(
                self.sim.world_mut(),
                ph2d_ecs::rewind_runtime::Renascimento::Recomecar,
            );
            self.t = 0;
            self.fila.clear();
            self.recomecos += 1;
        }
        self.drive.settle();
        q
    }

    /// `n` quadros sem sinais extra, com os eventos todos.
    fn corre(&mut self, n: u32) -> Vec<HealthEvent> {
        (0..n).flat_map(|_| self.quadro(&[]).eventos).collect()
    }

    fn vida_de(&self, e: Entity) -> f64 {
        self.sim
            .world()
            .get::<HealthNow>(e)
            .map_or(f64::NAN, |h| h.pontos)
    }

    /// As cópias vivas cujo nome começa por `nome`.
    fn copias(&mut self, nome: &str) -> Vec<Entity> {
        let w = self.sim.world_mut();
        let mut q = w.query_filtered::<(Entity, &Name), bevy_ecs::query::With<Spawned>>();
        q.iter(w)
            .filter(|(_, n)| n.as_str().starts_with(nome))
            .map(|(e, _)| e)
            .collect()
    }
}

fn danos(evs: &[HealthEvent], alvo: Entity) -> Vec<(Entity, f64)> {
    evs.iter()
        .filter(|e| e.target == alvo)
        .filter_map(|e| match e.kind {
            HealthEventKind::Damaged { amount } => Some((e.source, amount)),
            _ => None,
        })
        .collect()
}

fn morreu(evs: &[HealthEvent], alvo: Entity) -> bool {
    evs.iter()
        .any(|e| e.target == alvo && e.kind == HealthEventKind::Died)
}

fn perto(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-6
}

/// ⭐⭐⭐ **A Salamandra só cai ao GELO** — o fogo não a fere (nem o golpe nem a queimadura), e três
/// balas de gelo (`20` cada, o dobro) matam-na e ela grita a vitória.
///
/// **Mutações que devem sangrar:** tirar a imunidade ao fogo; baixar o `×2` do gelo.
#[test]
fn a_salamandra_so_cai_ao_gelo() {
    let mut j = Jogo::da_cena();
    j.corre(20);
    let sal = *j
        .copias("Salamandra")
        .first()
        .expect("a Salamandra não nasceu");
    let mut evs = j.quadro(&[SINAL_FOGO]).eventos;
    evs.extend(j.corre(240));
    assert!(
        danos(&evs, sal).is_empty(),
        "o fogo feriu a Salamandra: {:?}",
        danos(&evs, sal)
    );
    assert!(perto(j.vida_de(sal), f64::from(VIDA_DA_SALAMANDRA)));
    let mut todos = Vec::new();
    let mut viu_venceu = false;
    for _ in 0..3 {
        let q = j.quadro(&[SINAL_GELO]);
        todos.extend(q.eventos);
        for _ in 0..40 {
            let q = j.quadro(&[]);
            viu_venceu |= q.sinais.iter().any(|s| s == VENCEU);
            todos.extend(q.eventos);
        }
    }
    let d: Vec<f64> = danos(&todos, sal).into_iter().map(|(_, a)| a).collect();
    assert_eq!(d, vec![f64::from(DANO) * 2.0; 3], "três golpes de 20");
    assert!(morreu(&todos, sal), "a Salamandra não morreu ao 3.º gelo");
    viu_venceu |= j.quadro(&[]).sinais.iter().any(|s| s == VENCEU);
    assert!(viu_venceu, "a morte dela não gritou «{VENCEU}»");
    assert!(
        j.copias("Salamandra").is_empty(),
        "a Salamandra morta ficou na cena"
    );
}

/// ⭐⭐⭐ **Um morcego PERSEGUE o herói, morde `15` e SOME** — com o herói parado a `7,9` m do ninho.
///
/// **Mutações que devem sangrar:** o morcego perseguir ninguém (`NavTarget::None`); o dano do
/// morcego não o gastar (`OnHit::Stay`).
#[test]
fn um_morcego_persegue_morde_e_some() {
    let mut j = Jogo::da_cena();
    j.quadro(&[SINAL_MORCEGO]);
    let morcego = *j.copias("Morcego").first().expect("o morcego não nasceu");
    // A 2,2 m/s o caminho do ninho ao herói, à volta do muro (W8), leva ~4,4 s.
    let mut evs = Vec::new();
    for _ in 0..420 {
        evs.extend(j.quadro(&[]).eventos);
        if !danos(&evs, j.heroi).is_empty() {
            break;
        }
    }
    let mordidas: Vec<(Entity, f64)> = danos(&evs, j.heroi)
        .into_iter()
        .filter(|(s, _)| *s == morcego)
        .collect();
    assert_eq!(
        mordidas.len(),
        1,
        "o morcego não chegou a morder o herói em 7 s: {:?}",
        danos(&evs, j.heroi)
    );
    assert!(perto(mordidas[0].1, f64::from(MORDIDA)));
    j.corre(2);
    assert!(
        !j.copias("Morcego").contains(&morcego),
        "o morcego que mordeu ficou na cena"
    );
    assert!(perto(
        j.vida_de(j.heroi),
        f64::from(VIDA_DO_HEROI - MORDIDA)
    ));
}

/// ⭐⭐ **Um tiro mata um morcego** — o herói à altura do ninho e virado para ele; o morcego vem
/// direito, e a bala (um mover) acerta no corpo dele (outro mover).
///
/// **Mutação que deve sangrar:** o morcego nascer sem corpo sólido (a bala só vê formas sólidas).
#[test]
fn um_tiro_mata_um_morcego() {
    // ⚠️ (W8) À ESQUERDA do muro — do lado do herói a bala acertava nele.
    let mut j = Jogo::novo([0.5, NINHO_XY[1]], std::f32::consts::PI);
    j.quadro(&[SINAL_MORCEGO]);
    let morcego = *j.copias("Morcego").first().expect("o morcego não nasceu");
    j.corre(5);
    let mut evs = j.quadro(&[SINAL_GELO]).eventos;
    evs.extend(j.corre(90));
    assert!(morreu(&evs, morcego), "a bala não matou o morcego");
    assert!(
        danos(&evs, j.heroi).is_empty(),
        "o morcego morto ainda mordeu"
    );
    assert!(!j.copias("Morcego").contains(&morcego));
}

/// ⭐⭐⭐ **A LAVA queima enquanto se pisa, em pulsos de `3` a cada meio segundo, e continua DEPOIS
/// de sair** — e não arma a invencibilidade (é por isso que ela não dá golpe nenhum; ver o
/// cabeçalho da cena).
///
/// ⚠️ **«Sair» é a lava deixar de tocar** — tirar o `Damage` dela é o mesmo facto para a ponte (o
/// toque acaba) sem mexer no mover que ela conduz.
///
/// **Mutações que devem sangrar:** a lava golpear (`amount > 0` com a invencibilidade do herói);
/// a lava sem queimadura depois (`over_time_s` a zero).
#[test]
fn a_lava_queima_enquanto_se_pisa_e_depois() {
    let mut j = Jogo::novo(LAVA_XY, 0.0);
    let dentro = j.corre(180);
    let h = j.heroi;
    // Só os PULSOS (a fonte de um pulso é o próprio alvo) — um morcego pode morder no meio.
    let pulsos: Vec<f64> = danos(&dentro, h)
        .into_iter()
        .filter(|(s, _)| *s == h)
        .map(|(_, a)| a)
        .collect();
    let por_pulso = f64::from(LAVA_POR_S * LAVA_CADA_S);
    assert!(
        pulsos.len() >= 5,
        "3 s na lava deram {} pulso(s): {pulsos:?}",
        pulsos.len()
    );
    assert!(pulsos.iter().all(|&p| perto(p, por_pulso)), "{pulsos:?}");
    assert!(
        danos(&dentro, h).iter().all(|(s, a)| *s != h || *a > 0.0),
        "a lava deu um golpe de zero com número"
    );
    let cfg = j
        .sim
        .world()
        .get::<Health>(h)
        .expect("o herói tem vida")
        .config();
    let invencivel = j
        .ponte
        .health_of(h)
        .is_some_and(|s| s.vida().invencivel(&cfg));
    let mordido = danos(&dentro, h).iter().any(|(s, _)| *s != h);
    assert!(
        mordido || !invencivel,
        "a lava armou a invencibilidade do herói"
    );
    // A lava deixa de tocar.
    let lava = {
        let w = j.sim.world_mut();
        let mut q = w.query::<(Entity, &Name)>();
        q.iter(w)
            .find(|(_, n)| n.as_str() == "Lava")
            .map(|(e, _)| e)
            .expect("a lava existe")
    };
    j.sim.world_mut().entity_mut(lava).remove::<Damage>();
    let fora = j.corre(240);
    let depois: Vec<f64> = danos(&fora, h)
        .into_iter()
        .filter(|(s, _)| *s == h)
        .map(|(_, a)| a)
        .collect();
    // ⚠️ **Medido, não suposto:** a queimadura renova-se a `LAVA_DEPOIS_S` no último tique de
    // contacto e a FASE continua — o 1.º pulso depois de sair leva também o tempo que já corria
    // desde o pulso anterior, e o último leva só a fracção que falta (a lei das aflições). ⇒ o
    // total depois de sair fica em `por_s × [dur, dur + intervalo)`, e não em `por_s × dur` exacto
    // (a 1.ª redacção deste gate dizia «exactamente 4 pulsos» e leu `3·3·3·3·2,8`).
    let total: f64 = depois.iter().sum();
    let piso = f64::from(LAVA_POR_S * LAVA_DEPOIS_S);
    let tecto = f64::from(LAVA_POR_S * (LAVA_DEPOIS_S + LAVA_CADA_S));
    assert!(
        total >= piso - 1e-6 && total < tecto,
        "depois de sair a lava tirou {total} ({depois:?}), fora de [{piso}, {tecto})"
    );
    assert!(
        depois[..depois.len() - 1]
            .iter()
            .all(|&p| perto(p, por_pulso)),
        "só o último pulso pode ser fraccionário: {depois:?}"
    );
}

/// ⭐⭐⭐ **O coração CURA `25` um herói ferido e some; com a vida cheia some na mesma** — o `Vanish`
/// é de quem toca, cure ou não (e o roteiro di-lo).
///
/// **Mutações que devem sangrar:** tirar a resistência `cura` do herói (o coração FERIRIA);
/// o coração sem `Vanish`.
#[test]
fn o_coracao_cura_o_ferido_e_some_sempre() {
    // Ferido.
    let mut j = Jogo::novo(CORACAO_XY, 0.0);
    j.ponte.pede_vida(j.heroi, PedidoDeVida::Dano(40.0));
    j.corre(2);
    assert!(
        perto(j.vida_de(j.heroi), 60.0),
        "o pedido de dano não entrou"
    );
    j.quadro(&[SINAL_CORACAO]);
    j.corre(10);
    assert!(
        perto(j.vida_de(j.heroi), 60.0 + f64::from(CURA)),
        "a vida ficou em {}",
        j.vida_de(j.heroi)
    );
    assert!(j.copias("Coracao").is_empty(), "o coração ficou na cena");
    // Cheio.
    let mut j = Jogo::novo(CORACAO_XY, 0.0);
    j.corre(2);
    j.quadro(&[SINAL_CORACAO]);
    j.corre(10);
    assert!(perto(j.vida_de(j.heroi), f64::from(VIDA_DO_HEROI)));
    assert!(
        j.copias("Coracao").is_empty(),
        "com a vida cheia o coração ficou — o roteiro promete o contrário"
    );
}

/// ⭐⭐ **Um morcego não GASTA o coração** — o coração é da equipa dos monstros, e um dano da mesma
/// equipa não toca.
///
/// ⛔⛔ **A 1.ª redacção TELETRANSPORTAVA o morcego para cima do coração e a mutação SOBREVIVEU:** o
/// morcego é um mover cuja pose a ponte conduz, logo escrever-lhe o `Transform` não o move — a
/// fixtura não continha o fenómeno. ⇒ a FÁBRICA do coração (documento, não conduzida) muda-se para
/// o NINHO, e os dois NASCEM sobrepostos, que é o caminho que o produto percorre (um corpo que nasce
/// sobreposto recebe o toque — é o mesmo caminho do coração que cura o herói).
///
/// **Mutação que deve sangrar:** o coração sem equipa.
#[test]
fn um_morcego_nao_gasta_o_coracao() {
    let mut j = Jogo::da_cena();
    let fab = {
        let w = j.sim.world_mut();
        let mut q = w.query::<(Entity, &Name)>();
        q.iter(w)
            .find(|(_, n)| n.as_str() == "Fabrica: Coracao")
            .map(|(e, _)| e)
            .expect("a fábrica do coração existe")
    };
    j.sim
        .world_mut()
        .get_mut::<Transform>(fab)
        .expect("pose")
        .translation = Vec2::new(NINHO_XY[0], NINHO_XY[1]);
    let mut evs = j.quadro(&[SINAL_CORACAO, SINAL_MORCEGO]).eventos;
    let coracao = *j.copias("Coracao").first().expect("o coração não nasceu");
    let morcego = *j.copias("Morcego").first().expect("o morcego não nasceu");
    evs.extend(j.corre(3));
    assert!(
        j.copias("Coracao").contains(&coracao),
        "o morcego gastou o coração"
    );
    assert!(danos(&evs, morcego).is_empty(), "o coração feriu o morcego");
}

/// ⭐⭐⭐ **O LAÇO INTEIRO: o herói morre, grita, e um segundo e meio depois a corrida RECOMEÇA
/// sozinha** — com a vida cheia, sem queimadura, e a Salamandra de volta.
///
/// ⚠️ **Pela tabela que o artista escreveu**: o `heroi-caiu` arranca o relógio, o relógio publica o
/// `recomecar`, e a linha `RestartRun` pede o recomeço — cada elo é o do produto.
///
/// **Mutações que devem sangrar:** tirar a linha do `RestartRun`; o relógio do recomeço com outro
/// sinal; o herói sem `on_death`.
#[test]
fn o_heroi_morre_e_a_corrida_recomeca() {
    let mut j = Jogo::novo(LAVA_XY, 0.0);
    j.corre(60);
    assert!(
        j.ponte
            .health_of(j.heroi)
            .is_some_and(|s| !s.aflicoes().0.is_empty()),
        "o controlo: o herói na lava está a arder antes de morrer"
    );
    j.ponte.pede_vida(j.heroi, PedidoDeVida::Dano(1000.0));
    let mut caiu_em = None;
    let mut recomecou_em = None;
    for q in 0..300_u32 {
        let r = j.quadro(&[]);
        if caiu_em.is_none() && r.sinais.iter().any(|s| s == CAIU) {
            caiu_em = Some(q);
        }
        if j.recomecos > 0 {
            recomecou_em = Some(q);
            break;
        }
    }
    let caiu = caiu_em.expect("o herói morreu calado");
    let recomecou = recomecou_em.expect("a corrida não recomeçou");
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let esperado = (RECOMECO_US / DT_US) as u32;
    assert!(
        recomecou - caiu >= esperado - 2 && recomecou - caiu <= esperado + 2,
        "o recomeço chegou {} quadros depois da morte (esperado ~{esperado})",
        recomecou - caiu
    );
    j.corre(30);
    assert!(
        perto(j.vida_de(j.heroi), f64::from(VIDA_DO_HEROI)),
        "o herói renasceu com {}",
        j.vida_de(j.heroi)
    );
    assert_eq!(j.copias("Salamandra").len(), 1, "a Salamandra não voltou");
    assert_eq!(j.recomecos, 1, "recomeçou mais de uma vez");
}

/// ⭐⭐ **Os sinais da cena casam dos dois lados** — cada linha do herói ouve um nome que alguém
/// DIZ, e o relógio que ela arranca existe.
///
/// **Mutação que deve sangrar:** trocar o `CAIU` por outro nome numa das pontas.
#[test]
fn a_corrente_do_recomeco_casa_dos_dois_lados() {
    let mut j = Jogo::da_cena();
    let h = j.heroi;
    let w = j.sim.world_mut();
    let tabela = w.get::<SignalActions>(h).expect("tabela").0.clone();
    let relogios = w.get::<Timers>(h).expect("relógios").0.clone();
    let vida = w.get::<Health>(h).expect("vida").clone();
    let mut ditos: Vec<String> = relogios.iter().map(|t| t.signal.clone()).collect();
    ditos.push(vida.on_death.clone());
    for l in &tabela {
        assert!(ditos.contains(&l.on), "ninguém diz «{}»", l.on);
        assert_eq!(l.from, SignalFrom::Myself);
    }
    let arrancar = tabela
        .iter()
        .find(|l| l.verb == SignalVerb::StartTimer)
        .expect("a linha que arranca o relógio");
    assert!(relogios.iter().any(|t| t.name == arrancar.arg));
    assert!(tabela.iter().any(|l| l.verb == SignalVerb::RestartRun));
}

/// ⭐ **O prólogo cria as acções das DUAS armas** — as mesmas da cena `=3`.
#[test]
fn o_prologo_cria_as_duas_armas() {
    assert_eq!(
        crate::vida_smoke::accoes(4),
        &crate::vida_tipos_smoke::ACCOES[..]
    );
}

/// ⭐⭐ **O exercício 3 do tutorial: sem a resistência `cura`, o coração FERE** — a mesma peça a
/// fazer o contrário só porque quem a leva mudou. ⚠️ Um passo de tutorial que promete um efeito é
/// uma AFIRMAÇÃO sobre o produto, e esta tem gate.
///
/// **Mutação que deve sangrar:** o coração nascer da equipa do herói (não o tocaria nunca).
#[test]
fn sem_a_resistencia_o_coracao_fere() {
    let mut j = Jogo::novo(CORACAO_XY, 0.0);
    j.sim
        .world_mut()
        .get_mut::<Health>(j.heroi)
        .expect("o herói tem vida")
        .resistances
        .clear();
    j.corre(2);
    // ⚠️ O coração pode tocar no MESMO quadro em que nasce (o sensor já sobrepõe) — os eventos
    // desse quadro contam.
    let mut evs = j.quadro(&[SINAL_CORACAO]).eventos;
    evs.extend(j.corre(10));
    let d: Vec<f64> = danos(&evs, j.heroi).into_iter().map(|(_, a)| a).collect();
    assert_eq!(d, vec![f64::from(CURA)], "o coração devia ferir {CURA}");
    assert!(perto(j.vida_de(j.heroi), f64::from(VIDA_DO_HEROI - CURA)));
}

/// ⭐⭐ **O roteador CONTA a arena** — cada nível de `1` a `CENAS` monta a SUA cena, e a `=4` é
/// esta (um herói chamado `Heroi` com a tabela do recomeço). ⚠️ Sem isto um `CENAS` que não subisse
/// deixava a arena **muda** (o registo da família corta acima do tecto), e um braço esquecido no
/// `montar` caía em silêncio na cena `=1`.
///
/// **Mutações que devem sangrar:** `CENAS = 3`; o braço `4` do `montar` apagado.
#[test]
fn cada_nivel_ate_cenas_monta_a_sua_cena() {
    assert_eq!(crate::vida_smoke::CENAS, 4, "a arena é a quarta cena");
    for nivel in 1..=crate::vida_smoke::CENAS {
        let mut sim = SimWorld::new();
        let m = crate::vida_smoke::montar(sim.world_mut(), nivel);
        assert_eq!(m.nivel, nivel, "o nível {nivel} montou outra cena");
    }
    let mut sim = SimWorld::new();
    let m = crate::vida_smoke::montar(sim.world_mut(), 4);
    let heroi = Entity::from_bits(m.escolhido);
    assert!(
        sim.world().get::<SignalActions>(heroi).is_some(),
        "a cena =4 nasce com o herói que tem a tabela do recomeço"
    );
}

#[path = "vida_arena_nav_tests.rs"]
mod nav;

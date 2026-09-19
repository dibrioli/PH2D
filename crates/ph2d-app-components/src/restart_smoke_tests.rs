//! Os gates da cena do FIM DE JOGO. ⚠️ A régua mais importante é a que nenhuma cena irmã tem:
//! *o jogo chega a RECOMEÇAR, e o que a corrida escreveu volta ao princípio?*

use super::*;
use ph2d_ecs::{SimWorld, Visibility};

fn montada() -> (SimWorld, Montada) {
    let mut sim = SimWorld::new();
    let m = montar(sim.world_mut(), 1);
    (sim, m)
}

fn heroi(sim: &mut SimWorld) -> Entity {
    let m = sim.world_mut();
    m.query::<(Entity, &Name)>()
        .iter(m)
        .find(|(_, n)| n.0 == "Heroi")
        .map(|(e, _)| e)
        .expect("a cena tem de ter o herói")
}

/// ⭐⭐⭐ **A CENA TEM AS SEIS PEÇAS DO LAÇO, e nenhuma sozinha faz alguma coisa.**
#[test]
fn a_cena_tem_o_laco_inteiro() {
    let (mut sim, _) = montada();
    let m = sim.world_mut();
    assert_eq!(
        m.query::<&SignalOnHit>().iter(m).count(),
        usize::try_from(ESPINHOS).unwrap(),
        "quem BATE"
    );
    let m = sim.world_mut();
    assert_eq!(m.query::<&Counter>().iter(m).count(), 1, "quem CONTA");
    let m = sim.world_mut();
    assert_eq!(m.query::<&CounterWatch>().iter(m).count(), 1, "quem PERDE");
    let m = sim.world_mut();
    assert_eq!(m.query::<&Timers>().iter(m).count(), 1, "a BATIDA");
    let m = sim.world_mut();
    assert_eq!(
        m.query::<&SignalActions>().iter(m).count(),
        1,
        "quem LIGA os quatro"
    );
    let m = sim.world_mut();
    assert_eq!(
        m.query::<&ph2d_physics_ecs::TopDownPlayer>()
            .iter(m)
            .count(),
        1,
        "quem ANDA"
    );
}

/// ⭐⭐⭐ **A TABELA fecha a corrente, e o último elo é o verbo desta wave.**
///
/// ⚠️ **Cada `on` tem de casar com um `signal` que alguém DIZ** — uma const trocada de um lado
/// monta uma cena que parece certa e não faz nada, e nenhum gate de presença o vê.
///
/// **Mutações que devem sangrar:** trocar o `RECOMECA` por outro nome numa das duas pontas ·
/// tirar a linha do `RestartRun`.
#[test]
fn a_corrente_dos_sinais_casa_dos_dois_lados() {
    let (mut sim, _) = montada();
    let h = heroi(&mut sim);
    let tabela = sim.world().get::<SignalActions>(h).unwrap().0.clone();
    let vigia = sim.world().get::<CounterWatch>(h).unwrap().0.clone();
    let relogios = sim.world().get::<Timers>(h).unwrap().0.clone();

    // quem GRITA cada nome
    let mut ditos: Vec<String> = vigia.iter().map(|r| r.signal.clone()).collect();
    ditos.extend(relogios.iter().map(|t| t.signal.clone()));
    ditos.push(GOLPE.to_owned()); // o espinho

    for l in &tabela {
        assert!(
            ditos.contains(&l.on),
            "a linha ouve «{}», e ninguém nesta cena o diz",
            l.on
        );
    }
    // ⭐⭐⭐ **E O SENTIDO CONTRÁRIO — cada sinal DITO tem de ter quem o OUÇA.**
    //
    // ⚠️⚠️ **Esta metade nasceu de uma MUTAÇÃO SOBREVIVENTE:** apagar a linha
    // `morri → StartTimer` deixava o gate VERDE, porque o laço acima só pergunta *«alguém diz o
    // que esta linha ouve?»* — e um sinal que ninguém ouve passa despercebido. *Uma corrente
    // medida num sentido só não é uma corrente: é uma lista.*
    let ouvidos: Vec<&str> = tabela.iter().map(|l| l.on.as_str()).collect();
    for dito in &ditos {
        assert!(
            ouvidos.contains(&dito.as_str()),
            "alguém diz «{dito}» e ninguém nesta cena o ouve — a corrente parte aqui"
        );
    }
    // ⭐ E a corrente acaba no verbo desta wave.
    let fim = tabela
        .iter()
        .find(|l| l.verb == SignalVerb::RestartRun)
        .expect("a cena tem de ter a linha que RECOMEÇA");
    assert_eq!(fim.on, RECOMECA);
    assert!(
        fim.target.is_empty(),
        "o `Restart Run` não tem alvo: o sujeito dele é a CORRIDA"
    );
    // ⛔ **E a batida existe**: sem ela a última luz apaga e reacende no MESMO quadro.
    assert!(
        relogios
            .iter()
            .any(|t| t.signal == RECOMECA && !t.autostart),
        "o relógio da batida tem de dizer «{RECOMECA}» e NÃO arrancar sozinho"
    );
}

/// ⭐⭐⭐ **O JOGO PERDE-SE E RECOMEÇA, pelo caminho do PRODUTO** — o único gate que percorre a cena
/// montada com a ponte e o renascimento.
///
/// ⚠️⚠️ **A metade que decide a wave é a TERCEIRA:** as vidas voltam **e** a luz que o `Hide`
/// apagou volta a acender. *Sem ela, o dono vê «as vidas voltaram a três e o painel ficou às
/// escuras» — um recomeço pela metade, que é pior que nenhum.*
///
/// **Mutações que devem sangrar:** tirar o `release_all_to_authored` · tirar o
/// `rewind_runtime_state` · o verbo a não anunciar.
#[test]
fn o_jogo_perde_se_e_recomeca() {
    use crate::signal_actions_bridge::{Som, apply};
    use ph2d_ecs::SignalEffect;
    use ph2d_preview_drive::PreviewDrive;

    let (mut sim, _) = montada();
    let h = heroi(&mut sim);
    let luz1 = {
        let m = sim.world_mut();
        m.query::<(Entity, &Name)>()
            .iter(m)
            .find(|(_, n)| n.0 == format!("{LUZ}1"))
            .map(|(e, _)| e)
            .expect("a cena tem de ter a luz 1")
    };
    let mut drive = PreviewDrive::default();
    let mut mudo = |_: &mut SimWorld, _: Som, _: Entity| false;

    // ── PERDER: três golpes, e a luz apaga-se ────────────────────────────────────────────────
    for _ in 0..VIDAS_INICIAIS {
        let fx = SignalEffect {
            target: h,
            verb: SignalVerb::AddToCounter,
            arg: "-1".to_owned(),
            source: h,
        };
        apply(&mut sim, &[fx], &mut drive, &mut mudo);
    }
    assert_eq!(
        sim.world().get::<CounterRuntime>(h).unwrap().value,
        0,
        "três golpes têm de gastar as três vidas"
    );
    let esconde = SignalEffect {
        target: luz1,
        verb: SignalVerb::Hide,
        arg: String::new(),
        source: h,
    };
    apply(&mut sim, &[esconde], &mut drive, &mut mudo);
    assert!(
        sim.world()
            .get::<Visibility>(luz1)
            .is_some_and(|v| v.hidden),
        "a fixtura tem de CONTER o fenómeno: a luz tem de estar apagada antes do recomeço"
    );

    // ── RECOMEÇAR: o verbo anuncia, e o renascimento serve ───────────────────────────────────
    let pedido = SignalEffect {
        target: h,
        verb: SignalVerb::RestartRun,
        arg: String::new(),
        source: h,
    };
    let r = apply(&mut sim, &[pedido], &mut drive, &mut mudo);
    assert!(r.recomecar, "o verbo tem de ANUNCIAR o recomeço");

    // O que a shell faz ao servir: renascer o estado vivo + devolver as conduções.
    ph2d_ecs::rewind_runtime::rewind_runtime_state(sim.world_mut());
    let devolvidas = drive.release_all_to_authored(&mut sim);

    assert_eq!(
        sim.world().get::<CounterRuntime>(h).unwrap().value,
        VIDAS_INICIAIS,
        "as vidas não voltaram ao princípio"
    );
    assert!(devolvidas >= 1, "nada foi devolvido ao autorado");
    assert!(
        !sim.world()
            .get::<Visibility>(luz1)
            .is_some_and(|v| v.hidden),
        "⛔ a luz ficou APAGADA depois do recomeço — as vidas voltaram a três e o painel ficou às \
         escuras, que é um recomeço pela metade"
    );
}

/// ⭐⭐ **O herói ANDA** — a lição que o dono devolveu em 19/09, na cena irmã do abanão.
///
/// ⚠️ A ponte varre `self.bodies`: quem não tem CORPO nunca entra no laço, e as setas não fazem
/// nada com todos os números certos.
#[test]
fn o_heroi_anda_e_nao_cai() {
    use ph2d_physics_ecs::{PhysicsBridge, PlayerInput};
    let (mut sim, _) = montada();
    let h = heroi(&mut sim);
    let antes = sim.world().get::<Transform>(h).unwrap().translation;
    let mut bridge = PhysicsBridge::new();
    for t in 1..=60 {
        bridge.set_player_input(
            h,
            PlayerInput {
                drive: 1.0,
                ..PlayerInput::default()
            },
        );
        bridge.dispatch(&mut sim, true, t);
    }
    let depois = sim.world().get::<Transform>(h).unwrap().translation;
    assert!(
        (depois.x - antes.x) > 2.0,
        "com a seta segurada um segundo o herói andou {:.4} m",
        depois.x - antes.x
    );
    assert!(
        (depois.y - antes.y).abs() < 0.05,
        "o herói CAIU {:.3} m — o corpo é dinâmico e a gravidade manda nele",
        antes.y - depois.y
    );
}

/// ⚠️ **E o dedo do dono ALCANÇA-O** — a metade abaixo do canal interno da ponte (a rotura do #13).
#[test]
fn a_porta_do_teclado_alcanca_o_heroi() {
    let (mut sim, _) = montada();
    let h = heroi(&mut sim);
    let mut alcancados = Vec::new();
    ph2d_physics_ecs::keyboard_driven::for_each_keyboard_driven(sim.world(), |e| {
        alcancados.push(e);
    });
    assert_eq!(alcancados, vec![h], "só o herói lê o teclado nesta cena");
}

/// ⭐ **Quem nasce escolhido tem a secção que o roteiro nomeia, e um CORPO para o dedo pegar.**
///
/// ⛔ A lição do #15: uma entidade sem `Sprite` não é devolvida pelo pick, e o passo (4) do roteiro
/// — *«o herói já está escolhido, role até Signal Actions»* — seria impossível como gesto.
#[test]
fn quem_nasce_escolhido_tem_a_seccao_e_o_corpo() {
    let (sim, m) = montada();
    let e = Entity::from_bits(m.escolhido);
    assert!(sim.world().get::<SignalActions>(e).is_some());
    assert!(sim.world().get::<CounterWatch>(e).is_some());
    assert!(
        sim.world().get::<Sprite>(e).is_some(),
        "sem corpo o dedo do dono não lhe chega no canvas"
    );
}

/// ⭐⭐⭐ **TUDO O QUE A CENA MONTA ESTÁ DENTRO DA BANDA QUE O DONO VÊ** — e a régua mudou depois
/// da FOTO.
///
/// ⛔⛔ **A 1.ª redacção deste gate media a ALTURA do conteúdo** (`4,0 m` contra uma banda de
/// `5,0 m`) e dava **verde** com o herói FORA DO ECRÃ, a `−2,0` m. *Uma altura não diz ONDE*: a
/// banda de canvas que sobra com a timeline aberta **não está centrada na origem** — sobram
/// `~4,2 m` acima e `~1,3 m` abaixo (medido na foto, `1930×1012` a `100 %`).
///
/// ⚠️ É a mesma família do defeito que o abanão pagou dois dias antes (*a bomba fora do ecrã com os
/// dez gates da cena verdes*), com a **assimetria** no lugar da distância.
///
/// **Mutações que devem sangrar:** pôr qualquer peça abaixo de `−BANDA_ABAIXO` ou acima de
/// `BANDA_ACIMA`.
#[test]
fn tudo_o_que_a_cena_monta_esta_dentro_da_banda() {
    let (mut sim, _) = montada();
    let mundo = sim.world_mut();
    let fora: Vec<(String, f32)> = mundo
        .query::<(&Name, &Transform, &Sprite)>()
        .iter(mundo)
        // ⚠️ **O chão fica de fora**: ele é maior que a vista de propósito, e mede-se pelo CENTRO.
        .filter(|(n, _, _)| n.0 != "Ground")
        .map(|(n, t, _)| (n.0.clone(), t.translation.y))
        .filter(|(_, y)| *y > BANDA_ACIMA || *y < -BANDA_ABAIXO)
        .collect();
    assert!(
        fora.is_empty(),
        "estas peças ficam FORA DO ECRÃ (a banda vai de −{BANDA_ABAIXO} a +{BANDA_ACIMA} m): \
         {fora:?}"
    );
    // ⛔ **E o CONTROLO: a cena tem mesmo peças** — sem ele um `montar` que não montasse nada
    // passaria por vácuo.
    let mundo = sim.world_mut();
    let pecas = mundo.query::<&Sprite>().iter(mundo).count();
    assert!(pecas >= 7, "piso de população: a cena monta {pecas} peças");
}

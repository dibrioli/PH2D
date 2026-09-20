//! Os gates da cena do FIM DE JOGO. ⚠️ A régua mais importante é a que nenhuma cena irmã tem:
//! *o jogo chega a RECOMEÇAR, e o que a corrida escreveu volta ao princípio?*

use super::*;
use ph2d_ecs::SimWorld;

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

// ⛔⛔⛔ **O gate `o_jogo_perde_se_e_recomeca` foi SUBSTITUÍDO pelo
// [`o_jogo_joga_se_recomeca_e_as_luzes_voltam`]**, e a razão é o report do dono de 19/09.
//
// Ele montava o estado à mão (três `AddToCounter`, um `Hide`, o pedido) e **não corria o `settle`
// do fim do quadro** ⇒ o ledger guardava o autorado para sempre e a devolução acendia as luzes
// sozinha. *Um arnês que não corre o que o quadro corre não afirma nada sobre o quadro* — e foi
// exactamente essa a diferença entre o gate verde e o dono a ver as luzes apagadas.

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

/// ⭐⭐⭐ **A CORRENTE INTEIRA, QUADRO A QUADRO — e as LUZES VOLTAM A ACENDER.**
///
/// ⛔⛔⛔ **Este gate nasceu de um report do dono** (*«funcionou mas não recomeça e as luzes não
/// voltam nem com Rewind»*, 19/09), e ele mede o que os outros oito não mediam: **a cena a
/// jogar-se**. Os outros perguntam *«as peças estão lá?»* e *«a ponte anuncia?»*; este percorre os
/// mesmos passos que o quadro percorre — o tique dos relógios · a vigia · a tabela — até ao
/// recomeço.
///
/// ⚠️⚠️ **E o que ele apanha é que o report tinha DUAS metades e só uma era um defeito:** o verbo
/// **corria** (medido na app: o relógio voltava a `0,0000`, duas vezes, a `5,5 s` uma da outra), e
/// o que não voltava eram as **luzes** — logo o recomeço era **invisível**, que se lê exactamente
/// como *«não recomeça»*.
///
/// ⛔ **A causa não é o verbo: é a lei do `ph2d-preview-drive`.** O que um verbo escreve é
/// pré-visualização, e o `settle` de cada quadro esquece quem não foi declarado ⇒ um `Hide`
/// sobrevive **dois quadros** e depois é DOCUMENTO. *Nem o recomeço nem o `Rewind` desfazem um
/// facto do documento — só o `Ctrl+Z`.* ⇒ a cura é a que o modelo prescreve e o artista escreve:
/// **o que a corrida escreve, a corrida desfaz** (três linhas `Show` no mesmo sinal).
///
/// **Mutações que devem sangrar:** tirar as três linhas `Show` · tirar a linha do `RestartRun` ·
/// tirar a batida.
#[test]
fn o_jogo_joga_se_recomeca_e_as_luzes_voltam() {
    use crate::signal_actions_bridge::{Som, apply};
    use ph2d_ecs::{Disparo, SignalEffect, Visibility, resolve_signal_actions};
    use ph2d_preview_drive::PreviewDrive;
    use ph2d_tags::TagTree;

    const DT: f64 = 1.0 / 60.0;
    /// Quantos quadros o gate corre. ⚠️ **Derivado da BATIDA**, e não escolhido: o recomeço chega
    /// um segundo depois da última vida, mais os quadros dos golpes.
    const QUADROS: u32 = 200;

    let (mut sim, _) = montada();
    let h = heroi(&mut sim);
    let luzes: Vec<Entity> = {
        let m = sim.world_mut();
        m.query::<(Entity, &Name)>()
            .iter(m)
            .filter(|(_, n)| n.0.starts_with(LUZ))
            .map(|(e, _)| e)
            .collect()
    };
    assert_eq!(
        luzes.len(),
        usize::try_from(VIDAS_INICIAIS).unwrap(),
        "piso de população: uma luz por vida"
    );

    let tags = TagTree::default();
    let mut drive = PreviewDrive::default();
    let mut mudo = |_: &mut SimWorld, _: Som, _: Entity| false;
    let mut fila: Vec<String> = Vec::new();
    let mut recomecou = false;
    let mut apagou_todas = false;

    for q in 0..QUADROS {
        // (a) o tique dos relógios — o que o `timer_tick::tick_timers` da shell faz.
        ph2d_ecs::reconcile_timers(sim.world_mut());
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let dt_us = (DT * 1e6) as u64;
        let mundo = sim.world_mut();
        let mut disparos_timer: Vec<String> = Vec::new();
        let mut qt = mundo.query::<(&Timers, &mut ph2d_ecs::TimerRuntime)>();
        for (cfg, mut rt) in qt.iter_mut(mundo) {
            for (t, st) in cfg.0.iter().zip(rt.0.iter_mut()) {
                if ph2d_ecs::timer_advance(t, st, dt_us).fires > 0 && !t.signal.is_empty() {
                    disparos_timer.push(t.signal.clone());
                }
            }
        }
        fila.extend(disparos_timer);

        // (b) a vigia dos contadores.
        let vf = crate::counter_watch_bridge::frame(&mut sim, true, 1);
        fila.extend(vf.disparos.into_iter().map(|(_, _, nome)| nome));

        // (c) o dono a tocar num espinho — um por quadro, os três primeiros.
        if q < u32::try_from(ESPINHOS).unwrap() {
            fila.push(GOLPE.to_owned());
        }

        // (d) a tabela.
        if !fila.is_empty() {
            let disparos: Vec<Disparo<'_>> = fila
                .iter()
                .map(|n| Disparo {
                    nome: n,
                    quem: Some(h),
                    outro: None,
                })
                .collect();
            let efeitos: Vec<SignalEffect> =
                resolve_signal_actions(sim.world_mut(), &tags, &disparos);
            if !efeitos.is_empty() {
                let r = apply(&mut sim, &efeitos, &mut drive, &mut mudo);
                if r.recomecar {
                    // O que a shell faz ao servir — ver a `fase_fabrica_e_morte`.
                    //
                    // ⛔⛔⛔ **E o que ela NÃO faz, com a medição ao lado:** a 1.ª redacção desta
                    // wave devolvia aqui TODAS as conduções ao autorado. Ela **lutava contra o
                    // artista**: as três linhas `Show` do mesmo sinal já tinham corrido, e o valor
                    // «autorado» que o ledger guardava por baixo delas era o **apagado** (o `Hide`
                    // já era documento) ⇒ a devolução **desfazia o `Show`** e a luz ficava às
                    // escuras. *Uma porta que devolve «o que estava antes» não sabe distinguir o
                    // que a corrida escreveu do que o artista acabou de mandar escrever.*
                    ph2d_ecs::rewind_runtime::rewind_runtime_state(
                        sim.world_mut(),
                        ph2d_ecs::rewind_runtime::Renascimento::Recomecar,
                    );
                    recomecou = true;
                }
            }
            fila.clear();
        }

        // ⭐⭐⭐ **O `settle` DO FIM DO QUADRO, e sem ele este gate media OUTRO PROGRAMA.**
        //
        // ⛔⛔ **Uma mutação SOBREVIVENTE apanhou-o:** apagar as três linhas `Show` deixava o gate
        // **verde** — porque sem o `settle` o ledger guardava o valor autorado para sempre e a
        // devolução do recomeço acendia as luzes sozinha. *É exactamente a diferença entre o arnês
        // e a app*, e é ela que explica porque o gate passava enquanto o dono via as luzes
        // apagadas: no quadro a sério o `post_frame_undo` corre isto **todo quadro**, e um `Hide`
        // sobrevive **dois** quadros antes de ser documento.
        //
        // ⚠️ *Um arnês que não corre o que o quadro corre não afirma nada sobre o quadro.*
        drive.settle();

        // ⭐ **O CONTROLO de meio caminho:** as três luzes CHEGAM a apagar-se. Sem ele, uma cena em
        // que nada acontecesse e um `Show` que nunca fosse preciso dariam o mesmo verde.
        if !recomecou
            && luzes
                .iter()
                .all(|e| sim.world().get::<Visibility>(*e).is_some_and(|v| v.hidden))
        {
            apagou_todas = true;
        }
        if recomecou {
            break;
        }
    }

    assert!(
        apagou_todas,
        "as três luzes nunca se apagaram — a cena não chegou a perder"
    );
    assert!(recomecou, "a corrente não chegou ao recomeço");
    assert_eq!(
        sim.world().get::<CounterRuntime>(h).unwrap().value,
        VIDAS_INICIAIS,
        "as vidas não voltaram ao princípio"
    );
    // ⭐⭐⭐ **E A METADE DO REPORT:** as luzes voltam a acender.
    for (i, e) in luzes.iter().enumerate() {
        assert!(
            !sim.world().get::<Visibility>(*e).is_some_and(|v| v.hidden),
            "a luz {} ficou APAGADA depois do recomeço — o recomeço fica INVISÍVEL, e o dono lê \
             isso como «não recomeça»",
            i + 1
        );
    }
}

//! Os gates dos componentes do HUD (TOP-20 #20).

use super::{Counter, CounterRuntime, LabelSource, UiButton, UiCanvas, UiLabel, texto, valor};
use crate::timer::{Timer, TimerRuntime, TimerState, Timers};
use bevy_ecs::world::World;
use ph2d_hud::{Fit, Valor};
use ph2d_tags::TagTree;

fn mundo() -> World {
    World::new()
}

/// ⛔⛔ **A CONFIG grava-se; o valor VIVO não.** Sem esta fronteira, cada ponto marcado seria um
/// passo de `Ctrl+Z` e entraria no ficheiro.
#[test]
fn o_contador_grava_a_config_e_nunca_o_valor_vivo() {
    let mut reg = crate::scene::ComponentRegistry::new();
    crate::scene::register_ecs_components(&mut reg);
    assert!(
        reg.get_by_name("ph2d::ecs::Counter").is_some(),
        "a CONFIG do contador tem de viajar no ficheiro"
    );
    assert!(
        reg.get_by_name("ph2d::ecs::CounterRuntime").is_none(),
        "o valor VIVO não pode estar registado — registá-lo põe cada ponto no undo e no save"
    );
    // ⭐ E a porta fecha-se pelo TIPO, não por esta lista: o `CounterRuntime` não deriva
    // `Serialize`, logo a linha de registo dele nem compilaria. Este gate é o aviso alto para
    // quem, um dia, lhe acrescentar o derive.
    assert!(
        reg.get_by_name("ph2d::ecs::UiCanvas").is_some()
            && reg.get_by_name("ph2d::ecs::UiLabel").is_some()
            && reg.get_by_name("ph2d::ecs::UiButton").is_some(),
        "os três do HUD são CONFIG e gravam-se"
    );
}

/// Rebobinar é RENASCER — e nascer aqui é o `start` da config, nunca `Default`.
#[test]
fn rebobinar_devolve_o_contador_ao_start_e_nao_a_zero() {
    let mut w = mundo();
    let tres_vidas = w
        .spawn((
            Counter {
                name: "vidas".into(),
                start: 3,
            },
            CounterRuntime { value: 0 },
        ))
        .id();
    let do_zero = w
        .spawn((
            Counter {
                name: "pontos".into(),
                start: 0,
            },
            CounterRuntime { value: 42 },
        ))
        .id();
    let n = crate::rewind_runtime::rewind_runtime_state(&mut w);
    assert!(n >= 2, "os dois contadores foram tocados");
    assert_eq!(
        w.get::<CounterRuntime>(tres_vidas).expect("vivo").value,
        3,
        "⛔ um `Default` poria isto a ZERO e apagaria as três vidas autoradas"
    );
    assert_eq!(
        w.get::<CounterRuntime>(do_zero).expect("vivo").value,
        0,
        "o CONTROLO: quem começa em zero volta a zero"
    );
}

/// Um nome em branco não é um sinal — a regra emprestada do `SignalOnHit`, palavra por palavra.
#[test]
fn um_botao_sem_nome_nao_e_um_sinal() {
    for cru in ["", "   ", "\t"] {
        assert!(
            UiButton {
                signal: cru.into(),
                disabled: false
            }
            .name()
            .is_none(),
            "{cru:?} não é um contrato que alguém possa casar"
        );
    }
    assert_eq!(
        UiButton {
            signal: "  recomecar ".into(),
            disabled: false
        }
        .name(),
        Some("recomecar"),
        "o CONTROLO positivo, e ele vem aparado"
    );
}

/// A fonte de um rótulo é uma SOMA, nunca «o primeiro» — a ordem de iteração não é prometida.
#[test]
fn dois_contadores_com_o_mesmo_nome_somam() {
    let mut w = mundo();
    for v in [10_i64, 7] {
        w.spawn((
            Counter {
                name: "pontos".into(),
                start: 0,
            },
            CounterRuntime { value: v },
        ));
    }
    // um terceiro, com OUTRO nome, que não pode entrar na conta
    w.spawn((
        Counter {
            name: "vidas".into(),
            start: 0,
        },
        CounterRuntime { value: 100 },
    ));
    let l = UiLabel {
        source: LabelSource::Counter("pontos".into()),
        ..Default::default()
    };
    assert_eq!(
        valor(&mut w, &TagTree::default(), &l),
        Some(Valor::Inteiro(17))
    );
}

/// ⛔ `None` e «zero» são factos DIFERENTES: um rótulo preso a um contador que ninguém criou tem
/// de continuar a mostrar o que o artista escreveu.
#[test]
fn um_contador_que_nao_existe_devolve_nada_e_nao_zero() {
    let mut w = mundo();
    let t = TagTree::default();
    for fonte in [
        LabelSource::Counter("pontos".into()),
        LabelSource::Counter(String::new()),
        LabelSource::TimerLeft("relogio".into()),
        LabelSource::TagCount("inimigo".into()),
    ] {
        let l = UiLabel {
            source: fonte.clone(),
            ..Default::default()
        };
        assert_eq!(
            valor(&mut w, &t, &l),
            None,
            "fonte {fonte:?} num mundo vazio"
        );
    }
    // O CONTROLO: com o contador na cena, a mesma fonte responde.
    w.spawn((
        Counter {
            name: "pontos".into(),
            start: 0,
        },
        CounterRuntime { value: 5 },
    ));
    let l = UiLabel {
        source: LabelSource::Counter("pontos".into()),
        ..Default::default()
    };
    assert_eq!(valor(&mut w, &t, &l), Some(Valor::Inteiro(5)));
}

/// O relógio mostra o MENOR tempo que falta — o que vai tocar primeiro.
#[test]
fn o_relogio_mostra_o_menor_que_falta() {
    let mut w = mundo();
    let timer = |us: u64| Timer {
        name: "ronda".into(),
        duration_us: us,
        repeat: false,
        autostart: true,
        signal: String::new(),
    };
    w.spawn((
        Timers(vec![timer(10_000_000), timer(3_000_000)]),
        TimerRuntime(vec![TimerState::default(), TimerState::default()]),
    ));
    let l = UiLabel {
        source: LabelSource::TimerLeft("ronda".into()),
        ..Default::default()
    };
    assert_eq!(
        valor(&mut w, &TagTree::default(), &l),
        Some(Valor::Segundos(3.0)),
        "⛔ somar dois tempos que correm em paralelo não significa nada"
    );
}

/// O NEUTRO: `Authored` não deriva nada, e é isso que deixa o desenho byte-idêntico.
#[test]
fn o_authored_nao_deriva_nada() {
    let mut w = mundo();
    w.spawn((
        Counter {
            name: "pontos".into(),
            start: 0,
        },
        CounterRuntime { value: 9 },
    ));
    let l = UiLabel::default();
    assert_eq!(l.source, LabelSource::Authored, "é o valor de fábrica");
    assert_eq!(valor(&mut w, &TagTree::default(), &l), None);
    assert_eq!(texto(&mut w, &TagTree::default(), &l), None);
}

/// A linha inteira: prefixo + número + sufixo.
#[test]
fn o_texto_e_o_prefixo_mais_o_numero_mais_o_sufixo() {
    let mut w = mundo();
    w.spawn((
        Counter {
            name: "pontos".into(),
            start: 0,
        },
        CounterRuntime { value: 12 },
    ));
    let l = UiLabel {
        source: LabelSource::Counter("pontos".into()),
        prefix: "Pontos: ".into(),
        suffix: " !".into(),
    };
    assert_eq!(
        texto(&mut w, &TagTree::default(), &l).as_deref(),
        Some("Pontos: 12 !")
    );
}

/// A caixa de referência viaja no documento, com o `Fit` dentro — um enum gémeo no `ph2d-ecs`
/// teria sido a segunda resposta à mesma pergunta.
#[test]
fn a_caixa_do_canvas_atravessa_o_ficheiro() {
    let c = UiCanvas {
        ref_w: 32.0,
        ref_h: 18.0,
        fit: Fit::Stretch,
    };
    let bytes = postcard::to_allocvec(&c).expect("serializa");
    assert_eq!(
        postcard::from_bytes::<UiCanvas>(&bytes).expect("volta"),
        c,
        "ida e volta"
    );
    assert_eq!(
        UiCanvas::default().fit,
        Fit::Keep,
        "o de fábrica não distorce"
    );
}

/// ⭐ **O caminho inteiro do placar, em unidade:** um sinal chega à tabela, a tabela nomeia o
/// contador, e o verbo soma.
///
/// ⛔⛔ **E a metade de baixo é a ARMADILHA que custou seis corridas de foto:** um reactor sem
/// `Transform` e sem `ChildOf` **nunca recebe um `StableId`** (é o critério do
/// [`crate::assign_missing_stable_ids`], escrito lá com o porquê), e o
/// [`crate::signal_actions::resolve`] colhe os reactores com `(Entity, &SignalActions, &StableId)`
/// ⇒ **ele não entra na consulta**. O sinal soa, o toast aparece na tela, e NADA acontece.
///
/// *Um placar que não conta e um placar que não existe leem-se exactamente igual.*
#[test]
fn o_sinal_chega_ao_contador_pelo_nome_e_so_com_identidade() {
    use crate::{SignalAction, SignalActions, SignalFrom, SignalTarget, SignalVerb};

    fn placar(w: &mut World, com_pose: bool) -> usize {
        let tabela = SignalActions(vec![SignalAction {
            on: "tick".into(),
            target: "Placar".into(),
            verb: SignalVerb::AddToCounter,
            arg: "1".into(),
            target_by: SignalTarget::Named,
            from: SignalFrom::Anyone,
        }]);
        let e = w
            .spawn((
                Counter {
                    name: "pontos".into(),
                    start: 0,
                },
                CounterRuntime { value: 0 },
                crate::Name::new("Placar"),
                tabela,
            ))
            .id();
        if com_pose {
            w.entity_mut(e).insert(crate::Transform::default());
        }
        crate::assign_missing_stable_ids(w);
        crate::signal_actions::resolve(
            w,
            &TagTree::default(),
            &[crate::signal_actions::Disparo::anonimo("tick")],
        )
        .len()
    }

    // ⛔ O CONTROLO NEGATIVO: sem pose, o reactor é invisível — e em silêncio.
    assert_eq!(
        placar(&mut mundo(), false),
        0,
        "sem `Transform` nem `ChildOf` não há identidade, e sem identidade não há reactor"
    );
    // ⭐ E com ela, o caminho inteiro fecha.
    let mut w = mundo();
    assert_eq!(placar(&mut w, true), 1, "⛔ o alvo resolve-se pelo NOME");
    let efeitos = crate::signal_actions::resolve(
        &mut w,
        &TagTree::default(),
        &[crate::signal_actions::Disparo::anonimo("tick")],
    );
    assert_eq!(efeitos[0].verb, SignalVerb::AddToCounter);
    assert_eq!(efeitos[0].arg, "1");
}

/// ⭐⭐⭐ **Um clique no RÓTULO de um botão é um clique no botão** — as três metades, porque as três
/// falham por motivos diferentes.
///
/// A auto-conferência da cena mediu o defeito que isto cura: no centro do `+10` o hit-test de
/// objecto devolve o caminho do RÓTULO, que é a forma mais ao topo a conter o ponto. Sem a subida
/// da cadeia, o alvo maior da tela é o único inalcançável no meio dele.
///
/// ⚠️ A 3.ª metade é a que impede a cura barata: se a resposta fosse *«o ancestral mais próximo com
/// qualquer coisa»*, uma forma solta do canvas passaria a carregar num botão que não é dela.
#[test]
fn o_dedo_no_rotulo_de_um_botao_encontra_o_botao() {
    let mut w = mundo();
    let botao = w.spawn(UiButton::default()).id();
    let rotulo = w.spawn((crate::ChildOf(botao), UiLabel::default())).id();
    let estranho = w.spawn(UiLabel::default()).id();

    assert_eq!(
        super::botao_de(&w, rotulo),
        Some(botao),
        "o rótulo pertence ao botão — é ele que o dedo pressiona"
    );
    assert_eq!(
        super::botao_de(&w, botao),
        Some(botao),
        "o próprio botão continua a responder por si"
    );
    assert_eq!(
        super::botao_de(&w, estranho),
        None,
        "uma forma que não vive sob botão nenhum não pressiona nada"
    );
}

/// ⛔ **E um CICLO de `ChildOf` não pendura o quadro.** A cerca é a mesma do `container_of` do
/// envelope, e existe porque um pai pode ser reescrito por um gesto de reparent.
#[test]
fn a_subida_da_cadeia_tem_fundo() {
    let mut w = mundo();
    let a = w.spawn_empty().id();
    let b = w.spawn(crate::ChildOf(a)).id();
    w.entity_mut(a).insert(crate::ChildOf(b));
    assert_eq!(super::botao_de(&w, b), None, "sem botão na cadeia: `None`");
}

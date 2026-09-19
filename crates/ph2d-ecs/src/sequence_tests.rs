//! Os gates do [`SequencePlayer`] (TOP-20 #19).

use super::SequencePlayer;

fn p(nome: &str) -> SequencePlayer {
    SequencePlayer {
        container: nome.to_owned(),
    }
}

/// ⭐ **O nome resolve para o ÍNDICE, e é o nome que viaja.** Um índice guardado no objecto tocaria
/// a cutscene do vizinho no dia em que alguém apagasse o container de cima.
#[test]
fn o_nome_resolve_para_o_indice_do_container() {
    let nomes = ["Intro", "Porta", "Fim"];
    assert_eq!(p("Porta").resolve(nomes), Some(1));
    assert_eq!(p("Intro").resolve(nomes), Some(0));
    assert_eq!(p("Fim").resolve(nomes), Some(2));
}

/// ⛔ **Um nome em branco NÃO é uma sequência** — a mesma regra do `SignalOnHit` e do marcador da
/// timeline, para que «sem cutscene» e «cutscene por escolher» não leiam igual.
#[test]
fn um_nome_em_branco_nao_e_uma_sequencia() {
    assert_eq!(p("").name(), None);
    assert_eq!(p("   ").name(), None);
    assert_eq!(p("   ").resolve(["Intro"]), None);
}

/// ⚠️ **Os DOIS lados são aparados** — um container gravado com um espaço à direita e um componente
/// sem ele são a mesma cutscene para quem os escreveu.
#[test]
fn os_dois_lados_sao_aparados() {
    assert_eq!(p(" Porta ").resolve(["Intro", "Porta  "]), Some(1));
}

/// ⛔ **Um nome que não existe não toca nada** — e não o PRIMEIRO, que é o modo de falha caro: a
/// cutscene errada a correr lê-se como um defeito do motor.
#[test]
fn um_nome_que_nao_existe_nao_toca_nada() {
    assert_eq!(p("Ausente").resolve(["Intro", "Porta"]), None);
    assert_eq!(p("Intro").resolve([]), None);
}

/// ⚠️ **Dois containers com o mesmo nome: ganha o PRIMEIRO, e isso é uma decisão declarada.**
/// Devolver `None` faria o artista perder a cutscene por uma duplicação que ele não vê.
#[test]
fn com_nomes_duplicados_ganha_o_primeiro_e_isso_e_declarado() {
    assert_eq!(p("Porta").resolve(["Porta", "Porta"]), Some(0));
}

use super::{EmCorrida, em_corrida};
use crate::timer::{TimerRuntime, TimerState};
use bevy_ecs::world::World;

fn objecto(w: &mut World, container: &str, estados: Vec<TimerState>) -> bevy_ecs::entity::Entity {
    w.spawn((
        SequencePlayer {
            container: container.to_owned(),
        },
        TimerRuntime(estados),
    ))
    .id()
}

fn correndo(us: u64) -> TimerState {
    TimerState {
        elapsed_us: us,
        running: true,
        finished: false,
    }
}

fn parado(us: u64) -> TimerState {
    TimerState {
        elapsed_us: us,
        running: false,
        finished: false,
    }
}

/// ⭐ **O instante da cutscene é o decorrido do relógio do objecto**, em segundos.
#[test]
fn a_cutscene_corre_no_relogio_do_objecto() {
    let mut w = World::new();
    let e = objecto(&mut w, "Porta", vec![correndo(1_500_000)]);
    assert_eq!(
        em_corrida(&mut w, &["Intro", "Porta"]),
        vec![EmCorrida {
            entity: e,
            container: 1,
            t: 1.5
        }]
    );
}

/// ⛔ **Parada = ausente da lista** — e é isso que devolve o objecto à pose da cena, sem uma linha
/// de código a repô-la.
#[test]
fn um_relogio_parado_nao_toca_nada() {
    let mut w = World::new();
    objecto(&mut w, "Porta", vec![parado(1_500_000)]);
    assert!(em_corrida(&mut w, &["Porta"]).is_empty());
}

/// ⛔⛔ **Um nome que não resolve NÃO cai no container `0`** — tocar a cutscene errada lê-se como um
/// defeito do motor; não tocar nada lê-se como o nome que está mal escrito, que é a verdade.
#[test]
fn um_nome_que_nao_resolve_nao_cai_no_primeiro() {
    let mut w = World::new();
    objecto(&mut w, "Ausente", vec![correndo(1_000_000)]);
    assert!(em_corrida(&mut w, &["Intro", "Porta"]).is_empty());
}

/// ⚠️ **O relógio é o PRIMEIRO, e o gate prende a escolha.** Com dois timers, o segundo a correr
/// não conduz a cutscene — senão ela trocaria de relógio quando o artista arranca outro para outra
/// coisa, em silêncio.
#[test]
fn o_relogio_e_o_primeiro_e_nao_o_primeiro_a_correr() {
    let mut w = World::new();
    objecto(
        &mut w,
        "Porta",
        vec![parado(9_000_000), correndo(2_000_000)],
    );
    assert!(
        em_corrida(&mut w, &["Porta"]).is_empty(),
        "com o timer 0 parado a cutscene não corre, mesmo havendo outro a andar"
    );
}

/// ⛔ **Sem relógio nenhum não há cutscene** — o caso que o `requires` do descritor existe para
/// evitar, e que um ficheiro montado à mão pode produzir na mesma.
#[test]
fn sem_relogio_nao_ha_cutscene() {
    let mut w = World::new();
    objecto(&mut w, "Porta", vec![]);
    assert!(em_corrida(&mut w, &["Porta"]).is_empty());
}

/// ⭐ **Duas cutscenes ao mesmo tempo são duas entradas** — cada uma no relógio dela.
#[test]
fn duas_cutscenes_correm_cada_uma_no_relogio_dela() {
    let mut w = World::new();
    let a = objecto(&mut w, "Intro", vec![correndo(500_000)]);
    let b = objecto(&mut w, "Porta", vec![correndo(2_250_000)]);
    let mut v = em_corrida(&mut w, &["Intro", "Porta"]);
    v.sort_by_key(|c| c.container);
    assert_eq!(
        v,
        vec![
            EmCorrida {
                entity: a,
                container: 0,
                t: 0.5
            },
            EmCorrida {
                entity: b,
                container: 1,
                t: 2.25
            }
        ]
    );
}

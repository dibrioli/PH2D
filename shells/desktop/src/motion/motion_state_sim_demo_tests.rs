//! **OS GATES DA CENA `=113`** — e a prova de que cada passo do anúncio produz o que promete.
//!
//! ⛔⛔ *Uma cena de smoke que ensina o CONTRÁRIO do que acontece é pior que uma cena ausente*
//! (`CLAUDE.md` §5.0) — a ausente não é acreditada. O anúncio desta cena manda o dono trocar
//! `Acts As` de `Force` para `Target Velocity` e diz-lhe o que ele vai ver; aqui isso é
//! **medido sobre a cena do produto**, com um param de diferença entre os dois braços.

use super::*;
use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    reg
}

/// Corre a cena `secs` segundos e devolve `(a maior rapidez vista em QUALQUER tique, as
/// posições finais)`.
///
/// ⚠️ **O pico é sobre a corrida inteira de propósito:** o que separa os dois modos é aquilo
/// a que a queda **chega a fazer**, e ler só o último quadro perderia a aceleração.
fn corre(modo: Option<f32>, secs: f64) -> (f32, Vec<[f32; 2]>) {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build(&mut doc, &reg).expect("a cena é bem tipada");
    if let Some(m) = modo {
        // O ÚNICO param que difere entre os dois braços.
        let vento = doc
            .graph
            .nodes()
            .iter()
            .find(|n| n.type_name == "force.wind")
            .expect("a cena tem um `force.wind`")
            .id;
        doc.graph.set_param(vento, "mode", m);
    }
    let mut cook = Cook::new();
    let last = (secs * 60.0) as u64;
    let (mut pico, mut fim) = (0.0f32, vec![]);
    for k in 0..=last {
        #[expect(clippy::cast_precision_loss, reason = "um indice de tique")]
        let t = k as f64 / 60.0;
        let s = cook.cook(&doc.graph, &reg, sinks[0], t).expect("cozinha")[0]
            .as_stream()
            .clone();
        if let Some(Column::Vec2(v)) = s.get("vel") {
            pico = v.iter().fold(pico, |m, w| m.max(w[0].hypot(w[1])));
        }
        if k == last
            && let Some(Column::Vec2(v)) = s.get("P")
        {
            fim = v.clone();
        }
        cook.advance_tick(&doc.graph, &reg, t).expect("avança");
    }
    (pico, fim)
}

/// **A CHUVA CAI, E O BLOCO APANHA PARTE DELA — parte, não toda.**
///
/// ⚠️ **As duas metades são o gate.** Se o bloco apanhasse tudo, a cena ensinaria que um
/// colisor é um chão; se não apanhasse nada, o passo 2 do anúncio (arrastar a caixa e ver o
/// monte mudar de sítio) não teria monte nenhum para mover.
#[test]
fn the_block_catches_part_of_the_rain_and_the_rest_falls_past_it() {
    let (_, p) = corre(None, 2.2);
    assert_eq!(
        p.len(),
        (ROWS * COLS) as usize,
        "a nuvem inteira tem de estar la'"
    );
    let topo = BLOCO_Y + BLOCO_H * 0.5;
    let em_cima = p.iter().filter(|q| q[1] >= topo - PECA).count();
    let passaram = p.iter().filter(|q| q[1] < BLOCO_Y - BLOCO_H).count();
    assert!(
        em_cima >= 4,
        "so' {em_cima} peca(s) pousaram no bloco -- sem monte, o passo 2 do anuncio (arrastar \
         a caixa e ver o monte mudar de sitio) nao tem o que mover"
    );
    assert!(
        passaram >= 4,
        "so' {passaram} peca(s) passaram ao lado do bloco -- se ele apanha tudo, a cena ensina \
         que um colisor e' um chao, e a largura dele deixa de querer dizer alguma coisa"
    );
}

/// ⭐⭐⭐ **O PASSO 3 DO ANÚNCIO PRODUZ O QUE ELE DIZ: `Target Velocity` SATURA.**
///
/// ⛔ **Sem barra escolhida.** A saturação *é* o significado do modo: a lei é
/// `a = resistência · (alvo − v)`, que empurra cada vez menos e **pára** quando a peça
/// alcança o vento — logo a rapidez **nunca** passa da `Strength` do nó. O braço `Force`
/// acelera para sempre e passa-a, e é ele que prova que havia o que saturar.
///
/// ⚠️ **É a MESMA cena nos dois braços, com UM param de diferença** — o `mode`. Um segundo
/// documento montado à mão mediria outro programa.
#[test]
fn the_target_velocity_mode_caps_the_fall_and_force_does_not() {
    let (pico_forca, _) = corre(Some(0.0), 2.2);
    let (pico_alvo, _) = corre(Some(1.0), 2.2);
    assert!(
        pico_alvo <= GRAVIDADE + 1e-3,
        "com `Target Velocity` a rapidez chegou a {pico_alvo:.3}, acima da `Strength` \
         ({GRAVIDADE}) -- a lei `a = resistencia * (alvo - v)` nao pode ultrapassar o alvo, \
         entao ou o modo nao esta' a ser lido ou a cena mudou de forca"
    );
    assert!(
        pico_forca > GRAVIDADE * 1.5,
        "com `Force` a rapidez so' chegou a {pico_forca:.3} -- o braco de CONTROLO tem de \
         passar claramente a `Strength` ({GRAVIDADE}), senao os dois modos leriam igual e \
         este gate estaria a comparar duas quedas que ja' eram a mesma"
    );
}

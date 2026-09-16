//! Gates da cena de smoke do cérebro — ela tem de PRODUZIR o sujeito que o roteiro promete.

use super::*;
use ph2d_ecs::{SimWorld, StateMachineRuntime};

fn cena() -> SimWorld {
    let mut sim = SimWorld::new();
    assert_eq!(montar(sim.world_mut(), 1), 1);
    sim
}

fn por_nome(sim: &SimWorld, nome: &str) -> ph2d_ecs::Entity {
    let world = sim.world();
    let mut q = world
        .try_query::<(ph2d_ecs::Entity, &Name)>()
        .expect("query");
    for (e, n) in q.iter(world) {
        if n.as_str() == nome {
            return e;
        }
    }
    panic!("a cena tem de ter «{nome}»");
}

/// ⭐⭐⭐ **A cena produz o SUJEITO: uma porta com cérebro e outra sem.**
///
/// ⚠️ Sem as duas, a cena ensina metade — e a metade que falta é a que torna a outra legível.
#[test]
fn a_cena_traz_a_porta_e_o_controlo() {
    let sim = cena();
    let esq = por_nome(&sim, "Door");
    let dir = por_nome(&sim, "Door (no brain)");
    assert!(
        sim.world().get::<StateMachine>(esq).is_some(),
        "a da esquerda TEM cerebro"
    );
    assert!(
        sim.world().get::<StateMachine>(dir).is_none(),
        "o CONTROLO nao tem cerebro — e' isso que ele controla"
    );
    // ⚠️ **E as duas tabelas têm o MESMO tamanho**: um controlo com menos acções mediria outra
    // coisa (*«a da direita faz menos»*), e não o que um ESTADO compra.
    let n_esq = sim
        .world()
        .get::<SignalActions>(esq)
        .expect("tabela")
        .0
        .len();
    let n_dir = sim
        .world()
        .get::<SignalActions>(dir)
        .expect("tabela")
        .0
        .len();
    assert_eq!(n_esq, n_dir, "as duas tabelas tem de ter as MESMAS accoes");
}

/// ⭐⭐⭐ **O ciclo anda UM passo por toque** — o que o roteiro promete ao dono.
///
/// **Mutação que deve sangrar:** apagar o `gasto[i] = true` da lei (o ciclo daria a volta inteira
/// num tique, e a porta ficaria sempre na mesma cor).
#[test]
fn um_toque_do_botao_da_um_passo_no_ciclo() {
    let mut sim = cena();
    let porta = por_nome(&sim, "Door");
    let cfg = sim
        .world()
        .get::<StateMachine>(porta)
        .expect("cerebro")
        .clone();
    let mut rt = ph2d_ecs::state_machine::born(&cfg);
    ph2d_ecs::state_machine::advance(&cfg, &mut rt, &[]); // a entrada no inicial

    for esperado in [1_u8, 2, 0, 1] {
        let a = ph2d_ecs::state_machine::advance(&cfg, &mut rt, &["botao"]);
        assert_eq!(a.steps, 1, "um toque = um passo");
        assert_eq!(rt.current, esperado);
    }
    let _ = sim.world_mut();
}

/// ⭐⭐ **Cada estado acende uma placa DIFERENTE** — três nomes, três linhas.
///
/// ⚠️ Sem nomes distintos a máquina andava e o ecrã não mudava: o artista leria *«não faz nada»*
/// sobre um cérebro que funciona.
#[test]
fn os_tres_estados_anunciam_nomes_diferentes() {
    let sim = cena();
    let porta = por_nome(&sim, "Door");
    let m = sim.world().get::<StateMachine>(porta).expect("cerebro");
    let mut nomes: Vec<&str> = m.states.iter().map(|s| s.on_enter.as_str()).collect();
    assert_eq!(nomes.len(), 3);
    nomes.sort_unstable();
    nomes.dedup();
    assert_eq!(nomes.len(), 3, "tres nomes DISTINTOS");
    assert!(
        nomes.iter().all(|n| !n.is_empty()),
        "um nome vazio e' um estado CALADO — a placa dele nunca acenderia"
    );
}

/// ⚠️ **O botão é um RELÓGIO que repete** — sem `repeat` a cena dava um passo e parava, e o dono
/// leria *«travou»*.
#[test]
fn o_botao_repete_e_arranca_sozinho() {
    let sim = cena();
    let b = por_nome(&sim, "Button");
    let t = &sim.world().get::<Timers>(b).expect("o relogio").0[0];
    assert!(t.repeat, "sem repetir, a cena da' um passo e para");
    assert!(
        t.autostart,
        "sem autostart, nada acontece ate' alguem mexer"
    );
    assert_eq!(t.signal, "botao", "e' este o nome que as setas ouvem");
}

/// ⭐ **E a máquina de facto responde ao nome que o relógio publica** — a costura inteira, sem a
/// qual os dois lados podem estar certos e não se conhecerem.
#[test]
fn o_nome_que_o_relogio_publica_e_o_que_as_setas_ouvem() {
    let sim = cena();
    let b = por_nome(&sim, "Button");
    let sinal = sim.world().get::<Timers>(b).expect("o relogio").0[0]
        .signal
        .clone();
    let porta = por_nome(&sim, "Door");
    let m = sim.world().get::<StateMachine>(porta).expect("cerebro");
    assert!(
        m.transitions.iter().all(|t| t.on == sinal),
        "as tres setas tem de ouvir «{sinal}»"
    );
    // E o vivo ainda não existe — ele nasce na ponte, no primeiro avanço.
    assert!(sim.world().get::<StateMachineRuntime>(porta).is_none());
}

/// ⚠️ **O roteador nunca devolve acima do tecto que ele declara.**
///
/// ⭐ É este gate que mantém o [`CENAS`] honesto agora que o `montar` não tem `match` (uma cena só,
/// e o `clippy` recusa um braço único). *O número conta-se do produto, nunca de uma nota.*
///
/// **Mutação que deve sangrar:** `CENAS = 2` sem uma segunda cena — o roteador ofereceria um nível
/// que devolve a `=1`, e o artista leria *«a `=2` está partida»*.
#[test]
fn o_roteador_nunca_devolve_acima_do_tecto() {
    let mut vistos = std::collections::BTreeSet::new();
    for nivel in 0..=5u32 {
        let mut sim = SimWorld::new();
        let m = montar(sim.world_mut(), nivel);
        assert!(
            m >= 1 && m <= CENAS,
            "nivel {nivel} devolveu {m}, tecto {CENAS}"
        );
        vistos.insert(m);
    }
    assert_eq!(
        vistos.len() as u32,
        CENAS,
        "o roteador tem de ALCANCAR todas as cenas que declara: {vistos:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// ⭐⭐⭐ O QUE TEM CÉREBRO TEM DE TER CORPO (report do dono, 2026-09-15)
// ─────────────────────────────────────────────────────────────────────────────
//
// O dono correu a cena e escreveu: *«não apareceu no painel a seção state machine»*. A fiação do
// painel estava inteira — o defeito era **a cena**: o cérebro morava numa entidade **sem `Sprite`**,
// logo invisível e **impossível de apontar**, enquanto o que se vê no ecrã (as placas coloridas) não
// tem cérebro nenhum. O Inspector mostrava a verdade sobre o objecto escolhido.
//
// ⚠️ **Nenhum dos seis gates acima o via, e a cegueira é estrutural:** eles todos leem a cena como
// DADOS (*«a porta tem `StateMachine`?»*) e nenhum pergunta o que o roteiro promete — *«o artista
// consegue CHEGAR a ela?»*. ⛔ Uma cena correcta como dados e impossível como gesto ensina o
// contrário do que diz, que é pior que uma cena ausente (`CLAUDE.md` §5.0).

/// O ponto do ecrã onde o roteiro manda tocar — o meio do corpo da porta da esquerda.
///
/// ⚠️ **Não é o meio da PORTA**: a faixa de cor ocupa o topo dela, e um ponto ali apanharia a
/// faixa. O número sai da geometria que o [`cena_um`] monta, e é por isso que ele vive ao lado
/// dela.
const ONDE_O_DONO_TOCA: [f32; 2] = [-4.5, -0.7];

/// As entidades VISÍVEIS cujo rectângulo contém o ponto, pela ordem em que o mundo as devolve.
///
/// ⚠️ **É uma REPRODUÇÃO da lei do [`ph2d_render::pick_sprite_at_world`]**, não ela: aquela corre
/// sobre o mundo de PRESENTE (`RenderInstance` + `GlobalTransform`), que esta crate não monta. A
/// divergência é declarada e **fechada por asserção**: a reprodução só vale para sprites sem
/// rotação, escala, âncora nem deslocamento, e o gate afirma isso de cada uma antes de a medir.
///
/// ⚠️ **Um sprite `hidden` NÃO entra** — ele não emite `RenderInstance` (`sim_extract`), logo não
/// é apontável. É isso que torna as placas apagadas invisíveis ao dedo, e não só ao olho.
///
/// ⛔ **`world.get` por entidade, nunca um `Option<&Visibility>` dentro da consulta:** o
/// `try_query` do `bevy_ecs` devolve `None` quando **qualquer** componente dela não está registado,
/// e um mundo sem `Visibility` responderia *«ninguém»* — o defeito que a `lifetime.rs` já nomeia.
fn quem_o_dedo_apanha(sim: &SimWorld, ponto: [f32; 2]) -> Vec<String> {
    let world = sim.world();
    let mut q = world
        .try_query::<(ph2d_ecs::Entity, &Transform, &Sprite)>()
        .expect("a cena tem sprites");
    let mut apanhados = Vec::new();
    for (e, t, s) in q.iter(world) {
        let nome = world
            .get::<Name>(e)
            .map_or_else(|| format!("{e:?}"), |n| n.as_str().to_string());
        assert_eq!(
            (t.rotation, t.scale.x, t.scale.y, t.skew_x, t.skew_y),
            (0.0, 1.0, 1.0, 0.0, 0.0),
            "«{nome}»: a reproducao da lei do pick so' vale sem rotacao/escala/skew"
        );
        assert_eq!(
            (s.anchor, s.offset, s.centered),
            ([0.0, 0.0], [0.0, 0.0], true),
            "«{nome}»: a reproducao da lei do pick so' vale com a ancora no centro"
        );
        if world.get::<Visibility>(e).is_some_and(|v| v.hidden) {
            continue;
        }
        let dx = (ponto[0] - t.translation.x).abs();
        let dy = (ponto[1] - t.translation.y).abs();
        if dx <= s.size[0] * 0.5 && dy <= s.size[1] * 0.5 {
            apanhados.push(nome);
        }
    }
    apanhados
}

/// ⭐⭐⭐ **O cérebro mora num objecto que EXISTE NO ECRÃ.**
///
/// ⛔ Sem isto, a cena monta-se, os seis gates acima passam, e o roteiro manda o dono escolher uma
/// coisa que **não se pode escolher**: um objecto sem `Sprite` não emite `RenderInstance`, logo o
/// [`ph2d_render::pick_sprite_at_world`] nunca o devolve, e o Inspector — correctamente — mostra o
/// que ele escolheu em vez dele.
///
/// **Mutação que deve sangrar:** tirar o `Sprite` da porta (o estado exacto de 2026-09-15).
#[test]
fn o_que_tem_cerebro_tem_corpo() {
    let sim = cena();
    for nome in ["Door", "Door (no brain)"] {
        let e = por_nome(&sim, nome);
        let s = sim
            .world()
            .get::<Sprite>(e)
            .unwrap_or_else(|| panic!("«{nome}» tem de ter CORPO — sem sprite ninguem lhe toca"));
        assert!(
            s.size[0] > 0.0 && s.size[1] > 0.0,
            "«{nome}»: um corpo de area zero e' o mesmo que nenhum"
        );
        assert!(
            !sim.world().get::<Visibility>(e).is_some_and(|v| v.hidden),
            "«{nome}»: um corpo ESCONDIDO nao emite RenderInstance — e' o mesmo que nenhum"
        );
    }
}

/// ⭐⭐⭐ **E o dedo apanha-o: no ponto que o roteiro nomeia, a porta é o único objecto dela ali.**
///
/// ⚠️ **O chão é esperado e não é ambiguidade:** ele é a primeira raiz criada, desenha ATRÁS de
/// tudo, e nenhum outro objecto da porta disputa aquele ponto — logo o dedo não tem por onde se
/// enganar. ⛔ O que este gate proíbe é a faixa de cor cobrir o corpo: aí o clique escolheria a
/// placa, que é exactamente o report de 2026-09-15.
///
/// **Mutação que deve sangrar:** pôr a faixa por cima do corpo inteiro (o desenho de 2026-09-15).
#[test]
fn o_dedo_do_dono_apanha_a_porta_e_nao_uma_placa() {
    let sim = cena();
    let apanhados = quem_o_dedo_apanha(&sim, ONDE_O_DONO_TOCA);
    assert!(
        apanhados.contains(&"Door".to_string()),
        "o ponto do roteiro tem de apanhar a porta; apanhou {apanhados:?}"
    );
    let intrusos: Vec<&String> = apanhados
        .iter()
        .filter(|n| n.as_str() != "Door" && n.as_str() != "Floor")
        .collect();
    assert!(
        intrusos.is_empty(),
        "so' o chao pode partilhar o ponto com a porta; tambem la' estao {intrusos:?}"
    );
}

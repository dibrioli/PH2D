//! Gates dos **PRODUTORES** do `motion.wave` — os *Producers* do AE Wave World (doc 89,
//! folha 06, célula 35).
//!
//! ⚠️ Arquivo próprio por TETO DE LOC (HR-18): o `lib_tests.rs` cruzou as 700 linhas ao
//! receber esta família. O corte é por ASSUNTO — ali mora a lei do passo e a borda, aqui a
//! pergunta *"de onde uma onda pode nascer?"*.

use super::tests::params;
use super::*;

/// Um campo injectado numa célula só, no formato que a família `field.*` emite.
fn injected_at(p: &Params, cell: usize, amount: f32) -> Vec<f32> {
    let mut v = vec![0.0f32; p.count()];
    v[cell] = amount;
    v
}

/// Corre `ticks` com o mesmo campo injectado todos os tiques.
fn run_injected(p: &Params, inject: &[f32], ticks: usize) -> Vec<Vec<f32>> {
    let mut state = Stream::new(0);
    let mut frames = Vec::new();
    for k in 0..ticks {
        let t = k as f32 / 60.0;
        let out = simulate(None, &state, inject, t, p);
        state = out.clone();
        frames.push(scalar_col(&out, "wave_h"));
    }
    frames
}

/// ⭐⭐ **A ENTREGA: um produtor fora do centro faz ondas.** É a mesma lei que a composição de
/// quatro nós entregava, agora numa aresta — e sem o artista ter de saber que a coluna de
/// estado se chama `wave_h`.
#[test]
fn an_injected_producer_makes_waves_from_where_it_sits() {
    let mut p = params(21, 21, 0.4, 0.0);
    p.inject_gain = 0.5;
    let off_centre = 10 * 21 + 4; // linha do meio, bem à esquerda do centro
    let frames = run_injected(&p, &injected_at(&p, off_centre, 1.0), 90);
    let last = frames.last().expect("tiques");
    // O berço está onde o produtor está…
    assert!(
        last[off_centre].abs() > 0.05,
        "o bercao: {}",
        last[off_centre]
    );
    // …e a ondulação ALCANÇOU o outro lado (um bump que não propaga é tinta, não fonte).
    let far = 10 * 21 + 16;
    assert!(
        last[far].abs() > 1e-3,
        "alcancou o outro lado: {}",
        last[far]
    );
}

/// ⭐ **`Source Strength = 0` é a porta INERTE, e o gate mede o par.** Um documento de hoje
/// tem a porta desligada, então nada nele muda — e ligar o fio sem subir o ganho também não.
#[test]
fn the_port_is_inert_until_the_strength_says_otherwise() {
    let p = params(21, 21, 0.4, 0.0); // inject_gain = 0.0
    let field = injected_at(&p, 10 * 21 + 4, 1.0);
    assert_eq!(
        run_injected(&p, &field, 60),
        run_injected(&p, &[], 60),
        "com ganho zero, o campo ligado nao pode mover um bit"
    );
    // ⚠️ O CONTROLE: com ganho, ele move — senão a igualdade acima seria vácua.
    let mut on = p;
    on.inject_gain = 0.5;
    assert_ne!(run_injected(&on, &field, 60), run_injected(&on, &[], 60));
}

/// ⚠️ **Os produtores SOMAM, e é isso que os faz compor.** O pino de Dirichlet do centro
/// CRAVA — e é por isso que ele apaga o que outra fonte pôs, o defeito que a folha mediu.
/// Dois produtores em sítios diferentes têm de dar um campo diferente de qualquer um deles.
#[test]
fn two_producers_compose_where_a_pin_would_erase() {
    let mut p = params(21, 21, 0.4, 0.0);
    p.inject_gain = 0.5;
    let (a, b) = (10 * 21 + 4, 10 * 21 + 16);
    let one = run_injected(&p, &injected_at(&p, a, 1.0), 60);
    let mut both_field = injected_at(&p, a, 1.0);
    both_field[b] = 1.0;
    let both = run_injected(&p, &both_field, 60);
    assert_ne!(one.last(), both.last(), "o segundo produtor tem de contar");
    // E o primeiro berço continua lá — somar não é substituir.
    assert!(
        both.last().expect("tiques")[a].abs() > 0.05,
        "o 1o bercao sobreviveu ao 2o"
    );
}

/// ⚠️ **A injecção entra SÓ no `h`, nunca no par do leapfrog.** Escrevê-la também no `prev`
/// diria que a fonte já lá estava no tique passado — o campo leria velocidade zero onde há uma
/// frente a nascer, e a onda sairia com metade da energia.
#[test]
fn the_source_lands_on_the_height_not_on_its_own_past() {
    let mut p = params(11, 11, 0.4, 0.0);
    p.inject_gain = 1.0;
    let field = injected_at(&p, 5 * 11 + 5, 1.0);
    // Um tique só: o `h` recebeu a fonte…
    let out = simulate(None, &Stream::new(0), &field, 0.0, &p);
    let h0 = scalar_col(&out, "wave_h");
    // (o 1.º tique semeia plano — a fonte entra no 2.º, que é quando o passo corre)
    let out2 = simulate(None, &out, &field, 1.0 / 60.0, &p);
    let h = scalar_col(&out2, "wave_h");
    let prev = scalar_col(&out2, "wave_prev");
    assert!(h0.iter().all(|x| *x == 0.0), "o 1o tique semeia plano");
    assert!(
        h[5 * 11 + 5] > 0.5,
        "o `h` recebeu a fonte: {}",
        h[5 * 11 + 5]
    );
    assert!(
        prev[5 * 11 + 5] == 0.0,
        "o `prev` NAO a recebeu: {}",
        prev[5 * 11 + 5]
    );
}

/// **A porta é a ÚLTIMA do manifesto** — as arestas de um documento salvo guardam o ÍNDICE,
/// então inserir no meio trocaria as ligações de toda cena já autorada.
#[test]
fn the_producer_port_is_appended_never_inserted() {
    let names: Vec<_> = MANIFEST.inputs.iter().map(|i| i.name).collect();
    assert_eq!(
        names,
        vec!["drive", "state", "inject"],
        "a ordem e' contrato"
    );
}

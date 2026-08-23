//! Os gates da porta dos modais e do relógio que eles congelam.

use super::{chrome_dt, note_stall, take_stall, timed};

/// ⭐ **O GATE-MÃE, e ele é o sintoma do Enio escrito em números.**
///
/// *"não vejo em nenhum lugar a mensagem"*: um toast criado depois de um diálogo que ficou 20 s
/// aberto era morto pelo `tick` do quadro seguinte, porque o `wall_dt` daquele quadro **inclui o
/// diálogo**. Com o congelamento descontado, o que sobra é o tempo em que a tela de facto andou.
#[test]
fn a_frozen_loop_does_not_age_the_message_it_was_about_to_show() {
    // ⚠️ **O TTL é lido do dono** (`ph2d_editor::Toast`), nunca escrito aqui: um `3.0` local viraria
    // uma segunda verdade sobre quanto uma mensagem dura, e no dia em que o dono mudasse este gate
    // continuaria verde a medir um número que já não existe.
    let ttl = f64::from(ph2d_editor::Toast::DEFAULT_TTL_S);

    // O quadro que abriu o diálogo: 20,016 s de parede, dos quais 20 s congelado.
    let wall = 20.016;
    let frozen = 20.0;

    assert!(
        wall > ttl,
        "a fixture só prova alguma coisa se o quadro sozinho já matasse o toast — \
         {wall} s contra um TTL de {ttl} s"
    );
    let ui = chrome_dt(wall, frozen);
    assert!(
        ui < ttl,
        "com o congelamento descontado sobram {ui} s, e a mensagem tem de sobreviver ao \
         tick do quadro seguinte (TTL {ttl} s)"
    );
    // E o que sobra é o quadro de verdade, não um número inventado.
    assert!(
        (ui - 0.016).abs() < 1e-9,
        "o que sobra é o tempo em que a tela ANDOU: {ui}"
    );
}

/// ⚠️ **Sem diálogo nenhum, o relógio do chrome é o relógio.** Sem esta metade, a cura degenerada
/// («devolver sempre zero») passaria no gate acima — e aí nenhum toast expiraria nunca.
#[test]
fn without_a_dialog_the_chrome_clock_is_the_wall_clock() {
    assert!((chrome_dt(0.016, 0.0) - 0.016).abs() < 1e-9);
    assert!((chrome_dt(1.5, 0.0) - 1.5).abs() < 1e-9);
}

/// ⚠️ **Nunca negativo.** Dois relógios a medir o mesmo intervalo podem discordar por microssegundos,
/// e um `dt` negativo faria o toast **rejuvenescer** — um mundo em que ele nunca expira.
#[test]
fn the_chrome_clock_never_runs_backwards() {
    assert!((chrome_dt(1.0, 1.000_001) - 0.0).abs() < 1e-12);
}

/// ⭐ **O congelamento é tirado UMA vez.** Se ficasse pousado, todo quadro seguinte descontaria o
/// mesmo diálogo e a UI congelaria de vez: `chrome_dt` daria zero para sempre.
#[test]
fn the_stall_is_taken_once_and_then_it_is_gone() {
    let _ = take_stall(); // o processo é partilhado com outros gates; começa limpo
    note_stall(std::time::Duration::from_millis(500));
    note_stall(std::time::Duration::from_millis(250));
    assert!(
        (take_stall() - 0.75).abs() < 1e-9,
        "dois diálogos no mesmo quadro somam"
    );
    assert!(
        (take_stall() - 0.0).abs() < 1e-12,
        "e o segundo `take` do mesmo quadro não pode devolver o mesmo congelamento outra vez"
    );
}

/// ⭐ **OS MODAIS DO MÓDULO 3D PASSAM PELA PORTA** — um `rfd::FileDialog` aberto à mão volta a
/// congelar o loop **sem declarar**, e a mensagem escrita a seguir volta a viver um quadro só.
///
/// ⚠️ **O escopo é `field3d_*`, e isso é uma medição, não um descuido.** Há **25** chamadas de
/// `rfd::FileDialog` em **12** arquivos deste shell (medido 2026-08-22); as outras 23 são de outras
/// linhas — `sculpt3d`, image tools, tokens, sheet, texto vetorial — e continuam a perder a
/// mensagem que escrevem a seguir. Alargar este gate a elas hoje entregaria um vermelho ao
/// integrador sobre código que não é desta linha. *O defeito é da casa, tem endereço no doc §38, e
/// a porta já existe para quem chegar lá.*
#[test]
fn every_field3d_modal_goes_through_the_door() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut bad = Vec::new();
    for entry in std::fs::read_dir(&dir)
        .expect("o src do shell existe")
        .flatten()
    {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        if !name.starts_with("field3d_") || !name.ends_with(".rs") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        // ⚠️ A agulha é a CHAMADA que bloqueia, não o tipo: construir o `FileDialog` é inofensivo
        // (e os dois módulos continuam a fazê-lo, para montar os filtros).
        //
        // ⚠️ **COMENTÁRIOS FORA**, e isto não é higiene: a primeira versão deste gate leu o arquivo
        // inteiro e reprovou sobre o **comentário** que explica a regra — o texto que diz *"nunca
        // chame isto direto"* contém, por construção, exactamente a agulha. Um gate que lê a prosa
        // sobre a lei em vez do código que a obedece reprova quem a documenta.
        for line in text.lines().filter(|l| !l.trim_start().starts_with("//")) {
            for needle in [".save_file()", ".pick_file()"] {
                if line.contains(needle) {
                    bad.push(format!("{name} chama `{needle}` fora da porta"));
                }
            }
        }
    }
    assert!(
        bad.is_empty(),
        "diálogo modal aberto sem declarar o congelamento — use `crate::modal::save_file` / \
         `pick_file`:\n  {}",
        bad.join("\n  ")
    );
}

/// ⭐ **O LOOP LÊ O NÚMERO DESCONTADO** — e sem este gate o [`chrome_dt`] podia estar perfeito e
/// não ser usado por ninguém.
///
/// ⚠️ É a lição da W34 uma wave depois: *provar o cálculo não prova a alcançabilidade dele*. Os
/// gates acima medem a função pura; este mede que os dois relógios do chrome — os toasts e a UI
/// viva — de facto a consomem, e que o medidor de fps e a simulação **continuam** com o `wall_dt`
/// inteiro (para eles o tempo passou mesmo).
#[test]
fn the_chrome_clock_reads_the_discounted_dt() {
    let src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/render_loop/mod.rs"),
    )
    .expect("o loop existe");
    // ⚠️ Comentários fora, pela razão escrita no gate da porta.
    let code: String = src
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");

    for needed in ["toasts.tick(ui_dt", "hero.tick_motion(ui_dt)"] {
        assert!(
            code.contains(needed),
            "o relógio do chrome tem de andar com o `ui_dt` (o `wall_dt` menos o congelamento \
             declarado) — não achei `{needed}`"
        );
    }
    // E o outro lado: quem mede o QUADRO continua com o número inteiro.
    assert!(
        code.contains("self.fixed_step.advance(wall_dt)"),
        "a simulação lê o `wall_dt` INTEIRO — para ela o tempo passou mesmo, e descontar o \
         congelamento faria a cena saltar menos do que o relógio de parede diz"
    );
}

/// ⭐ **A PORTA CRONOMETRA O QUE PASSA POR ELA** — e este gate existe porque uma prova de mutação o
/// exigiu: com o cronómetro dentro do `save_file`, tirá-lo de lá deixava a suíte inteira VERDE.
///
/// ⚠️ **Só um piso, nunca um teto.** Um `>=` contra o que se dormiu é uma afirmação sobre o
/// escalonador que ele só pode cumprir por excesso; um teto seria um gate de RAZÃO de relógio — a
/// família de flake que este repo já paga cinco vezes.
#[test]
fn the_door_times_what_goes_through_it() {
    let _ = take_stall();
    let out = timed(|| {
        std::thread::sleep(std::time::Duration::from_millis(25));
        "o que a porta devolve"
    });
    assert_eq!(
        out, "o que a porta devolve",
        "a porta não pode comer o valor"
    );
    let stalled = take_stall();
    assert!(
        stalled >= 0.010,
        "a porta tem de DECLARAR o que congelou; declarou {stalled} s depois de 25 ms parada"
    );
}

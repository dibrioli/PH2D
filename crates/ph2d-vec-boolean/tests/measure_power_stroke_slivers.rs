//! **Sonda: o Power Stroke deixa lascas visíveis à volta da fita?** (report do Enio, 2026-08-30)
//!
//! O report foi *«manchas animadas parecendo TV antiga»* à volta das formas vectoriais, e a
//! segunda mensagem apontou a ferramenta **Width**. A largura viva re-coze a fita **por quadro**
//! (`profile_live.rs`, ADR-0148), e a fita passa por um *sweep* booleano: `ribbon_into` →
//! `Region::of(NonZero)` → `drop_slivers`. Onde a curvatura aperta mais que a largura os dois
//! trilhos cruzam-se, e o sweep emite peças degeneradas.
//!
//! `drop_slivers` varre as que têm **área ≤ 1e-4 do total**. ⚠️ Uma lasca a `1,1e-4` **sobrevive**
//! — e uma peça com um milésimo da área de um traço longo ainda é tinta que se vê.
//!
//! ## O que esta sonda responde, e o que ela NÃO responde
//!
//! Responde: *quantas peças o assador devolve, e qual a razão entre a menor e a maior*. Se houver
//! peças logo acima do piso, a lista de suspeitos do report ganha um nome com número.
//!
//! ⛔ **Não responde à parte «animada».** A fita é função pura da geometria de MUNDO (o memo do
//! `profile_live` tem essa chave), logo um documento parado coze a mesma fita — o que é afirmado
//! pelo gate `the_ribbon_is_a_pure_function_of_its_input` abaixo, que **não** é sonda.
//!
//! `#[ignore]` na sonda: ela imprime, não afirma. Corra com
//! `cargo test -p ph2d-vec-boolean --release measure_power_stroke_slivers -- --ignored --nocapture`.

use ph2d_vec_scene::{Rgba8, StrokeSpec, VecPath, VecVertex, WidthProfile};

/// Uma senoide de 4 cúbicas — a curvatura muda de sinal, os trilhos cruzam-se, e é onde o
/// doc-comment do `drop_slivers` diz que as lascas nascem. ⚠️ Uma fixtura RECTA não contém o
/// fenómeno e daria esta sonda por limpa sem ter medido nada.
fn sine(width: f64) -> VecPath {
    let mut verts = Vec::new();
    for i in 0..5 {
        let x = f64::from(i) * 2.0;
        let y = if i % 2 == 0 { -1.0 } else { 1.0 };
        let mut v = VecVertex::corner([x, y]);
        v.in_handle = [x - 0.9, y];
        v.out_handle = [x + 0.9, y];
        verts.push(v);
    }
    let mut p = VecPath {
        verts,
        closed: false,
        ..VecPath::default()
    };
    p.stroke = Some(StrokeSpec::new(Rgba8::new(0, 0, 0, 255), width));
    p
}

/// ⚠️ Do **catálogo**, nunca de números escritos aqui: um perfil acrescentado lá tem de entrar
/// nesta medição sozinho, senão a sonda envelhece calada.
fn profiles() -> Vec<(&'static str, WidthProfile)> {
    ph2d_vec_scene::WIDTH_PRESETS
        .iter()
        .filter(|p| !p.profile.is_uniform())
        .map(|p| (p.key, p.profile))
        .collect()
}

#[test]
#[ignore = "sonda de calibração (imprime, não afirma) — rode com --ignored --nocapture"]
fn measure_power_stroke_slivers() {
    println!("preset                                    w   pecas  menor/maior      menor/piso");
    for (key, prof) in profiles() {
        // Larguras crescentes: quanto mais grossa a fita para a mesma curvatura, mais os trilhos
        // se cruzam. É a varredura que contém o fenómeno.
        for w in [0.2_f64, 0.6, 1.2, 2.0, 3.0] {
            let path = sine(w);
            let out = ph2d_vec_boolean::power_stroke(&path, &prof.to_stops());
            let areas: Vec<f64> = out.iter().map(ph2d_vec_boolean::area).collect();
            let total: f64 = areas.iter().sum();
            let (mn, mx) = areas
                .iter()
                .fold((f64::MAX, 0.0_f64), |(a, b), &x| (a.min(x), b.max(x)));
            if out.is_empty() {
                println!("{key:<40} {w:>4.1}  (vazio)");
                continue;
            }
            // O piso do `drop_slivers` é `total * 1e-4`. «menor/piso» perto de 1 = uma peça que
            // escapou por pouco, e é essa que se vê como mancha solta.
            let floor = total * 1e-4;
            println!(
                "{key:<40} {w:>4.1}  {:>5}  {:>12.6}  {:>13.2}",
                out.len(),
                if mx > 0.0 { mn / mx } else { 0.0 },
                if floor > 0.0 { mn / floor } else { 0.0 },
            );
        }
    }
}

/// **A fita é função PURA da entrada — cozer duas vezes dá os MESMOS bits.**
///
/// Isto não é higiene: a largura viva re-coze por quadro, e o memo do `profile_live` tem por
/// chave a geometria de mundo. Se o assador tivesse qualquer dependência de estado — ordem de
/// iteração instável, um acumulador global, um `HashMap` — a fita mudaria de quadro para quadro
/// com o documento parado, e a borda **cintilaria**. É a metade «animada» do report de 2026-08-30,
/// e sem este gate ninguém saberia dizer se ela é possível.
///
/// ⚠️ O `linesweeper` guarda a ordem dos pares num `FxHashMap` — determinístico por semente fixa,
/// mas essa é uma propriedade da dependência, não nossa. Este gate torna-a **nossa**: no dia em
/// que uma subida a trocar por um `HashMap` com semente aleatória, ele fica vermelho aqui em vez
/// de virar cintilação no canvas de alguém.
#[test]
fn the_ribbon_is_a_pure_function_of_its_input() {
    for (key, prof) in profiles() {
        for w in [0.2_f64, 1.2, 3.0] {
            let path = sine(w);
            let stops = prof.to_stops();
            let a = ph2d_vec_boolean::power_stroke(&path, &stops);
            let b = ph2d_vec_boolean::power_stroke(&path, &stops);
            assert_eq!(
                a.len(),
                b.len(),
                "{key} @ w={w}: dois cozimentos da MESMA entrada deram {} e {} peças",
                a.len(),
                b.len()
            );
            for (i, (x, y)) in a.iter().zip(b.iter()).enumerate() {
                assert_eq!(
                    x.verts.len(),
                    y.verts.len(),
                    "{key} @ w={w}: a peça {i} mudou de contagem de âncoras entre cozimentos"
                );
                for (j, (u, v)) in x.verts.iter().zip(y.verts.iter()).enumerate() {
                    assert!(
                        u.anchor[0].to_bits() == v.anchor[0].to_bits()
                            && u.anchor[1].to_bits() == v.anchor[1].to_bits(),
                        "{key} @ w={w}: a âncora {j} da peça {i} moveu-se entre dois cozimentos \
                         da MESMA entrada ({:?} vs {:?}). A largura viva re-coze por quadro — \
                         isto é cintilação na borda do traço.",
                        u.anchor,
                        v.anchor
                    );
                }
            }
            // Metade JUSTA: uma entrada que não produzisse fita nenhuma tornaria tudo acima
            // vacuamente verdadeiro.
            assert!(
                !a.is_empty(),
                "{key} @ w={w}: o assador devolveu vazio — esta fixtura não exercita nada"
            );
        }
    }
}

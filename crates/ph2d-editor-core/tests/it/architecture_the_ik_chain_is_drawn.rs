//! **Arch-gate: a faixa da corrente de IK CHEGA À TELA.**
//!
//! ⛔⛔ **Report do dono** (2026-09-14): *«não temos uma linha indicativa do IK Chain»*. O `Chain` é
//! um número num painel e o que ele significa é **quais ossos obedecem** — uma pergunta sobre a
//! cena, que só o desenho responde.
//!
//! ⚠️ **A lei da porta está gateada onde ela vive** (`ph2d_app_skeleton::goal::chains` mede quais
//! juntas cada `Chain` cobre); o que NENHUM teste de unidade alcança é o **fio**: a chamada vive
//! numa fase do `render_frame`, que precisa de janela e GPU. É a forma do
//! `the_onion_speaks_the_clip_clock`, e mora aqui pela mesma razão.
//!
//! ⛔ *Uma porta sem chamador e uma lei ausente produzem o mesmo app* — e esta linha já pagou isso
//! duas vezes (o `sprite_world_to_uv` da W10, o `dock_columns::close` do §5).

use std::path::Path;

#[test]
fn the_governed_chain_reaches_the_canvas() {
    let f = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crates/<x>/ tem dois pais")
        .join("shells/desktop/src/render_loop/fase_vector_bone_overlay.rs");
    let src: String = std::fs::read_to_string(&f)
        .expect("a fase que desenha o esqueleto no canvas")
        .lines()
        .map(|l| l.split_once("//").map_or(l, |(antes, _)| antes))
        .collect::<Vec<_>>()
        .join("\n");
    // Controlo positivo: é ESTA a fase que desenha o esqueleto — sem ela o gate mede o nada.
    assert!(
        src.contains("draw_bones("),
        "{} deixou de desenhar os ossos — este gate perdeu o sujeito",
        f.display()
    );
    assert!(
        src.contains("draw_chains(&crate::skeleton_goal::chains(sim)")
            || src.contains("draw_chains(\n                    &crate::skeleton_goal::chains(sim)"),
        "{} não desenha a corrente que a âncora governa: o `Chain` fica um número sem nada na tela \
         que diga quais ossos ele apanha.",
        f.display()
    );
    // ⚠️ **Por BAIXO dos ossos, e é o que a torna um REALCE** — a cheio e por cima ela esconderia o
    // corpo do osso, que é o que o artista agarra.
    let faixa = src.find("draw_chains(").expect("afirmado acima");
    let ossos = src.find("draw_bones(").expect("afirmado acima");
    assert!(
        faixa < ossos,
        "{} desenha a faixa da corrente POR CIMA dos ossos: ela é um realce, não um desenho novo",
        f.display()
    );
}

/// ⭐⭐⭐ **MUDAR O `Chain` RE-CAPTURA O LADO DA DOBRA** — ordem do dono (2026-09-14: *«o lado da
/// dobra é capturado no momento em que carrega Add IK e sempre que IK Chain for mudado»*).
///
/// ⚠️ **A LEI tem gate onde ela vive** (`side_for_chain`, na `ph2d-app-skeleton`); o que nenhum
/// teste de unidade alcança é o FIO — o braço que aplica o número vive numa fase do `render_frame`.
///
/// ⛔ E a agulha exige a **corrente NOVA**: ler o lado com o `chain` que ainda lá está devolveria o
/// que já existe, e o gesto ficaria a não fazer nada com a suíte inteira verde.
#[test]
fn changing_the_chain_recaptures_the_bend_side() {
    let f = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crates/<x>/ tem dois pais")
        .join("shells/desktop/src/render_loop/fase_bone_smart_and_knobs.rs");
    let src: String = std::fs::read_to_string(&f)
        .expect("a fase que aplica os números do osso")
        .lines()
        .map(|l| l.split_once("//").map_or(l, |(antes, _)| antes))
        .collect::<Vec<_>>()
        .join("\n");
    // Controlo positivo: é ESTE o sítio que escreve o `Chain`.
    assert!(
        src.contains("IkKnob::Chain => g.chain ="),
        "{} deixou de escrever o Chain — este gate perdeu o sujeito",
        f.display()
    );
    assert!(
        src.contains("ph2d_skeleton_live::goal::side_for_chain("),
        "{} escreve o `Chain` e não re-captura o lado da dobra: o bit guardado passa a falar de uma \
         corrente que já não é a que está debaixo do artista (ordem do dono, 2026-09-14).",
        f.display()
    );
    assert!(
        src.contains("g.bend = lado;"),
        "{} calcula o lado novo e DEITA-O FORA — citar a porta não é consultá-la.",
        f.display()
    );
    // ⛔⛔ **E o MISTO é a EXCEPÇÃO da re-captura** (2026-09-14). O que a porta devolve é um lado
    // FORÇADO (`Ccw`/`Cw`); escrevê-lo por cima de um `Mixed` apagaria a escolha do artista em
    // silêncio, no gesto mais provável de todos — pôr o `IK Chain` no tamanho certo DEPOIS de
    // escolher o modo. ⭐ E o misto não precisa dela: ele lê o lado de cada junta da pose autorada a
    // cada resolução, logo uma corrente maior traz juntas novas já com o lado delas.
    assert!(
        src.contains("BendSide::Mixed"),
        "{} re-captura o lado sem excluir o modo MISTO: mudar o `IK Chain` apagaria a escolha do \
         artista em silêncio.",
        f.display()
    );
}

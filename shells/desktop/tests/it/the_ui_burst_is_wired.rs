//! ⭐⭐⭐ **A POEIRA DE IMPACTO está fiada, das quatro pontas** (estudo de UI viva, D2).
//!
//! ⚠️ **A shell não é alcançável de um teste** — o `App` segura uma surface de janela real. É a
//! mesma razão pela qual o undo do filtro do sculpt3d e a fileira *Type* do traço têm um gate que
//! lê o FONTE.
//!
//! ⚠️⚠️ **Cada agulha aqui mata a feature sozinha, e nenhuma quebra a compilação:** um campo que
//! ninguém envelhece, uma armação que ninguém dispara, e um desenho que ninguém chama são todos
//! **verdes de compilador**. É exactamente a espécie 2 do `CLAUDE.md` §5.0 — *o fio está completo e
//! o valor não chega a um consumidor* — e esta linha já a pagou duas vezes hoje.

use std::fs;
use std::path::Path;

/// O fonte **sem comentários** — senão o gate aprova quem documenta a lei em vez de quem a obedece.
fn code(rel: &str) -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(rel);
    fs::read_to_string(&p)
        .unwrap_or_else(|e| panic!("{}: {e}", p.display()))
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// **As QUATRO pontas do fio**, e cada uma mata a feature sozinha.
#[test]
fn the_ui_burst_is_wired_at_all_four_ends() {
    for (ficheiro, agulha, o_que) in [
        (
            "app_state.rs",
            "ui_burst: ph2d_editor::motion_burst::BurstField",
            "o campo do quadro",
        ),
        (
            "render_loop/mod.rs",
            "self.ui_burst.tick(",
            "o RELOGIO: sem ele a faisca nasce e fica parada para sempre",
        ),
        (
            "render_loop/mod.rs",
            "crate::ui_burst_paint::paint(&self.ui_burst,",
            "o DESENHO: sem ele a lei corre e ninguem ve' nada",
        ),
        (
            "radial_input.rs",
            "self.ui_burst.emit(",
            "a ARMACAO: sem ela nada emite, e o resto e' codigo morto",
        ),
    ] {
        assert!(
            code(ficheiro).contains(agulha),
            "{o_que} saiu do `{ficheiro}` (`{agulha}`)"
        );
    }
}

/// ⛔⛔ **O RELÓGIO É O DO CHROME, e não o do quadro.**
///
/// Um diálogo modal congela o laço; uma faísca não pode envelhecer enquanto nada é desenhado. É a
/// mesma lei que o `crate::modal` impõe aos toasts, e ela custou um report (*"não vejo em nenhum
/// lugar a mensagem"*, 2026-08-22) para ser descoberta.
#[test]
fn the_burst_ages_on_the_chrome_clock_never_on_the_wall_clock() {
    let render = code("render_loop/mod.rs");
    let i = render
        .find("self.ui_burst.tick(")
        .expect("o relogio existe");
    let linha = &render[i..render[i..].find('\n').map_or(render.len(), |o| i + o)];
    assert!(
        linha.contains("ui_dt"),
        "a poeira envelhece com outro relogio que nao o `ui_dt`: `{linha}` - com o `wall_dt` uma \
         faisca morre inteira dentro de um dialogo modal, sem ninguem a ver"
    );
}

/// ⛔⛔⛔ **A CERCA é a do artista, e não uma cópia por omissão.**
///
/// ⚠️ A 1.ª redacção da armação passou um `UiMotion::default()`: a cerca teria lido o carácter de
/// fábrica em vez do que está no `~/.ph2d/prefs.txt`, e um `reduced_motion=1` esquecido continuaria
/// a faiscar. *Perguntar a uma cópia por omissão é não perguntar.*
#[test]
fn the_burst_asks_the_artists_motion_not_a_default_copy() {
    let radial = code("radial_input.rs");
    let i = radial
        .find("self.ui_burst.emit(")
        .expect("a armacao existe");
    let janela = &radial[i..(i + 120).min(radial.len())];
    assert!(
        !janela.contains("UiMotion::default()"),
        "a cerca voltou a perguntar a uma copia por omissao - o `reduced_motion` do artista deixa \
         de ser lido"
    );
    assert!(
        janela.contains("&hero.motion"),
        "a armacao nao passa o `UiMotion` do ARTISTA: `{janela}`"
    );
}

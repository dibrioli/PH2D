//! **O anel do pincel 3D é desenhado no ACERTO, não no pixel do mouse**
//! (ADR-0150, W12).
//!
//! ## Por que isto é um arch-gate e não um teste de unidade
//!
//! O anel é DESENHO puro no shell: o `cursor_mark` precisa de uma
//! `Sculpt3dScene`, que precisa de um `wgpu::Device` — e a decisão que importa
//! (*de onde vem o CENTRO*) é uma linha dentro de um bloco que só roda com
//! janela. Mutá-la deixa a workspace inteira VERDE, que é exatamente o critério
//! que o Painter usou para o `the_brush_ring_wears_the_live_dab_rotor`.
//!
//! ## O que ele afirma, e por que essa propriedade
//!
//! O report do Enio foi *"o lugar onde o mouse toca não corresponde ao local na
//! malha"*, e as sondas headless inocentaram a matemática (desvio de 0,0024 sob
//! topologia dinâmica; 0,0005 px no ida-e-volta da câmera). O que faltava era um
//! INSTRUMENTO na janela — e ele só é instrumento se o centro do anel for o
//! **acerto reprojetado**, o inverso exato do raio que o dab usa. Centrado no
//! pixel cru ele viraria um enfeite que concorda com o mouse por construção,
//! **incapaz de mostrar o defeito que existe para revelar**.
//!
//! Mutação que sangra: trocar o `project(...)` por `(x, y)`.

use std::fs;

const CURSOR: &str = "src/sculpt3d_cursor.rs";

fn source(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("não consegui ler {path}: {e}"))
}

/// **O centro do anel sai da PROJEÇÃO do acerto.**
#[test]
fn the_ring_is_centred_on_the_reprojected_hit() {
    let src = source(CURSOR);
    // ⚠️ As âncoras são CHAMADAS (`.pick(` / `.project(`), não a expressão
    // inteira: a primeira versão pedia `self.pick(x, y)` e o `rustfmt` já a
    // tinha reescrito — um gate ancorado na FORMATAÇÃO expira na primeira
    // passagem do formatador, não na primeira regressão.
    assert!(
        src.contains(".pick(x, y)"),
        "o cursor não pergunta onde o raio ACERTA — sem isso ele não pode \
         mostrar um pick errado, que é a única razão de ele existir"
    );
    assert!(
        src.contains(".project("),
        "o cursor não REPROJETA o acerto: sem o inverso exato do `ray_through` \
         o anel concordaria com o mouse por construção"
    );
    // E o fallback existe, com a semântica certa: no vazio o anel fica no pixel
    // cru, que é onde a ÓRBITA vai começar.
    assert!(
        src.contains("hit.unwrap_or((x, y))"),
        "o anel não tem fallback para o vazio — ou ele some (escondendo que o \
         gesto mudou de significado) ou ele mente sobre um acerto que não houve"
    );
}

/// **O traço é sob `Affine::IDENTITY`.**
///
/// No Vello o transform do `stroke` **multiplica a largura**: entregar um afim
/// de mundo→tela transforma 1,5 px em `1,5 × pixels-por-unidade` de tinta. Não é
/// hipótese — é o que aconteceu com o realce do Flip (smoke, 2026-07-13) e o
/// motivo pelo qual o `flip_cursor` sempre desenhou assim.
#[test]
fn the_ring_is_stroked_in_screen_space() {
    let src = source("src/render_loop/mod.rs");
    let at = src
        .find("O ANEL DO PINCEL 3D")
        .expect("o bloco do anel sumiu do render_loop");
    let block = &src[at..(at + 2000).min(src.len())];
    assert!(
        block.contains("Affine::IDENTITY"),
        "o anel é traçado sob um afim de mundo — no Vello isso MULTIPLICA a \
         largura e 1,5 px viram centenas"
    );
}

/// **Sob painel o anel não é desenhado.**
///
/// O ponteiro ali não é da cena (o `pointer_down` já recusa pela MESMA porta), e
/// uma mira sobre o chrome promete um gesto que o clique não faz.
#[test]
fn the_ring_does_not_float_over_the_panel() {
    let src = source("src/render_loop/mod.rs");
    let at = src
        .find("O ANEL DO PINCEL 3D")
        .expect("o bloco do anel sumiu do render_loop");
    let block = &src[at..(at + 2000).min(src.len())];
    assert!(
        block.contains("over_panel") && block.contains("SCULPT3D_PANEL"),
        "o anel é desenhado sem perguntar se o cursor está sobre o painel"
    );
}

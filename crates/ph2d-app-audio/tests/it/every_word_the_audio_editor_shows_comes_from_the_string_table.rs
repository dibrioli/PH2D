//! ⭐⭐⭐ **NENHUMA PALAVRA DO RACK NEM DA PONTE DO AUDIO EDITOR É ESCRITA NO FONTE** — o HR-15 na
//! crate de FAMÍLIA do áudio.
//!
//! ⚠️ **Porque este gate mora aqui e não no painel** (a forma do §37.3 do handoff de 14/09): o
//! painel `ph2d-panel-audio-editor` pinta os nomes dos efeitos, dos parâmetros e dos presets, e a
//! régua dele lê o `src/` DELE — o texto morava nesta crate, e ninguém o via. Medido em 2026-09-16:
//! **403** literais só em `fx_presets.rs` · `fx_param_specs.rs` · `fx_params_table.rs`.
//!
//! ⭐ A identidade saiu do texto: cada efeito tem um `FxKind::id` estável (o que um preset grava) e
//! cada parâmetro uma `const` de chave (`fx_param_keys.rs`). Os nomes que os ficheiros `v1` gravavam
//! ficam legíveis como PADRÕES de `match` (`fx_presets::legacy_alias`), que a régua isenta.

use ph2d_label_census::gate::{self, Excecao};

const TABLE: &str = "crates/ph2d-i18n/src/audio_fx.rs";

/// ⭐ As excepções, **com o mecanismo** — tudo o que sobra é consola ou formato de ficheiro.
const NOT_LANGUAGE: &[Excecao] = &[
    (
        "fx_presets.rs",
        "# PH2D audio chain v2",
        "o CABEÇALHO do formato de ficheiro dos presets do utilizador — uma assinatura que o leitor \
         reconhece, igual em toda língua",
    ),
    (
        "editor/export.rs",
        ", smpl loop",
        "sufixo de uma linha de CONSOLA (`println!` do export de WAV), nunca pintado",
    ),
    (
        "editor/fx_rack.rs",
        "ADR-0120 warm-up (fills a scratch: one copy, twice per selection)",
        "diagnóstico de CONSOLA do preview do rack (`println!` do custo por quadro)",
    ),
    (
        "editor/fx_rack.rs",
        "ADR-0120 (region rewrite) -- the steady state of a drag",
        "diagnóstico de CONSOLA do preview do rack (`println!` do custo por quadro)",
    ),
    (
        "editor/fx_rack.rs",
        "full render (whole-clip copy) -- the pre-ADR-0120 path",
        "diagnóstico de CONSOLA do preview do rack (`println!` do custo por quadro)",
    ),
    (
        "editor/fx_rack.rs",
        "  <-- OVER BUDGET (this is the stutter)",
        "diagnóstico de CONSOLA do preview do rack — o veredito do orçamento de um quadro",
    ),
    (
        "editor/knob_smoke.rs",
        "SLOW forced -- the pre-ADR-0120 whole-clip render",
        "instrução de CONSOLA da cena de smoke do botão (`PH2D_AUDIO_KNOB_SMOKE`), para quem a corre",
    ),
    (
        "editor/knob_smoke.rs",
        "A/B:  now re-run WITHOUT PH2D_AUDIO_SLOW_PREVIEW to get the fast path back.",
        "instrução de CONSOLA da cena de smoke do botão — o comando para o lado A/B",
    ),
    (
        "editor/knob_smoke.rs",
        "ADR-0120 -- the region rewrite (this is the shipping path)",
        "instrução de CONSOLA da cena de smoke do botão (`PH2D_AUDIO_KNOB_SMOKE`), para quem a corre",
    ),
    (
        "editor/knob_smoke.rs",
        "A/B:  re-run with PH2D_AUDIO_SLOW_PREVIEW=1 to force the OLD whole-clip render.",
        "instrução de CONSOLA da cena de smoke do botão — o comando para o lado A/B",
    ),
    (
        "editor/ml_smoke.rs",
        "will flash past (that is the product being fast, not the bar being broken)",
        "instrução de CONSOLA da cena de smoke do denoise (`PH2D_AUDIO_ML_SMOKE`), para quem a corre",
    ),
    (
        "editor/ml_smoke.rs",
        "should be plainly visible",
        "instrução de CONSOLA da cena de smoke do denoise (`PH2D_AUDIO_ML_SMOKE`), para quem a corre",
    ),
];

#[test]
fn every_word_the_audio_editor_shows_comes_from_the_string_table() {
    let (src, _) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let intrusos = gate::intrusos(&src, NOT_LANGUAGE);
    assert!(
        intrusos.is_empty(),
        "estes textos com cara de língua estão escritos no fonte da ponte do áudio (HR-15):\n  {}\n\n\
         A cura é uma chave `audio.fx.*` / `audio.editor.*` em `{TABLE}` e um `tr(\"…\")` no sítio (uma \
         frase com peças do código: `tr_with`). Um NOME que é também identidade (um efeito, um \
         parâmetro) ganha id estável e a chave ao lado — ver o cabeçalho deste ficheiro.",
        intrusos.join("\n  ")
    );
}

#[test]
fn every_named_exception_still_shelters_a_real_literal() {
    let (src, _) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let mortas = gate::excecoes_mortas(&src, NOT_LANGUAGE);
    assert!(
        mortas.is_empty(),
        "excepções mortas:\n  {}",
        mortas.join("\n  ")
    );
}

#[test]
fn every_key_of_the_audio_bridge_exists_on_both_sides() {
    let (_, repo) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    // ⛔ Controlo de vacuidade: o vocabulário medido na migração (42 + 44 + 23 + 7 no rack, 14 na
    //    ponte), menos folga para encolher.
    for (prefixo, piso) in [("audio.fx.", 100), ("audio.editor.", 10)] {
        let c = gate::chaves(&repo, prefixo, &[TABLE]);
        assert!(
            c.declaradas >= piso && c.usadas >= piso,
            "`{prefixo}`: o censo achou {} declaradas e {} usadas — está a ler o sítio errado",
            c.declaradas,
            c.usadas
        );
        assert!(
            c.sem_traducao.is_empty(),
            "usadas e NÃO declaradas em `{TABLE}` — o `tr` pinta o identificador cru:\n  {}",
            c.sem_traducao.join("\n  ")
        );
        assert!(
            c.orfas.is_empty(),
            "declaradas e ninguém as usa — apague-as:\n  {:?}",
            c.orfas
        );
    }
}

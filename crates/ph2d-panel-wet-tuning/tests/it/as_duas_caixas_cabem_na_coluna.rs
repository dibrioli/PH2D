//! ⭐⭐⭐ **AS DUAS CAIXAS DESTE PAINEL CABEM NA COLUNA DELAS.**
//!
//! ⛔⛔ Até 2026-09-16 nenhuma declarava a secção a que pertence, logo caíam na **metade cega** da
//! linha — e são os dois nomes mais longos de toda a varredura de caixas de marcar do app:
//!
//! | nome | mede | coluna cega a `220` / `245` / `273,3` |
//! |---|---|---|
//! | `Pigment mixing (K-M)` | `123,9` | `84,0` ⛔ · `96,5` ⛔ · `110,6` ⛔ |
//! | `Glaze layering (K-M)` | `114,2` | idem ⛔ |
//!
//! ⇒ os dois saíam cortados **em todo o curso útil do dock**. Com a secção declarada, a coluna
//! cresce até o mais largo dos dois e o corte desaparece de `245` para cima.
//!
//! ⚠️ **No mínimo do dock (`220`) o corte fica**, e é o TECTO: a coluna não pode passar
//! `usable − vão − `[`NUMBER_INPUT_MIN_W_PX`](ph2d_editor_core::widget::NUMBER_INPUT_MIN_W_PX), e
//! esse piso é uma ordem do dono de 2026-05-24.
//!
//! ⚠️ **As linhas de NÚMERO deste painel não têm coluna de nome nenhuma** — elas são a *caixa
//! única* (o rótulo vive DENTRO da barra, spec §2), logo alargar a coluna das caixas de marcar não
//! pode desalinhá-las de nada. *É isso que torna esta cura local e segura aqui, e NÃO no painel de
//! vetor, onde as linhas de número partilham a coluna.*

use ph2d_text::TextSystem;
use ph2d_tokens::TypeToken;

/// Os dois nomes, pelas chaves que o painel pinta.
const CAIXAS: &[&str] = &["panel.wet_tuning.km_mixing", "panel.wet_tuning.km_glaze"];

/// ⏳ **Quantos elidem, por largura de painel — e só ENCOLHE.**
const ELIDEM_POR_LARGURA: &[(f32, usize)] =
    &[(220.0, 2), (245.0, 1), (273.3, 0), (304.0, 0), (720.0, 0)];

/// A linha de uma caixa deste painel, à largura do painel.
fn linha(painel: f32) -> f32 {
    painel - 2.0 * ph2d_tokens::PANEL_HEAD_PAD_PX
}

fn elidem(ts: &mut TextSystem, painel: f32, declarada: bool) -> usize {
    let fonte = TypeToken::Sm.px();
    let rotulos: Vec<&str> = CAIXAS.iter().map(|k| ph2d_i18n::tr(k)).collect();
    let quer = declarada.then(|| {
        rotulos
            .iter()
            .map(|t| ts.prefix_width(t, fonte))
            .fold(0.0_f32, f32::max)
    });
    let col = ph2d_editor_core::widget::property_label_col_w_for(
        0.0,
        linha(painel),
        quer,
        Some(ph2d_editor_core::widget::NUMBER_INPUT_MIN_W_PX),
    );
    rotulos
        .iter()
        .filter(|t| ts.prefix_width(t, fonte) > col)
        .count()
}

#[test]
fn as_duas_caixas_cabem_na_coluna() {
    let mut ts = TextSystem::new();
    for (painel, tecto) in ELIDEM_POR_LARGURA {
        let n = elidem(&mut ts, *painel, true);
        assert!(
            n <= *tecto,
            "painel {painel}: {n} elidem, tecto {tecto} — subiu"
        );
        // A metade de OBSOLESCÊNCIA (`CLAUDE.md` §5.0): se melhorou, o número desce AQUI.
        assert!(
            n == *tecto,
            "painel {painel}: elidem {n} e a tabela diz {tecto} — aperte"
        );
    }
}

/// ⭐ **O CONTROLO** — sem ele, apagar o `.seccao(…)` do pintor deixaria o gate de cima verde.
#[test]
fn e_a_metade_cega_cortava_os_dois() {
    let mut ts = TextSystem::new();
    assert_eq!(
        elidem(&mut ts, 273.3, false),
        2,
        "a metade cega devia cortar os DOIS na largura do dono — se já não corta, o mecanismo \
         mudou e a tabela deste ficheiro está a descrever outro painel"
    );
    assert_eq!(elidem(&mut ts, 273.3, true), 0);
}

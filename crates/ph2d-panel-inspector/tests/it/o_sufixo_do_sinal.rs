//! ⭐ GEF02 onda 1: a caixa de nome de sinal diz quem está na outra ponta (`1 listens` ×
//! `nobody listens`), e cada caixa de sinal do censo de 09/11 está na tabela do sufixo.

use crate::o_cartao_da_fisica::{cena, pinta, sai};
use crate::seam_physics::with_body;
use ph2d_editor_core::widget::card_censo;
use ph2d_panel_inspector::signal_hint::{CAIXAS, set_signal_census};
use ph2d_signal_ends::{Role, SignalCensus};

fn textos_com(signal: &str, censo: SignalCensus) -> Vec<String> {
    let mut info = with_body();
    info.signal = signal.into();
    let (mut host, mut st) = cena(info, [1.0, 1.0]);
    set_signal_census(censo);
    let (_, c) = card_censo::grava(|| pinta(&mut host, &mut st));
    sai();
    set_signal_census(SignalCensus::default());
    c.pecas.into_iter().map(|p| p.nome).collect()
}

#[test]
fn o_on_hit_sem_ouvinte_diz_nobody_listens_e_com_um_diz_1_listens() {
    let mut so_eu = SignalCensus::default();
    so_eu.add("coin_taken", Role::Emit);
    let t = textos_com("coin_taken", so_eu.clone());
    assert!(t.iter().any(|s| s == "nobody listens"), "{t:?}");
    assert!(!t.iter().any(|s| s == "1 listens"), "{t:?}");

    let mut com_ouvinte = so_eu;
    com_ouvinte.add("coin_taken", Role::Listen);
    let t = textos_com("coin_taken", com_ouvinte);
    assert!(t.iter().any(|s| s == "1 listens"), "{t:?}");
    assert!(!t.iter().any(|s| s == "nobody listens"), "{t:?}");

    // Vazio é mudo de propósito: nenhum sufixo.
    let t = textos_com("", SignalCensus::default());
    assert!(!t.iter().any(|s| s.contains("listen")), "{t:?}");
}

/// As 31 caixas do censo de 09/11 (`f3_corpo_e_gatilho_plano.md` §Onda 1).
const CENSO_0911: &[&str] = &[
    "INSP_ACTION_ON",
    "INSP_WEAPON_ON_SIGNAL",
    "INSP_WEAPON_RELOAD_ON",
    "INSP_FACTORY_ON_SIGNAL",
    "INSP_SM_TRANS_ON",
    "INSP_EMITTER_ON",
    "INSP_TRIGGER_SIGNAL",
    "INSP_WATCH_SIGNAL",
    "INSP_TIMER_SIGNAL",
    "INSP_WEAPON_ON_FIRE",
    "INSP_WEAPON_ON_EMPTY",
    "INSP_WEAPON_ON_RELOADED",
    "INSP_FACTORY_ON_SPAWNED",
    "INSP_FACTORY_ON_EXHAUSTED",
    "INSP_LIFE_ON_DEATH",
    "INSP_VIDA_ON_DAMAGE",
    "INSP_VIDA_ON_HEAL",
    "INSP_VIDA_ON_DEATH",
    "INSP_NAV_ON_ARRIVED",
    "INSP_NAV_ON_NO_PATH",
    "INSP_NAV_ON_STUCK",
    "INSP_NAV_LINK_ON_CROSSED",
    "INSP_RAY_ON_ENTER",
    "INSP_RAY_ON_EXIT",
    "INSP_PHYS_SIGNAL",
    "INSP_PHYS_SIGNAL_LEAVE",
    "INSP_SM_STATE_ON_ENTER",
    "INSP_SM_STATE_ON_EXIT",
    "INSP_ANIM_SIGNAL_FINISH",
    "INSP_ANIM_SIGNAL_LOOP",
    "INSP_ANIM_MARKER_SIGNAL",
];

/// O código de um ficheiro sem a prosa (`//` até ao fim da linha).
fn codigo(s: &str) -> String {
    s.lines()
        .map(|l| l.split("//").next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Os ids `INSP_*` inteiros que aparecem em `code` (nunca um prefixo de outro).
fn ids_em(code: &str) -> std::collections::BTreeSet<String> {
    code.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .filter(|w| w.starts_with("INSP_"))
        .map(str::to_owned)
        .collect()
}

#[test]
fn cada_caixa_de_sinal_tem_o_seu_sufixo() {
    let raiz = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let tabela = ids_em(&codigo(
        &std::fs::read_to_string(raiz.join("signal_hint.rs")).unwrap(),
    ));
    // Controlo positivo: a lista tem 31, a tabela tem 31 linhas e acha os 31.
    assert_eq!(CENSO_0911.len(), 31);
    assert_eq!(CAIXAS.len(), 31);
    let esperado: std::collections::BTreeSet<String> =
        CENSO_0911.iter().map(|s| (*s).to_owned()).collect();
    assert_eq!(tabela, esperado, "a tabela do sufixo diverge do censo");
    // Cada id da tabela é PINTADO por uma secção (o código das secções o nomeia).
    let mut nas_seccoes = std::collections::BTreeSet::new();
    for e in std::fs::read_dir(raiz.join("sections")).unwrap() {
        let p = e.unwrap().path();
        if p.extension().is_some_and(|x| x == "rs") {
            nas_seccoes.extend(ids_em(&codigo(&std::fs::read_to_string(p).unwrap())));
        }
    }
    let achados: Vec<_> = esperado
        .iter()
        .filter(|i| nas_seccoes.contains(*i))
        .collect();
    assert_eq!(
        achados.len(),
        31,
        "ids sem secção: {:?}",
        esperado.difference(&nas_seccoes).collect::<Vec<_>>()
    );
}

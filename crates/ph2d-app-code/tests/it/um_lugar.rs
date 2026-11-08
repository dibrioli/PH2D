//! ⭐ **Onda A — um problema, um lugar** (`docs/CodeEditor/13_plano_bancada.md` §4.1–2): o erro do
//! Play vai aos Problems, agrupado com os objectos e a contagem; o do texto na MESMA linha funde-se
//! nele; o contador da barra conta a lista fundida; o `print` leva quem e quando.

use crate::Mesa;
use ph2d_ecs::{Name, StableId, Transform};
use ph2d_editor_core::doc_surface::Key;
use ph2d_host::Modifiers;
use ph2d_panel_code::{BandTab, Severity};
use ph2d_script::{LuauScript, ScriptId};

/// `names` objectos com nome a correr o script `id`.
fn com_nomes(mut m: Mesa, id: ScriptId, names: &[&str]) -> Mesa {
    for (k, n) in names.iter().enumerate() {
        m.sim.world_mut().spawn((
            Transform::IDENTITY,
            StableId(k as u64 + 1 + id.0 * 100),
            LuauScript::library(id),
            Name::new(*n),
        ));
    }
    m.com_cena(id, 0)
}

const NIL: &str = "function update(self, dt)\n  local x = nil\n  local y = x * 2\nend\n";

/// (t1) Três objectos com o MESMO erro na MESMA linha → UMA linha nos Problems com os três nomes e
/// «×3»; o console não tem linha de erro nenhuma; a faixa abre nos Problems.
#[test]
fn tres_objectos_o_mesmo_erro_uma_linha_com_os_tres_nomes() {
    let (m, ids) = Mesa::new(&[("nil.luau", NIL)]);
    let id = ids[0];
    let mut m = com_nomes(m, id, &["Circle", "Star", "Hexagon"]);
    m.pede(id);
    m.cena();
    m.quadro();
    let ps = m.docs().problems_of(id);
    assert_eq!(ps.len(), 1, "{ps:?}");
    let p = &ps[0];
    let shown = p.explanation.as_ref().map_or(&p.message, |e| &e.title);
    assert_eq!(p.line, 2, "{ps:?}");
    assert_eq!(p.severity, Severity::Error);
    assert!(
        shown.ends_with("— on Circle, Star, Hexagon · ×3"),
        "{shown}"
    );
    assert!(
        m.docs().console_texts(id).is_empty(),
        "nenhum erro no console"
    );
    assert_eq!(m.docs().band_tab_of(id), BandTab::Problems);
    // Controlo: o explicador do Luau traduziu a frase crua (não é «attempt to perform…»).
    assert!(p.explanation.is_some(), "{p:?}");
}

const TYPO: &str = "ph2d.property(\"amplitude\", 3)\n\nfunction update(self, dt)\n  local y = self.ampiltude * 2\nend\n";

/// (t2) O aviso do texto e o erro do Play na MESMA linha → UMA linha: o texto do aviso (a causa) e o
/// sufixo do Play; o tom é erro. (t3) O contador da barra conta essa lista: 1 erro, 0 avisos.
#[test]
fn o_aviso_do_texto_e_o_erro_do_play_na_mesma_linha_sao_uma() {
    let (m, ids) = Mesa::new(&[("typo.luau", TYPO)]);
    let id = ids[0];
    let mut m = com_nomes(m, id, &["Circle", "Star"]);
    m.pede(id);
    crate::completar::quadro_em(&mut m, crate::completar::T0);
    let antes = m.docs().problems_of(id);
    assert_eq!(
        antes.len(),
        1,
        "controlo: o aviso do texto existe: {antes:?}"
    );
    assert_eq!(antes[0].line, 3);
    m.cena();
    crate::completar::quadro_em(&mut m, crate::completar::T0 + 1);
    let ps = m.docs().problems_of(id);
    assert_eq!(ps.len(), 1, "{ps:?}");
    let p = &ps[0];
    let shown = p.explanation.as_ref().map_or(&p.message, |e| &e.title);
    let causa = antes[0]
        .explanation
        .as_ref()
        .map_or(&antes[0].message, |e| &e.title);
    assert!(shown.starts_with(causa.as_str()), "{shown} ⊄ {causa}");
    assert!(shown.ends_with("— on Circle, Star · ×2 in Play"), "{shown}");
    assert_eq!(p.severity, Severity::Error);
    m.pinta(id);
    assert_eq!(
        m.docs().problems_counter_label_of(id).as_deref(),
        Some("Errors: 1, Warnings: 0"),
        "um contador só"
    );
    let banda = m.docs().band_of(id).unwrap();
    assert_eq!(banda.tabs[0].0, "Console");
    assert_eq!(banda.tabs[1].0, "Problems (1)");
}

/// (t4) O `print` de um objecto leva o nome dele e a hora do jogo (do passo em que falou).
#[test]
fn o_print_leva_o_objecto_e_a_hora_do_jogo() {
    const COINS: &str = "function update(self, dt)\n  print(\"coins: 3\")\nend\n";
    let (m, ids) = Mesa::new(&[("coins.luau", COINS)]);
    let id = ids[0];
    let mut m = com_nomes(m, id, &["Hero"]);
    m.pede(id);
    for _ in 0..61 {
        m.cena();
    }
    m.quadro();
    let textos = m.docs().console_texts(id);
    let tags = m.docs().console_tags(id);
    assert_eq!(textos.len(), 61);
    assert_eq!(textos[0], "coins: 3");
    assert_eq!(tags[0].as_deref(), Some("Hero · 0:00"));
    assert_eq!(
        tags[60].as_deref(),
        Some("Hero · 0:01"),
        "o 61.º passo é 1 s de jogo"
    );
}

/// Uma recarga bem sucedida limpa os problemas do Play do script.
#[test]
fn recarregar_bem_limpa_os_problemas_do_play() {
    let (m, ids) = Mesa::new(&[("nil.luau", NIL)]);
    let id = ids[0];
    let mut m = com_nomes(m, id, &["Circle"]);
    m.pede(id);
    m.cena();
    m.quadro();
    assert_eq!(m.docs().problems_of(id).len(), 1, "controlo");
    m.host.as_mut().unwrap().request_reload(id);
    m.cena();
    m.quadro();
    let ps = m.docs().problems_of(id);
    // O mesmo texto recarrega bem; a linha do Play saiu — e o objecto, renascido, falha outra vez
    // no passo seguinte (por isso pode voltar com ×1).
    assert!(
        ps.iter().all(|p| {
            let s = p.explanation.as_ref().map_or(&p.message, |e| &e.title);
            !s.contains("×")
        }),
        "{ps:?}"
    );
}

/// ⭐ Parar o jogo (o rebobinar por onde o Stop passa) limpa os problemas do Play; o controlo: o
/// rebobinar com o relógio parado (o de cada quadro parado) não os apaga.
#[test]
fn parar_o_jogo_limpa_os_problemas_do_play() {
    let (m, ids) = Mesa::new(&[("nil.luau", NIL)]);
    let id = ids[0];
    let mut m = com_nomes(m, id, &["Circle"]);
    m.pede(id);
    m.cena();
    m.quadro();
    assert_eq!(
        m.docs().problems_of(id).len(),
        1,
        "controlo: o erro do Play"
    );
    m.host.as_mut().unwrap().scene_rewind();
    m.quadro();
    assert!(m.docs().problems_of(id).is_empty(), "o Stop limpa");
    m.cena();
    m.quadro();
    assert_eq!(
        m.docs().problems_of(id).len(),
        1,
        "a nova corrida falha de novo"
    );
    let antes = m.host.as_ref().unwrap().play_run();
    m.host.as_mut().unwrap().scene_rewind();
    m.host.as_mut().unwrap().scene_rewind();
    assert_eq!(
        m.host.as_ref().unwrap().play_run(),
        antes + 1,
        "o 2.º rebobinar, com o relógio já parado, não conta outra corrida"
    );
}

/// O texto que a lista mostra de cada problema.
fn mostrados(m: &mut Mesa, id: ScriptId) -> Vec<(usize, String)> {
    m.docs()
        .problems_of(id)
        .iter()
        .map(|p| {
            let s = p.explanation.as_ref().map_or(&p.message, |e| &e.title);
            (p.line, s.clone())
        })
        .collect()
}

/// O texto TYPO a correr em dois objectos: o aviso do texto e o erro do Play fundidos na linha 3.
fn typo_no_play() -> (Mesa, ScriptId) {
    let (m, ids) = Mesa::new(&[("typo.luau", TYPO)]);
    let id = ids[0];
    let mut m = com_nomes(m, id, &["Circle", "Star"]);
    m.pede(id);
    crate::completar::quadro_em(&mut m, crate::completar::T0);
    m.cena();
    crate::completar::quadro_em(&mut m, crate::completar::T0 + 1);
    let ps = mostrados(&mut m, id);
    assert_eq!(ps.len(), 1, "controlo: {ps:?}");
    assert_eq!(ps[0].0, 3);
    assert!(ps[0].1.ends_with("in Play"), "controlo: {ps:?}");
    assert_eq!(m.docs().cursor_of(id), Some((0, 0)), "controlo: o cursor");
    (m, id)
}

/// ⭐ (dono 08/11, «os erros deveriam sumir ao consertar e não quando salvar») O erro do Play
/// pertence ao TEXTO que correu: consertar a linha (sem salvar) tira-o com o aviso do texto; o
/// Ctrl+Z que devolve a linha devolve-o (é derivado, não apagado).
#[test]
fn o_erro_do_play_some_ao_consertar_a_linha_e_volta_ao_desfazer() {
    let (mut m, id) = typo_no_play();
    for _ in 0..3 {
        m.tecla(id, Key::Down, Modifiers::default(), None);
    }
    m.tecla(id, Key::End, Modifiers::default(), None);
    for _ in 0.." * 2".len() + "ampiltude".len() {
        m.tecla(id, Key::Backspace, Modifiers::default(), None);
    }
    m.escreve(id, "amplitude * 2");
    assert_ne!(m.docs().text_of(id).as_deref(), Some(TYPO), "editou");
    assert!(
        m.docs().text_of(id).unwrap().contains("self.amplitude * 2"),
        "consertou"
    );
    crate::completar::quadro_em(&mut m, crate::completar::T0 + 10_000_000_000);
    let ps = mostrados(&mut m, id);
    assert!(ps.is_empty(), "nem o aviso nem o Play: {ps:?}");
    m.pinta(id);
    assert_eq!(
        m.docs().problems_counter_label_of(id).as_deref(),
        Some("Errors: 0, Warnings: 0"),
        "o contador segue"
    );
    for _ in 0..64 {
        if m.docs().text_of(id).as_deref() == Some(TYPO) {
            break;
        }
        m.tecla(id, Key::Char('z'), crate::ctrl(), Some("z"));
    }
    assert_eq!(m.docs().text_of(id).as_deref(), Some(TYPO), "desfez");
    // O aviso do texto volta quando o texto pára (a pausa do lint), como a qualquer edição.
    for k in 2..6 {
        crate::completar::quadro_em(&mut m, crate::completar::T0 + k * 10_000_000_000);
    }
    let ps = mostrados(&mut m, id);
    assert_eq!(ps.len(), 1, "{ps:?}");
    assert_eq!(ps[0].0, 3);
    assert!(
        ps[0].1.ends_with("— on Circle, Star · ×2 in Play"),
        "{ps:?}"
    );
}

/// ⭐ Uma linha nova ACIMA (sem salvar) leva o erro do Play com o texto: segue para a linha 4,
/// fundido com o aviso do texto que também desceu.
#[test]
fn uma_linha_acima_leva_o_erro_do_play_para_baixo() {
    let (mut m, id) = typo_no_play();
    m.escreve(id, "\n");
    assert_eq!(
        m.docs().text_of(id).as_deref(),
        Some(format!("\n{TYPO}").as_str()),
        "controlo: só a linha em branco no topo"
    );
    crate::completar::quadro_em(&mut m, crate::completar::T0 + 10_000_000_000);
    let ps = mostrados(&mut m, id);
    assert_eq!(ps.len(), 1, "{ps:?}");
    assert_eq!(ps[0].0, 4, "seguiu o texto: {ps:?}");
    assert!(
        ps[0].1.ends_with("— on Circle, Star · ×2 in Play"),
        "{ps:?}"
    );
}

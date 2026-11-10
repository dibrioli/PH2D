//! Onda B (`docs/CodeEditor/13_plano_bancada.md` §4, itens 8 e 10) pela porta do produto: a célula
//! da grelha escrita vira a edição do Inspector no bus (o desfazer é o do projecto), e o rato parado
//! num nome cede a área da aba à cena com o objecto pedido em `ScenePeek`.

use crate::definicao::clica_em;
use crate::{Mesa, centro, ctrl};
use ph2d_app_code::smoke_drive::{Conductor, drive};
use ph2d_code_bancada::ScenePeek;
use ph2d_code_smoke::STEPS_13;
use ph2d_ecs::{Name, StableId, Transform};
use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
use ph2d_editor_core::doc_surface::{DocumentSurface, Key};
use ph2d_editor_core::script_edits::ScriptFieldEdit;
use ph2d_host::Modifiers;
use ph2d_panel_code::bancada::{Aim, PEEK_PAUSE_NS};
use ph2d_script::{LuauScript, ScriptId, ScriptValue};

const TEXTO: &str = "ph2d.property(\"speed\", 220, { min = 0, max = 1000 })\n\
                     function on_update(self, dt)\n\
                     \x20 print(self.speed)\n\
                     end\n";

/// A aba aberta e pintada, com Ana e Bia a correr o script (postas fora da ordem da `StableId`).
fn aberta() -> (Mesa, ScriptId, [u64; 2]) {
    let (m, ids) = Mesa::new(&[("a.luau", TEXTO)]);
    let id = ids[0];
    let mut m = m.com_cena(id, 0);
    let mut poe = |sid: u64, nome: &str| {
        let lib = LuauScript::library(id);
        let w = m.sim.world_mut();
        w.spawn((Transform::IDENTITY, StableId(sid), Name(nome.into()), lib))
            .id()
            .to_bits()
    };
    let bia = poe(2, "Bia");
    let ana = poe(1, "Ana");
    m.pede(id);
    m.quadro();
    m.pinta(id);
    (m, id, [ana, bia])
}

fn byte_de(needle: &str) -> usize {
    TEXTO.find(needle).expect("no texto")
}

/// ⭐ A célula de Ana escrita («300» + Enter) é UMA `InspectorComponentEdit` no bus, a mesma do
/// cartão Script; enquanto a grelha tem o teclado, o Ctrl+Z é do projecto, e um clique no texto
/// devolve-o ao código.
#[test]
fn a_celula_escrita_vai_ao_bus_como_a_edicao_do_inspector() {
    let (mut m, id, [ana, _]) = aberta();
    assert!(
        !m.docs().undo_is_the_projects(),
        "a aba abre com o desfazer do texto"
    );
    let r = m
        .docs()
        .bench
        .target(Aim::Cell(0, 0))
        .expect("a célula de Ana pintada");
    let (x, y) = centro(r);
    m.clica(id, x, y, Modifiers::default());
    assert!(
        m.docs().undo_is_the_projects(),
        "o clique na grelha dá-o ao projecto"
    );
    m.tecla(id, Key::Char('a'), ctrl(), None);
    m.escreve(id, "300");
    m.tecla(id, Key::Enter, Modifiers::default(), Some("\r"));
    m.quadro();

    let edicoes: Vec<EditorAction> = m
        .hero
        .bus
        .iter()
        .filter(|a| matches!(a, EditorAction::InspectorComponentEdit { .. }))
        .cloned()
        .collect();
    assert_eq!(
        edicoes,
        [EditorAction::InspectorComponentEdit {
            entity_bits: ana,
            edit: ComponentEdit::Script(ScriptFieldEdit::SetNumber("speed".into(), 300.0)),
        }]
    );
    assert!(
        m.docs().undo_is_the_projects(),
        "depois do Enter, a grelha fica com ele"
    );
    // Smoke do dono 09/11 (passo 5): a TECLA também — o Ctrl+Z não é da aba (segue para a shell,
    // que desfaz o projecto) e o texto não muda.
    let antes = m.docs().text_of(id);
    let z = |m: &mut Mesa| {
        let mut ts = ph2d_text::TextSystem::without_system_fonts();
        let input = ph2d_editor_core::doc_surface::KeyInput {
            key: Key::Char('z'),
            mods: ctrl(),
            pressed: true,
            text: None,
            paste: None,
        };
        m.docs().key(id.0, &mut ts, input).consumed
    };
    assert!(!z(&mut m), "com a grelha em foco o Ctrl+Z é do projecto");
    assert_eq!(m.docs().text_of(id), antes, "o texto não se desfez");
    clica_em(&mut m, id, byte_de("print") + 2, Modifiers::default());
    assert!(
        !m.docs().undo_is_the_projects(),
        "um clique no texto devolve o desfazer ao código"
    );
    assert!(z(&mut m), "e o Ctrl+Z volta a ser da aba (o texto)");
}

/// ⭐ O rato parado no nome de Bia a pausa do balão: a aba cede à cena e pede Bia contornada na cor
/// dela; antes da pausa nada; o rato fora do nome devolve a aba.
#[test]
fn o_rato_parado_no_nome_espia_o_objecto_na_cena() {
    let (mut m, id, [_, bia]) = aberta();
    m.docs().set_clock(1_000, false);
    let r = m
        .docs()
        .bench
        .target(Aim::Holder(1))
        .expect("o nome de Bia pintado");
    let (x, y) = centro(r);
    m.rato(id, x, y);
    assert_eq!(m.docs().bench.aim(), Aim::Holder(1));
    m.docs().bench.tick(1_000 + PEEK_PAUSE_NS - 1);
    assert_eq!(m.docs().bench.peeking(), None, "antes da pausa, a aba fica");
    m.docs().bench.tick(1_000 + PEEK_PAUSE_NS);
    m.quadro();
    assert_eq!(
        *m.sim.world().resource::<ScenePeek>(),
        ScenePeek(Some((bia, 1)))
    );
    assert!(m.docs().yields_to_scene());

    let (fx, fy) = m
        .docs()
        .glyph_center(id, byte_de("print"))
        .expect("o texto à vista");
    m.rato(id, fx, fy);
    m.quadro();
    assert_eq!(*m.sim.world().resource::<ScenePeek>(), ScenePeek(None));
    assert!(!m.docs().yields_to_scene());
}

/// O `player.luau` da cena 13 (o MESMO ficheiro que o `ph2d_code_scenes::bancada` corre).
const PLAYER: &str =
    include_str!("../../../../docs/CodeEditor/ferramentas/smoke/bancada_player.luau");

/// ⭐ A cena 13, passo 1, pelo roteiro do smoke (`STEPS_13[0]`): o rato na marca ◆ de `speed` abre o
/// balão com Hero 220 · default e Hero 2 180 · changed (o Hero 2 com 180 no Inspector, como a cena).
#[test]
fn o_passo_1_da_cena_13_abre_o_balao_de_speed_com_os_dois_herois() {
    let (m, ids) = Mesa::new(&[("player.luau", PLAYER)]);
    let id = ids[0];
    let mut m = m.com_cena(id, 0);
    for (sid, nome, speed) in [(1, "Hero", None), (2, "Hero 2", Some(180.0))] {
        let mut lib = LuauScript::library(id);
        if let Some(v) = speed {
            lib.own.insert("speed".into(), ScriptValue::Number(v));
        }
        let w = m.sim.world_mut();
        w.spawn((Transform::IDENTITY, StableId(sid), Name(nome.into()), lib));
    }
    m.pede(id);
    let mut c = Conductor::scene(STEPS_13, 1);
    for _ in 0..20 {
        m.quadro();
        m.pinta(id);
        if c.finished() {
            break;
        }
        drive(m.docs(), &mut c);
    }
    assert!(c.finished() && !c.failed(), "o passo 1 chegou ao fim");
    let (_, rows) = m.docs().bench.popover_table().expect("o balão aberto");
    let texto = |h: usize| {
        rows[h]
            .cells
            .iter()
            .map(|c| c.text.as_str())
            .collect::<Vec<_>>()
    };
    assert_eq!(texto(0), ["Hero", "220", "default"]);
    assert_eq!(texto(1), ["Hero 2", "180", "changed"]);
}

/// ⭐ Uma peça da Bancada que não se pinta neste quadro não deixa alvo vivo: a aba de ajuda (que
/// pinta a página no lugar do código) e a aba cujo documento fechou não respondem ao rato nem ao
/// clique onde estava o nome de Bia no quadro de antes.
#[test]
fn sem_grelha_pintada_nenhum_alvo_do_quadro_anterior_responde() {
    for ajuda in [true, false] {
        let (mut m, id, _) = aberta();
        m.docs().set_clock(1_000, false);
        let r = m
            .docs()
            .bench
            .target(Aim::Holder(1))
            .expect("o nome de Bia pintado");
        let (x, y) = centro(r);
        let outra = if ajuda {
            ScriptId(ph2d_app_code::HELP_TAG)
        } else {
            m.docs().close(id.0);
            id
        };
        m.pinta(outra);
        assert_eq!(m.docs().bench.target(Aim::Holder(1)), None, "ajuda={ajuda}");
        m.rato(outra, x, y);
        assert_eq!(m.docs().bench.aim(), Aim::Nothing, "ajuda={ajuda}");
        let mut ts = ph2d_text::TextSystem::without_system_fonts();
        let input = ph2d_editor_core::doc_surface::PointerInput {
            text: &mut ts,
            mods: Modifiers::default(),
            now_ns: 1,
        };
        let (down, primary) = (
            ph2d_host::PointerKind::Down,
            ph2d_host::PointerButton::Primary,
        );
        let _ = m.docs().pointer(outra.0, input, down, primary, x, y);
        assert!(
            m.docs().bench.take_intents().is_empty(),
            "ajuda={ajuda}: o clique não pediu nada"
        );
    }
}

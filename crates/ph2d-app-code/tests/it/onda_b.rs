//! Onda B (`docs/CodeEditor/13_plano_bancada.md` §4, itens 8 e 10) pela porta do produto: a célula
//! da grelha escrita vira a edição do Inspector no bus (o desfazer é o do projecto), e o rato parado
//! num nome cede a área da aba à cena com o objecto pedido em `ScenePeek`.

use crate::definicao::clica_em;
use crate::{Mesa, centro, ctrl};
use ph2d_code_bancada::ScenePeek;
use ph2d_ecs::{Name, StableId, Transform};
use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
use ph2d_editor_core::doc_surface::{DocumentSurface, Key};
use ph2d_editor_core::script_edits::ScriptFieldEdit;
use ph2d_host::Modifiers;
use ph2d_panel_code::bancada::{Aim, PEEK_PAUSE_NS};
use ph2d_script::{LuauScript, ScriptId};

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
    clica_em(&mut m, id, byte_de("print") + 2, Modifiers::default());
    assert!(
        !m.docs().undo_is_the_projects(),
        "um clique no texto devolve o desfazer ao código"
    );
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

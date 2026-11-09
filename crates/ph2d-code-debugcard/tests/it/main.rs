//! O quadro do depurador (plano 14): as linhas da planilha, os caminhos das tabelas aninhadas e o
//! «mudou desde o passo anterior».

use ph2d_code_debugcard::{
    CardState, StepMemory, VarGroup, WhoTag, sheet_rows, shown_value, stop_card,
};
use ph2d_ecs::Entity;
use ph2d_script::{DebugStop, ScriptId, StopFrame, StopKind, ValueKind, Var};
use std::collections::BTreeMap;

fn table(name: &str, fields: Vec<Var>) -> Var {
    Var {
        field_count: u32::try_from(fields.len()).unwrap(),
        fields,
        ..Var::leaf(name, "table", ValueKind::Table)
    }
}

fn stop(t: &str, x: &str) -> DebugStop {
    DebugStop {
        entity: Entity::from_bits(7),
        script: ScriptId(3),
        why: StopKind::Step,
        frame: Some(StopFrame {
            line: 14,
            function: "update".into(),
            locals: vec![
                Var::leaf("dt", "0.017", ValueKind::Number),
                table("self", vec![]),
            ],
            this: vec![
                Var::leaf("t", t, ValueKind::Number),
                table(
                    "pos",
                    vec![
                        Var::leaf("x", x, ValueKind::Number),
                        Var::leaf("y", "0", ValueKind::Number),
                    ],
                ),
                table("trail", vec![Var::leaf("[1]", "1", ValueKind::Number)]),
            ],
            stack: vec![("update".into(), 14)],
        }),
        waiting: vec![Entity::from_bits(8)],
    }
}

fn who(n: &str) -> WhoTag {
    WhoTag {
        name: n.into(),
        color: Some(0),
    }
}

#[test]
fn o_stopped_agrupa_locais_e_self_com_caminhos_estaveis_e_o_self_nao_repete() {
    let c = stop_card(
        &stop("0.133", "1"),
        who("Circle"),
        vec![who("Star")],
        &BTreeMap::new(),
    );
    assert_eq!(c.state, CardState::Paused);
    assert_eq!(c.place(), "line 14 \u{00b7} update()");
    let locals: Vec<&str> = c.locals.iter().map(|r| r.key.as_str()).collect();
    assert_eq!(
        locals,
        ["dt"],
        "o self das locais sai: os campos estão no grupo dele"
    );
    let this: Vec<(&str, u8)> = c.this.iter().map(|r| (r.key.as_str(), r.depth)).collect();
    assert_eq!(
        this,
        [
            ("self.t", 0),
            ("self.pos", 0),
            ("self.pos.x", 1),
            ("self.pos.y", 1),
            ("self.trail", 0),
            ("self.trail[1]", 1),
        ]
    );
    assert_eq!(shown_value(&c.this[1]), "2 fields");
    assert_eq!(c.groups()[1].0, VarGroup::This);
    assert_eq!(c.waiting_text().as_deref(), Some("1 more waiting"));
}

#[test]
fn o_passo_seguinte_acende_o_que_mudou_e_abre_a_tabela_dele() {
    let mut mem = StepMemory::default();
    let owner = (7, 3);
    let first = stop_card(&stop("0.133", "1"), who("Circle"), vec![], &BTreeMap::new());
    assert!(
        mem.observe(owner, first.values()).is_empty(),
        "o 1.º passo não compara"
    );
    // O mesmo passo outra vez (o jogo parado, quadro a quadro): nada muda.
    assert!(mem.observe(owner, first.values()).is_empty());
    let s2 = stop("0.15", "2");
    let now = stop_card(&s2, who("Circle"), vec![], &BTreeMap::new()).values();
    let before = mem.observe(owner, now).clone();
    let c = stop_card(&s2, who("Circle"), vec![], &before);
    let old = |k: &str| {
        c.this
            .iter()
            .find(|r| r.key == k)
            .and_then(|r| r.old.clone())
    };
    assert_eq!(old("self.t").as_deref(), Some("0.133"));
    assert_eq!(old("self.pos.x").as_deref(), Some("1"));
    assert_eq!(old("self.pos.y"), None);
    assert_eq!(
        old("self.pos"),
        None,
        "a tabela não «muda»: as filhas é que mudam"
    );
    let rows = sheet_rows(&c);
    let row = |k: &str| rows.iter().find(|r| r.key == k).expect(k);
    assert!(row("self.pos").default_open, "abre-se sozinha: x mudou");
    assert!(!row("self.trail").default_open, "a outra fica dobrada");
    assert!(row("self.t").changed);
    assert_eq!(row("self.t").cells[2].text, "number \u{00b7} changed");
    assert_eq!(row("#self").group.as_deref(), Some("\u{00b7} 3 fields"));
    // Outro objecto parado: começa de novo.
    assert!(mem.observe((8, 3), first.values()).is_empty());
}

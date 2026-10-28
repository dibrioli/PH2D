//! A FRENTE do cartão (W3) pela costura: o documento muda, um quadro corre como a shell o corre, e
//! a sonda lê o que cada frente PINTOU — o distintivo das datas com a cor do instante, o campo «na
//! frente», as capas metade e inteira, e a altura que a pintura percorre == a que a disposição mediu.

use crate::Mesa;
use ph2d_editor_core::IconId;
use ph2d_kanban_model::{
    Card, Cover, CoverFill, Due, FieldDef, FieldId, FieldKind, FieldOption, FieldValue, FileId,
    FracKey, Hue, KanbanId, KanbanOp, Meta, Moment, OptionId, Record,
};
use ph2d_tokens::ColorToken as T;

const HORA: i64 = 3_600_000;

fn agora() -> i64 {
    let d = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("relógio");
    i64::try_from(d.as_millis()).expect("ms")
}

/// Muda o cartão `titulo` do Kanban `id` no documento.
fn muda(m: &Mesa, id: KanbanId, titulo: &str, f: impl FnOnce(&mut Card)) {
    let shared = m.hero.documents.kanbans_shared();
    let mut set = shared.borrow_mut();
    let mut c = set
        .get(id)
        .and_then(|k| k.cards.values().find(|c| c.title == titulo).cloned())
        .unwrap_or_else(|| panic!("cartão {titulo}"));
    f(&mut c);
    c.meta.version += 1;
    set.apply(id, vec![KanbanOp::put(Record::Card(c))]);
}

fn prazo(at: i64, done: bool) -> Option<Due> {
    Some(Due {
        at: Moment(at),
        done,
        remind_before_min: None,
    })
}

#[test]
fn frente_pinta_datas_com_o_tom_do_instante_campos_capas_e_a_altura_medida() {
    let (mut m, id) = Mesa::cena1();
    m.docs().probe_arm_fronts();
    let now = agora();
    let prioridade = FieldDef {
        id: FieldId(9001),
        name: "Priority".into(),
        kind: FieldKind::Dropdown(vec![FieldOption {
            id: OptionId(1),
            text: "High".into(),
            color: Some(Hue::Orange),
        }]),
        on_front: true,
        pos: FracKey::between(None, None),
        meta: Meta::default(),
    };
    m.hero
        .documents
        .kanbans_shared()
        .borrow_mut()
        .apply(id, vec![KanbanOp::put(Record::Field(prioridade))]);
    muda(&m, id, "Animate shot 12", |c| {
        c.due = prazo(now - HORA, false)
    });
    muda(&m, id, "Design the fox", |c| {
        c.due = prazo(now + 2 * HORA, false)
    });
    muda(&m, id, "Lock the script", |c| {
        c.due = prazo(now - HORA, true)
    });
    muda(&m, id, "Write the logline", |c| {
        c.start = Some(Moment(now - 48 * HORA));
        c.due = prazo(now + 240 * HORA, false);
    });
    muda(&m, id, "Clean up shot 08", |c| {
        c.fields
            .insert(FieldId(9001), FieldValue::Option(OptionId(1)));
    });
    let imagem = FileId([7; 32]);
    muda(&m, id, "Voice casting", |c| {
        c.cover = Some(Cover {
            fill: CoverFill::Image(imagem),
            full: false,
        });
    });
    muda(&m, id, "Pick the aspect ratio", |c| {
        c.cover = Some(Cover {
            fill: CoverFill::Image(imagem),
            full: true,
        });
    });
    muda(&m, id, "Pick the music", |c| {
        c.cover = Some(Cover {
            fill: CoverFill::Color(Hue::Purple),
            full: true,
        });
    });
    m.quadros(2);

    let due = |m: &mut Mesa, t: &str| {
        m.docs()
            .probe_painted_front(t)
            .unwrap_or_else(|| panic!("{t} pintado"))
            .3
            .unwrap_or_else(|| panic!("{t}: distintivo das datas"))
    };
    assert_eq!(
        due(&mut m, "Animate shot 12").1,
        Some(T::KanbanDueOver),
        "atrasado"
    );
    assert_eq!(
        due(&mut m, "Design the fox").1,
        Some(T::KanbanDueSoon),
        "< 24 h"
    );
    assert_eq!(
        due(&mut m, "Lock the script").1,
        Some(T::KanbanDueDone),
        "concluído"
    );
    let (texto, fundo) = due(&mut m, "Write the logline");
    assert!(texto.contains('\u{2013}'), "início → entrega: {texto}");
    assert_eq!(fundo, None, "a dez dias: sem cor");

    let campos = m
        .docs()
        .probe_painted_front("Clean up shot 08")
        .expect("pintado")
        .4;
    assert_eq!(campos, vec!["Priority: High".to_owned()]);
    let icones = m
        .docs()
        .probe_painted_front("Animate shot 12")
        .expect("pintado")
        .5;
    assert_eq!(icones.first(), Some(&IconId::Timer), "a data abre a fila");

    // As capas: metade de imagem = a faixa de 128; inteiras = a altura mínima, só o título.
    let k = m.kanban(id);
    let rect = |m: &mut Mesa, t: &str| {
        let c = k.cards.values().find(|c| c.title == t).expect("cartão").id;
        m.rect_cartao(id, c)
    };
    let sem_capa = rect(&mut m, "Clean up shot 08").h;
    assert!(rect(&mut m, "Voice casting").h > 128.0 && rect(&mut m, "Voice casting").h > sem_capa);
    assert!((rect(&mut m, "Pick the aspect ratio").h - 152.0).abs() < 0.01);
    assert!((rect(&mut m, "Pick the music").h - 152.0).abs() < 0.01);

    // ⭐ A altura medida == a pintada, cartão a cartão (a Review está recolhida: não pinta).
    let mut vistos = 0;
    for c in k.cards.values() {
        let Some(p) = m.docs().probe_painted_front(&c.title) else {
            continue;
        };
        let medida = m.rect_cartao(id, c.id).h;
        assert!(
            (medida - p.0).abs() < 0.01,
            "{}: medida {medida} == pintada {}",
            c.title,
            p.0
        );
        vistos += 1;
    }
    assert!(vistos >= 12, "{vistos} cartões pintados");
}

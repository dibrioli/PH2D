//! ⭐⭐⭐ **As secções HEALTH e DAMAGE são PINTADAS, estão VIVAS sob o dedo, e mostram os números
//! DO OBJECTO** (plano 28, W3).
//!
//! Irmã do [`super::a_seccao_weapon_esta_viva`], e pelo mesmo motivo: *a semente é a única metade de
//! uma secção cujo sujeito é o WIDGET* — os gates da lei e do dreno entram **abaixo** do
//! `WidgetStore`, e nenhum deles vê um campo pintado a mostrar o valor de FÁBRICA.
//!
//! # ⭐ E há uma pergunta que as irmãs não tinham: as LINHAS QUE SOMEM
//!
//! O atraso da regeneração só aparece com regeneração, as quatro do escudo só com escudo, a semente
//! só com esquiva. ⇒ o gate tem as **duas** metades: com tudo ligado todo campo é pintado, e com
//! tudo desligado os condicionais **não** o são — *uma regra de esconder que nunca esconde nada e
//! uma que esconde sempre passam as duas na primeira metade.*

use ph2d_editor_core::vida_edits::{
    InspectorDamageInfo, InspectorHealthInfo, InspectorVidaInfo, VidaAgora,
};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::{InspectorPanel, InspectorState, ids, set_current_inspector_vida};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 3200.0,
};

/// ⭐ Uma vida que **NÃO** é o `Health::default()` em nenhum campo, com os três interruptores de
/// linhas condicionais LIGADOS (regeneração · escudo · esquiva) — *um corpus no NEUTRO de um knob
/// não testa esse knob*, e aqui o neutro de três deles ESCONDE linhas.
fn vida() -> InspectorHealthInfo {
    InspectorHealthInfo {
        max: 120.0,
        start: 90.0,
        invincible_s: 0.75,
        overheal: false,
        regen: 4.0,
        regen_delay_s: 1.5,
        shield_start: 30.0,
        shield_max: 60.0,
        shield_duration_s: 8.0,
        shield_regen: 6.0,
        shield_regen_delay_s: 2.5,
        shield_blocks_excess: false,
        armor_flat: 3.0,
        armor_percent: 0.25,
        dodge: 0.15,
        team: "enemies".into(),
        on_damage: "ai".into(),
        on_heal: "cura".into(),
        on_death: "morreu".into(),
        seed: 42,
        agora: Some(VidaAgora {
            pontos: 70.0,
            escudo: 12.0,
            morta: false,
        }),
    }
}

fn dano() -> InspectorDamageInfo {
    InspectorDamageInfo {
        amount: 15.0,
        team: "player".into(),
        per_second: false,
        ignores_shield: false,
        ignores_armor: false,
        vanish: false,
    }
}

fn info(
    health: Option<InspectorHealthInfo>,
    damage: Option<InspectorDamageInfo>,
) -> InspectorVidaInfo {
    InspectorVidaInfo {
        entity_bits: 0x00AB_1234,
        health,
        damage,
        bar: None,
        has_body: true,
        clock_playing: true,
        selected_count: 1,
    }
}

fn host(i: InspectorVidaInfo) -> (MockPanelHost, InspectorState) {
    let h = MockPanelHost::with_panel::<InspectorPanel>();
    set_current_inspector_vida(Some(i));
    (h, InspectorState::default())
}

/// Os QUINZE números das duas secções, com o valor que a fixtura tem.
const NUMEROS: [(ph2d_a11y::NodeId, f64); 15] = [
    (ids::INSP_VIDA_MAX, 120.0),
    (ids::INSP_VIDA_START, 90.0),
    (ids::INSP_VIDA_INVINCIBLE, 0.75),
    (ids::INSP_VIDA_REGEN, 4.0),
    (ids::INSP_VIDA_REGEN_DELAY, 1.5),
    (ids::INSP_VIDA_SHIELD_START, 30.0),
    (ids::INSP_VIDA_SHIELD_MAX, 60.0),
    (ids::INSP_VIDA_SHIELD_DURATION, 8.0),
    (ids::INSP_VIDA_SHIELD_REGEN, 6.0),
    (ids::INSP_VIDA_SHIELD_REGEN_DELAY, 2.5),
    (ids::INSP_VIDA_ARMOR, 3.0),
    (ids::INSP_VIDA_ARMOR_PCT, 0.25),
    (ids::INSP_VIDA_DODGE, 0.15),
    (ids::INSP_VIDA_SEED, 42.0),
    (ids::INSP_DANO_AMOUNT, 15.0),
];

/// Os CINCO nomes, com o texto que a fixtura tem.
const NOMES: [(ph2d_a11y::NodeId, &str); 5] = [
    (ids::INSP_VIDA_TEAM, "enemies"),
    (ids::INSP_VIDA_ON_DAMAGE, "ai"),
    (ids::INSP_VIDA_ON_HEAL, "cura"),
    (ids::INSP_VIDA_ON_DEATH, "morreu"),
    (ids::INSP_DANO_TEAM, "player"),
];

/// As SEIS caixas.
const CAIXAS: [ph2d_a11y::NodeId; 6] = [
    ids::INSP_VIDA_OVERHEAL,
    ids::INSP_VIDA_SHIELD_BLOCKS,
    ids::INSP_DANO_PER_SECOND,
    ids::INSP_DANO_IGNORES_SHIELD,
    ids::INSP_DANO_IGNORES_ARMOR,
    ids::INSP_DANO_VANISH,
];

/// As linhas que SÓ aparecem com o interruptor delas ligado.
const CONDICIONAIS: [ph2d_a11y::NodeId; 7] = [
    ids::INSP_VIDA_REGEN_DELAY,
    ids::INSP_VIDA_SEED,
    ids::INSP_VIDA_SHIELD_DURATION,
    ids::INSP_VIDA_SHIELD_REGEN,
    ids::INSP_VIDA_SHIELD_REGEN_DELAY,
    ids::INSP_VIDA_SHIELD_BLOCKS,
    ids::INSP_VIDA_OVERHEAL,
];

/// ⭐⭐ **TODO campo das duas secções é pintado com área clicável.**
///
/// **Mutação que deve sangrar:** tirar qualquer linha dos dois pintores.
#[test]
fn todo_campo_das_duas_seccoes_e_pintado() {
    let (mut h, mut st) = host(info(Some(vida()), Some(dano())));
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let ids_pintados = NUMEROS
        .iter()
        .map(|(i, _)| *i)
        .chain(NOMES.iter().map(|(i, _)| *i))
        .chain(CAIXAS);
    for id in ids_pintados {
        let r = rects
            .iter()
            .find(|(n, _)| *n == id)
            .map(|(_, r)| *r)
            .unwrap_or_else(|| panic!("o campo {id:?} nao foi PINTADO com area clicavel"));
        assert!(r.w > 0.0 && r.h > 0.0, "campo {id:?} sem area: {r:?}");
    }
    set_current_inspector_vida(None);
}

/// ⭐⭐⭐ **As linhas condicionais SOMEM com o interruptor delas desligado** — a metade negativa.
///
/// ⚠️ O `max` fica `0`, que é a condição do *Overheal*: sem máximo não há nada acima do qual curar.
///
/// **Mutações que devem sangrar:** tirar qualquer um dos `if` dos pintores.
#[test]
fn as_linhas_condicionais_somem_sem_o_interruptor() {
    let mut v = vida();
    v.regen = 0.0;
    v.dodge = 0.0;
    v.shield_start = 0.0;
    v.shield_max = 0.0;
    v.max = 0.0;
    let (mut h, mut st) = host(info(Some(v), Some(dano())));
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    for id in CONDICIONAIS {
        assert!(
            !rects.iter().any(|(n, _)| *n == id),
            "a linha condicional {id:?} foi pintada com o interruptor dela DESLIGADO — o painel \
             entrega um controlo morto"
        );
    }
    // O CONTROLO: as linhas incondicionais continuam lá, senão a metade negativa passaria com a
    // secção inteira apagada.
    assert!(
        rects.iter().any(|(n, _)| *n == ids::INSP_VIDA_REGEN),
        "a linha da regeneração (que liga as outras) sumiu — a metade negativa mede o NADA"
    );
    set_current_inspector_vida(None);
}

/// ⭐⭐⭐ **TODO campo está VIVO SOB O DEDO** — o gesto REAL, e não um `WidgetEvent` sintético.
///
/// **Mutação que deve sangrar:** tirar qualquer id do `populate_vida`.
#[test]
fn todo_campo_esta_vivo_sob_o_dedo() {
    for id in NUMEROS
        .iter()
        .map(|(i, _)| *i)
        .chain(NOMES.iter().map(|(i, _)| *i))
    {
        let (mut h, mut st) = host(info(Some(vida()), Some(dano())));
        let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
        let r = rects
            .iter()
            .find(|(n, _)| *n == id)
            .map(|(_, r)| *r)
            .unwrap_or_else(|| panic!("{id:?} nao foi pintado"));
        let _ = h.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5);
        assert_eq!(
            h.store().focus_id(),
            Some(id),
            "clicar no meio de {id:?} nao lhe deu o foco — falta o registo no `populate_vida`, e \
             ele esta' MORTO SOB O DEDO"
        );
        set_current_inspector_vida(None);
    }
}

/// ⭐⭐⭐ **OS CAMPOS MOSTRAM O QUE O OBJECTO TEM, e não os valores de fábrica.**
///
/// **Mutação que deve sangrar:** tirar a chamada do `sync_vida` do `sync_sections`.
#[test]
fn os_campos_mostram_o_que_o_objecto_tem() {
    let (mut h, mut st) = host(info(Some(vida()), Some(dano())));
    let _ = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    for (id, esperado) in NUMEROS {
        let lido = h
            .store()
            .number_value(id)
            .unwrap_or_else(|| panic!("o campo {id:?} nem sequer esta' registado"));
        assert!(
            (lido - esperado).abs() < 1.0e-5,
            "o campo {id:?} mostra {lido} e o objecto tem {esperado} — o painel mostra os valores \
             de FABRICA do `populate_vida`"
        );
    }
    for (id, esperado) in NOMES {
        assert_eq!(
            h.store().text(id),
            Some(esperado),
            "o campo de texto {id:?} nao traz o nome que o objecto tem"
        );
    }
    set_current_inspector_vida(None);
}

/// ⭐⭐⭐ **Cada secção só existe para quem TEM o componente dela** (ADR-0166) — e as duas são
/// independentes: um espinho tem dano e não tem vida.
#[test]
fn cada_seccao_so_existe_para_quem_tem_o_componente() {
    let so_dano = info(None, Some(dano()));
    let (mut h, mut st) = host(so_dano);
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    assert!(
        rects.iter().any(|(n, _)| *n == ids::INSP_DANO_AMOUNT),
        "o CONTROLO: a secção DAMAGE tem de ser pintada num espinho"
    );
    assert!(
        !rects.iter().any(|(n, _)| *n == ids::INSP_VIDA_MAX),
        "a secção HEALTH foi pintada num objecto SEM vida"
    );
    set_current_inspector_vida(None);

    let mut h = MockPanelHost::with_panel::<InspectorPanel>();
    let mut st = InspectorState::default();
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    assert!(
        !rects
            .iter()
            .any(|(n, _)| *n == ids::INSP_VIDA_MAX || *n == ids::INSP_DANO_AMOUNT),
        "uma das secções foi pintada sem instantâneo nenhum"
    );
}

/// ⭐⭐⭐ **O que se ESCREVE num campo de texto chega ao BARRAMENTO, e com a variante DELE.**
///
/// **Mutações que devem sangrar:** tirar qualquer braço do `match` do `texto` · mandar sempre a
/// mesma variante.
#[test]
fn escrever_num_nome_chega_ao_barramento_com_a_variante_dele() {
    use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
    use ph2d_editor_core::interaction::WidgetEvent;
    use ph2d_editor_core::vida_edits::VidaFieldEdit as E;

    const ESCRITO: &str = "escrito-pelo-gate";
    let esperado = |id: ph2d_a11y::NodeId| -> E {
        let t = ESCRITO.to_owned();
        match id {
            i if i == ids::INSP_VIDA_TEAM => E::Team(t),
            i if i == ids::INSP_VIDA_ON_DAMAGE => E::OnDamage(t),
            i if i == ids::INSP_VIDA_ON_HEAL => E::OnHeal(t),
            i if i == ids::INSP_VIDA_ON_DEATH => E::OnDeath(t),
            i if i == ids::INSP_DANO_TEAM => E::DamageTeam(t),
            _ => panic!("{id:?} nao esta' na tabela de NOMES deste gate"),
        }
    };
    for (id, _) in NOMES {
        let (mut h, mut st) = host(info(Some(vida()), Some(dano())));
        let _ = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
        h.set_text(id, ESCRITO);
        let _ = h.apply_panel_event::<InspectorPanel>(&mut st, WidgetEvent::TextChanged(id));
        let edits: Vec<_> = h
            .drained_actions()
            .into_iter()
            .filter_map(|a| match a {
                EditorAction::InspectorComponentEdit {
                    edit: ComponentEdit::Vida(e),
                    ..
                } => Some(e),
                _ => None,
            })
            .collect();
        set_current_inspector_vida(None);
        assert_eq!(
            edits,
            vec![esperado(id)],
            "escrever em {id:?} tem de produzir UMA edicao, e a variante dele"
        );
    }
}

/// ⭐⭐⭐ **Uma caixa pede o CONTRÁRIO do que o objecto tem** — lido do instantâneo, nunca do store.
///
/// **Mutação que deve sangrar:** o `caixa` deixar de inverter · trocar dois braços.
#[test]
fn uma_caixa_pede_o_contrario_do_que_o_objecto_tem() {
    use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
    use ph2d_editor_core::interaction::WidgetEvent;
    use ph2d_editor_core::vida_edits::VidaFieldEdit as E;

    let mut v = vida();
    v.overheal = true;
    let mut d = dano();
    d.vanish = true;
    let esperado = [
        (ids::INSP_VIDA_OVERHEAL, E::Overheal(false)),
        (ids::INSP_VIDA_SHIELD_BLOCKS, E::ShieldBlocksExcess(true)),
        (ids::INSP_DANO_PER_SECOND, E::PerSecond(true)),
        (ids::INSP_DANO_IGNORES_SHIELD, E::IgnoresShield(true)),
        (ids::INSP_DANO_IGNORES_ARMOR, E::IgnoresArmor(true)),
        (ids::INSP_DANO_VANISH, E::Vanish(false)),
    ];
    for (id, e) in esperado {
        let (mut h, mut st) = host(info(Some(v.clone()), Some(d.clone())));
        let _ = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
        let _ = h.apply_panel_event::<InspectorPanel>(&mut st, WidgetEvent::Toggled(id));
        let edits: Vec<_> = h
            .drained_actions()
            .into_iter()
            .filter_map(|a| match a {
                EditorAction::InspectorComponentEdit {
                    edit: ComponentEdit::Vida(e),
                    ..
                } => Some(e),
                _ => None,
            })
            .collect();
        set_current_inspector_vida(None);
        assert_eq!(edits, vec![e], "a caixa {id:?} pediu a edição errada");
    }
}

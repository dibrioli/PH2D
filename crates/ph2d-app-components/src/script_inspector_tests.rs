//! O instantâneo e o dreno da secção SCRIPT, sobre a VM real e ficheiros reais.

use std::sync::atomic::{AtomicU32, Ordering};

use ph2d_ecs::{SimWorld, StableId, Transform};
use ph2d_editor_core::script_edits::{
    InspectorScriptStatus as S, InspectorScriptValue as V, ScriptFieldEdit as E,
};
use ph2d_script::{LuauScript, ScriptHost, ScriptValue};

use super::{apply, build_info};

fn script_file(source: &str) -> String {
    static N: AtomicU32 = AtomicU32::new(0);
    let dir = std::env::temp_dir().join(format!("ph2d_script_insp_{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp");
    let p = dir.join(format!("i{}.luau", N.fetch_add(1, Ordering::Relaxed)));
    std::fs::write(&p, source).expect("escreve");
    p.to_string_lossy().into_owned()
}

const DOIS: &str = r#"
ph2d.property("amplitude", 1.5, { min = 0, max = 10 })
ph2d.property("vidas", 3)
ph2d.property("ativo", true)
ph2d.property("nome", "bob")
"#;

fn cena(cfg: LuauScript) -> (SimWorld, u64) {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((Transform::IDENTITY, StableId(1), cfg))
        .id();
    (sim, e.to_bits())
}

fn pronto(sim: &mut SimWorld) -> ScriptHost {
    let mut h = ScriptHost::new().expect("vm");
    h.scene_sync(sim.world_mut());
    h
}

#[test]
fn sem_o_componente_nao_ha_seccao() {
    let mut sim = SimWorld::new();
    let e = sim.world_mut().spawn(Transform::IDENTITY).id();
    assert!(build_info(&sim, None, e.to_bits(), 1, false).is_none());
}

#[test]
fn o_estado_do_ficheiro_chega_ao_painel() {
    let (mut sim, bits) = cena(LuauScript::default());
    let h = pronto(&mut sim);
    let st = |sim: &SimWorld, h: Option<&ScriptHost>| {
        build_info(sim, h, bits, 1, false).expect("tem").status
    };
    assert_eq!(st(&sim, Some(&h)), S::NoFile);
    assert_eq!(
        st(&sim, None),
        S::Unavailable,
        "sem VM, di-lo — mesmo sem ficheiro"
    );

    let ok = script_file(DOIS);
    assert!(apply(&mut sim, bits, &E::Source(ok.clone())));
    assert_eq!(st(&sim, Some(&h)), S::Loading, "nomeado e ainda por ler");
    let h = pronto(&mut sim);
    assert_eq!(st(&sim, Some(&h)), S::Ready);

    let partido = script_file("function (");
    apply(&mut sim, bits, &E::Source(partido));
    let h = pronto(&mut sim);
    assert!(matches!(st(&sim, Some(&h)), S::Broken(m) if m.starts_with("syntax error")));

    apply(&mut sim, bits, &E::Source("/nao/ha/x.luau".into()));
    let h = pronto(&mut sim);
    assert_eq!(st(&sim, Some(&h)), S::Missing);
}

#[test]
fn as_linhas_sao_as_declaracoes_com_a_origem_e_o_passo() {
    let (mut sim, bits) = cena(LuauScript::at(script_file(DOIS)));
    apply(&mut sim, bits, &E::SetNumber("vidas".into(), 5.0));
    let h = pronto(&mut sim);
    let info = build_info(&sim, Some(&h), bits, 1, true).expect("tem");
    let nomes: Vec<&str> = info.props.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(nomes, ["amplitude", "vidas", "ativo", "nome"]);
    let amp = &info.props[0];
    assert_eq!((amp.value.clone(), amp.own), (V::Number(1.5), false));
    assert_eq!(
        (amp.min, amp.max, amp.step),
        (Some(0.0), Some(10.0), Some(0.1))
    );
    let vidas = &info.props[1];
    assert_eq!((vidas.value.clone(), vidas.own), (V::Number(5.0), true));
    assert_eq!(
        vidas.step,
        Some(1.0),
        "um default inteiro pede passos inteiros"
    );
    assert_eq!(info.props[2].value, V::Bool(true));
    assert_eq!(info.props[3].value, V::Text("bob".into()));
}

/// ⭐ **D1 no dreno:** pôr o MESMO número que o default muda o documento — ele passa a ser próprio.
#[test]
fn por_o_numero_do_default_torna_o_proprio_e_so_o_reset_o_larga() {
    let (mut sim, bits) = cena(LuauScript::at(script_file(DOIS)));
    assert!(apply(&mut sim, bits, &E::SetNumber("vidas".into(), 3.0)));
    assert!(
        !apply(&mut sim, bits, &E::SetNumber("vidas".into(), 3.0)),
        "a segunda vez é igual ao que já lá está"
    );
    let h = pronto(&mut sim);
    let info = build_info(&sim, Some(&h), bits, 1, false).expect("tem");
    assert!(info.props[1].own);
    assert!(apply(&mut sim, bits, &E::Forget("vidas".into())));
    assert!(!apply(&mut sim, bits, &E::Forget("vidas".into())));
    let info = build_info(&sim, Some(&h), bits, 1, false).expect("tem");
    assert!(!info.props[1].own);
}

#[test]
fn os_orfaos_dizem_o_que_o_script_quer_agora() {
    let mut cfg = LuauScript::at(script_file(DOIS));
    cfg.own.insert("velho".into(), ScriptValue::Number(9.0));
    cfg.own.insert("nome".into(), ScriptValue::Number(2.0));
    let (mut sim, bits) = cena(cfg);
    let h = pronto(&mut sim);
    let info = build_info(&sim, Some(&h), bits, 1, false).expect("tem");
    let o: Vec<(&str, ph2d_editor_core::script_edits::PorqueOrfao)> = info
        .orphans
        .iter()
        .map(|o| (o.name.as_str(), o.wants))
        .collect();
    assert_eq!(
        o,
        [
            (
                "nome",
                ph2d_editor_core::script_edits::PorqueOrfao::OutroTipo("string")
            ),
            (
                "velho",
                ph2d_editor_core::script_edits::PorqueOrfao::NaoDeclarado
            )
        ]
    );
    assert_eq!(
        info.props[3].value,
        V::Text("bob".into()),
        "D3: o do tipo errado não se aplica"
    );
}

/// ⚠️ **«Não sei» não é «nada»** — com o ficheiro partido os valores contam-se e não se oferecem.
#[test]
fn com_o_script_partido_os_valores_contam_se_como_guardados() {
    let mut cfg = LuauScript::at(script_file("function ("));
    cfg.own.insert("vidas".into(), ScriptValue::Number(9.0));
    let (mut sim, bits) = cena(cfg);
    let h = pronto(&mut sim);
    let info = build_info(&sim, Some(&h), bits, 1, false).expect("tem");
    assert!(info.orphans.is_empty());
    assert!(info.props.is_empty());
    assert_eq!(info.kept, 1);
}

#[test]
fn a_falha_da_corrida_chega_ao_painel() {
    let (mut sim, bits) = cena(LuauScript::at(script_file(
        r#"function update(self, dt) error("partiu") end"#,
    )));
    let mut h = pronto(&mut sim);
    h.scene_tick(sim.world_mut(), 0.1);
    let info = build_info(&sim, Some(&h), bits, 1, true).expect("tem");
    assert!(info.failure.is_some_and(|m| m.contains("partiu")));
}

#[test]
fn browse_nao_escreve_e_um_corpo_dinamico_e_avisado() {
    let (mut sim, bits) = cena(LuauScript::default());
    assert!(!apply(&mut sim, bits, &E::Browse));
    let e = ph2d_ecs::Entity::from_bits(bits);
    sim.world_mut()
        .entity_mut(e)
        .insert(ph2d_physics_ecs::RigidBody::default());
    let info = build_info(&sim, None, bits, 2, false).expect("tem");
    assert!(info.also_physics);
    assert_eq!(info.selected_count, 2);
}

#[test]
fn browse_escreve_o_que_o_dialogo_devolve_e_cancelar_nao_escreve_nada() {
    // ⚠️ **Parte de um caminho NÃO vazio**: com o default (`""`), um cancelamento que escrevesse um
    // caminho vazio não mudaria nada e passaria por correcto.
    let (mut sim, bits) = cena(LuauScript::at("/antes.luau"));
    let lote = [(bits, E::Browse)];
    assert!(!super::apply_all(&mut sim, &lote, || None), "cancelado");
    let e = ph2d_ecs::Entity::from_bits(bits);
    assert_eq!(
        sim.world().get::<LuauScript>(e).expect("cfg").source,
        "/antes.luau"
    );
    assert!(super::apply_all(&mut sim, &lote, || Some(
        "/a/b.luau".into()
    )));
    assert_eq!(
        sim.world().get::<LuauScript>(e).expect("cfg").source,
        "/a/b.luau"
    );
}

/// ⭐⭐⭐ **As duas edições novas chegam ao documento, e a IDEMPOTÊNCIA fica** — pôr o mesmo par
/// duas vezes não toca no componente na segunda.
///
/// ⚠️ **A 2.ª metade é a que importa e a 1.ª sozinha mente:** o `bevy` marca a alteração no
/// `deref_mut`, logo um dreno que escrevesse sempre poria toda amostra de cor a acordar quem lê
/// `Changed<…>` a cada quadro em que o selector estivesse aberto — e o selector espelha a cor
/// viva **por quadro**. ⭐ E a 3.ª é a divergência D1: pôr o valor pela PRIMEIRA vez muda, mesmo
/// igual ao default.
#[test]
fn a_posicao_e_a_cor_chegam_ao_documento_e_nao_mexem_duas_vezes() {
    const TIPOS: &str = r#"
ph2d.property("offset", ph2d.vec2(0, 1))
ph2d.property("tint", ph2d.color(1, 1, 1))
"#;
    let (mut sim, bits) = cena(LuauScript::at(script_file(TIPOS)));
    let h = pronto(&mut sim);
    let ler = |sim: &SimWorld, n: &str| {
        build_info(sim, Some(&h), bits, 1, false)
            .expect("tem")
            .props
            .iter()
            .find(|p| p.name == n)
            .map(|p| (p.value.clone(), p.own))
            .unwrap_or_else(|| panic!("`{n}` nao resolveu"))
    };
    assert_eq!(ler(&sim, "offset"), (V::Vec2([0.0, 1.0]), false));
    assert_eq!(ler(&sim, "tint"), (V::Color([1.0, 1.0, 1.0, 1.0]), false));

    assert!(apply(
        &mut sim,
        bits,
        &E::SetVec2("offset".into(), [2.0, -3.0])
    ));
    assert_eq!(ler(&sim, "offset"), (V::Vec2([2.0, -3.0]), true));
    assert!(
        !apply(&mut sim, bits, &E::SetVec2("offset".into(), [2.0, -3.0])),
        "o mesmo par duas vezes tocou no componente na segunda"
    );

    let cor = [0.5, 0.25, 0.0, 1.0];
    assert!(apply(&mut sim, bits, &E::SetColor("tint".into(), cor)));
    assert_eq!(ler(&sim, "tint"), (V::Color(cor), true));
    assert!(!apply(&mut sim, bits, &E::SetColor("tint".into(), cor)));

    // ⭐ D1: pôr o valor IGUAL ao default pela primeira vez torna-o PRÓPRIO.
    assert!(apply(
        &mut sim,
        bits,
        &E::SetColor("tint".into(), [1.0, 1.0, 1.0, 1.0])
    ));
    assert_eq!(ler(&sim, "tint"), (V::Color([1.0; 4]), true));

    // ⭐ E o `Reset` larga-o — a MESMA porta dos outros tipos.
    assert!(apply(&mut sim, bits, &E::Forget("tint".into())));
    assert_eq!(ler(&sim, "tint"), (V::Color([1.0; 4]), false));
}

/// ⭐ **O passo de arrasto de um `vec2` lê o eixo com CASAS DECIMAIS** — `ph2d.vec2(0, 1.5)` pede
/// passos finos, e a regra é a mesma dos números: *o que o autor ESCREVEU*.
///
/// ⚠️ **Mutação que deve sangrar:** ler sempre o eixo `x` — com `(0, 1.5)` ele é redondo, e a
/// fileira inteira passaria a arrastar de um em um.
#[test]
fn o_passo_de_uma_posicao_sai_do_eixo_com_casas_decimais() {
    let passo = |src: &str| {
        let (mut sim, bits) = cena(LuauScript::at(script_file(src)));
        let h = pronto(&mut sim);
        build_info(&sim, Some(&h), bits, 1, false)
            .expect("tem")
            .props[0]
            .step
    };
    assert_eq!(passo("ph2d.property(\"p\", ph2d.vec2(0, 1.5))"), Some(0.1));
    assert_eq!(passo("ph2d.property(\"p\", ph2d.vec2(2, 3))"), Some(1.0));
    // ⭐ E o `step` declarado ganha sempre — ele é o que o autor ESCREVEU.
    assert_eq!(
        passo("ph2d.property(\"p\", ph2d.vec2(2, 3), { step = 0.25 })"),
        Some(0.25)
    );
}

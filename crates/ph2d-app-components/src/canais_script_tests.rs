//! Os gates da [`super`] — a família SCRIPT de ponta a ponta: publica o valor que o objecto USA,
//! a curva escreve um valor PRÓPRIO, e o corte de script só aponta o que a biblioteca tem.

use super::*;
use crate::component_registry_for_tests::registo;
use ph2d_script::props::PropHint;
use ph2d_script::{ScriptLibrary, ScriptValue as V};

fn decl(name: &str, default: V) -> PropDecl {
    PropDecl {
        name: name.to_owned(),
        default,
        hint: PropHint::default(),
    }
}

/// Um objecto com o script «mover» (que declara `speed = 1` e `cor`) e o `speed` posto a 3.
fn cena() -> (SimWorld, Entity, ScriptId) {
    let mut sim = SimWorld::new();
    let mut lib = ScriptLibrary::new();
    let id = lib.insert("mover.luau", "");
    ph2d_script::ProjectScripts::install(sim.world_mut(), lib);
    let mut cfg = LuauScript::library(id);
    cfg.own.insert("speed".into(), V::Number(3.0));
    let e = sim.world_mut().spawn(cfg).id();
    let decls = vec![
        decl("speed", V::Number(1.0)),
        decl("cor", V::Color([1.0, 0.0, 0.0, 1.0])),
    ];
    sim.world_mut()
        .insert_resource(Declaracoes([(id, decls)].into_iter().collect()));
    (sim, e, id)
}

fn lido(sim: &SimWorld, e: Entity, k: FieldKey) -> Option<f32> {
    sim.world()
        .get_resource::<CanaisLidos>()?
        .valor(e.to_bits(), k)
}

/// ⭐ Publica o que o objecto USA: o próprio (`speed = 3`) e o default da declaração (a cor), cada
/// faixa na sua chave. (Mutação: publicar só o `own` ⇒ a cor some.)
#[test]
fn publica_o_proprio_e_o_default_da_declaracao() {
    let (mut sim, e, _) = cena();
    let mut textos = TextosDosCanais::default();
    publicar(&mut sim, &[e.to_bits()], &mut textos);
    let speed = cs::chave("speed", cs::Tipo::Number, 0, &mut textos).unwrap();
    assert_eq!(lido(&sim, e, speed), Some(3.0));
    let verde = cs::chave("cor", cs::Tipo::Color, 1, &mut textos).unwrap();
    assert_eq!(lido(&sim, e, verde), Some(0.0));
    let vermelho = cs::chave("cor", cs::Tipo::Color, 0, &mut textos).unwrap();
    assert_eq!(lido(&sim, e, vermelho), Some(1.0));
}

/// ⭐ A curva escreve um valor PRÓPRIO — na faixa dela, e o resto da cor fica — e declara a escrita
/// à pré-visualização (o documento não a guarda).
#[test]
fn a_curva_escreve_um_valor_proprio_na_faixa_dela() {
    let (mut sim, e, _) = cena();
    let reg = registo();
    let mut textos = TextosDosCanais::default();
    let verde = cs::chave("cor", cs::Tipo::Color, 1, &mut textos).unwrap();
    sim.world_mut()
        .get_resource_or_insert_with(CanaisEscritos::default)
        .escrever(e.to_bits(), verde, 0.5);
    let mut drive = PreviewDrive::default();
    assert!(aplicar(&mut sim, &reg, &mut drive, &textos));
    let own = &sim.world().get::<LuauScript>(e).unwrap().own;
    assert_eq!(own.get("cor"), Some(&V::Color([1.0, 0.5, 0.0, 1.0])));
    assert_eq!(own.get("speed"), Some(&V::Number(3.0)), "o resto fica");
    let mut textos = textos;
    publicar(&mut sim, &[e.to_bits()], &mut textos);
    assert_eq!(
        lido(&sim, e, verde),
        Some(0.5),
        "o quadro seguinte lê o que ficou"
    );
}

/// ⛔ O corte de script só aponta um script que o projeto TEM (o id é da biblioteca); `0` tira-o.
/// (Mutação: tirar o `script_existe` do funil ⇒ o objecto aponta o `#9`, que não existe.)
#[test]
fn o_corte_de_script_so_aponta_o_que_a_biblioteca_tem() {
    let (mut sim, e, id) = cena();
    let reg = registo();
    let k = ph2d_ecs::canal_componente::chaves_do_rotulo("component.field.script_fields.1")[0];
    let textos = TextosDosCanais::default();
    let fonte = |sim: &SimWorld| sim.world().get::<LuauScript>(e).unwrap().source.clone();
    assert!(
        crate::canais_componente::escrever(&reg, sim.world_mut(), e, k, 9.0, &textos).is_none()
    );
    assert_eq!(fonte(&sim), ScriptSource::Library(id));
    assert!(
        crate::canais_componente::escrever(&reg, sim.world_mut(), e, k, 0.0, &textos).is_some()
    );
    assert_eq!(fonte(&sim), ScriptSource::None);
    #[expect(clippy::cast_precision_loss, reason = "um id pequeno")]
    let x = id.0 as f32;
    assert!(crate::canais_componente::escrever(&reg, sim.world_mut(), e, k, x, &textos).is_some());
    assert_eq!(fonte(&sim), ScriptSource::Library(id));
}

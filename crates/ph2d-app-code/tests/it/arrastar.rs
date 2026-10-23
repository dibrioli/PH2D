//! Item 8 (`docs/CodeEditor/07_arrastar_para_o_codigo.md`): um arrasto entre painéis sobre a aba de script
//! — o cursor fino onde cairia, o balão das teclas, e a soltura num passo de undo, pelas portas do
//! `DocumentSurface` que a shell chama (`drop_hover` · `drop`).

use crate::Mesa;
use ph2d_editor_core::doc_surface::{DocumentSurface, DropAt, Dropping};
use ph2d_host::Modifiers;
use ph2d_script::ScriptId;

const TEXTO: &str = "function update(self)\n  ph2d.set(, \"x\", 1)\nend\n";

fn mesa() -> (Mesa, ScriptId) {
    let (mut m, ids) = Mesa::new(&[("a.luau", TEXTO)]);
    let id = ids[0];
    m.pede(id);
    m.pinta(id);
    (m, id)
}

/// O ponto do ecrã logo à direita do início do byte `b` (o clique cai nele).
fn ponto(m: &mut Mesa, id: ScriptId, b: usize) -> (f32, f32) {
    let c = m.docs().caret_rect_at(id, b).expect("à vista");
    (c.x + 1.0, c.y + c.h * 0.5)
}

fn em<'a>(p: (f32, f32), names: &'a [String], ctrl: bool) -> DropAt<'a> {
    DropAt {
        x: p.0,
        y: p.1,
        what: Dropping::Objects(names),
        mods: Modifiers {
            ctrl,
            ..Modifiers::default()
        },
    }
}

#[test]
fn por_cima_do_texto_mostra_onde_cai_e_largar_escreve_num_passo_de_undo() {
    let (mut m, id) = mesa();
    let b = TEXTO.find(", \"x\"").expect("agulha");
    let p = ponto(&mut m, id, b);
    let porta = ["Porta".to_owned()];
    assert!(
        m.docs().drop_hover(id.0, Some(em(p, &porta, false))),
        "sobre o texto aceita"
    );
    let (at, balao) = m.docs().drop_mark_of(id).expect("a marca");
    assert_eq!(at, b, "o cursor fino é o byte do clique");
    assert_eq!(balao.len(), 2, "o que larga e o que o Ctrl faz: {balao:?}");
    m.pinta(id);
    assert!(m.docs().drop(id.0, em(p, &porta, false)));
    assert!(m.docs().drop_mark_of(id).is_none(), "largar apaga a marca");
    let esperado = TEXTO.replacen("set(,", "set(ph2d.object(\"Porta\"),", 1);
    assert_eq!(m.docs().text_of(id).as_deref(), Some(esperado.as_str()));
    let fim = b + "ph2d.object(\"Porta\")".len();
    assert_eq!(
        m.docs().selection_of(id),
        [fim..fim],
        "o cursor no fim do que se escreveu"
    );
    assert!(m.docs().undo(id.0, false));
    assert_eq!(
        m.docs().text_of(id).as_deref(),
        Some(TEXTO),
        "um Ctrl+Z desfaz a soltura inteira"
    );
}

#[test]
fn com_ctrl_a_marca_e_a_escrita_vao_para_o_inicio_da_linha() {
    let (mut m, id) = mesa();
    let b = TEXTO.find(", \"x\"").expect("agulha");
    let p = ponto(&mut m, id, b);
    let nomes = ["Porta".to_owned(), "Caixa Grande".to_owned()];
    assert!(m.docs().drop_hover(id.0, Some(em(p, &nomes, true))));
    let linha = TEXTO.find("ph2d.set").expect("agulha");
    assert_eq!(
        m.docs().drop_mark_of(id).map(|d| d.0),
        Some(linha),
        "o Ctrl escreve no início da linha"
    );
    assert!(m.docs().drop(id.0, em(p, &nomes, true)));
    assert_eq!(
        m.docs().text_of(id).as_deref(),
        Some(
            "function update(self)\n  local porta = ph2d.object(\"Porta\")\n  local caixa_grande = ph2d.object(\"Caixa Grande\")\n  ph2d.set(, \"x\", 1)\nend\n"
        )
    );
}

#[test]
fn fora_do_texto_nao_aceita_e_sair_apaga_a_marca() {
    let (mut m, id) = mesa();
    let b = TEXTO.find(", \"x\"").expect("agulha");
    let p = ponto(&mut m, id, b);
    let porta = ["Porta".to_owned()];
    assert!(m.docs().drop_hover(id.0, Some(em(p, &porta, false))));
    assert!(
        m.docs().drop_hover(id.0, None) == false && m.docs().drop_mark_of(id).is_none(),
        "saiu"
    );
    let calha = (1.0, p.1);
    assert!(
        !m.docs().drop_hover(id.0, Some(em(calha, &porta, false))),
        "na calha não"
    );
    assert!(!m.docs().drop(id.0, em(calha, &porta, false)));
    assert_eq!(
        m.docs().text_of(id).as_deref(),
        Some(TEXTO),
        "nada se escreveu"
    );
}

// ── Os nomes da cena: o aviso, o Ctrl+clique, o renomear (§3.5–8) ──────────────────────────────

use crate::completar::{T0, quadro_em};
use crate::definicao::clica_em;
use ph2d_ecs::{Entity, MasterRoot, Name, Transform};

fn objecto(m: &mut Mesa, nome: &str) -> Entity {
    m.sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new(nome)))
        .id()
}

fn tipos(m: &mut Mesa, id: ScriptId) -> Vec<&'static str> {
    m.docs()
        .problems_of(id)
        .into_iter()
        .filter_map(|p| p.explanation.map(|e| e.kind))
        .collect()
}

#[test]
fn o_nome_que_a_cena_nao_tem_e_um_aviso_e_some_quando_a_cena_o_tem() {
    let src = "function update(self)\n  ph2d.set(ph2d.object(\"Portaa\"), \"x\", 1)\n  ph2d.set(ph2d.object(\"Caixa\"), \"x\", 1)\n  ph2d.spawn(\"Bala\")\n  ph2d.find_by_name(\"Ninguem\")\nend\n";
    let (mut m, ids) = Mesa::new(&[("a.luau", src)]);
    let id = ids[0];
    let porta = objecto(&mut m, "Porta");
    objecto(&mut m, "Caixa");
    objecto(&mut m, "Caixa");
    m.sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Tiro"), MasterRoot));
    m.pede(id);
    quadro_em(&mut m, T0);
    assert_eq!(
        tipos(&mut m, id),
        ["unknown_object_near", "object_twice", "unknown_master"],
        "o find_by_name sem objecto não acusa: nil é resposta"
    );
    let p = m.docs().problems_of(id).remove(0);
    assert!(p.message.contains("Porta"), "o «quis dizer»: {}", p.message);
    m.sim
        .world_mut()
        .entity_mut(porta)
        .insert(Name::new("Portaa"));
    quadro_em(&mut m, T0 + 1);
    assert_eq!(
        tipos(&mut m, id),
        ["object_twice", "unknown_master"],
        "a cena mudou: o aviso some sem tocar no texto"
    );
}

#[test]
fn ctrl_clique_no_nome_selecciona_o_objecto() {
    let src = "local p = ph2d.object(\"Porta\")\n";
    let (mut m, ids) = Mesa::new(&[("a.luau", src)]);
    let id = ids[0];
    objecto(&mut m, "Outra");
    let porta = objecto(&mut m, "Porta");
    m.pede(id);
    quadro_em(&mut m, T0);
    m.pinta(id);
    clica_em(
        &mut m,
        id,
        src.find("Porta").expect("agulha") + 2,
        crate::ctrl(),
    );
    quadro_em(&mut m, T0 + 1);
    assert_eq!(m.hero.gizmo.selection, Some(porta.to_bits()));
}

#[test]
fn renomear_o_objecto_reescreve_os_scripts_e_um_undo_global_devolve() {
    let aberto = "local p = ph2d.object(\"Porta\")\n";
    let fechado = "local q = ph2d.find_by_name('Porta')\nph2d.spawn(\"Porta\")\n";
    let (mut m, ids) = Mesa::new(&[("a.luau", aberto), ("b.luau", fechado)]);
    let porta = objecto(&mut m, "Porta");
    m.pede(ids[0]);
    quadro_em(&mut m, T0);
    // O que a shell faz no commit do renomear (`hierarchy_rename.rs`).
    m.sim
        .world_mut()
        .entity_mut(porta)
        .insert(Name::new("Portão"));
    ph2d_app_code::object_renamed(&mut m.hero, m.sim.world_mut(), porta, "Porta", "Portão");
    let novo_a = "local p = ph2d.object(\"Portão\")\n";
    let novo_b = "local q = ph2d.find_by_name('Portão')\nph2d.spawn(\"Porta\")\n";
    assert_eq!(
        m.docs().text_of(ids[0]).as_deref(),
        Some(novo_a),
        "a aba aberta"
    );
    assert_eq!(m.texto_salvo(ids[0]), novo_a, "a biblioteca do aberto");
    assert_eq!(
        m.texto_salvo(ids[1]),
        novo_b,
        "o fechado; o spawn é de um MESTRE e fica"
    );
    let elo = m
        .hero
        .undo_links
        .pending
        .take()
        .expect("o passo do renomear leva o elo");
    m.hero.undo_links.alive.insert(elo);
    quadro_em(&mut m, T0 + 1);
    assert_eq!(
        m.docs().text_of(ids[0]).as_deref(),
        Some(novo_a),
        "o quadro não desfaz nada"
    );
    // O Ctrl+Z global: a fila desfaz o nome e publica o elo desfeito.
    m.sim
        .world_mut()
        .entity_mut(porta)
        .insert(Name::new("Porta"));
    m.hero.undo_links.done.push((elo, false));
    quadro_em(&mut m, T0 + 2);
    assert_eq!(m.docs().text_of(ids[0]).as_deref(), Some(aberto));
    assert_eq!(m.texto_salvo(ids[0]), aberto);
    assert_eq!(m.texto_salvo(ids[1]), fechado);
}

/// ⭐ Doc CodeEditor 08 §3.4: um `componente.campo` em `ph2d.get/set` que nenhum componente tem é um
/// aviso com «quis dizer»; a pose (`"x"`) e um nome montado não contam.
#[test]
fn o_nome_de_propriedade_que_nenhum_componente_tem_e_um_aviso() {
    let src = "function update(self)\n  ph2d.set(self.id, \"health.maxx\", 1)\n  ph2d.get(self.id, \"health.max\")\n  ph2d.get(self.id, \"x\")\n  ph2d.get(self.id, \"vida\" .. \".max\")\n  ph2d.get(self.id, \"jetpack.fuel\")\nend\n";
    let (mut m, ids) = Mesa::new(&[("a.luau", src)]);
    let id = ids[0];
    m.pede(id);
    quadro_em(&mut m, T0);
    assert_eq!(
        tipos(&mut m, id),
        ["unknown_property_near", "unknown_property"]
    );
    let p = m.docs().problems_of(id).remove(0);
    assert!(
        p.message.contains("\"health.max\""),
        "o «quis dizer»: {}",
        p.message
    );
}

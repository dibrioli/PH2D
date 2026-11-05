//! ⭐ **C3 — o losango de um campo PRESO, o seletor de cor e o resto do alfinete** (`docs/CartoesUI/09`),
//! por gesto real sobre a `Mesa` do C2: prende-se a A com B escolhido; o estado do losango é o de A e o
//! clique grava em A, em todos os cartões presos. O Auto Key tem o gate dele na shell
//! (`autokey_preso_tests.rs`).

use super::*;
use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_app_vec::marcas::Contexto;
use ph2d_core::Playhead;
use ph2d_editor_core::widget::key_mark::{self, KeyMark, key_mark_of, take_key_mark_click};
use ph2d_host_presos::losango;
use ph2d_panel_inspector::{
    CARD_APPEARANCE, CARD_MATERIAL, CARD_OBJECT, CARD_ORDERING, CARD_TRANSFORM,
};
use ph2d_timeline::{Instante, PropKind, TimelineIntent, TimelineState};

const INST: Instante = Instante {
    t_cut: 0.0,
    solo: true,
};

fn valor(_: u64, _: PropKind) -> Option<f32> {
    Some(1.0)
}

fn autorar(e: u64, p: PropKind) -> Option<TimelineIntent> {
    Some(TimelineIntent::AddKey {
        entity: e,
        prop: p,
        t: RationalTime::from_seconds(0.0),
        value: AnimValue::Float(1.0),
        interp: Interp::Linear,
    })
}

/// A timeline de uma `Mesa`: o que a `fase_autokey` publica antes da pintura.
struct Linha {
    st: TimelineState,
    ph: Playhead,
}

impl Linha {
    fn nova() -> Self {
        let mut st = TimelineState::new();
        st.keys_mode = true;
        Self {
            st,
            ph: Playhead::new(1.0 / 60.0),
        }
    }

    fn sel(m: &Mesa) -> Vec<u64> {
        m.sel.iter().map(|i| m.bits[*i]).collect()
    }

    /// Um quadro com os canais e os dois mapas publicados, e os blocos que o `Mesa::quadro` não publica.
    fn quadro(&self, m: &mut Mesa) {
        let todos = m.bits.clone();
        ph2d_app_vec::ids_da_pilha::passe(&mut m.sim, &mut m.scene);
        ph2d_app_vec::canais::publicar(
            &mut m.sim,
            &m.scene,
            &ph2d_app_vec::canais::ConducaoVetor::default(),
            &todos,
        );
        let pins = losango::pinos(&m.h.store);
        insp::com_cada_preso(&pins, |b| {
            componentes(b, b == todos[0]);
        });
        componentes(todos[m.sel[0]], m.sel[0] == 0);
        self.publica(m, &pins);
        m.quadro();
    }

    fn publica(&self, m: &Mesa, pins: &[u64]) {
        let ctx = Contexto::do_painel(None);
        let sel = Self::sel(m);
        let da_selecao = losango::marcas(&self.st.doc, &m.sim, ctx, &sel, INST, valor);
        losango::publica_marcas(&self.st.doc, &m.sim, &m.scene, pins, INST, valor);
        key_mark::set_key_marks(da_selecao);
    }

    fn quadros(&self, m: &mut Mesa, n: usize) {
        for _ in 0..n {
            self.quadro(m);
        }
    }

    /// As intenções que o clique pendente pede, pela porta da `fase_autokey`.
    fn pede(&self, m: &Mesa) -> (Option<u64>, Vec<TimelineIntent>) {
        let c = take_key_mark_click().expect("o clique não chegou ao losango");
        let sel = Self::sel(m);
        let ctx = Contexto::do_painel(None);
        let (ctx_c, alvo) = losango::alvo_do_clique(&c, &m.sim, &m.scene, ctx, &sel);
        // O que o mundo publica, e os campos do componente clicado (a `Mesa` não tem o corpo dele).
        let do_cartao: Vec<ph2d_ecs::FieldKey> = (c.fields.iter())
            .flat_map(|f| ph2d_app_components::marcas_inspector::canal_do_campo(*f).into_owned())
            .filter_map(|p| match p {
                PropKind::Field(k) => Some(k),
                _ => None,
            })
            .collect();
        let do_cartao = &do_cartao;
        let lidos = |e| {
            let mundo = ph2d_ecs::canal::campos_lidos(&m.sim, e);
            let mut v: Vec<ph2d_ecs::FieldKey> = mundo.into_iter().map(|(k, _)| k).collect();
            v.extend(do_cartao.iter().copied());
            v
        };
        let i = losango::clique(
            &self.st.doc,
            &m.sim,
            ctx_c,
            &c.fields,
            &alvo,
            INST,
            lidos,
            valor,
            autorar,
        );
        (c.sujeito, i)
    }

    fn aplica(&mut self, i: Vec<TimelineIntent>) {
        for x in i {
            ph2d_timeline::apply_intent(&mut self.st, &mut self.ph, x);
        }
    }
}

/// Ordering, Material e o olho de um objecto.
fn componentes(b: u64, a: bool) {
    super::super::o_alfinete_dos_cartoes_c1::blocos(b, a);
}

/// Os losangos que a mão acha dentro do solto: `(ponto, id do alvo)`.
fn losangos(m: &Mesa, k: NodeId) -> Vec<((f32, f32), NodeId)> {
    losangos_em(&m.h, k)
}

fn losangos_em(h: &HeroScreen, k: NodeId) -> Vec<((f32, f32), NodeId)> {
    let s = h.store.panel_rect(loose_rect_of(k)).expect("o solto");
    let mut out: Vec<((f32, f32), NodeId)> = Vec::new();
    let mut y = s.y;
    while y < s.y + s.h {
        let mut x = s.x;
        while x < s.x + s.w {
            if let Some((id, r)) = h.hit_index.hit_with_rect(x, y)
                && key_mark::key_mark_target(id).is_some()
                && !out.iter().any(|(_, o)| *o == id)
            {
                out.push((centro(r), id));
            }
            x += 2.0;
        }
        y += 2.0;
    }
    out
}

/// O cartão `sec` solto no canvas, aberto e preso a A; depois `sel` escolhido.
fn prende_aberto(m: &mut Mesa, l: &Linha, sec: NodeId, sel: &[usize]) -> NodeId {
    m.escolhe(&[0]);
    m.h.store.set_collapsed(sec, false);
    m.h.store.set_section_open_live(sec, 1.0);
    l.quadros(m, 2);
    m.h.store
        .detach_card(sec, INSP_PANEL, (640.0, 120.0), 300.0);
    l.quadros(m, 2);
    m.h.store.toggle_loose_pin(sec);
    assert_eq!(
        m.h.store.loose_card(sec).and_then(|c| c.pin),
        Some(m.bits[0])
    );
    m.escolhe(sel);
    l.quadros(m, 3);
    sec
}

/// ⭐⭐ **Todos os cartões, todo losango:** o clique é de A, nunca de quem está escolhido, e o estado do
/// losango no cartão passa a ser o de A; a vista da seleção não muda; desfazer tira a chave.
/// *Mutação: `alvo_do_clique` a ignorar o sujeito ⇒ a chave cai em B. É o defeito que isto repara.*
#[test]
fn every_diamond_of_every_pinned_card_keys_its_object_and_shows_its_state() {
    let cartoes = [
        ("Transform", CARD_TRANSFORM, 1),
        ("Appearance", CARD_APPEARANCE, 1),
        ("Fill", CARD_FILL, 2),
        ("Stroke", CARD_STROKE, 2),
    ];
    for (nome, sec, piso) in cartoes {
        let mut m = Mesa::nova();
        let mut l = Linha::nova();
        let k = prende_aberto(&mut m, &l, sec, &[1]);
        let achados = losangos(&m, k);
        assert!(
            achados.len() >= piso,
            "{nome}: {} losangos no cartão preso",
            achados.len()
        );
        let (mut chaves, b, a) = (0, m.bits[1], m.bits[0]);
        for (ponto, id) in achados {
            let campos = key_mark::key_mark_target(id).expect("o alvo").fields;
            let _ = m.h.store.record_pointer_down(Some(NodeId(1)), 0);
            mao(&mut m.h, PointerKind::Down, ponto);
            mao(&mut m.h, PointerKind::Up, ponto);
            let (sujeito, pedido) = l.pede(&m);
            assert_eq!(sujeito, Some(a), "{nome}: o clique não sabe de quem é");
            let mesmas: Vec<u64> = pedido
                .iter()
                .filter_map(|i| match i {
                    TimelineIntent::AddKey { entity, .. } => Some(*entity),
                    _ => None,
                })
                .collect();
            assert!(
                mesmas.iter().all(|e| *e == a),
                "{nome} {id:?}: a chave caiu noutro objecto ({mesmas:x?}, B = {b:x})"
            );
            if mesmas.is_empty() {
                continue;
            }
            chaves += 1;
            let antes = key_mark_of(&campos);
            l.aplica(pedido);
            l.quadros(&mut m, 1);
            let do_preso = key_mark::com_marcas_de(a, || key_mark_of(&campos));
            assert!(
                matches!(do_preso, Some(KeyMark::Chave | KeyMark::Mudado)),
                "{nome} {id:?}: o losango do preso não mostra a chave de A: {do_preso:?}"
            );
            assert_eq!(
                key_mark_of(&campos),
                antes,
                "{nome} {id:?}: a vista da seleção herdou a chave de A"
            );
            assert!(l.st.undo(), "{nome}: sem passo de desfazer");
            l.quadros(&mut m, 1);
            assert_eq!(
                key_mark::com_marcas_de(a, || key_mark_of(&campos)),
                Some(KeyMark::Livre),
                "{nome} {id:?}: desfazer não tirou a chave"
            );
        }
        eprintln!("[c3] {nome}: {chaves} losangos gravam em A");
        assert!(chaves >= piso, "{nome}: só {chaves} losangos gravaram");
    }
}

/// O mesmo em B+C e em A+B: A sozinho, com o estado da seleção à parte.
#[test]
fn a_diamond_of_a_pinned_card_keys_only_its_object_with_any_selection() {
    for sel in SELECOES {
        let mut m = Mesa::nova();
        let mut l = Linha::nova();
        let k = prende_aberto(&mut m, &l, CARD_STROKE, sel);
        let (ponto, _) = *losangos(&m, k).first().expect("um losango");
        let _ = m.h.store.record_pointer_down(Some(NodeId(1)), 0);
        mao(&mut m.h, PointerKind::Down, ponto);
        mao(&mut m.h, PointerKind::Up, ponto);
        let (sujeito, pedido) = l.pede(&m);
        assert_eq!(sujeito, Some(m.bits[0]));
        let chaves = pedido
            .iter()
            .filter(|i| matches!(i, TimelineIntent::AddKey { .. }))
            .count();
        assert!(chaves > 0, "{sel:?}: nada a gravar");
        l.aplica(pedido);
        let b = |e: u64| l.st.doc.bindings().iter().any(|x| x.entity == e);
        assert!(b(m.bits[0]), "{sel:?}: A sem chave");
        assert!(
            !b(m.bits[1]) && !b(m.bits[2]),
            "{sel:?}: a chave espalhou-se"
        );
    }
}

/// Um cartão de componente (o Player): o estado é do mapa de A e o clique grava só em A, com outro escolhido.
#[test]
fn a_diamond_of_a_pinned_component_card_is_of_its_object() {
    use super::super::o_bloco8_player_solto::{hero_com_aberto, solto_de};
    use super::super::o_inspector_armado::{arma_tudo, fisica_armada};
    use ph2d_app_components::marcas_inspector as comp;
    use ph2d_editor_core::ids::INSP_LIVE_PLAYER_SECTION as PLAYER;
    arma_tudo();
    let (a, b) = (fisica_armada().entity_bits, 0xB_u64);
    let mut h = hero_com_aberto(PLAYER);
    solto_de(&mut h, PLAYER, 300.0);
    clica(&mut h, loose_pin_of(PLAYER));
    assert_eq!(h.store.loose_card(PLAYER).and_then(|c| c.pin), Some(a));
    let tl = ph2d_timeline::TimelineDoc::new();
    let mapa = |e| comp::marcas(&tl, &[e], INST, valor);
    insp::com_cada_preso(&[a], |_| arma_tudo());
    for _ in 0..3 {
        // Os rótulos que o quadro anterior pintou resolvem os campos do catálogo.
        let (dele, do_outro) = (mapa(a), mapa(b));
        key_mark::set_key_marks_dos_presos(std::collections::BTreeMap::from([(a, dele)]));
        key_mark::set_key_marks(do_outro);
        pinta(&mut h, 1);
    }
    let achados = losangos_em(&h, PLAYER);
    eprintln!("[c3] Player: {} losangos", achados.len());
    assert!(!achados.is_empty(), "o Player preso sem losangos");
    let (ponto, _) = achados[0];
    mao(&mut h, PointerKind::Down, ponto);
    let c = take_key_mark_click().expect("o clique no losango");
    assert_eq!(c.sujeito, Some(a));
    let (sim, cena) = (SimWorld::new(), VecScene::new());
    let ctx = Contexto::default();
    let (ctx_c, alvo) = losango::alvo_do_clique(&c, &sim, &cena, ctx, &[b]);
    let do_cartao: Vec<ph2d_ecs::FieldKey> = (c.fields.iter())
        .flat_map(|f| comp::canal_do_campo(*f).into_owned())
        .filter_map(|p| match p {
            PropKind::Field(k) => Some(k),
            _ => None,
        })
        .collect();
    let do_cartao = &do_cartao;
    let pedido = losango::clique(
        &tl,
        &sim,
        ctx_c,
        &c.fields,
        &alvo,
        INST,
        |_| do_cartao.clone(),
        valor,
        autorar,
    );
    let donos: Vec<u64> = pedido
        .iter()
        .filter_map(|i| match i {
            TimelineIntent::AddKey { entity, .. } => Some(*entity),
            _ => None,
        })
        .collect();
    assert!(
        !donos.is_empty() && donos.iter().all(|e| *e == a),
        "{donos:x?}"
    );
}

/// Um losango na lista, com um preso aberto, continua a ser da seleção.
#[test]
fn a_diamond_of_the_docked_card_stays_of_the_selection() {
    let mut m = Mesa::nova();
    let l = Linha::nova();
    let _ = prende_aberto(&mut m, &l, CARD_STROKE, &[1]);
    let painel_inteiro = m.h.store.panel_rect(INSP_PANEL).expect("o painel");
    let mut achado = None;
    let mut y = painel_inteiro.y;
    while achado.is_none() && y < painel_inteiro.y + painel_inteiro.h {
        let mut x = painel_inteiro.x;
        while x < painel_inteiro.x + painel_inteiro.w {
            if let Some((id, r)) = m.h.hit_index.hit_with_rect(x, y)
                && key_mark::key_mark_target(id).is_some()
            {
                achado = Some(centro(r));
                break;
            }
            x += 2.0;
        }
        y += 2.0;
    }
    let ponto = achado.expect("um losango na lista");
    mao(&mut m.h, PointerKind::Down, ponto);
    mao(&mut m.h, PointerKind::Up, ponto);
    let (sujeito, pedido) = l.pede(&m);
    assert_eq!(sujeito, None, "o losango da lista virou do preso");
    assert!(
        pedido.iter().all(|i| match i {
            TimelineIntent::AddKey { entity, .. } => *entity == m.bits[1],
            _ => true,
        }),
        "o losango da lista não é da seleção"
    );
}

/// Sem mapa do preso publicado, o corpo dele não pinta o da seleção.
/// *Mutação: `com_marcas_de` fora do `corpo_do_preso` ⇒ o cartão preso mostra o estado da seleção.*
#[test]
fn a_pinned_card_body_never_reads_the_selection_map() {
    let mut m = Mesa::nova();
    let l = Linha::nova();
    let k = prende_aberto(&mut m, &l, CARD_STROKE, &[1]);
    assert!(!losangos(&m, k).is_empty(), "controlo: há losangos");
    let pins = losango::pinos(&m.h.store);
    l.publica(&m, &pins);
    key_mark::set_key_marks_dos_presos(std::collections::BTreeMap::new());
    m.quadro();
    assert!(
        losangos(&m, k).is_empty(),
        "o corpo do preso leu o mapa da seleção"
    );
}

/// ⭐ Item 2: o seletor de um cartão sem alfinete fecha com um clique fora, como antes; o do preso fica
/// aberto e em A, com o clique no canvas a escolher outra forma.
/// *Mutações: a guarda de `picker_subject` fora ⇒ o 2.º caso reprova; sempre ⇒ o 1.º reprova.*
#[test]
fn an_unpinned_picker_closes_on_a_click_outside_and_a_pinned_one_stays_on_its_object() {
    let mut m = Mesa::nova();
    let l = Linha::nova();
    // O cartão do Fill na lista, sem alfinete, com B escolhido.
    m.escolhe(&[1]);
    l.quadros(&mut m, 3);
    let fora = {
        let c = m.h.last_layout.expect("um quadro").canvas;
        (c.x + c.w * 0.1, c.y + c.h * 0.9)
    };
    clica(&mut m.h, v::VECTOR_FILL_SWATCH);
    assert_eq!(m.h.store.picker_target(), Some(v::VECTOR_FILL_SWATCH));
    assert_eq!(m.h.store.picker_subject(), None);
    mao(&mut m.h, PointerKind::Down, fora);
    mao(&mut m.h, PointerKind::Up, fora);
    assert_eq!(
        m.h.store.picker_target(),
        None,
        "o seletor sem alfinete deixou de fechar com um clique fora"
    );

    // O cartão Fill solto SEM alfinete: é da seleção, fecha igual.
    let solto = {
        m.h.store
            .detach_card(CARD_FILL, INSP_PANEL, (640.0, 120.0), 300.0);
        l.quadros(&mut m, 3);
        CARD_FILL
    };
    m.clica_em(solto, v::VECTOR_FILL_SWATCH);
    assert_eq!(m.h.store.picker_target(), Some(v::VECTOR_FILL_SWATCH));
    assert_eq!(m.h.store.picker_subject(), None);
    mao(&mut m.h, PointerKind::Down, fora);
    mao(&mut m.h, PointerKind::Up, fora);
    assert_eq!(m.h.store.picker_target(), None, "o solto sem alfinete");

    // Preso a A: o clique fora (o canvas, que escolhe outra forma) deixa-o aberto, e a cor vai a A.
    m.escolhe(&[0]);
    l.quadros(&mut m, 2);
    m.h.store.toggle_loose_pin(solto);
    m.escolhe(&[1]);
    l.quadros(&mut m, 3);
    m.clica_em(solto, v::VECTOR_FILL_SWATCH);
    assert_eq!(m.h.store.picker_subject(), Some(m.bits[0]));
    mao(&mut m.h, PointerKind::Down, fora);
    mao(&mut m.h, PointerKind::Up, fora);
    m.escolhe(&[2]);
    l.quadros(&mut m, 2);
    assert_eq!(
        m.h.store.picker_target(),
        Some(v::VECTOR_FILL_SWATCH),
        "o seletor do preso fechou com o clique no canvas"
    );
    assert_eq!(m.h.store.picker_subject(), Some(m.bits[0]));
    let (b0, c0) = (m.forma(1), m.forma(2));
    m.escolhe_a_cor(NOVA);
    assert_eq!(cor_de_fill(&m.a()).r, NOVA[0], "a cor não foi a A");
    assert_eq!((m.forma(1), m.forma(2)), (b0, c0), "a cor saiu de A");
}

/// O atalho `X` do vetor abre o seletor da SELEÇÃO, mesmo a última pressão tendo sido num preso.
/// *Mutação: `open_picker_on` no `X` ⇒ o seletor fica do preso e não fecha com um clique fora.*
#[test]
fn the_x_shortcut_after_a_press_on_a_pinned_card_opens_the_selections_picker() {
    let mut m = Mesa::nova();
    let l = Linha::nova();
    let k = prende_aberto(&mut m, &l, CARD_STROKE, &[1]);
    let dentro = m.no_solto(k, v::VECTOR_WIDTH_NUM);
    mao(&mut m.h, PointerKind::Down, dentro);
    mao(&mut m.h, PointerKind::Up, dentro);
    assert_eq!(m.h.store.card_press_subject(), Some(m.bits[0]));
    m.h.store.set_focus(None);
    m.h.panel_visibility.insert("vector", true);
    l.quadros(&mut m, 1);
    m.h.apply_event(ph2d_editor_core::interaction::WidgetEvent::Click(
        v::VECTOR_COLOUR_FOCUS,
    ));
    l.quadros(&mut m, 1);
    assert!(
        m.h.store.picker_target().is_some(),
        "o X não abriu o seletor"
    );
    assert_eq!(
        m.h.store.picker_subject(),
        None,
        "o X abriu o seletor do preso"
    );
}

/// ⭐ Item 3: o «+» do estilo num cartão preso cria o estilo com a cor de A e liga só A; renomear,
/// duplicar e apagar são do estilo e não mexem nas tintas de B.
/// *Mutação: as formas do `Novo` = a seleção ⇒ B liga-se.*
#[test]
fn the_new_style_of_a_pinned_card_takes_its_objects_colour_and_links_only_it() {
    for sel in SELECOES {
        // Um fio por seleção: a lista aberta e o «Edit style» são estado do fio.
        std::thread::spawn(move || novo_estilo_com(sel))
            .join()
            .expect("o caso");
    }
}

fn novo_estilo_com(sel: &'static [usize]) {
    use ph2d_editor_core::panel::cards::estilos::Pedido;
    {
        let mut m = Mesa::nova();
        let k = m.prende(CARD_FILL, 0, sel);
        let _ = m.scene.add_estilo("Azul", Rgba8::new(1, 1, 200, 255));
        m.quadros(2);
        m.clica_em(k, estilos::botao_de(CARD_FILL));
        assert_eq!(estilos::sujeito_aberto(), Some(m.bits[0]));
        let mais =
            m.h.hit_index
                .rect_for(estilos::MAIS)
                .expect("o «+» da lista");
        let (b0, c0) = (m.forma(1), m.forma(2));
        mao(&mut m.h, PointerKind::Down, centro(mais));
        mao(&mut m.h, PointerKind::Up, centro(mais));
        m.quadros(3);
        let criado = m.scene.estilos().last().expect("o estilo novo").clone();
        m.h.store.set_focus(None);
        let a = m.a();
        let item = a.item_da_linha(0).expect("a linha de cima do Fill");
        assert_eq!(
            m.scene.estilo_de((m.ids[0], item)).map(|e| e.id),
            Some(criado.id),
            "A não ligou"
        );
        assert_eq!(
            criado.color,
            camada_cor(&a, 0),
            "{sel:?}: o estilo não nasceu com a cor de A"
        );
        for (i, f0) in [(1, &b0), (2, &c0)] {
            let f = m.forma(i);
            assert_eq!(&f, f0, "{sel:?}: {i} recebeu");
            let base = f.item_da_linha(LINHA_BASE_FILL).expect("a base");
            assert!(m.scene.estilo_de((m.ids[i], base)).is_none(), "{i} ligou");
        }
        // As acções do estilo, com o gesto no preso: mudam o estilo; as tintas de B ficam.
        let id = criado.id;
        estilos::pede(Pedido::Renomear {
            estilo: id,
            nome: "Novo nome".to_owned(),
        });
        m.quadros(1);
        assert!(m.scene.estilos().iter().any(|e| e.name == "Novo nome"));
        estilos::pede(Pedido::Duplicar {
            estilo: id,
            nome: "Cópia".to_owned(),
        });
        m.quadros(1);
        assert!(m.scene.estilos().iter().any(|e| e.name == "Cópia"));
        estilos::pede(Pedido::Apagar { estilo: id });
        m.quadros(1);
        assert!(m.scene.estilos().iter().all(|e| e.id != id));
        assert_eq!(
            (m.forma(1), m.forma(2)),
            (b0, c0),
            "{sel:?}: as acções mexeram em B/C"
        );
    }
}

/// Uma mesa da régua de custo: `formas` objectos, os cartões presos a `objectos` deles e a última forma
/// escolhida.
struct Variante {
    nome: &'static str,
    m: Mesa,
    l: Linha,
    pins: Vec<u64>,
    quadro: Vec<f64>,
    frio: Vec<f64>,
    mapas: Vec<f64>,
}

impl Variante {
    /// `objectos == 0`: sem presos.
    fn nova(nome: &'static str, formas: usize, objectos: usize) -> Self {
        let (mut m, l) = (Mesa::com_formas(formas), Linha::nova());
        if objectos > 0 {
            for (i, sec) in [
                CARD_FILL,
                CARD_STROKE,
                CARD_TRANSFORM,
                CARD_APPEARANCE,
                CARD_MATERIAL,
            ]
            .into_iter()
            .enumerate()
            {
                m.escolhe(&[i % objectos]);
                m.h.store.set_collapsed(sec, false);
                l.quadros(&mut m, 2);
                m.h.store
                    .detach_card(sec, INSP_PANEL, (500.0 + 40.0 * i as f32, 100.0), 300.0);
                l.quadros(&mut m, 2);
                m.h.store.toggle_loose_pin(sec);
            }
            assert_eq!(m.h.store.loose_cards_of(INSP_PANEL).len(), 5);
        }
        m.escolhe(&[formas - 1]);
        l.quadros(&mut m, 5);
        let pins = losango::pinos(&m.h.store);
        assert_eq!(pins.len(), objectos, "{nome}: objectos presos");
        Self {
            nome,
            m,
            l,
            pins,
            quadro: Vec::new(),
            frio: Vec::new(),
            mapas: Vec::new(),
        }
    }

    /// Um bloco: o 1.º quadro à parte (as mesas partilham as caches do painel, e a anterior sujou-as) e
    /// os outros, depois os mesmos mapas do losango, em ms por quadro.
    fn bloco(&mut self) {
        use std::time::Instant;
        const QUADROS: u32 = 6;
        let t = Instant::now();
        self.l.quadro(&mut self.m);
        self.frio.push(t.elapsed().as_secs_f64() * 1e3);
        let t = Instant::now();
        for _ in 1..QUADROS {
            self.l.quadro(&mut self.m);
        }
        self.quadro
            .push(t.elapsed().as_secs_f64() * 1e3 / f64::from(QUADROS - 1));
        let t = Instant::now();
        for _ in 0..QUADROS {
            self.l.publica(&self.m, &self.pins);
        }
        self.mapas
            .push(t.elapsed().as_secs_f64() * 1e3 / f64::from(QUADROS));
    }
}

/// O mínimo e a mediana (`docs/DevOps/MEDIR_VELOCIDADE.md`).
fn minimo_e_mediana(v: &[f64]) -> (f64, f64) {
    let mut o = v.to_vec();
    o.sort_by(f64::total_cmp);
    (o[0], o[o.len() / 2])
}

/// ⭐ O custo por quadro de 5 presos com as tintas no retrato, em 2 e em 5 objectos, cada um contra a
/// mesa do mesmo tamanho sem presos. Mesas no MESMO processo, blocos intercalados por ordem rodada, o
/// mínimo com a mediana ao lado. Resultado: `docs/CartoesUI/09`, «Custo por quadro».
/// `bash scripts/cargo-test-narrow.sh ph2d-panel-registry-init --profile smoke -- --ignored custo_por_quadro_de_cinco_presos --nocapture`.
#[test]
#[ignore = "mede: só em --profile smoke"]
fn custo_por_quadro_de_cinco_presos() {
    const RONDAS: usize = 60;
    let load = || std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    eprintln!(
        "[custo] debug_assertions {} · load {}",
        cfg!(debug_assertions),
        load()
    );
    let mut v = [
        Variante::nova("3 formas, sem presos", 3, 0),
        Variante::nova("3 formas, 5 cartões em 2 objectos", 3, 2),
        Variante::nova("6 formas, sem presos", 6, 0),
        Variante::nova("6 formas, 5 cartões em 5 objectos", 6, 5),
    ];
    for x in &mut v {
        x.bloco(); // aquece, fora da régua
        x.quadro.clear();
        x.frio.clear();
        x.mapas.clear();
    }
    for r in 0..RONDAS {
        for i in 0..v.len() {
            let n = v.len();
            v[(i + r) % n].bloco();
        }
    }
    let mut res = Vec::new();
    for x in &v {
        let (q, qm) = minimo_e_mediana(&x.quadro);
        let (l, lm) = minimo_e_mediana(&x.mapas);
        let (f, fm) = minimo_e_mediana(&x.frio);
        eprintln!(
            "[custo] {}: quadro {q:.3} (mediana {qm:.3}) · 1.º do bloco {f:.3} (mediana {fm:.3}) · mapas {l:.3} (mediana {lm:.3}) ms",
            x.nome
        );
        res.push((q, qm, l, lm));
    }
    for (nome, a, b) in [("2 objectos", 0, 1), ("5 objectos", 2, 3)] {
        eprintln!(
            "[custo] {nome}: +{:.3} ms no mínimo, +{:.3} na mediana; só os mapas +{:.3}, +{:.3}",
            res[b].0 - res[a].0,
            res[b].1 - res[a].1,
            res[b].2 - res[a].2,
            res[b].3 - res[a].3,
        );
    }
    eprintln!("[custo] load {}", load());
}

/// O alfinete não põe nem tira losangos: o mesmo cartão solto, antes e depois de preso, tem os mesmos.
/// Object, Material e Ordering não têm nenhum (0 = 0, medido); um campo novo neles entra na mesma lei.
#[test]
fn pinning_a_card_neither_adds_nor_removes_diamonds() {
    for sec in [
        CARD_TRANSFORM,
        CARD_OBJECT,
        CARD_APPEARANCE,
        CARD_ORDERING,
        CARD_MATERIAL,
        CARD_FILL,
        CARD_STROKE,
    ] {
        let mut m = Mesa::nova();
        let l = Linha::nova();
        m.escolhe(&[0]);
        m.h.store.set_collapsed(sec, false);
        l.quadros(&mut m, 2);
        m.h.store
            .detach_card(sec, INSP_PANEL, (640.0, 120.0), 300.0);
        l.quadros(&mut m, 2);
        let solto = losangos(&m, sec).len();
        m.h.store.toggle_loose_pin(sec);
        l.quadros(&mut m, 3);
        let preso = losangos(&m, sec).len();
        eprintln!("[c3] {sec:?}: {solto} solto, {preso} preso");
        assert_eq!(solto, preso, "{sec:?}");
    }
}

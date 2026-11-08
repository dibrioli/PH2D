use std::collections::BTreeMap;

use winit::keyboard::KeyCode as K;

use crate::*;

#[test]
fn a_fabrica_nao_toma_tecla_do_app() {
    for (a, k, _) in FACTORY {
        assert!(reserved(*k).is_none(), "{a:?} = {k} é do app");
    }
}

#[test]
fn a_fabrica_so_da_tecla_de_sentido_igual_ao_do_app() {
    for (c, want, _) in SAME_MEANING {
        for (a, k, x) in FACTORY {
            if k == c && *x == Context::Normal {
                assert_eq!(a, want, "{k}");
            }
        }
    }
}

#[test]
fn a_fabrica_nao_tem_duas_accoes_na_mesma_tecla_e_contexto() {
    let mut seen = BTreeMap::new();
    for (a, k, c) in FACTORY {
        let key = (k.to_string(), *c);
        if let Some(b) = seen.insert(key.clone(), *a) {
            panic!("{key:?}: {a:?} e {b:?}");
        }
    }
}

#[test]
fn cada_linha_do_aseprite_esta_na_fabrica_ou_e_excecao_do_dono() {
    for (a, k) in ASEPRITE {
        let bound = FACTORY.iter().any(|(x, y, _)| x == a && y == k);
        let exc = OWNER_EXCEPTIONS.iter().any(|(x, y, _)| x == a && y == k);
        assert!(bound ^ exc, "{a:?} {k}: na fábrica={bound}, excepção={exc}");
    }
}

#[test]
fn o_texto_de_cada_tecla_de_fabrica_ida_e_volta() {
    for k in FACTORY
        .iter()
        .map(|(_, k, _)| k)
        .chain(RESERVED_GLOBAL.iter().map(|(k, _)| k))
    {
        let s = k.to_string();
        assert_eq!(s.parse::<Chord>().as_ref(), Ok(k), "{s}");
    }
    assert_eq!(
        "ctrl+shift+x".parse::<Chord>(),
        Ok(Chord::new(K::KeyX).ctrl().shift())
    );
    assert!("Ctrl+Nope".parse::<Chord>().is_err());
    assert!("Hyper+A".parse::<Chord>().is_err());
}

#[test]
fn cada_accao_tem_nome_em_ingles_e_id_unico() {
    let mut ids = std::collections::BTreeSet::new();
    for a in PixelAction::ALL {
        assert!(ids.insert(a.id()), "id repetido {}", a.id());
        assert_ne!(a.name(), a.name_key(), "sem tradução: {}", a.name_key());
        assert_eq!(PixelAction::from_id(a.id()), Some(*a));
    }
    for (_, n) in RESERVED_GLOBAL {
        assert_ne!(ph2d_i18n::tr(n), *n);
    }
    for (_, _, n) in SAME_MEANING {
        assert_ne!(ph2d_i18n::tr(n), *n);
    }
    for c in Context::ALL {
        assert_ne!(ph2d_i18n::tr(c.name_key()), c.name_key());
    }
}

#[test]
fn o_delta_so_grava_o_mudado_e_rele_igual() {
    let f = Keymap::factory();
    assert_eq!(f.to_text(), "");
    let mut m = f.clone();
    m.assign(
        PixelAction::Transform,
        Chord::new(K::KeyT).shift(),
        Context::Normal,
    )
    .unwrap();
    m.remove(PixelAction::Fill, Chord::new(K::KeyF));
    m.assign(
        PixelAction::LoopSection,
        Chord::new(K::KeyL).alt(),
        Context::Selection,
    )
    .unwrap();
    let t = m.to_text();
    assert_eq!(t.lines().count(), 3, "{t}");
    assert!(t.contains("edit.fill = -"), "{t}");
    assert!(t.contains("frame.loop_section = Alt+L @Selection"), "{t}");
    let (back, rep) = Keymap::from_text(&t);
    assert_eq!(rep, TextReport::default());
    assert_eq!(back.to_text(), t);
    for a in PixelAction::ALL {
        let mut x = back.keys_of(*a);
        let mut y = m.keys_of(*a);
        x.sort_by_key(|(k, c)| (k.to_string(), *c));
        y.sort_by_key(|(k, c)| (k.to_string(), *c));
        assert_eq!(x, y, "{a:?}");
    }
}

#[test]
fn o_delta_salta_id_desconhecido_e_tecla_do_app() {
    let (m, rep) = Keymap::from_text("tool.zoom = Z\nedit.fill = Ctrl+K, Shift+F\nlixo\n");
    assert_eq!(rep.unknown_ids, vec!["tool.zoom".to_owned()]);
    assert_eq!(rep.rejected, vec!["Ctrl+K".to_owned(), "lixo".to_owned()]);
    assert_eq!(
        m.keys_of(PixelAction::Fill),
        vec![(Chord::new(K::KeyF).shift(), Context::Normal)]
    );
}

#[test]
fn tecla_do_app_e_recusada_e_nada_muda() {
    let mut m = Keymap::factory();
    let before = m.clone();
    for (k, n) in RESERVED_GLOBAL {
        assert_eq!(
            m.assign(PixelAction::Fill, *k, Context::Normal),
            Err(Conflict::Reserved(n))
        );
        assert_eq!(
            m.assign_replacing(PixelAction::Fill, *k, Context::Selection),
            Err(Conflict::Reserved(n))
        );
    }
    assert!(matches!(
        m.assign(PixelAction::Fill, Chord::new(K::F2), Context::Normal),
        Err(Conflict::Reserved(_))
    ));
    assert_eq!(m, before);
}

#[test]
fn tecla_ocupada_responde_taken_e_nao_rouba() {
    let mut m = Keymap::factory();
    let before = m.clone();
    let s = Chord::new(K::KeyS);
    assert_eq!(
        m.assign(PixelAction::Fill, s, Context::Normal),
        Err(Conflict::Taken(PixelAction::Stroke))
    );
    assert_eq!(m, before);
    // Outro contexto não é conflito (O7).
    assert!(m.assign(PixelAction::Fill, s, Context::Selection).is_ok());
    assert_eq!(m.resolve(s, Context::Selection), Some(PixelAction::Fill));
    assert_eq!(m.resolve(s, Context::Normal), Some(PixelAction::Stroke));
    m.assign_replacing(PixelAction::Fill, s, Context::Normal)
        .unwrap();
    assert_eq!(m.resolve(s, Context::Normal), Some(PixelAction::Fill));
    assert!(m.keys_of(PixelAction::Stroke).is_empty());
}

#[test]
fn repor_uma_accao_so_mexe_nela() {
    let mut m = Keymap::factory();
    m.assign_replacing(PixelAction::Fill, Chord::new(K::KeyS), Context::Normal)
        .unwrap();
    m.remove(PixelAction::Transform, Chord::new(K::KeyT).ctrl());
    m.reset(PixelAction::Stroke);
    assert_eq!(
        m.resolve(Chord::new(K::KeyS), Context::Normal),
        Some(PixelAction::Stroke)
    );
    assert_eq!(
        m.keys_of(PixelAction::Fill),
        vec![(Chord::new(K::KeyF), Context::Normal)]
    );
    assert!(m.keys_of(PixelAction::Transform).is_empty());
    m.reset_all();
    assert_eq!(m, Keymap::factory());
}

#[test]
fn o_contexto_decide_e_cai_no_normal() {
    let m = Keymap::factory();
    let esc = Chord::new(K::Escape);
    let enter = Chord::new(K::Enter);
    assert_eq!(m.resolve(esc, Context::Normal), None);
    assert_eq!(
        m.resolve(esc, Context::Selection),
        Some(PixelAction::ClearSelection)
    );
    assert_eq!(
        m.resolve(esc, Context::Transformation),
        Some(PixelAction::CancelTransform)
    );
    assert_eq!(
        m.resolve(esc, Context::Stroking),
        Some(PixelAction::CancelStroke)
    );
    assert_eq!(m.resolve(enter, Context::Normal), Some(PixelAction::Play));
    assert_eq!(
        m.resolve(enter, Context::Transformation),
        Some(PixelAction::CommitTransform)
    );
    let f = Chord::new(K::KeyF);
    assert_eq!(m.resolve(f, Context::Selection), Some(PixelAction::Fill));
    assert_eq!(m.resolve(f, Context::Stroking), None);
    assert_eq!(
        m.resolve(Chord::new(K::KeyB), Context::Stroking),
        Some(PixelAction::ToolPencil)
    );
    assert_eq!(m.resolve(Chord::new(K::KeyG), Context::Normal), None);
}

#[test]
fn a_busca_flp_poe_o_flip_primeiro() {
    let r = search("flp");
    assert!(!r.is_empty());
    assert!(r[0].0.name().starts_with("Flip"), "{:?}", r[0].0.name());
    assert!(search("zzzq").is_empty());
    assert_eq!(search("pencil")[0].0, PixelAction::ToolPencil);
}

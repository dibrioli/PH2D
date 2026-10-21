//! Gates do [`super`] (`canais_interface.rs`) — âncoras, recorte, auto layout, pele do widget.

use super::*;

fn k(campo: u16, j: u16) -> FieldKey {
    FieldKey::new(VETOR, campo, 0, j)
}

/// Uma moldura com um filho.
fn moldura() -> (SimWorld, Entity, Entity) {
    let mut sim = SimWorld::new();
    let pai = sim.world_mut().spawn(VecFrame).id();
    let filho = sim.world_mut().spawn(ChildOf(pai)).id();
    (sim, pai, filho)
}

fn publicados(sim: &SimWorld, e: Entity, fechado: bool) -> Vec<FieldKey> {
    let mut out = Vec::new();
    campos(sim, e, fechado, &mut out);
    out
}

/// Escrever o que se leu, em TODO campo publicado, não muda o valor (a régua do eco).
fn identidade(sim: &mut SimWorld, e: Entity, fechado: bool) {
    let mut t = TextosDosCanais::default();
    for kk in publicados(sim, e, fechado) {
        assert!(e_da_interface(kk), "{kk:?}");
        let v = ler(sim, e, kk, &mut t).expect("publicado ⇒ legível");
        escrever(sim, e, kk, v, &t);
        assert_eq!(ler(sim, e, kk, &mut t), Some(v), "{kk:?}");
    }
}

/// ⭐ As âncoras: o filho de uma moldura publica o «existe» (o interruptor) mesmo sem elas;
/// ligá-lo arma, e a base e as fracções da curva aplicam-se a seguir — sem arredondar o `0,5`.
#[test]
fn as_ancoras_armam_no_filho_da_moldura_e_nao_arredondam_a_fracao() {
    let (mut sim, pai, filho) = moldura();
    let mut t = TextosDosCanais::default();
    assert_eq!(
        publicados(&sim, filho, false),
        vec![k(c::ANCORAS_EXISTE, 0)]
    );
    assert!(
        !publicados(&sim, pai, false).contains(&k(c::ANCORAS_EXISTE, 0)),
        "a moldura raiz não tem a quem se ancorar"
    );
    escrever(&mut sim, filho, k(c::ANCORA_MIN_X, 0), 0.5, &t);
    assert!(
        sim.world().get::<VecAnchors>(filho).is_none(),
        "ausente fica ausente"
    );
    escrever(&mut sim, filho, k(c::ANCORAS_EXISTE, 0), 1.0, &t);
    for (campo, v) in [
        (c::ANCORA_MIN_X, 0.5),
        (c::ANCORA_MAX_X, 0.5),
        (c::ANCORA_MAX_Y, 1.0),
        (c::ANCORA_BASE + 2, 120.0),
    ] {
        escrever(&mut sim, filho, k(campo, 0), v, &t);
        assert_eq!(
            ler(&sim, filho, k(campo, 0), &mut t),
            Some(v),
            "campo {campo}"
        );
    }
    escrever(&mut sim, filho, k(c::ANCORA_MIN_Y, 0), 3.0, &t);
    assert_eq!(sim.world().get::<VecAnchors>(filho).unwrap().min[1], 1.0);
    identidade(&mut sim, filho, false);
    escrever(&mut sim, filho, k(c::ANCORAS_EXISTE, 0), 0.0, &t);
    assert!(sim.world().get::<VecAnchors>(filho).is_none());
}

/// O recorte: só uma forma fechada publica o interruptor; ligar e desligar põem e tiram o marcador.
#[test]
fn o_recorte_e_o_marcador() {
    let (mut sim, pai, _) = moldura();
    let t = TextosDosCanais::default();
    assert!(!publicados(&sim, pai, false).contains(&k(c::MOLDURA_RECORTA, 0)));
    assert!(publicados(&sim, pai, true).contains(&k(c::MOLDURA_RECORTA, 0)));
    escrever(&mut sim, pai, k(c::MOLDURA_RECORTA, 0), 1.0, &t);
    assert!(sim.world().get::<VecClipContent>(pai).is_some());
    escrever(&mut sim, pai, k(c::MOLDURA_RECORTA, 0), 0.0, &t);
    assert!(sim.world().get::<VecClipContent>(pai).is_none());
}

/// ⭐ O auto layout: a direcção «Off» é a ausência do componente; os números só existem com ele;
/// o filho do fluxo e os limites são neutros pela AUSÊNCIA (o idioma do `layout_edit`).
#[test]
fn o_auto_layout_liga_pela_direcao_e_o_neutro_e_a_ausencia() {
    let (mut sim, pai, filho) = moldura();
    let mut t = TextosDosCanais::default();
    assert_eq!(ler(&sim, pai, k(c::LAYOUT_DIR, 0), &mut t), Some(0.0));
    assert!(!publicados(&sim, pai, false).contains(&k(c::LAYOUT_VAO_X, 0)));
    escrever(&mut sim, pai, k(c::LAYOUT_VAO_X, 0), 8.0, &t);
    assert!(
        sim.world().get::<VecLayout>(pai).is_none(),
        "ausente fica ausente"
    );
    escrever(&mut sim, pai, k(c::LAYOUT_DIR, 0), 4.0, &t);
    assert_eq!(
        sim.world().get::<VecLayout>(pai).unwrap().dir,
        LayoutDir::Grid
    );
    for (campo, v) in [
        (c::LAYOUT_VAO_X, 8.0),
        (c::LAYOUT_VAO_Y, 2.0),
        (c::LAYOUT_MARGEM + 3, 6.0),
        (c::LAYOUT_ALINHA, 3.0),
        (c::LAYOUT_DISTRIBUI, 4.0),
        (c::LAYOUT_COLUNAS, 3.0),
    ] {
        escrever(&mut sim, pai, k(campo, 0), v, &t);
        assert_eq!(
            ler(&sim, pai, k(campo, 0), &mut t),
            Some(v),
            "campo {campo}"
        );
    }
    escrever(&mut sim, pai, k(c::LAYOUT_COLUNAS, 0), 0.0, &t);
    assert_eq!(sim.world().get::<VecLayout>(pai).unwrap().columns, 1);
    // Os limites: o «tem» liga, o número vem depois; desligar tudo devolve a ausência.
    escrever(&mut sim, pai, k(c::LAYOUT_TEM_MIN, 1), 1.0, &t);
    escrever(&mut sim, pai, k(c::LAYOUT_MIN, 1), 40.0, &t);
    escrever(&mut sim, pai, k(c::LAYOUT_TAMANHO, 0), 1.0, &t);
    let s = *sim.world().get::<VecLayoutSize>(pai).unwrap();
    assert_eq!((s.min[1], s.size[0]), (Some(40.0), LayoutSize::Hug));
    identidade(&mut sim, pai, false);
    escrever(&mut sim, pai, k(c::LAYOUT_TEM_MIN, 1), 0.0, &t);
    escrever(&mut sim, pai, k(c::LAYOUT_TAMANHO, 0), 0.0, &t);
    assert!(
        sim.world().get::<VecLayoutSize>(pai).is_none(),
        "neutro = ausente"
    );
    // O filho do fluxo.
    for campo in [c::LAYOUT_CRESCE, c::LAYOUT_ABSOLUTO] {
        assert!(publicados(&sim, filho, false).contains(&k(campo, 0)));
    }
    escrever(&mut sim, filho, k(c::LAYOUT_CRESCE, 0), 2.0, &t);
    escrever(&mut sim, filho, k(c::LAYOUT_ABSOLUTO, 0), 1.0, &t);
    assert_eq!(sim.world().get::<VecLayoutItem>(filho).unwrap().grow, 2.0);
    assert!(sim.world().get::<VecLayoutAbsolute>(filho).is_some());
    identidade(&mut sim, filho, false);
    escrever(&mut sim, filho, k(c::LAYOUT_CRESCE, 0), 0.0, &t);
    assert!(
        sim.world().get::<VecLayoutItem>(filho).is_none(),
        "neutro = ausente"
    );
    escrever(&mut sim, pai, k(c::LAYOUT_DIR, 0), 0.0, &t);
    assert!(sim.world().get::<VecLayout>(pai).is_none());
}

/// A pele do widget: só um código que o app conhece se escreve; o ícone é um TEXTO (o índice na
/// tabela), `0` tira-o, e um ícone que o app não tem não se escreve.
#[test]
fn a_pele_do_widget_e_o_icone() {
    let mut sim = SimWorld::new();
    let e = sim.world_mut().spawn(()).id();
    let mut t = TextosDosCanais::default();
    assert!(
        publicados(&sim, e, true)
            .iter()
            .all(|kk| kk.campo != c::WIDGET_PELE)
    );
    let botao = WidgetKind::Button.code();
    sim.world_mut()
        .entity_mut(e)
        .insert(VecWidget { kind: botao });
    let outra = WidgetKind::ALL
        .iter()
        .map(|k| k.code())
        .find(|&x| x != botao)
        .unwrap();
    escrever(&mut sim, e, k(c::WIDGET_PELE, 0), f32::from(outra), &t);
    assert_eq!(sim.world().get::<VecWidget>(e).unwrap().kind, outra);
    let desconhecida = (0..u16::MAX)
        .find(|&x| WidgetKind::from_code(x).is_none())
        .unwrap();
    escrever(
        &mut sim,
        e,
        k(c::WIDGET_PELE, 0),
        f32::from(desconhecida),
        &t,
    );
    assert_eq!(sim.world().get::<VecWidget>(e).unwrap().kind, outra);

    let slug = IconId::all()[0].slug();
    let i = t.indice(Some(slug));
    let lixo = t.indice(Some("não-é-um-ícone"));
    escrever(&mut sim, e, k(c::WIDGET_ICONE, 0), i, &t);
    assert_eq!(sim.world().get::<VecWidgetIcon>(e).unwrap().slug, slug);
    assert_eq!(ler(&sim, e, k(c::WIDGET_ICONE, 0), &mut t), Some(i));
    escrever(&mut sim, e, k(c::WIDGET_ICONE, 0), lixo, &t);
    assert_eq!(sim.world().get::<VecWidgetIcon>(e).unwrap().slug, slug);
    identidade(&mut sim, e, false);
    escrever(&mut sim, e, k(c::WIDGET_ICONE, 0), 0.0, &t);
    assert!(sim.world().get::<VecWidgetIcon>(e).is_none());
}

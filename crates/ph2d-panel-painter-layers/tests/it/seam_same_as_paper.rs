//! **«Same as Paper» esconde o grão do pincel** (decisão do dono, 22/10). Só a Aquarela o tem: com ele
//! ligado a lavagem assenta no dente do PAPEL, e nada do slot Grain do pincel chega à tinta — a textura,
//! a miniatura, Mapping, Angle, Offset, Size, Depth, os parâmetros do padrão, Grain Behavior e Grain
//! Colors (medido pela sonda abaixo: `0` texels por controlo). Ficam o próprio interruptor e o Amount.
//!
//! O caso é o da captura do dono: Voronoi + Grain Colors escolhidos, DEPOIS «Same as Paper» marcado.

use super::censo_dos_controlos::{
    Bancada, Entrega, Gesto, diferenca, ensaio, gestos_da, nome, nomes,
};
use ph2d_a11y::NodeId;
use ph2d_editor_core::zones::Rect;
use ph2d_tool_painter::PaintMedia;
use ph2d_tool_painter::ids::*;

const MEIO: PaintMedia = PaintMedia::Watercolor;
const TAMANHO: f64 = 40.0;

fn voronoi() -> Gesto {
    Gesto::Opcao(
        PAINTER_BRUSH_TEXTURE_KIND,
        painter_brush_texture_kind_option_id(3),
        0,
        0,
    )
}

/// «Same as Paper» desligado (a fábrica o tem ligado), o Voronoi, as Grain Colors e um Size do artista.
fn grao_do_artista() -> Vec<Gesto> {
    vec![
        Gesto::Clique(PAINTER_WATERCOLOR_GRAN_SAME),
        voronoi(),
        Gesto::Clique(PAINTER_BRUSH_TEXTURE_RAMP_ENABLE),
        Gesto::Numero(PAINTER_BRUSH_TEXTURE_SIZE_X, TAMANHO),
    ]
}

/// …e depois «Same as Paper» marcado de novo.
fn com_o_papel() -> Vec<Gesto> {
    let mut v = grao_do_artista();
    v.push(Gesto::Clique(PAINTER_WATERCOLOR_GRAN_SAME));
    v
}

fn ids(r: &[(NodeId, Rect)]) -> Vec<NodeId> {
    r.iter().map(|(i, _)| *i).collect()
}

/// As linhas do grão PRÓPRIO, derivadas da tela: o que o Voronoi e as Grain Colors fizeram aparecer (contra
/// o mesmo estado sem textura) e o menu da textura.
fn linhas_do_grao_proprio() -> Vec<(NodeId, Rect)> {
    let sem = ids(Bancada::nova(MEIO, &grao_do_artista()[..1]).rects());
    let com = Bancada::nova(MEIO, &grao_do_artista());
    let v: Vec<(NodeId, Rect)> = com
        .rects()
        .iter()
        .filter(|(i, _)| !sem.contains(i) || *i == PAINTER_BRUSH_TEXTURE_KIND)
        .copied()
        .collect();
    for obrigatorio in [
        PAINTER_BRUSH_TEXTURE_KIND,
        PAINTER_BRUSH_TEXTURE_MAPPING,
        PAINTER_BRUSH_TEXTURE_ANGLE,
        PAINTER_BRUSH_TEXTURE_SIZE_X,
        PAINTER_BRUSH_GRAIN_DEPTH,
        PAINTER_BRUSH_GRAIN_BEHAVIOR_SECTION,
        PAINTER_BRUSH_TEXTURE_PARAMS[0],
        PAINTER_BRUSH_TEXTURE_RAMP_ENABLE,
    ] {
        assert!(
            v.iter().any(|(i, _)| *i == obrigatorio),
            "a população não tem {} — a fixture não armou o grão",
            nome(&nomes(), obrigatorio)
        );
    }
    v
}

/// (i) + (iv): com «Same as Paper» ligado nenhuma linha do grão próprio é pintada nem alcançável pelo
/// clique; o interruptor e o Amount ficam; o corpo abaixo sobe (a miniatura saiu com elas).
#[test]
fn same_as_paper_ligado_esconde_o_grao_do_pincel() {
    let nomes = nomes();
    let proprias = linhas_do_grao_proprio();
    let mut b = Bancada::nova(MEIO, &com_o_papel());
    let s = b.tool.brush_settings();
    assert!(s.granulation_use_paper && s.watercolor_active);
    let pintadas = ids(b.rects());
    let sangram: Vec<String> = proprias
        .iter()
        .filter(|(i, _)| pintadas.contains(i))
        .map(|(i, _)| nome(&nomes, *i))
        .collect();
    assert!(
        sangram.is_empty(),
        "com Same as Paper ligado ainda pintadas: {sangram:?}"
    );
    for fica in [PAINTER_WATERCOLOR_GRAN_SAME, PAINTER_WATERCOLOR_GRANULATION] {
        assert!(pintadas.contains(&fica), "{} sumiu", nome(&nomes, fica));
    }
    // A secção Grain fica do tamanho da da fábrica (só o interruptor e o Amount): a miniatura saiu também.
    let fabrica = Bancada::nova(MEIO, &[]);
    let abaixo = |r: &[(NodeId, Rect)]| {
        r.iter()
            .find(|(i, _)| *i == PAINTER_BRUSH_STROKE_SECTION)
            .map(|(_, r)| r.y)
            .expect("a secção Stroke foi pintada")
    };
    assert_eq!(
        abaixo(b.rects()),
        abaixo(fabrica.rects()),
        "a secção Grain não encolheu"
    );
    // (iv) o clique onde as linhas estavam não chega a nenhuma delas, nem mexe no grão do pincel.
    let antes = grao(&b.tool.brush_settings());
    for (id, r) in &proprias {
        let ev = b.host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5);
        let alvo = format!("{id:?}");
        assert!(
            !ev.iter().any(|e| format!("{e:?}").contains(&alvo)),
            "o clique onde estava {} chegou-lhe: {ev:?}",
            nome(&nomes, *id)
        );
    }
    b.aplica(&Gesto::Nenhum, &mut Entrega::default());
    assert_eq!(
        antes,
        grao(&b.tool.brush_settings()),
        "um clique no vazio mexeu no grão do pincel"
    );
}

/// Os valores do grão próprio no instantâneo do pincel.
fn grao(s: &ph2d_tool_painter::BrushSettings) -> String {
    format!(
        "{} {:?} {:?} {} {} {:?} {} {} {:?}",
        s.texture_kind,
        s.texture_size,
        s.texture_offset,
        s.texture_angle_deg,
        s.texture_mapping,
        s.texture_params,
        s.grain_depth,
        s.texture_ramp_enabled,
        s.texture_ramp_stops
    )
}

/// (ii): desmarcar volta a mostrar as linhas, com os valores que tinham.
#[test]
fn desmarcar_same_as_paper_devolve_as_linhas_com_os_valores() {
    let mut v = com_o_papel();
    v.push(Gesto::Clique(PAINTER_WATERCOLOR_GRAN_SAME));
    let b = Bancada::nova(MEIO, &v);
    let pintadas = ids(b.rects());
    let faltam: Vec<NodeId> = ids(&linhas_do_grao_proprio())
        .into_iter()
        .filter(|i| !pintadas.contains(i))
        .collect();
    assert!(faltam.is_empty(), "não voltaram: {faltam:?}");
    let s = b.tool.brush_settings();
    assert!(!s.granulation_use_paper);
    assert_eq!(s.texture_kind, 3, "a textura escolhida perdeu-se");
    assert!(s.texture_ramp_enabled, "as Grain Colors perderam-se");
    assert!(
        (f64::from(s.texture_size[0]) - TAMANHO).abs() < 1e-3,
        "o Size perdeu-se: {}",
        s.texture_size[0]
    );
    assert_eq!(
        b.host.store().number_value(PAINTER_BRUSH_TEXTURE_SIZE_X),
        Some(TAMANHO),
        "o Size não volta ao chip"
    );
}

/// (iii): com ele ligado a tinta é a de sem grão próprio (o papel manda).
#[test]
fn com_same_as_paper_a_tinta_e_a_do_papel() {
    let (com, _) = ensaio(MEIO, &com_o_papel(), &Gesto::Nenhum);
    let (papel, _) = ensaio(MEIO, &[], &Gesto::Nenhum);
    assert_eq!(
        diferenca(&com, &papel).0,
        0,
        "o grão do pincel pintou com Same as Paper ligado"
    );
    // O controlo: com ele desligado o mesmo grão MUDA a tinta (a igualdade acima não é vácua).
    let (sem, _) = ensaio(MEIO, &grao_do_artista(), &Gesto::Nenhum);
    assert!(
        diferenca(&sem, &papel).0 > 0,
        "o grão do artista não pinta nem desligado"
    );
}

/// A medição por trás da lei: cada controlo do grão próprio mexido com «Same as Paper» desligado e depois o
/// interruptor ligado — a tinta tem de ser a do papel. `SAP … n=<texels>`; medido 22/10: `0` em todos.
#[test]
#[ignore = "sonda: cada controlo do grão próprio com Same as Paper ligado"]
fn sonda_cada_controlo_do_grao_com_same_as_paper() {
    let nomes = nomes();
    let proprias = ids(&linhas_do_grao_proprio());
    let (papel, _) = ensaio(MEIO, &com_o_papel(), &Gesto::Nenhum);
    for g in gestos_da(Bancada::nova(MEIO, &grao_do_artista())) {
        let dono = match &g {
            Gesto::Opcao(menu, ..) => Some(*menu),
            _ => g.id(),
        };
        if !dono.is_some_and(|i| proprias.contains(&i)) {
            continue;
        }
        let mut armar = grao_do_artista();
        armar.push(g.clone());
        armar.push(Gesto::Clique(PAINTER_WATERCOLOR_GRAN_SAME));
        let r = std::panic::catch_unwind(|| ensaio(MEIO, &armar, &Gesto::Nenhum));
        let n = r.map_or(usize::MAX, |(px, _)| diferenca(&papel, &px).0);
        eprintln!(
            "SAP\t{}\t{g:?}\tn={n}",
            nome(&nomes, dono.unwrap_or(NodeId(0)))
        );
    }
}

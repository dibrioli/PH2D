//! ⭐ **O vocabulário de cada escolha do vetor** (F28, [plano](../../../docs/UI_New_and_Simple/F28_plano.md)): o menu,
//! a palavra da dope-sheet e o documento dizem a MESMA escolha, e nada fora da lista chega ao documento.

use crate::canais_campos::{campos, escrever, ler};
use ph2d_ecs::canal::familia::VETOR;
use ph2d_ecs::canal_palavra::prende;
use ph2d_ecs::canal_vetor as c;
use ph2d_ecs::{Especie, FieldKey, VecShape, VecTextParams};
use ph2d_vec_scene::paint_stack::{PaintEntry, PaintKind};
use ph2d_vec_scene::{
    BlendMode, MAX_BLEND_MODES, Paint, PatternFill, PatternSource, Rgba8, StrokePaint, StrokeSpec,
    VecPath, VecVertex,
};

fn k(campo: u16) -> FieldKey {
    FieldKey::new(VETOR, campo, 0, 0)
}

fn lista(campo: u16) -> &'static [&'static str] {
    match c::info(campo).map(|i| i.especie) {
        Some(Especie::Opcoes(l)) => l,
        e => panic!("o campo {campo} não é uma escolha: {e:?}"),
    }
}

/// ⭐ **Cada linha do menu de mistura escreve um código cuja palavra é a da linha**, e um código que o
/// menu não oferece (o Vello não o desenha) não tem palavra nem se escreve: «Linear Burn» na dope-sheet
/// com «Normal» no chip e no desenho era a palavra a mentir.
///
/// Mutação que deve sangrar: `MISTURA`/`PILHA_MISTURA` voltam a `op(MISTURAS)`.
#[test]
fn o_menu_de_mistura_e_a_palavra_dizem_o_mesmo_modo() {
    let oferecidos: Vec<u8> = ph2d_vec_render::blend::offered()
        .map(BlendMode::to_u8)
        .collect();
    for campo in [c::MISTURA, c::PILHA_MISTURA] {
        let palavras = lista(campo);
        assert_eq!(palavras.len(), usize::from(MAX_BLEND_MODES));
        for (linha, modo) in ph2d_vec_render::blend::offered().enumerate() {
            assert_eq!(
                palavras[usize::from(modo.to_u8())],
                modo.label_key(),
                "a linha {linha} do menu escreve {} e a dope-sheet diz outra palavra",
                modo.to_u8()
            );
        }
        for codigo in 0..MAX_BLEND_MODES {
            let oferecido = oferecidos.contains(&codigo);
            assert_eq!(
                !palavras[usize::from(codigo)].is_empty(),
                oferecido,
                "o código {codigo}: a palavra e o menu discordam sobre se ele existe"
            );
            let preso = prende(k(campo), f32::from(codigo));
            assert!(
                oferecidos.iter().any(|&o| f32::from(o) == preso),
                "o código {codigo} preso em {preso}, que o menu não tem"
            );
        }
    }
}

/// ⭐ **Alinhamento e junção da pilha só guardam as três escolhas** — o escritor recebia `0..=255`.
///
/// Mutação que deve sangrar: tirar o `prende` do início do `canais_campos::escrever`.
#[test]
fn alinhamento_e_juncao_so_aceitam_as_tres_escolhas() {
    let mut p = VecPath {
        verts: vec![VecVertex::corner([0.0, 0.0]), VecVertex::corner([9.0, 0.0])],
        paints: vec![PaintEntry {
            id: ph2d_ecs::ItemId(3),
            ..PaintEntry::new(PaintKind::Fill(Paint::Solid(Rgba8::new(0, 0, 0, 255))))
        }],
        ..VecPath::default()
    };
    let mut texto = VecShape::Text(VecTextParams {
        text: "Olá".to_string(),
        origin: [0.0, 0.0],
        family: None,
        size: 24.0,
        weight: 400.0,
        line_height: 1.2,
        tracking: 0.0,
        align: 0,
        axes: Vec::new(),
        wrap_width: None,
    });
    escrever(&mut p, Some(&mut texto), k(c::TEXTO_ALINHAMENTO), 200.0);
    let VecShape::Text(t) = &texto else { panic!() };
    assert_eq!(t.align, 2, "200 num alinhamento é «Right»");
    let juncao = FieldKey::do_item(VETOR, c::PILHA_DILATA_JUNCAO, ph2d_ecs::ItemId(3), 0);
    escrever(&mut p, None, juncao, 200.0);
    assert_eq!(p.paints[0].dilate_join, 2, "200 na junção é «Bevel»");
    assert_eq!(ler(&p, None, juncao), Some(2.0));
}

/// ⭐ **O traço só declara os campos que algum traço tem** — gradientes e multiponto são do
/// preenchimento: declarados no traço, eram canais sem leitor.
///
/// Mutação que deve sangrar: `tinta()` volta a declarar `N_PARAGENS` para o traço.
#[test]
fn o_traco_so_declara_o_que_tem() {
    let declarados: Vec<u16> = (0..c::TINTA_LARGURA)
        .filter(|&o| c::info(c::TINTA_TRACO + o).is_some())
        .collect();
    let padrao = PatternFill::new(
        PatternSource::Shape(9),
        [4.0, 4.0],
        Rgba8::new(1, 2, 3, 255),
    );
    let mut lidos: Vec<u16> = Vec::new();
    for tinta in [
        StrokePaint::Solid(Rgba8::new(0, 0, 0, 255)),
        StrokePaint::Pattern(Box::new(padrao)),
        StrokePaint::Brush(Box::default()),
    ] {
        let mut s = StrokeSpec::new(Rgba8::new(0, 0, 0, 255), 2.0);
        s.paint = tinta;
        let p = VecPath {
            verts: vec![VecVertex::corner([0.0, 0.0]), VecVertex::corner([9.0, 0.0])],
            stroke: Some(s),
            ..VecPath::default()
        };
        for kk in campos(&p, None) {
            let o = kk.campo.wrapping_sub(c::TINTA_TRACO);
            if kk.i == 0 && o < c::TINTA_LARGURA && !lidos.contains(&o) {
                assert!(
                    ler(&p, None, kk).is_some(),
                    "o deslocamento {o} publicado sem valor"
                );
                lidos.push(o);
            }
        }
    }
    lidos.sort_unstable();
    assert_eq!(
        declarados, lidos,
        "o traço declara o que nenhum traço lê (ou o contrário)"
    );
}

/// ⭐ **Os presets do Envelope: o botão do painel e a palavra da dope-sheet são a mesma chave** — três
/// listas (o rótulo do motor, a ponte do painel e o vocabulário do canal) que só concordavam por acaso.
///
/// Mutação que deve sangrar: tirar `"Rise"` do `nomes_do_motor::chave_do_efeito`.
#[test]
fn os_presets_do_envelope_dizem_a_palavra_do_canal() {
    let palavras = lista(c::ENVELOPE_WARP);
    let todos = ph2d_ecs::EnvelopeWarp::ALL;
    assert_eq!(palavras.len(), todos.len() + 1, "o 0 é «nenhum»");
    for (i, w) in todos.iter().enumerate() {
        assert_eq!(
            ph2d_panel_vector::nomes_do_motor::chave_do_efeito(w.label()),
            Some(palavras[i + 1]),
            "o preset {} pinta-se cru no painel ou diz outra palavra na dope-sheet",
            w.label()
        );
    }
}

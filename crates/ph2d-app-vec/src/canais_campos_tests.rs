use super::*;
use ph2d_ecs::VecTextParams;
use ph2d_vec_scene::effect::{FxEntry, PathEffect};
use ph2d_vec_scene::fx_trim::TrimSpec;
use ph2d_vec_scene::paint_stack::PaintEntry;

/// Uma forma NÃO alinhada aos eixos (alça assimétrica num vértice, canto arredondado noutro), com
/// gradiente de 3 paragens, traço tracejado de 2 pares, uma entrada na pilha e um efeito.
fn forma() -> VecPath {
    let mut torta = VecVertex::smooth([10.0, 3.0], [4.0, -2.0], [17.0, 9.5]);
    torta.kind = VertexKind::Symmetric;
    let mut canto = VecVertex::corner([30.0, 25.0]);
    canto.corner_radius = 4.5;
    let mut traco = StrokeSpec::new(Rgba8::new(20, 40, 200, 255), 4.0);
    let mut dash = DashPattern::single(6.0, 3.0);
    dash.pairs[1] = [1.0, 2.0];
    traco.dash = Some(dash);
    VecPath {
        id: 1,
        verts: vec![VecVertex::corner([0.0, 0.0]), torta, canto],
        closed: true,
        fill: Some(Paint::Linear {
            stops: vec![
                GradientStop::new(0.0, Rgba8::new(255, 128, 0, 255)),
                GradientStop::new(0.4, Rgba8::new(200, 0, 90, 255)),
                GradientStop::new(1.0, Rgba8::new(80, 0, 160, 200)),
            ],
            start: [0.0, 0.0],
            end: [30.0, 25.0],
        }),
        stroke: Some(traco),
        paints: vec![PaintEntry::new(PaintKind::Fill(Paint::Solid(Rgba8::new(
            0, 0, 0, 90,
        ))))],
        effects: vec![FxEntry::new(PathEffect::Trim(TrimSpec::default()))],
        ..VecPath::default()
    }
}

/// ⭐ **Escrever o que se leu não muda nada**, para TODO campo que a forma publica — a lei que faz o
/// autokey não ver a mão onde ela não esteve. Um campo cujo leitor e escritor discordem (unidade,
/// índice, ordem de canais) reprova aqui.
#[test]
fn escrever_o_que_se_leu_e_identidade_em_todo_campo() {
    let original = forma();
    let todos = campos(&original, None);
    assert!(
        todos.len() > 60,
        "a fixtura tem de ter os blocos todos: {}",
        todos.len()
    );
    for kk in todos {
        let mut p = original.clone();
        let v = ler(&p, None, kk).expect("publicado ⇒ legível");
        escrever(&mut p, None, kk, v);
        assert_eq!(p, original, "escrever {v} lido de {kk:?} mudou o documento");
    }
}

/// Cada campo publicado MOVE o que diz mover: um valor diferente lê-se de volta (ou quantizado).
#[test]
fn todo_campo_continuo_vai_e_volta() {
    let original = forma();
    for kk in campos(&original, None) {
        if kk.info().is_none_or(|i| i.discreto) {
            continue;
        }
        let mut p = original.clone();
        let antes = ler(&p, None, kk).unwrap();
        let quer = if antes > 100.0 {
            antes - 7.0
        } else {
            antes + 0.25
        };
        escrever(&mut p, None, kk, quer);
        let depois = ler(&p, None, kk).unwrap();
        assert!(
            (depois - quer).abs() <= 0.5,
            "{kk:?}: escrevi {quer}, ficou {depois}"
        );
    }
}

/// A forma inteira: 3 vértices × 8 campos + fechado, todos no mesmo grupo.
#[test]
fn a_forma_publica_os_vertices_e_um_grupo_so() {
    let p = forma();
    let forma: Vec<FieldKey> = campos(&p, None)
        .into_iter()
        .filter(|kk| (c::ANCORA_X..=c::FECHADO).contains(&kk.campo))
        .collect();
    assert_eq!(forma.len(), 3 * 8 + 1);
    assert!(forma.iter().all(|kk| kk.mesmo_grupo(forma[0])));
    assert_eq!(
        ler(&p, None, k(c::TIPO_VERTICE, 0, 1)),
        Some(2.0),
        "Symmetric"
    );
    assert_eq!(ler(&p, None, k(c::CANTO, 0, 2)), Some(4.5));
}

/// A cor passa POR COMPONENTE, em sRGB (o oráculo: vermelho→azul a meio = `(0.5, 0, 0.5)`,
/// `oraculos/godot_interp.saida.txt`): 127,5 escrito num `u8` fica 128, sem conversão de espaço.
#[test]
fn a_cor_a_meio_e_por_componente_srgb() {
    let mut p = VecPath {
        fill: Some(Paint::Solid(Rgba8::new(255, 0, 0, 255))),
        ..VecPath::default()
    };
    for (canal, v) in [(0, 127.5), (1, 0.0), (2, 127.5), (3, 127.5)] {
        escrever(
            &mut p,
            None,
            k(c::TINTA_PREENCH + c::OFF_SOLIDA + canal, 0, 0),
            v,
        );
    }
    assert_eq!(p.fill, Some(Paint::Solid(Rgba8::new(128, 0, 128, 128))));
}

/// O tipo da tinta é estrutura: trocar sólida por linear converte guardando a cor; as paragens que
/// a curva traz preenchem a seguir.
#[test]
fn trocar_o_tipo_da_tinta_converte() {
    let mut p = VecPath {
        fill: Some(Paint::Solid(Rgba8::new(9, 8, 7, 255))),
        ..VecPath::default()
    };
    escrever(&mut p, None, k(c::TINTA_PREENCH + c::OFF_TIPO, 0, 0), 1.0);
    let Some(Paint::Linear { stops, .. }) = &p.fill else {
        panic!("{:?}", p.fill)
    };
    assert_eq!(stops.len(), 2);
    assert_eq!(stops[0].color, Rgba8::new(9, 8, 7, 255));
    escrever(
        &mut p,
        None,
        k(c::TINTA_PREENCH + c::OFF_N_PARAGENS, 0, 0),
        3.0,
    );
    escrever(
        &mut p,
        None,
        k(c::TINTA_PREENCH + c::OFF_PARAGEM, 0, 1),
        0.5,
    );
    assert_eq!(
        ler(&p, None, k(c::TINTA_PREENCH + c::OFF_N_PARAGENS, 0, 0)),
        Some(3.0)
    );
    assert_eq!(
        ler(&p, None, k(c::TINTA_PREENCH + c::OFF_PARAGEM, 0, 1)),
        Some(0.5)
    );
}

/// A geometria de uma forma paramétrica e de um texto é DERIVADA: o canal são os parâmetros.
#[test]
fn forma_parametrica_e_texto_publicam_parametros_e_nao_vertices() {
    let p = forma();
    let param = VecShape::Param {
        kind: 3,
        w: 40.0,
        h: 20.0,
        values: [5.0, 0.4, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    };
    let cs = campos(&p, Some(&param));
    assert!(!cs.iter().any(|kk| kk.campo == c::ANCORA_X));
    assert!(cs.contains(&k(c::PARAM_VALOR, 0, 1)));
    let mut sh = param.clone();
    escrever(
        &mut p.clone(),
        Some(&mut sh),
        k(c::PARAM_LARGURA, 0, 0),
        50.0,
    );
    assert_eq!(ler(&p, Some(&sh), k(c::PARAM_LARGURA, 0, 0)), Some(50.0));

    let texto = VecShape::Text(VecTextParams {
        text: "Olá".to_string(),
        origin: [0.0, 0.0],
        family: None,
        size: 24.0,
        weight: 400.0,
        line_height: 1.2,
        tracking: 0.0,
        align: 0,
        axes: vec![(*b"wght", 400.0)],
        wrap_width: None,
    });
    let cs = campos(&p, Some(&texto));
    assert!(cs.contains(&k(c::TEXTO_TAMANHO, 0, 0)));
    assert!(cs.contains(&k(c::TEXTO_EIXO, 0, 0)));
    assert!(
        !cs.contains(&k(c::TEXTO_QUEBRA, 0, 0)),
        "sem quebra, não há largura de quebra"
    );
}

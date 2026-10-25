use super::*;
use ph2d_ecs::{ItemId, VecTextParams};
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
        paints: vec![PaintEntry {
            id: ItemId(11),
            ..PaintEntry::new(PaintKind::Fill(Paint::Solid(Rgba8::new(0, 0, 0, 90))))
        }],
        effects: vec![FxEntry {
            id: ItemId(12),
            ..FxEntry::new(PathEffect::Trim(TrimSpec::default()))
        }],
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
        if kk.info().is_none_or(|i| i.discreto()) {
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

/// ⭐ Todo campo do vocabulário do vetor tem rótulo e nome de membro TRADUZIDOS — a palavra que o
/// dope-sheet e o losango mostram. Uma chave sem tradução pinta o identificador cru.
#[test]
fn todo_campo_do_vocabulario_tem_rotulo_traduzido() {
    let mut vistos = 0;
    // O vocabulário passa dos 255 (os componentes da entidade, a Interface): varre-se até 1023.
    for campo in 0..1024 {
        let Some(info) = ph2d_ecs::canal_vetor::info(campo) else {
            continue;
        };
        vistos += 1;
        assert_ne!(ph2d_i18n::tr(info.rotulo), info.rotulo, "campo {campo}");
        let m = ph2d_ecs::canal_vetor::membro(campo).expect("campo com info sem nome de membro");
        assert_ne!(ph2d_i18n::tr(m), m, "membro do campo {campo}");
    }
    assert!(vistos > 100, "o vocabulário encolheu? {vistos}");
}

/// ⭐ O PINCEL do traço (`PINCEL_*`, membros da tinta do traço `i`), o DESFASAMENTO do padrão
/// (`OFF_PADRAO_DESFASE`) e o MODO da quebra do texto (`TEXTO_QUEBRA_FIXA`): publicam-se onde
/// existem, escrever o que se leu é identidade, e um valor novo vai e volta no domínio do painel.
#[test]
fn o_pincel_o_desfase_e_a_quebra_fixa_vao_e_voltam() {
    let mut traco = StrokeSpec::new(Rgba8::new(0, 0, 0, 255), 3.0);
    traco.paint = StrokePaint::Brush(Box::new(ph2d_vec_scene::BrushStroke {
        spacing: 1.5,
        offset: -0.25,
        rotation_deg: 30.0,
        scale: 2.0,
        ..ph2d_vec_scene::BrushStroke::default()
    }));
    let mut padrao = ph2d_vec_scene::PatternFill::new(
        ph2d_vec_scene::PatternSource::Shape(9),
        [4.0, 4.0],
        Rgba8::new(1, 2, 3, 255),
    );
    padrao.offset_denom = 3;
    let original = VecPath {
        verts: vec![VecVertex::corner([0.0, 0.0]), VecVertex::corner([9.0, 0.0])],
        fill: Some(Paint::Pattern(Box::new(padrao))),
        stroke: Some(traco),
        ..VecPath::default()
    };
    let todos = campos(&original, None);
    for kk in [
        k(c::PINCEL_ESPACO, 0, 0),
        k(c::PINCEL_ESPELHO, 0, 0),
        k(c::TINTA_PREENCH + c::OFF_PADRAO_DESFASE, 0, 0),
    ] {
        assert!(todos.contains(&kk), "{kk:?} não publicado");
    }
    for kk in todos {
        let mut p = original.clone();
        let v = ler(&p, None, kk).expect("publicado ⇒ legível");
        escrever(&mut p, None, kk, v);
        assert_eq!(p, original, "escrever {v} lido de {kk:?} mudou o documento");
    }
    let mut p = original.clone();
    for (campo, v) in [
        (c::PINCEL_ESPACO, 0.75),
        (c::PINCEL_DESLOCA, 0.5),
        (c::PINCEL_ROTACAO, -45.0),
        (c::PINCEL_ESCALA, 3.0),
        (c::PINCEL_ESPELHO, 1.0),
    ] {
        escrever(&mut p, None, k(campo, 0, 0), v);
        assert_eq!(ler(&p, None, k(campo, 0, 0)), Some(v), "campo {campo}");
    }
    assert!(p.stroke.as_ref().unwrap().brush().unwrap().flip);
    let desfase = k(c::TINTA_PREENCH + c::OFF_PADRAO_DESFASE, 0, 0);
    escrever(&mut p, None, desfase, 4.6);
    assert_eq!(ler(&p, None, desfase), Some(5.0), "discreto: arredonda");
    escrever(&mut p, None, desfase, -3.0);
    assert_eq!(
        ler(&p, None, desfase),
        Some(1.0),
        "o domínio do painel é 1..=255"
    );
    // Sem pincel não há campo de pincel (o traço sólido não o publica nem o inventa).
    let mut solido = forma();
    assert!(
        !campos(&solido, None)
            .iter()
            .any(|kk| kk.campo == c::PINCEL_ESPACO)
    );
    let antes = solido.clone();
    escrever(&mut solido, None, k(c::PINCEL_ESPACO, 0, 0), 9.0);
    assert_eq!(solido, antes);

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
    let fixa = k(c::TEXTO_QUEBRA_FIXA, 0, 0);
    let largura = k(c::TEXTO_QUEBRA, 0, 0);
    assert!(campos(&p, Some(&texto)).contains(&fixa));
    assert_eq!(ler(&p, Some(&texto), fixa), Some(0.0));
    escrever(&mut p, Some(&mut texto), largura, 40.0);
    assert_eq!(
        ler(&p, Some(&texto), largura),
        None,
        "Auto: a largura não existe"
    );
    escrever(&mut p, Some(&mut texto), fixa, 1.0);
    escrever(&mut p, Some(&mut texto), largura, 40.0);
    assert_eq!(ler(&p, Some(&texto), largura), Some(40.0));
    assert!(
        prioridade(fixa) < prioridade(largura),
        "o modo antes da largura"
    );
    escrever(&mut p, Some(&mut texto), fixa, 0.0);
    let VecShape::Text(t) = &texto else { panic!() };
    assert_eq!(t.wrap_width, None);
}

fn duas_tintas() -> VecPath {
    let solida = |r, id| PaintEntry {
        id: ItemId(id),
        ..PaintEntry::new(PaintKind::Fill(Paint::Solid(Rgba8::new(r, 0, 0, 255))))
    };
    VecPath {
        verts: vec![VecVertex::corner([0.0, 0.0]), VecVertex::corner([9.0, 5.0])],
        closed: true,
        paints: vec![solida(10, 21), solida(200, 22)],
        ..VecPath::default()
    }
}

/// ⭐ F31: a chave de uma entrada da pilha é o ID dela — apagar a de baixo não passa a animação da
/// vizinha para outra tinta. Mutação que sangra: `campos`/`interno` de volta a `i = k+1`.
#[test]
fn apagar_uma_tinta_da_pilha_nao_muda_a_chave_da_vizinha() {
    let vermelho = c::TINTA_PREENCH + c::OFF_SOLIDA;
    let a = FieldKey::do_item(VETOR, vermelho, ItemId(21), 0);
    let b = FieldKey::do_item(VETOR, vermelho, ItemId(22), 0);
    let mut p = duas_tintas();
    let cs = campos(&p, None);
    assert!(cs.contains(&a) && cs.contains(&b));
    assert!(
        !cs.iter()
            .any(|kk| kk.i != 0 && c::lista(kk.campo) != Lista::Nenhuma),
        "nenhuma chave posicional de lista sai do funil"
    );
    assert_eq!(
        (ler(&p, None, a), ler(&p, None, b)),
        (Some(10.0), Some(200.0))
    );

    p.paints.remove(0);
    assert_eq!(
        ler(&p, None, b),
        Some(200.0),
        "a chave da vizinha lê a MESMA tinta"
    );
    assert_eq!(ler(&p, None, a), None, "a apagada não tem canal");
    assert!(!campos(&p, None).contains(&a));
    escrever(&mut p, None, b, 77.0);
    let PaintKind::Fill(Paint::Solid(cor)) = &p.paints[0].kind else {
        panic!()
    };
    assert_eq!(cor.r, 77, "e escreve a mesma tinta");
    // A chave POSICIONAL antiga é da migração, não do funil.
    let antiga = FieldKey::new(VETOR, vermelho, 1, 0);
    assert_eq!(ler(&p, None, antiga), None);
    escrever(&mut p, None, antiga, 5.0);
    assert_eq!(p, {
        let mut q = duas_tintas();
        q.paints.remove(0);
        if let PaintKind::Fill(Paint::Solid(c)) = &mut q.paints[0].kind {
            c.r = 77;
        }
        q
    });
}

/// Uma entrada com id por atribuir não tem canal.
#[test]
fn uma_entrada_sem_id_nao_tem_canal() {
    let mut p = duas_tintas();
    p.paints[1].id = ItemId::NONE;
    let com_item: Vec<FieldKey> = campos(&p, None)
        .into_iter()
        .filter(|kk| !kk.item.is_none())
        .collect();
    assert!(!com_item.is_empty() && com_item.iter().all(|kk| kk.item == ItemId(21)));
}

/// ⭐ F31: mover um efeito leva as chaves com ele. Mutação que sangra: a chave do efeito de volta a
/// `i = e`.
#[test]
fn mover_um_efeito_leva_as_chaves_com_ele() {
    let trim = |start, id| FxEntry {
        id: ItemId(id),
        ..FxEntry::new(PathEffect::Trim(TrimSpec {
            start,
            ..TrimSpec::default()
        }))
    };
    let mut p = VecPath {
        verts: vec![VecVertex::corner([0.0, 0.0]), VecVertex::corner([9.0, 5.0])],
        effects: vec![trim(0.1, 31), trim(0.4, 32)],
        ..VecPath::default()
    };
    let chaves = |id| {
        campos(&p, None)
            .into_iter()
            .filter(|kk| kk.item == ItemId(id))
            .collect::<Vec<_>>()
    };
    let (de31, de32) = (chaves(31), chaves(32));
    assert!(!de31.is_empty() && de31.len() == de32.len());
    let antes: Vec<_> = de31.iter().map(|&kk| ler(&p, None, kk)).collect();
    p.effects.swap(0, 1);
    let depois: Vec<_> = de31.iter().map(|&kk| ler(&p, None, kk)).collect();
    assert_eq!(antes, depois, "as chaves do 31 lêem o 31 onde ele estiver");
    let param = FieldKey::do_item(VETOR, c::EFEITO_PARAM, ItemId(31), 0);
    let v = ler(&p, None, param).expect("o 1.º parâmetro do Trim");
    escrever(&mut p, None, param, v + 0.25);
    assert_eq!(ler(&p, None, param), Some(v + 0.25));
    assert_eq!(p.effects[0].id, ItemId(32));
    assert_ne!(p.effects[0].effect, p.effects[1].effect);
    assert_eq!(
        ler(&p, None, FieldKey::new(VETOR, c::EFEITO_PARAM, 0, 0)),
        None,
        "posicional recusada"
    );
}

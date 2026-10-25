//! Gates do [`super`] (`canais_filtros.rs`) — os filtros da forma nos canais.

use super::*;

use ph2d_ecs::item_id::ComItens;

/// O id do degrau na posição `i` de [`pilha`] (`11 + i`).
fn id(i: u16) -> ItemId {
    ItemId(11 + u32::from(i))
}

/// A chave do degrau que [`pilha`] põe na posição `i`.
fn k(campo: u16, i: u16, j: u16) -> FieldKey {
    FieldKey::do_item(VETOR, campo, id(i), j)
}

/// Uma pilha com uma Drop Shadow (raio, deslocamento, cor), um Gradient Map de 3 paragens e um
/// Color Adjust.
fn pilha() -> VecFilter {
    let mut sombra = FxOp::neutral(FxOp::DROP_SHADOW);
    sombra.radius = 0.3;
    sombra.offset = [0.1, -0.2];
    sombra.color = [0.2, 0.4, 0.6, 0.8];
    sombra.opacity = 0.7;
    let mut rampa = FxOp::neutral(FxOp::GRADIENT_MAP);
    rampa.stop_count = 3;
    rampa.stop_pos[1] = 0.5;
    rampa.stops[2] = [1.0, 0.5, 0.25, 1.0];
    let mut ajuste = FxOp::neutral(FxOp::COLOR_ADJUST);
    ajuste.hue = 0.25;
    ajuste.sat = -0.5;
    let mut f = VecFilter::from_ops(vec![sombra, rampa, ajuste]);
    f.com_ids(&[id(0), id(1), id(2)]);
    f
}

/// ⭐ Cada degrau publica só os campos do TIPO dele (o painel não pinta um knob morto, o canal
/// não o publica), e escrever o que se leu é identidade em todos.
#[test]
fn o_filtro_publica_o_que_o_tipo_usa_e_escrever_o_lido_e_identidade() {
    let original = pilha();
    let todos = campos(&original);
    assert!(todos.contains(&k(c::FILTRO_RAIO, 0, 0)));
    assert!(todos.contains(&k(c::FILTRO_DESLOCA_Y, 0, 0)));
    assert!(todos.contains(&k(c::FILTRO_COR + 3, 0, 0)));
    assert!(
        !todos.contains(&k(c::FILTRO_MATIZ, 0, 0)),
        "a sombra não tem matiz"
    );
    assert!(
        todos.contains(&k(c::FILTRO_PARAGEM + 4, 1, 2)),
        "paragem 2, alfa"
    );
    assert!(
        !todos.contains(&k(c::FILTRO_PARAGEM, 1, 3)),
        "só as paragens que contam"
    );
    assert!(todos.contains(&k(c::FILTRO_MATIZ, 2, 0)));
    for kk in &todos {
        assert!(e_de_filtro(*kk));
        let mut f = original.clone();
        let v = ler(&f, *kk).expect("publicado ⇒ legível");
        escrever(&mut f, *kk, v);
        assert_eq!(f, original, "escrever {v} lido de {kk:?} mudou a pilha");
    }
    // O olho é um grupo; o resto do degrau, outro (por degrau).
    let olho = k(c::FILTRO_LIGADO, 0, 0);
    assert!(!olho.mesmo_grupo(k(c::FILTRO_RAIO, 0, 0)));
    assert!(k(c::FILTRO_RAIO, 0, 0).mesmo_grupo(k(c::FILTRO_COR, 0, 0)));
    assert!(!k(c::FILTRO_RAIO, 0, 0).mesmo_grupo(k(c::FILTRO_RAIO, 1, 0)));
}

/// Valores novos vão e voltam; os discretos arredondam e prendem ao domínio; o que não existe
/// (um degrau a mais, uma paragem que não conta) não se inventa.
#[test]
fn o_filtro_escreve_no_dominio_e_nao_inventa() {
    let mut f = pilha();
    for (campo, i, j, v) in [
        (c::FILTRO_RAIO, 0, 0, 0.9),
        (c::FILTRO_DESLOCA_X, 0, 0, -0.4),
        (c::FILTRO_COR + 1, 0, 0, 0.1),
        (c::FILTRO_OPACIDADE, 0, 0, 0.25),
        (c::FILTRO_LIGADO, 0, 0, 0.0),
        (c::FILTRO_PARAGEM, 1, 1, 0.75),
        (c::FILTRO_PARAGEM + 2, 1, 0, 0.5),
        (c::FILTRO_MATIZ, 2, 0, -0.125),
    ] {
        escrever(&mut f, k(campo, i, j), v);
        assert_eq!(ler(&f, k(campo, i, j)), Some(v), "campo {campo} ({i},{j})");
    }
    assert!(!f.ops()[0].enabled);
    escrever(&mut f, k(c::FILTRO_OPACIDADE, 0, 0), 3.0);
    assert_eq!(f.ops()[0].opacity, 1.0);
    escrever(&mut f, k(c::FILTRO_SATURACAO, 2, 0), -9.0);
    assert_eq!(f.ops()[2].sat, -1.0);
    escrever(&mut f, k(c::FILTRO_MISTURA, 0, 0), 4.4);
    assert_eq!(f.ops()[0].blend, 4);
    escrever(&mut f, k(c::FILTRO_MISTURA, 0, 0), 99.0);
    assert_eq!(f.ops()[0].blend, FxOp::BLEND_KINDS - 1);
    escrever(&mut f, k(c::FILTRO_DETALHE, 0, 0), 0.0);
    assert_eq!(f.ops()[0].detail, 1, "o ruído soma pelo menos uma oitava");
    escrever(&mut f, k(c::FILTRO_N_PARAGENS, 1, 0), 2.0);
    assert_eq!(
        ler(&f, k(c::FILTRO_PARAGEM, 1, 2)),
        None,
        "a 3.ª deixou de contar"
    );
    escrever(&mut f, k(c::FILTRO_N_PARAGENS, 1, 0), 40.0);
    assert_eq!(usize::from(f.ops()[1].stop_count), FxOp::MAX_GRADIENT_STOPS);
    let antes = f.clone();
    escrever(&mut f, k(c::FILTRO_RAIO, 7, 0), 1.0);
    escrever(&mut f, k(c::FILTRO_PARAGEM, 0, 0), 1.0);
    assert_eq!(
        f, antes,
        "um degrau a mais e uma paragem sem rampa não se inventam"
    );
    escrever(&mut f, k(c::FILTRO_TIPO, 2, 0), 99.0);
    assert_eq!(usize::from(f.ops()[2].kind), FxOp::KINDS - 1);
}

/// Um degrau sem id não tem canal; a chave POSICIONAL (item `NONE`, a v23 por migrar) não lê nem
/// escreve degrau nenhum.
#[test]
fn a_chave_posicional_e_o_degrau_sem_id_nao_tem_canal() {
    let mut f = pilha();
    f.push(FxOp::neutral(FxOp::BLUR));
    assert!(
        campos(&f).iter().all(|kk| !kk.item.is_none()),
        "o degrau novo espera o id"
    );
    assert_eq!(campos(&f).len(), campos(&pilha()).len());
    let antes = f.clone();
    for i in 0..4 {
        let posicional = FieldKey::new(VETOR, c::FILTRO_RAIO, i, 0);
        assert_eq!(ler(&f, posicional), None);
        escrever(&mut f, posicional, 0.5);
    }
    assert_eq!(f, antes);
}

/// ⭐⭐ **Apagar um filtro não muda a chave do vizinho** (F31 §B2): dois filtros com campos
/// animados; apaga-se o 0; a chave do 1 continua a ler e escrever o MESMO degrau, e a do apagado
/// fica órfã (não acorda no vizinho).
#[test]
fn apagar_um_filtro_nao_muda_a_chave_do_vizinho() {
    let mut f = pilha();
    let do_1 = k(c::FILTRO_PARAGEM, 1, 1);
    let do_0 = k(c::FILTRO_OPACIDADE, 0, 0);
    let valor = ler(&f, do_1).expect("a rampa tem a paragem 1");
    assert_eq!(f.remove(0).map(|o| o.kind), Some(FxOp::DROP_SHADOW));
    assert_eq!(f.ops()[0].kind, FxOp::GRADIENT_MAP, "a rampa subiu para 0");
    assert_eq!(ler(&f, do_1), Some(valor), "a chave da rampa segue a rampa");
    escrever(&mut f, do_1, 0.875);
    assert_eq!(f.ops()[0].stop_pos[1], 0.875, "e escreve na rampa");
    assert!(campos(&f).contains(&do_1));
    assert_eq!(ler(&f, do_0), None, "a chave do apagado fica órfã");
    let antes = f.clone();
    escrever(&mut f, do_0, 0.1);
    assert_eq!(f, antes, "e não escreve no vizinho");
}

/// ⭐⭐ **Mover um filtro leva as chaves com ele**: descer a sombra troca as posições, e cada chave
/// continua a ler o seu degrau.
#[test]
fn mover_um_filtro_leva_as_chaves() {
    let mut f = pilha();
    let raio = k(c::FILTRO_RAIO, 0, 0);
    let matiz = k(c::FILTRO_MATIZ, 2, 0);
    assert!(f.move_down(0));
    assert!(f.move_up(2));
    assert_eq!(
        f.ops().iter().map(|o| o.kind).collect::<Vec<_>>(),
        vec![FxOp::GRADIENT_MAP, FxOp::COLOR_ADJUST, FxOp::DROP_SHADOW]
    );
    assert_eq!(ler(&f, raio), Some(0.3), "o raio é da sombra, agora em 2");
    assert_eq!(
        ler(&f, matiz),
        Some(0.25),
        "a matiz é do ajuste, agora em 1"
    );
    escrever(&mut f, raio, 0.6);
    assert_eq!(f.ops()[2].radius, 0.6);
    assert_eq!(f.ids(), &[id(1), id(2), id(0)]);
}

//! Os gates da forma que o objecto declara (doc 115 W4).
//!
//! ⚠️⚠️ **Toda fixtura aqui tem `size ≠ 1`, e isso é a lei do ficheiro e não um gosto.** A lei
//! errada que a §10.1 do doc 115 escreveu — declarar `size / 2` em vez do quadrado unitário — dá
//! **exactamente a mesma resposta** quando `size = [1, 1]`, porque a porta da leitura multiplica
//! pelo `size`. *Um corpus no ponto neutro de uma conversão não testa essa conversão*, e o ponto
//! neutro aqui é o objecto mais natural que existe.

use super::MEIA_DO_OBJECTO;
use ph2d_contact::Forma;
use ph2d_nodegraph::attr::{
    BOUNCE_COLUMN, COLLIDER_BOX_COLUMN, COLLIDER_COLUMN, COLLIDER_OFFSET_COLUMN, Column,
    FRICTION_COLUMN, INV_INERTIA_COLUMN, Stream,
};

const BRANCO: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const UV: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

/// A caixa de MUNDO que a linha `i` de `s` declara, pela porta do produto.
fn caixa(s: &Stream, i: usize) -> Option<([f32; 2], [f32; 2])> {
    let c = ph2d_contact::colisores(s)?.get(i).copied().flatten()?;
    match c.forma {
        Forma::Caixa { meia, .. } => Some((meia, c.desvio)),
        Forma::Disco(_) => None,
    }
}

/// ⭐⭐⭐ **A LEI, medida pela porta que a lê: a caixa de mundo de um objecto é `size / 2`** — e a
/// declaração que a produz é o quadrado UNITÁRIO, porque [`ph2d_contact::declarado`] converte de
/// geometria para mundo multiplicando pelo `size` da própria linha.
///
/// ⛔ **É este gate que mata a redacção `ph2d_collider_box = size / 2`**: com ela, um objecto de
/// `4 × 1` declararia `[2; 0,5]` e a porta devolveria `[8; 0,5]` — o **dobro** do quadro desenhado
/// num eixo e certo no outro, que é a forma mais difícil de ver numa foto.
#[test]
fn a_caixa_de_mundo_de_um_objecto_e_metade_do_tamanho_dele() {
    for size in [[4.0, 1.0], [0.5, 0.5], [2.0, 3.0], [1.0, 1.0]] {
        let s = super::super::appearance_tile(size, BRANCO, UV, 7, false);
        let (meia, desvio) = caixa(&s, 0).expect("um objecto declara uma CAIXA");
        assert_eq!(
            meia,
            [size[0] / 2.0, size[1] / 2.0],
            "a caixa de mundo tem de ser metade do quadro desenhado (size = {size:?})"
        );
        assert_eq!(
            desvio,
            [0.0, 0.0],
            "o quadro de um objecto e' centrado no P"
        );
    }
}

/// ⭐⭐ **OS TRÊS MÉDIOS declaram, e o piso de população é o que impede este gate de medir dois.**
///
/// A membrana tem três construtores de aparência — sprite/ladrilho, vector vivo e grupo — e um
/// médio que fique de fora é um objecto que **não colide**, calado. ⚠️ O grupo entra com uma folha
/// **escalada** (`size` da folha já multiplicado pelo `acc.scale`), que é o caminho por onde a
/// conversão de unidades erraria sem ninguém ver.
#[test]
fn os_tres_construtores_da_aparencia_declaram_a_forma() {
    let esperado = [2.0, 0.5];
    let sprite = super::super::appearance_tile([4.0, 1.0], BRANCO, UV, 7, false);
    let vector = super::super::appearance_vector([4.0, 1.0], BRANCO, 3);
    let grupo = super::super::group_stream(&[folha([0.0, 0.0], [4.0, 1.0], 0.0)]);
    let medidos = [
        ("sprite/ladrilho", &sprite),
        ("vector vivo", &vector),
        ("grupo", &grupo),
    ];
    assert_eq!(medidos.len(), 3, "o piso: TRES construtores de aparencia");
    for (nome, s) in medidos {
        let (meia, _) =
            caixa(s, 0).unwrap_or_else(|| panic!("`{nome}` nao declarou forma nenhuma"));
        assert_eq!(meia, esperado, "`{nome}`");
    }
}

/// ⚠️ **Uma folha do grupo leva a ORIENTAÇÃO dela**, e a caixa é orientada com ela — o `rot` já
/// viajava na corrente e a porta da leitura lê-o. Sem isto, um grupo rodado separaria por uma
/// caixa alinhada aos eixos sobre arte que não está.
#[test]
fn a_caixa_de_uma_folha_rodada_roda_com_ela() {
    let s = super::super::group_stream(&[folha([0.0, 0.0], [4.0, 1.0], 90.0)]);
    let c = ph2d_contact::colisores(&s).expect("declara")[0].expect("uma caixa");
    let Forma::Caixa { meia, eixo } = c.forma else {
        panic!("uma folha e' uma caixa, nunca um disco");
    };
    assert_eq!(meia, [2.0, 0.5], "as meias sao da folha, nao do angulo");
    assert!(
        eixo[0].abs() < 1e-6 && (eixo[1] - 1.0).abs() < 1e-6,
        "a 90 graus o eixo x da caixa aponta para cima: {eixo:?}"
    );
}

/// ⛔⛔ **AS QUATRO AUSÊNCIAS, cada uma uma lei** — e um gate por todas, porque a cura de escrever
/// qualquer uma delas é a mesma: apagar.
///
/// O desvio (a arte é centrada), o raio (um objecto é um quadro, e a caixa ganha na porta), o
/// material e a inércia (um objecto **não tem cartão** — escrever um default seria autorar em nome
/// do artista, e a ausência é que quer dizer *«não declarei material nenhum»*).
#[test]
fn a_membrana_declara_a_forma_e_mais_nada_nenhuma() {
    let s = super::super::appearance_tile([4.0, 1.0], BRANCO, UV, 7, false);
    assert!(s.get(COLLIDER_BOX_COLUMN).is_some(), "a caixa, essa, vai");
    for c in [
        COLLIDER_OFFSET_COLUMN,
        COLLIDER_COLUMN,
        FRICTION_COLUMN,
        BOUNCE_COLUMN,
        INV_INERTIA_COLUMN,
    ] {
        assert!(
            s.get(c).is_none(),
            "a membrana escreveu `{c}` — ela nao tem cartao para o autorar, e a AUSENCIA e' a \
             declaracao (ver o cabecalho do `motion_bridge_objects_collider.rs`)"
        );
    }
}

/// ⚠️ **Uma corrente vazia sai como entrou.** Uma coluna de zero linhas não é uma declaração, e a
/// porta da leitura exige que todo comprimento case com o da nuvem — uma coluna a mais num stream
/// vazio seria uma pergunta sem resposta.
#[test]
fn uma_corrente_vazia_nao_declara_nada() {
    let vazia = Stream::new(0);
    let saida = super::com_colisor(vazia);
    assert_eq!(saida.count(), 0);
    assert!(saida.get(COLLIDER_BOX_COLUMN).is_none());
}

/// ⚠️ A constante é `0,5` e **não** `1`, que é a convenção da `source.shape` (geometria em raio 1).
/// Trocá-las dá um colisor com o dobro do objecto, e as duas leem-se igual numa tabela.
#[test]
fn a_meia_declarada_e_o_quadrado_unitario() {
    assert_eq!(MEIA_DO_OBJECTO, [0.5, 0.5]);
}

/// Uma folha de grupo, com o tamanho já composto (é o que o `resolve_leaf` entrega).
fn folha(p: [f32; 2], size: [f32; 2], rot_deg: f32) -> super::super::LeafInstance {
    super::super::LeafInstance {
        p,
        rot_deg,
        size,
        tint: BRANCO,
        uv: UV,
        tid: 7,
        gid: 0,
    }
}

/// O tipo da coluna é o que a porta exige — um `Vec2` do comprimento da nuvem.
#[test]
fn a_coluna_e_um_vec2_do_comprimento_da_nuvem() {
    let s = super::super::group_stream(&[
        folha([0.0, 0.0], [2.0, 2.0], 0.0),
        folha([3.0, 0.0], [1.0, 1.0], 0.0),
    ]);
    match s.get(COLLIDER_BOX_COLUMN) {
        Some(Column::Vec2(v)) => assert_eq!(v.len(), 2),
        outro => panic!("a caixa tem de ser um Vec2 por folha: {outro:?}"),
    }
}

//! ⭐⭐⭐ **OS GATES DAS DUAS ÚLTIMAS FORMAS DO CATÁLOGO** (W139) — a esfera com cratera e a lente.
//!
//! # ⚠️ Por que estes gates medem a FORMA, e não o campo
//!
//! O censo já mede o campo das 62 primitivas — o gradiente, o raio de contenção, o maior filete, o
//! que o campo sabe longe da peça. O que ele **não** pergunta é *«esta peça é a que o nome
//! promete?»*: uma cratera com a mordida no sítio errado sai uma **esfera lisa** e passa em tudo.
//!
//! ⚠️ **E a wave anterior tornou isso concreto:** a W138 entregou as duas por composição e o
//! catálogo, o alcance e a paleta ficaram todos verdes — *o nome prometia um buraco e ninguém
//! perguntava se ele lá estava*.

use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Primitive, Xform};
use ph2d_field_eval::Field;

fn campo(p: Primitive) -> Field {
    let doc = FieldDoc::new(
        vec![Node::new(Xform::IDENTITY, NodeKind::Leaf(p))],
        NodeId(0),
    )
    .expect("o documento tem de aceitar a peça");
    Field::new(&doc)
}

fn cratera(radius: f32, crater: f32, depth: f32, round: f32) -> Primitive {
    Primitive::CrateredSphere {
        radius,
        crater,
        depth,
        round,
        chamfer: 0.0,
    }
}

fn lente(radius: f32, offset: f32, round: f32) -> Primitive {
    Primitive::Lens {
        radius,
        offset,
        round,
        chamfer: 0.0,
    }
}

/// ⭐⭐⭐ **A ESFERA COM CRATERA TEM UMA CRATERA, e o FUNDO dela está onde o `depth` diz.**
///
/// ⚠️ **A régua é o eixo `+Z`, onde a mordida é por construção**, e ela mede as TRÊS coisas que
/// separam esta peça de uma esfera: o polo oposto ainda é sólido, o polo mordido é ar, e a
/// transição está a `radius − depth` — que é o que o rótulo *Depth* promete ao artista.
#[test]
fn the_crater_floor_sits_where_the_depth_says() {
    let (r, k, d) = (0.45_f32, 0.30_f32, 0.28_f32);
    let f = campo(cratera(r, k, d, 0.0));
    let fundo = f64::from(r - d);
    assert!(
        f.at(0.0, 0.0, -f64::from(r) * 0.9) < 0.0,
        "o polo oposto tem de continuar sólido"
    );
    assert!(
        f.at(0.0, 0.0, f64::from(r) * 0.9) > 0.0,
        "a cratera tem de ter comido o polo +Z"
    );
    assert!(
        f.at(0.0, 0.0, fundo - 0.04) < 0.0,
        "logo abaixo do fundo ainda é peça: {}",
        f.at(0.0, 0.0, fundo - 0.04)
    );
    assert!(
        f.at(0.0, 0.0, fundo + 0.04) > 0.0,
        "logo acima do fundo já é ar: {}",
        f.at(0.0, 0.0, fundo + 0.04)
    );
}

/// ⭐⭐⭐ **A LENTE É MAIS LARGA QUE GROSSA** — é isso que a distingue de uma esfera.
///
/// ⚠️ Com os centros a `±offset` a meia-espessura é `radius − offset` e o **aro** fica a
/// `√(radius² − offset²)` do eixo. ⛔ Uma régua que só perguntasse *«há peça?»* aprovaria as duas
/// esferas **sobrepostas por completo**, que é uma esfera.
#[test]
fn the_lens_is_wider_than_it_is_thick() {
    let (r, off) = (0.55_f32, 0.28_f32);
    let f = campo(lente(r, off, 0.0));
    let meia = f64::from(r - off);
    let aro = f64::from(r * r - off * off).sqrt();
    assert!(
        aro > meia * 1.5,
        "a fixtura tem de ser uma lente, não uma bola"
    );
    assert!(f.at(0.0, 0.0, meia * 0.8) < 0.0, "dentro, no eixo");
    assert!(f.at(0.0, 0.0, meia * 1.2) > 0.0, "fora, no eixo");
    assert!(f.at(aro * 0.9, 0.0, 0.0) < 0.0, "dentro, no equador");
    assert!(f.at(aro * 1.1, 0.0, 0.0) > 0.0, "fora, no equador");
}

/// ⭐⭐ **AFASTAR OS CENTROS AFINA A LENTE** — o controlo faz o que diz, ao longo do curso todo.
///
/// ⚠️ *Um gate no representante deixa o CURSO do controlo por medir* — e este é o único número que
/// a lente tem além do raio.
#[test]
fn raising_the_offset_thins_the_lens() {
    let r = 0.55_f32;
    let mut anterior = f64::INFINITY;
    for k in [0.15_f32, 0.30, 0.45] {
        let f = campo(lente(r, r * k, 0.0));
        // A meia-espessura, achada por bissecção no eixo — medida do PRODUTO, não da fórmula.
        let (mut lo, mut hi) = (0.0_f64, f64::from(r));
        for _ in 0..40 {
            let m = f64::midpoint(lo, hi);
            if f.at(0.0, 0.0, m) < 0.0 {
                lo = m;
            } else {
                hi = m;
            }
        }
        assert!(lo < anterior, "afastar os centros tem de AFINAR: {lo:.4}");
        anterior = lo;
    }
}

/// ⭐⭐⭐ **O ARO DE UMA CRATERA É UM LÁBIO, e o filete EMBOTA-O** — e esta afirmação nasceu ao
/// contrário.
///
/// # ⛔⛔ A primeira redacção deste gate dizia «o filete ENCHE», por ANALOGIA
///
/// A rosca (W135) pagou a lei *«um chanfro numa quina CÔNCAVA não corta — ele ENCHE»*, e eu apliquei
/// a mesma frase a uma cratera porque *«uma cratera é um vinco»*. **Não é.** Medido: o ganho de
/// matéria com o filete a `0,10` é **`0,00000`** em toda a secção — o filete nunca acrescenta nada.
///
/// ⭐ **A geometria diz porquê, e a conta cabe numa linha.** No aro do representante
/// (`r = 0,45 · k = 0,30 · d = 0,28`) as duas normais EXTERIORES da matéria fazem `105°` entre si
/// ⇒ o ângulo diedro da peça ali é `75°`, que é **convexo**: o que a bola e a parede da tigela
/// formam não é um vale, é um **lábio afiado**. ⇒ o filete corta-o, que é exactamente o que um
/// artista quer de um prato.
///
/// ⚠️ *Uma lei paga noutra forma aplica-se pelo MECANISMO, nunca pelo nome* — «cratera» soa a
/// concavidade e o aro dela é convexo.
#[test]
fn the_crater_rim_is_a_lip_and_the_fillet_blunts_it() {
    let (r, k, d) = (0.45_f32, 0.30_f32, 0.28_f32);
    let vivo = campo(cratera(r, k, d, 0.0));
    let macio = campo(cratera(r, k, d, 0.10));
    let (mut maior_corte, mut maior_ganho) = (0.0_f64, 0.0_f64);
    for i in 0..80 {
        for j in 0..80 {
            let x = f64::from(r) * 1.3 * (f64::from(i) / 79.0);
            let z = f64::from(r) * (2.4 * f64::from(j) / 79.0 - 0.4);
            let delta = macio.at(x, 0.0, z) - vivo.at(x, 0.0, z);
            maior_corte = maior_corte.max(delta);
            maior_ganho = maior_ganho.max(-delta);
        }
    }
    assert!(
        maior_corte > 0.01,
        "o filete não embotou lábio nenhum (maior corte {maior_corte:.5})"
    );
    // ⛔ **A METADE QUE CORRIGE A ANALOGIA**: ele nunca ENCHE. Sem esta linha o gate ficaria verde
    // sobre um operador que fizesse as duas coisas, e a lei que o doc afirma seria uma opinião.
    assert!(
        maior_ganho < 1.0e-4,
        "o filete acrescentou matéria ({maior_ganho:.5}) — num lábio convexo ele só pode CORTAR"
    );
}

/// ⭐⭐⭐ **AS DUAS CERCAS SÃO DE EXISTÊNCIA, e o documento RECUSA em voz alta.**
///
/// ⚠️ **Não são cercas de gosto:** com `depth ≥ 2·radius` a esfera que morde contém a bola inteira
/// e com `offset ≥ radius` as duas esferas da lente não se tocam — nos dois casos a peça fica
/// **vazia**, e um botão do catálogo que às vezes não cria nada é a affordance que mente.
///
/// ⛔ E o CONTROLE é a outra metade: logo abaixo da cerca as duas ainda são peça.
#[test]
fn the_document_refuses_the_two_shapes_that_would_be_empty() {
    let doc = |p: Primitive| {
        FieldDoc::new(
            vec![Node::new(Xform::IDENTITY, NodeKind::Leaf(p))],
            NodeId(0),
        )
    };
    assert!(
        doc(cratera(0.45, 0.30, 0.90, 0.0)).is_err(),
        "uma mordida de `2·radius` engole a bola — o documento tem de recusar"
    );
    assert!(
        doc(lente(0.55, 0.55, 0.0)).is_err(),
        "com os centros a um `radius` inteiro as esferas não se tocam — recusa"
    );
    // ⛔ **O CONTROLE**: logo abaixo da cerca as duas existem, e têm matéria.
    //
    // ⚠️ **VARRE-SE a peça, não se aponta um sítio.** A 1.ª redacção mediu `(0, 0, −0,40)` por
    // simetria com o polo oposto — e com `depth = 0,88` a mordida chega a `z = −0,43`, logo o ponto
    // escolhido está DENTRO dela. *Um controlo que aponta uma coordenada afirma onde a peça está;
    // a pergunta é só se ela existe.*
    let quase_toda = campo(cratera(0.45, 0.30, 0.88, 0.0));
    let mut tem_peca = false;
    for i in 0..40 {
        for j in 0..40 {
            let x = 0.9 * (f64::from(i) / 39.0);
            let z = 0.9 * (2.0 * f64::from(j) / 39.0 - 1.0);
            tem_peca |= quase_toda.at(x, 0.0, z) < 0.0;
        }
    }
    assert!(tem_peca, "logo abaixo da cerca ainda tem de sobrar peça");
    let finissima = campo(lente(0.55, 0.53, 0.0));
    assert!(
        finissima.at(0.0, 0.0, 0.0) < 0.0,
        "uma lente finíssima ainda é uma lente"
    );
}

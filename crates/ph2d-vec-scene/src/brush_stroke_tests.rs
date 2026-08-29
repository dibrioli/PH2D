//! **OS GATES DO PINCEL DE CONTORNO** (plano 36) — irmão do [`super::paint_pattern_tests`] pelo
//! teto de 700 LOC, e o corte é por MODELO, não por tamanho.
//!
//! ⚠️⚠️ **São duas leis CONTRÁRIAS, e é por isso que são dois ficheiros.** Ali o padrão é uma TINTA
//! que o contorno REVELA — normativo em SVG 2, e por isso um tracejado são **buracos** nela e ela
//! **não** escala com a largura. Aqui a arte **PERCORRE** o caminho, reinicia em cada traço e
//! **escala** com a largura, porque o pincel **É** a faixa. *Guardar as duas no mesmo ficheiro
//! convidaria a próxima janela a "unificar" o que o plano 36 §1 mediu como dois modelos.*

use super::{Rgba8, StrokeSpec};

/// A fixtura do PADRÃO, emprestada do irmão: metade dos gates daqui existe para provar que um
/// pincel **não** responde como padrão, e sem ela não haveria contra-exemplo.
use super::paint_pattern_tests::fill;
use super::paint_pattern_tests::stroke_com_padrao;

// ── ⭐⭐⭐ O PINCEL DE CONTORNO — plano 36, W1 ─────────────────────────────────────
//
// ⚠️ **Dois modelos, e é por isso que são duas variantes**: o `Pattern` é uma TINTA que o contorno
// revela (normativo em SVG 2), o `Brush` é uma ARTE que o percorre (o *Pattern Brush* do
// Illustrator). Os gates abaixo prendem o que os DISTINGUE, não o que têm em comum.

fn pincel() -> StrokeSpec {
    let b = crate::BrushStroke {
        art: Some(crate::VecPathId::from(42u64)),
        fallback: Rgba8::new(11, 22, 33, 200),
        spacing: 1.25,
        offset: -0.5,
        flip: true,
        rotation_deg: 90.0,
        scale: 2.0,
    };
    let mut s = StrokeSpec::new(Rgba8::new(1, 2, 3, 255), 0.4);
    s.paint = crate::StrokePaint::Brush(Box::new(b));
    s
}

/// ⭐⭐ **UM TRAÇO PODE CARREGAR UM PINCEL** — o buraco inteiro da W1.
#[test]
fn a_stroke_can_carry_a_brush() {
    let s = pincel();
    assert!(s.brush().is_some(), "o traco nao carrega o pincel");
    // ⚠️ **E ele NÃO é um padrão.** Os dois modelos vivem no mesmo enum, e uma porta que confundisse
    // os dois faria o desenho escolher a lei errada — que é o assunto inteiro do plano 36.
    assert!(
        s.pattern().is_none(),
        "um pincel respondeu como PADRAO - as duas leis sao contrarias (uma e' papel de parede que \
         o contorno revela, a outra e' arte que o percorre)"
    );
    // CONTROLO: os outros dois não respondem como pincel.
    assert!(
        StrokeSpec::new(Rgba8::new(1, 2, 3, 255), 1.0)
            .brush()
            .is_none()
    );
    assert!(stroke_com_padrao().brush().is_none());
}

/// ⭐ **A COR continua a ter resposta num traço com pincel** — a de recurso, que é o que a linha
/// pinta enquanto a arte não resolve.
///
/// ⚠️ É esta porta que mantém honesta a swatch, o token de cor e o `StrokeStyle` da shell: eles só
/// sabem perguntar *"de que cor é este traço?"*, e um modelo novo não os pode calar.
#[test]
fn the_stroke_colour_still_answers_for_a_brush() {
    assert_eq!(pincel().color(), Rgba8::new(11, 22, 33, 200));
}

/// **O pincel sobrevive ao save, com a lei INTEIRA.**
///
/// ⚠️ **Campo a campo, e não `assert_eq!` da struct:** um `PartialEq` verde diz que os bytes voltam,
/// e não QUAIS. Se alguém apender um campo e esquecer de o escrever, a igualdade passa (os dois
/// lados têm o default) e este gate também — a menos que ele NOMEIE os campos, que é o que faz a
/// fixtura ter valores distintos e não-default em todos.
#[test]
fn the_brush_survives_the_save() {
    let s = pincel();
    let bytes = postcard::to_allocvec(&s).expect("serializa");
    let back: StrokeSpec = postcard::from_bytes(&bytes).expect("desserializa");
    let b = back.brush().expect("continua a ser um pincel");
    assert_eq!(b.art, Some(crate::VecPathId::from(42u64)));
    assert_eq!(b.fallback, Rgba8::new(11, 22, 33, 200));
    assert!((b.spacing - 1.25).abs() < 1e-12, "spacing");
    assert!((b.offset + 0.5).abs() < 1e-12, "offset");
    assert!(b.flip, "flip");
    assert!((b.rotation_deg - 90.0).abs() < 1e-12, "rotation_deg");
    assert!((b.scale - 2.0).abs() < 1e-12, "scale");
    // ⚠️ **E nenhum campo é o default** — senão o round-trip aprovaria um escritor que não escreve.
    let d = crate::BrushStroke::default();
    assert_ne!(b.spacing, d.spacing);
    assert_ne!(b.scale, d.scale);
    assert_ne!(b.rotation_deg, d.rotation_deg);
}

/// ⛔⛔ **A ARTE DE UM PINCEL É UMA FORMA, e o modelo não sabe dizer outra coisa.**
///
/// O motor (`pattern_along`, plano 23) copia **GEOMETRIA**. Se o campo fosse um `PatternSource`
/// — que também aceita imagem — então `Brush(Image(..))` seria **gravável e indesenhável**: o
/// documento aceitaria um estado que nenhum desenho honra. É a MESMA lei que recusou reusar o
/// `Paint` do preenchimento como tinta de traço (plano 35 §2.1).
///
/// ⚠️ Este gate é sobre um TIPO, e o que ele prova é que a asserção é impossível de violar sem
/// mudar o tipo — é a forma mais forte de invariante que há, e a mais fácil de apagar sem dar por
/// isso num refactor.
#[test]
fn a_brush_can_only_name_a_shape_never_an_image() {
    let b = crate::BrushStroke::default();
    // Compila porque `art` é um `Option<VecPathId>`. ⛔ Se alguém o trocar por `PatternSource`, esta
    // linha deixa de compilar e o gate fala.
    let _: Option<crate::VecPathId> = b.art;
    // ⭐ E o `None` é REPRESENTÁVEL: *"sem arte"* não pode ser o mesmo byte que *"a arte é a forma
    // de id zero"* — um gate da W4 achou exactamente isso, e é por isso que o campo é um `Option`.
    assert_eq!(crate::BrushStroke::default().art, None);
}

/// ⚠️ **O PINCEL escala com a largura do traço; o PADRÃO não** — e as duas leis são deliberadas.
///
/// O plano 35 §2.3 fixou que uma TINTA não escala com a largura (*"a largura decide a faixa; o
/// padrão decide o que a preenche"*) — a queixa clássica do Illustrator, do lado certo. Um pincel é
/// o oposto **porque ele É a faixa**, e é o que o *Pattern Brush* faz.
///
/// ⇒ o pincel guarda um `scale` **relativo** (multiplica a altura derivada da largura), e o padrão
/// guarda um `size` **absoluto** em unidades de mundo. *Se os dois guardassem a mesma grandeza, uma
/// das duas leis estaria escrita no sítio errado.*
#[test]
fn the_brush_scale_is_relative_and_the_pattern_size_is_absolute() {
    assert!(
        (crate::BrushStroke::default().scale - 1.0).abs() < 1e-12,
        "o neutro do pincel e' `1,0` - um multiplicador cujo neutro nao e' um nao e' um \
         multiplicador"
    );
    // O padrão guarda unidades de mundo, e o default do construtor prova-o: ele RECEBE o tamanho.
    let p = fill();
    assert!(
        p.size[0] > 0.0 && p.size[1] > 0.0,
        "o padrao deixou de guardar um tamanho de MUNDO"
    );
}

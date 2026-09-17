//! ⭐⭐⭐ **AS OPÇÕES DE UM SELECTOR CHEGAM A TINTA COMO PALAVRAS, NÃO COMO IDENTIFICADORES.**
//!
//! # Por que este ficheiro existe
//!
//! Desde a 5.ª fatia do HR-15 o array de opções de um `ParamWidget::Enum` carrega **chaves**
//! (`node.motion.wave.param.mode.0`) e não texto, e quem as resolve é **quem desenha**. ⚠️⚠️
//! Isso é o oposto do que a casa faz com o `label` da row, que é resolvido na FRONTEIRA — e a
//! razão é o TIPO, não uma preferência: as opções vivem num `&'static [&'static str]` **dentro
//! de um `ParamUiHint` que é `Copy`** e existe em ~800 sítios de struct-literal; reconstruí-lo
//! na ponte obrigava a alocar por quadro no caminho de OMISSÃO (o cartão).
//!
//! ⛔⛔ **E é por isso que este gate tem de existir:** com a tradução no pintor, um `tr` em falta
//! deixa **todos** os gates de dados verdes — o array está bem derivado, as chaves resolvem-se na
//! tabela — e o artista lê `node.motion.wave.param.mode.0` no botão. É a lei do §5.0: *nenhum
//! instrumento pergunta se o VALOR chega a um consumidor.*
//!
//! # A régua, e por que é DIFERENCIAL
//!
//! O arnês conta **glifos**, não texto. ⇒ pinta-se a MESMA row duas vezes — uma com as chaves e
//! outra com as palavras que essas chaves resolvem — e exige-se a mesma contagem. ⭐ Isso afirma
//! a propriedade inteira sem escrever um número: se o pintor resolver, as duas pinturas são a
//! mesma; se não resolver, a das chaves emite muito mais glifos, porque uma chave é `~4×` mais
//! longa que a palavra dela. ⚠️ **Um `assert` contra um número escrito aqui mediria a fonte e o
//! tema**, e ficaria vermelho no dia em que alguém tocasse num token.

use super::*;
use ph2d_editor_core::zones::Rect;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1920.0,
    h: 1080.0,
};

/// As chaves de um selector REAL do catálogo. ⚠️ **A 1.ª redacção usou `motion.mirror::keep` e o
/// controlo de baixo reprovou-a**: aquele selector é servido por uma `const` partilhada, logo a
/// chave dele é do ENDEREÇO dela (`node.opts.…`) e não da forma inline — *a fixtura estava a
/// afirmar uma chave que o produto nunca emite, e sem o controlo o gate teria medido dois lados
/// iguais por serem os dois identificadores.*
const CHAVES: &[&str] = &[
    "node.motion.wave.param.edges.0",
    "node.motion.wave.param.edges.1",
];

fn linha(opcoes: &'static [&'static str]) -> ParamsSnapshot {
    ParamsSnapshot {
        node: 7,
        title: "Fixture".into(),
        modified: Default::default(),
        sections: Vec::new(),
        folded_by_default: std::collections::BTreeSet::new(),
        rows: vec![ParamRow::Enum(EnumRow {
            name: "keep",
            label: "Keep".into(),
            selected: 0,
            labels: opcoes,
        })],
    }
}

fn glifos(opcoes: &'static [&'static str]) -> u32 {
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<MotionParamsPanel>();
    set_current_params(Some(linha(opcoes)));
    let mut state = MotionParamsPanelState;
    host.paint_and_count_geometry::<MotionParamsPanel>(&mut state, VIEWPORT)
        .0
}

#[test]
fn the_option_captions_reach_ink_as_words_not_as_keys() {
    // ⛔ Controlo da própria fixtura: se a chave deixasse de resolver, os dois lados seriam
    // iguais **por serem os dois chaves**, e o gate passaria a medir nada.
    let palavras: Vec<&'static str> = CHAVES.iter().map(|k| ph2d_i18n::tr(k)).collect();
    for (k, p) in CHAVES.iter().zip(&palavras) {
        assert_ne!(
            k, p,
            "a chave {k:?} não resolve — a fixtura deste gate deixou de conter o fenómeno"
        );
    }
    let palavras: &'static [&'static str] = Box::leak(palavras.into_boxed_slice());

    let com_chaves = glifos(CHAVES);
    let com_palavras = glifos(palavras);
    assert_eq!(
        com_chaves, com_palavras,
        "a row das CHAVES emitiu {com_chaves} glifos e a das PALAVRAS {com_palavras} — o pintor \
         das opções não está a resolver a chave, e o artista lê o identificador no botão"
    );
}

/// ⭐⭐ **E o controlo POSITIVO da régua** — sem ele, um pintor que não emitisse glifo nenhum
/// (um `match` no braço errado) daria `0 == 0` e passaria.
#[test]
fn the_ruler_sees_the_captions_at_all() {
    let uma: &'static [&'static str] = &["node.motion.wave.param.edges.0"];
    let duas = glifos(CHAVES);
    let so_uma = glifos(uma);
    assert!(
        duas > so_uma && so_uma > 0,
        "duas opções emitiram {duas} glifos e uma só {so_uma} — ou as legendas não são pintadas, \
         ou o que cresce é a caixa e não o texto"
    );
}

use super::*;
use ph2d_node_fx_lens::{LUT, TYPE_NAME, param};

/// Um ficheiro só deste teste (o nome leva o teste e o processo: dois testes em paralelo nunca
/// partilham o mesmo).
fn ficheiro(nome: &str, texto: &str) -> String {
    let p = std::env::temp_dir().join(format!("ph2d_lens_{}_{nome}.cube", std::process::id()));
    std::fs::write(&p, texto).expect("escreve");
    p.to_string_lossy().into_owned()
}

fn cubo_2() -> String {
    let mut t = String::from("LUT_3D_SIZE 2\n");
    for b in 0..2 {
        for g in 0..2 {
            for r in 0..2 {
                t.push_str(&format!("{r} {g} {b}\n"));
            }
        }
    }
    t
}

fn grafo() -> (Graph, NodeId) {
    let mut g = Graph::new();
    let no = g.add_node(TYPE_NAME);
    (g, no)
}

/// Sem `fx.lens` a lente não corre, nem com o Motion na cena.
#[test]
fn without_the_node_the_lens_stays_off() {
    let mut l = MotionLente::default();
    l.prepara(&Graph::new(), 3);
    assert!(l.lente().is_none());
    assert!(!l.liga(true));
}

/// ⭐ **O nó largado liga a lente — mas só com o Motion na cena.**
#[test]
fn a_dropped_lens_runs_only_while_the_motion_draws() {
    let (g, _) = grafo();
    let mut l = MotionLente::default();
    l.prepara(&g, 3);
    assert!(l.liga(true));
    assert!(!l.liga(false));
}

/// Só a LUT (a vinheta a zero): a lente corre SE o ficheiro entrou.
#[test]
fn a_lut_alone_runs_only_when_the_file_was_read() {
    let (mut g, no) = grafo();
    g.set_param(no, param::VIN, 0.0);
    let mut l = MotionLente::default();
    l.prepara(&g, 0);
    assert!(!l.liga(true), "neutra sem LUT");
    g.set_text_param(no, LUT, ficheiro("so_lut", &cubo_2()));
    l.prepara(&g, 0);
    assert!(l.cubo().is_some());
    assert!(l.liga(true));
}

/// ⭐⭐ **A recusa diz-se no CARTÃO do nó** — com a razão, e só nesse nó.
/// (Mutação: `queixa` a devolver `None` ⇒ o cartão cala-se — RED.)
#[test]
fn a_refused_lut_speaks_on_its_card() {
    let (mut g, no) = grafo();
    let mut l = MotionLente::default();
    g.set_text_param(no, LUT, ficheiro("um_d", "LUT_1D_SIZE 4\n0 0 0\n"));
    l.prepara(&g, 0);
    assert_eq!(l.cubo(), None);
    assert_eq!(
        l.queixa(no).as_deref(),
        Some(ph2d_i18n::tr("app.motion.motion_lens.lut_1d"))
    );
    assert_eq!(l.queixa(NodeId(no.0 + 1)), None);

    g.set_text_param(no, LUT, "/não/existe/look.cube");
    l.prepara(&g, 0);
    assert_eq!(
        l.queixa(no).as_deref(),
        Some(ph2d_i18n::tr("app.motion.motion_lens.lut_unreadable"))
    );

    g.set_text_param(no, LUT, ficheiro("bom", &cubo_2()));
    l.prepara(&g, 0);
    assert_eq!(l.queixa(no), None);
}

/// Cada recusa tem a SUA frase — nenhuma cai na de outra.
#[test]
fn every_refusal_has_its_own_sentence() {
    let frases = [
        ErroLut::Ilegivel,
        ErroLut::Cubo(ErroCubo::So1d),
        ErroLut::Cubo(ErroCubo::SemTamanho),
        ErroLut::Cubo(ErroCubo::TamanhoInvalido(300)),
        ErroLut::Cubo(ErroCubo::LinhaInvalida(7)),
        ErroLut::Cubo(ErroCubo::Contagem {
            esperado: 8,
            lido: 5,
        }),
    ]
    .map(|e| frase(&e));
    for (i, f) in frases.iter().enumerate() {
        assert!(!f.starts_with("app.motion."), "chave sem texto: {f}");
        assert!(frases[..i].iter().all(|o| o != f), "frase repetida: {f}");
    }
    assert!(frases[3].contains("300") && frases[4].contains('7') && frases[5].contains('5'));
}

/// O disco lê-se UMA vez por ficheiro: o mesmo caminho com a mesma data não relê.
#[test]
fn the_same_file_is_read_once() {
    let (mut g, no) = grafo();
    g.set_text_param(no, LUT, ficheiro("uma_vez", &cubo_2()));
    let mut l = MotionLente::default();
    l.prepara(&g, 0);
    l.prepara(&g, 1);
    l.prepara(&g, 2);
    assert_eq!(l.geracoes, 1);
    g.set_text_param(no, LUT, ficheiro("outro", &cubo_2()));
    l.prepara(&g, 3);
    assert_eq!(l.geracoes, 2, "outro caminho relê");
}

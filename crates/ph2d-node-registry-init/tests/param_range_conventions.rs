//! **O SLIDER ARRASTA ONDE A MÃO TRABALHA; A CAIXA DIGITA ATÉ O TETO** (doc 88 §11).
//!
//! O teto de um param responde a DUAS perguntas que não são a mesma — *até onde o artista
//! arrasta?* e *o que a máquina aguenta?* —, e escrevê-las num campo só faz o slider virar a
//! régua da MÁQUINA. É a inversão do §0 do CLAUDE.md espelhada na UI, e ela tem número: com
//! `motion.emitter.max` arrastando até 4.194.304, **um pixel do track valia 27.000 partículas**
//! e o default de 512 não cabia no primeiro cinquentavo de pixel.
//!
//! ⚠️ **A barra é DERIVADA da geometria, não escolhida.** O track do painel mede
//! [`TRACK_PX`] e o mapeamento é estritamente linear (`row_value` = `min + track·span`), então
//! `span / TRACK_PX` **é o menor passo que um arrasto consegue** — e acima de
//! `span / default = TRACK_PX` esse passo mínimo passa do próprio default.

use ph2d_node_registry::{NodeRegistry, ParamWidget};

/// A largura do track em pixels, como o painel o desenha: `inner_w` (320 do dock menos o pad
/// da moldura) − `DEFAULT_LABEL_W` (70) − a caixa numérica (`MIN_W_PX`, 72) ≈ **154**.
///
/// ⚠️ Ela é conservadora **de propósito**: uma janela mais estreita dá um track MENOR, e um
/// track menor só torna a barra mais frouxa do que o produto de fato precisa. Um número maior
/// aqui é que seria desonesto.
const TRACK_PX: f32 = 154.0;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("every node registers");
    reg
}

/// **Um pixel de arrasto não pode mover mais que o próprio default.**
#[test]
fn the_slider_drags_where_the_hand_works() {
    let reg = registry();
    let mut scanned = 0usize;
    let mut wrong = Vec::new();

    for m in reg.manifests() {
        let hints = reg.param_ui(m.id).unwrap_or(&[]);
        for p in m.params {
            let Some(h) = hints.iter().find(|h| h.param == p.name) else {
                continue;
            };
            if !matches!(
                h.widget,
                ParamWidget::Slider | ParamWidget::IntSlider | ParamWidget::Angle
            ) {
                continue;
            }
            // ⚠️ Default ZERO fica de fora, e não é isenção preguiçosa: `amount = 0` é o
            // NEUTRO de um efeito, e a razão `span/default` não é sequer definida ali. Se a
            // resposta de um knob neutro-em-zero é torta, isso é a LEI dele (doc 88 §10), e
            // quem mede é a porta do produto — não esta tabela.
            if p.default <= 0.0 {
                continue;
            }
            scanned += 1;
            let step_per_px = (h.max - h.min) / TRACK_PX;
            if step_per_px > p.default {
                wrong.push(format!(
                    "{}.{} (default {}, curso {}..{} => {:.1} por pixel)",
                    m.name, p.name, p.default, h.min, h.max, step_per_px
                ));
            }
        }
    }

    // Controle positivo: uma varredura vazia passaria por vácuo.
    assert!(
        scanned >= 150,
        "a varredura olhou {scanned} params -- o scanner quebrou, nao o catalogo"
    );
    assert!(
        wrong.is_empty(),
        "um pixel de arrasto move mais que o proprio default: o teto SOFT ainda e um numero de \
         RECURSO onde devia ser faixa de autoria -- baixe o slider e ponha o teto de hoje num \
         `ParamHardMax` (nada fica inalcancavel): {wrong:?}"
    );
}

/// **O teto DURO alarga, nunca estreita** — a própria doc do `ParamHardMax` diz que ele tem de
/// ser ≥ o `max` do hint "para querer dizer alguma coisa". Um hard ABAIXO do soft seria um
/// slider que arrasta para além do que a caixa aceita.
#[test]
fn a_hard_ceiling_only_ever_widens_the_slider() {
    let reg = registry();
    let mut pairs = 0usize;
    let mut wrong = Vec::new();

    for m in reg.manifests() {
        for h in reg.param_ui(m.id).unwrap_or(&[]) {
            let Some(hard) = reg.param_hard_max(m.id, h.param) else {
                continue;
            };
            pairs += 1;
            if hard < h.max {
                wrong.push(format!(
                    "{}.{} soft {} > hard {hard}",
                    m.name, h.param, h.max
                ));
            }
        }
    }

    assert!(
        pairs >= 10,
        "achei {pairs} pares soft/hard -- o scanner quebrou, nao o catalogo"
    );
    assert!(
        wrong.is_empty(),
        "teto duro ESTREITANDO o slider: {wrong:?}"
    );
}

/// **Declarar não é registrar.** Um `static PARAM_HARD_MAX` sem a chamada de registro é o modo
/// de falha exato deste padrão: a tabela existe, o `cargo` não reclama (ela é lida pelo próprio
/// crate em gates), e o painel nunca a vê — o artista fica com o slider estreito **e** sem o
/// teto digitável, que é pior que não ter feito a wave.
#[test]
fn every_declared_hard_max_is_registered() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/");
    let mut declared = 0usize;
    let mut orphans = Vec::new();

    for entry in std::fs::read_dir(root).expect("crates/ is readable") {
        let dir = entry.expect("dir entry").path();
        let name = dir.file_name().and_then(|n| n.to_str()).unwrap_or_default();
        if !name.starts_with("ph2d-node-") {
            continue;
        }
        let src = dir.join("src");
        let Ok(files) = std::fs::read_dir(&src) else {
            continue;
        };
        let mut body = String::new();
        for f in files.flatten() {
            if f.path().extension().is_some_and(|e| e == "rs") {
                body.push_str(&std::fs::read_to_string(f.path()).unwrap_or_default());
            }
        }
        if !body.contains("static PARAM_HARD_MAX") {
            continue;
        }
        declared += 1;
        if !body.contains("register_param_hard_max") {
            orphans.push(name.to_string());
        }
    }

    assert!(
        declared >= 10,
        "achei {declared} crates com a tabela -- o scanner quebrou, nao o catalogo"
    );
    assert!(
        orphans.is_empty(),
        "PARAM_HARD_MAX declarado e NUNCA registrado (o painel nunca o ve): {orphans:?}"
    );
}

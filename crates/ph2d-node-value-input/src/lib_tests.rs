use super::*;
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::Graph;

struct Reg;
impl OpResolver for Reg {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        (ty == MANIFEST.id).then_some(&ValueInput as &dyn NodeOp)
    }
}

fn saidas(cook: &mut Cook, g: &Graph, n: ph2d_nodegraph::graph::NodeId) -> Vec<Vec<f32>> {
    let out = cook.cook(g, &Reg, n, 0.0).unwrap();
    out.iter()
        .map(|v| match v.as_stream().get("v") {
            Some(Column::Scalar(x)) => x.clone(),
            _ => vec![],
        })
        .collect()
}

fn publica(cook: &mut Cook, nome: &str, v: &[f32]) {
    cook.set_external(
        input_of(nome),
        Stream::new(v.len()).with(INPUT_VALUE, Column::Scalar(v.to_vec())),
    );
}

fn no(g: &mut Graph, kind: i32, nome: &str) -> ph2d_nodegraph::graph::NodeId {
    let n = g.add_node("value.input");
    g.set_param(n, KIND, kind as f32);
    g.set_text_param(n, NAME_KEY, nome);
    n
}

/// ⭐ Sem publicação vale o CARTÃO; publicado, vale o publicado — e volta a cozer quando ele muda
/// (o memo pela revisão).
#[test]
fn the_card_answers_until_the_game_publishes_and_then_the_game_does() {
    let mut g = Graph::new();
    let n = no(&mut g, KIND_NUMBER, "vida");
    g.set_param(n, VALUE_PARAM, 0.25);
    let mut cook = Cook::new();
    assert_eq!(saidas(&mut cook, &g, n)[0], vec![0.25]);
    publica(&mut cook, "vida", &[0.8]);
    assert_eq!(saidas(&mut cook, &g, n)[0], vec![0.8]);
    publica(&mut cook, "vida", &[0.3]);
    let s = saidas(&mut cook, &g, n);
    assert_eq!(s[0], vec![0.3]);
    assert_eq!(
        s[1..4],
        [vec![0.3], vec![0.3], vec![0.3]],
        "o número como cinza"
    );
    assert_eq!(s[4], vec![1.0]);
}

/// O nome é a morada: outra entrada com outro nome não chega, e um objecto chamado igual também
/// não (o espaço reservado).
#[test]
fn only_the_input_with_this_name_reaches_this_node() {
    let mut g = Graph::new();
    let n = no(&mut g, KIND_NUMBER, " vida ");
    let mut cook = Cook::new();
    publica(&mut cook, "mana", &[0.9]);
    cook.set_external("vida", Stream::new(1).with("v", Column::Scalar(vec![0.7])));
    assert_eq!(saidas(&mut cook, &g, n)[0], vec![0.0]);
    publica(&mut cook, "vida", &[0.6]);
    assert_eq!(saidas(&mut cook, &g, n)[0], vec![0.6], "o nome aparado");
}

#[test]
fn a_toggle_is_zero_or_one_and_a_trigger_is_the_published_pulse() {
    let mut g = Graph::new();
    let t = no(&mut g, KIND_TOGGLE, "ligado");
    let p = no(&mut g, KIND_TRIGGER, "dano");
    let mut cook = Cook::new();
    assert_eq!(saidas(&mut cook, &g, t)[0], vec![0.0]);
    g.set_param(t, STATE, 1.0);
    assert_eq!(saidas(&mut cook, &g, t)[0], vec![1.0]);
    publica(&mut cook, "ligado", &[0.2, 0.7]);
    assert_eq!(
        saidas(&mut cook, &g, t)[0],
        vec![0.0, 1.0],
        "uma linha por escritor"
    );
    assert_eq!(saidas(&mut cook, &g, p)[0], vec![0.0], "sem disparo, zero");
    publica(&mut cook, "dano", &[1.0]);
    assert_eq!(saidas(&mut cook, &g, p)[0], vec![1.0]);
}

/// A cor: o cartão, depois a publicada; `out` é a luminosidade OKLab (branco 1, preto 0).
#[test]
fn a_colour_input_drives_four_channels_and_reports_its_lightness() {
    let mut g = Graph::new();
    let c = no(&mut g, KIND_COLOR, "time");
    let mut cook = Cook::new();
    let s = saidas(&mut cook, &g, c);
    assert!(
        (s[0][0] - 1.0).abs() < 1e-4,
        "o branco de fábrica tem L = 1: {}",
        s[0][0]
    );
    cook.set_external(
        input_of("time"),
        Stream::new(1).with(INPUT_COLOR, Column::Vec4(vec![[0.0, 0.0, 1.0, 0.5]])),
    );
    let s = saidas(&mut cook, &g, c);
    assert_eq!([s[1][0], s[2][0], s[3][0], s[4][0]], [0.0, 0.0, 1.0, 0.5]);
    let l_azul = ph2d_color::perceptual::to_oklab([0.0, 0.0, 1.0])[0];
    assert_eq!(s[0][0], l_azul);
}

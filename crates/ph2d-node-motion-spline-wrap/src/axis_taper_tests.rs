//! **O EIXO e o AFUNILAMENTO** (doc 89 folha 04 — as duas últimas células da folha).
//!
//! ⚠️ **Os gates correm pelo `eval`, não pela função pura**, e é de propósito: o afunilamento
//! decide *escrever ou COPIAR* a coluna `size`, e essa decisão vive no `eval`. Um gate sobre
//! `wrap_with_frame` provaria a aritmética e deixaria a metade que importa por testar — a
//! causa nº 1 da semana perdida no Painter.

use super::*;
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph};

/// A fonte: um layout que o teste escolhe — uma FILA (ao longo de X) ou uma COLUNA (ao longo
/// de Y) —, opcionalmente com `size` próprio e com uma máscara.
static SRC: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.spline_wrap.axis.src"),
    name: "motion.spline_wrap.axis.src",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        // `0` fila em X (o layout de sempre) · `1` coluna em Y.
        ParamSpec {
            name: "column",
            default: 0.0,
        },
        // `< 0` ⇒ sem coluna `size`; `≥ 0` ⇒ um `size` uniforme com esse valor.
        ParamSpec {
            name: "own_size",
            default: -1.0,
        },
        // `< 0` ⇒ sem coluna `falloff`.
        ParamSpec {
            name: "mask",
            default: -1.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// Quantos elementos o layout tem.
const N: usize = 7;

struct Src;
impl NodeOp for Src {
    fn manifest(&self) -> &'static NodeManifest {
        &SRC
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let column = ctx.param("column") != 0.0;
        let p: Vec<P2> = (0..N)
            .map(|i| {
                let t = i as f32 - (N as f32 - 1.0) * 0.5;
                if column { [0.0, t] } else { [t, 0.0] }
            })
            .collect();
        let mut s = Stream::new(N).with("P", Column::Vec2(p));
        let own = ctx.param("own_size");
        if own >= 0.0 {
            s.set("size", Column::Vec2(vec![[own, own]; N]));
        }
        let m = ctx.param("mask");
        if m >= 0.0 {
            s.set("falloff", Column::Scalar(vec![m; N]));
        }
        ctx.emit(s);
    }
}

struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == SRC.id => Some(&Src),
            t if t == MANIFEST.id => Some(&MotionSplineWrap),
            _ => None,
        }
    }
}

/// O que o teste autora no nó, além dos defaults.
#[derive(Default, Clone, Copy)]
struct Setup {
    column: bool,
    own_size: f32,
    mask: f32,
    direction: f32,
    size_start: f32,
    size_end: f32,
    offset: f32,
}

impl Setup {
    fn new() -> Self {
        Self {
            own_size: -1.0,
            mask: -1.0,
            size_start: 1.0,
            size_end: 1.0,
            ..Self::default()
        }
    }
}

/// Coze o embrulho sobre um **S pronunciado** (os defaults do nó) e devolve o stream.
fn wrapped(set: Setup) -> Stream {
    let mut g = Graph::new();
    let src = g.add_node("motion.spline_wrap.axis.src");
    g.set_param(src, "column", f32::from(u8::from(set.column)));
    g.set_param(src, "own_size", set.own_size);
    g.set_param(src, "mask", set.mask);
    let sw = g.add_node("motion.spline_wrap");
    g.set_param(sw, taper::DIRECTION, set.direction);
    g.set_param(sw, taper::SIZE_TAPER.0, set.size_start);
    g.set_param(sw, taper::SIZE_TAPER.1, set.size_end);
    g.set_param(sw, "offset", set.offset);
    g.connect(Edge {
        from: (src, 0),
        to: (sw, 0),
        delayed: false,
    })
    .expect("in");
    let mut cook = Cook::new();
    cook.cook(&g, &Ops, sw, 0.0).expect("cook")[0]
        .as_stream()
        .clone()
}

fn pos(s: &Stream) -> Vec<P2> {
    match s.get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    }
}

fn size(s: &Stream) -> Option<Vec<[f32; 2]>> {
    match s.get("size") {
        Some(Column::Vec2(v)) => Some(v.clone()),
        _ => None,
    }
}

/// **Quanto o layout embrulhado ENTORTA** — o maior desvio à corda que liga a primeira à
/// última posição, em frações da própria corda. `0` é uma reta; a cúbica em S dos defaults
/// entorta muito.
///
/// ⚠️ **A primeira versão desta régua media a EXTENSÃO, e ela acusava código correto.** Uma
/// coluna que o nó não sabe embrulhar não colapsa num ponto: ela empilha-se ao longo de UMA
/// normal e mede exactamente a sua própria altura (`6,0000`, medido). *Extensão é uma
/// propriedade que uma reta também tem* — a pergunta era se o layout SEGUE a curva, e a
/// resposta a essa é a forma, não o tamanho.
fn bentness(p: &[P2]) -> f32 {
    let (a, b) = (p[0], p[p.len() - 1]);
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let chord = dx.hypot(dy);
    if chord < 1e-6 {
        return 0.0;
    }
    p.iter()
        .map(|q| ((q[0] - a[0]) * dy - (q[1] - a[1]) * dx).abs() / chord)
        .fold(0.0_f32, f32::max)
        / chord
}

// ─────────────────────────── o eixo ───────────────────────────

/// ⭐ **O DEFEITO E A CURA, lado a lado.** Uma COLUNA de elementos (todos com o mesmo `x`) não
/// tem extensão no eixo que o nó lia: `w` cai abaixo do `EPS`, todo `u` vira `0,5`, e os sete
/// **empilham-se ao longo de UMA normal** — um segmento RETO, que é o mesmo que dizer que a
/// curva deixou de existir para eles. Com o eixo em `90°` a mesma coluna percorre a curva.
///
/// ⚠️ É a fixtura que a lei antiga **não podia** exprimir — a régua da folha: um layout que a
/// versão anterior desenhava certo não falsifica um eixo novo.
#[test]
fn a_column_of_elements_goes_straight_on_the_old_axis_and_follows_the_curve_on_the_new_one() {
    let mut set = Setup::new();
    set.column = true;
    let straight = bentness(&pos(&wrapped(set)));
    set.direction = 90.0;
    let followed = bentness(&pos(&wrapped(set)));
    assert!(
        straight < 1e-4,
        "sem o eixo, a coluna tinha de sair RETA (entortou {straight:.5})"
    );
    assert!(
        followed > 0.02,
        "com o eixo a 90 graus ela tinha de seguir a curva (entortou so' {followed:.5})"
    );
}

/// **E a FILA de sempre é intocada pelo default** — o controle que impede o gate acima de
/// provar só metade, e que mostra a SIMETRIA do achado (a 90° é a fila que sai reta).
#[test]
fn the_row_the_node_always_wrapped_is_untouched_by_the_default() {
    let follows = bentness(&pos(&wrapped(Setup::new())));
    assert!(
        follows > 0.02,
        "a fila continua a seguir a curva ({follows:.5})"
    );
    let mut set = Setup::new();
    set.direction = 90.0;
    let straight = bentness(&pos(&wrapped(set)));
    assert!(
        straight < 1e-4,
        "a 90 graus a FILA e' que sai reta -- a simetria do achado ({straight:.5})"
    );
}

/// **`direction = 0` não toca no layout, AO BIT** — medido contra a lei escrita à mão.
///
/// ⚠️ O oráculo vive aqui e não chama a função sob teste: um oráculo derivado dela concorda
/// por construção, inclusive quando as duas estão erradas.
#[test]
fn the_default_axis_is_the_law_that_shipped_bit_for_bit() {
    let p: Vec<P2> = (0..N)
        .map(|i| [i as f32 - (N as f32 - 1.0) * 0.5, 0.4])
        .collect();
    let cp: [P2; 4] = [[-3.0, -1.5], [-1.0, 2.0], [1.0, -2.0], [3.0, 1.5]];
    let curve = Curve::cubic(&cp);
    let map = ArcMap {
        from: 0.0,
        to: 1.0,
        offset: 0.0,
    };
    // A lei de sempre: `u` sobre o `x` CRU, desvio pelo `y` CRU.
    let (mut xmin, mut xmax) = (f32::MAX, f32::MIN);
    for q in &p {
        xmin = xmin.min(q[0]);
        xmax = xmax.max(q[0]);
    }
    let w = xmax - xmin;
    let then: Vec<P2> = p
        .iter()
        .map(|q| {
            let u = if w < EPS { 0.5 } else { (q[0] - xmin) / w };
            let (b, _t, un) = curve.frame_at(map.s_at(u));
            [b[0] + un[0] * q[1], b[1] + un[1] * q[1]]
        })
        .collect();
    let now = wrap_with_frame(&p, &curve, 1.0, map, false, 0.0, &[], &[]).0;
    for (i, (a, b)) in now.iter().zip(&then).enumerate() {
        assert_eq!(
            (a[0].to_bits(), a[1].to_bits()),
            (b[0].to_bits(), b[1].to_bits()),
            "elemento {i}: {a:?} contra {b:?}"
        );
    }
}

// ─────────────────────── o afunilamento ───────────────────────

/// **Sem afunilamento, a coluna `size` é COPIADA** — a lei estrutural do `follow_rotation`,
/// aplicada à outra coluna.
#[test]
fn a_flat_taper_passes_the_size_column_through_untouched() {
    // Sem `size` na entrada, não nasce nenhum.
    assert!(
        size(&wrapped(Setup::new())).is_none(),
        "a coluna nao e' inventada"
    );
    // Com `size` na entrada, ele sai idêntico AO BIT.
    let mut set = Setup::new();
    set.own_size = 0.75;
    let out = size(&wrapped(set)).expect("o size proprio sobrevive");
    for (i, v) in out.iter().enumerate() {
        assert_eq!(
            (v[0].to_bits(), v[1].to_bits()),
            (0.75_f32.to_bits(), 0.75_f32.to_bits()),
            "elemento {i}: {v:?}"
        );
    }
}

/// ⭐ **A cauda AFINA ao longo da curva** — e o primeiro elemento fica com a espessura cheia.
#[test]
fn the_taper_thins_the_layout_along_the_curve() {
    let mut set = Setup::new();
    set.size_end = 0.2;
    let out = size(&wrapped(set)).expect("com afunilamento a coluna nasce");
    assert!(
        (out[0][0] - 1.0).abs() < 1e-4,
        "a cabeca fica cheia: {:?}",
        out[0]
    );
    assert!(
        (out[N - 1][0] - 0.2).abs() < 1e-3,
        "a cauda tinha de afinar ate' 0,2: {:?}",
        out[N - 1]
    );
    // E é monótono pelo meio — uma cauda que afina não engrossa no caminho.
    for w in out.windows(2) {
        assert!(w[1][0] <= w[0][0] + 1e-5, "subiu: {:?} -> {:?}", w[0], w[1]);
    }
    // Os dois eixos do `size` andam juntos: o afunilamento é um multiplicador, não uma lente.
    for (i, v) in out.iter().enumerate() {
        assert!(
            (v[0] - v[1]).abs() < 1e-6,
            "elemento {i} anisotropico: {v:?}"
        );
    }
}

/// ⭐⭐ **O PERFIL ESTÁ PREGADO NA CURVA, e este gate separa os dois desenhos possíveis.**
///
/// Deslizar o `offset` move o layout ao longo do arco; se o afunilamento corresse sobre o `u`
/// (a coordenada do LAYOUT), ele viajaria junto e as espessuras não mudariam. Correndo sobre o
/// `s` (a posição de ARCO), o layout passa POR ELE e engrossa ao entrar na parte grossa — que
/// é a leitura da referência e a única que a composição a montante não consegue.
#[test]
fn the_taper_is_pinned_to_the_curve_so_sliding_the_layout_through_it_changes_the_thickness() {
    let mut set = Setup::new();
    set.size_start = 2.0;
    set.size_end = 0.5;
    let at_home = size(&wrapped(set)).expect("nasce");
    set.offset = 0.4;
    let slid = size(&wrapped(set)).expect("nasce");
    let moved = at_home
        .iter()
        .zip(&slid)
        .map(|(a, b)| (a[0] - b[0]).abs())
        .fold(0.0_f32, f32::max);
    assert!(
        moved > 0.3,
        "deslizar o layout tinha de mudar a espessura (maior desvio {moved:.4}) -- \
         se nao mudou, o perfil esta' preso ao LAYOUT e nao a' curva"
    );
    // CONTROLE: o `offset` de facto moveu o layout (senão o gate acima mediria um no-op).
    let mut plain = Setup::new();
    let a = pos(&wrapped(plain));
    plain.offset = 0.4;
    let b = pos(&wrapped(plain));
    let dp = a
        .iter()
        .zip(&b)
        .map(|(p, q)| (p[0] - q[0]).abs().max((p[1] - q[1]).abs()))
        .fold(0.0_f32, f32::max);
    assert!(dp > 0.2, "CONTROLE: o offset nao moveu nada ({dp:.4})");
}

/// **O afunilamento MULTIPLICA o `size` que já lá estava** — ele compõe com um `motion.scale`
/// a montante em vez de o atropelar (a mesma lei do `rot`, que SOMA).
#[test]
fn the_taper_multiplies_the_size_the_stream_already_had() {
    let mut set = Setup::new();
    set.own_size = 3.0;
    set.size_end = 0.5;
    let out = size(&wrapped(set)).expect("nasce");
    assert!(
        (out[0][0] - 3.0).abs() < 1e-3,
        "a cabeca mantem o size proprio: {:?}",
        out[0]
    );
    assert!(
        (out[N - 1][0] - 1.5).abs() < 1e-2,
        "e a cauda e' metade DELE, nao metade de um: {:?}",
        out[N - 1]
    );
}

/// **O afunilamento honra a MESMA máscara que a posição.** Um elemento meio-embrulhado é
/// meio-afunilado — mascarar um e não o outro deixaria a espessura a mentir exactamente onde
/// o falloff está a funcionar.
#[test]
fn the_taper_honours_the_same_mask_as_the_position() {
    let mut set = Setup::new();
    set.size_end = 0.0;
    set.mask = 0.0;
    let out = size(&wrapped(set)).expect("a coluna nasce na mesma -- o param esta' autorado");
    for (i, v) in out.iter().enumerate() {
        assert!(
            (v[0] - 1.0).abs() < 1e-6,
            "com a mascara em zero nada afina: elemento {i} = {v:?}"
        );
    }
    // CONTROLE: sem a máscara, o mesmo afunilamento apaga a cauda.
    set.mask = -1.0;
    let live = size(&wrapped(set)).expect("nasce");
    assert!(
        live[N - 1][0] < 0.05,
        "CONTROLE: sem mascara a cauda some ({:?})",
        live[N - 1]
    );
}

/// **Os quatro params novos têm painel** — sem hint um knob é inalcançável, que é a metade
/// que o doc 90 (caça aos knobs mortos) mede.
#[test]
fn every_new_knob_is_reachable_from_the_panel() {
    for p in [
        taper::DIRECTION,
        taper::SIZE_TAPER.0,
        taper::SIZE_TAPER.1,
        taper::SIZE_TAPER.2,
    ] {
        assert!(
            MANIFEST.params.iter().any(|s| s.name == p),
            "`{p}` nao esta' no manifesto"
        );
        assert!(
            PARAM_HINTS.iter().any(|h| h.param == p),
            "`{p}` nao tem hint de painel"
        );
    }
}

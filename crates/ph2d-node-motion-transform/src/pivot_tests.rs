//! **O PIVÔ DA ESCALA DE LAYOUT** (doc 89 folha 05 — o P0).
//!
//! A célula descrevia o sintoma pela cena, e é a melhor descrição que existe:
//! *"um grid centrado em (5,0) com `scale=2` **também translada** para (10,0)"*.
//! É isso que estes gates medem — não a existência de um param.

use super::*;
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph};

/// Três elementos centrados em `(5, 0)` — a cena exacta da célula.
static SRC: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.transform.pivot.src"),
    name: "motion.transform.pivot.src",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[],
    lowerings: &[LoweringKind::Cpu],
};
struct Src;
impl NodeOp for Src {
    fn manifest(&self) -> &'static NodeManifest {
        &SRC
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(Stream::new(3).with("P", Column::Vec2(vec![[4.0, 0.0], [5.0, 0.0], [6.0, 0.0]])));
    }
}
struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == SRC.id => Some(&Src),
            t if t == MANIFEST.id => Some(&MotionTransform),
            _ => None,
        }
    }
}

/// Escala o layout pela porta do cook e devolve as posições.
fn scaled(scale: f32, mode: f32, px: f32, py: f32, ox: f32) -> Vec<[f32; 2]> {
    let mut g = Graph::new();
    let src = g.add_node("motion.transform.pivot.src");
    let tr = g.add_node("motion.transform");
    g.set_param(tr, "scale", scale);
    g.set_param(tr, "pivot_mode", mode);
    g.set_param(tr, "pivot_x", px);
    g.set_param(tr, "pivot_y", py);
    g.set_param(tr, "offset_x", ox);
    g.connect(Edge {
        from: (src, 0),
        to: (tr, 0),
        delayed: false,
    })
    .expect("in");
    let mut cook = Cook::new();
    let out = cook.cook(&g, &Ops, tr, 0.0).expect("cook");
    match out[0].as_stream().get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => panic!("P"),
    }
}

/// **O sintoma que a célula descreve, e ele é real.**
///
/// Com o pivô na origem — o que este nó sempre fez — dobrar a escala de um layout
/// centrado em `(5,0)` **também o translada** para `(10,0)`. O gate pina isso em
/// vez de o corrigir: é o comportamento default e todo documento já autorado o
/// tem, então mudá-lo em silêncio seria mover a arte de todo mundo.
#[test]
fn the_world_origin_pivot_translates_the_layout_and_that_is_the_default() {
    let p = scaled(2.0, 0.0, 0.0, 0.0, 0.0);
    assert_eq!(p, vec![[8.0, 0.0], [10.0, 0.0], [12.0, 0.0]]);
    let centre = p[1][0];
    assert!(
        (centre - 10.0).abs() < 1e-6,
        "o centro anda de 5 para 10 -- o sintoma da celula: {centre}"
    );
}

/// **Com o pivô no centro, a escala ESPALHA sem transladar.**
#[test]
fn a_pivot_at_the_centre_spreads_without_moving_the_layout() {
    let p = scaled(2.0, 1.0, 5.0, 0.0, 0.0);
    assert_eq!(p, vec![[3.0, 0.0], [5.0, 0.0], [7.0, 0.0]]);
    assert_eq!(p[1][0], 5.0, "o centro fica onde estava");
}

/// **O modo Centroid encontra esse centro sozinho** — e é o que faz disto um P0
/// e não um P1: o número não é digitado, é o layout que responde.
///
/// ⚠️ E ele só passou a ser POSSÍVEL de pedir em 2026-08-12: a célula media o
/// centroide como *inexprimível* (*"`P` não chega ao domínio de valor por rota
/// nenhuma"*), verdade até a `motion.expression` ganhar as lanes `x`/`y`. Como
/// modo, custa um controle; como cadeia, custa quatro nós — duas vezes, uma por
/// eixo.
#[test]
fn the_centroid_mode_finds_the_centre_without_being_told() {
    let typed = scaled(2.0, 1.0, 5.0, 0.0, 0.0);
    let found = scaled(2.0, 2.0, 0.0, 0.0, 0.0);
    assert_eq!(
        found, typed,
        "o centroide de [4,5,6] e 5, e o resultado e o mesmo do 5 digitado"
    );
    // E ele SEGUE o layout: mova a cena e o pivo move junto, que e a diferenca
    // inteira para um numero digitado.
    let moved = {
        let mut g = Graph::new();
        let src = g.add_node("motion.transform.pivot.src");
        // Um transform a montante desloca a cena em +10 antes do que escalamos.
        let pre = g.add_node("motion.transform");
        g.set_param(pre, "offset_x", 10.0);
        let tr = g.add_node("motion.transform");
        g.set_param(tr, "scale", 2.0);
        g.set_param(tr, "pivot_mode", 2.0);
        g.connect(Edge {
            from: (src, 0),
            to: (pre, 0),
            delayed: false,
        })
        .unwrap();
        g.connect(Edge {
            from: (pre, 0),
            to: (tr, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, tr, 0.0).unwrap();
        match out[0].as_stream().get("P") {
            Some(Column::Vec2(v)) => v.clone(),
            _ => panic!("P"),
        }
    };
    assert_eq!(
        moved,
        vec![[13.0, 0.0], [15.0, 0.0], [17.0, 0.0]],
        "centro em 15, espalhado em torno de si -- um pivo digitado em 5 teria \
         mandado isto para 25"
    );
}

/// **O default é o nó de antes, AO BIT** — e é estrutural, não aritmético.
///
/// ⚠️ `folded_offset` devolve o offset **verbatim** com pivô zero em vez de
/// computar `o + 0·(1−s)`: as duas expressões diferem só no sinal de um zero, que
/// nada enxerga — e é justamente por isso que a identidade tem de ser da
/// ESTRUTURA. Um gate que só comparasse números ficaria verde sobre a versão
/// aritmética e não diria nada sobre a promessa.
#[test]
fn the_default_pivot_is_the_node_that_shipped_bit_for_bit() {
    for (scale, ox) in [(2.0f32, 0.0f32), (0.5, 3.0), (1.0, -7.5), (0.0, 1.0)] {
        let with_mode = scaled(scale, 0.0, 0.0, 0.0, ox);
        let bits: Vec<u32> = with_mode.iter().flat_map(|p| [p[0].to_bits()]).collect();
        // A expressao que shipava, computada aqui sem passar pelo pivo.
        let want: Vec<u32> = [4.0f32, 5.0, 6.0]
            .iter()
            .map(|x| (x * scale + ox).to_bits())
            .collect();
        assert_eq!(bits, want, "scale {scale}, offset {ox}");
    }
    // E um offset de zero NEGATIVO -- o unico valor em que a rota aritmetica
    // divergiria, e o motivo de o atalho existir.
    let neg = scaled(2.0, 0.0, 0.0, 0.0, -0.0);
    assert_eq!(
        neg[0][0].to_bits(),
        (4.0f32 * 2.0 + -0.0).to_bits(),
        "offset -0.0 atravessa verbatim"
    );
}

/// **`folded_offset` é a lei, e ela é a álgebra do pivô — não uma segunda
/// expressão por elemento.**
#[test]
fn the_pivot_folds_into_the_offset_instead_of_becoming_a_second_affine() {
    // (p - c)*s + c + o  ==  p*s + (o + c(1-s))
    for (s, c, o) in [(2.0f32, 5.0f32, 1.0f32), (0.5, -3.0, 0.0), (1.0, 7.0, -2.0)] {
        let (fx, _) = folded_offset(s, s, o, 0.0, [c, 0.0]);
        for p in [-4.0f32, 0.0, 9.5] {
            let folded = p * s + fx;
            let literal = (p - c) * s + c + o;
            assert!(
                (folded - literal).abs() < 1e-4,
                "s={s} c={c} o={o} p={p}: dobrado {folded} contra literal {literal}"
            );
        }
    }
    // E o atalho do neutro devolve o offset intacto.
    assert_eq!(folded_offset(2.0, 2.0, 1.5, -2.5, [0.0, 0.0]), (1.5, -2.5));

    // ⚠️ E AQUI e onde o atalho deixa de ser cosmetico, no unico valor que o
    // separa da rota aritmetica: com `scale < 1` o termo `0 * (1 - s)` e `+0.0`,
    // entao `-0.0 + 0.0` vira `+0.0` e o offset TROCA DE SINAL DE ZERO. Invisivel
    // em qualquer pixel, e exatamente por isso a identidade tem de ser da
    // estrutura -- uma fixture com `scale = 2` NAO ve isto (`0 * (1 - 2)` e
    // `-0.0`, e a soma coincide), e foi assim que a mutacao que apaga o atalho
    // sobreviveu a primeira rodada deste arquivo.
    let (nx, ny) = folded_offset(0.5, 0.5, -0.0, -0.0, [0.0, 0.0]);
    assert_eq!(
        (nx.to_bits(), ny.to_bits()),
        ((-0.0f32).to_bits(), (-0.0f32).to_bits()),
        "o offset atravessa VERBATIM, sinal de zero incluido"
    );
    // A rota aritmetica, escrita aqui para o gate dizer o que evita.
    let arithmetic = -0.0f32 + 0.0 * (1.0 - 0.5);
    assert_eq!(
        arithmetic.to_bits(),
        0.0f32.to_bits(),
        "e ela daria +0.0 -- o mesmo angulo, outro numero"
    );
}

/// **Um stream sem posições cai na origem em vez de produzir NaN.**
///
/// A média de nada não é zero, é ausente — e um `0/0` no pivô poria `NaN` em todo
/// `P`, que é a arte a desaparecer sem nada dito. A origem é o transform que o
/// artista ainda vê.
#[test]
fn a_centroid_of_nothing_falls_back_to_the_origin() {
    let origin = [0.0f32, 0.0];
    assert_eq!(
        Pivot::Centroid.resolve([9.0, 9.0], positions(&Stream::new(0))),
        origin,
        "vazio nao tem centro"
    );
    let no_p = Stream::new(2).with("size", Column::Scalar(vec![1.0, 2.0]));
    assert_eq!(
        Pivot::Centroid.resolve([9.0, 9.0], positions(&no_p)),
        origin,
        "sem coluna P nao ha o que mediar"
    );
    // E a media de posicoes reais e a media.
    let three = Stream::new(3).with("P", Column::Vec2(vec![[4.0, 1.0], [5.0, 2.0], [6.0, 3.0]]));
    assert_eq!(
        Pivot::Centroid.resolve([9.0, 9.0], positions(&three)),
        [5.0, 2.0]
    );
}

/// ⭐⭐ **O CENTROIDE CHEGA AO DISPOSITIVO** (ciclo 3, W1 — [doc 106]).
///
/// ⚠️ **Esta era a recusa mais bem escrita do módulo, e estava ERRADA por um dia
/// que já tinha passado.** O doc-comment dela dizia: *«uma redução sobre o stream
/// inteiro não é um mapa por elemento… o canal `reduce → broadcast → map` que os
/// deformadores usam é o que a levantaria, e isso é uma wave, não uma linha»* —
/// e o canal **já tinha shipado** (GPU/M5), com o `motion.spherize` a medir o
/// centroide por duas somas no dispositivo desde então. *Uma recusa que nomeia
/// o próprio mecanismo da cura tem de ser reconferida quando esse mecanismo
/// nasce* (§0.0: quem move o número que tornava algo inalcançável reconfere a
/// nota).
///
/// O custo nomeado que ela declarava — *«um layout pivotando no próprio centro
/// perde a residência de GPU neste nó»* — era o custo REAL, e ele desaparece.
///
/// [doc 106]: ../../../docs/Motion%20Nodes/106_ciclo_3_transformes_e_deformadores.md
#[test]
fn the_kernel_takes_every_pivot_mode_including_the_centroid() {
    let ok = |mode: f32| {
        GPU_KERNEL.applicable.is_none_or(|f| {
            f(&(|name: &str| if name == "pivot_mode" { mode } else { 0.0 }) as &dyn Fn(&str) -> f32)
        })
    };
    assert!(ok(0.0), "a origem sao numeros");
    assert!(ok(1.0), "o ponto digitado tambem");
    assert!(
        ok(2.0),
        "o centroide e' uma REDUCAO, e a reducao corre no dispositivo (W1 do ciclo 3)"
    );
}

/// ⭐⭐ **ESTE NÓ NÃO TEM UMA SEGUNDA RESPOSTA A «EM TORNO DE QUÊ»** (ciclo 3, W1).
///
/// As duas metades da pergunta vêm da porta e nada aqui as reescreve: o hospedeiro chama
/// [`ph2d_nodegraph::pivot::PivotMode::resolve`] e o dispositivo declara as
/// [`ph2d_nodegraph::pivot::CENTROID_REDUCES`].
///
/// ⚠️⚠️ **A 1.ª redacção comparava PONTEIROS e reprovou sobre código correcto** — os dois
/// endereços diferem (`0x…21b0` contra `0x…9178`) mesmo com o `static` a ser lido de outra
/// crate, porque o `&[…]` que o inicializa é uma constante **promovida** e o leitor
/// re-materializa-a. *Uma régua de identidade que a linguagem não garante mede o compilador,
/// não o código.* O que fica é a comparação dos CAMPOS, que é a propriedade que interessa: o
/// nó declara **a mesma tabela** — nome, coluna, operador e identidade —, e um sétimo
/// vocabulário para a mesma pergunta reprova aqui.
#[test]
fn the_pivot_vocabulary_is_the_ports_and_not_a_copy() {
    let porta = ph2d_nodegraph::pivot::CENTROID_REDUCES;
    assert_eq!(REDUCES.len(), porta.len(), "a tabela tem de ser a da porta");
    for (a, b) in REDUCES.iter().zip(porta) {
        assert_eq!((a.name, a.column, a.op, a.value), (b.name, b.column, b.op, b.value));
        assert_eq!((a.dim, a.port, a.identity), (b.dim, b.port, b.identity));
    }
    // E os rotulos que o cartao pinta sao os da porta, na ordem da porta.
    let hint = PARAM_HINTS
        .iter()
        .find(|h| h.param == ph2d_nodegraph::pivot::PARAM)
        .expect("o no' declara o param do pivo");
    match hint.widget {
        ph2d_node_registry::ParamWidget::Enum { labels } => {
            assert_eq!(labels, ph2d_nodegraph::pivot::LABELS);
        }
        outro => panic!("o pivot_mode tem de ser um Enum, e' {outro:?}"),
    }
}

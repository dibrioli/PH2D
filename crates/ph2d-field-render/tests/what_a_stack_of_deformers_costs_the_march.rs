//! ⭐⭐⭐ **O QUE UMA PILHA DE DEFORMADORES CUSTA À MARCHA** — report do Enio, 2026-08-31: *«o render
//! fica bom. Algumas combinações muito lentas (`+ Taper` depois de `+ Bend` e `+ Twist`)»*.
//!
//! # ⭐ A régua é uma CONTAGEM, e não um relógio
//!
//! `passos ÷ raios` é determinístico: ele não se mexe com a carga da máquina, e por isso este gate
//! **não** é membro da família de flakes que o `CLAUDE.md` §5 lista. *Um gate de custo que lê o
//! relógio mede a máquina; um que conta passos mede o programa.*
//!
//! # O que a medição diz (caixa `0,35³`, `160²`, `half_extent 1,0`) — 2026-09-01
//!
//! | pilha | raio | caixa | divisor | `‖∇f‖` | folga | passos/raio |
//! |---|---:|---|---:|---:|---:|---:|
//! | `[]` | `0,579` | `0,350×0,350×0,300` | `1,00` | `1,000` | `1,0×` | `6,4` |
//! | `[Bend]` | `0,764` | `0,484×0,350×0,476` | `1,59` | `0,887` | `1,1×` | `10,7` |
//! | `[Twist]` | `0,579` | `0,495×0,495×0,300` | `1,82` | `0,776` | `1,3×` | `14,0` |
//! | `[Taper]` | `0,658` | `0,424×0,350×0,363` | `2,20` | `0,555` | `1,8×` | `16,6` |
//! | `[Bend, Twist]` | `0,764` | `0,597×0,597×0,476` | `3,97` | `0,497` | `2,0×` | `26,9` |
//! | `[Bend, Twist, Taper]` | `1,037` | `0,811×0,597×0,646` | `15,85` | `0,177` | **`5,6×`** | **`91,1`** |
//!
//! ⛔⛔ **O `[Bend, Twist]` SUBIU de `23,4` para `26,9`, e isso é uma CORRECÇÃO a mostrar o preço**
//! (report dos três cilindros cruzados, 2026-09-01): o atalho do `Ball::merge` deitava fora a caixa
//! da bola perdedora, e o **envelope de uma pilha** é uma sequência de `merge`. Com a torção a
//! preservar o raio da dobra, o atalho disparava e o envelope ficava com a caixa de um passo só —
//! logo a parede da dobra media menos do que a região que a marcha percorre. *A catraca apanhou-o,
//! que é o que uma catraca de duas metades existe para fazer.*
//!
//! ⚠️ **A tabela é DERIVADA** — corra a sonda `print_the_cost_table` (`--ignored --nocapture`) antes
//! de a citar. Ela já foi escrita de memória duas vezes.
//!
//! # ⭐⭐⭐ As três jornadas, e o que cada uma tirou
//!
//! | pilha | 30/08 (o 1.º report) | 31/08 (a bola aprende os EIXOS) | **01/09 (a caixa chega aos CONSUMIDORES)** |
//! |---|---:|---:|---:|
//! | `[Bend]` | `72,2` | `22,7` | **`10,7`** |
//! | `[Bend, Twist]` | `233,1` | `55,6` | **`26,9`** |
//! | `[Bend, Twist, Taper]` | `1 543,6` | `635,9` | **`91,1`** |
//! | divisor do trio | `240,29` | `100,87` | **`15,85`** |
//! | folga PROVADA do trio | `24,7×` | `22,3×` | **`5,6×`** |
//!
//! ⇒ **`16,9×`** mais barato do que o que o Enio reportou como *«muito lentas»*, e `7,0×` do que
//! ficou ontem. O trio custa hoje `14×` uma caixa nua, e custava `217×`.
//!
//! # ⭐ A lei que as três jornadas escreveram: *o raio de uma esfera não é a extensão de um eixo*
//!
//! Em 31/08 o bordo aprendeu as meias-extensões por eixo. Em 01/09 elas chegaram aos **três
//! consumidores** que continuavam a ler o raio:
//!
//! | quem lia o raio | o que passou a ler | o que isso vale |
//! |---|---|---|
//! | a caixa de RECORTE (`Ball::aabb`) | as meias-extensões | o raio entra mais tarde e sai mais cedo |
//! | a caixa da peça DOBRADA | o **arco**, por aritmética de intervalos (`bounds_bend`) | `[0,957, ·, 0,957]` para `[0,484, ·, 0,476]` |
//! | a parede do divisor da dobra (`bend_reach`) | a extensão em `X` do recorte | o divisor sai do tecto de `10` |
//! | o factor de secção da inclinação (`k_max`) | a extensão em `Y` | `1,347` para `1,21` |
//! | o raio, DE VOLTA (`Ball::of`) | `min(raio, ‖half‖)` | grátis, e aperta toda a pilha a jusante |
//!
//! # ⛔⛔⛔ E DUAS das cinco furaram — as duas foram medidas, revertidas e escritas
//!
//! - **o alcance da TORÇÃO pela caixa** (`stack::axis_reach`): `1,4×` mais barato e **fura** — 1 pixel
//!   muda ao dividir o passo por quatro. A tabela do `σ` da torção foi calibrada **com a esfera lá
//!   dentro**; apertar a entrada consome a margem que fazia a constante bastar.
//! - **a parede da CURVATURA pela caixa** (`stack_bend::bend_wall`): numa barra `0,10 × 0,10 × 0,80`
//!   o tecto sobe de `1,11` para `9,0` e a peça **enrola-se 1,6 voltas sobre si própria** — a esfera
//!   estava a cobrir o enrolamento, que nenhuma cerca nomeia.
//!
//! ⚠️ **E uma terceira consequência não era um bordo errado:** com a caixa justa o recorte **encosta**
//! na peça, e um traçador de esferas que entra em cima dela lê passo zero e fica parado. A cura é a
//! margem do `ph2d_field_eval::bounds_clip` (`0,01`, varrida), e o preço dela é `+2,1` passos/raio no
//! `[Taper]` e `+0,4` no `[Twist]` — os dois **declarados**, e as únicas células desta tabela que não
//! melhoraram desde 30/08.
//!
//! # ⛔⛔ A causa da folga que SOBRA: os divisores multiplicam-se
//!
//! Cada `step_divisor` é o pior caso do seu operador **sobre a caixa de recorte inteira**. O da
//! dobra é pior junto à parede congelada, o da torção no maior raio, o da inclinação no ápice — três
//! sítios distintos. Multiplicá-los assume que coincidem: `1,57 × 1,82 × 2,20 = 6,3` seria o produto
//! dos factores isolados, e o trio cobra `15,35`, porque cada um é medido no envelope que os outros
//! inflaram.
//!
//! ⚠️ **Três hipóteses foram MEDIDAS e não são a cura:**
//! - **ler a bola CORRENTE em vez do envelope final**: o custo caía `1,6×` e ⛔ dessincroniza os dois
//!   leitores da lei (a `stack` e o `field_shrink`), que o doc de lá chama de uma porta só;
//! - **baixar o `BEND_FOLD_MARGIN`**: funciona e é uma troca de PRODUTO, não uma cura;
//! - **culpar um dos três**: cada factor é honesto sozinho (`1,1×`–`1,8×` de folga).
//!
//! # ⛔⛔⛔ E A CURA ÓBVIA — O DIVISOR POR REGIÃO — ESTÁ REFUTADA POR MEDIÇÃO
//!
//! A ideia era natural: a marcha já especializa a árvore por **ladrilho × fatia de profundidade**, e
//! num ladrilho longe do centro do arco o divisor da dobra devia ser ~`1`. ⇒ medi o `‖∇f‖` do
//! envelope inteiro contra o de cada **oitavo** dele: o envelope inteiro `0,0405`, o **pior** oitavo
//! `0,0416`, o melhor `0,0173`.
//!
//! ⇒ **o pior oitavo mede o mesmo que a caixa toda.** *O desperdício não é espacial*, e cortar o
//! domínio não compra nada. ⚠️ E a mesma sonda achou um bloqueador: a
//! `ph2d_field_eval::RegionCompiler::is_worth_it` devolve `false` sem uma forma de **perfil**, então
//! numa caixa com deformadores a especialização por ladrilho está **desligada por inteiro**.
//!
//! ⇒ o que sobra (`5,4×`) é a **composição**, e apertá-la é demonstrar um bound melhor para o
//! produto dos três mapas — trabalho de matemática, não um botão.

use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Primitive, Unary, UnaryKind, Xform};
use ph2d_field_eval::{Field, hybrid::Registry, safe_march_step};
use ph2d_field_render::{MARCH_RAYS, Orbit, STEP_SAMPLES, trace_stepped_for_test};
use std::sync::atomic::Ordering;

fn vivo(k: UnaryKind) -> Unary {
    use ph2d_field::mods::{BEND_AXIS, TAPER_AXIS, TWIST_AXIS};
    match k {
        UnaryKind::Taper => Unary::Taper {
            slope: 0.6,
            axis: TAPER_AXIS,
        },
        UnaryKind::Twist => Unary::Twist {
            turns: 0.35,
            lower: -2.0,
            upper: 2.0,
            falloff: 0.1,
            axis: TWIST_AXIS,
        },
        _ => Unary::Bend {
            turns: 0.12,
            lower: -2.0,
            upper: 2.0,
            falloff: 0.1,
            axis: BEND_AXIS,
        },
    }
}

fn peca(ms: Vec<UnaryKind>) -> FieldDoc {
    let mut n = Node::new(
        Xform::IDENTITY,
        NodeKind::Leaf(Primitive::Box {
            half: [0.35, 0.35, 0.30],
            round: 0.0,
            chamfer: 0.0,
        }),
    );
    n.mods = ms.into_iter().map(vivo).collect();
    FieldDoc::new(vec![n], NodeId(0)).expect("peça")
}

/// Quantos passos a marcha anda por raio — **uma contagem**, ver o doc do módulo.
fn passos_por_raio(doc: &FieldDoc) -> f64 {
    let reg = Registry::default();
    let cam = Orbit {
        half_extent: 1.0,
        ..Orbit::default()
    };
    STEP_SAMPLES.store(0, Ordering::Relaxed);
    MARCH_RAYS.store(0, Ordering::Relaxed);
    let _ = trace_stepped_for_test(doc, &reg, &cam, 160, 160, safe_march_step(doc));
    let passos = STEP_SAMPLES.load(Ordering::Relaxed);
    let raios = MARCH_RAYS.load(Ordering::Relaxed).max(1);
    #[allow(clippy::cast_precision_loss)]
    let r = passos as f64 / raios as f64;
    r
}

/// ⛔ **A CATRACA, medida em 2026-09-01.** Ela só ENCOLHE — ver o doc do módulo.
const TOLERADO: &[(&str, f64)] = &[
    ("[Bend]", 10.7),
    ("[Bend, Twist]", 26.9),
    ("[Bend, Twist, Taper]", 91.1),
];

/// ⭐⭐⭐ **A TABELA DO DOC, DERIVADA** — e é por isso que ela pode ser acreditada.
///
/// ```text
/// cargo test -p ph2d-field-render --test what_a_stack_of_deformers_costs_the_march \
///     -- --ignored --nocapture
/// ```
///
/// ⚠️ **É uma SONDA, não um gate** (`#[ignore]`, logo o CI nunca a corre — a cerca é a catraca
/// acima). Ela existe porque as colunas `divisor` e `‖∇f‖` do doc deste módulo já foram escritas
/// duas vezes de memória, e uma tabela que ninguém consegue reproduzir com um comando é uma tabela
/// que envelhece na primeira wave.
#[test]
#[ignore = "sonda: imprime a tabela do doc do módulo"]
fn print_the_cost_table() {
    let reg = Registry::default();
    println!("| pilha | raio | caixa | divisor | grad | folga | passos/raio |");
    for ms in [
        vec![],
        vec![UnaryKind::Bend],
        vec![UnaryKind::Twist],
        vec![UnaryKind::Taper],
        vec![UnaryKind::Bend, UnaryKind::Twist],
        vec![UnaryKind::Bend, UnaryKind::Twist, UnaryKind::Taper],
    ] {
        let nome = format!("{ms:?}");
        let doc = peca(ms);
        let bola = ph2d_field_eval::bounds::bounding_ball(&doc, &reg).expect("bordo");
        let h = bola.half();
        let divisor = f64::from(ph2d_field_eval::field_shrink(&doc, &reg));
        let grad = worst_gradient(&doc, 40);
        println!(
            "| `{nome}` | {:.3} | {:.3}×{:.3}×{:.3} | {divisor:.2} | {grad:.4} | {:.1}× | {:.1} |",
            bola.radius,
            h[0],
            h[1],
            h[2],
            1.0 / grad.max(1e-9),
            passos_por_raio(&doc)
        );
    }
}

/// `‖∇f‖` **dentro da caixa de recorte** — a mesma lei do irmão em `ph2d-field-eval`: fora dela
/// ninguém pergunta nada, e medir lá acusa código correcto.
fn worst_gradient(doc: &FieldDoc, steps: i32) -> f64 {
    let reg = Registry::default();
    let Some(bola) = ph2d_field_eval::bounds::bounding_ball(doc, &reg) else {
        return 0.0;
    };
    let (lo, hi_box) = ph2d_field_eval::bounds_clip::march_clip(bola);
    let f = Field::new(doc);
    let mut hi = 0.0f64;
    for i in 0..=steps {
        for j in 0..=steps {
            for k in 0..=steps {
                let p = |n: i32, e: usize| {
                    let t = f64::from(n) / f64::from(steps);
                    f64::from(lo[e]) + t * f64::from(hi_box[e] - lo[e])
                };
                let g = f.gradient_norm(p(i, 0), p(j, 1), p(k, 2), 1.0e-5);
                if g.is_finite() && g > 1e-6 {
                    hi = hi.max(g);
                }
            }
        }
    }
    hi
}

/// ⭐⭐⭐ **O CUSTO DE CADA PILHA, com as duas metades da catraca.**
///
/// ⛔⛔ **Prova de mutação (2026-08-31), e a catraca já desceu uma vez por ela:** devolver a
/// meia-altura da dobra (`bounds::canonical_step`) ao **raio** da bola em vez do eixo dobrado leva
/// `[Bend]` de `22,7` a `72,2` e o trio de `635,9` a `1 543,6` — a metade de cima do gate reprova.
/// E a metade de baixo já reprovou a sério: foi ela que obrigou esta tabela a ser re-escrita quando
/// a caixa por eixo entrou. *Uma catraca que só sabe subir não é uma catraca.*
#[test]
fn a_stack_of_deformers_never_costs_the_march_more_than_it_did() {
    let base = passos_por_raio(&peca(Vec::new()));
    // ⛔ **O CONTROLE**: uma peça sem deformador nenhum é o chão, e ele tem de ser pequeno — senão
    // a sonda está a medir outra coisa e todas as razões abaixo são sobre um número inventado.
    assert!(
        base < 15.0,
        "uma caixa sem modificadores custa {base:.1} passos/raio — a sonda não está a medir a marcha"
    );
    let mut piorou = Vec::new();
    let mut obsoletas = Vec::new();
    for (nome, medido) in TOLERADO {
        let ms: Vec<UnaryKind> = match *nome {
            "[Bend]" => vec![UnaryKind::Bend],
            "[Bend, Twist]" => vec![UnaryKind::Bend, UnaryKind::Twist],
            _ => vec![UnaryKind::Bend, UnaryKind::Twist, UnaryKind::Taper],
        };
        let agora = passos_por_raio(&peca(ms));
        // ⚠️ A folga de `5 %` é do arredondamento da tabela, e não uma licença: o número é
        // determinístico, então a única fonte de diferença é o que ficou escrito aqui.
        if agora > medido * 1.05 {
            piorou.push(format!("{nome}: {agora:.1} contra os {medido:.1} medidos"));
        }
        // ⛔ **A metade que faz a catraca DESCER.** *Uma catraca sem censo de obsolescência não
        // desce: ela vira licença.*
        if agora < medido * 0.9 {
            obsoletas.push(format!("{nome}: {agora:.1}, e a tabela diz {medido:.1}"));
        }
    }
    assert!(
        piorou.is_empty(),
        "a marcha ficou mais cara nestas pilhas: {}",
        piorou.join(" · ")
    );
    assert!(
        obsoletas.is_empty(),
        "a marcha ficou MAIS BARATA — re-escreva a tabela do `TOLERADO` e o doc do módulo: {}",
        obsoletas.join(" · ")
    );
}

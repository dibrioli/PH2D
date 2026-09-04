//! ⭐⭐⭐ **O QUE UMA PILHA DE DEFORMADORES CUSTA À MARCHA** — report do Enio, 2026-08-31: *«o render
//! fica bom. Algumas combinações muito lentas (`+ Taper` depois de `+ Bend` e `+ Twist`)»*.
//!
//! # ⭐ A régua é uma CONTAGEM, e não um relógio
//!
//! `passos ÷ raios` é determinístico: ele não se mexe com a carga da máquina, e por isso este gate
//! **não** é membro da família de flakes que o `CLAUDE.md` §5 lista. *Um gate de custo que lê o
//! relógio mede a máquina; um que conta passos mede o programa.*
//!
//! # O que a medição diz (caixa `0,35³`, `160²`, `half_extent 1,0`) — 2026-09-02
//!
//! | pilha | raio | caixa | divisor | `‖∇f‖` | folga | passos/raio |
//! |---|---:|---|---:|---:|---:|---:|
//! | `[]` | `0,579` | `0,350×0,350×0,300` | `1,00` | `1,000` | `1,0×` | `6,4` |
//! | `[Bend]` | `0,764` | `0,484×0,350×0,476` | `1,59` | `0,887` | `1,1×` | `10,7` |
//! | `[Twist]` | `0,579` | `0,495×0,495×0,300` | `1,82` | `0,776` | `1,3×` | `14,0` |
//! | `[Taper]` | `0,658` | `0,424×0,350×0,363` | `2,20` | `0,555` | `1,8×` | `16,6` |
//! | `[Bend, Twist]` | `0,764` | `0,597×0,597×0,476` | `3,58` | `0,551` | `1,8×` | `24,5` |
//! | `[Bend, Twist, Taper]` | `1,037` | `0,811×0,597×0,646` | **`8,88`** | `0,317` | **`3,2×`** | **`54,9`** |
//!
//! ⭐⭐⭐ **O trio caiu de `91,1` para `54,9` em 2026-09-02** — o divisor deixou de ser o PRODUTO dos
//! `σ` e passou a ser o `σ_max` do produto das MATRIZES (`ph2d_field_eval::bounds_lip`), e a folga
//! provada caiu de `5,6×` para `3,2×`. Mecanismo, a refutação da sub-divisão uniforme e o preço:
//! `docs/3DModeling/09_o_bound_da_composicao.md`.
//!
//! ⛔⛔ **O `[Bend, Twist]` já SUBIU uma vez, de `23,4` para `26,9`, e isso foi uma CORRECÇÃO a
//! mostrar o preço** (report dos três cilindros cruzados, 2026-09-01): o atalho do `Ball::merge`
//! deitava fora a caixa da bola perdedora, e o **envelope de uma pilha** é uma sequência de `merge`.
//! Com a torção a preservar o raio da dobra, o atalho disparava e o envelope ficava com a caixa de
//! um passo só — logo a parede da dobra media menos do que a região que a marcha percorre. *A
//! catraca apanhou-o, que é o que uma catraca de duas metades existe para fazer.* ⚠️ Hoje ele está
//! em `24,5`, que o bound da composição devolveu.
//!
//! ⚠️ **A tabela é DERIVADA** — corra a sonda `print_the_cost_table` (`--ignored --nocapture`) antes
//! de a citar. Ela já foi escrita de memória duas vezes.
//!
//! # ⭐⭐⭐ As três jornadas, e o que cada uma tirou
//!
//! | pilha | 30/08 (o 1.º report) | 31/08 (a bola aprende os EIXOS) | **02/09 (o bound da COMPOSIÇÃO)** |
//! |---|---:|---:|---:|
//! | `[Bend]` | `72,2` | `22,7` | **`10,7`** |
//! | `[Bend, Twist]` | `233,1` | `55,6` | **`24,5`** |
//! | `[Bend, Twist, Taper]` | `1 543,6` | `635,9` | **`54,9`** |
//! | divisor do trio | `240,29` | `100,87` | **`8,88`** |
//! | folga PROVADA do trio | `24,7×` | `22,3×` | **`3,2×`** |
//!
//! ⇒ **`28,1×`** mais barato do que o que o Enio reportou como *«muito lentas»*, e `11,6×` do que
//! ficou ontem. O trio custa hoje `8,6×` uma caixa nua, e custava `217×`.
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
//! # ⭐⭐⭐ ONDE O QUADRO SE GASTA — e a resposta é UMA coluna (2026-09-01)
//!
//! Report do Enio depois da wave anterior: *«Box → + Bend → + Twist → + Taper fica extremamente
//! lento»*. A sonda `where_the_frame_goes_in_a_deformer_stack` (`320²`, `--release`):
//!
//! | pilha | quadro | amostras | passos/raio | ns/amostra |
//! |---|---:|---:|---:|---:|
//! | `[]` | `7,6 ms` | `172 739` | `6,5` | `43,8` |
//! | `[Bend]` | `20,9` | `423 081` | `10,4` | `49,3` |
//! | `[Bend, Twist]` | `85,8` | `1 593 456` | `24,7` | `53,8` |
//! | `[Bend, Twist, Taper]` | **`349,7`** | **`7 371 171`** | `84,9` | `47,4` |
//!
//! ⭐⭐⭐ **O `ns/amostra` é PLANO** — `43,8` para `47,4`, apenas `1,08×`. Uma pilha de três
//! deformadores traz `atan2`, `sqrt`, `sin`, `cos` e dois clamps suaves à fita, e **isso não custa
//! nada**: o quadro é `43×` mais caro porque tem `43×` mais AMOSTRAS. ⇒ *a cura é o número de
//! passos, e mais nenhuma* — a outra metade da pergunta que o doc da `measure_the_shape_of_the_march`
//! nomeia (*«caro por amostra ou caro em passos?»*) está respondida.
//!
//! ⚠️ **E não há contagem dupla:** o `safe_march_step` devolve `1,0000` em todas estas pilhas (os
//! deformadores entram pelo `field_shrink`, não pelo `gradient_bound`), então o divisor é pago uma
//! vez só.
//!
//! # ⛔⛔⛔ RECUSA MEDIDA (2026-09-01): a SOBRE-RELAXAÇÃO come a margem que não está provada
//!
//! A *Enhanced Sphere Tracing* (Keinert et al. 2014) é a saída publicada para um raio caro em
//! passos, e o doc da sonda irmã aponta-a pelo nome. Implementada com o teste de sobreposição
//! (`r + r_ant ≥ passo`, que é o que a torna demonstravelmente segura) e com o recuo a repor o passo
//! que a marcha de sempre daria:
//!
//! | `ω` | `[]` | `[Bend]` | `[Bend, Twist]` | trio | imagem contra o oráculo |
//! |---:|---:|---:|---:|---:|---|
//! | `1,0` | `6,4` | `10,7` | `26,9` | `91,1` | ✅ `6/6` |
//! | `1,6` | `7,3` | `10,5` | `20,5` | **`57,0`** | ⛔ `14` de `1 202` pixels VAZIOS |
//! | `2,5` | `6,4` | `9,2` | `22,4` | `74,1` | ⛔ |
//! | `4,0` | `7,7` | `10,6` | `27,9` | `92,1` | ⛔ |
//!
//! ⇒ o melhor caso compra **`1,6×`** e **perde peça**. ⭐ E o mecanismo é a própria razão de a folga
//! existir: o bound da composição é optimista na direcção do recuo (um deformador EXTERIOR expande a
//! região que o interior vê, e o divisor mede-a na caixa de recorte, não na imagem inversa dela).
//! *A marcha de sempre sobrevive nessa folga; a sobre-relaxação é literalmente o algoritmo desenhado
//! para a gastar.* ⛔ E acima de `ω = 2,5` ela nem sequer compra — os recuos comem o que a relaxação
//! ganha.
//!
//! ⚠️ **Duas armadilhas de implementação, as duas medidas antes de qualquer conclusão** (e a 1.ª
//! versão da tabela acima foi tirada com elas dentro, o que teria escrito uma recusa sobre um
//! programa partido): o estado por raio **nasce onde o raio entra no recorte**, e não em zero — com
//! `t_ant = 0` o primeiro teste de todo raio reprova e manda-o para trás da câmera (`2,0` passos por
//! raio, peça a desaparecer) —, e o recuo tem de **reiniciar** esse estado, senão o raio volta ao
//! mesmo ponto em ciclo.
//!
//! ⇒ para a sobre-relaxação valer aqui, o bound da composição tem de ser **demonstrado** primeiro.
//! É a mesma obra que a secção abaixo nomeia, e ela paga as duas de uma vez.
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
        UnaryKind::Bend => Unary::Bend {
            turns: 0.12,
            lower: -2.0,
            upper: 2.0,
            falloff: 0.1,
            axis: BEND_AXIS,
        },
        // ⛔⛔ **EXAUSTIVO de propósito, e ele era um `_ => Bend`** (2026-09-01). Enquanto só três
        // naturezas chegavam aqui ninguém notava; a sonda da composição passou-lhe o
        // `UnaryKind::ALL` e a tabela saiu com `[Shell, Shell, Shell]` a medir três dobras.
        // *Um catch-all numa fixtura não devolve um erro — devolve a peça errada com o nome certo.*
        UnaryKind::Shell => Unary::Shell { thickness: 0.06 },
        UnaryKind::Offset => Unary::Offset { distance: 0.05 },
        // ⚠️ **Este fica no plano `0` de propósito, ao contrário dos irmãos de correcção**: aqui
        // mede-se um RELÓGIO, e um plano vivo dobra a peça — o número passaria a descrever outra
        // cena e as barras desta bancada foram calibradas nesta. *A cegueira ao knob é real e é
        // paga noutro sítio* (`the_box_of_a_bound_contains_the_piece`).
        UnaryKind::Mirror => Unary::Mirror { offset: 0.0 },
        UnaryKind::MirrorY => Unary::MirrorY { offset: 0.0 },
        UnaryKind::MirrorZ => Unary::MirrorZ { offset: 0.0 },
        UnaryKind::Array => Unary::Array {
            count: 3,
            spacing: 0.5,
            joint: ph2d_field::Joint {
                chamfer: 0.0,
                fillet: 0.06,
            },
            axis: ph2d_field::mods::ARRAY_AXIS,
        },
        UnaryKind::Radial => Unary::Radial {
            count: 6,
            joint: ph2d_field::Joint {
                chamfer: 0.0,
                fillet: 0.06,
            },
            axis: ph2d_field::mods::RADIAL_AXIS,
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
    ("[Bend, Twist]", 24.5),
    ("[Bend, Twist, Taper]", 54.9),
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
    println!(
        "| pilha | raio | caixa | divisor | passo | div x 1/passo | grad | folga | passos/raio |"
    );
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
        let passo = f64::from(ph2d_field_eval::safe_march_step(&doc));
        println!(
            "| `{nome}` | {:.3} | {:.3}×{:.3}×{:.3} | {divisor:.2} | {passo:.4} | {:.2} | {grad:.4} | {:.1}× | {:.1} |",
            bola.radius,
            h[0],
            h[1],
            h[2],
            divisor / passo,
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

/// ⭐⭐⭐ **ONDE O QUADRO SE GASTA numa pilha de deformadores** — a metade do diagnóstico que faltava
/// (report do Enio, 2026-09-01: *«Box + Bend + Twist + Taper fica extremamente lento»*).
///
/// O doc da sonda irmã (`measure_the_shape_of_the_march`) diz que *«um raio que dá 8 passos e um que
/// dá 40 pedem curas OPOSTAS: o primeiro é caro por AMOSTRA, o segundo em PASSOS»*. A catraca acima
/// mede só os passos — e uma pilha de três deformadores **também alonga a fita** (a dobra sozinha
/// traz `atan2`, `sqrt`, `sin`, `cos` e um clamp suave). ⇒ sem esta coluna não se sabe qual das duas
/// curas o produto pede.
///
/// ⚠️ **Ela lê o RELÓGIO**, logo só vale com a máquina calma (`CLAUDE.md` §5) — por isso é sonda e
/// não gate, e por isso as duas medições correm no **mesmo processo**.
///
/// ```text
/// cargo test -p ph2d-field-render --release --test what_a_stack_of_deformers_costs_the_march \
///     -- --ignored --nocapture where_the_frame_goes
/// ```
#[test]
#[ignore = "sonda: lê o relógio"]
fn where_the_frame_goes_in_a_deformer_stack() {
    use std::time::Instant;
    let reg = Registry::default();
    let cam = Orbit {
        half_extent: 1.0,
        ..Orbit::default()
    };
    let (w, h) = (320u32, 320u32);
    println!("| pilha | quadro ms | amostras | passos/raio | ns/amostra | vs caixa |");
    let mut base_ns = 0.0f64;
    for ms in [
        vec![],
        vec![UnaryKind::Bend],
        vec![UnaryKind::Bend, UnaryKind::Twist],
        vec![UnaryKind::Bend, UnaryKind::Twist, UnaryKind::Taper],
    ] {
        let nome = format!("{ms:?}");
        let doc = peca(ms);
        // ⚠️ Uma corrida a frio paga a montagem da fita; o que se mede é a segunda.
        let _ = ph2d_field_render::trace_with(&doc, &reg, &cam, w, h, false, true);
        STEP_SAMPLES.store(0, Ordering::Relaxed);
        MARCH_RAYS.store(0, Ordering::Relaxed);
        let t = Instant::now();
        let _ = ph2d_field_render::trace_with(&doc, &reg, &cam, w, h, false, true);
        let elapsed = t.elapsed().as_secs_f64() * 1000.0;
        #[allow(clippy::cast_precision_loss)]
        let amostras = STEP_SAMPLES.load(Ordering::Relaxed) as f64;
        #[allow(clippy::cast_precision_loss)]
        let raios = MARCH_RAYS.load(Ordering::Relaxed).max(1) as f64;
        let ns = elapsed * 1.0e6 / amostras.max(1.0);
        if base_ns == 0.0 {
            base_ns = ns;
        }
        println!(
            "| `{nome}` | {elapsed:.1} | {amostras:.0} | {:.1} | {ns:.1} | {:.2}x |",
            amostras / raios,
            ns / base_ns
        );
    }
}

/// ⭐⭐⭐ **O BOUND COMPOSTO É O PRODUTO OU O MÁXIMO?** — o spike que escolhe a forma da cura
/// (Enio, 2026-09-01: *«a matemática, e leva o tempo que levar»*).
///
/// A folga do trio é `5,6×`, e ela mora na COMPOSIÇÃO. A pista: `2,61 × 2,66 × 2,24 = 15,85`
/// cobrados contra `2,81` verdadeiros — e `max(2,61, 2,66, 2,24) = 2,66`. ⇒ *se a verdade for o
/// máximo e não o produto, as três esticadelas são quase ortogonais entre si, e o bound certo é o
/// `σ_max` do PRODUTO das matrizes, não o produto dos `σ_max`.*
///
/// ⚠️ **A verdade aqui é `divisor × ‖∇f‖ medido`, e ela é um MINORANTE da verdade** (um máximo
/// amostrado não é um máximo). ⇒ este spike pode confirmar a hipótese, nunca refutá-la sozinho: se
/// `verdade > max`, a hipótese cai; se `verdade ≈ max`, ela sobrevive e vale construir.
#[test]
#[ignore = "spike: escolhe a forma do bound"]
fn is_the_composed_bound_the_product_or_the_max() {
    use ph2d_field::UnaryKind;
    let reg = Registry::default();
    let (mut pior_razao, mut nome_pior) = (0.0f64, String::new());
    let mut linhas = 0usize;
    println!("| pilha | Π σ (cobrado) | max σ | verdade | verdade/max | Π/verdade |");
    for a in UnaryKind::ALL {
        for b in UnaryKind::ALL {
            for c in UnaryKind::ALL {
                let ms = vec![a, b, c];
                let doc = peca(ms.clone());
                let Some(bola) = ph2d_field_eval::bounds::local_balls(&doc, &reg)
                    .first()
                    .copied()
                    .flatten()
                else {
                    continue;
                };
                let vivos: Vec<_> = ms.iter().map(|k| vivo(*k)).collect();
                let fs = ph2d_field_eval::stack_divisor_factors(&vivos, bola);
                let prod: f64 = fs.iter().product();
                let maxi = fs.iter().copied().fold(1.0f64, f64::max);
                if prod < 1.05 {
                    continue;
                }
                let g = worst_gradient(&doc, 32);
                let verdade = prod * g;
                linhas += 1;
                let razao = verdade / maxi;
                if razao > pior_razao {
                    pior_razao = razao;
                    nome_pior = format!("{ms:?}");
                }
                if linhas <= 14 || razao > 1.05 {
                    println!(
                        "| `{ms:?}` | {prod:.2} | {maxi:.2} | {verdade:.2} | {razao:.2} | {:.1}× |",
                        prod / verdade.max(1e-9)
                    );
                }
            }
        }
    }
    println!(
        "\n⇒ {linhas} pilhas. PIOR verdade/max = {pior_razao:.3} em {nome_pior} \
         (≤ 1 ⇒ a verdade é o MÁXIMO; ≫ 1 ⇒ é preciso mais do que o máximo)"
    );
}

/// ⛔⛔⛔ **UMA JUNTA VIVA NUMA REPETIÇÃO DEPOIS DE UM DEFORMADOR** — o que a sonda da composição
/// tropeçou (2026-09-01), e o gate dos trios não vê porque a fixtura dele é `Joint::SHARP`.
#[test]
#[ignore = "sonda"]
fn does_a_live_joint_on_a_repetition_tear_the_field() {
    use ph2d_field::{Joint, UnaryKind};
    let reg = Registry::default();
    println!("| pilha | junta | divisor | ‖∇f‖ | passos/raio |");
    for (nome, ms) in [
        (
            "[Bend, Radial, Radial]",
            vec![UnaryKind::Bend, UnaryKind::Radial, UnaryKind::Radial],
        ),
        (
            "[Bend, Array, Array]",
            vec![UnaryKind::Bend, UnaryKind::Array, UnaryKind::Array],
        ),
        (
            "[Bend, Twist, Radial]",
            vec![UnaryKind::Bend, UnaryKind::Twist, UnaryKind::Radial],
        ),
        ("[Radial]", vec![UnaryKind::Radial]),
        ("[Bend, Radial]", vec![UnaryKind::Bend, UnaryKind::Radial]),
        (
            "[Radial, Radial]",
            vec![UnaryKind::Radial, UnaryKind::Radial],
        ),
    ] {
        for (rot, junta) in [
            (
                "viva",
                Joint {
                    chamfer: 0.0,
                    fillet: 0.06,
                },
            ),
            ("SHARP", Joint::SHARP),
        ] {
            let vivos: Vec<Unary> = ms
                .iter()
                .map(|k| match vivo(*k) {
                    Unary::Array {
                        count,
                        spacing,
                        axis,
                        ..
                    } => Unary::Array {
                        count,
                        spacing,
                        joint: junta,
                        axis,
                    },
                    Unary::Radial { count, axis, .. } => Unary::Radial {
                        count,
                        joint: junta,
                        axis,
                    },
                    outro => outro,
                })
                .collect();
            let mut n = ph2d_field::Node::new(
                Xform::IDENTITY,
                NodeKind::Leaf(Primitive::Box {
                    half: [0.35, 0.35, 0.30],
                    round: 0.0,
                    chamfer: 0.0,
                }),
            );
            n.mods = vivos;
            let doc = FieldDoc::new(vec![n], NodeId(0)).expect("peça");
            println!(
                "| `{nome}` | {rot} | {:.2} | {:.4} | {:.1} |",
                f64::from(ph2d_field_eval::field_shrink(&doc, &reg)),
                worst_gradient(&doc, 40),
                passos_por_raio(&doc),
            );
        }
    }
}

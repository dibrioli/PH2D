//! ⭐⭐⭐ **O QUE UMA PILHA DE DEFORMADORES CUSTA À MARCHA** — report do Enio, 2026-08-31: *«o render
//! fica bom. Algumas combinações muito lentas (`+ Taper` depois de `+ Bend` e `+ Twist`)»*.
//!
//! # ⭐ A régua é uma CONTAGEM, e não um relógio
//!
//! `passos ÷ raios` é determinístico: ele não se mexe com a carga da máquina, e por isso este gate
//! **não** é membro da família de flakes que o `CLAUDE.md` §5 lista. *Um gate de custo que lê o
//! relógio mede a máquina; um que conta passos mede o programa.*
//!
//! # O que a medição diz (caixa `0,35³`, `160²`, `half_extent 1,0`)
//!
//! | pilha | raio da bola | divisor | `‖∇f‖` | **folga** | passos/raio |
//! |---|---:|---:|---:|---:|---:|
//! | `[]` | `0,579` | `1,00` | `1,000` | `1,0×` | **`7,1`** |
//! | `[Bend]` | `0,957` | `3,59` | `0,552` | `1,8×` | `22,7` |
//! | `[Twist]` | `0,579` | `1,82` | `0,796` | `1,3×` | `13,6` |
//! | `[Taper]` | `0,780` | `2,20` | `0,784` | `1,3×` | `14,5` |
//! | `[Bend, Twist]` | `0,957` | `9,00` | `0,351` | `2,8×` | `55,6` |
//! | `[Bend, Twist, Taper]` | `1,507` | `100,87` | `0,045` | **`22,3×`** | **`635,9`** |
//!
//! ⭐⭐⭐ **A coluna do RAIO é a que mudou tudo (2026-08-31).** O bordo passou a levar as
//! **meias-extensões por eixo** ao lado do raio (`bounds::Ball::half`), e a dobra deixou de medir a
//! altura do arco pelo raio da esfera: `[Bend]` de `72,2` para **`22,7`** passos/raio (`3,2×`),
//! `[Bend, Twist]` de `233,1` para **`55,6`** (`4,2×`), o trio de `1 543,6` para **`635,9`**
//! (`2,4×`). ⚠️ **E as cinco imagens contra a marcha honesta ficaram idênticas** — não é um
//! afrouxamento, é o bordo a deixar de mentir sobre o tamanho da peça.
//!
//! ⇒ **`90×` o custo de uma caixa** (era `217×`), e **`22,3×` disso continua a ser desperdício
//! PROVADO**: o campo podia ser `22,3×` maior e continuar a ser um minorante válido.
//!
//! # ⛔⛔ A causa: os divisores MULTIPLICAM-SE, e os piores casos estão em sítios DIFERENTES
//!
//! Cada `step_divisor` é o pior caso do seu operador **sobre a caixa de recorte inteira**. O da
//! dobra é pior junto à parede congelada, o da torção no maior raio, o da inclinação no ápice — três
//! sítios distintos. Multiplicá-los assume que coincidem.
//!
//! ⚠️ **Três hipóteses foram MEDIDAS e não são a cura:**
//! - **ler a bola CORRENTE em vez do envelope final**: o `‖∇f‖` sobe de `0,041` para `0,070` e o
//!   custo cai só `1,6×` — e ⛔ **dessincroniza os dois leitores da lei** (a [`ph2d_field_eval::stack`]
//!   e o `field_shrink`), que o doc de lá chama de uma porta só;
//! - **baixar o `BEND_FOLD_MARGIN`**: funciona e é uma troca de PRODUTO, não uma cura — ver a tabela
//!   no doc daquela constante;
//! - **culpar um dos três**: cada factor é honesto sozinho (`[Bend]` = `2,5×` de folga, `[Twist]` e
//!   `[Taper]` = `1,3×`). *Nenhum deles está errado; o que está errado é somá-los como se fossem.*
//!
//! # ⛔⛔⛔ E A CURA ÓBVIA — O DIVISOR POR REGIÃO — ESTÁ REFUTADA POR MEDIÇÃO
//!
//! A ideia era natural: a marcha já especializa a árvore por **ladrilho × fatia de profundidade**,
//! e num ladrilho longe do centro do arco o divisor da dobra devia ser ~`1`. ⇒ medi o `‖∇f‖` do
//! envelope inteiro contra o de cada **oitavo** dele:
//!
//! | região | `‖∇f‖` |
//! |---|---:|
//! | o envelope inteiro | `0,0405` |
//! | o **pior** oitavo | `0,0416` |
//! | o melhor oitavo | `0,0173` |
//!
//! ⇒ **o pior oitavo mede o mesmo que a caixa toda.** *O desperdício não é espacial:* o campo é
//! `24×` pequeno demais **em todo o lado**, não num canto. Cortar o domínio não compra nada.
//!
//! ⚠️ **E havia um segundo bloqueador que a mesma sonda encontrou:** a
//! `ph2d_field_eval::RegionCompiler::is_worth_it` devolve `false` sem uma forma de **perfil**, então
//! numa caixa com deformadores a especialização por ladrilho está **desligada por inteiro**. Quem
//! for por este caminho tem de a ligar primeiro — e agora sabe que não vale a pena.
//!
//! # ⭐ Onde a folga de facto está: o BOUND é frouxo, e a frouxidão MULTIPLICA
//!
//! | pilha | cobrado | de facto preciso | folga |
//! |---|---:|---:|---:|
//! | `[Bend]` | `3,59` | `2,0` | `1,8×` |
//! | `[Bend, Twist]` | `9,00` | `3,2` | `2,8×` |
//! | `[Bend, Twist, Taper]` | `100,87` | `4,5` | **`22,3×`** |
//!
//! Cada factor é frouxo por pouco (`1,3×`–`1,8×`); três frouxos multiplicam-se em `22,3×`. ⭐ E a
//! folga **por factor** já não é a dobra: ela caiu de `2,5×` para `1,8×` quando o bordo aprendeu os
//! eixos. O que sobra é a **composição**, e apertá-la é demonstrar um bound melhor para o produto
//! dos três mapas — trabalho de matemática, não um botão.

use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Primitive, Unary, UnaryKind, Xform};
use ph2d_field_eval::{hybrid::Registry, safe_march_step};
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

/// ⛔ **A CATRACA, medida em 2026-08-31.** Ela só ENCOLHE — ver o doc do módulo para as três
/// hipóteses já medidas e para a cura de fundo.
const TOLERADO: &[(&str, f64)] = &[
    ("[Bend]", 22.7),
    ("[Bend, Twist]", 55.6),
    ("[Bend, Twist, Taper]", 635.9),
];

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

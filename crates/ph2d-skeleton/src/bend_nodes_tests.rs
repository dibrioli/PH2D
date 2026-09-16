//! Os gates da **ESCOLHA DOS NÓS** e das **alças derivadas da corrente** — irmão do [`super`] pelo
//! tecto de LOC do HR-18, com o corte por RESPONSABILIDADE.
//!
//! O pai mede *que forma um osso curvo tem* (o neutro, a fenda, a ponta, a partição do peso); aqui
//! mede-se **como os nós do eixo são escolhidos** (a equalização que tirou a variação do esticão) e
//! **de onde vêm as duas alças** (autoradas ou derivadas dos vizinhos).
//!
//! ⚠️ Filho do arnês do pai (`#[path]`) para herdar as fixturas dele — uma cópia divergiria no
//! primeiro ajuste.

use super::*;

/// O comprimento de arco da curva entre `0` e `t`, integrado FINO — o oráculo da equalização.
///
/// ⚠️ **Ele não partilha a tabela do produto**: `4 096` passos contra as `n × 16` amostras da lei.
/// *Uma régua construída com a mesma discretização da lei é um espelho dela.*
fn arco_ate(len: f64, curva: Bend, t: f64) -> f64 {
    const PASSOS: usize = 4_096;
    let mut soma = 0.0;
    let mut anterior = point_at(len, curva, 0.0);
    for i in 1..=PASSOS {
        #[expect(clippy::cast_precision_loss, reason = "i <= 4096")]
        let u = t * (i as f64) / (PASSOS as f64);
        let p = point_at(len, curva, u);
        soma += (p[0] - anterior[0]).hypot(p[1] - anterior[1]);
        anterior = p;
    }
    soma
}

/// ⭐⭐⭐⭐ **O ESTICÃO NÃO VARIA AO LONGO DO OSSO** — o limite declarado em 2026-09-15, fechado.
///
/// > *«o esticão VARIA ao longo do osso (`13 %` com as alças a `0,2 L`) porque os nós saem do
/// > PARÂMETRO e não do ARCO»* — fila do módulo, F8.
///
/// ⚠️ **A régua é a DISPERSÃO do esticão axial** (`max/min − 1` sobre os `n` sub-ossos), e o
/// controlo é a lei ANTIGA corrida pela mesma porta ([`frames_com`] com `k/n`). *Sem o controlo,
/// este gate mediria a fixtura.*
///
/// Medido (`L = 10`, `8` sub-ossos):
///
/// | alças | dispersão ANTES | DEPOIS |
/// |---|---:|---:|
/// | `0,2 L` | `12,63 %` | **`0,000 %`** |
/// | `0,6 L` | `82,01 %` | **`0,000 %`** |
/// | eixo cruzado | `1 051,95 %` | **`0,000 %`** |
///
/// ⛔ **E o esticão não desaparece — nem devia:** um osso que arqueia percorre mais caminho, e o
/// valor comum a que ele converge é `arco / L`. O gate afirma as duas coisas.
#[test]
fn o_esticao_nao_varia_ao_longo_do_osso() {
    let (len, n) = (10.0_f64, 8u8);
    // O esticão axial de cada sub-osso é a norma da coluna do eixo do frame.
    let esticoes = |taus: &[f64], curva: Bend| -> Vec<f64> {
        crate::bend::frames_com(
            crate::bend::BoneSpec {
                length: len,
                strength: 1.0,
                segments: n,
                curve: curva,
            },
            taus,
        )
        .into_iter()
        .map(|f| f.0[0].hypot(f.0[1]))
        .collect()
    };
    let dispersao = |v: &[f64]| -> f64 {
        let (mut lo, mut hi) = (f64::INFINITY, 0.0_f64);
        for &x in v {
            lo = lo.min(x);
            hi = hi.max(x);
        }
        hi / lo - 1.0
    };
    let uniformes: Vec<f64> = (0..=n).map(|k| f64::from(k) / f64::from(n)).collect();

    // ⚠️ A terceira é o caso DURO (as alças no eixo e cruzadas, a curva quase pára a meio) — é ela
    // que separa a equalização por ARCO da correcção para a CORDA.
    for (nome, curva) in [
        (
            "0,2 L",
            Bend {
                inn: [0.0, 0.2],
                out: [0.0, 0.2],
            },
        ),
        (
            "0,6 L",
            Bend {
                inn: [0.0, 0.6],
                out: [0.0, 0.6],
            },
        ),
        (
            "eixo cruzado",
            Bend {
                inn: [0.9, 0.0],
                out: [-0.9, 0.0],
            },
        ),
    ] {
        let antes = dispersao(&esticoes(&uniformes, curva));
        let taus = crate::bend::nodes(len, n, curva);
        let depois_v = esticoes(&taus, curva);
        let depois = dispersao(&depois_v);
        println!("{nome:>14} | dispersao antes {antes:.4} | depois {depois:.6}");

        // ⛔ O CONTROLO: a lei antiga tem de reproduzir o defeito, senão a fixtura não o alcança.
        assert!(
            antes > 0.1,
            "a fixtura {nome} nao reproduz o defeito (dispersao {antes:.4})"
        );
        // ⭐ E a lei nova tem de o apagar. A barra é `1 %`, e o vale medido está duas ordens de
        // grandeza abaixo dela — ver a tabela do `AMOSTRAS_POR_SUB_OSSO`.
        assert!(
            depois < 0.01,
            "a equalizacao deixou {depois:.6} de dispersao em {nome}"
        );
        // ⭐⭐ **E o valor COMUM é o COMPRIMENTO DA POLILINHA sobre `L`** — o esticão é real, ele só
        // deixou de variar.
        //
        // ⚠️⚠️ **Não é `arco / L`, e a 1.ª redacção deste gate escreveu isso e reprovou por
        // `1,09e-3`:** cada sub-osso estica pela CORDA dele, e a corda de um pedaço curvo é mais
        // curta que o arco daquele pedaço. *A diferença é a discretização, não um erro* — e é
        // exactamente por isso que o número certo é a soma das cordas, que é o que a
        // [`polyline`] desenha e o [`arc_length`] mede.
        let esperado = arc_length(&polyline(BoneSpec {
            length: len,
            strength: 1.0,
            segments: n,
            curve: curva,
        })) / len;
        for (k, &e) in depois_v.iter().enumerate() {
            assert!(
                (e - esperado).abs() < 1e-6,
                "sub-osso {k}: esticao {e:.6} contra o da polilinha {esperado:.6}"
            );
        }
        // ⚠️ A polilinha nunca é mais longa que o arco — uma corda é o caminho mais curto.
        //
        // ⚠️⚠️ **E o DÉFICE não tem barra, de propósito:** a 1.ª redacção exigia `< 1 %` e reprovou
        // no caso do eixo cruzado, com razão — ali a curva **volta para trás** dentro de um
        // sub-osso, e a corda mede `1,000` contra `1,137` de arco. *Um limite de discretização
        // deixa de o ser quando a curva dobra sobre si mesma dentro de uma peça*, e a cura disso é
        // mais segmentos, não uma barra.
        let arco = arco_ate(len, curva, 1.0) / len;
        assert!(
            esperado <= arco + 1e-12,
            "a polilinha ({esperado:.6}) saiu mais longa que o arco ({arco:.6})"
        );
    }
}

/// ⭐⭐⭐ **OS NÓS REPARTEM A CORDA POR IGUAL** — a afirmação da lei, medida **directamente dos
/// pontos da curva** e não dos frames.
///
/// ⚠️⚠️ **A 1.ª redacção afirmava o ARCO e reprovou — com razão, e é esse o achado da wave:**
/// equalizar o arco NÃO iguala o esticão, porque a corda de um pedaço mais curvo é mais curta que
/// o arco dele. *A grandeza que o artista vê é a corda, e é ela que a lei iguala.*
///
/// ⚠️ **A régua não passa pelos frames**: ela mede `|P(t_{k+1}) − P(t_k)|` do [`point_at`]. O
/// gate irmão mede a mesma propriedade pelo lado dos frames, e os dois caminhos têm de concordar.
///
/// (Mutação: o `nodes` devolver `k/n`, ou saltar as rondas de correcção ⇒ RED.)
#[test]
fn os_nos_repartem_a_corda_por_igual() {
    let (len, n) = (10.0_f64, 8u8);
    for curva in [
        Bend {
            inn: [0.3, 0.6],
            out: [-0.2, 0.5],
        },
        Bend {
            inn: [0.9, 0.0],
            out: [-0.9, 0.0],
        },
    ] {
        let taus = crate::bend::nodes(len, n, curva);
        let pts: Vec<[f64; 2]> = taus.iter().map(|&t| point_at(len, curva, t)).collect();
        let cordas: Vec<f64> = pts
            .windows(2)
            .map(|w| (w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1]))
            .collect();
        let (mut lo, mut hi) = (f64::INFINITY, 0.0_f64);
        for &c in &cordas {
            lo = lo.min(c);
            hi = hi.max(c);
        }
        println!(
            "{curva:?}: cordas de {lo:.9} a {hi:.9} ({:.2e} de dispersao)",
            hi / lo - 1.0
        );
        assert!(
            hi / lo - 1.0 < 1e-6,
            "as cordas saíram de {lo:.6} a {hi:.6} — a lei nao as igualou"
        );
        // ⛔ **As duas pontas são EXACTAS**, e não «dentro da tolerância»: a Bézier começa na raiz
        // e acaba na ponta por definição, e um `1.0` obtido de uma divisão poria a ponta do osso
        // num sítio que o resto do app não lê como ponta.
        assert_eq!(taus[0], 0.0);
        assert_eq!(taus[usize::from(n)], 1.0);
        // ⚠️ E os nós ficam ORDENADOS: dois que se cruzem invertem um sub-osso.
        assert!(
            taus.windows(2).all(|w| w[0] <= w[1]),
            "os nos sairam fora de ordem: {taus:?}"
        );
    }
}

/// ⛔⛔ **O PONTO NEUTRO CONTINUA EXACTO DEPOIS DA EQUALIZAÇÃO** — a objecção que o limite
/// declarado registava (*«um somatório de cordas não devolve `L` ao bit»*) é verdadeira e **não
/// morde**, porque no neutro o somatório nunca corre.
///
/// (Mutação: apagar o `if bend.is_straight()` do [`nodes`] ⇒ RED.)
#[test]
fn a_equalizacao_nao_toca_no_ponto_neutro() {
    for n in [1u8, 2, 8, 32] {
        let taus = crate::bend::nodes(7.5, n, Bend::STRAIGHT);
        for k in 0..=n {
            assert_eq!(
                taus[usize::from(k)],
                f64::from(k) / f64::from(n),
                "no' {k} de {n} deixou de ser exacto"
            );
        }
        for k in 0..n {
            assert_eq!(
                frame(7.5, n, Bend::STRAIGHT, k),
                crate::Xform::IDENTITY,
                "o frame {k} de {n} deixou de ser a identidade AO BIT"
            );
        }
    }
}

/// ⏱️ **SONDA (`--ignored`) — a varredura que fixou o `AMOSTRAS_POR_SUB_OSSO` e o
/// `RONDAS_DA_CORDA`.**
///
/// ```text
/// cargo test -p ph2d-skeleton --lib -- --ignored --nocapture bend_measure_the_table
/// ```
#[test]
#[ignore = "sonda: imprime a tabela da densidade, sem barra"]
fn bend_measure_the_table() {
    let (len, n) = (10.0_f64, 8u8);
    let disp = |taus: &[f64], curva: Bend| -> f64 {
        let v: Vec<f64> = crate::bend::frames_com(
            BoneSpec {
                length: len,
                strength: 1.0,
                segments: n,
                curve: curva,
            },
            taus,
        )
        .into_iter()
        .map(|f| f.0[0].hypot(f.0[1]))
        .collect();
        let (mut lo, mut hi) = (f64::INFINITY, 0.0_f64);
        for &x in &v {
            lo = lo.min(x);
            hi = hi.max(x);
        }
        (hi / lo - 1.0) * 100.0
    };
    // ⚠️ A terceira é o caso DURO: as alças no eixo e cruzadas fazem a curva quase PARAR a meio,
    // que é onde o mapa `arco -> parametro` fica mais torto. *Uma varredura só sobre os casos
    // brandos escolhe uma tabela que o caso duro não aguenta.*
    let curvas = [
        Bend {
            inn: [0.0, 0.2],
            out: [0.0, 0.2],
        },
        Bend {
            inn: [0.0, 0.6],
            out: [0.0, 0.6],
        },
        Bend {
            inn: [0.9, 0.0],
            out: [-0.9, 0.0],
        },
    ];

    println!("dispersao do esticao, em %, com {n} sub-ossos\n");
    let uniformes: Vec<f64> = (0..=n).map(|k| f64::from(k) / f64::from(n)).collect();
    println!(
        "  (sem equalizar)  | {:>9.4} % | {:>9.4} % | {:>9.4} %",
        disp(&uniformes, curvas[0]),
        disp(&uniformes, curvas[1]),
        disp(&uniformes, curvas[2])
    );
    println!("\namostras x rondas | a 0,2 L | a 0,6 L | eixo cruzado");
    for amostras in [1usize, 2, 4, 8, 16, 32] {
        for rondas in [0usize, 1, 2, 4, 8] {
            let col: Vec<f64> = curvas
                .iter()
                .map(|&c| {
                    disp(
                        &crate::bend::nodes_com_rondas(len, n, c, amostras, rondas),
                        c,
                    )
                })
                .collect();
            println!(
                "{amostras:>8} x {rondas:<7} | {:>9.4} % | {:>9.4} % | {:>9.4} %",
                col[0], col[1], col[2]
            );
        }
    }
}

/// ⭐⭐⭐ **A ALÇA E A CURVATURA SÃO A MESMA EXPRESSÃO NOS DOIS SENTIDOS** — a ida e a volta fecham,
/// e **o PONTO NEUTRO fecha AO BIT**.
///
/// ⚠️⚠️ **As duas metades não são a mesma afirmação, e a 1.ª redacção deste gate escreveu uma só e
/// reprovou.** Fora do neutro a volta perde bits por construção: a ida soma o deslocamento ao
/// TERÇO (`base + L·o`) e a volta subtrai o mesmo terço, e `(base + x) − base ≠ x` em `f64` quando
/// os dois têm a mesma ordem de grandeza. Medido: `4e-17` sobre um deslocamento de `0,21`.
///
/// ⭐ **O neutro, esse, é exacto** — e é ele o que tem de ser: `handles(L, STRAIGHT)` dá `L/3` e
/// `2L/3` exactos, a volta subtrai o mesmo número e divide, e `0/L` é `0`. *Um osso recto tem de
/// continuar recto depois de alguém lhe tocar na alça sem a arrastar.*
///
/// (Mutação: no `bend_from_handle`, trocar `length / 3.0` por `length * 2.0 / 3.0` ⇒ RED.)
#[test]
fn a_alca_e_a_curvatura_sao_a_mesma_expressao_nos_dois_sentidos() {
    let len = 7.5;
    // ⭐ O NEUTRO, ao bit — a metade load-bearing.
    let [n_inn, n_out] = crate::bend::handles(len, Bend::STRAIGHT);
    assert_eq!(
        crate::bend::bend_from_handle(len, n_inn, false),
        Some([0.0, 0.0])
    );
    assert_eq!(
        crate::bend::bend_from_handle(len, n_out, true),
        Some([0.0, 0.0])
    );
    for curva in [
        Bend::STRAIGHT,
        Bend {
            inn: [0.13, -0.37],
            out: [-0.21, 0.44],
        },
        Bend {
            inn: [-1.5, 2.0],
            out: [3.25, -0.125],
        },
    ] {
        let [inn, out] = crate::bend::handles(len, curva);
        for (v, esperado, nome) in [
            (
                crate::bend::bend_from_handle(len, inn, false),
                curva.inn,
                "raiz",
            ),
            (
                crate::bend::bend_from_handle(len, out, true),
                curva.out,
                "ponta",
            ),
        ] {
            let v = v.expect("ha' comprimento");
            assert!(
                (v[0] - esperado[0]).abs() < 1e-15 && (v[1] - esperado[1]).abs() < 1e-15,
                "a alca da {nome} de {curva:?} voltou {v:?}"
            );
        }
    }
    // ⛔ Um osso de comprimento zero não tem espaço local, e a porta di-lo em vez de dividir por ele.
    assert_eq!(crate::bend::bend_from_handle(0.0, [1.0, 1.0], false), None);
    // ⭐ E no ponto NEUTRO as alças estão exactamente nos terços — é dali que o nome vem.
    assert_eq!(
        crate::bend::handles(len, Bend::STRAIGHT),
        [[len / 3.0, 0.0], [len * 2.0 / 3.0, 0.0]]
    );
}

// ─────────────────────────────────────────────────────────────────────────────────────────────
// As alças DERIVADAS da corrente — o `Handles::Auto`
// ─────────────────────────────────────────────────────────────────────────────────────────────

/// ⭐⭐⭐⭐ **UMA CORRENTE RECTA COM `Auto` FICA RECTA, AO BIT** — o ponto neutro da lei nova.
///
/// ⚠️ **Não é «quase recta»:** as duas tangentes de um osso no meio de uma corrente recta são o
/// próprio eixo, e a conta do [`crate::bend::auto_bend`] é *«um terço da tangente MENOS um terço do
/// eixo»* — `1/3 − 1/3` é zero **exacto**. ⇒ ligar o *From Chain* num rig recto **não move um
/// pixel**, e é isso que faz o modo ser seguro de experimentar.
///
/// (Mutação: escrever a conta como `(t[0] − 1.0) / 3.0` ⇒ ainda dá zero; escrevê-la como
/// `t[0] / 3.0 − 0.333…` ⇒ RED.)
#[test]
fn uma_corrente_recta_com_alcas_automaticas_fica_recta_ao_bit() {
    // O eixo, e as duas tangentes que uma corrente recta produz.
    let b = crate::bend::auto_bend([1.0, 0.0], [1.0, 0.0]);
    assert_eq!(b, Bend::STRAIGHT);
    assert!(b.is_straight(), "e o `is_straight` tem de o reconhecer");
    // ⭐ E daí sai o colapso da fábrica: o osso continua a ser UM osso.
    assert!(
        BoneSpec {
            length: 10.0,
            strength: 1.0,
            segments: 16,
            curve: b,
        }
        .is_rigid(),
        "uma corrente recta com Auto tinha de continuar rigida"
    );
}

/// ⭐⭐⭐ **A ALÇA APONTA PARA ONDE O VIZINHO ESTÁ** — a lei do *From Chain*, medida pela geometria.
///
/// Com o osso seguinte dobrado para CIMA, a tangente da ponta sobe ⇒ a alça da ponta desce (ela
/// fica *antes* da ponta), e o corpo arqueia. ⚠️ **A régua é o SINAL e a simetria**, nunca um
/// número escolhido: uma tangente espelhada tem de dar uma alça espelhada.
#[test]
fn a_alca_automatica_aponta_para_onde_o_vizinho_esta() {
    let s = std::f64::consts::FRAC_1_SQRT_2;
    let cima = crate::bend::auto_bend([1.0, 0.0], [s, s]);
    let baixo = crate::bend::auto_bend([1.0, 0.0], [s, -s]);
    // A alça da RAIZ não se mexe: o vizinho que mudou é o da ponta.
    assert_eq!(cima.inn, [0.0, 0.0]);
    assert_eq!(baixo.inn, [0.0, 0.0]);
    // ⭐ E a da PONTA espelha-se exactamente.
    assert!(
        cima.out[1] < 0.0,
        "a tangente a subir puxa a alca para baixo"
    );
    assert!((cima.out[1] + baixo.out[1]).abs() < 1e-15, "sem simetria");
    assert!(
        (cima.out[0] - baixo.out[0]).abs() < 1e-15,
        "o `x` nao devia depender do sinal do `y`"
    );
    // ⚠️ E a magnitude é o TERÇO: a alça fica a `1/3` da ponta, na direcção da tangente.
    let dist = (1.0 / 3.0 - cima.out[0]).hypot(-cima.out[1]);
    assert!(
        (dist - 1.0 / 3.0).abs() < 1e-12,
        "a alca tinha de ficar a um terco da ponta, e ficou a {dist}"
    );
}

//! Os gates da LEI do pente — a unidade, sem malha e sem pincel.
//!
//! ⚠️ **A régua é a MESMA do corpus do oráculo**, `Q = média(cos 4α)`, escrita
//! aqui sobre o anel de um vértice só: ligar a lei de unidade à régua pela qual
//! o produto é julgado é o que impede as duas de divergirem.

use crate::{V3, pentear};

/// A régua da espec, sobre as arestas de um vértice: `Q = média(cos 4α)`, com
/// `α` o ângulo entre a aresta e a direcção do traço, no plano tangente.
fn q_do_anel(posicoes: &[V3], v: usize, vizinhos: &[u32], normal: V3, direccao: V3) -> f64 {
    let ao_longo = crate::unitario(crate::sem_componente(direccao, normal)).expect("quadro vivo");
    let atraves = crate::produto_vectorial(normal, ao_longo);
    let mut soma = 0.0f64;
    let mut n = 0u32;
    for &u in vizinhos {
        let plana =
            crate::sem_componente(crate::subtrair(posicoes[u as usize], posicoes[v]), normal);
        let (c, s) = (
            f64::from(crate::produto_escalar(plana, ao_longo)),
            f64::from(crate::produto_escalar(plana, atraves)),
        );
        if c == 0.0 && s == 0.0 {
            continue;
        }
        soma += (4.0 * s.atan2(c)).cos();
        n += 1;
    }
    soma / f64::from(n.max(1))
}

const CIMA: V3 = [0.0, 0.0, 1.0];
const TRACO: V3 = [1.0, 0.0, 0.0];

/// ⭐⭐⭐ **A GRADE JÁ ALINHADA É PONTO FIXO** — a propriedade que diz que isto é
/// uma relaxação e não um arrasto.
#[test]
fn uma_grade_ja_alinhada_nao_se_mexe() {
    let mut pos: Vec<V3> = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, -1.0, 0.0],
    ];
    let antes = pos.clone();
    let movidos = pentear(
        &mut pos,
        &[0],
        &[CIMA],
        &[1.0],
        &[0, 4],
        &[1, 2, 3, 4],
        TRACO,
    );
    assert_eq!(movidos, 0, "uma grade alinhada foi mexida");
    assert_eq!(pos, antes, "a saida nao e' byte-identica a' entrada");
}

/// ⭐⭐ **UM ANEL TORTO ALINHA-SE, e a conta fecha à mão.**
///
/// Dois vizinhos: um a `45°` do traço (comprimento `√2`) e outro já alinhado
/// (comprimento `2`). O raio médio do anel é `1,70711`, logo os alvos são
/// `±1,70711·x̂` e as contribuições são `(−0,70711 , 1)` e `(−0,29289 , 0)`;
/// a média dá `(−0,5 , 0,5)`, que mede `0,70711` — **acima do carril**
/// (`0,34 × 1,70711 = 0,58042`) ⇒ o passo é encolhido por `0,82083` e o vértice
/// pára em **`(−0,41041 , 0,41041)`**. Com isso o `Q` do anel sobe de `0,000`
/// para **`+0,259`**: os ângulos passam de `(45°, 0°)` para `(22,69°, 14,47°)`.
///
/// ⚠️ **Esta fixtura exercita a lei E o carril**, e os dois números estão na
/// conta acima — o carril tem gate próprio, e aqui ele aparece porque um anel
/// com duas arestas e uma delas a `45°` pede mesmo um passo grande.
///
/// ⚠️ **A aresta que já estava alinhada PIORA**, e isso é a lei a funcionar: ela
/// optimiza a **grade** do anel inteiro, não uma aresta de cada vez.
///
/// ⛔⛔ **E esta conta MUDOU em 2026-09-18, com a lei.** A 1.ª redacção rodava
/// cada aresta mantendo o comprimento DELA e parava em `(−0,20711 , 0,5)` com
/// `Q = +0,232`; ela alinhava sem guardar o espaçamento, e media na chapa um
/// pior ângulo de triângulo de **`0,31°`** contra `7,86°` do lado desligado.
/// *O gate disparou com o número novo à vista, que é o que ele existe para
/// fazer.*
#[test]
fn um_anel_torto_alinha_se_e_a_conta_fecha() {
    let mut pos: Vec<V3> = vec![[0.0, 0.0, 0.0], [1.0, 1.0, 0.0], [-2.0, 0.0, 0.0]];
    let antes_q = q_do_anel(&pos, 0, &[1, 2], CIMA, TRACO);
    let movidos = pentear(&mut pos, &[0], &[CIMA], &[1.0], &[0, 2], &[1, 2], TRACO);
    assert_eq!(movidos, 1);
    assert!(
        (pos[0][0] - -0.410_416).abs() < 1e-5 && (pos[0][1] - 0.410_416).abs() < 1e-5,
        "o vertice foi para {:?}, e a conta a' mao da' (-0,41041 , 0,41041)",
        pos[0]
    );
    let depois_q = q_do_anel(&pos, 0, &[1, 2], CIMA, TRACO);
    assert!(
        antes_q.abs() < 1e-6 && (depois_q - 0.2590).abs() < 1e-3,
        "o Q do anel foi de {antes_q:.4} para {depois_q:.4}; a conta a' mao da' 0,000 -> +0,259"
    );
}

/// ⭐ **O passo é TANGENCIAL** — ele não engorda nem come a forma.
///
/// ⚠️⚠️ **E isto é uma DIVERGÊNCIA DECLARADA, não paridade:** o oráculo mede uma
/// componente normal real (razão tangencial/normal `4,37`, espec §3.2) e a nossa
/// é **exactamente zero por construção**, porque o passo é montado no plano
/// tangente do próprio vértice. ⇒ o gate `G-5` da espec é **vácuo** sobre esta
/// lei, e o que ele protege — *o pente não mexe na forma* — mede-se assim.
#[test]
fn o_passo_nao_sai_do_plano_tangente() {
    let normal = crate::unitario([0.3, -0.5, 0.81]).expect("normal viva");
    let mut pos: Vec<V3> = vec![[0.0, 0.0, 0.0], [1.0, 1.0, 0.2], [-2.0, 0.3, -0.4]];
    let antes = pos[0];
    let movidos = pentear(&mut pos, &[0], &[normal], &[1.0], &[0, 2], &[1, 2], TRACO);
    assert_eq!(movidos, 1, "o arranjo deixou de conter o fenomeno");
    let passo = crate::subtrair(pos[0], antes);
    let fora = crate::produto_escalar(passo, normal).abs();
    assert!(
        fora < 1e-6 && crate::norma(passo) > 1e-3,
        "o passo mede {:.3e} fora do plano sobre um andar de {:.3e}",
        fora,
        crate::norma(passo)
    );
}

/// ⛔ **A degenerescência é INERTE, e são DUAS** — a espec §4.3 mede que o alvo
/// não inventa direcção nenhuma.
#[test]
fn sem_direccao_no_plano_o_vertice_fica() {
    let arranjo = |direccao: V3| {
        let mut pos: Vec<V3> = vec![[0.0, 0.0, 0.0], [1.0, 1.0, 0.0], [-2.0, 0.0, 0.0]];
        let movidos = pentear(&mut pos, &[0], &[CIMA], &[1.0], &[0, 2], &[1, 2], direccao);
        (movidos, pos)
    };
    // (1) — o controlo POSITIVO: com direcção viva, este arranjo move-se.
    assert_eq!(arranjo(TRACO).0, 1, "o arranjo nao contem o fenomeno");
    // (2) — sem carimbo anterior não há direcção nenhuma.
    assert_eq!(arranjo([0.0, 0.0, 0.0]).0, 0);
    // (3) — e uma direcção PARALELA à normal não tem projecção no plano: ela
    //       não é «para a frente» nenhuma, e inventar uma seria escolher por ele.
    assert_eq!(arranjo(CIMA).0, 0);
}

/// ⚠️ **O CARRIL morde, e é LOCAL.**
///
/// Uma vizinhança degenerada — duas arestas quase colineares e muito compridas —
/// pede um passo enorme; o tecto prende-o em `0,34` da aresta média **daquele**
/// vértice.
#[test]
fn o_carril_prende_o_passo_em_unidades_das_arestas_locais() {
    let mut pos: Vec<V3> = vec![[0.0, 0.0, 0.0], [3.0, 3.0, 0.0], [3.1, 2.9, 0.0]];
    let antes = pos[0];
    assert_eq!(
        pentear(&mut pos, &[0], &[CIMA], &[1.0], &[0, 2], &[1, 2], TRACO,),
        1
    );
    let andado = crate::norma(crate::subtrair(pos[0], antes));
    let aresta_local = (crate::norma([3.0, 3.0, 0.0]) + crate::norma([3.1, 2.9, 0.0])) / 2.0;
    let tecto = crate::TECTO_DA_VIAGEM * aresta_local;
    assert!(
        (andado - tecto).abs() < 1e-5,
        "o passo andou {andado:.5} contra o tecto local de {tecto:.5} — se ele \
         ficou ABAIXO, o arranjo deixou de pedir um salto e o carril nao esta' \
         a ser medido"
    );
}

/// ⛔⛔ **O LIMITE DECLARADO: uma grade PERFEITAMENTE simétrica a `45°` é ponto
/// fixo, e nenhum gate deve ser escrito sobre ela.**
///
/// Com quatro vizinhos em losango exacto as quatro contribuições cancelam-se aos
/// pares, e o vértice não anda — *não por a lei falhar, mas por `45°` ser um
/// equilíbrio INSTÁVEL de qualquer energia de alinhamento*: não há direcção
/// preferida para onde ir. Uma malha de topologia dinâmica é irregular por
/// construção e nunca está neste ponto.
///
/// ⚠️ **Isto está aqui para ninguém escrever o gate óbvio** — *«um losango a 45°
/// endireita-se»* — e concluir que a lei está partida quando ela lê `0`.
#[test]
fn um_losango_exacto_a_quarenta_e_cinco_graus_e_ponto_fixo_declarado() {
    let mut pos: Vec<V3> = vec![
        [0.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [-1.0, 1.0, 0.0],
        [-1.0, -1.0, 0.0],
        [1.0, -1.0, 0.0],
    ];
    let antes = pos.clone();
    assert_eq!(
        pentear(
            &mut pos,
            &[0],
            &[CIMA],
            &[1.0],
            &[0, 4],
            &[1, 2, 3, 4],
            TRACO,
        ),
        0,
        "o losango exacto mexeu-se — se isto passou a acontecer, a lei mudou e \
         a declaracao deste gate tem de ser re-medida, nao apagada"
    );
    assert_eq!(pos, antes);
}

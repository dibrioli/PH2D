//! ⭐⭐⭐ **OS GATES DA RAMPA DE PESO** — a legibilidade medida, com as duas rampas RECUSADAS
//! calculadas dentro do próprio gate.
//!
//! ⚠️ **Os controlos vivem aqui e não numa tabela de números:** uma barra escrita à mão envelhece no
//! dia em que alguém mexe nas paradas, e ninguém saberia dizer contra o quê ela foi calibrada. Aqui
//! a pergunta é *«ela é melhor do que as duas que recusámos?»*, e as duas são construídas à frente.

use super::{PARADAS, Rampa};
use ph2d_tokens::{ColorToken, Theme};

/// Quantas amostras a régua tira — `0,05` de peso por passo, que é a resolução a que um artista
/// lê um número numa tela.
const AMOSTRAS: usize = 20;

/// OKLab de um byte-trio, pela mesma porta que a rampa usa.
fn lab(c: [u8; 3]) -> [f64; 3] {
    let (l, ch, h) = ph2d_tokens::srgb_to_oklch(c[0], c[1], c[2]);
    let r = h.to_radians();
    [l, ch * r.cos(), ch * r.sin()]
}

fn de(a: [u8; 3], b: [u8; 3]) -> f64 {
    let (x, y) = (lab(a), lab(b));
    let d: [f64; 3] = std::array::from_fn(|k| x[k] - y[k]);
    d[2].mul_add(d[2], d[1].mul_add(d[1], d[0] * d[0])).sqrt()
}

/// Os `AMOSTRAS` passos adjacentes de uma rampa qualquer, em OKLab.
fn passos(f: impl Fn(f64) -> [u8; 3]) -> Vec<f64> {
    let cs: Vec<[u8; 3]> = (0..=AMOSTRAS)
        .map(|i| f(i as f64 / AMOSTRAS as f64))
        .collect();
    (0..AMOSTRAS).map(|i| de(cs[i], cs[i + 1])).collect()
}

/// A rampa do PRODUTO, em bytes.
fn produto(t: f64) -> [u8; 3] {
    let c = Rampa::nova().cor(t);
    let [r, g, b, _] = c.to_rgba8().to_u8_array();
    [r, g, b]
}

/// ⛔ **A RECUSADA nº 1: as cinco paradas espaçadas em `t`**, que é como a rampa é publicada.
fn ingenua(t: f64) -> [u8; 3] {
    let t = t.clamp(0.0, 1.0) * 4.0;
    let i = (t.floor() as usize).min(3);
    let u = t - i as f64;
    std::array::from_fn(|k| {
        let (a, b) = (f64::from(PARADAS[i][k]), f64::from(PARADAS[i + 1][k]));
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "u esta' em 0..1 e os extremos sao bytes, logo a soma cabe em u8"
        )]
        {
            (a + (b - a) * u).round() as u8
        }
    })
}

/// ⛔⛔ **A RECUSADA nº 3: as MESMAS paradas em OKLab, espaçadas em `t`.**
///
/// ⚠️ **Ela não estava aqui e foi uma MUTAÇÃO SOBREVIVENTE que a trouxe:** trocar a colocação por
/// arco pelo espaçamento uniforme passava o gate do planalto, porque a barra *«metade do passo
/// médio»* apanha o porte em sRGB (`0,0082`) e **não** este (`0,0378` contra uma barra de `0,0359`,
/// `5 %` de folga). ⭐ O que os separa é a UNIFORMIDADE — `0,325` contra `0,651` —, e é por isso que
/// a colocação por arco tem de ser afirmada contra ESTA e não só contra as outras duas.
fn oklab_espacada_em_t(t: f64) -> [u8; 3] {
    let r = Rampa::nova();
    let t = t.clamp(0.0, 1.0) * 4.0;
    let i = (t.floor() as usize).min(3);
    let u = t - i as f64;
    let m: [f64; 3] = std::array::from_fn(|k| r.lab[i][k] + (r.lab[i + 1][k] - r.lab[i][k]) * u);
    ph2d_tokens::oklch_to_srgb(m[0], m[1].hypot(m[2]), m[2].atan2(m[1]).to_degrees())
}

/// ⛔ **A RECUSADA nº 2: a rampa de ontem** — dois tokens semânticos, misturados em bytes sRGB.
fn dois_tons(t: f64) -> [u8; 3] {
    let frio = ColorToken::Info.resolve(Theme::Forge);
    let quente = ColorToken::Danger.resolve(Theme::Forge);
    let t = t.clamp(0.0, 1.0);
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "t esta' em 0..1 e os extremos sao bytes"
    )]
    let mix = |x: u8, y: u8| (f64::from(x) + (f64::from(y) - f64::from(x)) * t).round() as u8;
    [
        mix(frio.r, quente.r),
        mix(frio.g, quente.g),
        mix(frio.b, quente.b),
    ]
}

/// ⭐⭐⭐ **NENHUM PASSO DA RAMPA É UM PLANALTO** — e a barra é DERIVADA, não escolhida.
///
/// A barra é **metade do passo médio**: uma rampa perfeitamente uniforme daria o passo médio em
/// todo o lado, e ficar abaixo de metade dele é o que faz dois pesos vizinhos serem a mesma cor.
///
/// ⛔ **É exactamente aqui que o porte INGÉNUO reprova** (o verde puro é um planalto), e o gate
/// mede-o à frente para a barra não poder ser lida como confortável.
#[test]
fn nenhum_passo_da_rampa_de_peso_e_um_planalto() {
    let p = passos(produto);
    let media: f64 = p.iter().sum::<f64>() / p.len() as f64;
    let pior = p.iter().copied().fold(f64::INFINITY, f64::min);
    assert!(
        pior >= 0.5 * media,
        "a rampa tem um planalto: pior passo {pior:.4} contra metade da media ({:.4})",
        0.5 * media
    );
    // ⭐ O CONTROLO: a rampa publicada — as mesmas cores, espaçadas em `t` — REPROVA nesta barra.
    // Sem ele, alguém leria a barra acima como uma formalidade.
    let q = passos(ingenua);
    let media_q: f64 = q.iter().sum::<f64>() / q.len() as f64;
    let pior_q = q.iter().copied().fold(f64::INFINITY, f64::min);
    assert!(
        pior_q < 0.5 * media_q,
        "o porte ingenuo deixou de ter planalto ({pior_q:.4} contra {:.4}) — a barra deixou de \
         discriminar as duas parametrizacoes, e este gate passou a nao afirmar nada",
        0.5 * media_q
    );
}

/// ⭐⭐⭐ **A RAMPA BATE AS DUAS QUE ELA RECUSA, no número que decide.**
///
/// ⛔⛔ **O número que decide é o PIOR passo e não o caminho total** — uma rampa pode percorrer
/// muito espaço de cor e ainda assim ter dois pesos vizinhos indistinguíveis, que é precisamente o
/// defeito do porte ingénuo (caminho `4,7 ×` o da rampa de ontem e pior passo `0,63 ×`).
#[test]
fn a_rampa_de_peso_e_mais_legivel_que_as_duas_recusadas() {
    let pior = |f: &dyn Fn(f64) -> [u8; 3]| passos(f).iter().copied().fold(f64::INFINITY, f64::min);
    let (nosso, ing, dois) = (pior(&produto), pior(&ingenua), pior(&dois_tons));
    assert!(
        nosso > dois * 2.0,
        "a rampa nova nao e' o DOBRO de legivel que a de ontem: pior passo {nosso:.4} contra \
         {dois:.4} ({:.2}x)",
        nosso / dois
    );
    assert!(
        nosso > ing * 2.0,
        "a rampa nova nao bate o porte ingenuo: {nosso:.4} contra {ing:.4}"
    );
    // ⭐ E a TERCEIRA: as mesmas paradas em OKLab, espaçadas em `t`. Ela é a que o gate do planalto
    // deixa passar, e é esta linha que torna a colocação por ARCO uma propriedade e não um detalhe.
    let plana = pior(&oklab_espacada_em_t);
    assert!(
        nosso > plana * 1.25,
        "a colocacao por ARCO deixou de comprar legibilidade sobre o espacamento uniforme:          {nosso:.4} contra {plana:.4} ({:.2}x)",
        nosso / plana
    );
}

/// ⭐ **AS PONTAS SÃO AS PARADAS, e a ida-e-volta pela porta de cor não as move.**
///
/// ⚠️ **Ela prova que a porta ([`ph2d_tokens::srgb_to_oklch`] / [`ph2d_tokens::oklch_to_srgb`]) é
/// usada com a convenção certa** — um ângulo em radianos onde ela quer graus daria uma rampa
/// plausível e errada, e nenhuma das réguas acima o veria.
#[test]
fn as_pontas_da_rampa_sao_as_paradas_declaradas() {
    assert_eq!(
        produto(0.0),
        PARADAS[0],
        "o peso zero deixou de ser a 1.a parada"
    );
    assert_eq!(
        produto(1.0),
        PARADAS[4],
        "o peso um deixou de ser a ultima parada"
    );
    // Fora da faixa satura, e nao envolve.
    assert_eq!(produto(-3.0), PARADAS[0]);
    assert_eq!(produto(9.0), PARADAS[4]);
    // ⭐ E cada parada interior é ALCANÇADA — se a posição por arco se perdesse, o verde sumia.
    for parada in PARADAS {
        let achou = (0..=200).any(|i| de(produto(f64::from(i) / 200.0), parada) < 0.02);
        assert!(
            achou,
            "a parada {parada:?} nao aparece em sitio nenhum da rampa"
        );
    }
}

/// Quantos caminhos o retículo encoda — o oráculo é o que foi MESMO desenhado.
fn caminhos(verts: &[[f64; 2]], pesos: &[f64], tris: &[[u32; 3]]) -> u32 {
    let mut cena = ph2d_vector::VectorScene::new();
    super::draw_weight_mesh(
        verts,
        pesos,
        tris,
        ph2d_vector::Affine::IDENTITY,
        Theme::Forge,
        &mut cena,
    );
    cena.inner().encoding().n_paths
}

/// ⭐⭐⭐ **CADA TRIÂNGULO ENCODA UM PREENCHIMENTO E TRÊS ARESTAS, E CADA VÉRTICE UM PONTO.**
///
/// ⛔⛔ **A premissa deste gate MUDOU e a morte está à vista no diff** (report do dono,
/// 2026-09-20: *«os pesos não estão nos vértices da malha»*). Ele esperava **um** caminho de
/// aresta por triângulo, o que é o mesmo que dizer *«a aresta é do TRIÂNGULO»* — e era isso que
/// fazia a cor de uma linha da grelha ser a de quem desenhou por último. Hoje são três, uma por
/// PAR de vértices, e mais um ponto por vértice.
///
/// ⚠️ **As metades NEGATIVAS ficam, e uma delas TROCOU de número com razão:** um índice fora da
/// malha salta **o triângulo** e os vértices continuam a ser desenhados — *o peso de um vértice
/// não depende de nenhum triângulo o citar*.
#[test]
fn cada_triangulo_encoda_o_preenchimento_as_arestas_e_os_vertices() {
    let v = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
    let w = [0.0, 0.5, 1.0, 0.25];
    let t = [[0, 1, 2], [0, 2, 3]];
    assert_eq!(
        caminhos(&v, &w, &t),
        2 * (1 + 3) + 4,
        "esperava um preenchimento e TRÊS arestas por triângulo, mais um ponto por vértice"
    );
    assert_eq!(caminhos(&v, &w, &[]), 0, "sem triângulos não há retículo");
    assert_eq!(
        caminhos(&v, &w[..3], &t),
        0,
        "um par que não fecha tem de ser RECUSADO — desenhá-lo dá a cada vértice o peso do vizinho"
    );
    assert_eq!(
        caminhos(&v, &w, &[[0, 1, 9]]),
        4,
        "um índice fora da malha tem de saltar o TRIÂNGULO e deixar os quatro vértices"
    );
}

/// ⭐⭐ **OS TRÊS CANTOS CHEGAM INTEIROS, E A MÉDIA É DERIVADA DELES.**
///
/// ⛔⛔ **A premissa deste gate MUDOU e a morte está à vista no diff.** Ele chamava-se
/// `a_cor_do_triangulo_e_a_media_dos_tres_cantos` e media uma `media()` que devolvia UM número —
/// e essa função foi a causa de a ARESTA ser pintada pela média do triângulo (ver o cabeçalho do
/// [`super::draw_weight_mesh`]): *com a média já feita aqui, quem desenha a aresta já não sabe
/// quais são os dois vértices dela*. A porta devolve os três, e a média passou a ser do
/// chamador.
///
/// ⚠️ A recusa fica: um peso não-finito ou um índice fora da tabela saltam o triângulo inteiro.
#[test]
fn os_tres_cantos_chegam_inteiros_e_a_media_e_derivada() {
    let w = [0.0, 0.6, 0.9, f64::NAN];
    let c = super::cantos(&w, &[0, 1, 2]).expect("os três cantos existem");
    assert_eq!(
        c,
        [0.0, 0.6, 0.9],
        "os cantos não chegam na ordem do triângulo"
    );
    let m = (c[0] + c[1] + c[2]) / 3.0;
    assert!((m - 0.5).abs() < 1e-12, "média errada: {m}");
    assert!(
        super::cantos(&w, &[0, 1, 3]).is_none(),
        "um peso não-finito tem de recusar o triângulo"
    );
    assert!(
        super::cantos(&w, &[0, 1, 9]).is_none(),
        "um índice fora da tabela tem de recusar o triângulo"
    );
}

/// As cores distintas que o retículo encoda — o oráculo é o `draw_data` da cena.
fn cores(verts: &[[f64; 2]], pesos: &[f64], tris: &[[u32; 3]]) -> std::collections::BTreeSet<u32> {
    let mut cena = ph2d_vector::VectorScene::new();
    super::draw_weight_mesh(
        verts,
        pesos,
        tris,
        ph2d_vector::Affine::IDENTITY,
        Theme::Forge,
        &mut cena,
    );
    cena.inner().encoding().draw_data.iter().copied().collect()
}

/// ⭐⭐⭐ **A ARESTA É A MESMA RAMPA DO PREENCHIMENTO** — uma leitura, duas intensidades.
///
/// ⛔⛔ **O defeito que ele impede tem foto** (2026-09-20): a 1.ª redacção pintava a aresta com um
/// token NEUTRO e o preenchimento com a rampa — *duas tintas para o mesmo número*, e o olho lê a
/// grelha como uma coisa e o campo como outra.
///
/// ⭐⭐⭐ **E A CONTAGEM SEPARA TRÊS LEIS DE UMA VEZ**, com os pesos `[0, 0, 1, 1]` sobre dois
/// triângulos que partilham a aresta `0–2`:
///
/// | lei da aresta | cores distintas |
/// |---|---:|
/// | uma tinta NEUTRA (a 1.ª redacção, com foto) | `3` |
/// | a média do TRIÂNGULO (a de 2026-09-20 de manhã) | `4` |
/// | **a média dos DOIS vértices dela** (hoje) | **`5`** |
///
/// ⭐ As três leem `{0, ½, 1}` nas arestas contra `{⅓, ⅔}` das médias — *é por a aresta partilhada
/// ser pintada pelos seus próprios extremos que aparece a terceira cor*.
///
/// ⚠️ **E o CONTROLO vem primeiro:** dois triângulos do MESMO peso têm de dar **duas** — sem ele,
/// um `draw_data` que guardasse uma cor por caminho passaria a contar sempre e o gate afirmaria
/// nada.
#[test]
fn a_aresta_do_reticulo_e_a_mesma_rampa_e_sai_dos_dois_vertices_dela() {
    let v = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
    let t = [[0, 1, 2], [0, 2, 3]];

    let iguais = cores(&v, &[0.5; 4], &t);
    assert_eq!(
        iguais.len(),
        2,
        "dois triângulos do mesmo peso têm de dar UM par de cores: {iguais:?}"
    );
    let distintos = cores(&v, &[0.0, 0.0, 1.0, 1.0], &t);
    assert_eq!(
        distintos.len(),
        5,
        "a aresta tem de sair dos DOIS vértices dela — pela média do triângulo isto lê 4 e com uma \
         tinta neutra lê 3: {distintos:?}"
    );
}

/// ⭐⭐⭐ **O PESO DE UM VÉRTICE CHEGA À TELA** — a resposta literal ao report de 2026-09-20.
///
/// ⚠️ **A fixtura é construída para que NENHUMA média o produza:** com `[0, 0, 1]` num triângulo
/// só, a média é `⅓` e as três arestas são `0`, `½` e `½` — *o peso `1` do vértice `2` não é a cor
/// de nenhum preenchimento nem de nenhuma aresta*. ⇒ se ele aparecer na tela, veio do PONTO.
///
/// ⛔ **A referência não é um número escrito à mão:** ela é a mesma malha com os três pesos a `1`,
/// e o que se afirma é a INTERSECÇÃO — *um `u32` de cor copiado para um gate envelhece no dia em
/// que o encoder mudar de formato*.
#[test]
fn cada_vertice_encoda_o_peso_dele() {
    let v = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]];
    let t = [[0, 1, 2]];
    let cheio = cores(&v, &[1.0; 3], &t);
    let c = cores(&v, &[0.0, 0.0, 1.0], &t);
    let comuns: Vec<u32> = c.intersection(&cheio).copied().collect();
    assert_eq!(
        comuns.len(),
        1,
        "a cor do peso 1 tem de estar na tela UMA vez (o ponto do vértice 2) e veio {}: {c:?} vs \
         {cheio:?}",
        comuns.len()
    );
    assert_eq!(
        caminhos(&v, &[0.0, 0.0, 1.0], &t),
        1 + 3 + 3,
        "um preenchimento, três arestas e três pontos"
    );
}

/// ⚠️ **SONDA — quantos BYTES de cor vale um degrau de peso?** É dela que sai a tolerância da
/// subdivisão do preenchimento: um triângulo cuja cor varia menos do que um byte não tem o que
/// mostrar por dentro.
#[test]
fn diag_quantos_bytes_vale_um_degrau_de_peso() {
    let r = super::Rampa::nova();
    let n = 4096usize;
    let mut pior = 0.0_f64;
    let mut onde = 0.0;
    for k in 0..n {
        let (a, b) = (k as f64 / n as f64, (k + 1) as f64 / n as f64);
        let (ca, cb) = (
            r.cor(a).to_rgba8().to_u8_array(),
            r.cor(b).to_rgba8().to_u8_array(),
        );
        let d = (0..3)
            .map(|i| (f64::from(ca[i]) - f64::from(cb[i])).abs())
            .fold(0.0_f64, f64::max)
            / (b - a);
        if d > pior {
            pior = d;
            onde = a;
        }
    }
    println!("\nbytes por unidade de peso: pior {pior:.1} em t = {onde:.3}");
    println!("⇒ 1 byte  = {:.5} de peso", 1.0 / pior);
    for k in 0..12 {
        let t = onde - 0.003 + f64::from(k) * 0.0006;
        let c = r.cor(t).to_rgba8().to_u8_array();
        println!("  t = {t:.5}  ->  {:>3} {:>3} {:>3}", c[0], c[1], c[2]);
    }
    // O percentil, que é o que descreve a rampa: o pior é UM degrau.
    let mut todos: Vec<f64> = (0..n)
        .map(|k| {
            let (a, b) = (k as f64 / n as f64, (k + 1) as f64 / n as f64);
            let (ca, cb) = (
                r.cor(a).to_rgba8().to_u8_array(),
                r.cor(b).to_rgba8().to_u8_array(),
            );
            (0..3)
                .map(|i| (f64::from(ca[i]) - f64::from(cb[i])).abs())
                .fold(0.0_f64, f64::max)
                / (b - a)
        })
        .collect();
    todos.sort_by(|a, b| a.partial_cmp(b).expect("finito"));
    println!(
        "p50 {:.0} · p99 {:.0} · max {:.0} bytes por unidade",
        todos[n / 2],
        todos[n * 99 / 100],
        todos[n - 1]
    );
}

/// ⚠️ **SONDA — o que o retículo custa a ENCODAR**, na malha medida da cena do dono (`498`
/// vértices, `878` triângulos). Ele é redesenhado a cada quadro enquanto o pincel de peso está na
/// mão, logo o orçamento é o do quadro (`16,67 ms`).
#[test]
fn diag_o_preco_de_encodar_o_reticulo() {
    let n = 498usize;
    let verts: Vec<[f64; 2]> = (0..n)
        .map(|i| {
            #[expect(clippy::cast_precision_loss, reason = "índice de fixtura")]
            let f = i as f64;
            [f * 0.017, (f * 0.31).sin()]
        })
        .collect();
    let pesos: Vec<f64> = (0..n)
        .map(|i| {
            #[expect(clippy::cast_precision_loss, reason = "índice de fixtura")]
            let f = i as f64;
            (f / n as f64).clamp(0.0, 1.0)
        })
        .collect();
    let tris: Vec<[u32; 3]> = (0..878u32)
        .map(|k| {
            let a = k % (n as u32 - 2);
            [a, a + 1, a + 2]
        })
        .collect();
    let t0 = std::time::Instant::now();
    let voltas = 20;
    for _ in 0..voltas {
        let mut cena = ph2d_vector::VectorScene::new();
        super::draw_weight_mesh(
            &verts,
            &pesos,
            &tris,
            ph2d_vector::Affine::IDENTITY,
            Theme::Forge,
            &mut cena,
        );
        std::hint::black_box(cena.inner().encoding().n_paths);
    }
    let ms = t0.elapsed().as_secs_f64() * 1000.0 / f64::from(voltas);
    println!(
        "\nencodar o retículo: {ms:.3} ms ({:.2} % de um quadro) — {} caminhos",
        ms / 16.67 * 100.0,
        tris.len() * 4 + n
    );
}

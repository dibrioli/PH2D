//! **OS GATES DA RÉGUA DA FILEIRA.**
//!
//! ⚠️ **Uma régua nova sem os DOIS extremos medidos não afirma nada.** Ela é
//! pregada entre uma malha que **é** uma grade (onde tem de ler a grade inteira)
//! e uma que não tem direcção nenhuma (onde tem de ler o ruído do acaso), e é
//! esse vale que dá sentido a qualquer leitura no meio.
//!
//! ```text
//! cargo test -p ph2d-sculpt3d --lib fileira
//! ```

use ph2d_mesh::{Face, Mesh};

use super::fileira_da_faixa;

/// Uma chapa `n × n` de lado `1`, triangulada por leque de quadrado — **é** uma
/// grade perfeita alinhada com `+x`.
fn grade(n: usize) -> Mesh {
    let mut pos = Vec::new();
    for j in 0..n {
        for i in 0..n {
            pos.push([
                i as f32 / (n - 1) as f32 - 0.5,
                j as f32 / (n - 1) as f32 - 0.5,
                0.0,
            ]);
        }
    }
    let mut faces = Vec::new();
    for j in 0..n - 1 {
        for i in 0..n - 1 {
            let a = (j * n + i) as u32;
            let (b, c, d) = (a + 1, a + n as u32, a + n as u32 + 1);
            faces.push(Face::tri(a, b, d));
            faces.push(Face::tri(a, d, c));
        }
    }
    Mesh::from_parts(pos, faces).expect("a grade é bem formada")
}

/// A MESMA grade com os vértices sacudidos — a malha **sem direcção** que serve
/// de chão.
///
/// ⚠️ O ruído é **grande** de propósito (`0,35` do passo): sacudir de leve
/// deixa uma grade torta, que ainda é uma grade, e o chão da régua leria alto.
fn sacudida(n: usize) -> Mesh {
    let mut m = grade(n);
    let h = 1.0 / (n - 1) as f32;
    let mut semente = 0x2545_f491_4f6c_dd1du64;
    for p in m.positions_mut() {
        for c in p.iter_mut().take(2) {
            semente ^= semente << 13;
            semente ^= semente >> 7;
            semente ^= semente << 17;
            let r = (semente >> 40) as f32 / 16_777_216.0 - 0.5;
            *c += r * h * 0.7;
        }
    }
    m
}

/// O percurso que a régua mede: uma linha ao longo de `+x`, no meio da chapa.
fn percurso() -> Vec<[f32; 3]> {
    (0..9)
        .map(|k| [-0.4 + 0.1 * k as f32, 0.0, 0.0])
        .collect()
}

/// ⭐⭐⭐ **GATE — NUMA GRADE A RÉGUA LÊ A GRADE INTEIRA.**
///
/// A chapa `25×25` tem `24` arestas de ponta a ponta e a régua lê **`23`** — a
/// aresta que falta é a da ponta, cujo ponto médio cai fora da faixa (a [`FAIXA`]
/// é meio raio à volta do PERCURSO, e o percurso acaba antes da borda). *A régua
/// mede a faixa, não a chapa.*
///
/// ⚠️ Sem este extremo, uma régua que lesse `3` em toda a parte pareceria estar
/// a funcionar.
///
/// [`FAIXA`]: crate::medida_do_pente::FAIXA
#[test]
fn numa_grade_a_fileira_atravessa_a_faixa() {
    let m = grade(25);
    let (p50, _, maior, n) = fileira_da_faixa(&m, &percurso(), 0.3);
    assert!(n >= 4, "cadeias: {n} — a faixa está a medir o nada");
    assert!(
        p50 >= 20.0,
        "numa GRADE a fileira mediana é de {p50} arestas — a régua não vê a \
         continuidade que ela existe para medir"
    );
    assert!(
        maior >= 23,
        "a maior fileira mede {maior} arestas — medido 23 de 24, e a que falta \
         e' a da ponta, fora da faixa"
    );
}

/// ⭐⭐⭐ **GATE — E NUMA MALHA SEM DIRECÇÃO ELA LÊ O CHÃO.**
///
/// ⛔⛔ **É esta metade que dá sentido à outra.** Sem ela, uma régua que
/// devolvesse *«a faixa inteira»* a toda a malha passaria no gate de cima e
/// aprovaria qualquer coisa — que é exactamente como as três réguas anteriores
/// desta cena deixaram passar três reprovações do dono.
#[test]
fn numa_malha_sacudida_a_fileira_e_curta() {
    let m = sacudida(25);
    let (p50, p90, _, n) = fileira_da_faixa(&m, &percurso(), 0.3);
    assert!(n >= 4, "cadeias: {n}");
    assert!(
        p50 <= 4.0,
        "numa malha SEM direcção a fileira mediana é de {p50} arestas — o chão \
         da régua está alto e ela vai aprovar retalhos"
    );
    let (g50, _, _, _) = fileira_da_faixa(&grade(25), &percurso(), 0.3);
    assert!(
        g50 >= p50 * 4.0,
        "a grade lê {g50} e a sacudida {p50} — a régua não SEPARA os dois \
         extremos, e uma régua que não separa não é uma régua"
    );
    // ⚠️ E o `p90` do chão é o número que qualquer leitura no meio tem de
    // bater para significar alguma coisa.
    assert!(p90 <= 8.0, "p90 do chão: {p90}");
}

/// ⚠️ **Uma faixa vazia devolve ZEROS, e quem chama tem de olhar para a
/// contagem** — *um zero de «não medido» e um de «sem fileira» são o mesmo
/// byte*, que é a lição que a `q_da_faixa` já traz escrita.
#[test]
fn uma_faixa_vazia_nao_finge_uma_leitura() {
    let m = grade(9);
    let fora = vec![[10.0, 10.0, 0.0], [10.1, 10.0, 0.0]];
    assert_eq!(fileira_da_faixa(&m, &fora, 0.3), (0.0, 0.0, 0, 0));
}

/// A mesma grade com cada fileira em **ZIGUE-ZAGUE** de `±14°` — arestas
/// alinhadas (a `14° < 15°` do traço) e **ligadas**, que a olho não são uma
/// linha.
fn ziguezague(n: usize) -> Mesh {
    let mut m = grade(n);
    let h = 1.0 / (n - 1) as f32;
    // `tan 14° ≈ 0,2493`: meio degrau para cada lado dá segmentos a `±14°`.
    let meio = 0.5 * h * 0.2493;
    for (i, p) in m.positions_mut().iter_mut().enumerate() {
        p[1] += if i % 2 == 0 { meio } else { -meio };
    }
    m
}

/// ⭐⭐⭐⭐ **GATE — UM ZIGUE-ZAGUE NÃO É UMA LINHA.**
///
/// ⛔⛔ **Ele nasceu de uma MUTAÇÃO SOBREVIVENTE**, e ela expôs que o limiar da
/// continuação estava **inerte por construção**: duas arestas cada uma a menos
/// de [`ALINHADA`] da mesma direcção diferem no máximo `2 × ALINHADA`, logo um
/// limiar em `30°` nunca recusava nada. *E nenhuma das fixturas que eu tinha
/// continha o fenómeno* — na grade as arestas continuam-se a `0°`, e na sacudida
/// elas nem chegam a ligar-se.
///
/// Aqui elas ligam-se **e** viram `28°` a cada passo: a régua tem de as CORTAR.
///
/// [`ALINHADA`]: super::ALINHADA
#[test]
fn uma_fileira_em_ziguezague_nao_e_uma_linha() {
    let m = ziguezague(25);
    let (p50, _, maior, n) = fileira_da_faixa(&m, &percurso(), 0.3);
    assert!(n >= 4, "cadeias: {n} — a faixa está a medir o nada");
    // ⭐ **O CONTROLO vem primeiro:** a mesma malha SEM o zigue-zague lê a grade
    // inteira, logo o que se mede aqui é o zigue-zague e não a fixtura.
    let (g50, _, gmax, _) = fileira_da_faixa(&grade(25), &percurso(), 0.3);
    assert!(g50 >= 20.0 && gmax >= 23, "o controlo mudou: {g50} / {gmax}");
    assert!(
        p50 <= 2.0,
        "um zigue-zague de ±14° leu fileiras de {p50} arestas — a régua encadeia \
         o que o olho não segue"
    );
    assert!(
        maior <= 4,
        "a maior cadeia do zigue-zague mede {maior} arestas"
    );
}

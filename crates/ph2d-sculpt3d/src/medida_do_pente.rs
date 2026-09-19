//! ⭐⭐⭐ **AS TRÊS COLUNAS DO PENTE** — a régua do alinhamento, a do pior
//! triângulo e a que o OLHO lê, numa porta só.
//!
//! ⛔⛔⛔ **A terceira chegou em 2026-09-18, e custou um report com FOTO**
//! (*«pouca ou nenhuma diferença»*): as duas primeiras são uma **média** e uma
//! **cerca**, e nenhuma responde *«que fracção das arestas mudou de rumo»* —
//! que é o que o olho faz. O `Q` subia `+0,09` e as duas imagens do arame eram
//! indistinguíveis. Ver [`grade_da_faixa`].
//!
//! # ⛔⛔ Porque a SEGUNDA existe: ela apanhou um defeito que a primeira aprovava
//!
//! Medido nesta linha: a 1.ª redacção da lei rodava cada aresta guardando o
//! comprimento **dela**, e isso alinha sem guardar o espaçamento — duas vizinhas
//! caem sobre a mesma linha e o triângulo entre elas fecha. A régua do
//! alinhamento subia (`−0,019 → +0,147`) enquanto o pior triângulo da faixa ia
//! de `7,86°` para **`0,31°`**, e um triângulo de três décimos de grau não tem
//! normal utilizável, logo não tem sombra. *Uma régua que vê só direcções
//! aprova uma malha destruída que por acaso ficou alinhada.*
//!
//! # ⚠️ Ela é uma PORTA porque tem DOIS consumidores em espaços diferentes
//!
//! A bancada da lei corre sobre uma **chapa** e a cena de smoke sobre uma
//! **bola**. Duas cópias divergiriam na primeira wave que mexesse numa delas, e
//! a que o dono vê é a que envelhece — a lei que este módulo já pagou com o
//! `stroke_uniform` e com o `compact_for_faces`.
//!
//! # ⭐⭐ E é por ser porta que ela deixou de ser PLANA
//!
//! A 1.ª redacção media `x` e `y` e deitava o `z` fora: ela nasceu sobre a chapa
//! do corpus, e *uma régua medida numa fixtura plana não afirma nada sobre uma
//! peça curva* — a mesma frase que o `Scene Project` pagou com um report.
//!
//! ⭐ **A generalização não precisou de base tangente nenhuma**, e é isso que a
//! torna exacta: `cos 4α` depende só de `|α|`, logo
//! `cos 4α = 8c⁴ − 8c² + 1` com `c = ê · d̂` **em 3D**. Sobre uma chapa (onde toda
//! aresta e toda direcção vivem no plano) as duas formas concordam a **`2,4e-8`**
//! — *a forma velha é um caso particular desta, não uma aproximação dela*, e o
//! resíduo é só o que separa um `atan2`+`cos` de um polinómio de grau quatro
//! sobre entradas nascidas em `f32`. Gate:
//! [`tests::sobre_uma_peca_plana_a_regua_3d_e_a_de_duas_coordenadas`].

//! # ⭐⭐⭐ A ESCADA QUE ESTAS DUAS COLUNAS LERAM
//!
//! Medida pelo produto, na chapa sacudida com o passe de refino a correr — é
//! ela que fixa o tecto do [`crate::Brush::pente`]:
//!
//! | pente | alinhamento `Q` | pior ângulo da faixa |
//! |---|---|---|
//! | `0,000` | `−0,0197` | `7,86°` |
//! | `0,125` | `+0,0148` | `8,06°` |
//! | `0,250` | `+0,0437` | **`8,21°`** |
//! | `0,500` | `+0,0840` | `6,57°` |
//! | `0,750` | `+0,1082` | `5,04°` |
//! | **`1,000`** | **`+0,1270`** | **`4,56°`** |
//! | `2,000` | `+0,1641` | `3,76°` |
//! | `3,000` | `+0,1781` | `0,62°` |
//!
//! ⚠️ **Ela foi RE-MEDIDA quando as duas réguas viraram esta porta**, e as duas
//! colunas dizem coisas diferentes sobre a troca: o pior ângulo saiu
//! **IDÊNTICO** em todos os degraus (a conta dele já era 3D; só a escolha da
//! faixa mudou, e ela não trocou de triângulos) e o `Q` moveu-se na **quarta
//! casa** — meia dúzia de arestas a entrar e a sair de uma faixa numa chapa que
//! o carimbo levanta.
//!
//! # ⛔⛔ A FRONTEIRA DOS 45°, que a chapa não continha
//!
//! A tabela acima corre sobre uma chapa cuja grade **já está ao longo** do
//! traço. Sobre uma bola de malha REGULAR, riscada em três rumos contra a grade,
//! com o pente no tecto:
//!
//! | rumo do traço | `Q` desligado → no tecto | triângulos `< 5°` |
//! |---|---|---|
//! | `30°` da grade | `−0,0778 → +0,0138` | **`0` de `3 418`** |
//! | **`45°`** | `−0,1882 → −0,0080` | **`3` de `3 356`** |
//! | `60°` | `−0,1199 → −0,0020` | **`0` de `3 331`** |
//!
//! ⇒ **a `45°` exactos ficam um a três triângulos degenerados**, e o mecanismo é
//! a própria lei: o alvo tem **QUATRO dobras**, logo as duas direcções de uma
//! grade valem *exactamente* o mesmo e a meio caminho entre elas ela não tem
//! para que lado virar.
//!
//! ⛔ **O tecto NÃO desceu por causa disto:** aplicar ali a regra do tecto daria
//! `0,375` (a `0,5` o pior triângulo cai de `12,03°` para `3,91°`), o que
//! custaria **60 % do curso em todos os outros rumos** por `0,09 %` dos
//! triângulos de um só. *Seria o caminho mais lento a definir o tecto do mais
//! rápido.* A fronteira fica **declarada**, o gate da cena `=49` conta-a e o
//! roteiro dela diz o número ao artista.

use ph2d_mesh::Mesh;

/// Quanto da faixa do traço entra na conta: **meio raio** de cada lado do
/// percurso.
///
/// ⚠️ **Meio e não um inteiro:** a queda do carimbo leva o peso a zero na borda,
/// logo o anel de fora do pincel está quase intocado e diluiria as duas colunas
/// com malha que ninguém penteou. É a mesma fracção com que o corpus do oráculo
/// declara a região central dele.
pub const FAIXA: f32 = 0.5;

/// O ponto do percurso mais perto de `p`, e a direcção LOCAL dele — ou `None`
/// quando `p` cai fora da faixa.
///
/// ⚠️ **`pub(crate)` porque a régua da FILEIRA a lê** ([`crate::medida_da_fileira`]):
/// *«que troço do traço é este ponto, e que faixa conta»* é uma pergunta só, e
/// duas respostas divergiriam no dia em que a [`FAIXA`] mudasse.
pub(crate) fn direccao_do_troco(percurso: &[[f32; 3]], p: [f32; 3], raio: f32) -> Option<[f32; 3]> {
    let mut melhor = f32::INFINITY;
    let mut direccao = None;
    for par in percurso.windows(2) {
        let d = dist(p, par[1]);
        if d < melhor {
            melhor = d;
            direccao = Some(sub(par[1], par[0]));
        }
    }
    (melhor <= raio * FAIXA).then_some(direccao?)
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = sub(a, b);
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

fn comprimento(v: [f32; 3]) -> f64 {
    f64::from(v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// **A PRIMEIRA COLUNA — `Q = média(cos 4α)` sobre as arestas da faixa.**
///
/// `Q = 0` é uma malha sem direcção nenhuma; `Q > 0` é uma **GRADE** alinhada
/// com o traço. Devolve `(Q, quantas arestas entraram)`.
///
/// ⛔ **É o parâmetro de ordem de QUATRO dobras, e a escolha é obrigatória:** o
/// de duas (`cos 2α`) é **cego a uma grade por construção** — ele lê `0` numa
/// grade perfeita, porque as duas famílias de linhas se cancelam.
///
/// ⚠️ **A contagem devolvida NÃO é decoração:** uma faixa vazia devolveria
/// `Q = 0`, que é exactamente o que uma malha desalinhada lê. *Um zero de «não
/// medido» e um de «sem direcção» são o mesmo byte* — quem chama tem de olhar
/// para `n`.
#[must_use]
pub fn q_da_faixa(malha: &Mesh, percurso: &[[f32; 3]], raio: f32) -> (f64, usize) {
    let pos = malha.positions();
    let mut vistas = std::collections::BTreeSet::new();
    let (mut soma, mut n) = (0.0f64, 0usize);
    for f in malha.faces() {
        let vs = f.verts();
        for k in 0..vs.len() {
            let (a, b) = (vs[k], vs[(k + 1) % vs.len()]);
            if !vistas.insert((a.min(b), a.max(b))) {
                continue;
            }
            let (pa, pb) = (pos[a as usize], pos[b as usize]);
            let meio = [
                (pa[0] + pb[0]) * 0.5,
                (pa[1] + pb[1]) * 0.5,
                (pa[2] + pb[2]) * 0.5,
            ];
            let Some(direccao) = direccao_do_troco(percurso, meio, raio) else {
                continue;
            };
            let aresta = sub(pb, pa);
            let (la, ld) = (comprimento(aresta), comprimento(direccao));
            if la <= 0.0 || ld <= 0.0 {
                continue;
            }
            let c = (f64::from(aresta[0]) * f64::from(direccao[0])
                + f64::from(aresta[1]) * f64::from(direccao[1])
                + f64::from(aresta[2]) * f64::from(direccao[2]))
                / (la * ld);
            // ⭐ `cos 4α` a partir de `cos α` (Chebyshev), que é o que dispensa
            // a base tangente — ver o cabeçalho.
            let c2 = c * c;
            soma += 8.0 * c2 * c2 - 8.0 * c2 + 1.0;
            n += 1;
        }
    }
    (soma / n.max(1) as f64, n)
}

/// ⭐⭐⭐ **A TERCEIRA COLUNA — a que o OLHO lê: que FRACÇÃO das arestas da faixa
/// corre com a grade do traço.**
///
/// Devolve as contagens em três faixas de `15°` do desvio à grade mais próxima
/// (`0°` = ao longo do traço **ou** exactamente atravessada) e o total.
///
/// # ⛔⛔⛔ Ela existe porque o `Q` é uma MÉDIA e o dono reprovou uma média
///
/// Report de 2026-09-18 (*«pouca ou nenhuma diferença»*, com foto do arame): o
/// [`q_da_faixa`] subia `+0,09` e as duas imagens eram **indistinguíveis**. A
/// razão é estrutural — *um `Q` de `+0,09` pode ser meia dúzia de arestas
/// perfeitamente alinhadas no meio de milhares que não mudaram*, e o olho conta
/// arestas em vez de as integrar.
///
/// ⭐ **O ZERO desta régua não é `0 %`, é `33 %`:** o desvio à grade de uma
/// direcção qualquer é uniforme em `[0°, 45°]`, logo uma malha **sem direcção
/// nenhuma** enche as três faixas por igual. Medido na bola da cena `=49` com o
/// pente desligado: `32,2 %`–`34,6 %` nos quatro rumos.
///
/// ⚠️ **A barra de quem a usa sai daí e nunca de `0`** — e o lado APROVADO
/// existe: a saída do próprio alvo sobre as fixturas de `rotacao/` lê `34,1 %`
/// desligado e **`43,6 %`** no tecto.
#[must_use]
pub fn grade_da_faixa(malha: &Mesh, percurso: &[[f32; 3]], raio: f32) -> ([usize; 3], usize) {
    let pos = malha.positions();
    let mut vistas = std::collections::BTreeSet::new();
    let (mut bins, mut n) = ([0usize; 3], 0usize);
    for f in malha.faces() {
        let vs = f.verts();
        for k in 0..vs.len() {
            let (a, b) = (vs[k], vs[(k + 1) % vs.len()]);
            if !vistas.insert((a.min(b), a.max(b))) {
                continue;
            }
            let (pa, pb) = (pos[a as usize], pos[b as usize]);
            let meio = [
                (pa[0] + pb[0]) * 0.5,
                (pa[1] + pb[1]) * 0.5,
                (pa[2] + pb[2]) * 0.5,
            ];
            let Some(direccao) = direccao_do_troco(percurso, meio, raio) else {
                continue;
            };
            let aresta = sub(pb, pa);
            let (la, ld) = (comprimento(aresta), comprimento(direccao));
            if la <= 0.0 || ld <= 0.0 {
                continue;
            }
            let c = (f64::from(aresta[0]) * f64::from(direccao[0])
                + f64::from(aresta[1]) * f64::from(direccao[1])
                + f64::from(aresta[2]) * f64::from(direccao[2]))
                / (la * ld);
            let angulo = c.clamp(-1.0, 1.0).acos().to_degrees() % 90.0;
            // A DOBRA para `[0, 45]`: uma aresta não tem sentido e as duas
            // famílias da grade (ao longo e atravessada) são a mesma coisa.
            let desvio = angulo.min(90.0 - angulo);
            n += 1;
            bins[((desvio / 15.0) as usize).min(2)] += 1;
        }
    }
    (bins, n)
}

/// ⭐⭐⭐ **A QUARTA COLUNA — o VINCO: o ângulo entre as normais de duas faces
/// vizinhas, em graus.** Devolve `(p50, p90, max, quantas arestas entraram)`.
///
/// ⛔⛔⛔ **Ela nasceu do report de 2026-09-19** (*«o resultado fica pior que o
/// original, com irregularidade a 90 graus da direcção do movimento»*, com foto
/// do RELEVO — não do arame). As três colunas anteriores medem a **ligação**
/// (que direcção as arestas tomam, que forma os triângulos têm) e **nenhuma**
/// mede o que a luz lê. *Uma malha pode ficar mais alinhada e mais feia ao mesmo
/// tempo, e até aqui esta linha não tinha como o dizer.*
///
/// ⚠️⚠️ **Ela é a régua certa e a [`rugosidade`] não era:** `|p − centroide do
/// anel|` muda quando a LIGAÇÃO muda, e uma troca de diagonal muda o anel **sem
/// mover um vértice** — o número sobe sem a superfície se mexer. O ângulo entre
/// normais também muda com a ligação, mas é **exactamente isso que o
/// sombreamento faz**: a face é o que a luz vê.
///
/// ⚠️ **O CONTROLO é obrigatório:** numa esfera lisa esta régua lê o facetado da
/// própria malha (`p50 ≈ 1,4°` na peça da `=49`), logo o que se julga é a
/// distância ao valor por pentear, nunca o valor.
#[must_use]
pub fn vinco_da_faixa(malha: &Mesh, percurso: &[[f32; 3]], raio: f32) -> (f64, f64, f64, usize) {
    let pos = malha.positions();
    let normal = |f: &ph2d_mesh::Face| {
        let v = f.verts();
        let (a, b, c) = (pos[v[0] as usize], pos[v[1] as usize], pos[v[2] as usize]);
        let n = cruz(sub(b, a), sub(c, a));
        let l = comprimento(n);
        (l > 0.0).then(|| [n[0] / l as f32, n[1] / l as f32, n[2] / l as f32])
    };
    let mut por_aresta: std::collections::BTreeMap<(u32, u32), Vec<usize>> =
        std::collections::BTreeMap::new();
    for (i, f) in malha.faces().iter().enumerate() {
        let vs = f.verts();
        if vs.len() != 3 {
            continue;
        }
        for k in 0..3 {
            let (a, b) = (vs[k], vs[(k + 1) % 3]);
            por_aresta.entry((a.min(b), a.max(b))).or_default().push(i);
        }
    }
    let mut vals: Vec<f64> = Vec::new();
    for ((a, b), faces) in &por_aresta {
        // ⛔ Uma aresta com uma face é BEIRA e com três é não-variedade: nem uma
        // nem outra tem um ângulo entre DUAS normais.
        if faces.len() != 2 {
            continue;
        }
        let (pa, pb) = (pos[*a as usize], pos[*b as usize]);
        let meio = [
            (pa[0] + pb[0]) * 0.5,
            (pa[1] + pb[1]) * 0.5,
            (pa[2] + pb[2]) * 0.5,
        ];
        if direccao_do_troco(percurso, meio, raio).is_none() {
            continue;
        }
        let (Some(n0), Some(n1)) = (
            normal(&malha.faces()[faces[0]]),
            normal(&malha.faces()[faces[1]]),
        ) else {
            continue;
        };
        let c = f64::from(n0[0] * n1[0] + n0[1] * n1[1] + n0[2] * n1[2]).clamp(-1.0, 1.0);
        vals.push(c.acos().to_degrees());
    }
    vals.sort_by(|x, y| x.partial_cmp(y).expect("sem NaN"));
    let n = vals.len();
    if n == 0 {
        return (0.0, 0.0, 0.0, 0);
    }
    (vals[n / 2], vals[(n * 9 / 10).min(n - 1)], vals[n - 1], n)
}

fn cruz(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// **A SEGUNDA COLUNA — o pior canto de triângulo da faixa, em graus.**
///
/// Devolve `(pior, quantos triângulos entraram)`. ⚠️ O par existe pela mesma
/// razão do da primeira: uma faixa vazia devolveria `180°`, que se lê como *«a
/// malha está perfeita»*.
///
/// ⚠️ **Os quads são saltados de propósito:** o que esta coluna caça é a LASCA,
/// e uma lasca é um triângulo. Um quad quase degenerado é outro defeito, com
/// outra régua (o aspecto), e misturá-los deixaria as duas sem barra.
#[must_use]
pub fn pior_angulo(malha: &Mesh, percurso: &[[f32; 3]], raio: f32) -> (f64, usize) {
    let pos = malha.positions();
    let (mut pior, mut n) = (180.0f64, 0usize);
    for f in malha.faces() {
        let vs = f.verts();
        if vs.len() != 3 {
            continue;
        }
        let p: Vec<[f32; 3]> = vs.iter().map(|&v| pos[v as usize]).collect();
        let centro = [
            (p[0][0] + p[1][0] + p[2][0]) / 3.0,
            (p[0][1] + p[1][1] + p[2][1]) / 3.0,
            (p[0][2] + p[1][2] + p[2][2]) / 3.0,
        ];
        if direccao_do_troco(percurso, centro, raio).is_none() {
            continue;
        }
        n += 1;
        for k in 0..3 {
            let (a, b, c) = (p[k], p[(k + 1) % 3], p[(k + 2) % 3]);
            let (u, w) = (sub(b, a), sub(c, a));
            let (lu, lw) = (comprimento(u), comprimento(w));
            if lu <= 0.0 || lw <= 0.0 {
                continue;
            }
            let cos = ((f64::from(u[0]) * f64::from(w[0])
                + f64::from(u[1]) * f64::from(w[1])
                + f64::from(u[2]) * f64::from(w[2]))
                / (lu * lw))
                .clamp(-1.0, 1.0);
            pior = pior.min(cos.acos().to_degrees());
        }
    }
    (pior, n)
}

/// **QUANTAS lascas a faixa tem** — `(abaixo de `grau`, total de triângulos)`.
///
/// # ⛔⛔ Porque ela existe AO LADO do [`pior_angulo`], e não no lugar dele
///
/// Medido nesta linha, sobre uma bola de malha regular riscada a **exactamente
/// 45°** da grade: o pior ângulo da faixa cai de `12,03°` para `1,13°` com o
/// pente no tecto — e a contagem diz que são **`3` triângulos em `3 356`**. A
/// `30°` e a `60°` da mesma grade a contagem é **ZERO** em todo o curso.
///
/// ⇒ *o mínimo sozinho não separa «três vértices na linha de água» de «um campo
/// de estrias»*, e as duas coisas pedem respostas opostas: a primeira é uma
/// fronteira a declarar, a segunda é uma lei a refazer. ⚠️ **E o mínimo continua
/// a ser a régua da bancada**, onde ele separou `4,56°` da lei de hoje de
/// `0,31°` da refutada — ali o campo inteiro colapsava.
///
/// ⚠️ **O mecanismo dos 45°:** o alvo do pente tem **quatro** dobras, logo as
/// duas direcções de uma grade valem exactamente o mesmo — a meio caminho entre
/// elas a lei não tem para que lado virar, e um punhado de vértices fica na
/// linha de água.
#[must_use]
pub fn lascas(malha: &Mesh, percurso: &[[f32; 3]], raio: f32, grau: f64) -> (usize, usize) {
    let pos = malha.positions();
    let (mut finos, mut total) = (0usize, 0usize);
    for f in malha.faces() {
        let vs = f.verts();
        if vs.len() != 3 {
            continue;
        }
        let p: Vec<[f32; 3]> = vs.iter().map(|&v| pos[v as usize]).collect();
        let centro = [
            (p[0][0] + p[1][0] + p[2][0]) / 3.0,
            (p[0][1] + p[1][1] + p[2][1]) / 3.0,
            (p[0][2] + p[1][2] + p[2][2]) / 3.0,
        ];
        if direccao_do_troco(percurso, centro, raio).is_none() {
            continue;
        }
        total += 1;
        let mut pior = 180.0f64;
        for k in 0..3 {
            let (a, b, c) = (p[k], p[(k + 1) % 3], p[(k + 2) % 3]);
            let (u, w) = (sub(b, a), sub(c, a));
            let (lu, lw) = (comprimento(u), comprimento(w));
            if lu <= 0.0 || lw <= 0.0 {
                continue;
            }
            let cos = ((f64::from(u[0]) * f64::from(w[0])
                + f64::from(u[1]) * f64::from(w[1])
                + f64::from(u[2]) * f64::from(w[2]))
                / (lu * lw))
                .clamp(-1.0, 1.0);
            pior = pior.min(cos.acos().to_degrees());
        }
        if pior < grau {
            finos += 1;
        }
    }
    (finos, total)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐⭐⭐ **GATE — sobre uma peça PLANA a régua 3D devolve o MESMO número que
    /// a de duas coordenadas.**
    ///
    /// ⛔⛔ **É o que torna honesta a tabela do [`crate::Brush::pente`]:** ela foi
    /// medida com a versão plana e re-medida com esta, e a afirmação escrita ao
    /// lado dela é *«a forma velha é um caso particular da nova, não uma
    /// aproximação dela»*. ⚠️ Sem este gate a afirmação é prosa — e uma mutação
    /// que zerasse o termo `z` desta porta passava por **toda** a suíte desta
    /// linha, incluindo o gate da cena de smoke, que é curva.
    ///
    /// ⚠️ **A fixtura é estritamente PLANA de propósito** (`z = 0` em toda a
    /// parte): é aí — e só aí — que as duas formas têm de coincidir. Sobre uma
    /// peça curva elas divergem *por construção*, e é essa divergência que a
    /// porta existe para trazer.
    #[test]
    fn sobre_uma_peca_plana_a_regua_3d_e_a_de_duas_coordenadas() {
        let (malha, percurso) = chapa_e_percurso();
        let (q3, n3) = q_da_faixa(&malha, &percurso, 0.30);
        let (q2, n2) = q_plana(&malha, &percurso, 0.30);
        assert!(
            n3 > 200,
            "a faixa tem {n3} arestas — a fixtura deixou de conter o fenomeno"
        );
        assert_eq!(n3, n2, "as duas formas escolheram faixas DIFERENTES");
        // ⚠️ **A barra é `1e-6` e não zero, e o número é MEDIDO: `2,4e-8`.** As
        // duas formas são algebricamente a mesma e numericamente não: uma passa
        // por `atan2` e `cos` (transcendentes) e a outra é um polinómio de grau
        // quatro, sobre entradas que nasceram em `f32`. ⛔ Uma barra em `1e-12`
        // reprovava sobre produto correcto — e esta separa por **quatro ordens
        // de grandeza** do que uma fórmula DIFERENTE custa (a mutação que zera
        // o termo `z` move o `Q` em `~1e-2`).
        assert!(
            (q3 - q2).abs() < 1e-6,
            "sobre uma peca plana a regua 3D le' {q3:+.12} e a de duas \
             coordenadas {q2:+.12} (desvio {:.3e}, medido 2,4e-8) — a tabela do \
             `Brush::pente` foi re-medida a afirmar que elas coincidem aqui",
            (q3 - q2).abs()
        );
    }

    /// ⭐⭐⭐ **GATE — sobre uma peça CURVA as duas formas DISCORDAM**, e é essa
    /// discordância que a porta existe para trazer.
    ///
    /// ⛔⛔ **Ele é o CONTROLO do gate acima, e sem ele o termo `z` desta porta
    /// não tem régua nenhuma:** medido, zerá-lo deixa o gate da redução verde
    /// **por construção** (a fixtura dele é estritamente plana) *e* o gate da
    /// cena de smoke verde (as barras dele são grosseiras de propósito). *Uma
    /// mutação que sobrevive a tudo não prova que a linha é inútil — prova que
    /// ninguém a mede.*
    #[test]
    fn sobre_uma_peca_curva_as_duas_formas_discordam() {
        let mut malha = ph2d_mesh::shapes::sphere_with_triangles(12_000, 1.0);
        malha.triangulate();
        // ⚠️ **O traço DÁ A VOLTA à bola** (`±1,4 rad`) e vai no meridiano: é aí
        // que a direcção dele tem o maior `z`, e é o `z` da direcção que a forma
        // plana deita fora. Medido (`diag_onde_as_formas_discordam`), o desvio
        // varre de `0,002` a **`0,309`** conforme a fixtura — *uma fixtura que
        // não contém o fenómeno leria `0,002` e deixaria a mutação passar*.
        let percurso: Vec<[f32; 3]> = (0..24)
            .map(|k| {
                let u = -1.4 + (2.8 / 23.0) * k as f32;
                [0.0, u.sin(), u.cos()]
            })
            .collect();
        let (q3, n3) = q_da_faixa(&malha, &percurso, 0.35);
        let (q2, n2) = q_plana(&malha, &percurso, 0.35);
        assert!(n3 > 500 && n2 > 500, "a faixa tem {n3}/{n2} arestas");
        // A barra está a METADE do desvio medido; o resíduo numérico do gate
        // irmão é `2,4e-8`, **sete ordens de grandeza** abaixo dela.
        let d = (q3 - q2).abs();
        assert!(
            d > 0.15,
            "sobre uma bola a regua 3D le' {q3:+.4} e a plana {q2:+.4} (desvio \
             {d:.4}, medido 0,3088) — se elas concordam aqui, o termo `z` desta \
             porta e' inerte e a regua voltou a ser plana"
        );
    }

    /// ⭐⭐⭐ **GATE — o `Q` de um triângulo cujo valor se calcula À MÃO.**
    ///
    /// # ⛔⛔ Ele existe porque uma MUTAÇÃO sobreviveu aos outros dois
    ///
    /// Zerar o termo `z` do produto escalar desta porta passava **por tudo**: o
    /// gate da redução tem fixtura estritamente plana (ali o termo é zero **por
    /// construção**), o da divergência compara duas réguas e a mutação afasta-as
    /// em vez de as juntar, e o da cena de smoke tem barras grosseiras de
    /// propósito. ⇒ *nenhum deles pergunta se o número está CERTO* — só se duas
    /// réguas concordam, ou discordam, ou se o produto se move.
    ///
    /// # A conta, que cabe em três linhas
    ///
    /// Um triângulo `A(0,0,0) · B(0,0,1) · C(1,0,0)` e um traço ao longo de `+z`:
    ///
    /// | aresta | ângulo com o traço | `cos 4α` |
    /// |---|---|---|
    /// | `AB = (0,0,1)` | `0°` | `+1` |
    /// | `BC = (1,0,−1)` | `135°` | `−1` |
    /// | `CA = (−1,0,0)` | `90°` | `+1` |
    ///
    /// ⇒ `Q = (1 − 1 + 1) / 3 = 1/3`, **exacto**. ⚠️ E o traço vai ao longo de
    /// `z` de propósito: é o eixo que uma régua plana deita fora, logo com ele a
    /// mutação lê `Q = 1` (todas as arestas a `0°` de uma direcção nula) e o
    /// desvio é de **dois terços**.
    #[test]
    fn o_q_de_um_triangulo_conhecido_e_um_terco() {
        let malha = Mesh::from_parts(
            vec![[0.0, 0.0, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]],
            vec![ph2d_mesh::Face::tri(0, 1, 2)],
        )
        .expect("o triangulo e' uma malha valida");
        // Um raio largo para que as três arestas caiam na faixa: o que este gate
        // mede é a CONTA, não a escolha da faixa (essa tem gate próprio).
        let percurso = vec![[0.0, 0.0, -1.0], [0.0, 0.0, 1.0]];
        let (q, n) = q_da_faixa(&malha, &percurso, 10.0);
        assert_eq!(n, 3, "as tres arestas tinham de entrar na faixa");
        assert!(
            (q - 1.0 / 3.0).abs() < 1e-6,
            "o Q deste triangulo le' {q:+.9} e a conta da' exactamente 1/3 \
             (+0,333333333) — a regua deixou de olhar para o eixo `z`, e com ele \
             fora ela le' +1"
        );
    }

    /// A régua ANTIGA, palavra por palavra: `atan2` do produto vectorial 2D
    /// sobre o escalar 2D. Ela vive **só aqui**, como oráculo.
    fn q_plana(malha: &Mesh, percurso: &[[f32; 3]], raio: f32) -> (f64, usize) {
        let pos = malha.positions();
        let mut vistas = std::collections::BTreeSet::new();
        let (mut soma, mut n) = (0.0f64, 0usize);
        for f in malha.faces() {
            let vs = f.verts();
            for k in 0..vs.len() {
                let (a, b) = (vs[k], vs[(k + 1) % vs.len()]);
                if !vistas.insert((a.min(b), a.max(b))) {
                    continue;
                }
                let (pa, pb) = (pos[a as usize], pos[b as usize]);
                let meio = [(pa[0] + pb[0]) * 0.5, (pa[1] + pb[1]) * 0.5];
                let mut melhor = f32::INFINITY;
                let mut direccao = [1.0f32, 0.0];
                for par in percurso.windows(2) {
                    let d = (meio[0] - par[1][0]).hypot(meio[1] - par[1][1]);
                    if d < melhor {
                        melhor = d;
                        direccao = [par[1][0] - par[0][0], par[1][1] - par[0][1]];
                    }
                }
                if melhor > raio * FAIXA {
                    continue;
                }
                let aresta = [pb[0] - pa[0], pb[1] - pa[1]];
                let c = f64::from(aresta[0] * direccao[0] + aresta[1] * direccao[1]);
                let s = f64::from(aresta[0] * direccao[1] - aresta[1] * direccao[0]);
                if c == 0.0 && s == 0.0 {
                    continue;
                }
                soma += (4.0 * s.atan2(c)).cos();
                n += 1;
            }
        }
        (soma / n.max(1) as f64, n)
    }

    /// Uma chapa sacudida em `z = 0` e um percurso recto sobre ela.
    ///
    /// ⚠️ **O interior é sacudido** porque uma grelha certinha lê `Q = +0,29` —
    /// seis vezes a barra — antes de alguém lhe tocar, e as duas formas
    /// coincidiriam sobre um número que não mede nada.
    fn chapa_e_percurso() -> (Mesh, Vec<[f32; 3]>) {
        let n = 41usize;
        let lado = 3.0f32;
        let passo = lado / (n - 1) as f32;
        let mut pos = Vec::with_capacity(n * n);
        for j in 0..n {
            for i in 0..n {
                let borda = i == 0 || j == 0 || i == n - 1 || j == n - 1;
                // Sacudidela determinística, como a da bancada.
                let h = ((i * 73_856_093) ^ (j * 19_349_663)) as f32;
                let d = if borda {
                    0.0
                } else {
                    (h % 1000.0) / 1000.0 - 0.5
                };
                pos.push([
                    -lado * 0.5 + i as f32 * passo + d * passo * 0.6,
                    -lado * 0.5 + j as f32 * passo + (d * 7.0).fract() * passo * 0.6,
                    0.0,
                ]);
            }
        }
        let mut faces = Vec::new();
        for j in 0..n - 1 {
            for i in 0..n - 1 {
                let (a, b) = ((j * n + i) as u32, (j * n + i + 1) as u32);
                let (c, d) = (((j + 1) * n + i + 1) as u32, ((j + 1) * n + i) as u32);
                faces.push(ph2d_mesh::Face::tri(a, b, c));
                faces.push(ph2d_mesh::Face::tri(a, c, d));
            }
        }
        let malha = Mesh::from_parts(pos, faces).expect("a chapa e' uma malha valida");
        let percurso = (0..24).map(|k| [-1.2 + 0.1 * k as f32, 0.0, 0.0]).collect();
        (malha, percurso)
    }

    /// SONDA — onde as duas formas mais discordam.
    #[test]
    #[ignore = "sonda"]
    fn diag_onde_as_formas_discordam() {
        for tri in [false, true] {
            for (nome, e) in [
                ("x", [1.0f32, 0.0]),
                ("y", [0.0, 1.0]),
                ("45", [0.707, 0.707]),
            ] {
                for amp in [0.55f32, 1.0, 1.4] {
                    let mut malha = ph2d_mesh::shapes::sphere_with_triangles(12_000, 1.0);
                    if tri {
                        malha.triangulate();
                    }
                    let percurso: Vec<[f32; 3]> = (0..24)
                        .map(|k| {
                            let u = -amp + (2.0 * amp / 23.0) * k as f32;
                            [u.sin() * e[0], u.sin() * e[1], u.cos()]
                        })
                        .collect();
                    let (q3, n) = q_da_faixa(&malha, &percurso, 0.35);
                    let (q2, _) = q_plana(&malha, &percurso, 0.35);
                    eprintln!(
                        "tri={tri:<5} rumo {nome:<3} amp {amp:.2}  3D={q3:+.4} plana={q2:+.4} \
                         desvio={:.4} (n={n})",
                        (q3 - q2).abs()
                    );
                }
            }
        }
    }
}

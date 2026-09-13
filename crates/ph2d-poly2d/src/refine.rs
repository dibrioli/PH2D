//! ⭐⭐⭐ **A MALHA REFINA-SE NA HORA DE DESENHAR, contra uma TOLERÂNCIA EM PIXELS.**
//!
//! # O report que a trouxe (dono, 2026-09-10, com foto e três setas)
//!
//! > *«malha bem desenhada. Contudo não é a solução perfeita em termos de deformação pois ao dobrar
//! > a articulação temos arestas retas na imagem. Estude um algoritmo com opção de um tipo de smooth
//! > na imagem e coloque como alternativa»*
//!
//! # ⭐⭐⭐ A lei, e a razão de ela existir
//!
//! **A malha NÃO é a deformação — ela é uma AMOSTRAGEM dela.** O campo
//! `Φ(p) = Σ wᵢ(p)·Mᵢ·p` está definido em **todo** ponto da imagem, porque os pesos são
//! **derivados** e não guardados. O que produz a aresta reta é o DESENHO: cada triângulo é pintado
//! com **um afim**, que é a aproximação de 1.ª ordem de um campo curvo.
//!
//! ⇒ O desvio entre a silhueta desenhada e o campo verdadeiro é **`O(h²)`**, e MEDIU-SE
//! (cápsula do smoke, cadeia de 3 ossos, silhueta analítica de 2000 pontos):
//!
//! | grelha | triângulos | desvio p99 a 60° | a 105° | a 150° |
//! |---|---:|---:|---:|---:|
//! | a de hoje | `216` | `3,85 px` | `6,85 px` | **`9,84 px`** |
//! | 2× mais fina | `768` | `1,09` | `1,98` | `2,79` |
//! | 4× mais fina | `3 086` | `0,29` | `0,54` | **`0,78`** |
//! | 8× mais fina | `11 472` | `0,07` | `0,13` | `0,19` |
//!
//! ⛔⛔ **E o erro NÃO mora nas articulações**, que era a premissa da grelha graduada: com as juntas
//! em `x = 0 · 107 · 213 · 320`, o pior desvio cai em **`x = 119 · 190 · 266`**. Apertar a banda à
//! volta das juntas leva `3,99 px` a `3,39 px` e mais nada — *a densidade que a DOBRA precisa e a
//! densidade que o DESENHO precisa não estão no mesmo sítio.* O erro vive onde os **pesos** variam
//! depressa, e com `raio = comprimento do osso` isso é o membro inteiro.
//!
//! # ⚠️ Porque isto é feito a DESENHAR e não a PRENDER
//!
//! No instante do *bind* a pose é a de repouso: **não há dobra nenhuma**, logo não há erro para
//! medir. A densidade necessária é função da POSE, que só existe no quadro. *Uma malha escolhida no
//! bind é escolhida antes de a pergunta ser feita.* ⭐ E a malha guardada fica pequena — o save e o
//! undo continuam a fotografar a mesma coisa.
//!
//! # ⚠️ A tolerância é em pixels de ECRÃ, e é por isso que ela precisa da câmara
//!
//! Um desvio de meia unidade local é meio pixel a zoom `1` e **quatro** pixels a zoom `8`. *A
//! suavidade que o olho vê é um facto de espaço de ECRÃ*, então quem chama converte a tolerância
//! pela escala da câmara antes de entrar aqui.
//!
//! # ⭐⭐⭐ A conformidade é EXACTA, e não uma tolerância
//!
//! O refinamento é **uniforme, com o mesmo `k` para toda a malha**, e cada ponto novo é nomeado
//! pela **aresta canónica** que o gera (`o vértice de índice menor primeiro`) — logo os dois
//! triângulos que partilham uma aresta calculam o ponto do meio dela a partir da **mesma expressão,
//! na mesma ordem**, e obtêm os **mesmos bits**. ⛔ Um `k` por triângulo abriria **nós pendurados**,
//! que é a fenda que a `grid.rs` recusa por escrito — e aqui seria pior, porque cada peça é um
//! recorte independente e a fenda é um fio de fundo a atravessar a arte.

use crate::Mesh2d;

/// ⭐ **Os números do refinamento.**
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RefineOptions {
    /// Quanto a silhueta desenhada pode afastar-se do campo verdadeiro, **em pixels de ecrã**.
    pub tolerance_px: f64,
    /// ⛔⛔⛔ **O TECTO É DO NÚMERO DE PEÇAS, e não do `k`** — e a diferença foi um report do dono
    /// (*«Smooth bugado quebrando a forma»*, 2026-09-10, com foto).
    ///
    /// ⛔⛔ **A 1.ª redacção deste doc atribuía o limite à CAMADA DE RECORTE do Vello, e a medição
    /// de 2026-09-13 refutou-a:** o que partia a imagem era o ATLAS — a pele desenhava cada peça
    /// pela porta crua, que guardava uma cópia inteira da imagem por peça (`7 776` peças perdiam
    /// `5 651`). Curado isso, o recurso que sobra é o buffer FIXO de informação por desenho do
    /// Vello, que é do QUADRO inteiro — e quem conhece o quadro é o chamador: o produto passa aqui a
    /// PARTE desta malha do orçamento do quadro (`ph2d_skeleton_live::skin_image::SKIN_FRAME_PIECES`).
    /// Um tecto no `k` não é um tecto nesse recurso: ele é **quadrático** nele, e a malha de partida
    /// pode ter qualquer tamanho.
    ///
    /// ⚠️⚠️ **A experiência que o dono correu sem saber é a prova:** com o braço quase RECTO (`2°`)
    /// o desvio já é `0,499 px`, logo o `k` saltava para o tecto e desenhava **7 776** recortes —
    /// *a mesma geometria que o `Fast` desenha em 216, e partida*. A malha estava provadamente
    /// correcta: área conservada ao cêntimo, zero triângulos saltados, zero arestas com mais de
    /// dois donos.
    ///
    /// ⭐ **E o custo está MEDIDO dos dois lados** (2026-09-13, W4 do plano `docs/Skeleton/03`;
    /// `load 3,7`–`3,9`, o MÍNIMO de 40/60 corridas — acima de `load ~5` uma leitura de relógio
    /// desta workstation não vale nada). Por peça ENTREGUE, num quadro com `Smooth`: **`1,08 µs`**
    /// (descodificar a malha `0,134` · deformar e montar `0,200` no `Fast` · recolher, costurar,
    /// enviar e desenhar `0,039`). ⇒ o tecto do produto é uma FATIA do quadro dividida por esse
    /// número: `ph2d_skeleton_live::skin_image::SKIN_FRAME_PIECES`.
    ///
    /// ⛔ A tabela anterior (`deformar`/`encodar`, até `10`–`16 %` a `7 776` peças) media o caminho
    /// do **Vello**, que a W2 retirou.
    pub max_pieces: usize,
}

impl Default for RefineOptions {
    fn default() -> Self {
        Self {
            // Meio pixel: abaixo disto o anti-aliasing da própria arte é mais largo que o erro.
            tolerance_px: 0.5,
            // ⚠️ **O neutro desta folha, NÃO o tecto do produto:** a folha não sabe o que é um
            // quadro nem um renderer. O produto substitui-o pela parte da malha no orçamento do
            // QUADRO (`ph2d_skeleton_live::skin_image::SKIN_FRAME_PIECES`, hoje derivado do TEMPO
            // do quadro com o custo por peça MEDIDO — W4 do plano `docs/Skeleton/03`).
            max_pieces: 1024,
        }
    }
}

/// Quem é um vértice da malha refinada — a chave que faz dois triângulos vizinhos concordarem.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum No {
    /// Um vértice da malha original.
    Canto(u32),
    /// O passo `s` de `k` sobre a aresta `(menor, maior)`, contado **a partir do menor**.
    Aresta(u32, u32, u32),
    /// Um ponto do miolo do triângulo `t`, na grelha baricêntrica `(i, j)`.
    Miolo(u32, u32, u32),
}

/// **Quanto o afim de cada triângulo erra o campo** — o maior desvio nos meios das arestas.
///
/// ⚠️ Os meios das arestas são o sítio certo para perguntar: é onde um interpolador linear erra
/// mais, e é onde o refinamento vai pôr o primeiro ponto novo.
#[must_use]
pub fn deviation(
    mesh: &Mesh2d,
    posed: &[[f64; 2]],
    deform: &mut dyn FnMut([f64; 2]) -> [f64; 2],
) -> f64 {
    let mut pior = 0.0_f64;
    for t in &mesh.tris {
        for k in 0..3 {
            let (i, j) = (t[k] as usize, t[(k + 1) % 3] as usize);
            let (Some(&ra), Some(&rb)) = (mesh.rest.get(i), mesh.rest.get(j)) else {
                continue;
            };
            let (Some(&pa), Some(&pb)) = (posed.get(i), posed.get(j)) else {
                continue;
            };
            let meio = deform([(ra[0] + rb[0]) / 2.0, (ra[1] + rb[1]) / 2.0]);
            let reta = [(pa[0] + pb[0]) / 2.0, (pa[1] + pb[1]) / 2.0];
            pior = pior.max((meio[0] - reta[0]).hypot(meio[1] - reta[1]));
        }
    }
    pior
}

/// ⭐ **Quantas vezes partir cada aresta** para o desvio caber na tolerância.
///
/// ⚠️ **A raiz quadrada é a lei `O(h²)`, medida** (a tabela no cabeçalho): partir ao meio divide o
/// desvio por `~3,6`, não por `2`. Um `k` linear no desvio pediria triângulos a mais.
#[must_use]
pub fn splits_for(desvio: f64, pecas: usize, opts: RefineOptions) -> u32 {
    let tol = opts.tolerance_px.max(f64::MIN_POSITIVE);
    if !desvio.is_finite() || desvio <= tol {
        return 1;
    }
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a raiz de uma razão finita e positiva, limitada logo a seguir pelo orçamento"
    )]
    let k = (desvio / tol).sqrt().ceil() as u32;
    k.clamp(1, max_split(pecas, opts))
}

/// ⭐ **Quantas partes o ORÇAMENTO ainda paga** — `k` tal que `peças · k² <= max_pieces`.
///
/// ⚠️ É aqui que o tecto deixa de ser um número escolhido e passa a ser uma **divisão**: uma malha
/// de `216` triângulos com orçamento `1024` pode ir a `k = 2`; uma de `50` pode ir a `k = 4`. *Um
/// tecto no `k` daria à segunda o mesmo direito que à primeira, e é a CONTAGEM que o renderer paga.*
#[must_use]
pub fn max_split(pecas: usize, opts: RefineOptions) -> u32 {
    if pecas == 0 {
        return 1;
    }
    #[expect(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "contagens de triângulos de uma malha de imagem, muito abaixo do limite exacto do f64"
    )]
    let k = ((opts.max_pieces as f64) / (pecas as f64)).sqrt() as u32;
    k.max(1)
}

/// ⭐⭐⭐ **A MALHA POSADA, refinada até à tolerância** — devolve `(malha, posições)`.
///
/// `deform` é o campo: ele leva um ponto de **repouso** (pixels da imagem) para onde ele está
/// **agora**. Ele é chamado uma vez por vértice novo, e ⛔ nunca duas vezes para o mesmo ponto —
/// é a chave [`No`] que o garante.
///
/// Com `k == 1` devolve a malha original e as posições dela, **ao bit**: é o caminho de omissão, e
/// ele tem de ser byte-idêntico ao que o desenho já fazia.
#[must_use]
pub fn refine_posed(
    mesh: &Mesh2d,
    deform: &mut dyn FnMut([f64; 2]) -> [f64; 2],
    opts: RefineOptions,
) -> (Mesh2d, Vec<[f64; 2]>, u32) {
    let posed: Vec<[f64; 2]> = mesh.rest.iter().map(|&p| deform(p)).collect();
    let tecto = max_split(mesh.tris.len(), opts);
    let k = splits_for(deviation(mesh, &posed, deform), mesh.tris.len(), opts);
    if k <= 1 {
        return (mesh.clone(), posed, 1);
    }
    let (r, p) = build(mesh, deform, k);
    // ⭐⭐⭐ **O ESTIMADOR CONFERE O QUE ENTREGOU, e corrige UMA vez.**
    //
    // ⚠️⚠️ **Medido, e foi um gate vermelho que o exigiu:** a lei `O(h²)` descreve a tendência, não
    // o valor — com a tolerância a pedir `0,954 px` o `k` de um só passo entregava `0,980`. *Um
    // número que se chama tolerância e não é honrado é um número que mente ao artista.*
    //
    // ⚠️ **UMA correcção, nunca um laço até convergir:** a segunda estimativa parte da medição já
    // feita **na malha refinada** (`k · √(d/tol)`), que é a lei aplicada onde ela vale — e um laço
    // seria trabalho por quadro sem tecto, exactamente o que o orçamento existe para impedir.
    let d = deviation(&r, &p, deform);
    if d <= opts.tolerance_px || k >= tecto {
        return (r, p, k);
    }
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a razão é finita e positiva, e o clamp logo abaixo é o tecto"
    )]
    let k2 = ((f64::from(k) * (d / opts.tolerance_px.max(f64::MIN_POSITIVE)).sqrt()).ceil() as u32)
        .clamp(k + 1, tecto.max(k + 1));
    let (r2, p2) = build(mesh, deform, k2);
    (r2, p2, k2)
}

/// A grelha baricêntrica de `k` partes por aresta, com os pontos das arestas **partilhados**.
fn build(
    mesh: &Mesh2d,
    deform: &mut dyn FnMut([f64; 2]) -> [f64; 2],
    k: u32,
) -> (Mesh2d, Vec<[f64; 2]>) {
    let kf = f64::from(k);
    let mut indice: std::collections::BTreeMap<No, u32> = std::collections::BTreeMap::new();
    let mut rest: Vec<[f64; 2]> = Vec::new();
    let mut agora: Vec<[f64; 2]> = Vec::new();
    let mut tris: Vec<[u32; 3]> = Vec::new();

    for (t_idx, t) in mesh.tris.iter().enumerate() {
        let (Some(&a), Some(&b), Some(&c)) = (
            mesh.rest.get(t[0] as usize),
            mesh.rest.get(t[1] as usize),
            mesh.rest.get(t[2] as usize),
        ) else {
            continue;
        };
        // A grelha baricêntrica `P(i,j) = A + (B−A)·i/k + (C−A)·j/k`, com `i + j <= k`.
        let mut no_de = |i: u32, j: u32| -> u32 {
            let chave = chave_do_no(t, k, i, j, t_idx);
            if let Some(&v) = indice.get(&chave) {
                return v;
            }
            let p = ponto_canonico(chave, [a, b, c], t, mesh, kf, i, j);
            #[expect(
                clippy::cast_possible_truncation,
                reason = "a malha refinada não passa de 2^32 nós: o orçamento de peças limita-a muito antes"
            )]
            let v = rest.len() as u32;
            rest.push(p);
            agora.push(deform(p));
            indice.insert(chave, v);
            v
        };
        for j in 0..k {
            for i in 0..(k - j) {
                let (p00, p10, p01) = (no_de(i, j), no_de(i + 1, j), no_de(i, j + 1));
                tris.push([p00, p10, p01]);
                if i + j + 1 < k {
                    let p11 = no_de(i + 1, j + 1);
                    tris.push([p10, p11, p01]);
                }
            }
        }
    }
    (
        Mesh2d {
            rest,
            tris,
            size: mesh.size,
        },
        agora,
    )
}

/// A chave canónica de um ponto da grelha baricêntrica.
fn chave_do_no(t: &[u32; 3], k: u32, i: u32, j: u32, t_idx: usize) -> No {
    let aresta = |u: u32, v: u32, s: u32| -> No {
        // ⭐ Sempre a partir do vértice de índice MENOR: os dois triângulos que partilham a aresta
        // chegam à mesma expressão, na mesma ordem, e logo aos mesmos bits.
        if u < v {
            No::Aresta(u, v, s)
        } else {
            No::Aresta(v, u, k - s)
        }
    };
    match (i, j) {
        (0, 0) => No::Canto(t[0]),
        _ if i == k => No::Canto(t[1]),
        _ if j == k => No::Canto(t[2]),
        (_, 0) => aresta(t[0], t[1], i),
        (0, _) => aresta(t[0], t[2], j),
        _ if i + j == k => aresta(t[1], t[2], j),
        #[expect(
            clippy::cast_possible_truncation,
            reason = "um índice de triângulo de uma malha de imagem cabe em u32 muito antes de a imagem caber em memória"
        )]
        _ => No::Miolo(t_idx as u32, i, j),
    }
}

/// A posição de repouso de um ponto, calculada **da chave** e nunca do triângulo em mãos.
fn ponto_canonico(
    chave: No,
    abc: [[f64; 2]; 3],
    t: &[u32; 3],
    mesh: &Mesh2d,
    kf: f64,
    i: u32,
    j: u32,
) -> [f64; 2] {
    match chave {
        No::Canto(v) => mesh.rest.get(v as usize).copied().unwrap_or(abc[0]),
        No::Aresta(u, v, s) => {
            let (Some(&pu), Some(&pv)) = (mesh.rest.get(u as usize), mesh.rest.get(v as usize))
            else {
                return abc[0];
            };
            let f = f64::from(s) / kf;
            [
                (pv[0] - pu[0]).mul_add(f, pu[0]),
                (pv[1] - pu[1]).mul_add(f, pu[1]),
            ]
        }
        No::Miolo(..) => {
            let (u, v) = (f64::from(i) / kf, f64::from(j) / kf);
            let _ = t;
            [
                (abc[2][0] - abc[0][0]).mul_add(v, (abc[1][0] - abc[0][0]).mul_add(u, abc[0][0])),
                (abc[2][1] - abc[0][1]).mul_add(v, (abc[1][1] - abc[0][1]).mul_add(u, abc[0][1])),
            ]
        }
    }
}

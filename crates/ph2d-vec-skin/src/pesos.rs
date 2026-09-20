//! ⭐⭐⭐ **O PADRÃO-OURO PARA UMA FORMA VECTORIAL** — a segunda mídia a deixar a lei euclidiana.
//!
//! # A dívida que isto paga
//!
//! Em 2026-09-15 os *Bounded Biharmonic Weights* entraram no produto **só para imagens**, porque
//! eles precisam de uma **malha do domínio** e uma Bézier não tem uma. ⛔ O preço foi um rig com
//! **duas leis**: a imagem deformava-se por uma e a forma vectorial por outra, e o sintoma seria *«o
//! braço desenhado não acompanha o braço vectorial»* — o mesmo defeito que o [`tendons_for`] do
//! esqueleto já nomeia por escrito, um nível acima.
//!
//! [`tendons_for`]: ph2d_skeleton
//!
//! # ⭐⭐ O domínio de um caminho é o INTERIOR dele, e a malha é a MESMA da imagem
//!
//! ```text
//! contornos FECHADOS ──▶ achatados em polígonos   (a fronteira, do documento)
//!                    ──▶ grelha graduada + orçamento   (ph2d_poly2d::grid_mesh_com)
//!                    ──▶ Bounded Biharmonic Weights    (ph2d_skin_weights)
//!                    ──▶ um peso por PONTO DE CONTROLO (baricêntrico na malha)
//! ```
//!
//! ⚠️ **A malha é a da imagem, e é de propósito:** a graduação pelas articulações, a conformidade, o
//! orçamento em triângulos e a renormalização são do GRID e não da mídia. *Uma segunda grelha
//! escrita para o vector divergiria desta no primeiro ajuste* — e a divergência seria exactamente o
//! defeito que esta wave existe para fechar.
//!
//! ⛔ **A malha NÃO é guardada nem desenhada.** Ela é um andaime do *bind*: o que sobrevive é um
//! peso por ponto de controlo. A forma continua a ser uma Bézier exacta e editável, que é a razão
//! de o vector usar LBS e não um envelope.
//!
//! # ⛔ Um caminho ABERTO não tem interior, e a resposta é *não sei*
//!
//! Uma linha de construção tem área zero: não há domínio, não há energia, não há pesos. Ela cai na
//! **lei derivada**, que é onde já estava — e a tabela vazia diz isso em voz alta, pela mesma porta
//! que uma imagem sem pesos usa. *Recusar é a resposta honesta; inventar um domínio para uma linha
//! seria inventar uma arte que o artista não desenhou.*

use ph2d_skin_weights::Handle;
use ph2d_vec_scene::VecPath;

/// Quantos pontos por segmento de cúbica ao achatar o contorno — o mesmo número que a
/// `ph2d_vec_scene::boundary` usa, para a fronteira do domínio ser a que o resto do app já enxerga.
const AMOSTRAS: usize = 16;

/// ⭐ **Quantos triângulos a malha do bind de um caminho tem.**
///
/// ⚠️ **Menos que os `3 000` de uma imagem, e o motivo é o que se mede sobre ela:** aqui os pesos
/// são amostrados em **pontos de controlo** (dezenas), não em cada vértice desenhado (milhares) —
/// a malha é um andaime que morre no fim do bind, e refiná-la só paga a solução, nunca o desenho.
const ALVO_DE_TRIANGULOS: usize = 1_200;

/// ⭐⭐⭐ **O CAMPO DE PESOS DO DOMÍNIO — a malha do bind, VIVA.**
///
/// # ⛔⛔ Por que ela deixou de ser deitada fora (2026-09-20)
///
/// Até aqui esta malha era **andaime**: construía-se, resolvia-se o padrão-ouro sobre ela e
/// guardava-se **uma linha por ponto de controlo**. Medido, o preço dessa amostragem é grande:
///
/// | forma | nós do desenho | vértices da malha | razão |
/// |---|---:|---:|---:|
/// | `Rectangle 40×10` | `4` | `516` | **`129×`** |
/// | `Ellipse 40×10` | `4` | `531` | **`133×`** |
/// | `Polygon 40×40` | `3` | `569` | **`190×`** |
/// | `Star 40×40` | `6` | `439` | **`73×`** |
///
/// ⭐⭐ **E a perda não é a resolução, é a FORMA da transição.** Entre dois nós a lei da curva
/// mistura as duas linhas em **linha recta** (`lerp(ra, rb, t)`), e o campo verdadeiro atravessa
/// uma junta num **«S»**. Medido num rectângulo de `40 × 10` com dois ossos, aresta a aresta:
///
/// | segmento | erro máx. da recta | erro médio | `t` do pior |
/// |---|---:|---:|---:|
/// | `(0,0) → (40,0)` — **cruza a junta** | **`0,3709`** | `0,2022` | `0,61` |
/// | `(40,0) → (40,10)` — não cruza | `0,0000` | `0,0000` | — |
/// | `(40,10) → (0,10)` — **cruza a junta** | **`0,3751`** | `0,2035` | `0,39` |
/// | `(0,10) → (0,0)` — não cruza | `0,0000` | `0,0000` | — |
///
/// ⇒ `0,375` é **37,5 % de um osso inteiro**, e as duas arestas que NÃO cruzam junta nenhuma leem
/// `0,0000` — *é esse par que prova que o número é a lei e não ruído da régua*.
///
/// ⛔ **E o «S» não se adivinha:** a recusa medida da F35 mostra que um `smoothstep` no lugar da
/// recta corta a quebra do alvo a meio (`p50 9,05 → 3,98`) e **não** cura o máximo
/// (`28,62 → 26,98`). *A forma da transição é do DOMÍNIO — ela mede-se, e quem a tem é esta malha.*
///
/// # O preço, medido em `--release`
///
/// Guardá-la custa `~16 KB` de posições e pesos por forma (`516` vértices, 2 ossos) mais os
/// triângulos; consultá-la custa `0,0015 ms` por amostra de curva (`0,095 ms` para as `64` de um
/// contorno de quatro segmentos, `~0,6 %` de um quadro). ⭐ **O caro já era pago:** a malha e o
/// padrão-ouro custam `31,9 ms` e correm **uma vez, ao prender** — o que esta struct muda é ela
/// não ser esquecida a seguir.
///
/// ⚠️ **A régua é guardada e não re-derivada.** Ela é `[origem_x, origem_y, escala]` do
/// [`malha_do_dominio_com_regua`], e re-calculá-la a jusante exigiria a caixa dos anéis **cozidos de hoje** —
/// que muda quando o artista edita a forma, enquanto a malha é a do **repouso do bind**.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CampoDoDominio {
    /// A malha do interior, em coordenadas próprias (pixels de uma caixa de lado maior `512`).
    pub malha: ph2d_poly2d::Mesh2d,
    /// **Achatado**: `pesos[v * ossos + j]` é a fracção do vértice `v` da malha que pertence ao
    /// tendão `j`.
    ///
    /// ⚠️ **Achatado e não `Vec<Vec<f64>>`**, pela mesma razão escrita na irmã da imagem: isto é
    /// lido **por amostra de curva, por quadro**, e um `Vec` por vértice seria uma alocação por
    /// consulta.
    pub pesos: Vec<f64>,
    /// A régua `local → malha`: `[origem_x, origem_y, escala]`.
    pub regua: [f64; 3],
}

impl CampoDoDominio {
    /// Quantos ossos a tabela cobre — `0` quando a malha não tem vértices. ⚠️ **DERIVADO**, como
    /// nas duas irmãs.
    #[must_use]
    pub fn ossos(&self) -> usize {
        self.pesos
            .len()
            .checked_div(self.malha.rest.len())
            .unwrap_or(0)
    }

    /// ⛔ **O par fecha?** — os pesos têm de ser um múltiplo exacto da contagem de vértices.
    #[must_use]
    pub fn valida(&self) -> bool {
        let n = self.malha.rest.len();
        n > 0 && self.pesos.len().is_multiple_of(n) && !self.malha.tris.is_empty()
    }

    /// O ponto `p` do espaço **LOCAL** do caminho, em coordenadas da malha.
    #[must_use]
    fn para_malha(&self, p: [f64; 2]) -> [f64; 2] {
        [
            (p[0] - self.regua[0]) * self.regua[2],
            (p[1] - self.regua[1]) * self.regua[2],
        ]
    }

    /// ⭐⭐⭐ **A LINHA DE PESOS DE UM PONTO QUALQUER do interior** — o que a tabela por ponto de
    /// controlo não sabe responder.
    ///
    /// `None` fora da malha ou com o par por fechar. ⚠️ **Fora dela quem chama NÃO cai em zeros:**
    /// a lei da curva mantém a mistura das linhas dos nós, que é a resposta de sempre — *um ponto
    /// sem domínio não é um ponto sem dono*.
    #[must_use]
    pub fn linha(&self, p_local: [f64; 2]) -> Option<Vec<f64>> {
        if !self.valida() {
            return None;
        }
        let n = self.ossos();
        amostra_achatada(&self.malha, &self.pesos, self.para_malha(p_local), n)
    }
}

/// ⭐⭐⭐ **O CAMPO do padrão-ouro para uma forma vectorial** — a malha do domínio com os pesos
/// resolvidos sobre ela, prontos a sobreviver ao bind.
///
/// `None` nos mesmos casos do [`pesos_do_caminho`] (caminho aberto, área nula, solver sem
/// resposta) — ⛔ e a razão de serem os mesmos é que **é a mesma função**: aquele delega neste.
#[must_use]
pub fn campo_do_caminho(path: &VecPath, ossos: &[Handle]) -> Option<CampoDoDominio> {
    if ossos.is_empty() {
        return None;
    }
    let aneis = contornos_fechados(path);
    if aneis.is_empty() {
        return None;
    }
    let (malha, regua) = malha_do_dominio_com_regua(&aneis, ossos)?;
    let para_malha = |p: [f64; 2]| [(p[0] - regua[0]) * regua[2], (p[1] - regua[1]) * regua[2]];
    let handles: Vec<Handle> = ossos
        .iter()
        .map(|o| Handle {
            a: para_malha(o.a),
            b: para_malha(o.b),
        })
        .collect();
    let w = ph2d_skin_weights::bounded_biharmonic(
        &malha,
        &handles,
        ph2d_skin_weights::Options::default(),
    )?;
    let n = ossos.len();
    let mut pesos = Vec::with_capacity(malha.rest.len() * n);
    for linha in &w.por_vertice {
        pesos.extend_from_slice(linha);
    }
    Some(CampoDoDominio {
        malha,
        pesos,
        regua,
    })
}

/// ⭐⭐⭐ **OS PESOS DE UMA FORMA VECTORIAL, pelo padrão-ouro** — um por ponto de controlo, achatado.
///
/// A ordem é a de [`VecPath::for_each_vert_mut`] — para o vértice `k`, as três casas
/// `3k`, `3k+1`, `3k+2` são **âncora**, **alça de entrada** e **alça de saída**, cada uma com
/// `ossos` pesos. ⚠️ Ela é estável porque o caminho GUARDADO é que a define, e ele não muda depois
/// do bind.
///
/// ⚠️⚠️ **Só a linha `3k` é LIDA desde 2026-09-19** — o peso é do NÓ ([`crate::dono_do_peso`]). As
/// duas das alças continuam a ser gravadas porque a forma da tabela viaja em bytes opacos dentro do
/// `SkinBind::source`, e encolhê-la mudaria o que já está guardado por uma economia que ninguém
/// mediu. ⛔ **Elas são AMOSTRAS e não incógnitas:** o sistema resolve-se na malha do domínio, logo
/// tirá-las não mexeria num único peso de âncora — *é dívida de tamanho, nunca de resultado*.
///
/// `ossos` são os eixos no espaço LOCAL do caminho (o mesmo em que os vértices vivem).
///
/// `None` quando não há domínio (caminho aberto, área nula) ou quando o solver não responde — ⛔ nos
/// dois casos a resposta certa é *não sei*, e quem chama cai na lei derivada.
#[must_use]
pub fn pesos_do_caminho(path: &VecPath, ossos: &[Handle]) -> Option<Vec<f64>> {
    Some(pesos_dos_pontos(path, &campo_do_caminho(path, ossos)?))
}

/// ⭐⭐ **A TABELA POR PONTO DE CONTROLO, amostrada de um campo JÁ RESOLVIDO.**
///
/// ⚠️ **Ela existe para o bind não pagar o padrão-ouro DUAS vezes:** desde que o
/// [`CampoDoDominio`] sobrevive, quem prende precisa dos dois (a tabela, que é o que a lei dos nós
/// lê; e o campo, que é o que a lei da curva consulta) — e o caro são os `31,9 ms` do solver, não
/// a amostragem.
///
/// ⚠️ A saída é **byte-idêntica** à que o [`pesos_do_caminho`] devolvia antes de 2026-09-20: a
/// aritmética baricêntrica é a mesma e a tabela achatada indexa as mesmas linhas.
#[must_use]
pub fn pesos_dos_pontos(path: &VecPath, campo: &CampoDoDominio) -> Vec<f64> {
    let (malha, w) = (&campo.malha, &campo.pesos);
    let n = campo.ossos();
    let mut out = Vec::new();
    let mut nos_de_fora = 0usize;
    for v in path.verts_all() {
        for (j, p) in [v.anchor, v.in_handle, v.out_handle]
            .into_iter()
            .enumerate()
        {
            let pm = campo.para_malha(p);
            match amostra_achatada(malha, w, pm, n) {
                Some(ws) => out.extend_from_slice(&ws),
                None => {
                    // ⚠️ **Uma ALÇA pode viver FORA da forma** (ela é uma tangente, não um ponto do
                    // desenho), e ali não há domínio. A resposta é o vértice da malha mais próximo —
                    // ⛔ nunca zeros, que a normalização a jusante leria como *«este ponto não é de
                    // ninguém»* e entregaria ao primeiro osso.
                    if crate::e_no(j) {
                        nos_de_fora += 1;
                    }
                    out.extend_from_slice(&mais_proximo_achatado(malha, w, pm, n));
                }
            }
        }
    }
    // ⛔⛔ **A queixa conta só os NÓS, e a mudança é de 2026-09-19** (auditoria do report do dono):
    // desde que o peso é do NÓ ([`crate::dono_do_peso`]), a linha de uma alça é **gravada e nunca
    // lida** — queixar-se dela é descrever uma condição que já não tem consumidor nenhum, e o
    // artista lia *«N pontos de controlo caem fora da forma»* sobre pontos cujo peso não governa
    // nada. ⚠️ **Um nó de fora continua a ser real e continua a falar:** ali o peso que o desenho
    // usa é herdado de um vizinho.
    if nos_de_fora > 0 {
        eprintln!(
            "[bone] {nos_de_fora} NO(S) do desenho caem FORA do interior da forma — cada um herdou \
             os pesos do vertice mais proximo da malha"
        );
    }
    out
}

/// Os contornos **fechados** do caminho cozido, achatados em polígonos de espaço LOCAL.
///
/// ⛔ Só os fechados: uma linha de construção não é fronteira de nada, e é essa a mesma regra que a
/// `ph2d_vec_scene::boundary::outline` já aplica.
fn contornos_fechados(path: &VecPath) -> Vec<Vec<[f64; 2]>> {
    let cozido = path.cooked();
    let mut out = Vec::new();
    for c in 0..cozido.contour_count() {
        let Some((verts, closed)) = cozido.contour(c) else {
            continue;
        };
        if !closed || verts.len() < 2 {
            continue;
        }
        let mut poly = Vec::with_capacity(verts.len() * AMOSTRAS);
        for i in 0..verts.len() {
            let a = &verts[i];
            let b = &verts[(i + 1) % verts.len()];
            for k in 0..AMOSTRAS {
                let t = k as f64 / AMOSTRAS as f64;
                poly.push(cubica(a.anchor, a.out_handle, b.in_handle, b.anchor, t));
            }
        }
        if ph2d_poly2d::signed_area(&poly).abs() > f64::EPSILON {
            out.push(poly);
        }
    }
    out
}

/// Um ponto da cúbica em `t`.
fn cubica(p0: [f64; 2], p1: [f64; 2], p2: [f64; 2], p3: [f64; 2], t: f64) -> [f64; 2] {
    let u = 1.0 - t;
    let (w0, w1, w2, w3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
    [
        w0 * p0[0] + w1 * p1[0] + w2 * p2[0] + w3 * p3[0],
        w0 * p0[1] + w1 * p1[1] + w2 * p2[1] + w3 * p3[1],
    ]
}

/// `true` quando `p` está DENTRO dos anéis, pela regra par-ímpar.
///
/// ⭐ **Par-ímpar e não *nonzero*, e a diferença é um FURO:** um caminho composto escreve o anel de
/// dentro com a mesma orientação do de fora tantas vezes quanto o artista quiser, e é a paridade que
/// faz o furo ser furo. *Um domínio que tape o furo dá peso a uma região que não é arte.*
fn dentro(aneis: &[Vec<[f64; 2]>], p: [f64; 2]) -> bool {
    let mut cruz = 0usize;
    for anel in aneis {
        for i in 0..anel.len() {
            let (a, b) = (anel[i], anel[(i + 1) % anel.len()]);
            if (a[1] > p[1]) != (b[1] > p[1]) {
                let x = (b[0] - a[0]) * (p[1] - a[1]) / (b[1] - a[1]) + a[0];
                if x > p[0] {
                    cruz += 1;
                }
            }
        }
    }
    cruz % 2 == 1
}

/// ⭐⭐ **O segmento `p→q` toca o rectângulo `[x0,y0]..[x1,y1]`?** — a segunda metade da cerca de
/// cobertura da célula.
///
/// ⚠️ **Ela responde «toca», nunca «está dentro»:** o que interessa é a célula ter arte a
/// atravessá-la, e um segmento que a corta de lado a lado não tem **nenhum** extremo lá dentro.
fn cruza_a_celula(p: [f64; 2], q: [f64; 2], x0: f64, y0: f64, x1: f64, y1: f64) -> bool {
    // Rejeição barata pela caixa do segmento — é ela que mantém isto `O(1)` no caso comum.
    if p[0].max(q[0]) < x0 || p[0].min(q[0]) > x1 || p[1].max(q[1]) < y0 || p[1].min(q[1]) > y1 {
        return false;
    }
    let dentro_do_rect = |r: [f64; 2]| r[0] >= x0 && r[0] <= x1 && r[1] >= y0 && r[1] <= y1;
    if dentro_do_rect(p) || dentro_do_rect(q) {
        return true;
    }
    // Sem extremo dentro, só resta o segmento cortar um LADO da célula.
    let lado = |a: [f64; 2], b: [f64; 2], c: [f64; 2], d: [f64; 2]| {
        let orientacao = |o: [f64; 2], u: [f64; 2], v: [f64; 2]| {
            ((u[0] - o[0]) * (v[1] - o[1]) - (v[0] - o[0]) * (u[1] - o[1])).signum()
        };
        orientacao(a, b, c) != orientacao(a, b, d) && orientacao(c, d, a) != orientacao(c, d, b)
    };
    let cantos = [[x0, y0], [x1, y0], [x1, y1], [x0, y1]];
    (0..4).any(|i| lado(p, q, cantos[i], cantos[(i + 1) % 4]))
}

/// A malha do domínio e a régua `local → malha`.
///
/// ⚠️ **A malha vive em coordenadas próprias, com a origem no canto da caixa** — a `ph2d_poly2d`
/// trabalha em pixels de imagem (`y` para baixo, origem no canto), e traduzir na fronteira é mais
/// barato que ensinar a grelha a viver noutro referencial.
/// ⚠️ **Ela devolve a régua como DADO (`[origem_x, origem_y, escala]`) e não como fecho.** Um fecho
/// serve quem a usa no mesmo instante e **não sobrevive ao bind** — e desde que o
/// [`CampoDoDominio`] existe a régua tem de viajar com a malha, senão o consumidor de amanhã
/// re-deriva-a da caixa dos anéis de **hoje**, que o artista já editou.
fn malha_do_dominio_com_regua(
    aneis: &[Vec<[f64; 2]>],
    ossos: &[Handle],
) -> Option<(ph2d_poly2d::Mesh2d, [f64; 3])> {
    let mut caixa = [
        f64::INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NEG_INFINITY,
    ];
    for p in aneis.iter().flatten() {
        caixa[0] = caixa[0].min(p[0]);
        caixa[1] = caixa[1].min(p[1]);
        caixa[2] = caixa[2].max(p[0]);
        caixa[3] = caixa[3].max(p[1]);
    }
    let (larg, alt) = (caixa[2] - caixa[0], caixa[3] - caixa[1]);
    if !(larg > 0.0 && alt > 0.0) {
        return None;
    }
    // A malha é indexada em «pixels» de uma caixa de `LADO` de lado maior — a grelha é adimensional
    // e o orçamento é em triângulos, então a escala só tem de ser a mesma nos dois eixos.
    const LADO: f64 = 512.0;
    let escala = LADO / larg.max(alt);
    let regua = [caixa[0], caixa[1], escala];
    let para_malha = |p: [f64; 2]| [(p[0] - regua[0]) * regua[2], (p[1] - regua[1]) * regua[2]];
    let aneis_malha: Vec<Vec<[f64; 2]>> = aneis
        .iter()
        .map(|a| a.iter().map(|&p| para_malha(p)).collect())
        .collect();
    let focos: Vec<[f64; 2]> = ossos
        .iter()
        .flat_map(|o| [para_malha(o.a), para_malha(o.b)])
        .collect();
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a caixa foi escalada para um lado maior de 512"
    )]
    let (w, h) = (
        (larg * escala).ceil() as u32 + 1,
        (alt * escala).ceil() as u32 + 1,
    );
    let malha = ph2d_poly2d::grid_mesh_com(
        &|x0, y0, x1, y1, folga| {
            let (a, b) = (x0 - folga, y0 - folga);
            let (c, d) = (x1 + folga, y1 + folga);
            // A célula tem arte se o centro OU algum canto (com a folga) estiver dentro.
            let cinco = [
                [(a + c) / 2.0, (b + d) / 2.0],
                [a, b],
                [c, b],
                [c, d],
                [a, d],
            ]
            .iter()
            .any(|&p| dentro(&aneis_malha, p));
            // ⭐⭐⭐ **OU a FRONTEIRA atravessa a célula** — e esta segunda metade é a que faz a
            // malha cobrir a ARTE, medida em 2026-09-20.
            //
            // ⛔⛔ As cinco amostras são um sorteio: uma ponta fina ou um entalhe estreito
            // atravessa a célula **sem** que nenhuma delas caia dentro, e a célula é descartada com
            // o desenho a passar por ela. Medido pela fracção de amostras da curva que ficam sem
            // malha: **`19,23 %` numa estrela** (pior a `36` px de todo vértice, numa caixa de
            // `512`) e `4,10 %` num polígono — contra `0,00 %` nas duas com esta metade.
            //
            // ⭐ E ela **não custa triângulos**: o orçamento redistribui-se, e a estrela até desce
            // (`654 → 608`). *O que faltava não era resolução, era a pergunta.*
            //
            // ⚠️ Isto corrige um defeito que já existia **antes** de alguém amostrar a curva: o
            // [`pesos_do_caminho`] já resolve os pontos de controlo contra esta malha, e um que
            // caia fora herda o vértice mais próximo — é a queixa que ele imprime.
            cinco
                || aneis_malha.iter().any(|anel| {
                    let n = anel.len();
                    (0..n).any(|i| cruza_a_celula(anel[i], anel[(i + 1) % n], a, b, c, d))
                })
        },
        w,
        h,
        &focos,
        ph2d_poly2d::GridOptions {
            target_tris: ALVO_DE_TRIANGULOS,
            ..ph2d_poly2d::GridOptions::default()
        },
    )?;
    Some((malha, regua))
}

/// Os pesos num ponto qualquer, por coordenadas baricêntricas. `None` fora da malha.
fn amostra_achatada(
    m: &ph2d_poly2d::Mesh2d,
    pesos: &[f64],
    p: [f64; 2],
    n: usize,
) -> Option<Vec<f64>> {
    let linha = |v: u32| -> &[f64] {
        let i = v as usize * n;
        pesos.get(i..i + n).unwrap_or(&[])
    };
    for t in &m.tris {
        let (a, b, c) = (
            m.rest[t[0] as usize],
            m.rest[t[1] as usize],
            m.rest[t[2] as usize],
        );
        let den = (b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1]);
        if den.abs() < 1e-12 {
            continue;
        }
        let u = ((p[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (p[1] - a[1])) / den;
        let v = ((b[0] - a[0]) * (p[1] - a[1]) - (p[0] - a[0]) * (b[1] - a[1])) / den;
        if u < -1e-9 || v < -1e-9 || u + v > 1.0 + 1e-9 {
            continue;
        }
        let (pa, pb, pc) = (linha(t[0]), linha(t[1]), linha(t[2]));
        if pa.len() < n || pb.len() < n || pc.len() < n {
            // ⛔ Tabela por fechar: devolver meia linha entregaria pesos plausíveis sobre os
            // pontos errados, que é pior que não saber.
            return None;
        }
        return Some(
            (0..n)
                .map(|k| (1.0 - u - v) * pa[k] + u * pb[k] + v * pc[k])
                .collect(),
        );
    }
    None
}

/// Os pesos do vértice da malha mais próximo — a resposta para uma alça que vive fora da forma.
fn mais_proximo_achatado(
    m: &ph2d_poly2d::Mesh2d,
    pesos: &[f64],
    p: [f64; 2],
    n: usize,
) -> Vec<f64> {
    let mut melhor = (f64::INFINITY, 0usize);
    for (i, q) in m.rest.iter().enumerate() {
        let d = (q[0] - p[0]).hypot(q[1] - p[1]);
        if d < melhor.0 {
            melhor = (d, i);
        }
    }
    let i = melhor.1 * n;
    pesos
        .get(i..i + n)
        .map(<[f64]>::to_vec)
        .unwrap_or_else(|| vec![1.0 / n as f64; n])
}

/// ⭐ **Os gates da malha do domínio**, num irmão — ver o cabeçalho dele.
#[cfg(test)]
#[path = "pesos_tests.rs"]
mod tests;

//! **A PARAMETRIZAÇÃO DE UM PATCH** — a grade deixa de ser construída no ESPAÇO e
//! passa a ser construída **na superfície**.
//!
//! # ⛔ O que ela substitui, e por que a substituição é de espécie
//!
//! O Coons interpola as quatro **curvas de bordo** e devolve um ponto no espaço
//! `ℝ³` que, em geral, **não está na peça**; o remédio era agarrá-lo à face mais
//! próxima ([`ph2d_remesh_iso::project_onto`]). Sobre um patch grande e curvo —
//! uma esfera com um gancho — o ponto interpolado mergulha para dentro da forma e
//! o ponto mais próximo dele fica **do outro lado**. A face entre dois vizinhos
//! assim aterrados vira: é a fenda escura que o artista fotografa.
//!
//! ⚠️ **Medido em 2026-08-21: o alisamento REPARA e não CAUSA** (`0 → 25 dobras`,
//! `6 → 17`, `24 → 18`), e trocar a superfície da projeção não move o número
//! (`17 → 15`). *Um remédio que melhora monotonicamente e estagna está a tratar o
//! sintoma.*
//!
//! # A construção certa, e a garantia que ela traz
//!
//! 1. Achata-se o patch — as suas faces, que são faces **da malha** — sobre um
//!    polígono convexo, pelo **embutimento baricêntrico de Tutte** (1963): a
//!    fronteira vai para o polígono, e cada vértice interior fica na média dos
//!    vizinhos. ⭐ **Com pesos positivos e fronteira convexa, o embutimento é
//!    garantidamente SEM DOBRAS** — é o teorema, não uma esperança.
//! 2. A grade é construída no **domínio**, onde ela é uma grade de um quadrado.
//! 3. Cada ponto dela volta pela triangulação achatada e aterra **exactamente
//!    sobre uma face do patch**.
//!
//! ⇒ nenhum ponto interior nasce fora da peça, e nenhum ponto de um patch aterra
//! no vizinho. *A projeção deixa de ser a forma de corrigir a construção e passa a
//! ser só o passo que troca a malha remalhada pela original.*
//!
//! ⚠️ **Clean-room e sem dono:** Tutte (1963) e a interpolação transfinita são
//! matemática clássica. Nenhuma linha vem de fonte GPL — ver ADR-0162.

use std::collections::BTreeMap;

use ph2d_mesh::Mesh;

use crate::weights::{cotangent_weights, mean_value_weights};

/// ⚠️ **Quantas rondas de Gauss–Seidel o achatamento corre no máximo.**
///
/// ⭐ **Ele é um teto de ESPERA e não de qualidade**, e o número saiu da medição e
/// não de uma opinião: a sonda `how_flat_does_the_patch_get` imprime, por patch, em
/// quantas rondas o resíduo desce abaixo de [`FLATTEN_TOL`]. ⛔ Quem o atingir sai
/// com um achatamento **válido** (Tutte só exige pesos positivos e fronteira
/// convexa, que a iteração preserva a cada passo) e apenas menos relaxado.
const FLATTEN_ROUNDS: usize = 4_000;

/// O resíduo abaixo do qual o achatamento pára — em unidades do domínio, que tem
/// raio `1`. ⚠️ `1e-5` é meia parte por 100 000 da peça: abaixo disso o movimento
/// já não muda a face em que um ponto de grade cai.
const FLATTEN_TOL: f32 = 1.0e-5;

/// ⭐⭐⭐ **O MAPA é o CONFORME (cotangente) ou o de valor médio?** — ver
/// [`cotangent_weights`], que traz a dedução que levou aqui.
///
/// ⚠️ **Com `false` o achatamento é byte-idêntico ao de sempre.** A rede que torna
/// esta escolha segura é a contagem de triângulos virados no domínio
/// ([`crate::aligned::flipped`]): quem virar recua para o de valor médio, e o
/// [`crate::FillReport::fell_back`] conta quantos.
const CONFORMAL_MAP: bool = false;

/// ⭐⭐⭐ **A FRONTEIRA VAI PARA O DOMÍNIO POR SEGMENTO, ou por COMPRIMENTO DE ARCO?**
///
/// # ⛔ O que a medição de 2026-08-23 mostrou
///
/// A grade sai com `18°` de enviesamento numa esfera **lisa** — e mede **`18,7°` já
/// no DOMÍNIO**, antes de tocar na superfície ([`crate::FillReport::domain_skew`]).
/// ⇒ *o mapa é inocente* (trocá-lo por cotangente não moveu nada) **e o defeito nasce
/// em onde os pontos de bordo são postos.**
///
/// # O mecanismo
///
/// O [`crate::fan::coons`] liga o ponto `k` do bordo `bottom` ao ponto `k` do bordo
/// `top`. Se os dois estiverem em **fracções diferentes** do respectivo lado, a linha
/// de grade que os une sai **inclinada** — e toda a coluna com ela.
///
/// ⚠️ **Dentro de UM arco os dois espaços são proporcionais** (a reamostragem é por
/// `τ` igual), então ali não há desvio. ⛔ **A divergência aparece quando um lado tem
/// VÁRIOS arcos:** um lado com arcos de `τ = 1,0 / 10 segmentos` e `τ = 2,0 /
/// 5 segmentos` põe o seu ponto médio em `τ` a `50 %` e em segmentos a `67 %`. O lado
/// oposto tem outra composição, e a diferença vira ângulo.
///
/// ⭐ **Em espaço de SEGMENTO o ponto `k` está em `k/s` por definição, nos dois
/// lados** — a linha de grade nasce recta.
///
/// ⚠️⚠️ **Os DOIS sítios que leem isto têm de concordar**: o pino dos vértices de
/// malha (aqui) e os pontos de saída ([`crate::patch::side_uv`]). *Se divergirem, o
/// `uv` de um ponto de saída deixa de cair entre os `uv` dos vértices à volta dele e a
/// malha rasga ao longo de toda fronteira de patch* — com um erro pequeno demais para
/// se ver num render.
pub(crate) const SEGMENT_SPACE_DOMAIN: bool = false;

/// **A RÉGUA DE UM LADO** — por vértice de malha daquele lado, a `(fracção de τ, uv)`.
///
/// ⚠️ **A fracção de `τ` vem primeiro porque é a chave de busca**: quem tem um ponto
/// de saída tem a fracção dele e quer o `uv`, nunca o contrário.
type SideChain = Vec<(f32, [f32; 2])>;

/// **UM PATCH ACHATADO** — a triangulação dele com uma coordenada 2D por vértice.
pub(crate) struct PatchParam {
    /// Por vértice local, o ponto no domínio.
    uv: Vec<[f32; 2]>,
    /// Por vértice local, a posição 3D na malha que o layout indexa.
    pos: Vec<[f32; 3]>,
    /// Os triângulos, em índices locais.
    tris: Vec<[u32; 3]>,
    /// Balde uniforme sobre `[-1,1]²`, para localizar o triângulo de um `uv`.
    bucket: Vec<Vec<u32>>,
    /// Quantas células por lado.
    side: usize,
    /// Quantas rondas o achatamento gastou — ⭐ é o que a sonda lê.
    pub(crate) rounds: usize,
    /// O resíduo com que ele parou.
    pub(crate) residual: f32,
    /// ⚠️ **O desacordo do campo penteado**, em graus — ver
    /// [`crate::aligned::Aligned::holonomy_deg`]. `0` quando o campo não veio.
    pub(crate) holonomy: f32,
    /// ⭐ **Este patch RECUOU para o achatamento harmónico** porque o alinhado
    /// virou triângulos no domínio. É a contagem que diz se a rede é teórica.
    pub(crate) fell_back: bool,
    /// ⭐⭐⭐ **A FRONTEIRA DESLIZOU** — ver [`crate::rectangle`]. Por lado, a
    /// cadeia de `(fracção de τ, uv)` dos vértices de malha daquele lado.
    ///
    /// ⛔ **`None` quando a fronteira foi PREGADA** (todo `n ≠ 4`, e o `n = 4` que
    /// recuou), e aí o `uv` de um ponto de saída sai da aresta do polígono como
    /// sempre. ⚠️ *É esta opção que impede os dois sítios de divergirem*: quem
    /// deslizou entrega a régua, quem não deslizou não a tem.
    side_chain: Option<Vec<SideChain>>,
    /// ⭐ **Por que este patch NÃO deslizou** — a coluna de
    /// [`crate::FillReport::slid_refused`], ou `None` quando ele deslizou (ou quando o
    /// mapa nem foi tentado).
    pub(crate) slid_refused: Option<usize>,
    /// ⭐⭐⭐ **QUÃO LONGE DE CONFORME este achatamento ficou** — a mediana da razão
    /// entre os valores singulares do jacobiano, por triângulo
    /// ([`crate::lscm::conformal_error`]). `1,0` é conforme perfeito.
    ///
    /// ⛔ **É o CONTROLO de toda troca de achatamento.** Sem ele, «o LSCM não melhorou
    /// a malha» não distingue *«o mapa não é o constrangimento»* de *«o meu LSCM está
    /// com um bug»* — e a primeira é uma conclusão sobre o algoritmo, a segunda sobre
    /// mim.
    pub(crate) conformal: f32,
}

/// O domínio vai de `-1` a `1` nos dois eixos (o polígono é inscrito no círculo
/// unitário; o quadrado usa os quatro cantos `(±1, ±1)/√2`).
const LO: f32 = -1.0;
const SPAN: f32 = 2.0;

impl PatchParam {
    /// **ACHATA UM PATCH.**
    ///
    /// `faces` são os índices, na `mesh`, das faces deste patch. `boundary` é a
    /// fronteira dele **por lado**: `boundary[i]` é a cadeia ordenada de vértices
    /// de malha do lado `i`, do canto de entrada ao de saída (o último vértice de
    /// um lado é o primeiro do seguinte).
    ///
    /// Devolve `None` quando o patch não é um disco triangulável — e ⚠️ **`None` é
    /// uma resposta e não uma falha**: o chamador volta à construção antiga, que
    /// dobra mas devolve malha.
    pub(crate) fn build(
        mesh: &Mesh,
        faces: &[u32],
        boundary: &[Vec<u32>],
        tau: &[Vec<f32>],
        seg: &[u32],
        field: Option<&[[f32; 3]]>,
    ) -> Option<Self> {
        let n = boundary.len();
        if n < 3 || faces.is_empty() {
            return None;
        }
        // ── Índices locais, em ordem determinística.
        let mut local: BTreeMap<u32, u32> = BTreeMap::new();
        let mut tris: Vec<[u32; 3]> = Vec::with_capacity(faces.len());
        // ⭐ **De que face de MALHA cada triângulo veio.** O campo cruzado é dado
        // por face da malha, e um leque parte uma face em vários triângulos — sem
        // esta correspondência o [`crate::aligned`] teria de a redescobrir.
        let mut tri_face: Vec<u32> = Vec::with_capacity(faces.len());
        let mesh_faces = mesh.faces();
        for &f in faces {
            let v = mesh_faces.get(f as usize)?.verts();
            // ⚠️ Um leque sobre o vértice 0 basta: as faces desta fase são
            // triângulos (o F1 tria a malha) e um quad convexo parte em dois.
            for k in 1..v.len() - 1 {
                let mut t = [0u32; 3];
                for (slot, &g) in t.iter_mut().zip([v[0], v[k], v[k + 1]].iter()) {
                    let next = u32::try_from(local.len()).ok()?;
                    *slot = *local.entry(g).or_insert(next);
                }
                if t[0] != t[1] && t[1] != t[2] && t[2] != t[0] {
                    tris.push(t);
                    tri_face.push(f);
                }
            }
        }
        let pos_src = mesh.positions();
        let mut pos = vec![[0.0f32; 3]; local.len()];
        for (&g, &l) in &local {
            pos[l as usize] = *pos_src.get(g as usize)?;
        }

        // ── A fronteira vai para o polígono convexo, por COMPRIMENTO DE ARCO.
        //
        // ⚠️ **Por comprimento e não por contagem**, e é a mesma lei do
        // [`crate::fan::resample`]: a cadeia de malha tem arestas de tamanhos
        // diferentes, e distribuir por contagem faria o domínio herdar a densidade
        // da triangulação em vez da geometria do lado. *É isto que faz o `uv` de um
        // ponto de saída — que o F5 amostrou por comprimento — bater com o `uv` dos
        // vértices de malha à volta dele.*
        let mut uv = vec![[0.0f32; 2]; local.len()];
        let mut fixed = vec![false; local.len()];
        // ⭐⭐⭐ **O domínio tem os lados na proporção dos SEGMENTOS** — ver
        // [`corners_for_sides`]. Um polígono regular fazia toda célula nascer
        // torta antes de tocar na superfície.
        let poly = crate::domain::corners_for_sides(seg);
        for (i, chain) in boundary.iter().enumerate() {
            let (a, b) = (poly[i], poly[(i + 1) % n]);
            // ⭐⭐ **Pelo `τ` do layout, a MESMA régua que o [`crate::patch::side_uv`]
            // usa para os pontos de saída.** Sem graduação ele é o comprimento de
            // arco e nada muda; com ela, um vértice de malha e um ponto de saída na
            // mesma posição do arco recebem o mesmo `uv`. *Duas réguas aqui dariam
            // uma grade torcida junto ao bordo, sem nada a acusar.*
            let side_tau = tau.get(i)?;
            if side_tau.len() != chain.len() {
                return None;
            }
            let total: f32 = side_tau.last().copied().unwrap_or(0.0);
            for (k, &g) in chain.iter().enumerate() {
                let run = side_tau[k];
                let f = if total > 0.0 { run / total } else { 0.0 };
                let l = *local.get(&g)? as usize;
                // ⚠️ O último vértice de um lado é o primeiro do seguinte; escrever
                // os dois dá o mesmo canto, então a ordem não importa.
                uv[l] = [f.mul_add(b[0] - a[0], a[0]), f.mul_add(b[1] - a[1], a[1])];
                fixed[l] = true;
            }
        }
        // ⭐⭐⭐ **A FRONTEIRA DESLIZA, se o patch for de quatro lados** — ver
        // [`crate::rectangle`]. ⚠️ Ela corre **antes** do Tutte porque *substitui* o
        // achatamento inteiro em vez de o afinar: o problema misto (Dirichlet em dois
        // lados, natural nos outros dois) é outro problema, não outra iteração.
        // ⛔ Falhar aqui é a resposta normal — o patch segue para o achatamento
        // pregado de sempre, e nenhum outro sítio precisa de saber.
        let mut slid_refusal: Option<usize> = None;
        // ⭐⭐⭐ **O ACHATAMENTO DE FRONTEIRA LIVRE, se ligado** — ver [`crate::lscm`].
        // ⚠️ Ele corre **antes** do mapa do rectângulo e serve **todo `n`**: é a única
        // das três construções sem condição de fronteira para errar.
        if crate::lscm::LSCM_MAP {
            let mut chains: Vec<Vec<u32>> = Vec::with_capacity(n);
            for chain in boundary {
                let mut c = Vec::with_capacity(chain.len());
                for g in chain {
                    c.push(*local.get(g)?);
                }
                chains.push(c);
            }
            if let Some((flat, r, res, per_side)) = crate::lscm::try_flatten(
                &tris,
                &pos,
                &chains,
                tau,
                crate::lscm::LSCM_ROUNDS,
                FLATTEN_TOL * 0.01,
            ) {
                let mut me = Self::with(flat, pos, tris);
                me.rounds = r;
                me.residual = res;
                me.side_chain = Some(per_side);
                return Some(me);
            }
        }
        if crate::rectangle::RECTANGLE_MAP && n == 4 {
            let mut chains: Vec<Vec<u32>> = Vec::with_capacity(n);
            for chain in boundary {
                let mut c = Vec::with_capacity(chain.len());
                for g in chain {
                    c.push(*local.get(g)?);
                }
                chains.push(c);
            }
            // ⚠️ **Cotangente, obrigatoriamente** — a condição natural de um
            // Laplaciano discreto é a de Neumann **do operador que se usou**, e só o
            // cotangente é o Laplace–Beltrami da superfície.
            let cot = cotangent_weights(&tris, &pos);
            let attempt = crate::rectangle::solve(&cot, &chains, FLATTEN_ROUNDS, FLATTEN_TOL);
            // ⭐ **A recusa fica GUARDADA, não descartada** — ver
            // [`crate::FillReport::slid_refused`].
            slid_refusal = attempt.as_ref().err().map(|e| e.slot());
            if let Ok(s) = attempt {
                // ⛔ Rede 3: o teorema de Tutte não cobre nem fronteira livre nem
                // peso negativo. *Conta-se.*
                if crate::aligned::flipped(&tris, &s.uv) > 0 {
                    // ⭐ A rede do achatamento é a 5.ª coluna: o mapa fechou e virou
                    // triângulos no domínio.
                    slid_refusal = Some(4);
                } else {
                    let mut per_side: Vec<SideChain> = Vec::with_capacity(n);
                    for (i, c) in chains.iter().enumerate() {
                        let side_tau = tau.get(i)?;
                        let total = side_tau.last().copied().unwrap_or(0.0);
                        per_side.push(
                            c.iter()
                                .zip(side_tau)
                                .map(|(&v, &t)| {
                                    let f = if total > 0.0 { t / total } else { 0.0 };
                                    (f, s.uv[v as usize])
                                })
                                .collect(),
                        );
                    }
                    let mut me = Self::with(s.uv, pos, tris);
                    me.rounds = s.rounds;
                    me.residual = s.residual;
                    me.side_chain = Some(per_side);
                    return Some(me);
                }
            }
        }

        if fixed.iter().all(|&f| f) {
            // Um patch sem interior nenhum: o achatamento é a fronteira, e ele
            // continua a servir para localizar pontos.
            let mut me = Self::with(uv, pos, tris);
            me.rounds = 0;
            me.slid_refused = slid_refusal;
            return Some(me);
        }

        // ── Tutte: cada interior na média PONDERADA dos vizinhos, por Gauss–Seidel.
        //
        // ⭐⭐ **Os pesos são as COORDENADAS DE VALOR MÉDIO** (Floater 2003), e a
        // troca não é cosmética. Medido em 2026-08-21 com pesos **uniformes** — o
        // Tutte de manual — na `hooked_sphere`: o embutimento é válido (nenhuma
        // dobra nasce dele) e **distorcidíssimo**, e a grade construída sobre ele
        // saiu com aresta máxima **4,80×** o alvo contra 3,62× da construção antiga,
        // sem baixar as dobras. *Um mapa válido não é um mapa útil: um passo igual
        // no domínio tem de valer um passo parecido na superfície.*
        //
        // ⚠️ **E elas mantêm a garantia**: `tan(α/2) + tan(β/2)` sobre `|pᵢ − pⱼ|` é
        // **sempre positivo** num triângulo, e o teorema de Tutte só exige pesos
        // positivos com fronteira convexa. *Cotangente seria harmónico e admite peso
        // negativo num triângulo obtuso — e é aí que a garantia se perde.*
        // ⭐⭐⭐ **QUAL MAPA** — ver [`CONFORMAL_MAP`] e [`cotangent_weights`].
        let mut nb = if CONFORMAL_MAP {
            cotangent_weights(&tris, &pos)
        } else {
            mean_value_weights(&tris, &pos)
        };
        // ⚠️ Um interior sem vizinho nenhum não tem média: ele não pertence a
        // triângulo nenhum deste patch, e o achatamento não o pode colocar.
        if (0..local.len()).any(|v| !fixed[v] && nb[v].is_empty()) {
            return None;
        }
        // ⭐⭐⭐ **O CAMPO, se o layout o trouxe** — ver [`crate::aligned`]. Ele não
        // muda o operador da esquerda (os pesos continuam os de valor médio, sempre
        // positivos); ele acrescenta um alvo a cada passo. ⚠️ **Com `None` o laço
        // abaixo é a lei antiga termo a termo**, e é isso que torna a inércia
        // demonstrável em vez de prometida.
        let pinned = uv.clone();
        let aligned = field.and_then(|dir| {
            crate::aligned::build(&tris, &tri_face, &pos, dir, &nb, &pinned, &fixed)
        });
        let holonomy = aligned.as_ref().map_or(0.0, |a| a.holonomy_deg);
        let solve_with = |nb: &Vec<Vec<(u32, f32)>>,
                          uv: &mut Vec<[f32; 2]>,
                          step: Option<&Vec<Vec<[f32; 2]>>>| {
            let (mut rounds, mut residual) = (0usize, f32::INFINITY);
            for r in 0..FLATTEN_ROUNDS {
                let mut worst = 0.0f32;
                for v in 0..local.len() {
                    if fixed[v] {
                        continue;
                    }
                    let (mut sx, mut sy, mut sw) = (0.0f32, 0.0f32, 0.0f32);
                    for (slot, &(w, k)) in nb[v].iter().enumerate() {
                        // ⭐ `uv_j − c·d_ij` — o vizinho, recuado do passo que o
                        // campo pede. Sem campo, `g = 0` e isto é `uv_j`.
                        let g = step.map_or([0.0f32; 2], |s| s[v][slot]);
                        sx += k * (uv[w as usize][0] - g[0]);
                        sy += k * (uv[w as usize][1] - g[1]);
                        sw += k;
                    }
                    if sw <= 0.0 {
                        continue;
                    }
                    let inv = 1.0 / sw;
                    let next = [sx * inv, sy * inv];
                    worst = worst.max((next[0] - uv[v][0]).abs().max((next[1] - uv[v][1]).abs()));
                    uv[v] = next;
                }
                rounds = r + 1;
                residual = worst;
                if worst < FLATTEN_TOL {
                    break;
                }
            }
            (rounds, residual)
        };
        let (mut rounds, mut residual) =
            solve_with(&nb, &mut uv, aligned.as_ref().map(|a| &a.step));

        // ⭐⭐ **A REDE, e ela é uma CONTAGEM e não uma esperança.** O teorema de
        // Tutte garante um embutimento sem dobras para o problema **harmónico**;
        // com um alvo no lado direito a garantia deixa de se aplicar, e a única
        // resposta honesta é medir. ⛔ Um patch que vire triângulos no domínio faz
        // dois pontos de grade vizinhos aterrarem em lados opostos da superfície —
        // é o defeito que esta fase inteira existiu para matar.
        let mut fell_back = false;
        if crate::aligned::flipped(&tris, &uv) > 0 {
            // ⭐⭐ **A REDE, e ela cobre as DUAS escolhas.** O teorema de Tutte
            // garante um embutimento sem dobras para o problema harmónico **com
            // pesos positivos**; nem o alvo do campo no lado direito nem os pesos
            // cotangentes preservam essa hipótese. ⛔ *Não se assume que a garantia
            // sobrevive — conta-se, e recua-se para a lei que a tem.*
            let mv = mean_value_weights(&tris, &pos);
            if (0..local.len()).all(|v| fixed[v] || !mv[v].is_empty()) {
                nb = mv;
                uv = pinned;
                let (r, res) = solve_with(&nb, &mut uv, None);
                rounds = r;
                residual = res;
                fell_back = true;
            }
        }
        let mut me = Self::with(uv, pos, tris);
        me.rounds = rounds;
        me.residual = residual;
        me.holonomy = holonomy;
        me.fell_back = fell_back;
        me.slid_refused = slid_refusal;
        Some(me)
    }

    fn with(uv: Vec<[f32; 2]>, pos: Vec<[f32; 3]>, tris: Vec<[u32; 3]>) -> Self {
        let conformal = crate::lscm::conformal_error(&tris, &pos, &uv);
        // ⚠️ **O balde é dimensionado pelo número de triângulos**, não por uma
        // constante: um patch de 20 faces e um de 20 000 pedem grelhas diferentes,
        // e uma célula com metade do patch dentro faz a localização voltar a ser
        // linear.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        #[allow(clippy::cast_precision_loss)]
        let side = ((tris.len() as f32).sqrt().ceil() as usize).clamp(1, 128);
        let mut bucket = vec![Vec::new(); side * side];
        for (i, t) in tris.iter().enumerate() {
            let (mut lo, mut hi) = ([f32::MAX; 2], [f32::MIN; 2]);
            for &v in t {
                let p = uv[v as usize];
                for k in 0..2 {
                    lo[k] = lo[k].min(p[k]);
                    hi[k] = hi[k].max(p[k]);
                }
            }
            let (c0, c1) = (cell(lo, side), cell(hi, side));
            for cy in c0[1]..=c1[1] {
                for cx in c0[0]..=c1[0] {
                    bucket[cy * side + cx].push(u32::try_from(i).unwrap_or(0));
                }
            }
        }
        Self {
            uv,
            pos,
            tris,
            bucket,
            side,
            rounds: 0,
            residual: 0.0,
            holonomy: 0.0,
            fell_back: false,
            side_chain: None,
            slid_refused: None,
            conformal,
        }
    }

    /// ⭐⭐ **O `uv` DE UM PONTO DE SAÍDA quando a fronteira DESLIZOU.** `f` é a
    /// fracção de `τ` ao longo do lado — a **mesma** régua que o
    /// [`crate::patch::side_uv`] já usava.
    ///
    /// ⚠️ **Interpola entre os dois vértices de MALHA à volta dele**, e não sobre a
    /// aresta do polígono: com a fronteira a deslizar o lado deixa de ter os pontos
    /// a espaçamento igual, e é precisamente por isto que o `uv` de um ponto de saída
    /// continua a cair dentro do triângulo que o [`Self::sample`] vai encontrar.
    /// ⭐ **Este patch achou o mapa do rectângulo** — ver [`crate::FillReport::slid`].
    pub(crate) fn slid(&self) -> bool {
        self.side_chain.is_some()
    }

    pub(crate) fn uv_on_side(&self, i: usize, f: f32) -> Option<[f32; 2]> {
        let chain = self.side_chain.as_ref()?.get(i)?;
        if chain.len() < 2 {
            return None;
        }
        let (last, _) = *chain.last()?;
        let f = f.clamp(0.0, last.max(0.0));
        let k = chain
            .partition_point(|&(t, _)| t < f)
            .max(1)
            .min(chain.len() - 1);
        let (t0, a) = chain[k - 1];
        let (t1, b) = chain[k];
        let w = if t1 > t0 { (f - t0) / (t1 - t0) } else { 0.0 };
        Some([w.mul_add(b[0] - a[0], a[0]), w.mul_add(b[1] - a[1], a[1])])
    }

    /// **DE VOLTA À SUPERFÍCIE** — o ponto 3D sobre o triângulo que contém `q`.
    ///
    /// ⚠️ **Procura na célula do balde e depois nas oito à volta.** Um `uv` sobre a
    /// aresta partilhada por dois triângulos cai no primeiro que o aceitar, e os
    /// dois devolvem o mesmo ponto — a interpolação baricêntrica é contínua
    /// através da aresta.
    pub(crate) fn sample(&self, q: [f32; 2]) -> Option<([f32; 3], [f32; 3])> {
        let c = cell(q, self.side);
        for ring in 0..=1i64 {
            for dy in -ring..=ring {
                for dx in -ring..=ring {
                    if ring > 0 && dx.abs() < ring && dy.abs() < ring {
                        continue;
                    }
                    let (x, y) = (c[0] as i64 + dx, c[1] as i64 + dy);
                    if x < 0 || y < 0 || x >= self.side as i64 || y >= self.side as i64 {
                        continue;
                    }
                    #[allow(clippy::cast_sign_loss)]
                    for &t in &self.bucket[y as usize * self.side + x as usize] {
                        if let Some(p) = self.at(t as usize, q) {
                            return Some(p);
                        }
                    }
                }
            }
        }
        None
    }

    /// O ponto 3D e a **NORMAL do triângulo** em que ele caiu.
    ///
    /// ⭐⭐ **A normal viaja com o ponto porque a reprojeção precisa dela.** Ver
    /// [`ph2d_remesh_iso::project_facing`]: dentro de um vinco côncavo o ponto mais
    /// próximo da superfície original pode estar do **outro lado** da dobra, e é
    /// isso que rasga a malha. O ponto sabe de que lado veio — ele nasceu sobre uma
    /// face concreta da malha achatada —, e essa informação estava a ser deitada
    /// fora uma linha antes de quem precisava dela.
    fn at(&self, t: usize, q: [f32; 2]) -> Option<([f32; 3], [f32; 3])> {
        let tri = self.tris[t];
        let (a, b, c) = (
            self.uv[tri[0] as usize],
            self.uv[tri[1] as usize],
            self.uv[tri[2] as usize],
        );
        let area = edge(a, b, c);
        if area.abs() <= f32::EPSILON {
            return None;
        }
        let inv = 1.0 / area;
        let w = [
            edge(q, b, c) * inv,
            edge(a, q, c) * inv,
            edge(a, b, q) * inv,
        ];
        // ⚠️ A folga é do TAMANHO DO DOMÍNIO e não relativa ao triângulo: um `uv`
        // exactamente sobre a fronteira do patch tem de ser aceite por alguém, e o
        // arredondamento de `f32` num domínio de raio 1 vive perto de `1e-7`.
        if w.iter().any(|&x| x < -1.0e-4) {
            return None;
        }
        let mut p = [0.0f32; 3];
        for (k, &v) in tri.iter().enumerate() {
            let s = self.pos[v as usize];
            for j in 0..3 {
                p[j] = w[k].mul_add(s[j], p[j]);
            }
        }
        let (p0, p1, p2) = (
            self.pos[tri[0] as usize],
            self.pos[tri[1] as usize],
            self.pos[tri[2] as usize],
        );
        let (e1, e2) = (
            [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]],
            [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]],
        );
        let n = [
            e1[1].mul_add(e2[2], -(e1[2] * e2[1])),
            e1[2].mul_add(e2[0], -(e1[0] * e2[2])),
            e1[0].mul_add(e2[1], -(e1[1] * e2[0])),
        ];
        Some((p, n))
    }
}

fn cell(p: [f32; 2], side: usize) -> [usize; 2] {
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    let f = |v: f32| ((((v - LO) / SPAN) * side as f32).floor().max(0.0) as usize).min(side - 1);
    [f(p[0]), f(p[1])]
}

fn edge(a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> f32 {
    (b[0] - a[0]).mul_add(c[1] - a[1], -((c[0] - a[0]) * (b[1] - a[1])))
}

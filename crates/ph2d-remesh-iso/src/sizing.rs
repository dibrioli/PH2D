//! ⭐⭐⭐ **O ALVO POR SÍTIO — e as DUAS cercas que ele destravou e que NÃO shipam.**
//!
//! Irmão de [`crate`] por RESPONSABILIDADE: o laço principal responde *«que aresta se
//! divide e que aresta colapsa?»*, e este módulo responde à pergunta anterior —
//! *«qual é o alvo AQUI?»*. ⛔ Um alvo único não pode representar uma agulha, e é isso
//! que amputava as pontas do artista antes de qualquer outra fase existir.
//!
//! ⚠️⚠️ **As duas portas deste módulo nascem DESLIGADAS, cada uma com a tabela da
//! rejeição no seu doc** ([`adaptive_on`] e [`facing_on`]). *As duas curam esta fase e
//! partem a seguinte* — a lei que este módulo pagou duas vezes é que **uma fase medida
//! sozinha pode melhorar e piorar o produto**, e a cura verdadeira trata as duas ao
//! mesmo tempo.

use ph2d_mesh::Mesh;

/// ⭐⭐⭐ **UMA CALOTA RESOLVIDA** — o pedido de quem chama: *«aqui o passo é no máximo `step`»*.
///
/// # ⛔⛔⛔ Por que a curvatura sozinha não a produz
///
/// O campo desta grelha é **a curvatura normalizada pela mediana**, e depois **renormalizado
/// pela contagem** ([`SizingGrid::build`]) para o orçamento não inflar. As duas leis juntas são
/// o que faz a agulha sobreviver — e o que deixa o **bico** a `1,3`–`2,3 ×` o passo da grade de
/// quads (medido 2026-09-02, `docs/3D/quad-remesh/PLANO_a_graduacao_da_ponta.md` §101): a
/// renormalização engrossa tudo, e um bico é uma região **pequena**, logo ele paga a factura de
/// uma peça inteira de superfície chapada.
///
/// ⚠️ **E `2 ×` o passo é o defeito que o dono fotografou duas vezes.** As singularidades do
/// campo cruzado vivem em **vértices desta malha**; o pólo `+1` que fecha um bico (quatro
/// `+¼` a `≤ 2 h`, que é o que a malha aprovada por ele tem) precisa de **`≥ 2` células
/// resolvidas** de calota. Com o bico a `2 h` por vértice, não há onde as pôr — *a grade
/// termina a meio caminho, e a extracção tapa o resto com uma face grande.*
///
/// ⛔ **Não é um refinamento global** (`PH2D_F1_TARGET=1`, a fase zero inteira ao alvo, foi
/// medido e **refutado**: `χ = 1`, `4` arestas de bordo, `123` dobras): a afinação tem de ser
/// **local**, e o orçamento reposto pela renormalização é o que a torna barata.
///
/// ⚠️ **Por POSIÇÃO e raio EUCLIDIANO**, como toda esta grelha: as portas de topologia
/// renumeram (`Remap`) dentro da própria chamada, então um índice de vértice não sobrevive à
/// ronda em que foi calculado.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cap {
    /// O bico — em coordenadas da malha que se remalha.
    pub at: [f32; 3],
    /// Até onde a calota alcança, em unidades da peça. ⛔ `≤ 0` é um no-op.
    pub radius: f32,
    /// O passo máximo lá dentro. ⛔ `≤ 0` é um no-op.
    pub step: f32,
}

impl Cap {
    /// Este pedido alcança este ponto?
    fn covers(&self, p: [f32; 3]) -> bool {
        if !(self.radius > 0.0 && self.step > 0.0) {
            return false;
        }
        let d = [p[0] - self.at[0], p[1] - self.at[1], p[2] - self.at[2]];
        d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])) <= self.radius * self.radius
    }
}

/// ⭐⭐⭐ **O ALVO POR SÍTIO, numa grelha grosseira** — o que as portas de topologia consultam.
///
/// ⛔⛔ **Ela existe porque um alvo ÚNICO não pode representar uma agulha.** Na peça do artista
/// (2026-08-29) o alvo é `0,089` e o **raio local** de um espinho cai a `0,037`: o passe de
/// colapso come toda aresta abaixo de `0,071`, e as arestas que dão a volta ao tubo são
/// justamente essas — *a agulha fecha-se sobre si antes de qualquer outra fase existir.*
///
/// ⚠️ **Por POSIÇÃO e não por índice**, porque é isso que as portas aceitam: elas renumeram
/// (`Remap`) e correm várias passagens por chamada, então um vetor por vértice ficaria
/// obsoleto dentro da própria chamada.
///
/// ⚠️ **A célula é o alvo GLOBAL**, e a consulta leva o **mínimo dos 27 vizinhos**: uma grelha
/// que respondesse só pela célula própria daria um degrau na fronteira dela, e um degrau no
/// limiar de colapso é uma fileira de arestas que morre de um lado e vive do outro.
pub(crate) struct SizingGrid {
    cell: f32,
    fallback: f32,
    /// ⭐⭐⭐ **A ORIGEM DA GRELHA — e ela é da PEÇA, nunca do MUNDO (2026-08-31).**
    ///
    /// ⛔⛔⛔ **Sem ela esta grelha punha a retopologia a depender de ONDE o objecto está na
    /// cena.** O [`Self::key_of`] dividia a coordenada de mundo pela célula, logo mover a
    /// peça movia as fronteiras dos baldes — e como cada balde guarda o **mínimo** e o
    /// [`Self::at`] lê o mínimo de 27, um deslocamento muda que região herda a finura de uma
    /// agulha.
    ///
    /// ⚠️ **Medido na `uv_sphere(96, 144)`, a mesma malha só transladada em `x`:**
    ///
    /// | | `0` | `½` | `1` | `2` | dispersão |
    /// |---|---|---|---|---|---|
    /// | ⛔ chave de mundo | `2 633` | `2 712` | `2 679` | `2 586` | **`4,9 %`** |
    /// | ⭐ ancorada na peça | `2 687` | `2 687` | `2 687` | `2 687` | **`0,0 %`** |
    ///
    /// ⚠️ **O canto MÍNIMO da caixa, e não o centroide:** o centroide dos vértices é uma
    /// propriedade da *amostragem* (é o defeito que a `reach` do shell pagou no mesmo dia); um
    /// extremo da caixa é função só da forma, e acompanha-a exactamente numa translação.
    ///
    /// ⛔ **E o report que a exigiu é do dono:** *«o remesh deve funcionar perfeitamente em
    /// qualquer lugar»* — depois de a MESMA escultura dar `0` de `4` pontas cortadas na
    /// origem e `2` de `4` onde o importador a ancora (`IMPORT_SPAN`, fora da origem).
    ///
    /// ⚠️ **O que ela NÃO compra, dito:** bit-exactidão a qualquer distância. A subtracção
    /// `p − origin` perde bits quando a peça está longe, e o remalhador é **iterativo** — um
    /// bit muda uma decisão de corte e a diferença cascateia. A `16` unidades a dispersão
    /// volta, e o gate declara essa cerca em vez de a esconder.
    origin: [f32; 3],
    want: std::collections::BTreeMap<(i32, i32, i32), f32>,
}

impl SizingGrid {
    /// ⭐ O alvo local sai da **curvatura normalizada pela mediana** — a mesma lei livre de
    /// escala que a `ph2d_quadflow::ScaleField` usa.
    ///
    /// # ⭐⭐⭐ A CONTAGEM É RENORMALIZADA (2026-08-31)
    ///
    /// ⛔⛔⛔ **Até esta data o tecto era `1`** — *«esta grelha nunca engrossa, então não pode
    /// piorar nenhuma região que o laço já resolveu»*. ⚠️ **Essa frase descrevia a intenção e
    /// escondia o preço:** um campo que só afina **acrescenta** trabalho em vez de o mover, e
    /// a malha de trabalho ia de `3 982` para **`33 156`** faces (`8,3×`). *É essa inflação —
    /// e não a graduação — que a jusante não digere*, e é o que as TRÊS recusas desta fase
    /// têm em comum (ver [`adaptive_on`], [`facing_on`] e o `PH2D_F1_TARGET` do shell).
    ///
    /// ⭐ A lei nova é a que a irmã um nível acima já tem e já é gateada
    /// (`ph2d_quadflow::ScaleField` + a renormalização da `sizing_field` do shell): *a
    /// adaptação **move** os quads; ela não os cria.*
    ///
    /// ⇒ o campo é escalado por `√(N_previsto / N_pedido)`, medido **pela própria grelha**
    /// ([`Self::count_factor`]) e não pelo campo por vértice.
    ///
    /// ⚠️ **A consequência honesta:** o factor sai `> 1` (o campo só afina, logo ele prevê mais
    /// faces que o alvo escalar), então a grelha passa a ser **mais grossa que o alvo** nas
    /// regiões chapadas. *Esse é exactamente o invariante que a fazia inflar* — e o que se
    /// compra com ele é a agulha, que é o que o dono fotografou.
    ///
    /// ⛔⛔ **E uma BANDA SIMÉTRICA foi construída, MEDIDA e REVERTIDA no mesmo dia.** A ideia
    /// era `[alvo/√R, alvo·√R]` em vez do tecto `1`; a mutação que a apagava **sobreviveu aos
    /// dois gates** (com a renormalização por cima, o tecto deixa de ser observável no
    /// intervalo), e o A/B ponta a ponta deu-lhe a resposta: pela régua **por ponta**, o tecto
    /// `1` é melhor nas duas peças do dono — `_base_sculpt` pior corte `−24,3 % → −8,4 %`,
    /// `sculpt_antes` `2/6 → 1/6` cortadas. *Uma mutação que sobrevive pode ser código inerte;
    /// aqui ela era código que fazia a coisa errada.*
    ///
    /// # ⭐⭐⭐ A CALOTA entra ANTES da renormalização e é RECLAMADA depois (2026-09-03)
    ///
    /// Ver [`Cap`]. As duas metades são deliberadas e cada uma responde a uma pergunta:
    ///
    /// - **antes**: o pedido da calota entra no campo por vértice, logo a
    ///   [`Self::count_factor`] **vê-o** e a factura dele é paga pelo resto da peça — *a
    ///   adaptação move os quads, ela não os cria*, que é a lei desta grelha;
    /// - **depois**: o factor sai `> 1` (o campo só afina), logo ele **engrossaria a calota
    ///   que acabou de ser pedida** — e uma calota a `s ×` o passo não é a calota. As células
    ///   pedidas são reclamadas ao valor pedido no fim.
    ///
    /// ⛔ *Sem a 1.ª metade a contagem estoura; sem a 2.ª o pedido não chega ao bico.*
    pub(crate) fn build(mesh: &Mesh, target: f32, caps: &[Cap]) -> Option<Self> {
        let curv = mesh.curvatures();
        if curv.is_empty() {
            return None;
        }
        let mut mags: Vec<f32> = curv.iter().map(|k| k.abs()).collect();
        mags.sort_by(f32::total_cmp);
        let median = mags[mags.len() / 2];
        if median <= 1.0e-9 {
            return None;
        }
        let cell = target.max(1.0e-6);
        let mut want: std::collections::BTreeMap<(i32, i32, i32), f32> =
            std::collections::BTreeMap::new();
        let pos = mesh.positions();
        let mut origin = [f32::INFINITY; 3];
        for p in pos {
            for k in 0..3 {
                origin[k] = origin[k].min(p[k]);
            }
        }
        if !origin.iter().all(|v| v.is_finite()) {
            origin = [0.0; 3];
        }
        let mut per_vertex: Vec<f32> = Vec::with_capacity(pos.len());
        let mut capped: std::collections::BTreeMap<(i32, i32, i32), f32> =
            std::collections::BTreeMap::new();
        for (v, p) in pos.iter().enumerate() {
            let k = curv.get(v).copied().unwrap_or(0.0).abs().max(1.0e-9);
            let mut h = target * (median / k).clamp(1.0 / ADAPT_RATIO, 1.0);
            let key = Self::key_of(*p, origin, cell);
            for c in caps.iter().filter(|c| c.covers(*p)) {
                h = h.min(c.step);
                let slot = capped.entry(key).or_insert(c.step);
                *slot = slot.min(c.step);
            }
            per_vertex.push(h);
            let slot = want.entry(key).or_insert(h);
            if h < *slot {
                *slot = h;
            }
        }
        // ⭐⭐⭐ **QUANTOS VÉRTICES BATEM NO PISO** — a coluna que o report de 31/08 exigiu
        // (*«dentre várias pontas uma apenas foi amputada, a menos densa em faces»*).
        //
        // ⛔ Um vértice **saturado** é um sítio onde a forma pediu mais resolução do que o
        // [`ADAPT_RATIO`] permite: a partir dali, *uma agulha mais fina recebe exactamente a
        // mesma grelha que uma mais grossa*. É a assinatura de uma ponta que morre enquanto as
        // irmãs vivem. ⚠️ `PH2D_ISO_LOG=1` imprime; ela não muda saída nenhuma.
        if std::env::var("PH2D_ISO_LOG").as_deref() == Ok("1") {
            let piso = target / ADAPT_RATIO;
            #[allow(clippy::cast_precision_loss)]
            let n = pos.len().max(1) as f32;
            #[allow(clippy::cast_precision_loss)]
            let sat = per_vertex.iter().filter(|h| **h <= piso * 1.000_01).count() as f32;
            let (mut lo, mut hi) = (f32::INFINITY, 0.0f32);
            for h in &per_vertex {
                lo = lo.min(*h);
                hi = hi.max(*h);
            }
            eprintln!(
                "[iso] grelha: alvo {target:.5} · piso {piso:.5} · h {lo:.5}..{hi:.5} · \
                 NO PISO {:.1} % ({} de {})",
                100.0 * sat / n,
                sat as usize,
                pos.len(),
            );
        }
        let mut grid = Self {
            cell,
            fallback: target,
            origin,
            want,
        };
        let s = grid.count_factor(mesh, target);
        if s.is_finite() && s > 0.0 && (s - 1.0).abs() > 1.0e-6 {
            for h in grid.want.values_mut() {
                *h *= s;
            }
            grid.fallback = target * s;
        }
        // ⭐⭐⭐ **A CALOTA É RECLAMADA** — ver o doc desta função. ⛔ Sem isto o pedido chega ao
        // bico multiplicado pelo factor de contagem, que é `> 1` por construção.
        for (key, step) in &capped {
            if let Some(h) = grid.want.get_mut(key) {
                *h = h.min(*step);
            }
        }
        Some(grid)
    }

    /// ⭐⭐⭐ **O factor que repõe o ORÇAMENTO** — `√(N_previsto / N_pedido)`, com
    /// `N = Σ_face área / h²`.
    ///
    /// ⛔⛔ **Ele mede a GRELHA, não o campo por vértice**, e a diferença não é cosmética: o
    /// [`Self::at`] leva o **mínimo dos 27 vizinhos** (para não haver degrau no limiar de
    /// colapso), logo a resposta da grelha é sistematicamente mais fina que o campo de que
    /// ela nasceu. *Normalizar uma coisa e consultar outra deixaria a inflação de pé, mais
    /// pequena.*
    ///
    /// ⚠️ **Uma passagem chega:** a escala é uniforme, e `at()` é um mínimo — logo
    /// `at_escalado ≡ s · at`, exactamente.
    fn count_factor(&self, mesh: &Mesh, target: f32) -> f32 {
        let pos = mesh.positions();
        let (mut pred, mut area) = (0.0f64, 0.0f64);
        for f in mesh.faces() {
            let v = f.verts();
            for k in 1..v.len().saturating_sub(1) {
                let (a, b, c) = (
                    pos[v[0] as usize],
                    pos[v[k] as usize],
                    pos[v[k + 1] as usize],
                );
                let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
                let w = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
                let n = [
                    u[1].mul_add(w[2], -(u[2] * w[1])),
                    u[2].mul_add(w[0], -(u[0] * w[2])),
                    u[0].mul_add(w[1], -(u[1] * w[0])),
                ];
                let tri =
                    f64::from(n[0].mul_add(n[0], n[1].mul_add(n[1], n[2] * n[2])).sqrt()) * 0.5;
                let mid = [
                    (a[0] + b[0] + c[0]) / 3.0,
                    (a[1] + b[1] + c[1]) / 3.0,
                    (a[2] + b[2] + c[2]) / 3.0,
                ];
                let h = f64::from(self.at(mid).max(1.0e-9));
                pred += tri / (h * h);
                area += tri;
            }
        }
        let want = area / f64::from(target.max(1.0e-9)).powi(2);
        if pred > 0.0 && want > 0.0 {
            #[allow(clippy::cast_possible_truncation)]
            let k = (pred / want).sqrt() as f32;
            k
        } else {
            1.0
        }
    }

    /// ⚠️ **Relativa à [`Self::origin`]** — ver o doc dela. ⛔ E os DOIS lados têm de a usar:
    /// mudar só a construção (ou só a consulta) faz as chaves nunca casarem, o [`Self::at`]
    /// cai no `fallback` constante e o campo **morre** — que se lê como perfeitamente
    /// invariante em qualquer régua que só meça dispersão. *Duas mutações sobreviveram ao
    /// gate por isso, até ele passar a exigir também que a graduação MUDE a malha.*
    #[allow(clippy::cast_possible_truncation)]
    fn key_of(p: [f32; 3], origin: [f32; 3], cell: f32) -> (i32, i32, i32) {
        (
            ((p[0] - origin[0]) / cell).floor() as i32,
            ((p[1] - origin[1]) / cell).floor() as i32,
            ((p[2] - origin[2]) / cell).floor() as i32,
        )
    }

    /// O alvo local: o **mínimo** entre a célula e as 26 vizinhas.
    pub(crate) fn at(&self, p: [f32; 3]) -> f32 {
        let (x, y, z) = Self::key_of(p, self.origin, self.cell);
        let mut best = f32::INFINITY;
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    if let Some(h) = self.want.get(&(x + dx, y + dy, z + dz)) {
                        best = best.min(*h);
                    }
                }
            }
        }
        if best.is_finite() {
            best
        } else {
            self.fallback
        }
    }
}

/// ⭐⭐⭐ **A CERCA POR SÍTIO está ligada?** — LIGADA por omissão; `PH2D_ISO_ADAPT=0` desliga.
///
/// # ⛔⛔ A tabela de recusa que estava aqui descrevia um comportamento que JÁ NÃO EXISTE
///
/// Até 2026-08-31 esta porta nascia **desligada**, com a medição *«cura a agulha e parte a
/// cadeia»* (`χ` de `1` para `−7`, bordo de `4` para `62`, `6×` o relógio). ⚠️ **O que a
/// partia era a INFLAÇÃO, não a graduação:** o campo só afinava (o tecto era `1`), logo ele
/// **acrescentava** trabalho — a malha de trabalho ia de `3 982` para `33 156` faces.
/// Com a banda simétrica e a renormalização da contagem ([`SizingGrid::build`]) o orçamento
/// fica, e a avaria desaparece.
///
/// # ⭐⭐⭐ A medição de 2026-08-31 — `Detail 0,85`, o botão de ponta a ponta
///
/// A régua da amputação é o **suporte POR PONTA** (a distância que a superfície alcança na
/// direcção de cada ápice). ⛔ *O ALCANCE é um extremo global e esconde uma ponta cortada
/// atrás de outra que sobreviveu* — na `sculpt_antes` ele **piora** enquanto as pontas
/// cortadas caem de `3` para `1`.
///
/// | peça | pontas cortadas (desl. → lig.) | furos na saída | alcance |
/// |---|---|---|---|
/// | `espinhos:6 σ=0,30` | `0/6` → `0/6` | `0` → `0` | `+2,8 %` → `+1,8 %` |
/// | ⭐ `espinhos:6 σ=0,14` | **`5/6` → `0/6`** | `0` → `0` | `+3,7 %` → `+0,3 %` |
/// | ⭐⭐⭐ `espinhos:6 σ=0,07` | `6/6` pior `−20,5 %` → **`−7,6 %`** | ⛔ `4` → ⭐ **`0`** | `−15,5 %` → ⭐ **`−3,5 %`** |
/// | ⭐⭐ `_base_sculpt` (a escultura do dono) | `3/4` pior `−41,2 %` → **`−8,4 %`** | `0` → `0` | ⭐⭐ `−41,8 %` → **`−11,1 %`** |
/// | ⭐ `sculpt_antes` | **`3/6` → `1/6`** | ⭐ `4` → **`0`** | ⚠️ `−13,6 %` → `−16,8 %` |
///
/// ⭐⭐⭐ **Cinco de cinco melhoram ou empatam nas pontas cortadas E nos furos**, e a agulha
/// mais fina — que antes saía com `χ = 1` e `4` arestas de bordo — passa a fechar (`χ = 2`,
/// zero bordo).
///
/// ⭐ **O preço de contagem:** `+7 %` a `+15 %` de faces na malha de trabalho, contra os
/// `7`–`8×` da versão anterior.
///
/// ⚠️ **A única coluna que piora numa peça é o ALCANCE da `sculpt_antes`**, e a régua por
/// ponta diz o contrário na mesma corrida (`3` cortadas → `1`): *um máximo global move-se com
/// a pior ponta, e a pior ponta mudou de identidade.*
///
/// ⇒ ⏳ **A forma FINAL desta cura não é um interruptor global:** a fase zero graduada devia
/// ser mais uma **candidata** da corrida do botão, com o `worse` a decidir por peça — e para
/// isso o `worse` precisa de uma chave de **amputação**, que ele não tem. Ver
/// `docs/3D/quad-remesh/PLANO_a_graduacao_da_ponta.md` §22-§27.
pub fn adaptive_on() -> bool {
    std::env::var("PH2D_ISO_ADAPT").as_deref() != Ok("0")
}

/// ⭐ **Quantas vezes o alvo pode encolher onde a forma aperta.**
///
/// # ⛔⛔⛔ O `4` era EMPRESTADO, e respondia a outra pergunta
///
/// Até 2026-08-31 este número era `4`, *«a mesma cerca de gradação que a
/// `ph2d_quadflow::MAX_ADAPTIVE_RATIO` declara noutra crate»*. ⚠️ **Aquela cerca é sobre a
/// GRADE DE QUADS transitar sem rasgar** (*duas células cujas escalas diferem por mais do que
/// isto deixam de ter aresta comum*); o consumidor aqui é um **remalhador de triângulos**, que
/// não tem essa restrição. ⇒ *um limite legítimo diz de que recurso ele é, e este dizia de um
/// recurso de outro subsistema* (`CLAUDE.md` §0.0).
///
/// # ⭐⭐⭐ O report que o mediu (Enio, 31/08)
///
/// *«Dentre várias pontas uma apenas foi amputada — a menos densa em faces.»* ⭐⭐ **A
/// observação dele é o mecanismo:** com o alvo da fase zero em `0,105`, o piso é `0,105/4 ≈
/// 0,026`, e uma agulha mais fina que isso **satura** — *a partir dali ela recebe exactamente a
/// mesma grelha que uma mais grossa.* Medido na peça dele: entre `8,5 %` e `14,3 %` dos
/// vértices batiam no piso.
///
/// | `ADAPT_RATIO` | vértices NO PISO | `_base_sculpt` | `sculpt_antes` | `σ=0,07` | `σ=0,14` |
/// |---|---|---|---|---|---|
/// | ⛔ `4` | `8,5 %`–`14,3 %` | `1/4` pior `−5,9 %` | `3/6` pior `−23,2 %` | `6/6` pior `−36,4 %` | `1/6` pior `−2,0 %` |
/// | ⚠️ `8` | `2,6 %` | ⛔ `1/4` pior **`−43,0 %`** | — | — | — |
/// | ⭐⭐⭐ **`16`** | ⭐ `0,2 %` | ⭐ **`0/4`** pior **`−0,4 %`** | ⭐ `2/6` pior **`−18,9 %`** | ⭐ `6/6` pior **`−11,2 %`** | `1/6` pior `−2,1 %` |
/// | `32` | `0,0 %` | `2/4` pior `−2,9 %` | — | — | — |
///
/// ⭐⭐ **O `16` é melhor ou igual nas QUATRO peças**, e a topologia é idêntica em todas as
/// células (`χ = 2`, zero bordo, zero não-manifold). ⛔ **Subi-lo é barato só porque a
/// renormalização já lá está**: sem ela, `16×` de refinamento local **multiplicaria** a malha;
/// com ela, o orçamento é o mesmo e só **muda de sítio**.
///
/// ⚠️⚠️ **E a linha do `8` é o outro achado:** com a fase zero **perfeita** a saída cortou a
/// ponta mais longa em `−43 %`, porque a corrida trocou de vencedora e o `worse` não tinha
/// **chave de amputação**. *A curva não é monótona — ela mede a selecção, não só a graduação*,
/// e é por isso que a escolha correu em quatro peças e só depois de a chave existir.
const ADAPT_RATIO: f32 = 16.0;

/// ⭐⭐⭐ **A reprojecção exige que o pé CONCORDE com a normal** — ver o uso.
///
/// ⚠️ **Lida uma vez por passe** e não por vértice: `env::var` aloca, e este laço corre sobre
/// a malha inteira em cada uma das [`crate::MAX_ROUNDS`] rondas.
///
/// # ⛔⛔⛔ MEDIDA, e NÃO ADOPTADA — ela cura esta fase e parte a seguinte
///
/// ⭐ **Ela faz exactamente o que promete.** Na peça do artista (2026-08-29) o alcance que a
/// fase zero come cai de **`−15,9 %` para `−5,7 %`** — melhor que os `−13,2 %` da ferramenta
/// de terceiros com que ele a comparou. Nas fixturas de espinhos é **inerte** onde não há
/// agulha (`σ ≥ 0,14`, saída idêntica) e ganha a `σ = 0,07` (`−12,9 % → −7,9 %`).
///
/// ⛔⛔ **E a cadeia a jusante desaba.** Medida de ponta a ponta pelo botão, na mesma peça,
/// `Detail 0,85`:
///
/// | | alcance final | `χ` | bordo | ilhas | dobras | `>60°` | relógio |
/// |---|---|---|---|---|---|---|---|
/// | ⭐ desligada (o que shipa) | `−12,4 %` | `1` | **`4`** | `1` | `76` | `2` | **`31 s`** |
/// | ⛔ ligada | ⛔ `−14,2 %` | ⛔ **`−16`** | ⛔ **`250`** | ⛔ **`5`** | ⛔ `798` | ⛔ `41` | ⛔ `79 s` |
///
/// ⚠️ **O mecanismo do estrago:** manter o vértice do seu lado guarda a agulha e deixa lá uma
/// malha **emaranhada** — a malha de trabalho passa de `3 982` para `9 458` faces com
/// valência até `23` (contra `8`). O campo cruzado e o traçado, que dependem de uma
/// triangulação bem comportada, perdem-se nela. *E o alcance FINAL até piora: a ponta
/// guardada não sobrevive à cadeia.*
///
/// ⭐ **A lição, e ela é a razão de esta função ficar:** *uma fase medida sozinha pode
/// melhorar e piorar o produto.* A cura verdadeira tem de tratar as duas ao mesmo tempo —
/// guardar a agulha **e** entregar ao campo uma malha que ele saiba ler.
pub(crate) fn facing_on() -> bool {
    std::env::var("PH2D_ISO_FACING").as_deref() == Ok("1")
}

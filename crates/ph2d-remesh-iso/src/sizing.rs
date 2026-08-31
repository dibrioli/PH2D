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
    pub(crate) fn build(mesh: &Mesh, target: f32) -> Option<Self> {
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
        for (v, p) in pos.iter().enumerate() {
            let k = curv.get(v).copied().unwrap_or(0.0).abs().max(1.0e-9);
            let h = target * (median / k).clamp(1.0 / ADAPT_RATIO, 1.0);
            let key = Self::key_of(*p, cell);
            let slot = want.entry(key).or_insert(h);
            if h < *slot {
                *slot = h;
            }
        }
        let mut grid = Self {
            cell,
            fallback: target,
            want,
        };
        let s = grid.count_factor(mesh, target);
        if s.is_finite() && s > 0.0 && (s - 1.0).abs() > 1.0e-6 {
            for h in grid.want.values_mut() {
                *h *= s;
            }
            grid.fallback = target * s;
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

    #[allow(clippy::cast_possible_truncation)]
    fn key_of(p: [f32; 3], cell: f32) -> (i32, i32, i32) {
        (
            (p[0] / cell).floor() as i32,
            (p[1] / cell).floor() as i32,
            (p[2] / cell).floor() as i32,
        )
    }

    /// O alvo local: o **mínimo** entre a célula e as 26 vizinhas.
    pub(crate) fn at(&self, p: [f32; 3]) -> f32 {
        let (x, y, z) = Self::key_of(p, self.cell);
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
/// ⚠️ **É a mesma cerca de gradação que a [`ph2d_quadflow::MAX_ADAPTIVE_RATIO`] declara**
/// noutra crate (*duas células cujas escalas diferem por mais do que isto deixam de ter
/// aresta comum*), e o número aqui é o mesmo `4` — a agulha recebe até `4×` mais resolução
/// linear que o corpo.
const ADAPT_RATIO: f32 = 4.0;

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

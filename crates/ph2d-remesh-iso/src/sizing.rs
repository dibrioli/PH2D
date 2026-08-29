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
    /// escala que a `ph2d_quadflow::ScaleField` usa. ⚠️ **O tecto é `1`**: esta grelha nunca
    /// grosseira, então não pode piorar nenhuma região que o laço já resolveu.
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
        Some(Self {
            cell,
            fallback: target,
            want,
        })
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

/// ⭐⭐⭐ **A CERCA POR SÍTIO está ligada?** — `PH2D_ISO_ADAPT=1` liga.
///
/// # ⭐ Ela CURA a agulha, e a medição é dramática
///
/// Alcance perdido pela fase zero, na fixtura de espinhos (sem → com):
///
/// | `σ` | sem | com |
/// |---|---|---|
/// | `0,30` | `+0,1 %` | `+0,3 %` |
/// | `0,20` | `−0,9 %` | ⭐ `+0,8 %` |
/// | `0,14` | `−1,6 %` | ⭐ `+0,8 %` |
/// | `0,10` | `−5,8 %` | ⭐ **`−0,8 %`** |
/// | `0,07` | `−12,9 %` | ⭐ **`−1,3 %`** |
/// | `0,05` | ⛔ `−15,8 %` | ⭐⭐⭐ **`−0,8 %`** |
///
/// ⭐ E a topologia da malha de trabalho fica **perfeita** (`χ = 2`, zero bordo, zero
/// não-manifold, valência máxima `10`). *A agulha sobrevive.*
///
/// # ⛔⛔⛔ E PARTE A CADEIA A JUSANTE — a mesma lei do [`facing_on`], um nível mais fundo
///
/// Medida de ponta a ponta **pelo botão**, na peça do artista, `Detail 0,85`:
///
/// | | alcance final | `χ` | bordo | não-manif. | dobras | relógio |
/// |---|---|---|---|---|---|---|
/// | ⭐ desligada (o que shipa) | `−12,4 %` | `1` | **`4`** | `0` | `76` | **`27,8 s`** |
/// | ⛔ ligada | ⛔ `−17,3 %` | ⛔ `−7` | ⛔ **`62`** | ⛔ `2` | `101` | ⛔ **`167 s`** |
///
/// ⚠️ **O mecanismo é o mesmo das duas vezes:** a malha de trabalho passa de `3 982` para
/// `33 156` faces e deixa de ser **isotrópica**; o campo cruzado, o traçado e o mapa — que
/// dependem de uma triangulação bem comportada — perdem-se nela, e o alcance FINAL até piora.
///
/// ⭐⭐⭐ **A lei, agora confirmada DUAS vezes:** *uma fase medida sozinha pode melhorar e
/// piorar o produto.* ⇒ a cadeia inteira tem de ser consciente do sizing, e isso é a wave do
/// **factor de escala conforme** que a `sizing_field` do shell já nomeia — não uma cerca só
/// no F1.
pub(crate) fn adaptive_on() -> bool {
    std::env::var("PH2D_ISO_ADAPT").as_deref() == Ok("1")
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

//! **A FRAÇÃO DO PIXEL COBERTA POR UMA UNIÃO DE SEMI-PLANOS** — geometria pura, sem saber o que é
//! um traço.
//!
//! ## Por que ela existe
//!
//! O anti-aliasing do percurso era `edge = clamp(0,5 − sd, 0, 1)`: o filtro-caixa **1-D ao longo da
//! normal** da silhueta. Ele é EXATO quando a borda é paralela a um eixo do pixel e **só então** —
//! a área de um quadrado unitário cortada por um semi-plano depende do ÂNGULO da borda, e a 45° a
//! resposta certa é `(s+1)²/2` (com `s = d√2`), não a rampa. Medido no produto, num flanco reto
//! longe de qualquer tampa: **0,00/255 a 0° e 90° · 5,76 a 15° · 8,54 a 30° · 9,72 a 45°**.
//!
//! E onde DUAS bordas atravessam o mesmo pixel — uma quina, um cruzamento — a distância com sinal
//! sozinha não determina área nenhuma: o `min` dos dois SDFs diz que o centro está *em cima* da
//! fronteira enquanto a união cobre ¾ do pixel. Medido: **63,75/255**, que é exatamente ¼.
//!
//! ## A resposta, e por que ela apaga os dois casos de uma vez
//!
//! O conjunto NÃO coberto é a interseção dos semi-planos de FORA. Interseção de semi-planos com um
//! quadrado é um **polígono convexo**, então a área sai de um recorte de Sutherland-Hodgman + a
//! fórmula do sapateiro — exata, sem caso especial, sem transcendental (HR-5), e com o mesmo
//! mecanismo respondendo a *um* plano (o ângulo) e a *dois* (a quina). É a mesma decomposição que o
//! empuxo da linha de física usa para "quanto deste corpo está dentro da água".
//!
//! ⚠️ **A aproximação que sobra é a CURVATURA, e ela é deliberada:** cada passagem entra como o
//! plano TANGENTE à sua silhueta no ponto mais próximo. Dentro de um pixel de 1×1, a borda de uma
//! cápsula de raio ≥ 1 px é reta a menos de `1/(8r)`; abaixo disso o traço já é sub-pixel e a lei da
//! tinta (`τ`) domina o que se vê.

/// Um semi-plano de FORA, em coordenadas **relativas ao centro do pixel**: fora é `n·u + sd > 0`.
///
/// `n` aponta para fora (do ponto mais próximo na silhueta em direção ao pixel) e `sd` é a distância
/// com sinal — negativa dentro. Em `u = 0` (o centro) o predicado vira `sd > 0`, que é exatamente o
/// que "o centro está fora" significa.
#[derive(Copy, Clone, Debug)]
pub(crate) struct OutsidePlane {
    pub n: [f32; 2],
    pub sd: f32,
}

/// ⚠️ **O alcance de um plano que ainda corta o quadrado.** Sobre `u ∈ [−½, ½]²` vale
/// `|n·u| ≤ √2/2`, então um plano com `sd ≥ √2/2` mantém o quadrado inteiro (não corta nada) e um
/// com `sd ≤ −√2/2` o elimina (cobertura cheia). Fora desta faixa o plano é **descartável**, e é
/// isso que também descarta o caso degenerado — um pixel exatamente sobre o eixo da cápsula tem
/// `dist = 0` e normal indefinida, mas ali `sd = −r`, muito além do alcance.
pub(crate) const PLANE_REACH: f32 = core::f32::consts::FRAC_1_SQRT_2;

/// Quantos planos um pixel carrega. Quando sobra, ficam os de **menor `sd`** — são os que mais
/// recortam, logo os que mais aproximam a resposta cheia (recortar menos SUPERESTIMA o descoberto).
///
/// ⚠️ **O valor é MEDIDO, e as duas metades da medição estão aqui** (§0.0). *Quantos o desenho
/// usa:* a quina mede **2**, e o pior pixel de um zigue-zague de passo sub-pixel — a única figura
/// em que consegui pôr três bordas perto do mesmo pixel — também mede **2**
/// (`measure_what_the_third_and_fourth_plane_buy`). *Quanto cada vaga custa:* no device, a 200
/// traços e 1080p, o frame do percurso vale **3,46 ms com 2 · 3,65 com 3 · 4,71 com 4** — o degrau
/// de 3 para 4 é o array de recorte deixando de caber em registrador. Três fica **acima da maior
/// contagem que consegui produzir e abaixo do degrau**; a lei antiga, para referência, custava 2,72.
///
/// ⚠️ Truncar SEMPRE erra para menos cobertura, nunca para mais — e ainda assim para mais que a lei
/// antiga, que enxergava **um** plano só.
pub(crate) const MAX_PLANES: usize = 3;

/// O conjunto de planos que o percurso acumula enquanto varre as passagens.
#[derive(Copy, Clone, Debug, Default)]
pub(crate) struct PlaneSet {
    planes: [Option<OutsidePlane>; MAX_PLANES],
}

impl PlaneSet {
    /// Oferece um plano ao conjunto. Fora do [`PLANE_REACH`] ele é ignorado; dentro, entra ordenado
    /// por `sd` crescente e empurra o maior para fora quando o conjunto está cheio.
    ///
    /// ⚠️ **A ordem é por `sd`, não de chegada** — assim o descarte é o dos planos que menos
    /// recortam, e o resultado não depende da ordem em que as passagens aparecem na lista.
    pub fn offer(&mut self, plane: OutsidePlane) {
        if plane.sd >= PLANE_REACH {
            return;
        }
        let mut entrando = plane;
        for slot in &mut self.planes {
            match slot {
                None => {
                    *slot = Some(entrando);
                    return;
                }
                Some(atual) if entrando.sd < atual.sd => core::mem::swap(atual, &mut entrando),
                Some(_) => {}
            }
        }
    }

    /// Quantos planos de fato alcançam o pixel — só a sonda que mede o teto pergunta isto.
    #[cfg(test)]
    #[must_use]
    pub fn len(&self) -> usize {
        self.planes.iter().flatten().count()
    }

    /// A fração do pixel COBERTA pela união dos semi-planos complementares.
    ///
    /// Recorta o quadrado unitário por cada plano de fora, mede o que sobra (o descoberto) e devolve
    /// o complemento. Sem planos o pixel está descoberto ⇒ **0**.
    ///
    /// ⚠️ **Os dois atalhos NÃO são aproximações — saem da mesma aritmética do [`PLANE_REACH`]:**
    /// sobre o quadrado vale `|n·u| ≤ √2/2`, então *nenhum plano em alcance* ⇒ nada coberto, e *o
    /// plano mais recortante já engole o quadrado* ⇒ tudo coberto.
    ///
    /// ⚠️ **E eles quase não pagam no DEVICE, o que é um fato sobre SIMD e não sobre a lei:** medido,
    /// 4,91 → 4,71 ms por frame (4%). Os pixels de fronteira são espalhados, então quase todo warp
    /// tem pelo menos um — e um atalho divergente não economiza tempo se alguém no warp toma o
    /// caminho longo. Ficam por serem exatos e de graça, não por serem a otimização.
    #[must_use]
    pub fn coverage(&self) -> f32 {
        match self.planes[0] {
            None => return 0.0,
            Some(p) if p.sd <= -PLANE_REACH => return 1.0,
            Some(_) => {}
        }
        // ⚠️ O buffer é dimensionado pelo [`POLY_CAP`], **não por um `8` escrito à mão** — a versão
        // com o literal compilava e mentia assim que o [`MAX_PLANES`] mudava de valor.
        let mut poly = [[0.0_f32; 2]; POLY_CAP];
        poly[..4].copy_from_slice(&[[-0.5, -0.5], [0.5, -0.5], [0.5, 0.5], [-0.5, 0.5]]);
        let mut n = 4_usize;
        for plane in self.planes.iter().flatten() {
            n = clip(&mut poly, n, plane);
            if n == 0 {
                // O plano engoliu o quadrado: nada descoberto, cobertura cheia.
                return 1.0;
            }
        }
        (1.0 - shoelace(&poly[..n])).clamp(0.0, 1.0)
    }
}

/// A capacidade do buffer de recorte: um convexo de `k` lados cortado por uma reta tem no máximo
/// `k + 1` lados, e são no máximo [`MAX_PLANES`] cortes sobre o quadrado.
const POLY_CAP: usize = 4 + MAX_PLANES;

/// Sutherland-Hodgman contra **um** semi-plano, mantendo o lado de FORA (`n·u + sd ≥ 0`).
fn clip(poly: &mut [[f32; 2]; POLY_CAP], n_in: usize, plane: &OutsidePlane) -> usize {
    let dentro = |q: &[f32; 2]| plane.n[0] * q[0] + plane.n[1] * q[1] + plane.sd;
    let entrada: [[f32; 2]; POLY_CAP] = *poly;
    let mut out = 0_usize;
    for i in 0..n_in {
        let (a, b) = (entrada[i], entrada[(i + 1) % n_in]);
        let (da, db) = (dentro(&a), dentro(&b));
        if da >= 0.0 {
            poly[out] = a;
            out += 1;
        }
        // Só cruza quando os sinais diferem — e `da != db` é garantido nesse ramo, então a divisão
        // é segura sem épsilon.
        if (da >= 0.0) != (db >= 0.0) {
            let t = da / (da - db);
            poly[out] = [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t];
            out += 1;
        }
    }
    out
}

/// A área (positiva) de um polígono convexo, pela fórmula do sapateiro.
fn shoelace(poly: &[[f32; 2]]) -> f32 {
    let mut duas_vezes = 0.0_f32;
    for i in 0..poly.len() {
        let (a, b) = (poly[i], poly[(i + 1) % poly.len()]);
        duas_vezes += a[0] * b[1] - b[0] * a[1];
    }
    (duas_vezes * 0.5).abs()
}

#[cfg(test)]
#[path = "pixel_area_tests.rs"]
mod tests;

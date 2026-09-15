//! Brush-dab **flatten + rotate** — the per-pixel footprint deformation shared by the falloff
//! envelope, the Shape silhouette, and the View-mapped Grain, so they flatten + rotate TOGETHER while
//! each slot keeps its own relative Size/Offset/Angle on top (Procreate's Shape panel; Enio
//! 2026-06-26). Transcendental-free (HR-5): the rotation reuses the baked [`crate::texture::rotate_by_degrees`]
//! unit vector, so the result is bit-identical on every platform.

use crate::texture::rotate_by_degrees;

/// Largest **Flatten** value: the minor axis shrinks to `1 - DAB_FLATTEN_MAX` of the major, never to
/// zero (a true line would be degenerate / un-paintable).
pub const DAB_FLATTEN_MAX: f32 = 0.95;

/// ⭐⭐⭐ **A CURVATURA DA ARTE DEBAIXO DO DAB** — os termos de grau `2` e `3` da pegada.
///
/// # Porque ela existe (medido, 2026-09-14 — [`crate::canvas_warp`] e a sonda `sprite_mesh_warp_probe`)
///
/// O kernel avalia `t = |M · p|` por texel com `M` **linear**, e uma dobra não é linear: sobre arte
/// dobrada a marca sai oval mesmo com a deformação local perfeitamente medida. ⛔ **E não é
/// facetagem** — refinar a malha de `32` para `8 192` triângulos deixa o desvio onde estava
/// (`1,050`/`1,107`). O que sobra é a **curvatura da dobra dentro do próprio dab**, e por isso ela
/// cresce com o RAIO do pincel:
///
/// | raio do dab | `g = 1` (só a elipse) | `g = 2` | `g = 3` |
/// |---|---|---|---|
/// | `0,06` | `1,054` | `1,010` | `1,008` |
/// | `0,125` | `1,118` | `1,021` | `1,005` |
/// | `0,20` | `1,197` | `1,055` | **`1,006`** |
///
/// ⇒ o grau `3` leva o pior caso a `1,006`, abaixo do que a métrica distingue. É esse.
///
/// ⚠️ **Os monómios são avaliados no ponto JÁ RODADO para o referencial da elipse** — é isso que
/// deixa a [`FootprintDeform::rotated_by`] continuar a ser uma composição de rotores sem mexer num
/// único coeficiente: o giro entra pelo ARGUMENTO.
///
/// ⛔ **Plana ⇒ o caminho de sempre, AO BIT** — e a prova de mutação diz *porquê*, que não é o que
/// eu tinha escrito aqui: somar sete zeros é **exacto** em IEEE, logo apagar o atalho do
/// [`FootprintDeform::apply`] **sobrevive** a todos os gates de identidade. ⇒ *o atalho é uma cerca
/// de CUSTO, nunca de bits* — ele tira `14` multiplicações-e-soma por texel de toda pincelada do
/// app sobre arte que não está dobrada, que é a esmagadora maioria delas.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FootprintCurve {
    /// Uma linha por eixo de saída; as colunas são `x²`, `x·y`, `y²`, `x³`, `x²·y`, `x·y²`, `y³`.
    rows: [[f32; 7]; 2],
}

impl Default for FootprintCurve {
    fn default() -> Self {
        Self::flat()
    }
}

impl FootprintCurve {
    /// Sem curvatura — a pegada é a elipse de sempre.
    #[must_use]
    pub const fn flat() -> Self {
        Self {
            rows: [[0.0; 7]; 2],
        }
    }

    /// Os sete coeficientes de cada eixo, na ordem `x²`, `x·y`, `y²`, `x³`, `x²·y`, `x·y²`, `y³`.
    #[must_use]
    pub const fn from_rows(rows: [[f32; 7]; 2]) -> Self {
        Self { rows }
    }

    /// Ver [`Self::from_rows`].
    #[must_use]
    pub const fn rows(self) -> [[f32; 7]; 2] {
        self.rows
    }

    /// Nenhum coeficiente ⇒ o caminho linear de sempre.
    #[must_use]
    pub fn is_flat(self) -> bool {
        self.rows.iter().flatten().all(|c| *c == 0.0)
    }

    /// Acrescenta os termos de grau `2` e `3` avaliados em `r` (o ponto no referencial da elipse).
    #[inline]
    #[must_use]
    fn add_to(self, base: [f32; 2], r: [f32; 2]) -> [f32; 2] {
        let (x, y) = (r[0], r[1]);
        let (xx, xy, yy) = (x * x, x * y, y * y);
        let m = [xx, xy, yy, xx * x, xx * y, xy * y, yy * y];
        let mut out = base;
        for (o, row) in out.iter_mut().zip(self.rows.iter()) {
            for (c, mk) in row.iter().zip(m.iter()) {
                *o += c * mk;
            }
        }
        out
    }
}

/// Baked flatten + rotate, applied to a footprint-relative unit coord. [`Self::apply`] un-rotates by
/// the dab angle into the ellipse frame, then stretches the minor (post-rotation vertical) axis by
/// `1 / (1 - flatten)`, so a round footprint reads as a rotated ellipse. Identity at flatten 0 / angle 0.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FootprintDeform {
    cos: f32,
    sin: f32,
    inv_minor: f32,
    /// ⭐ A curvatura da arte debaixo do dab ([`FootprintCurve`]). Plana em tudo o que não é arte
    /// deformada — e aí esta struct comporta-se AO BIT como antes de ela existir.
    curve: FootprintCurve,
}

impl FootprintDeform {
    /// The identity (no flatten, no rotation) — byte-identical to the round footprint.
    #[must_use]
    pub fn identity() -> Self {
        Self {
            cos: 1.0,
            sin: 0.0,
            inv_minor: 1.0,
            curve: FootprintCurve::flat(),
        }
    }

    /// Bake from a `flatten` (`0..1`, clamped to [`DAB_FLATTEN_MAX`]) and a whole-degree dab angle. A
    /// rotation with no flatten still rotates the sampled pattern (the falloff is rotation-invariant).
    #[must_use]
    pub fn new(flatten: f32, angle_deg: u16) -> Self {
        let [cos, sin] = rotate_by_degrees(angle_deg);
        let f = flatten.clamp(0.0, DAB_FLATTEN_MAX);
        Self {
            cos,
            sin,
            inv_minor: 1.0 / (1.0 - f),
            curve: FootprintCurve::flat(),
        }
    }

    /// Whether this deform changes a footprint at all (any flatten or any rotation) — lets hot paths
    /// skip the transform entirely for a plain round dab.
    #[must_use]
    pub fn is_identity(self) -> bool {
        self.inv_minor == 1.0 && self.cos == 1.0 && self.sin == 0.0 && self.curve.is_flat()
    }

    /// ⭐⭐⭐ **A pegada com a CURVATURA da arte por baixo dela** ([`FootprintCurve`]).
    ///
    /// ⚠️ **Quem a põe é [`crate::canvas_warp::warped_dab`]**, e só ele: os coeficientes vivem no
    /// referencial da elipse e só lá são o que dizem ser. ⛔ Uma segunda porta a escrevê-los seria
    /// a segunda resposta à pergunta que esta wave existe para ter numa só.
    #[must_use]
    pub fn with_curve(self, curve: FootprintCurve) -> Self {
        Self { curve, ..self }
    }

    /// A curvatura que esta pegada carrega.
    #[must_use]
    pub fn curve(self) -> FootprintCurve {
        self.curve
    }

    /// Compose an extra rotation `rotor` (a unit vector `[cos, sin]`) onto this deform's frame — the
    /// per-dab **Jitter Rotate** spins the whole footprint (flatten + the falloff/Shape/View-Grain it
    /// drives) by a random angle each dab, on top of the brush's own dab angle. `rotor = [1, 0]` ⇒
    /// unchanged. Transcendental-free (complex multiply of the two rotors); the flatten is preserved.
    ///
    /// ⭐ **A curvatura viaja SEM SER TOCADA, e é por desenho:** os monómios da [`FootprintCurve`]
    /// são avaliados no ponto já rodado para o referencial da elipse, logo o giro entra por dentro
    /// do argumento. ⛔ Guardá-los no referencial do texel obrigaria a rodar sete coeficientes por
    /// eixo aqui — a conta que esta escolha apaga.
    #[must_use]
    pub fn rotated_by(self, rotor: [f32; 2]) -> Self {
        Self {
            cos: self.cos * rotor[0] - self.sin * rotor[1],
            sin: self.cos * rotor[1] + self.sin * rotor[0],
            inv_minor: self.inv_minor,
            curve: self.curve,
        }
    }

    /// Apply to a footprint unit coord `[u, v]` (pixel offset ÷ radius): rotate by `-angle` into the
    /// ellipse frame, then stretch the minor axis. The falloff reads [`Self::falloff_t`]; the Shape /
    /// Grain samplers feed `apply(f)` into their own Size/rotation/offset.
    #[must_use]
    pub fn apply(self, p: [f32; 2]) -> [f32; 2] {
        if self.is_identity() {
            return p;
        }
        // Rotate by −angle (the transpose of the `(cos, sin)` basis), then stretch the minor axis.
        let ru = p[0] * self.cos + p[1] * self.sin;
        let rv = -p[0] * self.sin + p[1] * self.cos;
        let lin = [ru, rv * self.inv_minor];
        // ⛔ **O atalho é de CUSTO, e a mutação disse-o:** apagá-lo deixa TODOS os gates verdes —
        // somar sete zeros é exacto em IEEE. O que ele tira são `14` multiplicações-e-soma por
        // texel em toda pincelada sobre arte não dobrada. *Não confundir com a cerca de bits do
        // `is_identity`, essa medida por mutação VERMELHA.*
        if self.curve.is_flat() {
            return lin;
        }
        self.curve.add_to(lin, [ru, rv])
    }

    /// A fração do eixo MAIOR que o eixo MENOR mede (`1` = redondo, `1 − Flatten` achatado).
    ///
    /// É o número de que a admissibilidade da LUT do filme é feita ([`crate::height_film::FilmLut`]):
    /// o erro da expansão escala com a **CURVATURA** da silhueta, e a curvatura é governada pelo menor
    /// raio local — que num bico achatado é `raio × minor`, não `raio`. Medido: uma elipse de
    /// `minor = 0,45` erra **6×** a redonda no mesmo raio, e `1/0,45² = 4,9`.
    #[must_use]
    pub fn minor_fraction(self) -> f32 {
        1.0 / self.inv_minor
    }

    /// ⭐⭐⭐ **O CONTORNO do dab** — o ponto da fronteira da pegada no parâmetro `t ∈ [0, 1)`, em
    /// unidades de RAIO.
    ///
    /// ⛔⛔ **Ele existe para o anel do cursor não reconstruir a elipse por fora** (item 1 da fila do
    /// esqueleto, 2026-09-14). O anel desenhava `(cos θ, m·sin θ)` rodado pelo rotor vivo — a mesma
    /// conta, escrita noutra crate — e por isso continuava a mostrar a forma de REPOUSO quando a
    /// pegada passou a carregar a deformação da arte. *Uma lei escrita em dois sítios ainda não é
    /// uma lei.*
    ///
    /// ⭐ **E a amarra é demonstrável, não prometida:** por construção `falloff_t` deste ponto é
    /// exactamente `1` — a fronteira da pegada É a curva de nível que o amostrador usa. Há gate.
    #[must_use]
    pub fn outline_at(self, t: f32) -> [f32; 2] {
        let (sen, cos) = (t * std::f32::consts::TAU).sin_cos();
        let menor = sen * self.minor_fraction();
        let elipse = [
            cos * self.cos - menor * self.sin,
            cos * self.sin + menor * self.cos,
        ];
        if self.curve.is_flat() {
            return elipse;
        }
        // ⭐ Com curvatura a curva de nível **não é uma elipse**, e o anel tem de a percorrer — não
        // uma elipse que a tinta não pinta. A direcção é a mesma; o RAIO resolve-se contra o próprio
        // amostrador, que é o que mantém a amarra `falloff_t(outline_at(t)) == 1` a ser uma medição
        // e não uma promessa.
        //
        // ⚠️ **A cerca é a estrelaridade:** para uma dobra forte a ponto de a curva de nível deixar
        // de ser vista da origem, a bissecção devolve o primeiro cruzamento — o anel fica MENOR que
        // a tinta em vez de partido. É a degradação certa para desenhar um contorno.
        let (mut lo, mut hi) = (0.0_f32, 4.0_f32);
        for _ in 0..24 {
            let mid = 0.5 * (lo + hi);
            if self.falloff_t(elipse[0] * mid, elipse[1] * mid) < 1.0 {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        let s = 0.5 * (lo + hi);
        [elipse[0] * s, elipse[1] * s]
    }

    /// ⭐⭐⭐ **As duas LINHAS do mapa LINEAR da pegada** (rodar + achatar), SEM a curvatura.
    ///
    /// ⛔⛔ **Ela existe porque `apply` deixou de ser linear, e quem publica uma matriz não pode
    /// continuar a tirá-la dos vectores da base.** `apply([1,0])` e `apply([0,1])` só determinam
    /// uma matriz enquanto o mapa for linear — com curvatura eles trazem os monómios avaliados nos
    /// versores dentro, e o carimbo do device sairia com uma deformação que ninguém autorou.
    /// *A premissa que era gate virou porta.*
    #[must_use]
    pub fn linear_rows(self) -> [[f32; 2]; 2] {
        [
            [self.cos, self.sin],
            [-self.sin * self.inv_minor, self.cos * self.inv_minor],
        ]
    }

    /// ⭐ A curvatura re-escrita na coordenada CRUA da pegada (o `p` que o kernel recebe), em vez
    /// do ponto já rodado em que ela é guardada.
    ///
    /// ⚠️ **É para quem não sabe rodar** — o shader do device recebe uma matriz e os monómios, e
    /// pedir-lhe que reconstrua o referencial da elipse obrigá-lo-ia a inverter o achatamento por
    /// texel. ⛔ A conversão é a MESMA máquina da composição
    /// ([`super::canvas_warp_curve::compoe`], com a saída neutra e a entrada rodada ao contrário),
    /// e não uma segunda escrita da mesma álgebra.
    #[must_use]
    pub fn curve_in_input_frame(self) -> [[f32; 7]; 2] {
        const I: [[f32; 2]; 2] = [[1.0, 0.0], [0.0, 1.0]];
        super::canvas_warp_curve::compoe(self.curve.rows(), I, I, 1.0, [self.cos, -self.sin]).rows()
    }

    /// ⭐⭐⭐ **A pegada é AMOSTRÁVEL?** — a cerca que impede uma dobra violenta de sair PIOR do que
    /// não corrigir nada.
    ///
    /// ⛔⛔ **Este módulo já pagou este erro uma vez** (a W11b: *«com um pincel GRANDE sobre uma
    /// malha grossa, a correcção deixava a marca MENOS redonda do que não corrigir nada»*). Uma
    /// curvatura grande demais deixa o mapa de **dobrar sobre si mesmo** dentro do próprio dab: o
    /// `|apply|` deixa de crescer ao longo de um raio, a curva de nível deixa de ser vista da
    /// origem, e tanto o amostrador como o anel passam a ler o cruzamento errado. Medido: numa
    /// dobra em que o termo de grau `2` vale `61 %` do linear, a marca sai a `0,994` da elipse
    /// autorada **contra `0,611` sem correcção nenhuma**.
    ///
    /// ⇒ o critério é a **monotonia radial**: ao longo de cada direcção, `|apply|` tem de crescer
    /// até ao bordo do que o kernel varre. ⚠️ **O recurso é nomeado e é geométrico** — não é uma
    /// margem de segurança: abaixo dele a fronteira é única, acima dela não existe resposta certa
    /// para «qual dos cruzamentos é o bordo».
    ///
    /// ⚠️ **`1,2` é o alcance do que o kernel avalia**, não um palpite: o dab varre a caixa do seu
    /// raio, e o canto dela está a `√2`; medir até lá pediria monotonia onde a tinta já é zero.
    #[must_use]
    pub fn is_sampleable(self) -> bool {
        if self.curve.is_flat() {
            return true;
        }
        const DIRECCOES: usize = 24;
        const PASSOS: usize = 8;
        const ALCANCE: f32 = 1.2;
        for k in 0..DIRECCOES {
            let t = (k as f32) * std::f32::consts::TAU / (DIRECCOES as f32);
            let (sen, cos) = t.sin_cos();
            let mut anterior = 0.0_f32;
            for j in 1..=PASSOS {
                let s = ALCANCE * (j as f32) / (PASSOS as f32);
                let d = self.falloff_t(cos * s, sen * s);
                // ⚠️ O `is_nan` é explícito de propósito: um `NaN` num coeficiente tem de LEVAR à
                // elipse, e `NaN <= x` é `false` — a forma curta deixava-o passar.
                if d.is_nan() || d <= anterior {
                    return false;
                }
                anterior = d;
            }
        }
        true
    }

    /// The deformed radial distance `length(apply([u, v]))` — the falloff index for an elliptical dab.
    /// At identity this is the plain `sqrt(u² + v²)`. Rotation alone preserves it (a circle is
    /// rotation-invariant); only the flatten makes it elliptical.
    #[must_use]
    pub fn falloff_t(self, u: f32, v: f32) -> f32 {
        let d = self.apply([u, v]);
        (d[0] * d[0] + d[1] * d[1]).sqrt()
    }
}

#[cfg(test)]
mod outline_tests {
    use super::*;

    /// ⭐⭐⭐ **O CONTORNO É A CURVA DE NÍVEL DO AMOSTRADOR** — `falloff_t` da fronteira é `1`.
    ///
    /// ⛔ É esta a amarra que impede o anel do cursor de desenhar uma elipse que a tinta não pinta:
    /// o anel percorre [`FootprintDeform::outline_at`] e o motor lê [`FootprintDeform::falloff_t`],
    /// e as duas só podem discordar se esta asserção cair.
    ///
    /// ⚠️ O corpus varre achatamento **e** ângulo: com `flatten = 0` a elipse é um círculo e
    /// qualquer contorno passa — *um corpus redondo não mede a forma*.
    #[test]
    fn the_outline_is_the_sampler_level_set() {
        let mut casos = 0;
        for flatten in [0.0_f32, 0.2, 0.5, DAB_FLATTEN_MAX] {
            for angle in [0_u16, 17, 90, 233] {
                for curve in CURVAS {
                    let fp = FootprintDeform::new(flatten, angle).with_curve(curve);
                    for k in 0..64 {
                        let p = fp.outline_at(k as f32 / 64.0);
                        let t = fp.falloff_t(p[0], p[1]);
                        casos += 1;
                        assert!(
                            (t - 1.0).abs() < 1e-4,
                            "com flatten {flatten}, ângulo {angle}° e curvatura {curve:?}, o \
                             contorno em {k}/64 tem falloff {t} — ele não é a fronteira que o \
                             amostrador usa"
                        );
                    }
                }
            }
        }
        assert_eq!(casos, 4 * 4 * CURVAS.len() * 64, "o corpus mudou de tamanho");
    }

    /// O corpus de curvaturas: a plana (o caminho de sempre) e três dobras reais — uma quadrática
    /// pura, uma cúbica pura e uma mista. ⚠️ **Sem a plana lá dentro estes gates deixariam de medir
    /// a inércia; sem as outras três eles passariam sobre uma pegada que ignora a curvatura.**
    const CURVAS: [FootprintCurve; 4] = [
        FootprintCurve::flat(),
        FootprintCurve::from_rows([[0.25, 0.0, -0.15, 0.0, 0.0, 0.0, 0.0], [0.0; 7]]),
        FootprintCurve::from_rows([[0.0; 7], [0.0, 0.0, 0.0, 0.18, -0.09, 0.0, 0.12]]),
        FootprintCurve::from_rows([
            [0.11, -0.07, 0.05, 0.04, 0.0, -0.03, 0.0],
            [-0.06, 0.13, 0.0, 0.0, 0.07, 0.0, -0.05],
        ]),
    ];

    /// ⭐⭐⭐ **PLANA ⇒ O CAMINHO DE SEMPRE, AO BIT** — a metade que diz que esta wave não tocou em
    /// nenhuma pincelada sobre arte não deformada.
    ///
    /// ⛔ **E ela é um `if`, não uma esperança:** somar sete monómios a zero devolve o mesmo número
    /// *quase* sempre — `x + 0.0` é exacto em IEEE, mas a ORDEM da soma e o `f32` do produto
    /// intermédio não são coisas que um gate deva confiar. O atalho existe e este gate prova-o.
    #[test]
    fn a_flat_curve_is_the_footprint_that_shipped_bit_for_bit() {
        for flatten in [0.0_f32, 0.2, 0.5, DAB_FLATTEN_MAX] {
            for angle in [0_u16, 17, 90, 233] {
                let nu = FootprintDeform::new(flatten, angle);
                let com = nu.with_curve(FootprintCurve::flat());
                assert_eq!(nu, com, "pôr a curvatura PLANA mudou a pegada");
                for k in 0..32 {
                    let t = k as f32 / 32.0;
                    let p = [(t * 6.0).cos() * 0.7, (t * 6.0).sin() * 0.7];
                    assert_eq!(
                        nu.apply(p),
                        com.apply(p),
                        "a pegada plana divergiu da de sempre em {p:?}"
                    );
                    assert_eq!(nu.outline_at(t), com.outline_at(t), "o contorno divergiu em {t}");
                }
            }
        }
    }

    /// ⭐⭐⭐ **A CURVATURA CHEGA A QUEM AMOSTRA** — e este é o gate que mata o atalho errado.
    ///
    /// ⛔⛔ Uma pegada **sem rotação e sem achatamento** mas COM curvatura lê-se, nos três campos
    /// antigos, exactamente como a identidade. Se o [`FootprintDeform::is_identity`] continuasse a
    /// perguntar só por eles, o `apply` devolvia `p` e a curvatura **evaporava em silêncio** — sem
    /// erro de compilação, sem gate vermelho, e com a foto do dono a repetir-se.
    #[test]
    fn a_curved_footprint_is_never_mistaken_for_the_identity() {
        let curva = CURVAS[3];
        let fp = FootprintDeform::new(0.0, 0).with_curve(curva);
        assert!(
            !fp.is_identity(),
            "uma pegada com curvatura declarou-se identidade — o `apply` vai devolver `p`"
        );
        let p = [0.6_f32, 0.35];
        let reta = FootprintDeform::new(0.0, 0).apply(p);
        let dobrada = fp.apply(p);
        let d = (reta[0] - dobrada[0]).hypot(reta[1] - dobrada[1]);
        assert!(
            d > 1e-3,
            "a curvatura não chegou ao amostrador (desvio {d}) — ela foi engolida pelo atalho"
        );
    }

    /// ⭐ **E ela SOBREVIVE ao giro do dab** (o *Jitter Rotate* e o rumo vivo do traço).
    ///
    /// ⛔ **A metade anti-vácuo é a segunda asserção:** largar `curve` no
    /// [`FootprintDeform::rotated_by`] deixa a primeira passar — uma pegada sem curvatura também
    /// roda. O que a mutação apaga é a curva de nível deixar de ser uma elipse, e é isso que se mede.
    #[test]
    fn the_curvature_survives_the_dab_spin() {
        let fp = FootprintDeform::new(0.3, 40).with_curve(CURVAS[3]);
        let rodada = fp.rotated_by([0.6, 0.8]);
        assert!(
            !rodada.curve().is_flat(),
            "o giro do dab deitou fora a curvatura da arte"
        );
        // A curva de nível da rodada não pode ser uma elipse: numa elipse os raios opostos são
        // IGUAIS, e uma dobra quebra essa simetria.
        let (mut pior, mut casos) = (0.0_f32, 0);
        for k in 0..32 {
            let a = rodada.outline_at(k as f32 / 64.0);
            let b = rodada.outline_at(k as f32 / 64.0 + 0.5);
            let (ra, rb) = (a[0].hypot(a[1]), b[0].hypot(b[1]));
            pior = pior.max((ra - rb).abs());
            casos += 1;
        }
        assert_eq!(casos, 32);
        assert!(
            pior > 1e-2,
            "o contorno da pegada rodada é simétrico como uma elipse (pior assimetria {pior}) — \
             a curvatura não está a ser honrada depois do giro"
        );
    }

    /// ⭐ **E a rotação do dab MOVE o contorno** — a metade anti-vácuo: sem ela a asserção de cima
    /// passa sobre um contorno que ignorasse o ângulo (num círculo tudo é fronteira).
    #[test]
    fn the_outline_turns_with_the_dab() {
        let reto = FootprintDeform::new(0.5, 0);
        let torto = FootprintDeform::new(0.5, 90);
        let a = reto.outline_at(0.0);
        let b = torto.outline_at(0.0);
        let d = (a[0] - b[0]).hypot(a[1] - b[1]);
        assert!(
            d > 0.5,
            "o contorno não roda com o ângulo do dab (desvio {d}): ele está a ignorar a orientação"
        );
    }
}

//! ⭐⭐⭐ **O SEGUNDO CORPO — as duas rotas medidas contra a lei que ship.**
//!
//! Ordem do dono (2026-09-20): *«vamos tentar dar à forma presa dois corpos SE o custo em
//! performance não for muito alto»*, e depois *«e se fizer um bake para imagem e usar a imagem
//! como referência para usar a técnica de arc»*.
//!
//! ⛔ **Nenhuma das duas tem consumidor de produto** — elas são a medição que responde à condição
//! dele, e ficam com as tabelas ao lado porque *o que foi medido e rejeitado não se reconstrói*.
//! A [`refit_pela_curva`] chama a lei de dentro do fitter e custa `3`–`5 ms` por forma; a
//! [`refit_pelo_bake`] assa primeiro e custa `521`–`658 µs`, **`7,7×` menos e melhor nas duas
//! colunas**.
//!
//! ⚠️ Ficheiro próprio por tecto de LOC (a [`super`] foi a `884` ao recebê-las), e o corte é por
//! RESPONSABILIDADE: *ali vive a lei que SHIPA, aqui as duas que foram medidas contra ela.*

use kurbo::{Point, Vec2};
use ph2d_skeleton::{Correccao, Skin};
use ph2d_vec_scene::VecPath;

use super::{LeituraDoCampo, SegmentoDaPele, cubica, linha};

/// **Quanto o bake amostra e com que tolerância ajusta** — os dois números que andam juntos.
///
/// ⚠️ **Eles são UM par de propósito:** amostrar mais com a mesma tolerância não melhora
/// monotonamente (a `128` amostras/segmento o pior desvio *piora* `4,8×`, porque a amostragem
/// passa a resolver os bicos da malha em vez de os alisar). *Dois números cuja resposta conjunta
/// não é a composição das respostas separadas pertencem ao mesmo tipo.*
#[derive(Clone, Copy)]
pub struct Bake {
    /// Pontos por segmento-fonte que alimentam o bake.
    pub amostras: usize,
    /// A tolerância de Fréchet do [`kurbo::fit_to_bezpath`].
    pub tolerancia: f64,
}

/// ⭐⭐⭐ **O SEGUNDO CORPO — a curva RE-AJUSTADA, com os pontos que ela precisar.**
///
/// ⚠️⚠️ **Ela NÃO escreve no caminho: devolve um caminho NOVO.** É essa a diferença inteira com a
/// [`aplica_pela_curva_com`] — aquela deforma *o caminho do artista* e por isso está presa à
/// contagem de nós dele; esta produz *o que se desenha*, e pode pôr quantos nós forem precisos.
///
/// # ⛔⛔⛔ ELA FOI MEDIDA E RECUSADA — e fica com a tabela ao lado
///
/// Ordem do dono (2026-09-20): *«vamos tentar dar à forma presa dois corpos **se o custo em
/// performance não for muito alto**»*. A condição era o preço; ele foi medido **antes** de a rota
/// existir, e não passa. Barra da cena a `90°` em S, `--release`, `load < 6`, pela porta do
/// produto (sonda [`ph2d_skeleton_live::skinned_mesh::rive_tests`]):
///
/// | lei | nós | µs/forma | desvio ao ouro (p90) |
/// |---|---:|---:|---:|
/// | **a lei de HOJE** (o bind subdivide, o ajuste corrige) | `54` | **`105`** | **`0,00094`** |
/// | este refit, campo `C¹`, tolerância `0,03 %` | `61` | `4 889` | `0,00221` |
/// | *(sobre a fonte SEM o bind)* a lei de hoje | `8` | `53` | `0,22281` |
/// | *(sobre a fonte SEM o bind)* este refit | `40` | `4 401` | `0,00256` |
///
/// ⭐⭐⭐ **As duas últimas linhas são a razão inteira.** Sobre `8` nós o refit é **`87×`** mais
/// fiel — o mecanismo é real, e é exactamente o que faz o `Effects: Arc` parecer perfeito com
/// poucos pontos. ⇒ **mas o `Bind` já acrescenta os pontos, UMA VEZ**
/// ([`ph2d_skeleton_live::subdivisao::DIVISOES_POR_OSSO`]) — e a linha 1 contra a linha 4 é a
/// comparação que importa: **`bind + lei de hoje` é `2,7×` mais fiel que `sem bind + refit`, por
/// `1/42` do preço.** (Sobre a MESMA fonte de `54` nós a margem é `2,4×`, linha 1 contra 2.)
/// *Acrescentar pontos ao PRENDER ganha de acrescentá-los por QUADRO, nas duas colunas.*
///
/// ⛔ E o preço em si: `3`–`4,9 ms` por forma por quadro é **`18`–`29 %` de um quadro para UMA
/// forma**, e `1,5`–`2,3` quadros às oito formas presas da cena. A condição do dono não é
/// cumprida por um factor de `~45`.
///
/// ⚠️ **O que a medição comprou não foi esta rota — foi o [`crate::pesos::IndiceDoCampo`]:**
/// procurando o preço aqui, achou-se que **`96 %`** do custo de amostrar um ponto era uma busca
/// linear sobre os `878` triângulos da malha, paga **também pela lei que ship**. Indexá-la levou o
/// recook de `336` para **`105 µs`**.
///
/// ⚠️ Ela FICA, com esta tabela, porque *o que foi medido e rejeitado não se reconstrói* — e
/// porque a sonda que a mede é a prova executável da recusa.
///
/// # ⛔⛔⛔ O `c1` NÃO é um acabamento aqui — ele é a PRÉ-CONDIÇÃO
///
/// A tabela de pesos do padrão-ouro é lida por coordenadas **baricêntricas** sobre a malha BBW,
/// que é um elemento finito **LINEAR**: o valor é contínuo e o **gradiente SALTA em cada aresta**
/// da malha. O contorno da barra do dono atravessa **119** delas ⇒ a curva verdadeira tem um bico
/// de tangente em cada uma, e *um fitter adaptativo põe um nó em cada bico*. Medido na barra a
/// `90°` em S, a partir de `54` nós:
///
/// | leitura do campo | nós que o refit emite | pior desvio |
/// |---|---:|---:|
/// | baricêntrica (`c1 = false`) | **`147`** | **`1,588`** |
/// | **`C¹`** ([`crate::pesos_suave`]) | **`80`** | **`0,0041`** |
/// | sem campo nenhum (a mistura, contínua) | `54` | `0,012` |
///
/// ⇒ **`389×`** de fidelidade e **`~2×`** menos nós, só por o gradiente ser contínuo. ⚠️ E a linha
/// de baixo é o controlo que nomeia a causa: sem campo a lei é contínua e o fitter não acrescenta
/// **um único nó**.
///
/// ⭐ Isto reescreve o valor de fábrica da porta `C¹`: pelo caminho de HOJE ela compra um
/// acabamento invisível (`κ+ 3,16° → 1,98°`) por `17 %` de um quadro; por ESTE caminho ela é a
/// diferença entre funcionar e não.
#[must_use]
pub fn refit_pela_curva(
    pele: &Skin,
    fonte: &VecPath,
    pesos: &[f64],
    correcoes: &[Correccao],
    rigido: bool,
    leitura: LeituraDoCampo<'_>,
    tolerancia: f64,
) -> VecPath {
    let LeituraDoCampo { campo, c1 } = leitura;
    let mut out = fonte.clone();
    // ⚠️ **Aqui a leitura `C¹` é PARÂMETRO e não uma variável de ambiente**, ao contrário da irmã
    // [`aplica_pela_curva_com`] — que a lê lá dentro e cuja dívida está nomeada no gate
    // `a_leitura_c1_cura_o_campo_e_nao_chega_ao_desenho`. *Uma lei que um gate não pode controlar
    // sem o ambiente é uma lei que, sob `cargo test`, vaza entre testes.*
    let suave = campo
        .filter(|_| c1)
        .and_then(crate::pesos_suave::CampoSuave::novo);
    let indice = campo.and_then(|c| crate::pesos::IndiceDoCampo::novo(&c.malha));
    let ossos = if pesos.is_empty() {
        0
    } else {
        pesos.len() / (fonte.verts_all().count() * 3).max(1)
    };
    let mut base = 0usize;
    for c in 0..fonte.contour_count() {
        let Some((verts, fechado)) = fonte.contour(c) else {
            continue;
        };
        let n = verts.len();
        let segs = if fechado { n } else { n.saturating_sub(1) };
        if segs == 0 {
            continue;
        }
        let mut cubicas: Vec<[Point; 3]> = Vec::new();
        let mut inicio = Point::ZERO;
        for k in 0..segs {
            let s = SegmentoDaPele {
                src: cubica(verts, k, n),
                pele,
                ra: linha(pesos, ossos, base + k),
                rb: linha(pesos, ossos, base + (k + 1) % n),
                correcoes,
                rigido,
                campo,
                indice: indice.as_ref(),
                suave: suave.as_ref(),
            };
            if k == 0 {
                inicio = s.ponto(0.0);
            }
            let fitado = kurbo::fit_to_bezpath(&s, tolerancia);
            ph2d_vec_envelope::push_cubics(&fitado, &mut cubicas);
        }
        if let Some((alvo, _)) = out.contour_mut(c) {
            *alvo = ph2d_vec_envelope::rebuild(&cubicas, inicio, fechado);
        }
        base += n;
    }
    out
}

/// ⭐⭐⭐ **O SEGUNDO CORPO PELO BAKE — a ideia do dono, e ela é de outra classe.**
///
/// Report do dono (2026-09-20): *«e se fizer um bake para imagem e usar a imagem como referência
/// para usar a técnica de arc»*.
///
/// # A diferença com o [`refit_pela_curva`], que é a razão de esta existir
///
/// Aquele chama a LEI **de dentro do fitter**, e o fitter amostra adaptativamente: `8 371`
/// consultas, `93 %` do preço numa leitura `C¹` que custa `0,326 µs` cada. Esta **assa primeiro**:
/// percorre a curva verdadeira num número FIXO de amostras por segmento, com a leitura
/// baricêntrica barata (`0,066 µs`, indexada), e só depois ajusta — contra os pontos já assados.
///
/// ⭐⭐ **E o bake resolve DE GRAÇA o que obrigava o `C¹`:** o fitter precisa de uma tangente
/// contínua, e aqui ela sai de uma **Catmull-Rom sobre as amostras**, não do gradiente do campo.
/// Os bicos que o elemento finito linear deixa em cada uma das `119` arestas da malha ficam
/// **abaixo da amostragem**, em vez de forçarem um nó cada um.
///
/// # A medição, na barra da cena a `90°` em S (`--release`, `load < 8`)
///
/// | lei | nós | µs/forma | ouro p90 | máx |
/// |---|---:|---:|---:|---:|
/// | a lei de HOJE (um corpo só) | `54` | **`106`** | `0,00094` | `0,00341` |
/// | o [`refit_pela_curva`] (chama a lei de dentro do fitter) | `61` | `3 993` | `0,00221` | `0,00466` |
/// | **este bake, `16` amostras/seg, tol `0,03 %`** | `104` | **`521`** | `0,00092` | `0,00198` |
/// | **este bake, `32` amostras/seg** | `110` | `658` | **`0,00070`** | **`0,00194`** |
/// | o CHÃO do modelo aos `54` nós | `54` | — | `0,00095` | `0,00334` |
///
/// ⭐⭐⭐ **Ela é `7,7×` mais barata que o refit adaptativo E melhor nas duas colunas** — e passa
/// **por baixo do CHÃO** dos `54` nós, que é exactamente o que acrescentar pontos compra.
/// ⭐ E **não precisa do `C¹`**: a leitura barata chega, porque quem dá a continuidade é a
/// Catmull-Rom sobre o bake.
///
/// # ⛔⛔ Ela NÃO substitui a subdivisão do `Bind`, e isso está medido
///
/// | sobre a fonte CRUA de `8` nós | nós | µs | ouro p90 | máx |
/// |---|---:|---:|---:|---:|
/// | `16` amostras/seg | `41` | `227` | `0,02144` | `0,06738` |
/// | `32` | `42` | `245` | `0,00426` | `0,01289` |
/// | `64` | `45` | `303` | `0,00165` | `0,00456` |
/// | `128` | `59` | `421` | `0,00121` | **`0,02191`** |
///
/// Mesmo a `64` amostras ela fica `2,9×` mais cara e `1,3`–`1,8×` pior do que `bind + lei de
/// hoje`. ⇒ *a subdivisão do bind faz trabalho a sério: ela põe nós onde o ESQUELETO precisa
/// (graduada pelas juntas), e isso é informação que o fitter não tem.*
///
/// ⚠️⚠️ **E a linha dos `128` é o achado que não se adivinha: amostrar MAIS piora.** O máximo
/// salta de `0,00456` para `0,02191`, porque a amostragem passa a resolver os bicos que a malha
/// linear deixa em cada uma das `119` arestas em vez de os alisar. *O alisamento do bake não é um
/// efeito colateral — é o que o protege, e existe uma amostragem ÓPTIMA.*
///
/// `amostras` é quantos pontos por segmento-fonte alimentam o bake; `tolerancia` é a de Fréchet
/// que o [`kurbo::fit_to_bezpath`] recebe.
#[must_use]
pub fn refit_pelo_bake(
    pele: &Skin,
    fonte: &VecPath,
    pesos: &[f64],
    correcoes: &[Correccao],
    rigido: bool,
    leitura: LeituraDoCampo<'_>,
    bake: Bake,
) -> VecPath {
    let (
        LeituraDoCampo { campo, c1 },
        Bake {
            amostras,
            tolerancia,
        },
    ) = (leitura, bake);
    let mut out = fonte.clone();
    let suave = campo
        .filter(|_| c1)
        .and_then(crate::pesos_suave::CampoSuave::novo);
    let indice = campo.and_then(|c| crate::pesos::IndiceDoCampo::novo(&c.malha));
    let ossos = if pesos.is_empty() {
        0
    } else {
        pesos.len() / (fonte.verts_all().count() * 3).max(1)
    };
    let mut base = 0usize;
    for c in 0..fonte.contour_count() {
        let Some((verts, fechado)) = fonte.contour(c) else {
            continue;
        };
        let n = verts.len();
        let segs = if fechado { n } else { n.saturating_sub(1) };
        if segs == 0 {
            continue;
        }
        let mut cubicas: Vec<[Point; 3]> = Vec::new();
        let mut inicio = Point::ZERO;
        for k in 0..segs {
            let s = SegmentoDaPele {
                src: cubica(verts, k, n),
                pele,
                ra: linha(pesos, ossos, base + k),
                rb: linha(pesos, ossos, base + (k + 1) % n),
                correcoes,
                rigido,
                campo,
                indice: indice.as_ref(),
                suave: suave.as_ref(),
            };
            // ⭐ O BAKE: um número FIXO de pontos, com a leitura barata.
            let assado: Vec<Point> = (0..=amostras)
                .map(|i| {
                    #[expect(clippy::cast_precision_loss, reason = "i <= amostras")]
                    let t = i as f64 / amostras as f64;
                    s.ponto(t)
                })
                .collect();
            if k == 0 {
                inicio = assado[0];
            }
            let fitado = kurbo::fit_to_bezpath(&Assado(&assado), tolerancia);
            ph2d_vec_envelope::push_cubics(&fitado, &mut cubicas);
        }
        if let Some((alvo, _)) = out.contour_mut(c) {
            *alvo = ph2d_vec_envelope::rebuild(&cubicas, inicio, fechado);
        }
        base += n;
    }
    out
}

/// A polilinha assada vista como curva paramétrica **lisa** — o que o fitter recebe.
///
/// ⭐⭐ **A tangente é a de CATMULL-ROM** e não a diferença dos vizinhos: ela é `C¹` por
/// construção, e é isso que impede o fitter de perseguir os bicos que a lei tem. *O bake não
/// alisa a verdade por acidente — ele alisa-a exactamente à escala da amostragem.*
struct Assado<'a>(&'a [Point]);

impl Assado<'_> {
    /// `(ponto, tangente)` em `t ∈ [0, 1]`, por Catmull-Rom uniforme.
    fn em(&self, t: f64) -> (Point, Vec2) {
        let n = self.0.len();
        if n < 2 {
            return (
                self.0.first().copied().unwrap_or(Point::ZERO),
                Vec2::new(1.0, 0.0),
            );
        }
        #[expect(clippy::cast_precision_loss, reason = "n é um punhado de amostras")]
        let u = t.clamp(0.0, 1.0) * (n - 1) as f64;
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "u >= 0 e finito"
        )]
        let i = (u as usize).min(n - 2);
        #[expect(clippy::cast_precision_loss, reason = "i < n")]
        let f = u - i as f64;
        let g = |k: isize| self.0[(k.max(0) as usize).min(n - 1)];
        #[expect(clippy::cast_possible_wrap, reason = "i < n, um punhado")]
        let ii = i as isize;
        let (p0, p1, p2, p3) = (g(ii - 1), g(ii), g(ii + 1), g(ii + 2));
        let (m0, m1) = ((p2 - p0) * 0.5, (p3 - p1) * 0.5);
        let (f2, f3) = (f * f, f * f * f);
        let h = |a: f64, b: f64, c: f64, d: f64| a + b + c + d;
        let p = Point::new(
            h(
                2.0f64.mul_add(f3, -3.0 * f2) * p1.x + p1.x,
                (-2.0f64).mul_add(f3, 3.0 * f2) * p2.x,
                f3.mul_add(1.0, -2.0 * f2 + f) * m0.x,
                f3.mul_add(1.0, -f2) * m1.x,
            ),
            h(
                2.0f64.mul_add(f3, -3.0 * f2) * p1.y + p1.y,
                (-2.0f64).mul_add(f3, 3.0 * f2) * p2.y,
                f3.mul_add(1.0, -2.0 * f2 + f) * m0.y,
                f3.mul_add(1.0, -f2) * m1.y,
            ),
        );
        // A derivada em `f`, vezes `n-1` para ela ser em `t`.
        #[expect(clippy::cast_precision_loss, reason = "n é um punhado")]
        let esc = (n - 1) as f64;
        let d = |a: f64, b: f64, ma: f64, mb: f64| {
            6.0f64.mul_add(f2 - f, 0.0).mul_add(a - b, 0.0)
                + 3.0f64.mul_add(f2, -4.0 * f + 1.0) * ma
                + 3.0f64.mul_add(f2, -2.0 * f) * mb
        };
        let tang = Vec2::new(
            d(p1.x, p2.x, m0.x, m1.x) * esc,
            d(p1.y, p2.y, m0.y, m1.y) * esc,
        );
        (p, tang)
    }
}

impl kurbo::ParamCurveFit for Assado<'_> {
    fn sample_pt_tangent(&self, t: f64, _sign: f64) -> kurbo::CurveFitSample {
        let (p, tangent) = self.em(t);
        kurbo::CurveFitSample { p, tangent }
    }

    fn sample_pt_deriv(&self, t: f64) -> (Point, Vec2) {
        self.em(t)
    }

    fn break_cusp(&self, _range: core::ops::Range<f64>) -> Option<f64> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::Assado;
    /// ⭐⭐⭐ **GATE — O BAKE INTERPOLA, E A TANGENTE DELE É A DERIVADA DE VERDADE.**
    ///
    /// ⛔⛔ **A segunda metade não é zelo:** o cabeçalho da [`ph2d_vec_envelope::Warp`] mede o sintoma
    /// de uma tangente inconsistente com o ponto — o `fit_to_bezpath` **não converge**, ele subdivide
    /// para reconciliar um par que não pertence à mesma curva. *Um bake com a derivada errada não dá
    /// uma imagem errada: dá um fitter que nunca pára.*
    #[test]
    fn o_bake_interpola_e_a_tangente_e_a_derivada() {
        use kurbo::Point;
        // Uma amostragem de uma curva conhecida, irregular de propósito.
        let pts: Vec<Point> = (0..=12)
            .map(|i| {
                let t = f64::from(i) / 12.0;
                Point::new(
                    10.0f64.mul_add(t, 2.0 * (t * 6.0).sin()),
                    3.0f64.mul_add((t * 4.0).cos(), t * t * 5.0),
                )
            })
            .collect();
        let a = Assado(&pts);
        // (1) Ela passa PELAS amostras.
        for (i, p) in pts.iter().enumerate() {
            #[expect(clippy::cast_precision_loss, reason = "i <= 12")]
            let t = i as f64 / (pts.len() - 1) as f64;
            let q = a.em(t).0;
            assert!(
                (q - *p).hypot() < 1e-9,
                "em t={t} o bake devolveu {q:?} e a amostra é {p:?} — ele deixou de INTERPOLAR"
            );
        }
        // (2) A tangente é a derivada da posição, por diferença central da PRÓPRIA função.
        let (mut pior, mut escala) = (0.0_f64, 0.0_f64);
        for k in 1..200 {
            let t = f64::from(k) / 200.0;
            const H: f64 = 1e-6;
            let fd = (a.em(t + H).0 - a.em(t - H).0) / (2.0 * H);
            let tg = a.em(t).1;
            pior = pior.max((fd - tg).hypot());
            escala = escala.max(tg.hypot());
        }
        assert!(
            escala > 1.0,
            "a fixtura tem tangente ~nula ({escala}) — ela não contém o fenómeno"
        );
        assert!(
            pior < 1e-4 * escala,
            "a tangente do bake desvia {pior} da derivada dele (escala {escala}) — com ela \
             inconsistente o `fit_to_bezpath` NÃO CONVERGE, e o sintoma é o fitter a subdividir sem fim"
        );
    }
}

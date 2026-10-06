//! ⭐⭐⭐ **O SEGUNDO CORPO — as duas rotas medidas contra a lei que ship.**
//!
//! Ordem do dono (2026-09-20): *«vamos tentar dar à forma presa dois corpos SE o custo em
//! performance não for muito alto»*, e depois *«e se fizer um bake para imagem e usar a imagem
//! como referência para usar a técnica de arc»*.
//!
//! ⭐⭐⭐ **O BAKE SHIPA desde 2026-09-29** ([`assa_a_pele`], chamado pela gaveta do
//! `ph2d_skeleton_live::skin_desenho`) — ordem do dono, *«siga como achar melhor, buscando o padrão
//! ouro»*, com a recusa de 2026-09-20 relida contra a base certa: ela foi medida contra a
//! subdivisão do `Bind` que o commit seguinte RETIROU, e sobre os `8` nós que ficaram o bake é
//! `108`–`149×` mais fiel ao padrão-ouro (barra da cena, `30°`–`90°` em S). ⛔ O
//! [`refit_pela_curva`] continua **sem** consumidor — é a recusa medida, com a tabela ao lado,
//! porque *o que foi medido e rejeitado não se reconstrói*.
//! A [`refit_pela_curva`] chama a lei de dentro do fitter e custa `3`–`5 ms` por forma; a
//! [`refit_pelo_bake`] assa primeiro e custa `521`–`658 µs`, **`7,7×` menos e melhor nas duas
//! colunas**.
//!
//! ⚠️ Ficheiro próprio por tecto de LOC (a [`super`] foi a `884` ao recebê-las), e o corte é por
//! RESPONSABILIDADE: *ali vive a lei que SHIPA, aqui as duas que foram medidas contra ela.*

use kurbo::{Point, Vec2};
use ph2d_skeleton::{Correccao, Skin};
use ph2d_vec_scene::VecPath;

use super::{CampoIndexado, LeituraDoCampo, SegmentoDaPele, cubica, linha};

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
    /// ⭐ O refino LOCAL onde a pose estica o contorno ([`Refino`], A13) — `None` é a amostragem
    /// uniforme de antes, a referência dos gates.
    pub refino: Option<Refino>,
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
    let LeituraDoCampo { campo, c1 } = leitura;
    let suave = campo
        .filter(|_| c1)
        .and_then(crate::pesos_suave::CampoSuave::novo);
    let indice = campo.and_then(|c| crate::pesos::IndiceDoCampo::novo(&c.malha));
    assa_a_pele(
        pele,
        fonte,
        pesos,
        correcoes,
        rigido,
        CampoIndexado {
            campo,
            indice: indice.as_ref(),
            suave: suave.as_ref(),
        },
        bake,
    )
}

/// ⭐⭐⭐ **O BAKE com o campo JÁ INDEXADO** — o corpo da [`refit_pelo_bake`], e a porta que o
/// PRODUTO chama desde 2026-09-29 ([`ph2d_skeleton_live::skin_desenho`]), com o índice guardado
/// por bind em vez de refeito por quadro.
#[must_use]
pub fn assa_a_pele(
    pele: &Skin,
    fonte: &VecPath,
    pesos: &[f64],
    correcoes: &[Correccao],
    rigido: bool,
    lido: CampoIndexado<'_>,
    bake: Bake,
) -> VecPath {
    assa_a_pele_com_nos(pele, fonte, pesos, correcoes, rigido, lido, bake).0
}

/// ⭐⭐ **O BAKE e ONDE caiu cada nó da fonte** — a [`assa_a_pele`] mais a posição assada do nó `k`
/// de cada contorno, na ordem de [`VecPath::verts_all`] (F42, report do dono de 2026-09-30).
///
/// ⛔ **O assado deixa nós que o artista NUNCA pôs** — o ajuste parte cada segmento em quantas
/// cúbicas a tolerância pedir, e na DOBRA do mapa um desses nós vira `180°` (medido na cena `=4` a
/// `85°`: um gancho de `0,015` no vinco). Quem pergunta *«esta quina é do artista?»* precisa de
/// saber quais nós do assado são os DELE, e a resposta é esta lista: o `k`-ésimo ponto é, ao bit,
/// a âncora do vértice que abre o segmento `k` no resultado.
#[must_use]
pub fn assa_a_pele_com_nos(
    pele: &Skin,
    fonte: &VecPath,
    pesos: &[f64],
    correcoes: &[Correccao],
    rigido: bool,
    lido: CampoIndexado<'_>,
    bake: Bake,
) -> (VecPath, Vec<[f64; 2]>) {
    let mut nos = Vec::with_capacity(fonte.verts_all().count());
    let (
        CampoIndexado {
            campo,
            indice,
            suave,
        },
        Bake {
            amostras,
            tolerancia,
            refino,
        },
    ) = (lido, bake);
    let mut out = fonte.clone();
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
                indice,
                suave,
            };
            // ⭐ O BAKE: um número FIXO de pontos, com a leitura barata — e, com o [`Refino`], os
            // pontos a mais onde a pose ESTICA o contorno.
            let (assado, nos_t) = refino::amostra(&s, amostras, tolerancia, refino);
            if k == 0 {
                inicio = assado[0];
            }
            // ⚠️ O nó é a ÂNCORA que o `rebuild` lhe dá — o `inicio` no primeiro, e o fim da
            // última cúbica do segmento anterior nos outros —, porque quem compara compara ao bit e
            // é ESSA a posição que o vértice tem. ⚠️ Nomeado: no corpus o `ponto(0)` do segmento
            // coincide com ela ao bit (a mutação que o usa SOBREVIVE, F42) — é a definição, não
            // uma divergência medida.
            let no = cubicas.last().map_or(inicio, |c| c[2]);
            nos.push([no.x, no.y]);
            let fitado = ajusta(&Assado(&assado, nos_t.as_deref()), tolerancia);
            ph2d_vec_envelope::push_cubics(&fitado, &mut cubicas);
        }
        if let Some((alvo, _)) = out.contour_mut(c) {
            *alvo = ph2d_vec_envelope::rebuild(&cubicas, inicio, fechado);
        }
        if !fechado {
            // O último nó de um contorno ABERTO não abre segmento: é o fim do último.
            nos.push(
                cubicas
                    .last()
                    .map_or([inicio.x, inicio.y], |c| [c[2].x, c[2].y]),
            );
        }
        base += n;
    }
    (out, nos)
}

/// ⭐⭐⭐ **O AJUSTE com a conferência nos DOIS sentidos** — o `kurbo::fit_to_bezpath` com a
/// aceitação que ele não tem.
///
/// ⛔⛔ **O defeito que isto cura (report do dono, 2026-09-29, com foto: *«uma linha anómala no
/// stroke, atravessando a forma»*):** o `fit_to_cubic` do `kurbo` 0.13 mede o erro só no sentido
/// FONTE → CÚBICA (um raio normal por amostra da fonte), e só liga a medida por comprimento de arco
/// quando a fonte é *«picante»*. Num trecho QUASE RECTO nenhuma das duas vê uma cúbica que sai ao
/// longo da própria recta e volta: cada amostra acha a cúbica ao pé de si. Medido na barra da cena
/// em **C a `60°`**: um pedaço de corda `0,94` aceite com uma alça a **`9,5`** de distância — o
/// espeto de `1,41` que o traço desenha através da forma.
///
/// ⇒ a mesma recursão do `kurbo` (ajustar, e partir ao meio se não serve), com UMA cláusula a mais:
/// a cúbica só entra se **toda ela** ficar perto da fonte ([`fecha`]). No fundo da recursão (um
/// pedaço mais curto que o passo da amostragem) sai a Hermite do próprio bake — que É a curva que
/// o fitter recebe, logo não há o que conferir.
fn ajusta(src: &Assado<'_>, tolerancia: f64) -> kurbo::BezPath {
    let mut path = kurbo::BezPath::new();
    ajusta_rec(src, 0.0..1.0, tolerancia, &mut path);
    path
}

fn ajusta_rec(
    src: &Assado<'_>,
    range: core::ops::Range<f64>,
    tolerancia: f64,
    path: &mut kurbo::BezPath,
) {
    let c = if src.cabe_num_intervalo(&range) {
        Some(src.hermite(range.clone()))
    } else {
        kurbo::fit_to_cubic(src, range.clone(), tolerancia)
            .map(|(c, _)| c)
            .filter(|c| fecha(src, *c, range.clone(), tolerancia))
    };
    if let Some(c) = c {
        if path.elements().is_empty() {
            path.move_to(c.p0);
        }
        path.curve_to(c.p1, c.p2, c.p3);
        return;
    }
    let meio = 0.5 * (range.start + range.end);
    ajusta_rec(src, range.start..meio, tolerancia, path);
    ajusta_rec(src, meio..range.end, tolerancia, path);
}

/// **A metade que o `kurbo` não confere: CÚBICA → FONTE.** Cada ponto da cúbica tem de ficar a
/// menos de `2 × tolerância` da fonte (a folga cobre a corda da polilinha densa em que ela é
/// medida). Com a metade dele (fonte → cúbica, já feita), é a distância de Hausdorff.
fn fecha(
    src: &Assado<'_>,
    c: kurbo::CubicBez,
    range: core::ops::Range<f64>,
    tolerancia: f64,
) -> bool {
    use kurbo::{ParamCurve, ParamCurveDeriv, ParamCurveNearest};
    const NA_CUBICA: usize = 24;
    let fonte: Vec<Point> = src.densos(&range);
    let folga2 = (2.0 * tolerancia).powi(2);
    let deriv = c.deriv();
    (0..=NA_CUBICA).all(|i| {
        #[expect(clippy::cast_precision_loss, reason = "um punhado")]
        let t = i as f64 / NA_CUBICA as f64;
        let p = c.eval(t);
        let Some((d2, w)) = fonte
            .windows(2)
            .map(|w| {
                (
                    kurbo::Line::new(w[0], w[1]).nearest(p, 1e-12).distance_sq,
                    w,
                )
            })
            .min_by(|a, b| a.0.total_cmp(&b.0))
        else {
            return false;
        };
        // ⭐⭐ **E ANDA NO SENTIDO DA FONTE** (report do dono de 2026-10-03, «vários defeitos no
        // stroke»: quartos de círculo ao longo do traço de uma forma com efeito). Um nó de quina
        // (alça nula) deixa o `fit_to_cubic` com um braço `0` e o outro maior que a corda: a
        // cúbica volta para trás numa laçada MENOR que a tolerância — a distância aceita-a, e o
        // traço grosso desenha a meia-volta. Medido no *Twist* a `60°`: `9` voltas de `~180°`
        // (barra sem efeito: `0`). Um ponto de derivada nula (a própria cúspide) não decide.
        let v = deriv.eval(t).to_vec2();
        let u = w[1] - w[0];
        d2 <= folga2 && (v.hypot2() < 1e-18 || u.hypot2() < 1e-18 || v.dot(u) >= 0.0)
    }) && {
        // ⚠️ A laçada do braço `0` mora a `t < 0,01` de uma ponta — entre as amostras acima. Ali a
        // conferência é EXACTA: a projecção da derivada (uma Bézier quadrática) na direcção da
        // fonte nessa ponta, mínima no primeiro e no último quarto.
        let ponta = |a: Point, b: Point| b - a;
        let (n0, n1) = (fonte.len().min(2), fonte.len());
        let u0 = ponta(fonte[0], fonte[n0 - 1]);
        let u1 = ponta(fonte[n1.saturating_sub(2)], fonte[n1 - 1]);
        anda_para_a_frente(c, u0, 0.0, 0.25) && anda_para_a_frente(c, u1, 0.75, 1.0)
    }
}

/// A derivada de `c` projectada em `u` é `≥ 0` em todo `t ∈ [t0, t1]`? — exacta: a projecção é
/// uma quadrática em `t`, e o mínimo dela num intervalo está nas pontas ou no vértice.
fn anda_para_a_frente(c: kurbo::CubicBez, u: Vec2, t0: f64, t1: f64) -> bool {
    if u.hypot2() < 1e-24 {
        return true;
    }
    let a = 3.0 * (c.p1 - c.p0).dot(u);
    let b = 3.0 * (c.p2 - c.p1).dot(u);
    let d = 3.0 * (c.p3 - c.p2).dot(u);
    let f = |t: f64| (1.0 - t).powi(2) * a + 2.0 * t * (1.0 - t) * b + t * t * d;
    let curv = a - 2.0 * b + d;
    let mut ts = vec![t0, t1];
    if curv.abs() > 1e-300 {
        let tv = (a - b) / curv;
        if tv > t0 && tv < t1 {
            ts.push(tv);
        }
    }
    // Uma tolerância RELATIVA: a derivada nula na ponta (o braço `0`) é a cúspide em si, não uma
    // volta — só um valor claramente negativo é ir para trás.
    let escala = a.abs().max(b.abs()).max(d.abs());
    ts.into_iter().all(|t| f(t) >= -1e-9 * escala)
}

/// A polilinha assada vista como curva paramétrica **lisa** — o que o fitter recebe.
///
/// ⭐⭐ **A tangente é a de CATMULL-ROM** e não a diferença dos vizinhos: ela é `C¹` por
/// construção, e é isso que impede o fitter de perseguir os bicos que a lei tem. *O bake não
/// alisa a verdade por acidente — ele alisa-a exactamente à escala da amostragem.*
///
/// O 2.º campo são os NÓS (o `t` da fonte de cada amostra) quando o [`refino`] partiu algum
/// intervalo; `None` é a uniforme de sempre, e então a aritmética é a de antes, ao bit.
struct Assado<'a>(&'a [Point], Option<&'a [f64]>);

impl Assado<'_> {
    /// `(ponto, tangente)` em `t ∈ [0, 1]`, por Catmull-Rom uniforme.
    fn em(&self, t: f64) -> (Point, Vec2) {
        if let Some(nos) = self.1 {
            return self.em_nos(nos, t);
        }
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

impl Assado<'_> {
    /// O pedaço `range` da própria curva do bake, como UMA cúbica de Hermite — exacto quando o
    /// pedaço cabe num intervalo de amostragem, porque ali o bake É uma Hermite.
    fn hermite(&self, range: core::ops::Range<f64>) -> kurbo::CubicBez {
        let (p0, d0) = self.em(range.start);
        let (p1, d1) = self.em(range.end);
        let k = (range.end - range.start) / 3.0;
        kurbo::CubicBez::new(p0, p0 + d0 * k, p1 - d1 * k, p1)
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

/// A amostragem do bake e a sua generalização a nós NÃO uniformes (medição A13), num irmão.
#[path = "curva_segundo_corpo_refino.rs"]
mod refino;
pub use refino::{REFINO_DO_PRODUTO, Refino};

#[cfg(test)]
#[path = "curva_segundo_corpo_espeto_tests.rs"]
mod espeto_tests;

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
        let a = Assado(&pts, None);
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

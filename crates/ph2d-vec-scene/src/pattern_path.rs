//! **Pattern Along Path** — um motivo copiado, rígido, ao longo de um caminho-guia
//! (plano [`docs/Vector Module/23_plano_pattern_along_path.md`], item #11 da pesquisa 20).
//!
//! É o **irmão do texto em caminho**: onde o texto cavalga a curva com **glifos** rígidos, o
//! pattern a cavalga com **cópias de uma forma**. Um glifo é uma forma; o motor é o mesmo — o
//! [`crate::arc_path::ArcPath`] localiza o arco e o [`crate::text_path::GlyphFrame`] devolve o afim
//! **rígido** (rotação + translação) que leva o motivo ao mundo. Como o mapa é afim, ele **comuta**
//! com a avaliação de Bézier ([`crate::text_path`] §Rigidez): a curva que sai é a imagem EXATA da
//! que entra, e **não há refit** — que é o que separa este (rígido) do Envelope (ADR-0129), cujo
//! mapa não é afim e por isso precisa de `fit_to_bezpath`.
//!
//! # A cópia é ancorada pelo CENTRO
//!
//! Cada cópia ocupa uma fatia de arco de comprimento `avanço = largura(bbox) × spacing`, e o
//! caminho é amostrado no **centro** da fatia — a mesma convenção do `<textPath>` (o glifo roda em
//! torno de `x + avanço/2`), pela mesma razão: ancorar pela borda faria a forma girar **para fora**
//! da curva. O vértice do motivo entra em espaço local `[mx − cx, my − cy]` (o motivo recentrado no
//! seu bbox), e o `apply` do frame leva `+x` do motivo **ao longo** da guia e `+y` **à normal**.
//!
//! # A rotação é CONSTANTE e relativa à TANGENTE
//!
//! O [`PatternSpec::rotation_deg`] gira o motivo dentro do referencial da cópia — ou seja, **em
//! relação à guia**, não ao mundo. Toda cópia recebe o MESMO ângulo, então numa curva as cópias
//! continuam a acompanhar a tangente e só ganham uma orientação fixa em cima dela (90° = de pé,
//! atravessadas na curva). Isto **não** é o que o § seguinte recusa: ali o que não existe é uma
//! rotação que **progride** de cópia para cópia (um leque), que é do Repeater e é dirigida por
//! índice, não pela guia.
//!
//! ⚠️ **A rotação alimenta a MEDIDA do motivo, e tem de alimentar.** O avanço por cópia é
//! `largura(bbox) × spacing`, e o contrato do `spacing` é *"1.0 encaixa borda-a-borda"* — mas o que
//! ocupa a guia é a extensão do motivo **já girado**: um traço 1×10 deitado ocupa 1, de pé ocupa
//! 10. Medir o bbox antes de girar deixaria o `spacing` a dizer uma coisa e a fazer outra, em
//! silêncio, para todo ângulo ≠ 0. Por isso [`motif_bbox`] mede **no referencial girado** e é a
//! porta ÚNICA de onde saem o avanço E a centragem — dois números que têm de concordar sobre o
//! mesmo bbox.
//!
//! ⚠️ **O pivô da rotação é irrelevante, e isso é um teorema, não um descuido.** A cópia é
//! re-centrada no bbox girado, e a re-centragem absorve o pivô: girar em torno da origem dá
//! `R(p) − centro(R(p))`, girar em torno de `c` dá `R(p−c) − centro(R(p−c))`, e como
//! `R(p) = R(p−c) + R(c)` os dois centros diferem exatamente por `R(c)`, que cancela. Não há
//! *"gira em torno de quê?"* a responder errado.
//!
//! # O que este módulo NÃO faz (plano §3)
//!
//! Não **deforma** o motivo para dobrar ao longo da curva (isso é o Envelope) · não escala nem roda
//! **por progressão** (isso é o Repeater, `fx_repeat`, dirigido por índice de cópia e não pela
//! guia — vide o § da rotação acima, que é constante) · não **estica o avanço** para fechar exato
//! no fim (a cauda pode sobrar; *fit* é refinamento próprio).

use crate::arc_path::ArcPath;
use crate::text_path::GlyphFrame;
use crate::{Contour, VecPath, VecVertex};

/// Avanço mínimo, em unidades de mundo. Um motivo degenerado (largura ~0, uma reta vertical) daria
/// avanço zero e uma divisão por zero na contagem de cópias; o piso o transforma numa tilagem
/// densa em vez de num laço infinito. **Não é um teto de produto** — é uma guarda de representação
/// (a contagem `total/avanço` tem de ser finita); o `spacing` é quem controla o espaçamento real.
const MIN_ADVANCE: f64 = 1e-3;

/// Teto de cópias emitidas.
///
/// ⚠️ **Ele ERA inalcançável e deixou de ser — a nota foi reconferida quando o número que o
/// tornava inalcançável se moveu.** Enquanto o piso do Spacing era `0.25`, este teto era guarda de
/// **memória** que nenhum documento tocava, e este comentário dizia *"não é um limite de produto"*.
/// Com o piso em `0.01` (2026-07-23, pedido do Enio) ele **é alcançado**, e passou a ser também o
/// teto de **TEMPO** — MEDIDO, motivo de 4 vértices, guia reta:
///
/// | guia | spacing | cópias | re-cook |
/// |---|---|---|---|
/// | 400 | 0,25 | 40 | 0,08 ms |
/// | 400 | 0,01 | 1000 | 1,81 ms |
/// | 4000 | 0,05 | 2000 | 3,60 ms |
/// | 4000 | 0,01 | **4096 (no teto)** | 7,53 ms |
///
/// 4096 cópias custam **7,53 ms**, contra o kill de 8 ms do re-cook por-frame
/// (`a_keystroke_recook_stays_under_the_kill`; não há memo — vide `pattern_live`). Subir o teto
/// **estoura o orçamento de frame**, então ele fica onde está: hoje o recurso que ele guarda é
/// tempo, não memória.
///
/// ⚠️ **A condição em que o artista o encontra, e o que ele vê:** o teto morde quando
/// `comprimento(guia) > 40,96 × largura(motivo)` no spacing mínimo — e a tilagem simplesmente
/// **para no meio da curva, sem aviso**. Isso é conhecido e NÃO está resolvido: a saída natural é
/// um cache por-params (plano 23 §0) que tire o re-cook do frame e liberte o teto, não subir o
/// número com o custo onde está.
const MAX_COPIES: usize = 4096;

/// Folga de encaixe, em **frações de fatia** (unidades de `k`), na fronteira "a cópia cabe exato".
///
/// O `total` de um contorno é MEDIDO (Gauss-Legendre), não exato — o círculo de quatro cúbicas
/// carrega ~1,4e-4 relativo. Numa guia cujo comprimento é múltiplo do avanço, a última cópia cabe
/// **exatamente**, e uma comparação estrita a deixaria dentro ou fora conforme o ruído da
/// quadratura. Esta folga (1% de uma fatia = invisível) absorve o ruído sem nunca admitir uma cópia
/// que transborde de verdade.
const FIT_EPS: f64 = 1e-2;

/// **Os parâmetros de um pattern-along.**
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PatternSpec {
    /// Arco onde a tilagem começa, em unidades de mundo ao longo da guia. É o irmão do
    /// `start_offset` do texto: correr este valor desliza o padrão inteiro pela curva.
    pub start_offset: f64,
    /// Arco onde a tilagem TERMINA, em unidades de mundo. As cópias caem no trecho
    /// `[start_offset, end_offset]` (o limite superior efetivo é `min(end_offset, total)`).
    /// `f64::INFINITY` = a curva inteira, e é o default (nenhum consumidor que não queira limitar
    /// precisa saber que o campo existe).
    pub end_offset: f64,
    /// Multiplica a **largura** do bbox do motivo para dar o avanço por cópia. `1.0` encaixa as
    /// cópias borda-a-borda; `0.5` sobrepõe metade; `2.0` deixa um vão de uma cópia.
    pub spacing: f64,
    /// Desvio da guia ao longo da **normal**, positivo para a ESQUERDA do sentido de marcha (a
    /// mesma convenção — e o mesmo sinal — do `dy` do texto, `GlyphFrame::on_path`).
    pub offset: f64,
    /// Põe o padrão do **outro lado** da curva, a percorrê-la ao contrário.
    pub flip: bool,
    /// **Orientação do motivo sobre a curva, em GRAUS**, dentro do referencial da cópia — `90` põe o
    /// motivo de pé, atravessado na guia. Constante em todas as cópias (§ do módulo); a unidade
    /// está no NOME porque um ângulo sem unidade declarada é o bug que não dá erro em lado nenhum
    /// (a mutação que troca `to_radians` por radianos crus sangra em três gates).
    ///
    /// `0.0` é o neutro: os gates de rotação-zero que já existiam (`the_copies_tile_the_straight_guide`,
    /// `the_copies_rotate_to_the_tangent_on_a_curve`) seguem verdes sem tocar num número.
    pub rotation_deg: f64,
}

impl Default for PatternSpec {
    fn default() -> Self {
        Self {
            start_offset: 0.0,
            end_offset: f64::INFINITY,
            spacing: 1.0,
            offset: 0.0,
            flip: false,
            rotation_deg: 0.0,
        }
    }
}

/// A rotação da cópia como um par `(cos, sin)`, construído **uma vez por re-cook** — nunca por
/// vértice (o `sin_cos` é o único transcendental do módulo, e é assim que ele fica fora do laço).
///
/// O `identity` faz `rotation_deg == 0.0` devolver o ponto **intacto**. ⚠️ **Ele NÃO é
/// load-bearing para a geometria, e isso foi MEDIDO, não suposto:** com `cos = 1, sin = 0` a
/// multiplicação já é bit-exata para todo valor ordinário — os únicos bits que se movem são os de
/// um **zero negativo** (`-0.0` vira `+0.0`), que desenha idêntico. Ou seja: não existe gate
/// não-vácuo a escrever para este guard, porque para toda coordenada que um artista consegue
/// autorar ele é um no-op. Fica por tornar *"rotação zero não toca em nada"* verdade **por
/// construção** em vez de verdade por sorte da IEEE — e este comentário existe para o próximo
/// leitor não ir procurar o gate que o justificaria.
#[derive(Copy, Clone)]
struct Rotor {
    cos: f64,
    sin: f64,
    identity: bool,
}

impl Rotor {
    fn new(deg: f64) -> Self {
        if deg == 0.0 {
            return Self {
                cos: 1.0,
                sin: 0.0,
                identity: true,
            };
        }
        let (sin, cos) = deg.to_radians().sin_cos();
        Self {
            cos,
            sin,
            identity: false,
        }
    }

    fn apply(self, p: [f64; 2]) -> [f64; 2] {
        if self.identity {
            return p;
        }
        [
            p[0] * self.cos - p[1] * self.sin,
            p[0] * self.sin + p[1] * self.cos,
        ]
    }
}

/// A largura do bbox do motivo **no referencial girado** (ao longo de x, que é a direção de marcha
/// na guia) e o **centro** desse mesmo bbox.
///
/// Porta ÚNICA da medida: o avanço por cópia e a centragem da cópia na fatia saem os DOIS daqui, do
/// mesmo bbox — se saíssem de medidas diferentes, girar o motivo desalinharia o desenho do
/// espaçamento sem que nada acusasse (§ do módulo).
///
/// Sobre âncoras **e alças**: o casco convexo dos pontos de controle limita a curva, então a caixa
/// que os inclui é um superconjunto que de fato contém o desenho — que é a medida certa para
/// encaixar cópias sem sobrepor. Uma caixa só das âncoras cortaria uma curva que estufa para fora
/// delas.
///
/// Com `rot` identidade (`rotation_deg == 0`) os pontos passam intactos e o resultado é o mesmo
/// número, ao bit, de antes de existir rotação.
fn motif_bbox(motif: &VecPath, rot: Rotor) -> (f64, [f64; 2]) {
    let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
    let mut seen = false;
    for v in motif.verts_all() {
        for p in [v.anchor, v.in_handle, v.out_handle] {
            seen = true;
            let p = rot.apply(p);
            for k in 0..2 {
                lo[k] = lo[k].min(p[k]);
                hi[k] = hi[k].max(p[k]);
            }
        }
    }
    if !seen {
        return (0.0, [0.0, 0.0]);
    }
    (
        hi[0] - lo[0],
        [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5],
    )
}

/// Um vértice do motivo levado ao mundo pelo referencial da cópia, RÍGIDO.
///
/// Âncora **e as duas alças** passam pelo mesmo afim — é isso que mantém a curva sendo a imagem da
/// curva. `corner_radius` é comprimento LOCAL e o afim é rotação + translação (sem escala), então
/// ele sobrevive intacto (a mesma razão do `map_vert` do Repeater) — e a rotação da cópia, sendo
/// também rígida, não o toca.
///
/// A ordem é **gira → recentra → leva ao mundo**: o `center` já veio medido no referencial girado
/// (por [`motif_bbox`]), então subtraí-lo depois de girar é o que põe a cópia no meio da fatia.
fn map_vert(v: &VecVertex, frame: &GlyphFrame, rot: Rotor, center: [f64; 2]) -> VecVertex {
    let m = |p: [f64; 2]| {
        let p = rot.apply(p);
        frame.apply([p[0] - center[0], p[1] - center[1]])
    };
    VecVertex {
        anchor: m(v.anchor),
        in_handle: m(v.in_handle),
        out_handle: m(v.out_handle),
        kind: v.kind,
        corner_radius: v.corner_radius,
    }
}

/// **Tila o `motif` ao longo da `guide`**, devolvendo **uma cópia por `VecPath`**.
///
/// Cada cópia é o motivo inteiro (contorno primário + subpaths) transformado, herdando o
/// `fill`/`stroke`/`fill_rule` dele. Uma cópia por path — e não um compound único com todos os
/// contornos — de propósito: um motivo com `fill_rule` (uma rosca `EvenOdd`) num compound só faria
/// o exterior de uma cópia furar o interior da vizinha. A cópia carrega `id` default (irrelevante:
/// a `LiveGeometry` é DESENHADA, não vive na cena) e `effects` vazia (o motivo já entra cozido).
///
/// Vazio se a guia é degenerada (`total <= 0`) ou nenhuma cópia cabe. A cópia `k` ocupa a fatia de
/// arco `[start + avanço·k, start + avanço·(k+1)]`, com o centro em `start + avanço·(k + ½)`; emitem-se
/// as cópias cuja **fatia inteira cabe** no trecho `[start, min(end, total)]` — nada transborda as
/// pontas (a cauda pode sobrar, plano 23 §3, e é isso que distingue de deixar meia-cópia pendurada).
///
/// ⚠️ Numa **cúspide** o [`GlyphFrame::on_path`] devolve `None` (não há tangente) e a cópia é
/// **pulada** — a mesma escolha do texto: sem direção, inventar um referencial poria a forma num
/// ângulo que ninguém autorou.
#[must_use]
pub fn pattern_along(motif: &VecPath, guide: &ArcPath, spec: &PatternSpec) -> Vec<VecPath> {
    let total = guide.total();
    if total <= 0.0 {
        return Vec::new();
    }
    // O rotor UMA vez por re-cook; a medida do motivo sai do referencial DELE (§ do módulo).
    let rot = Rotor::new(spec.rotation_deg);
    let (width, center) = motif_bbox(motif, rot);
    let advance = (width * spec.spacing).max(MIN_ADVANCE);
    let inv = 1.0 / advance;

    // As cópias cuja FATIA `[lead_k, lead_k + avanço]` cabe em `[0, total]`, `lead_k = start + avanço·k`.
    // `k_lo` = primeiro `k` com `lead_k >= 0`; `k_hi` = último com `lead_k + avanço <= total`. Fechado
    // para o laço ser O(cópias) mesmo com um `start_offset` muito negativo (saltar as cópias antes do
    // começo num laço à parte seria O(k) desperdiçado). `FIT_EPS` absorve o ruído da medida de arco na
    // fronteira do encaixe exato.
    // Ambos são FINITOS: `total > 0` já foi testado e `advance >= MIN_ADVANCE`, então `inv` e os dois
    // são finitos; `k_lo` nunca é NaN (o `.max(0.0)` o garante). Um `<` cru basta e é claro.
    // O limite superior é o FIM do trecho: `min(end_offset, total)` — o `end_offset` default é
    // `INFINITY`, então o `min` o reduz ao `total` (a curva inteira) sem um caso especial.
    let hi_bound = spec.end_offset.min(total);
    let k_lo = (-spec.start_offset * inv).ceil().max(0.0);
    let k_hi = ((hi_bound - spec.start_offset) * inv - 1.0 + FIT_EPS).floor();
    if k_hi < k_lo {
        return Vec::new();
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let count = ((k_hi - k_lo + 1.0).min(MAX_COPIES as f64)) as usize;

    let map_contour = |verts: &[VecVertex], frame: &GlyphFrame| -> Vec<VecVertex> {
        verts
            .iter()
            .map(|v| map_vert(v, frame, rot, center))
            .collect()
    };

    let mut out = Vec::new();
    for j in 0..count {
        #[allow(clippy::cast_precision_loss)]
        let k = k_lo + j as f64;
        let s = spec.start_offset + advance * (k + 0.5);
        let Some(frame) = GlyphFrame::on_path(guide, s, spec.offset, spec.flip) else {
            continue; // cúspide: sem tangente, pula a cópia (como o texto pula o glifo)
        };
        let subpaths = motif
            .subpaths
            .iter()
            .map(|c| Contour {
                verts: map_contour(&c.verts, &frame),
                closed: c.closed,
            })
            .collect();
        out.push(VecPath {
            verts: map_contour(&motif.verts, &frame),
            closed: motif.closed,
            fill: motif.fill.clone(),
            stroke: motif.stroke,
            subpaths,
            fill_rule: motif.fill_rule,
            ..VecPath::default()
        });
    }
    out
}

#[cfg(test)]
#[path = "pattern_path_tests.rs"]
mod tests;

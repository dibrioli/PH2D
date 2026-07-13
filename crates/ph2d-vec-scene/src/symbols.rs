//! **Símbolos** — módulo irmão de [`crate::shapes`].
//!
//! Coração, raio, lua, gota, escudo, etiqueta, engrenagem, cruz, check e banner: o conjunto
//! que todo app de diagrama traz. **Autorado na caixa unitária** ([`crate::space`]), com
//! `v = 0` no TOPO — a convenção das referências (SVG / ECMA-376) de onde estas formas vêm.
//!
//! ## A doutrina: autore no espaço natural, depois AJUSTE
//!
//! 1. Cada forma é construída no espaço em que a sua geometria é EXATA — o disco de raio 1
//!    da lua, o escudo de altura `s + √(R − ¼)`, o coração do ECMA que estoura a caixa em
//!    0,36%. Nada de deformar a fórmula para caber.
//! 2. Arco é **cúbica exata** ([`Unit::arc`], comprimento de handle `(4/3)·tan(Δ/4)·r`),
//!    nunca polígono.
//! 3. No fim, [`fit`]: a bbox da CURVA (âncoras **e** extremos de Bézier — a curva passeia
//!    fora do casco dos seus pontos) vira a caixa do gesto, exatamente.
//!
//! É o que o próprio ECMA-376 faz à mão: as constantes mágicas `hf = 105146` e
//! `vf = 110557` da `star5` são `1/cos 18°` e `2/(1 + cos 36°)` — os fatores de ajuste de um
//! pentagrama, pré-calculados na espec. Aqui a conta é feita no cozimento, para QUALQUER
//! parâmetro. O ganho: **nenhuma forma precisa de clamp anti-transbordo**. Os clamps que
//! sobram são de VALIDADE (auto-interseção, contenção do furo, inversão de mitra) e estão
//! anotados um a um.
//!
//! ## A orientação é o contrato
//!
//! Bico do coração para BAIXO · bico da gota para CIMA · ponta do escudo para BAIXO ·
//! cornetas da lua para a DIREITA · bico da etiqueta à ESQUERDA · cotovelo do check
//! embaixo-à-esquerda. A versão anterior deste módulo foi escrita direto em coordenadas de
//! mundo, tratando `cy − hh` como "o topo": **toda** forma assimétrica nasceu espelhada.
//! Os testes de orientação abaixo são a rede — enunciados como ordem RELATIVA no mundo,
//! nunca como constante mágica.

use crate::space::{Unit, Uv, add_sub, closed, fit, poly, punch, straighten};
use crate::{VecPath, VecVertex, VertexKind};

/// Funde a costura entre dois arcos que compartilham o MESMO ponto: fica UMA âncora, com o
/// handle de entrada de quem chega e o de saída de quem sai. Sem isto o contorno carrega
/// uma âncora duplicada — que o usuário vê, arrasta e não entende — e um micro-segmento
/// degenerado no meio da curva.
fn join(verts: &mut Vec<VecVertex>, next: Vec<VecVertex>) {
    let mut it = next.into_iter();
    let Some(first) = it.next() else { return };
    match verts.last_mut() {
        Some(last) => {
            last.out_handle = first.out_handle;
            // Duas curvas se encontrando num ponto ANGULOSO: os handles são independentes.
            last.kind = VertexKind::Corner;
        }
        None => verts.push(first),
    }
    verts.extend(it);
}

// ─────────────────────────────────────────────────────────────────────────────
// CORAÇÃO
// ─────────────────────────────────────────────────────────────────────────────

/// Deslocamento em `u` do handle de saída do vale (ECMA-376 `heart`: `dx2 = 10/48·w`).
const HEART_LOBE_U: f64 = 10.0 / 48.0;
/// Altura ABSOLUTA do mesmo handle (`y1 = −h/3`). É absoluta de propósito: fixar a altura
/// (e não o comprimento do handle) é o que faz `cleft` cavar o vale de verdade — com o
/// deslocamento fixo, aprofundar o vale arrastaria os lóbulos junto e o ajuste devolveria
/// quase a mesma silhueta.
const HEART_LOBE_V: f64 = -1.0 / 3.0;
/// Deslocamento em `u` do handle de chegada no bico (`dx1 = 49/48·w`).
const HEART_SIDE_U: f64 = 49.0 / 48.0;
/// Altura absoluta desse handle. Junto com [`HEART_SIDE_U`] fixa o ângulo interno do bico
/// em `2·atan(HEART_SIDE_U / (1 − HEART_SIDE_V))` = **107,4°** — a silhueta do ECMA.
const HEART_SIDE_V: f64 = 0.25;

/// **Coração** — o bico aponta para BAIXO, duas cúbicas.
///
/// A construção do ECMA-376 (`heart`), decodificada: vale em `(½, cleft)`, bico em `(½, 1)`,
/// e os dois handles em posição ABSOLUTA. Nos defaults da espec (`cleft = ¼`) os pontos de
/// controle são exatamente `(0,7083, −0,3333)` e `(1,5208, 0,25)` — e o coração do ECMA
/// **não cabe na própria caixa** (estoura 0,36% em `u`, sobra 1,24% em `v`): [`fit`] mede a
/// curva de verdade e o assenta, o que devolve a tabela "box-exact" do padrão-ouro.
///
/// `cleft` = profundidade do vale (fração da altura). Clamp de VALIDADE em `0,45`: acima
/// disso o vale desce abaixo dos handles e a curva se cruza.
#[must_use]
pub fn heart(a: [f64; 2], b: [f64; 2], cleft: f64) -> VecPath {
    let u = Unit::of(a, b);
    let c = cleft.clamp(0.02, 0.45);
    let mirror = |(x, y): Uv| (1.0 - x, y);

    let lobe: Uv = (0.5 + HEART_LOBE_U, HEART_LOBE_V);
    let side: Uv = (0.5 + HEART_SIDE_U, HEART_SIDE_V);

    let mut p = closed(vec![
        // O vale: sai pelo lóbulo direito, chega pelo esquerdo.
        VecVertex {
            anchor: u.p((0.5, c)),
            in_handle: u.p(mirror(lobe)),
            out_handle: u.p(lobe),
            kind: VertexKind::Corner,
            corner_radius: 0.0,
        },
        // O bico: chega pelo lado direito, sai pelo esquerdo.
        VecVertex {
            anchor: u.p((0.5, 1.0)),
            in_handle: u.p(side),
            out_handle: u.p(mirror(side)),
            kind: VertexKind::Corner,
            corner_radius: 0.0,
        },
    ]);
    fit(&u, &mut p);
    p
}

// ─────────────────────────────────────────────────────────────────────────────
// RAIO
// ─────────────────────────────────────────────────────────────────────────────

/// A grade do ECMA-376 (as coordenadas da espec são inteiros sobre 21600).
const BOLT_GRID: f64 = 21600.0;
/// Os **11 vértices** do `lightningBolt` do ECMA-376. O raio é a única forma irregular do
/// catálogo: **não há fórmula fechada** — a espec publica a tabela, desenhada à mão (os
/// passos não são uniformes; um gerador de escadinha regular não a reproduz). É esta tabela
/// que o gerador devolve no default, e o teste `the_bolt_reproduces_the_ecma_vertex_table`
/// prende isso. Zero RNG, ordem: bico de cima → aresta de ataque → bico de baixo → aresta
/// de fuga → ponta esquerda.
const BOLT_TABLE: [Uv; 11] = [
    (8472.0, 0.0),      // 0 — bico de cima
    (12860.0, 6080.0),  // 1
    (11050.0, 6797.0),  // 2 — degrau para trás
    (16577.0, 12007.0), // 3
    (14767.0, 12877.0), // 4 — degrau para trás
    (21600.0, 21600.0), // 5 — bico de baixo
    (10012.0, 14915.0), // 6
    (12222.0, 13987.0), // 7 — degrau para trás
    (5022.0, 9705.0),   // 8
    (7602.0, 8382.0),   // 9 — degrau para trás
    (0.0, 3890.0),      // 10 — ponta esquerda
];
/// Os pares (aresta de ataque, aresta de fuga) que se enfrentam ao longo da banda. O bico de
/// baixo (índice 5) é COMUM às duas arestas — lá a banda fecha num ponto e não tem par.
const BOLT_PAIR: [(usize, usize); 5] = [(0, 10), (1, 9), (2, 8), (3, 7), (4, 6)];
/// A espessura que a tabela do ECMA já traz. É o ponto de IDENTIDADE do parâmetro: em
/// `waist = 0,40` o gerador devolve a tabela byte a byte.
const BOLT_WAIST_CANON: f64 = 0.40;

/// **Raio** (lightning bolt) — a tabela do ECMA-376, com a espessura da banda parametrizada.
///
/// Cada par de vértices que se enfrentam (ataque × fuga) é escalado em torno do MEIO deles:
/// `waist` multiplica o meio-vetor que os separa. Em `waist = 0,40` o fator é 1 e a forma é
/// exatamente a da espec (inclusive a conicidade desenhada à mão, que estreita a banda em
/// direção ao bico de baixo); abaixo, a banda afina; acima, engorda. [`fit`] reassenta o
/// resultado — por isso a banda pode afinar sem "encolher" a forma dentro do gizmo.
///
/// Clamp de VALIDADE `[0,05 · 0,80]`: em `waist → 0` as duas arestas colapsam numa reta.
#[must_use]
pub fn bolt(a: [f64; 2], b: [f64; 2], waist: f64) -> VecPath {
    let u = Unit::of(a, b);
    let s = waist.clamp(0.05, 0.80) / BOLT_WAIST_CANON;

    let mut uv: [Uv; 11] = BOLT_TABLE.map(|(x, y)| (x / BOLT_GRID, y / BOLT_GRID));
    for (lead, trail) in BOLT_PAIR {
        let (lx, ly) = uv[lead];
        let (tx, ty) = uv[trail];
        let mid = (0.5 * (lx + tx), 0.5 * (ly + ty));
        let half = (0.5 * (tx - lx), 0.5 * (ty - ly));
        uv[lead] = (mid.0 - s * half.0, mid.1 - s * half.1);
        uv[trail] = (mid.0 + s * half.0, mid.1 + s * half.1);
    }

    let mut p = poly(&u, &uv);
    fit(&u, &mut p);
    p
}

// ─────────────────────────────────────────────────────────────────────────────
// LUA
// ─────────────────────────────────────────────────────────────────────────────

/// **Lua crescente** — as cornetas abrem para a DIREITA, a barriga fica à esquerda.
///
/// A lua ASTRONÔMICA, não a do ECMA (que junta os dois arcos num ângulo de 20,6°, o
/// "crescente de logotipo"): o terminador de verdade é a projeção do círculo de iluminação,
/// uma **meia-elipse de mesmo semi-eixo VERTICAL do limbo** — logo as duas curvas têm
/// tangente horizontal nas cornetas e o encontro é **tangencial** (cornetas de ângulo zero,
/// agulhadas). Isso vale para qualquer fase.
///
/// Autorado num disco de raio ½ centrado na caixa: o limbo é o semicírculo ESQUERDO e o
/// terminador vai da corneta de cima à de baixo com semi-eixo horizontal `phase/2`. A região
/// entre eles ocupa só a metade esquerda do disco — [`fit`] a estica de volta, e é isso que
/// leva as cornetas para `u = 1` (a borda direita) e a barriga para `u = 0`.
///
/// `phase` = o semi-eixo do terminador (fração do raio). Perto de 0 é meia-lua (terminador
/// reto); perto de 1, uma foice fina. Clamp de VALIDADE em `0,95`: em 1 a área é zero.
#[must_use]
pub fn moon(a: [f64; 2], b: [f64; 2], phase: f64) -> VecPath {
    let u = Unit::of(a, b);
    let term_u = 0.5 * phase.clamp(0.05, 0.95);
    let c: Uv = (0.5, 0.5);

    // Limbo: da corneta de BAIXO (90°) pela esquerda (180°) à de CIMA (270°).
    let limb = u.arc(c, 0.5, 0.5, 90.0, 180.0);
    // Terminador: de volta, da de cima à de baixo, também pela esquerda (sweep negativo).
    // O raio VERTICAL é o mesmo do limbo — é daí que sai a tangência nas cornetas.
    let term = u.arc(c, term_u, 0.5, 270.0, -180.0);

    let mut verts = Vec::with_capacity(4);
    // Corneta de baixo: chega pelo terminador, sai pelo limbo (as duas tangentes horizontais
    // apontam para o MESMO lado — é uma cúspide, não uma junção suave).
    verts.push(VecVertex {
        anchor: limb[0].anchor,
        in_handle: term[term.len() - 1].in_handle,
        out_handle: limb[0].out_handle,
        kind: VertexKind::Corner,
        corner_radius: 0.0,
    });
    verts.extend(limb[1..limb.len() - 1].iter().copied()); // a barriga
    // Corneta de cima: chega pelo limbo, sai pelo terminador.
    verts.push(VecVertex {
        anchor: limb[limb.len() - 1].anchor,
        in_handle: limb[limb.len() - 1].in_handle,
        out_handle: term[0].out_handle,
        kind: VertexKind::Corner,
        corner_radius: 0.0,
    });
    verts.extend(term[1..term.len() - 1].iter().copied()); // o terminador

    let mut p = closed(verts);
    fit(&u, &mut p);
    p
}

// ─────────────────────────────────────────────────────────────────────────────
// GOTA
// ─────────────────────────────────────────────────────────────────────────────

/// O bulbo mais gordo que a gota admite (quase meia caixa: em `½` o ápice degenera).
const DROP_BULB_BLUNT: f64 = 0.499;
/// O bulbo mais magro (bico afiado, pescoço alto).
const DROP_BULB_SHARP: f64 = 0.200;

/// **Gota** — o bico aponta para CIMA.
///
/// A construção correta é a das **tangentes**, não "um círculo com um triângulo em cima": o
/// bulbo é a elipse de centro `(½, 1 − ry)` e raios `(½, ry)`, o ápice é `(½, 0)`, e os dois
/// flancos são as retas TANGENTES à elipse tiradas do ápice. Mandando a elipse para o
/// círculo unitário (`X = (x−½)/½`, `Y = (y−(1−ry))/ry`) o ápice vira `(0, −m)` com
/// `m = (1−ry)/ry`, a polar dá `Y = −1/m` e daí
///
/// ```text
/// β = acos(1/m)      T± = (½ ± ½·sin β ,  (1−ry) − ry²/(1−ry))
/// ```
///
/// Afim preserva tangência, então os flancos encostam na elipse com **G1 exato** — sem o
/// vinco que aparece quando se liga o ápice aos extremos laterais do bulbo. O arco retido é
/// o resto do bulbo (`360° − 2β`, ≈ 248° no default).
///
/// `tip` = afiação (0 = bulbo gordo e bico rombo; 1 = bico agudo).
#[must_use]
pub fn drop(a: [f64; 2], b: [f64; 2], tip: f64) -> VecPath {
    let u = Unit::of(a, b);
    let t = tip.clamp(0.0, 1.0);
    let ry = DROP_BULB_BLUNT + t * (DROP_BULB_SHARP - DROP_BULB_BLUNT);
    let beta = (ry / (1.0 - ry)).acos().to_degrees(); // acos(1/m), m = (1−ry)/ry

    let mut verts = vec![u.corner((0.5, 0.0))]; // o ápice
    // Do ponto de tangência da direita, dando a volta pelo bulbo, até o da esquerda.
    verts.extend(u.arc((0.5, 1.0 - ry), 0.5, ry, beta - 90.0, 360.0 - 2.0 * beta));
    straighten(&mut verts, 0); // ápice → T₊ é RETA (é a própria tangente)
    let last = verts.len() - 1;
    straighten(&mut verts, last); // T₋ → ápice, idem

    let mut p = closed(verts);
    fit(&u, &mut p);
    p
}

// ─────────────────────────────────────────────────────────────────────────────
// ESCUDO
// ─────────────────────────────────────────────────────────────────────────────

/// O ombro: quanto o escudo desce RETO antes de os arcos começarem.
const SHIELD_SHOULDER: f64 = 0.30;
/// Raio das laterais em `curve = 1` — os dois centros coincidem e a base vira um semicírculo
/// (o escudo "de selo", barrigudo).
const SHIELD_R_ROUND: f64 = 0.50;
/// Raio em `curve = 0` — laterais quase retas (o escudo em pá / triângulo).
const SHIELD_R_STRAIGHT: f64 = 1.75;

/// **Escudo** heráldico ("heater") — a ponta aponta para BAIXO.
///
/// A construção da heráldica: retângulo de ombro `s`, e duas circunferências de raio `R`
/// **centradas na linha `v = s`**, em `u = R` e `u = 1 − R`. Cada arco sai do seu canto com
/// tangente VERTICAL (o raio é horizontal ali) — logo o encontro com a lateral reta é G1
/// para qualquer `R` —, e os dois se cruzam no eixo em
///
/// ```text
/// H = s + √(R² − (R − ½)²) = s + √(R − ¼)
/// ```
///
/// `R = 1` (raio = largura do escudo) é o **heater clássico**: a ponta é o terceiro vértice
/// do triângulo equilátero sobre a largura. O ajuste final normaliza `v` por `H`, o que
/// transforma os arcos circulares em elípticos — exatamente representáveis.
///
/// `curve` = curvatura das laterais: `0` quase reto, `0,6` o heater clássico (`R = 1`),
/// `1` a barriga redonda.
#[must_use]
pub fn shield(a: [f64; 2], b: [f64; 2], curve: f64) -> VecPath {
    let u = Unit::of(a, b);
    let k = curve.clamp(0.0, 1.0);
    let r = SHIELD_R_ROUND + (1.0 - k) * (SHIELD_R_STRAIGHT - SHIELD_R_ROUND);
    let s = SHIELD_SHOULDER;
    let drop_h = (r - 0.25).sqrt(); // H − s
    let phi = drop_h.atan2(r - 0.5).to_degrees(); // o ângulo do bico visto do centro

    let mut verts = vec![u.corner((0.0, 0.0)), u.corner((1.0, 0.0))];
    verts.extend(u.arc((1.0 - r, s), r, r, 0.0, phi)); // lateral direita: (1, s) → bico
    join(&mut verts, u.arc((r, s), r, r, 180.0 - phi, phi)); // bico → (0, s)
    straighten(&mut verts, 1); // (1,0) → (1,s) é reta
    let last = verts.len() - 1;
    straighten(&mut verts, last); // (0,s) → (0,0) também

    let mut p = closed(verts);
    fit(&u, &mut p);
    p
}

// ─────────────────────────────────────────────────────────────────────────────
// ETIQUETA
// ─────────────────────────────────────────────────────────────────────────────

/// Onde o ilhó fica, em fração do comprimento do bico (o furo mora DENTRO do afunilamento).
const TAG_EYE_AT: f64 = 0.60;
/// Folga do furo até a aresta do bico (1,0 = tangente).
const TAG_EYE_MARGIN: f64 = 0.80;
/// Raio dos dois cantos direitos (em `v`).
const TAG_CORNER: f64 = 0.10;

/// **Etiqueta** — o bico aponta para a ESQUERDA; o ilhó é um furo de verdade.
///
/// Bico em `(0, ½)`, afunilamento até `(p, 0)` e `(p, 1)`, corpo retangular com os dois
/// cantos DIREITOS arredondados. Canto e furo são **redondos no MUNDO** (o raio em `u`
/// carrega o `aspect` da caixa), não elipses achatadas pela proporção do gesto.
///
/// O furo tem um clamp de **contenção** — este é um clamp de verdade, não de bbox: o ilhó
/// não pode furar a borda. Mandando a elipse do furo para o círculo unitário, a distância do
/// centro à aresta do bico tem de ser ≥ 1, o que dá o teto exato
///
/// ```text
/// r_v ≤ ½·TAG_EYE_AT / √(1 + (½·aspect/p)²)
/// ```
///
/// `point` = comprimento do bico; `hole` = raio do ilhó (0 = sem furo).
#[must_use]
pub fn tag(a: [f64; 2], b: [f64; 2], point: f64, hole: f64) -> VecPath {
    let u = Unit::of(a, b);
    let p = point.clamp(0.05, 0.60);
    let asp = u.aspect().clamp(0.05, 20.0);

    // Canto redondo-no-mundo, encolhido se a caixa for alta a ponto de ele comer o corpo.
    let rv = TAG_CORNER.min(0.40 * (1.0 - p) / asp);
    let ru = rv * asp;

    let mut verts = vec![u.corner((0.0, 0.5)), u.corner((p, 0.0))];
    verts.extend(u.arc((1.0 - ru, rv), ru, rv, -90.0, 90.0)); // canto superior direito
    verts.extend(u.arc((1.0 - ru, 1.0 - rv), ru, rv, 0.0, 90.0)); // canto inferior direito
    verts.push(u.corner((p, 1.0)));
    straighten(&mut verts, 1); // topo reto
    straighten(&mut verts, 3); // lateral direita reta
    straighten(&mut verts, 5); // base reta

    let mut path = closed(verts);

    let eye_u = TAG_EYE_AT * p;
    let fit_v = 0.5 * TAG_EYE_AT / (1.0 + (0.5 * asp / p).powi(2)).sqrt();
    let eye_rv = hole.max(0.0).min(TAG_EYE_MARGIN * fit_v);
    if eye_rv > 1e-6 {
        // Sentido INVERSO ao do contorno externo (com `EvenOdd` o furo já seria furo, mas um
        // contorno interno bem orientado também vaza certo em `NonZero`).
        let mut eye = u.arc((eye_u, 0.5), eye_rv * asp, eye_rv, 0.0, -360.0);
        eye.pop(); // o último vértice repete o primeiro
        add_sub(&mut path, eye, true);
        punch(&mut path);
    }

    fit(&u, &mut path);
    path
}

// ─────────────────────────────────────────────────────────────────────────────
// ENGRENAGEM
// ─────────────────────────────────────────────────────────────────────────────

/// Fração do passo angular que o dente ocupa na RAIZ. `gear6` do ECMA dá
/// `2·atan(0,09263/0,35) / 60°` = **0,494** — o ciclo de trabalho de 50% clássico.
const GEAR_TOOTH_WIDTH: f64 = 0.494;
/// Largura do dente no TOPO ÷ largura na raiz (`(l3 − lFD)/l3` do `gear6`): os flancos são
/// desenhados para dentro.
const GEAR_TOOTH_TAPER: f64 = 0.62;
/// Fase: um dente no TOPO (o ângulo cresce de `+u` para `+v`, que aponta para baixo).
const GEAR_PHASE: f64 = -90.0;

/// **Engrenagem** — `N` dentes idênticos entre a circunferência de topo e a de raiz, mais o
/// furo central.
///
/// O gerador radial do padrão-ouro (o mesmo de sol e estrela): `Δ = 2π/N`, meio-ângulo do
/// dente na raiz `αr = (Δ/2)·0,494`, no topo `αt = αr·0,62`, raiz em `R₀ = ½·(1 − depth)`.
/// Cada dente é flanco reto → **arco de topo** → flanco reto → **arco de raiz** (`Δ − 2αr`):
/// os vales e os topos são arcos de verdade, não cordas. Os defaults reproduzem o `gear6`/
/// `gear9` do ECMA-376.
///
/// O furo é um contorno próprio de sentido invertido + `EvenOdd` — sem isso a engrenagem
/// seria um disco dentado.
#[must_use]
pub fn gear(a: [f64; 2], b: [f64; 2], teeth: f64, depth: f64, hole: f64) -> VecPath {
    let u = Unit::of(a, b);
    let n = teeth.clamp(3.0, 64.0).round() as usize;
    let d = depth.clamp(0.05, 0.60);
    let c: Uv = (0.5, 0.5);
    let (r_tip, r_root) = (0.5, 0.5 * (1.0 - d));
    let step = 360.0 / n as f64;
    let (ar, at) = (
        0.5 * step * GEAR_TOOTH_WIDTH,
        0.5 * step * GEAR_TOOTH_WIDTH * GEAR_TOOTH_TAPER,
    );

    let mut verts: Vec<VecVertex> = Vec::with_capacity(n * 4);
    let mut seams: Vec<usize> = Vec::with_capacity(n * 2);
    for i in 0..n {
        let th = GEAR_PHASE + step * i as f64;
        let tip = u.arc(c, r_tip, r_tip, th - at, 2.0 * at);
        seams.push(verts.len() + tip.len() - 1); // fim do topo → flanco RETO
        verts.extend(tip);
        let root = u.arc(c, r_root, r_root, th + ar, step - 2.0 * ar);
        seams.push(verts.len() + root.len() - 1); // fim do vale → flanco RETO
        verts.extend(root);
    }
    for s in seams {
        straighten(&mut verts, s);
    }
    if let Some(first) = verts.first_mut() {
        first.in_handle = first.anchor; // o flanco que FECHA o contorno também é reto
    }

    let mut path = closed(verts);
    let bore = (0.5 * hole.clamp(0.0, 1.0)).min(0.9 * r_root);
    if bore > 1e-6 {
        let mut eye = u.arc(c, bore, bore, 0.0, -360.0);
        eye.pop();
        add_sub(&mut path, eye, true);
        punch(&mut path);
    }

    fit(&u, &mut path);
    path
}

// ─────────────────────────────────────────────────────────────────────────────
// CRUZ
// ─────────────────────────────────────────────────────────────────────────────

/// **Cruz / mais** — o polígono de 12 quinas do `plus`/`mathPlus` do ECMA-376.
///
/// `arm` = espessura dos braços (fração da caixa). A tabela da espec, com as barras
/// centradas: `L = ½ − arm/2`, `R = ½ + arm/2` nos dois eixos.
#[must_use]
pub fn cross(a: [f64; 2], b: [f64; 2], arm: f64) -> VecPath {
    let u = Unit::of(a, b);
    let w = arm.clamp(0.05, 0.95);
    let (lo, hi) = (0.5 - 0.5 * w, 0.5 + 0.5 * w);
    let mut p = poly(
        &u,
        &[
            (0.0, lo),
            (lo, lo),
            (lo, 0.0),
            (hi, 0.0),
            (hi, lo),
            (1.0, lo),
            (1.0, hi),
            (hi, hi),
            (hi, 1.0),
            (lo, 1.0),
            (lo, hi),
            (0.0, hi),
        ],
    );
    fit(&u, &mut p);
    p
}

// ─────────────────────────────────────────────────────────────────────────────
// CHECK
// ─────────────────────────────────────────────────────────────────────────────

/// Braço curto ÷ braço longo.
const CHECK_ARM_RATIO: f64 = 0.40;
/// Comprimento do braço longo, no espaço de autoria (o ajuste final normaliza).
const CHECK_LONG: f64 = 1.0;
/// Subida dos DOIS braços. 45°/45° = ângulo reto no cotovelo — a convenção medida no
/// Material Design e no Feather.
const CHECK_ANGLE: f64 = 45.0;
/// Teto de VALIDADE da espessura. A aresta interna do braço curto tem comprimento
/// `Ls − τ/2`: em `τ = 2·Ls` ela some e a mitra inverte. `1,5·Ls` guarda a margem.
const CHECK_MITER_MAX: f64 = 1.5 * CHECK_ARM_RATIO * CHECK_LONG;

/// **Check** (o "certo") — o cotovelo fica EMBAIXO à ESQUERDA.
///
/// Centro-linha de três pontos (ponta curta ↖, cotovelo, ponta longa ↗) deslocada de `±τ/2`
/// por **mitra exata**: para normais unitárias `n₁`, `n₂` do mesmo lado,
///
/// ```text
/// m = (n₁ + n₂) / (1 + n₁·n₂)        |m| = 1/cos(θ/2)
/// ```
///
/// (num cotovelo reto, `n₁·n₂ = 0` e a mitra mede `(τ/2)·√2` — bate com os números do
/// Material). Sai um hexágono de pontas cortadas a esquadro (cap `Butt`, o canônico).
///
/// `weight` = espessura do traço.
#[must_use]
pub fn check(a: [f64; 2], b: [f64; 2], weight: f64) -> VecPath {
    let u = Unit::of(a, b);
    let half = 0.5 * weight.clamp(0.02, CHECK_MITER_MAX);
    let (sin_a, cos_a) = CHECK_ANGLE.to_radians().sin_cos();

    let elbow: Uv = (0.0, 0.0);
    let short: Uv = (
        -CHECK_ARM_RATIO * CHECK_LONG * cos_a,
        -CHECK_ARM_RATIO * CHECK_LONG * sin_a,
    );
    let long: Uv = (CHECK_LONG * cos_a, -CHECK_LONG * sin_a);

    let dir = |from: Uv, to: Uv| {
        let (dx, dy) = (to.0 - from.0, to.1 - from.1);
        let len = dx.hypot(dy).max(1e-12);
        (dx / len, dy / len)
    };
    let (d1, d2) = (dir(short, elbow), dir(elbow, long));
    let (n1, n2) = ((d1.1, -d1.0), (d2.1, -d2.0));
    let k = 1.0 + n1.0 * n2.0 + n1.1 * n2.1;
    let miter: Uv = ((n1.0 + n2.0) / k, (n1.1 + n2.1) / k);
    let off = |p: Uv, n: Uv, side: f64| (p.0 + side * half * n.0, p.1 + side * half * n.1);

    let mut p = poly(
        &u,
        &[
            off(short, n1, 1.0),
            off(elbow, miter, 1.0),
            off(long, n2, 1.0),
            off(long, n2, -1.0),
            off(elbow, miter, -1.0),
            off(short, n1, -1.0),
        ],
    );
    fit(&u, &mut p);
    p
}

// ─────────────────────────────────────────────────────────────────────────────
// BANNER
// ─────────────────────────────────────────────────────────────────────────────

/// Largura do painel central (`adj2` do `ribbon` do ECMA-376).
const BANNER_PANEL: f64 = 0.50;
/// Altura da faixa dobrada (`adj1`).
const BANNER_FOLD: f64 = 1.0 / 6.0;
/// Raio HORIZONTAL dos cantos da dobra (`1/32` da espec). O vertical é `fold/4` — os cantos
/// são ELÍPTICOS, não circulares, e a razão entre os dois (0,75) é o que dá o formato da
/// dobra. Numa caixa larga, um raio em fração da LARGURA esticaria a dobra num gancho; por
/// isso o raio viaja com o `aspect` (a dobra é um detalhe de tamanho fixo, não uma proporção
/// da largura) — em caixa quadrada o valor volta a ser exatamente o da espec.
const BANNER_RX: f64 = 1.0 / 32.0;
/// Teto de VALIDADE do entalhe: em `notch = x2` o "V" da cauda alcançaria a borda do painel
/// e cortaria a fita ao meio.
const BANNER_NOTCH_MAX: f64 = 0.90;

/// O **vinco** de uma dobra: a aresta que dobra PARA TRÁS. Sai da silhueta no ombro
/// `(x5, ry)`, curva, volta para a esquerda e desce até o topo do painel — e fecha a face
/// dobrada contra o vinco vertical. É a face `darkenLess` da espec, contornada.
///
/// Espelhar em `u` é: `x → 1 − x`, ângulo `a → 180° − a`, sweep `s → −s`.
fn banner_crease(u: &Unit, rx: f64, ry: f64, x3: f64, x4: f64, mirrored: bool) -> Vec<VecVertex> {
    let m = |x: f64| if mirrored { 1.0 - x } else { x };
    let deg = |a: f64| if mirrored { 180.0 - a } else { a };
    let sw = |s: f64| if mirrored { -s } else { s };

    // O ombro: a metade do arco da cauda que fica DENTRO da forma (a de fora é silhueta).
    let mut cr = u.arc((m(x4), ry), rx, ry, deg(0.0), sw(90.0));
    let seam = cr.len() - 1;
    // O degrau de volta e a descida até o topo do painel.
    cr.extend(u.arc((m(x3), 3.0 * ry), rx, ry, deg(-90.0), sw(-180.0)));
    straighten(&mut cr, seam);
    let seam = cr.len() - 1;
    // E a base da face, de volta ao vinco vertical da silhueta.
    cr.push(u.corner((m(x4 + rx), 4.0 * ry)));
    straighten(&mut cr, seam);
    cr
}

/// **Banner / fita** — a dobra fica em CIMA, o painel pendura embaixo (o `ribbon` do
/// ECMA-376).
///
/// As caudas ocupam `v ∈ [0, 1 − fold]` e o painel `v ∈ [fold, 1]`: a cauda é MAIS ALTA que o
/// painel, e o que os liga é a **dobra** — a fita passa para trás e volta. A silhueta da
/// dobra é o ombro arredondado da cauda descendo até o topo do painel; a aresta dobrada em
/// si (o vinco) entra como contorno ABERTO, e é ela que faz a fita parecer dobrada.
///
/// **Cuidado com a espec aqui:** o `moveTo/lnTo/arcTo` do `ribbon` desenha só as faces da
/// FRENTE — sozinho ele deixa um BURACO no meio da dobra, que o ECMA tapa com uma segunda
/// face (`fill="darkenLess"`, a parte de trás do papel). Uma forma nossa cozinha para UM
/// `VecPath`, então o contorno externo aqui é a silhueta da UNIÃO (frente + face dobrada) —
/// sem buraco — e a face dobrada aparece pelo seu vinco. Copiar o path da espec ao pé da
/// letra produz a fita furada.
///
/// `notch` = profundidade do "V" (rabo-de-peixe) de cada cauda.
#[must_use]
pub fn banner(a: [f64; 2], b: [f64; 2], notch: f64) -> VecPath {
    let u = Unit::of(a, b);
    // Raio da dobra: fixo em relação à ALTURA (ver [`BANNER_RX`]), com teto para não engolir
    // a cauda numa caixa muito alta.
    let rx = (BANNER_RX * u.aspect()).clamp(0.006, 0.055);
    let ry = 0.25 * BANNER_FOLD; // quatro deles empilham a dobra inteira (`4·ry = fold`)

    let x2 = 0.5 - 0.5 * BANNER_PANEL; // borda do painel
    let x3 = x2 + rx; // onde o vinco encosta no topo do painel
    let x4 = x3 + 2.0 * rx; // onde o ombro da dobra encosta na cauda
    let x5 = x4 + rx; // o vinco VERTICAL (borda da face dobrada)
    let (x6, x7, x8, x9) = (1.0 - x5, 1.0 - x4, 1.0 - x3, 1.0 - x2);
    let y_fold = 4.0 * ry; // = BANNER_FOLD: o topo do painel
    let y_tail = 1.0 - BANNER_FOLD; // a base das caudas
    let y_pan = 1.0 - ry; // centro dos cantos de baixo do painel
    let n = notch.clamp(0.0, BANNER_NOTCH_MAX * x2);
    let apex = 0.5 * y_tail; // o "V" morde no meio da cauda

    let mut verts = vec![u.corner((0.0, 0.0))];
    // Cada índice em `seams` é o ÚLTIMO vértice antes de uma RETA — sem `straighten` a reta
    // herda os handles do arco (o usuário vê e arrasta fantasmas).
    let mut seams: Vec<usize> = vec![0];

    // Ombro da dobra esquerda: da aresta da cauda até o vinco vertical.
    let arc = u.arc((x4, ry), rx, ry, -90.0, 90.0);
    seams.push(verts.len() + arc.len() - 1);
    verts.extend(arc);
    verts.push(u.corner((x5, y_fold))); // desce o vinco
    verts.push(u.corner((x6, y_fold))); // topo do painel
    seams.push(verts.len() - 1);
    // Ombro da dobra direita (espelho).
    let arc = u.arc((x7, ry), rx, ry, 180.0, 90.0);
    seams.push(verts.len() + arc.len() - 1);
    verts.extend(arc);

    verts.push(u.corner((1.0, 0.0))); // cauda direita, aresta de cima
    verts.push(u.corner((1.0 - n, apex))); // o "V"
    verts.push(u.corner((1.0, y_tail))); // cauda direita, aresta de baixo
    verts.push(u.corner((x9, y_tail)));
    seams.push(verts.len() - 1);

    for (c, start) in [((x8, y_pan), 0.0), ((x3, y_pan), 90.0)] {
        let arc = u.arc(c, rx, ry, start, 90.0);
        seams.push(verts.len() + arc.len() - 1);
        verts.extend(arc);
    }

    verts.push(u.corner((x2, y_tail)));
    verts.push(u.corner((0.0, y_tail))); // cauda esquerda, aresta de baixo
    verts.push(u.corner((n, apex))); // o "V"
    for s in seams {
        straighten(&mut verts, s);
    }

    let mut p = closed(verts);
    for mirrored in [false, true] {
        add_sub(&mut p, banner_crease(&u, rx, ry, x3, x4, mirrored), false);
    }
    fit(&u, &mut p);
    p
}

#[cfg(test)]
#[path = "symbols_tests.rs"]
mod tests;

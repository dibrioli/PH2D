//! ⭐⭐⭐ **A DEFORMAÇÃO DEBAIXO DE UM DAB MEDE-SE AO TAMANHO DO DAB, nunca num ponto.**
//!
//! # O report que a pediu
//!
//! *«melhor. quase bom. Talvez artefato inevitável devido à natureza das deformações do mesh»*
//! (dono, 2026-09-14, terceira foto do mesmo pincel). As waves anteriores puseram a tinta no texel
//! certo (W10) e deram-lhe a forma que a deformação endireita (W11/W11b) — e o que sobrava era
//! real. ⛔ **Mas não era inevitável, e a medição diz porquê e quanto.**
//!
//! # O mecanismo
//!
//! Uma malha é **afim por triângulo**. Dentro de UM triângulo a deformação é constante e a elipse
//! que o pincel pinta volta ao ecrã como um disco **exacto**. Um dab que se estende por VÁRIOS
//! triângulos é corrigido pela deformação do triângulo debaixo do CENTRO, e as partes dele que
//! caem nos vizinhos recebem a correcção errada.
//!
//! ⇒ a grandeza certa não é a deformação **no ponto**, é a que a malha de facto faz **sobre o
//! disco que o dab ocupa**: o melhor afim (mínimos quadrados) do mapa da malha sobre esse disco.
//!
//! # ⭐ Porque ela degenera no de sempre
//!
//! Se todas as amostras do bordo caem no MESMO triângulo do centro, não há nada para ajustar — o
//! afim daquele triângulo já é exacto ali — e a porta devolve a resposta de hoje **sem tocar num
//! float**. É o que mantém byte a byte todo dab pequeno, que é o caso comum. *«Byte a byte» não é
//! uma promessa que uns mínimos quadrados cumpram: é um `if`* (a mesma lei que a
//! [`ph2d_painter_brush::canvas_warp`] pagou na W11).
//!
//! # A MEDIÇÃO (sonda sobre dois leques, 4 raios e 3 pontos — 24 células)
//!
//! Redondeza da marca no ecrã (`1,000` = disco perfeito), pior caso de cada coluna:
//!
//! | regime | sem correcção | facete (W11b) | **ao tamanho do dab** |
//! |---|---|---|---|
//! | dab DENTRO de um triângulo | `1,86`–`2,31` | `1,006` | `1,006` (o mesmo, ao bit) |
//! | dab sobre `~2` triângulos | `2,20` | `1,21` | **`1,11`** |
//! | dab sobre `~4` triângulos, leque forte | `1,19` | `1,38` | **`1,18`** |
//!
//! ⛔⛔ **A linha do fundo é a que obrigou aquela wave:** com um pincel GRANDE sobre uma malha
//! grossa, a correcção da W11b deixava a marca **menos redonda do que não corrigir nada** (`1,383`
//! contra `1,188`) — porque ela aplica ao dab inteiro a deformação de um pedaço dele.
//!
//! # ⭐⭐⭐ E DEPOIS DISSO SOBRAVA `1,1`–`1,2`, QUE ESTA PORTA CHAMOU DE INEVITÁVEL — E NÃO ERA
//!
//! *«quase bom»* (dono, 2026-09-14, 4.ª foto, seta sobre a marca no ponto de maior dobra). A
//! redacção anterior desta secção dizia que o resíduo era *«inerente a uma elipse por dab»* e
//! nomeava dois diminuidores: **a malha mais fina** e **o pincel menor**.
//!
//! ⛔ **A primeira metade está REFUTADA por medição** (`sprite_mesh_warp_probe`, §1): variando SÓ a
//! contagem de triângulos, o desvio **estanca** — `32` → `1,347`, `128` → `1,129`, `512` → `1,101`,
//! `2 048` → `1,050`, `8 192` → `1,050`. Quadruplicar a malha não move o terceiro decimal, logo o
//! resíduo **não é facetagem**, e o `Smooth` do esqueleto — que é exactamente essa alavanca — não o
//! alcança.
//!
//! ⭐ A segunda metade era a pista: o resíduo cresce com o RAIO do pincel porque ele é a
//! **CURVATURA da dobra dentro do próprio dab**, e um mapa LINEAR não acompanha uma curva. Medido
//! na direcção que o motor avalia (texel → ecrã), aproximando o mapa por um polinómio de grau `g`:
//!
//! | raio do dab | `g = 1` | `g = 2` | `g = 3` |
//! |---|---|---|---|
//! | `0,06` | `1,054` | `1,010` | `1,008` |
//! | `0,125` | `1,118` | `1,021` | `1,005` |
//! | `0,20` | `1,197` | `1,055` | **`1,006`** |
//!
//! ⇒ esta porta devolve um **polinómio de grau `3`** ([`crate::sprite_mesh_fit`]), e a curvatura
//! viaja até ao kernel dentro da [`ph2d_painter_brush::FootprintCurve`]. Resultado no produto, pela
//! mesma sonda §1 que refutou a facetagem:
//!
//! | triângulos | antes | **depois** |
//! |---|---|---|
//! | `128` | `1,129` · `1,161` | `1,129` · **`1,069`** |
//! | `512` | `1,101` · `1,119` | **`1,067`** · **`1,015`** |
//! | `2 048` | `1,050` · `1,108` | **`1,010`** · **`1,007`** |
//! | `8 192` | `1,050` · `1,107` | **`1,003`** · **`1,002`** |
//!
//! ⭐⭐ **E agora ele CONVERGE com a malha**, que antes não convergia: a facetagem deixou de ser o
//! chão porque o que sobrava por cima dela era outra coisa.
//!
//! # O CUSTO, medido (`--release`, uma chamada por evento de ponteiro)
//!
//! | triângulos da malha | a deformação num PONTO | **ao tamanho do dab** |
//! |---|---|---|
//! | `128` (o `Fast`) | `0,07 µs` | `0,83 µs` |
//! | `2 048` | `0,78 µs` | `11,65 µs` |
//! | `7 688` (o tecto do `Smooth`) | `3,06 µs` | `43,38 µs` |
//!
//! ⭐ **Triplicar as amostras (`8` → `24`, que o grau `3` exige) custou `~7 %`** — de `40,7` para
//! `43,4 µs` no pior caso. ⚠️ **O recurso é a VARREDURA DOS TRIÂNGULOS, não a contagem de
//! amostras**, e é por isso que ela pode ser UMA passagem e o rejeito por caixa em UV existe. O
//! pior caso é `0,26 %` de um quadro de 60 fps. ⛔ Quem apontar (o picking, as caixas) passa
//! `[0, 0]` e paga a coluna do meio.
//!
//! # ⛔ As DUAS cercas, as duas medidas
//!
//! 1. **`FACETAS_MIN`** (aqui): abaixo de `8` facetas distintas sob o dab, o que o ajuste lê como
//!    curvatura são as ARESTAS das facetas — e a marca sai PIOR. A tabela está no corpo.
//! 2. **A amostrabilidade** ([`ph2d_painter_brush::FootprintDeform::is_sampleable`]): uma dobra
//!    violenta faz o mapa dobrar sobre si mesmo dentro do dab. As duas degradam para a elipse, que
//!    é o produto de ontem.

use crate::sprite_mesh::{SpriteMesh, barycentric, barycentric_raw, corners, uv_corners, warp_of};

/// ⭐⭐⭐ **A DEFORMAÇÃO DA ARTE SOB O DAB** — a matriz local **mais a curvatura**.
///
/// ⚠️⚠️ **Ela é gémea da `ph2d_painter_brush::canvas_warp::CanvasWarp` de propósito, e a fronteira
/// é que obriga:** aquela crate é **dev-dependency** desta (os gates reconciliam contra a lei
/// canónica em vez de a re-implementar), logo o tipo dela não pode atravessar em produção — e uma
/// dependência de release do renderer para o motor de pincel puxaria o pincel inteiro para o grafo
/// de desenho. ⛔ A conversão vive num sítio SÓ, na costura da shell, e há gate a mantê-la assim.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeshWarp {
    /// Ecrã por textura, adimensional — **identidade em repouso**. O que sempre viajou aqui.
    pub linear: [[f32; 2]; 2],
    /// Os graus `2` e `3` (`x²`, `x·y`, `y²`, `x³`, `x²·y`, `x·y²`, `y³`), com a entrada em raios do
    /// footprint e a saída na mesma escala de `linear`. ⛔ **Zero em repouso.**
    pub curve: [[f32; 7]; 2],
}

impl MeshWarp {
    /// A arte em repouso.
    #[must_use]
    pub const fn rest() -> Self {
        Self::linear([[1.0, 0.0], [0.0, 1.0]])
    }

    /// Só a matriz local, sem curvatura — o que as portas que **apontam** sabem responder.
    #[must_use]
    pub const fn linear(linear: [[f32; 2]; 2]) -> Self {
        Self {
            linear,
            curve: [[0.0; 7]; 2],
        }
    }
}

/// Quantos ÂNGULOS o footprint leva. ⚠️ **As harmónicas angulares de um polinómio de grau `3` vão
/// até `3`, logo o mínimo estrito é `7`** — `12` resolve-as com folga. (A redacção anterior media
/// `8` contra `32` para um ajuste LINEAR, cuja harmónica é `1`: *uma contagem de amostras é função
/// do GRAU que se ajusta, e aquele número respondia a outra pergunta.*)
const ANGULOS: usize = 12;

/// Quantos ANÉIS. ⛔ **Com um só o sistema é singular por construção** — nada separa o que é grau
/// `2` do que é grau `3` se todas as amostras estiverem ao mesmo raio.
const ANEIS: usize = 2;

/// O total. O recurso é a contagem de testes ponto-em-triângulo por evento de ponteiro, e a
/// varredura continua a ser **UMA** passagem sobre a malha.
const AMOSTRAS: usize = ANGULOS * ANEIS;

/// Quantas FACETAS distintas as amostras têm de tocar para a curvatura ser lida. Ver a cerca no
/// corpo de [`warp_over`], que tem a tabela medida.
const FACETAS_MIN: usize = 8;

/// O ponto LOCAL POSADO de `t` nos pesos `w`.
fn posado(mesh: &SpriteMesh, t: [usize; 3], w: [f32; 3]) -> [f32; 2] {
    let l = corners(mesh, t);
    [
        w[0] * l[0][0] + w[1] * l[1][0] + w[2] * l[2][0],
        w[0] * l[0][1] + w[1] * l[1][1] + w[2] * l[2][1],
    ]
}

/// ⭐⭐⭐ **A PORTA**: a deformação que a malha faz sobre o disco de raio `footprint_uv` (em UV de
/// REPOUSO, por eixo) à volta do ponto POSADO `p`. Ver o cabeçalho do módulo.
///
/// ⭐⭐⭐ **Ela devolve um POLINÓMIO, não uma matriz** — a parte linear de sempre **mais** os graus
/// `2` e `3`, que são a curvatura da dobra dentro do próprio dab. O porquê, com a tabela, está em
/// [`crate::sprite_mesh_fit`]; o que a consome é a [`ph2d_painter_brush::canvas_warp::warped_dab`].
///
/// `footprint_uv` nulo ou não finito ⇒ a deformação da facete, que é a lei de [`warp_of`] e o que
/// as portas que não pintam (o picking, as caixas) querem — ali a curvatura é **plana**, e é isso
/// que as mantém a pagar o que pagavam.
#[must_use]
pub(crate) fn warp_over(
    mesh: &SpriteMesh,
    p: [f32; 2],
    size: [f32; 2],
    footprint_uv: [f32; 2],
) -> Option<MeshWarp> {
    if size[0] <= 0.0 || size[1] <= 0.0 {
        return None;
    }
    // ⚠️ **O triângulo é o do POSADO, e por isso a dobra obedece à mesma regra do desenho** (ganha o
    // desenhado por último) — é a razão de não se perguntar em UV quem contém o centro.
    let (t0, w0) = mesh
        .triangles()
        .rev()
        .find_map(|t| barycentric(p, corners(mesh, t)).map(|w| (t, w)))?;
    let facete = warp_of(mesh, t0, size)?;
    let (fx, fy) = (footprint_uv[0], footprint_uv[1]);
    if !(fx.is_finite() && fy.is_finite()) || fx <= 0.0 || fy <= 0.0 {
        return Some(MeshWarp::linear(facete));
    }
    let uv0 = uv_corners(mesh, t0);
    let centro = [
        w0[0] * uv0[0][0] + w0[1] * uv0[1][0] + w0[2] * uv0[2][0],
        w0[0] * uv0[0][1] + w0[1] * uv0[1][1] + w0[2] * uv0[2][1],
    ];
    // As amostras, em UV de repouso — e a MESMA amostra na coordenada normalizada do footprint
    // (`|p̂| ≤ 1`), que é a variável em que o polinómio vive.
    let mut amostra_uv = [[0.0f32; 2]; AMOSTRAS];
    let mut amostra_norm = [[0.0f32; 2]; AMOSTRAS];
    for anel in 0..ANEIS {
        // ⚠️ O anel de dentro fica a `0,6` e não a meio: é onde o termo cúbico ainda tem sinal
        // sobre o ruído da malha, e mais perto do centro ele afoga-se no quadrático.
        let rho = if anel + 1 == ANEIS {
            1.0
        } else {
            0.6 * (anel + 1) as f32 / (ANEIS - 1).max(1) as f32
        };
        for ang in 0..ANGULOS {
            let k = anel * ANGULOS + ang;
            let a = (ang as f32) * std::f32::consts::TAU / (ANGULOS as f32);
            let (sen, cos) = a.sin_cos();
            amostra_norm[k] = [rho * cos, rho * sen];
            amostra_uv[k] = [
                centro[0] + fx * amostra_norm[k][0],
                centro[1] + fy * amostra_norm[k][1],
            ];
        }
    }
    // ⭐ **UMA passagem sobre os triângulos** (o custo de um `uv_under`), com um rejeito por caixa
    // em UV à frente: sem ele seriam `AMOSTRAS` varreduras, e o `Smooth` refina a malha a milhares
    // de peças dentro do quadro.
    let (lo, hi) = (
        [centro[0] - fx, centro[1] - fy],
        [centro[0] + fx, centro[1] + fy],
    );
    let mut posicao = [[0.0f32; 2]; AMOSTRAS];
    let mut achou = [false; AMOSTRAS];
    let mut escapou = false;
    // ⭐ Quantas FACETAS distintas as amostras tocam — a régua de se a malha resolve a dobra ou se
    // o que o ajuste vê são as arestas dela. Ver [`FACETAS_MIN`].
    let mut facetas = 1usize;
    for t in mesh.triangles() {
        if t == t0 {
            continue;
        }
        let c = uv_corners(mesh, t);
        if c.iter().all(|q| q[0] < lo[0])
            || c.iter().all(|q| q[0] > hi[0])
            || c.iter().all(|q| q[1] < lo[1])
            || c.iter().all(|q| q[1] > hi[1])
        {
            continue;
        }
        let mut escapou_nesta = false;
        for k in 0..AMOSTRAS {
            if achou[k] {
                continue;
            }
            if let Some(w) = barycentric(amostra_uv[k], c) {
                posicao[k] = posado(mesh, t, w);
                achou[k] = true;
                if !escapou_nesta {
                    escapou_nesta = true;
                    facetas += 1;
                }
                escapou = true;
            }
        }
    }
    // ⭐⭐⭐ **NADA escapou do triângulo do centro ⇒ a resposta de hoje, ao bit.** O afim dele é
    // exacto sobre o footprint inteiro, e uns mínimos quadrados sobre ele devolveriam o mesmo
    // número com o ruído de um `f32` por cima — que o atalho da identidade do pincel não perdoa.
    if !escapou {
        return Some(MeshWarp::linear(facete));
    }
    // ⚠️ **Uma amostra FORA da malha responde pelo afim do centro** — ela contribui exactamente o
    // que a facete já diz, logo a borda da arte comporta-se como hoje em vez de puxar o ajuste.
    for k in 0..AMOSTRAS {
        if !achou[k] {
            let Some(w) = barycentric_raw(amostra_uv[k], uv0) else {
                return Some(MeshWarp::linear(facete));
            };
            posicao[k] = posado(mesh, t0, w);
        }
    }
    let p0 = posado(mesh, t0, w0);
    // ⭐ **O raio do footprint em unidades LOCAIS** — a média geométrica dos dois semi-eixos.
    //
    // ⚠️ **A cerca, nomeada:** os dois semi-eixos só diferem se a sprite estiver esticada de
    // maneira NÃO-uniforme em relação à imagem dela, e aí um dab redondo já chega ao ecrã
    // elíptico por uma lei que esta porta não conhece (o `√|det|` do traço vectorial). A curvatura
    // fica aproximada exactamente nesse caso, e a parte linear — que é quem carrega o grosso —
    // continua exacta por construção.
    let (ax, ay) = (fx * size[0], fy * size[1]);
    let rf = (ax * ay).sqrt();
    if !(rf.is_finite() && rf > 0.0) {
        return Some(MeshWarp::linear(facete));
    }
    // A saída em unidades do raio do footprint, na base de ECRÃ (`y` para baixo — a mesma das duas
    // pontas, que é o que a W11b pagou para aprender).
    let mut saida = [[0.0f32; 2]; AMOSTRAS];
    for k in 0..AMOSTRAS {
        saida[k] = [
            (posicao[k][0] - p0[0]) / rf,
            -(posicao[k][1] - p0[1]) / rf,
        ];
    }
    // ⭐⭐⭐ **A CERCA DAS FACETAS.** Um ajuste de grau `3` sobre uma malha grossa demais não lê a
    // dobra da ARTE: lê as ARESTAS das facetas. O mapa de uma malha é afim por peça, e um cúbico
    // que atravessa duas ou três peças oscila entre as amostras em vez de descrever alguma coisa.
    //
    // ⚠️⚠️ **O PISO É MEDIDO, varrendo-o contra a linha de base** (`FACETAS_MIN` efectivamente
    // infinito = só a recta). Redondeza da marca no leque de `1,8 rad`, os dois raios de dab:
    //
    // | triângulos | só a recta | piso `4` | piso **`8`** | piso `12` |
    // |---|---|---|---|---|
    // | `32` | `1,316` · `1,312` | `1,133` · `1,453` ⛔ | `1,316` · `1,312` | `1,316` · `1,312` |
    // | `128` | `1,129` · `1,161` | `1,095` · `1,069` | `1,129` · **`1,069`** | `1,129` · `1,161` ⛔ |
    // | `512` | `1,101` · `1,119` | `1,067` · `1,015` | **`1,067`** · **`1,015`** | `1,101` ⛔ · `1,015` |
    // | `2 048` | `1,050` · `1,108` | `1,010` · `1,007` | **`1,010`** · **`1,007`** | `1,010` · `1,007` |
    // | `8 192` | `1,050` · `1,107` | `1,003` · `1,002` | **`1,003`** · **`1,002`** | `1,003` · `1,002` |
    //
    // ⇒ **`8` é o MENOR piso em que nenhuma célula fica pior que a recta**, e `12` já deita fora
    // ganhos reais (as duas células marcadas). ⛔ Não é uma margem de segurança: é o teorema da
    // amostragem — não há dobra a ler onde não há peças que a resolvam, e o que o ajuste lê ali são
    // as ARESTAS das facetas.
    //
    // ⭐ E note-se a coluna de `8 192` contra a da recta: `1,107 → 1,002`. Abaixo do piso a resposta
    // é a RECTA, que é exactamente o produto de ontem.
    let cubico = (facetas >= FACETAS_MIN)
        .then(|| crate::sprite_mesh_fit::ajusta(&amostra_norm, &saida))
        .flatten();
    // A parte LINEAR volta à convenção de sempre (entrada `duv · size`, identidade em repouso): o
    // ajuste corre em `p̂`, e `p̂ = [pv₀/aₓ, pv₁/a_y]`.
    let recta = crate::sprite_mesh_fit::ajusta_linear(&amostra_norm, &saida);
    let (coef_lin, curve) = match (cubico, recta) {
        (Some(c), _) => (
            [[c[0][0], c[0][1]], [c[1][0], c[1][1]]],
            std::array::from_fn(|eixo| std::array::from_fn(|col| c[eixo][2 + col] as f32)),
        ),
        (None, Some(l)) => (l, [[0.0f32; 7]; 2]),
        (None, None) => return Some(MeshWarp::linear(facete)),
    };
    let linear = [
        [
            (coef_lin[0][0] * f64::from(rf / ax)) as f32,
            (coef_lin[0][1] * f64::from(rf / ay)) as f32,
        ],
        [
            (coef_lin[1][0] * f64::from(rf / ax)) as f32,
            (coef_lin[1][1] * f64::from(rf / ay)) as f32,
        ],
    ];
    // ⛔ Um ajuste COLAPSADO (uma dobra que fecha o footprint sobre si mesmo) não é a resposta a
    // nada: ali a facete continua a ser o que se vê debaixo do cursor.
    let dm = linear[0][0] * linear[1][1] - linear[0][1] * linear[1][0];
    if !linear.iter().flatten().all(|v| v.is_finite()) || dm.abs() < 1e-9 {
        return Some(MeshWarp::linear(facete));
    }
    if !curve.iter().flatten().all(|v| v.is_finite()) {
        return Some(MeshWarp::linear(facete));
    }
    Some(MeshWarp { linear, curve })
}

#[cfg(test)]
#[path = "sprite_mesh_warp_tests.rs"]
mod tests;

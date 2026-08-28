//! ⭐⭐⭐ **O ALISAMENTO QUE OLHA PARA O ÂNGULO** — a relaxação por ajuste de
//! quadrado, e a cura medida do defeito que o artista chamou «péssimo»
//! (2026-08-22).
//!
//! # ⛔ Porque o Laplaciano não podia curar isto
//!
//! [`crate::stitch`] aplica um **Laplaciano tangencial**: cada vértice anda na
//! direção do centróide dos vizinhos. ⚠️ **Ele trata a malha de quads como um
//! GRAFO** — só sabe quem é vizinho de quem — e por isso iguala **comprimentos de
//! aresta** e não sabe o que é um ângulo. *Um losango perfeito de 30° tem todas as
//! arestas iguais e é um ponto FIXO do Laplaciano.*
//!
//! ⭐ **E o defeito medido é exactamente esse.** Na `wrinkled_sphere`, com a
//! quantização a encomendar células praticamente quadradas (razão entre lados
//! vizinhos `1,14` na mediana), a saída entregava:
//!
//! | grandeza | oráculo | nós, com o Laplaciano |
//! |---|---|---|
//! | aspecto p50 | `1,08` | `1,28` — **quase certo** |
//! | ⛔ enviesamento p50 | `5°` | `18°` |
//! | ⛔ enviesamento p99 | `17°` | **`87°`** |
//! | ⛔ faces com um canto pior que 60° | **`0`** | **`8 281` de `29 468` (28 %)** |
//!
//! *O comprimento estava certo e o ângulo estava destruído* — que é a assinatura
//! de um alisador cego ao ângulo, e a razão de esta crate ter passado semanas com
//! réguas verdes sobre uma malha que o artista recusava.
//!
//! # ⭐ A lei: o quadrado mais próximo de quatro pontos tem forma FECHADA
//!
//! No plano do quad, com os quatro cantos vistos como números complexos
//! `z₀..z₃`, o conjunto dos quadrados com aquela ordem de cantos é o subespaço
//! gerado por `u = (1,1,1,1)` (a translação) e `v = (1, i, −1, −i)` (a forma).
//! ⚠️ **Os dois são ortogonais**, então a projecção de mínimos quadrados é uma
//! média pesada e não uma iteração:
//!
//! ```text
//!     c = ¼ Σ zₖ                          (o centro)
//!     a = ¼ (z₀ − i·z₁ − z₂ + i·z₃)       (a forma)
//!     wₖ = c + a·iᵏ                        (o quadrado mais próximo)
//! ```
//!
//! ⚠️ **Isto é a transformada discreta de Fourier de quatro pontos**, e `a` é o
//! primeiro harmónico. Um quadrado perfeito devolve-se a si mesmo (`|a|` é o
//! raio, os outros harmónicos são zero); um losango de 30° tem harmónico de
//! ordem 2 grande, e é ele que esta projecção **deita fora**.
//!
//! ⚠️ **A mão do quad decide-se medindo, não assumindo.** O harmónico da volta ao
//! contrário é `b = ¼ (z₀ + i·z₁ − z₂ − i·z₃)`, e para um quad bem formado ele é
//! zero. ⛔ **Um quad DOBRADO tem a volta invertida no plano dele** e `|b| > |a|`
//! — pedir-lhe o quadrado da mão errada puxaria os cantos para o lado oposto e
//! agravaria a dobra. *Escolher pelo maior módulo não é esconder um sinal: é a
//! pergunta «de que lado este quad está virado», respondida.*
//!
//! # A malha inteira: local-global
//!
//! Cada face diz onde gostaria que os seus quatro cantos estivessem; cada vértice
//! vai para a **média** dos pedidos que recebeu. ⚠️ É o esquema local-global das
//! famílias ARAP/shape-matching, e a ronda é uma contracção — por isso ela
//! converge e por isso [`LAMBDA`] amortece em vez de saltar.
//!
//! ⚠️ **Só a parte TANGENTE anda, e reprojecta-se sempre** — as duas leis que o
//! [`crate::stitch`] já pagou: a componente normal encolheria a peça a cada ronda,
//! e a reprojecção sem direcção é deliberada (uma normal estimada sobre a malha
//! que a própria ronda está a mexer realimenta-se; medido em 2026-08-22, as dobras
//! foram de 1 para 10).

use ph2d_mesh::Mesh;

/// ⛔⛔ **ZERO — MEDIDO E REJEITADO como cura** (2026-08-22). Ver este módulo,
/// que fica vivo e testado porque a **medição** é que é o resultado, não o código.
///
/// ⚠️ **O número sai da tabela, não de uma opinião** — e a tabela mede a grandeza
/// que o artista viu (`enviesamento`), não a que era fácil de medir.
///
/// # A hipótese, e porque era boa
///
/// O alisador de [`crate::stitch`] é um **Laplaciano**: trata a malha como um grafo,
/// iguala **comprimentos de aresta** e é cego ao ângulo — *um losango perfeito é
/// ponto fixo dele*. A relaxação por ajuste de quadrado ataca exactamente o que
/// falta: cada face pede o quadrado mais próximo de si (forma fechada, ver
/// [`nearest_square`]) e cada vértice vai para a média dos pedidos.
///
/// # ⛔ A tabela — orelha, `d = 1,0`, 78 403 quads
///
/// | rondas | aspecto p99 | aspecto max | `> 4×` | ⭐ **enviesamento p50** | `> 60°` | ⛔ **dobras** | ms |
/// |---|---|---|---|---|---|---|---|
/// | **0** | `7,4` | `122,7` | 3 558 | **`27°`** | 9 159 | **171** | 5 063 |
/// | 2 | `6,3` | `80,9` | 3 346 | `27°` | 8 801 | 306 | 6 346 |
/// | 4 | `5,4` | `38,6` | 3 032 | `26°` | 8 587 | 395 | 7 617 |
/// | 8 | `4,9` | `32,7` | 2 595 | `26°` | 8 276 | 497 | 10 275 |
/// | 16 | `4,6` | `30,3` | 2 143 | **`26°`** | 7 886 | **576** | 16 009 |
///
/// ⭐ **A cauda melhora muito** (o aspecto máximo cai `4×`) ⛔ **e a mediana do
/// enviesamento não se mexe: `27°` → `26°` em dezasseis rondas.** O preço são
/// `3,4×` mais dobras e `3,2×` o relógio.
///
/// # ⭐⭐⭐ O que a tabela PROVA, e é mais valioso que a feature
///
/// **Uma relaxação move vértices e mais nada.** Se dezasseis rondas de um método
/// cuja função-objectivo *é* a esquadria não movem a mediana, então endireitar um
/// quad desendireita o vizinho — ⇒ **o esmagamento está na CONECTIVIDADE**, em que
/// direcção as linhas da grade correm, e nenhum alisador lhe toca.
///
/// ⚠️ **E há um mecanismo para as dobras a mais:** num vértice irregular o pedido é
/// *contraditório* — três quads a pedir 90° cada somam 270° e têm de fechar 360°.
/// A relaxação puxa com força onde não existe solução, e a reprojecção (sem
/// direcção, deliberadamente — ver [`crate::stitch`]) aterra do lado errado do vinco.
///
/// ⭐ **A cura verdadeira ficou NOMEADA** pela sonda irmã (`sculpt3d_field_follow`):
/// medindo o desvio da grade ao campo cruzado **por família de linhas**, a nossa
/// primeira família segue o campo (`9,9°` no gancho) e a segunda não fica ortogonal
/// a ela (`19,2°` com as duas), enquanto no oráculo as duas quase coincidem
/// (`5,1°` → `7,6°`). É a assinatura da interpolação transfinita: casa com a
/// fronteira do patch e **enviesa no meio**. ⇒ *o interior de um patch tem de nascer
/// de uma parametrização alinhada ao campo* — e note-se que [`crate::fill_with`] nem sequer
/// **recebe** o campo.
///
/// ⛔ **Não volte a subir este número sem uma tabela nova.** Ligá-lo compra cauda e
/// paga dobras; o defeito que o artista fotografa é a mediana.
pub const SQUARE_ROUNDS: usize = 0;

/// **O amortecimento.** ⚠️ Meio passo, como no irmão Laplaciano: a projecção dá o
/// alvo, não o destino desta ronda.
const LAMBDA: f32 = 0.5;

/// ⭐⭐⭐ **N RONDAS de relaxação por ajuste de quadrado** — a porta pública desta lei.
///
/// `surface` é a malha em que a saída pousa (a que o artista fez), e é sobre ela que
/// cada ronda reprojecta.
///
/// # ⚠️ Por que ela nasce em 2026-08-28 e não em 2026-08-22
///
/// ⛔ A tabela de [`SQUARE_ROUNDS`] mediu esta mesma lei sobre a saída do
/// [`crate::fill`] — a montagem por patches, com `27°` de enviesamento mediano — e
/// concluiu, correctamente, que **o defeito estava na conectividade**: dezasseis
/// rondas de um método cuja função-objectivo *é* a esquadria moviam a mediana de
/// `27°` para `26°`.
///
/// ⭐⭐⭐ **Essa conclusão continua verdadeira e deixou de ser a nossa situação.** A
/// cadeia que shipa desde 2026-08-25 não monta patches: ela **extrai** as isolinhas
/// inteiras de um mapa de grade global, e a conectividade que sai dali entrega
/// `1,10` de aspecto e `3,8°`–`6,5°` de enviesamento à densidade do oráculo. *Uma
/// recusa medida responde UMA pergunta;* esta lei nunca correu sobre esta
/// conectividade, e a porta existe para que ela possa correr.
///
/// ⛔ **Ela NÃO é chamada por [`crate::fill`]** — lá a tabela ainda manda, e
/// [`SQUARE_ROUNDS`] continua `0`.
///
/// ⚠️ **Sem cerca de viagem** — é a porta das VARREDURAS. O produto usa
/// [`square_relax_capped`], e a cerca é o que separa as duas.
pub fn square_relax(mesh: &mut Mesh, surface: &Mesh, rounds: usize) -> SquareReport {
    square_relax_capped(mesh, surface, rounds, f32::INFINITY, 0.0)
}

/// O que uma corrida de relaxação mediu **de si própria**.
///
/// ⚠️ **A viagem é a coluna que decide**, e não o número de rondas: ver
/// [`square_relax_capped`].
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct SquareReport {
    /// Rondas que correram de facto — menos que o tecto quando ela assentou.
    pub rounds: usize,
    /// O maior movimento de um vértice na última ronda.
    pub last_move: f32,
    /// Mediana da distância entre onde cada vértice **acabou** e onde a extracção o pôs.
    pub travel_p50: f32,
    /// O maior desses.
    pub travel_max: f32,
    /// Quantos vértices bateram na cerca em alguma ronda.
    pub clamped: usize,
}

/// ⭐⭐⭐ **A RELAXAÇÃO COM CERCA DE VIAGEM** — a porta do produto.
///
/// `max_travel` é a distância máxima, em unidades de mundo, que um vértice pode acabar
/// **da posição em que a extracção o pôs**. `settle` termina cedo quando o maior movimento
/// de uma ronda desce abaixo dele.
///
/// # ⛔⛔⛔ Por que a cerca existe, e por que um número de rondas NÃO serve
///
/// Medido 2026-08-28 na `sculpt_wrinkled` à densidade grossa (531 quads), sem cerca:
///
/// | rondas | enviesamento p50 | ⛔ **relevo** |
/// |---|---|---|
/// | 0 | `8,6°` | **`11,9°`** |
/// | 80 | `5,3°` | `15,9°` |
/// | 160 | `4,0°` | `17,3°` |
/// | 320 | `3,1°` | ⛔ **`18,7°`** |
/// | 1280 | `2,7°` | ⛔ **`19,1°`** |
///
/// ⚠️ **`22,5°` é o valor de uma grade que IGNORA o relevo.** Sem cerca, a relaxação
/// converge para um ponto fixo que é mais quadrado **e quase cego**: ela desliza a grade
/// pela superfície até os quads serem quadrados, e ao fazê-lo apaga a única coisa que
/// distingue uma retopologia por campo cruzado de um remesh por voxel. *O Enio nomeou essa
/// propriedade no smoke de 24/08 — «obedece razoavelmente o relevo».*
///
/// ⛔ **E o número de rondas não pode ser a cerca**, porque a taxa de convergência depende
/// do tamanho da malha: `320` rondas mal se notam a `4 500` quads (relevo `11,7° → 13,6°`)
/// e quase cegam a `531` (`11,9° → 18,7°`). *Um tecto de rondas é uma cerca cujo tamanho
/// muda com a peça.*
///
/// ⭐ **A cerca é uma DISTÂNCIA, e a distância tem dono:** o alvo de aresta. A extracção
/// pôs cada vértice onde o campo mandou; o acabamento pode **polir** aquele vértice, não
/// **mudar a grade de sítio**.
pub fn square_relax_capped(
    mesh: &mut Mesh,
    surface: &Mesh,
    rounds: usize,
    max_travel: f32,
    settle: f32,
) -> SquareReport {
    square_relax_aligned(mesh, surface, rounds, max_travel, settle, 0.0)
}

/// ⭐⭐⭐ **A RELAXAÇÃO QUE OLHA PARA O RELEVO** — a porta que o produto usa.
///
/// `pull` multiplica a confiança de cada face (ver [`crate::quality::Hint`]) antes de ela
/// rodar o quadrado; `0` devolve [`square_relax_capped`] **ao bit**.
///
/// # ⛔⛔⛔ Por que ela existe: a relaxação cega CONVERGE PARA UMA GRADE CEGA
///
/// Medido 2026-08-28, `sculpt_wrinkled` grossa, sem alinhamento: o enviesamento mediano cai
/// de `8,6°` para `3,2°` **e o relevo sobe de `11,9°` para `18,8°`** (`22,5°` = uma grade
/// que ignora a forma). A relaxação desliza a grade pela superfície até os quads serem
/// quadrados, e ao fazê-lo apaga a única propriedade que distingue uma retopologia por campo
/// cruzado de um remesh por voxel. ⚠️ **O Enio nomeou essa propriedade no smoke de 24/08** —
/// *«obedece razoavelmente o relevo»*.
///
/// ⭐ **E a saída não é alisar menos.** O oráculo `quadwild-bimdf` corre um passe de
/// acabamento que compra forma **sem pagar relevo**: na `sculpt_wrinkled` ele vai de
/// `5,1° → 4,8°` de enviesamento com o relevo a ir de `7,1° → 7,0°`, e no `sculpt_hooked` o
/// relevo até **melhora** (`15,1° → 13,3°`). *Um acabamento que estraga o alinhamento não é
/// o acabamento certo com demasiadas rondas; é outro acabamento.*
///
/// ⛔ **E uma cerca de viagem sozinha foi MEDIDA e não serve** (`sculpt_wrinkled` grossa): a
/// `0,35 h` ela guarda o relevo (`11,6°`) e paga o `p99` do enviesamento — `52,8°` contra os
/// `34,5°` de hoje —, porque a cerca prende exactamente os vértices que mais precisavam de
/// andar. *A cerca limita a distância; o defeito não é distância, é direcção.*
pub fn square_relax_aligned(
    mesh: &mut Mesh,
    surface: &Mesh,
    rounds: usize,
    max_travel: f32,
    settle: f32,
    pull: f32,
) -> SquareReport {
    let mut rep = SquareReport::default();
    if rounds == 0 {
        return rep;
    }
    // ⚠️ **Amostrado UMA vez** — ver [`crate::quality::surface_hint`].
    let hint: Vec<crate::quality::Hint> = if pull > 0.0 {
        crate::quality::surface_hint(surface, mesh)
    } else {
        Vec::new()
    };
    let origin: Vec<[f32; 3]> = mesh.positions().to_vec();
    let floor = crate::finish::bbox_seed(surface);
    for r in 0..rounds {
        // ⭐⭐ **O raio da reprojecção encolhe com o movimento, e isso é EXACTO.** Depois da
        // 1.ª ronda todo vértice está **sobre** a superfície, então a distância dele a ela
        // é no máximo o que ele acabou de andar; uma esfera de `2×` esse movimento contém
        // com folga o pé mais próximo, e `faces_in_sphere` devolve toda face que a corte.
        // ⚠️ *Um raio grande não é mais correcto — é só mais caro*, e era ele que fazia a
        // ronda custar `~5,6 µs` por vértice.
        let seed = if r == 0 { floor } else { 1.0e-6 };
        let mv = square_once(mesh, surface, seed, &origin, max_travel, &hint, pull);
        rep.rounds = r + 1;
        rep.last_move = mv;
        if mv <= settle {
            break;
        }
    }
    let mut travel: Vec<f32> = mesh
        .positions()
        .iter()
        .zip(&origin)
        .map(|(p, o)| norm(sub(*p, *o)))
        .collect();
    // ⚠️ **Quem está NA cerca conta-se no RESULTADO, nunca durante o laço.** Um contador
    // incrementado por ronda mede vértice×ronda — um número que cresce com o tecto e que se
    // leria como «mais vértices presos».
    rep.clamped = travel
        .iter()
        .filter(|t| max_travel.is_finite() && **t >= max_travel * 0.999)
        .count();
    travel.sort_by(f32::total_cmp);
    rep.travel_p50 = travel.get(travel.len() / 2).copied().unwrap_or(0.0);
    rep.travel_max = travel.last().copied().unwrap_or(0.0);
    rep
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0].mul_add(b[0], a[1].mul_add(b[1], a[2] * b[2]))
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1].mul_add(b[2], -(a[2] * b[1])),
        a[2].mul_add(b[0], -(a[0] * b[2])),
        a[0].mul_add(b[1], -(a[1] * b[0])),
    ]
}

fn norm(a: [f32; 3]) -> f32 {
    dot(a, a).sqrt()
}

/// ⭐⭐⭐ **O QUADRADO MAIS PRÓXIMO de quatro pontos do plano** — a lei do módulo,
/// em forma fechada e sem iteração.
///
/// Recebe os quatro cantos **já centrados** (`Σ zₖ = 0`) e devolve os quatro
/// cantos do quadrado de mínimos quadrados, na mesma ordem.
///
/// ⚠️ **É `pub(crate)` e separada da ronda de propósito:** ela é a única parte
/// desta crate que é matemática pura, e uma troca de sinal aqui produziria uma
/// malha *plausível* e errada. *Uma lei que se pode testar sem malha nenhuma
/// testa-se sem malha nenhuma.*
#[must_use]
pub fn nearest_square(z: [[f32; 2]; 4]) -> [[f32; 2]; 4] {
    let (h, ccw) = square_harmonic(z);
    square_from(h, ccw)
}

/// ⭐ **O harmónico e a MÃO** — a metade da lei que decide *que quadrado*, separada da que
/// o escreve em cantos.
///
/// ⚠️ **Separada porque o alinhamento ao relevo entra AQUI** (ver [`steer`]): ele mantém
/// `|h|` — o tamanho que os quatro pontos pedem — e roda a fase. *Se a rotação entrasse
/// depois dos cantos, seria uma segunda lei a discordar desta.*
pub(crate) fn square_harmonic(z: [[f32; 2]; 4]) -> ([f32; 2], bool) {
    // `a` = harmónico da mão directa, `b` = da mão inversa. `i·(x,y) = (−y, x)`.
    let a = [
        0.25 * (z[0][0] + z[1][1] - z[2][0] - z[3][1]),
        0.25 * (z[0][1] - z[1][0] - z[2][1] + z[3][0]),
    ];
    let b = [
        0.25 * (z[0][0] - z[1][1] - z[2][0] + z[3][1]),
        0.25 * (z[0][1] + z[1][0] - z[2][1] - z[3][0]),
    ];
    let ccw = a[0].mul_add(a[0], a[1] * a[1]) >= b[0].mul_add(b[0], b[1] * b[1]);
    (if ccw { a } else { b }, ccw)
}

/// ⭐⭐⭐ **RODA O QUADRADO PARA A DIREÇÃO QUE A SUPERFÍCIE PEDE** — sem lhe tocar no tamanho.
///
/// `f` é a direção-alvo já no plano do quad (2-D, não precisa de estar normalizada) e `w` é
/// quanto ela vale, em `[0, 1]`.
///
/// # A lei
///
/// As **arestas** do quadrado `h·iᵏ` correm a `arg(h) + 45° + 90°k`, então a orientação de
/// uma grade é um ângulo **módulo 90°**. O desvio até ao alvo dobra-se para `[−45°, 45°]` —
/// *rodar 90° é a mesma grade* — e aplica-se `w` dele. ⚠️ **`w = 0` devolve `h` ao bit**, e
/// é isso que faz uma esfera (sem direção preferida) continuar a ver a lei do quadrado puro.
pub(crate) fn steer(h: [f32; 2], f: [f32; 2], w: f32) -> [f32; 2] {
    let lf = f[0].mul_add(f[0], f[1] * f[1]).sqrt();
    let lh = h[0].mul_add(h[0], h[1] * h[1]).sqrt();
    if w <= 0.0 || lf < 1.0e-12 || lh < 1.0e-12 {
        return h;
    }
    let quarter = std::f32::consts::FRAC_PI_2;
    // A orientação da GRADE é a da aresta, que está a 45° do harmónico.
    let edge = h[1].atan2(h[0]) + std::f32::consts::FRAC_PI_4;
    let target = f[1].atan2(f[0]);
    let mut d = (target - edge).rem_euclid(quarter);
    if d > quarter * 0.5 {
        d -= quarter;
    }
    let (s, c) = (w * d).sin_cos();
    [
        h[0].mul_add(c, -(h[1] * s)),
        h[0].mul_add(s, h[1] * c),
    ]
}

/// Escreve os quatro cantos do quadrado de harmónico `h` e mão `ccw`.
pub(crate) fn square_from(h: [f32; 2], ccw: bool) -> [[f32; 2]; 4] {
    let mut out = [[0.0f32; 2]; 4];
    for (k, o) in out.iter_mut().enumerate() {
        // `w = h · iᵏ` (ou `h · (−i)ᵏ` na mão inversa), em componentes.
        *o = match (k, ccw) {
            (0, _) => h,
            (1, true) | (3, false) => [-h[1], h[0]],
            (2, _) => [-h[0], -h[1]],
            _ => [h[1], -h[0]],
        };
    }
    out
}

/// **UMA RONDA de relaxação por ajuste de quadrado**, seguida de reprojecção.
///
/// ⚠️ **Faces que não são quads contribuem com a posição que já têm** — neutras,
/// não ausentes. Um vértice que só toca faces não-quad ficaria com `cnt = 0` e o
/// código teria de o tratar à parte; assim a lei é uma só. *A promessa desta
/// família é `100 %` de quads e o `non_quads` já a guarda; isto é a rede.*
pub(crate) fn square_once(
    mesh: &mut Mesh,
    reference: &Mesh,
    seed: f32,
    origin: &[[f32; 3]],
    max_travel: f32,
    hint: &[crate::quality::Hint],
    pull: f32,
) -> f32 {
    let n = mesh.vert_count();
    let before: Vec<[f32; 3]> = mesh.positions().to_vec();
    let mut acc = vec![[0.0f32; 3]; n];
    let mut cnt = vec![0u32; n];
    {
        let pos = mesh.positions();
        for (fi, f) in mesh.faces().iter().enumerate() {
            let v = f.verts();
            if v.len() != 4 {
                for &i in v {
                    let p = pos[i as usize];
                    for k in 0..3 {
                        acc[i as usize][k] += p[k];
                    }
                    cnt[i as usize] += 1;
                }
                continue;
            }
            let p = [
                pos[v[0] as usize],
                pos[v[1] as usize],
                pos[v[2] as usize],
                pos[v[3] as usize],
            ];
            let c3 = [
                0.25 * (p[0][0] + p[1][0] + p[2][0] + p[3][0]),
                0.25 * (p[0][1] + p[1][1] + p[2][1] + p[3][1]),
                0.25 * (p[0][2] + p[1][2] + p[2][2] + p[3][2]),
            ];
            // ⚠️ **Newell, não o produto de duas arestas.** Um quad alabeado — e
            // quase todos são, sobre uma superfície curva — não tem normal única;
            // Newell dá a do plano de mínimos quadrados, que é o plano onde o
            // ajuste tem de correr.
            let mut nrm = [0.0f32; 3];
            for k in 0..4 {
                let (a, b) = (p[k], p[(k + 1) % 4]);
                nrm[0] += (a[1] - b[1]) * (a[2] + b[2]);
                nrm[1] += (a[2] - b[2]) * (a[0] + b[0]);
                nrm[2] += (a[0] - b[0]) * (a[1] + b[1]);
            }
            let nl = norm(nrm);
            // Quad degenerado: sem plano não há ajuste, e forçar um seria inventar
            // uma direcção. Contribui neutro.
            if nl < 1.0e-12 {
                for &i in v {
                    let q = pos[i as usize];
                    for k in 0..3 {
                        acc[i as usize][k] += q[k];
                    }
                    cnt[i as usize] += 1;
                }
                continue;
            }
            let nu = [nrm[0] / nl, nrm[1] / nl, nrm[2] / nl];
            // A base do plano. ⚠️ `e2 = n × e1` faz `e1 × e2 = n`, e é isso que
            // garante que um quad enrolado no sentido directo em 3D se lê no
            // sentido directo em 2D — sem essa escolha o harmónico da mão certa
            // seria o outro.
            let r = sub(p[0], c3);
            let along = dot(r, nu);
            let e1r = [
                along.mul_add(-nu[0], r[0]),
                along.mul_add(-nu[1], r[1]),
                along.mul_add(-nu[2], r[2]),
            ];
            let e1l = norm(e1r);
            if e1l < 1.0e-12 {
                for &i in v {
                    let q = pos[i as usize];
                    for k in 0..3 {
                        acc[i as usize][k] += q[k];
                    }
                    cnt[i as usize] += 1;
                }
                continue;
            }
            let e1 = [e1r[0] / e1l, e1r[1] / e1l, e1r[2] / e1l];
            let e2 = cross(nu, e1);
            let mut z = [[0.0f32; 2]; 4];
            for k in 0..4 {
                let d = sub(p[k], c3);
                z[k] = [dot(d, e1), dot(d, e2)];
            }
            let (mut hz, ccw) = square_harmonic(z);
            // ⭐⭐⭐ **O RELEVO ENTRA AQUI** — ver [`steer`] e [`crate::quality::surface_hint`].
            // A direção-alvo vem em espaço de MUNDO e é lida no plano do próprio quad; sem
            // essa projecção uma direção quase perpendicular ao quad daria um ângulo que não
            // existe na face.
            if let Some(hint) = hint.get(fi) {
                if hint.weight > 0.0 {
                    let f2 = [dot(hint.dir, e1), dot(hint.dir, e2)];
                    hz = steer(hz, f2, (hint.weight * pull).clamp(0.0, 1.0));
                }
            }
            let w = square_from(hz, ccw);
            for k in 0..4 {
                let i = v[k] as usize;
                for t in 0..3 {
                    acc[i][t] += w[k][1].mul_add(e2[t], w[k][0].mul_add(e1[t], c3[t]));
                }
                cnt[i] += 1;
            }
        }
    }
    let normals: Vec<[f32; 3]> = mesh.normals().to_vec();
    let mut next = vec![[0.0f32; 3]; n];
    {
        let pos = mesh.positions();
        for v in 0..n {
            let p = pos[v];
            if cnt[v] == 0 {
                next[v] = p;
                continue;
            }
            #[allow(clippy::cast_precision_loss)]
            let inv = 1.0 / cnt[v] as f32;
            let d = [
                acc[v][0].mul_add(inv, -p[0]),
                acc[v][1].mul_add(inv, -p[1]),
                acc[v][2].mul_add(inv, -p[2]),
            ];
            let nv = normals[v];
            let along = dot(d, nv);
            let mut q = [
                LAMBDA.mul_add(along.mul_add(-nv[0], d[0]), p[0]),
                LAMBDA.mul_add(along.mul_add(-nv[1], d[1]), p[1]),
                LAMBDA.mul_add(along.mul_add(-nv[2], d[2]), p[2]),
            ];
            // ⭐⭐⭐ **A CERCA DE VIAGEM** — ver [`square_relax_capped`]. Ela mede da posição
            // que a EXTRACÇÃO deu, nunca da ronda anterior: uma cerca por-ronda seria um
            // limite de velocidade, e o que se quer limitar é a **distância percorrida**.
            if max_travel.is_finite() {
                if let Some(o) = origin.get(v) {
                    let t = sub(q, *o);
                    let l = norm(t);
                    if l > max_travel {
                        let s = max_travel / l;
                        q = [
                            t[0].mul_add(s, o[0]),
                            t[1].mul_add(s, o[1]),
                            t[2].mul_add(s, o[2]),
                        ];
                    }
                }
            }
            next[v] = q;
        }
    }
    // ⚠️ **O raio sai do MAIOR movimento desta ronda** — ver a nota em
    // [`square_relax_capped`]. Ele é decidido **antes** de projectar porque é a projecção
    // que ele governa, e o `seed` do chamador é o piso (a 1.ª ronda ainda não sabe se os
    // vértices estão sobre a superfície).
    let mut moved = 0.0f32;
    for (q, p) in next.iter().zip(&before) {
        moved = moved.max(norm(sub(*q, *p)));
    }
    let radius = (2.0 * moved).max(seed);
    for q in &mut next {
        *q = ph2d_remesh_iso::project_onto(reference, *q, radius);
    }
    let mut real = 0.0f32;
    for (q, p) in next.iter().zip(&before) {
        real = real.max(norm(sub(*q, *p)));
    }
    mesh.positions_mut().copy_from_slice(&next);
    mesh.rebuild();
    real
}

#[cfg(test)]
#[path = "relax_tests.rs"]
mod tests;

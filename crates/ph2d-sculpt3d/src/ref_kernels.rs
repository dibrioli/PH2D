//! **OS KERNELS DA REFERÊNCIA** — o porte 1:1 do SculptGL (MIT), aritmética
//! `f64` e armazenamento `f32`, gateado bit a bit contra o JS EXECUTANDO.
//!
//! # Por que um porte 1:1, e não uma tradução para o nosso vocabulário
//!
//! A ordem do Enio é **paridade bit-idêntica**, e ela não é uma exigência sobre
//! o *resultado* — é uma exigência sobre a **ESTRUTURA**. O original escreve
//!
//! ```text
//! vAr[ind] = vx + anx * fallOff       // f64 no cálculo, UMA arredondada no store
//! ```
//!
//! e o nosso motor escreve `lerp(base, target, accum)` sobre o `pre` congelado.
//! Os dois **não podem** coincidir, e não é questão de afinar constante: em
//! `lerp` o termo `(base + n·reach) − base` não devolve `n·reach` nem em `f64`
//! (a subtração perde os bits baixos que a soma acabou de introduzir), e o
//! resultado passa por **duas** arredondadas em vez de uma. *A estrutura do
//! kernel é a lei*, e é por isso que este módulo existe em vez de uma emenda no
//! `stroke.rs`.
//!
//! # A disciplina, e o precedente
//!
//! É a mesma da [`ph2d-wet-paint`], que porta o JS da aquarela: **`f64` na
//! aritmética, `f32` no armazenamento, semântica do JS onde ela difere**. Um
//! `Float32Array` do JS lê `f32 → f64`, faz a conta inteira em `f64` e arredonda
//! **uma vez** ao guardar; um `[f32]` de Rust com aritmética `f32` arredonda a
//! cada operação. Ignorar isto é a diferença entre *"o desenho ficou parecido"*
//! e *"os bits são os mesmos"*.
//!
//! # A fronteira deste módulo, dita para não ser esquecida
//!
//! Ele porta **o laço por-vértice, e só ele**. Ficam de fora, de propósito:
//!
//! - **a PEGADA** (quem está sob o pincel) — nosso octree responde, e o
//!   `verts_in_sphere` já tem gates próprios;
//! - **a SIMETRIA, o `pre` congelado e a janela de undo** — são do
//!   [`crate::SculptStroke`], e o original não tem equivalente delas;
//! - **o ESPAÇAMENTO** — o `walk` já é o `SculptBase.sculptStroke`.
//!
//! # ⚠️ A polaridade da máscara é INVERTIDA entre os dois mundos
//!
//! No original `mAr[ind + 2]` é um **multiplicador**: `1` é livre e `0` é
//! travado, e o kernel o multiplica direto. No nosso `Mesh` a máscara é o
//! contrário — `DEFAULT_MASK = 0` significa *totalmente esculpível*, e quem
//! converte é [`crate::mask_ops::free_weight`]. Este módulo fala a língua da
//! REFERÊNCIA (o parâmetro se chama `free`), porque é ela que ele reproduz; a
//! conversão mora em quem chama, uma vez, onde dá para ver.
//!
//! # ⚠️ O alpha entra como `1.0` e isso não é uma simplificação
//!
//! `picking.getAlpha` devolve `1.0` sem textura de alpha (`Picking.js:60-62`),
//! que é o `_idAlpha = 0` de fábrica das dez tools. O nosso padrão por imagem é
//! uma feature NOSSA, com kernel próprio, e enfiá-la aqui faria este módulo
//! deixar de ser um porte.
//!
//! Fonte: <https://github.com/stephaneginier/sculptgl>, `src/editing/tools/`.
//! Licença MIT (o texto está em `crates/ph2d-mesh-render/assets/matcaps/`).
//! Oráculo executável: `docs/3D/ferramentas/sculptgl_oracle.mjs`.

/// **A CURVA** — o falloff único da referência, e o número que ele dá.
///
/// ```text
/// var fallOff = dist * dist;
/// fallOff = 3.0 * fallOff * fallOff - 4.0 * fallOff * dist + 1.0;
/// ```
///
/// É `3d⁴ − 4d³ + 1`: vale `1` no centro, `0` na borda, e tem **derivada zero
/// nas duas pontas** — é isso que faz um traço não deixar degrau na fronteira
/// do pincel nem bico no meio dele.
///
/// ⚠️ **Ela NÃO é negativa fora do raio, ela CRESCE** — e a diferença importa
/// para o único kernel sem a guarda de raio (o [`pinch`]). Fatorando,
/// `3d⁴ − 4d³ + 1 = (d − 1)²(3d² + 2d + 1)`, e o quadrático tem discriminante
/// **−8**: ele nunca troca de sinal. Logo a curva é `≥ 0` em toda parte, com
/// raiz **DUPLA** em `d = 1` (é daí que sai a derivada zero na borda), e passa
/// a subir: `0,31` em `d = 1,2`, `2,69` em `1,5`, **`17` em `2,0`**. Quem
/// contém isso é a PEGADA, nunca a curva.
///
/// ⚠️ **A associação é a do JS, e o preço de mudá-la está MEDIDO — não é o que
/// eu tinha escrito aqui.** A primeira versão deste doc dizia que reordenar
/// *"muda os bits"*, e a mutação que reordena **sobreviveu ao gate**. Medido
/// sobre 4 M amostras:
///
/// - `4.0 * f * dist` → `4.0 * (f * dist)`: **zero** divergências. Quatro é
///   potência de dois, logo multiplicar por ele é EXATO, e as duas ordens são a
///   mesma arredondada de `f · dist`. A mutação era semanticamente neutra.
/// - `3.0 * f * f` → `3.0 * (f * f)`: **38,3%** das avaliações divergem, por
///   **≤ 4,4e-16** — sete ordens abaixo do ulp do `f32` em que o resultado é
///   guardado, e por isso invisível através do store.
///
/// A ordem fica sendo a da referência porque é a dela; o que **não** se pode
/// afirmar é que um gate a defende. Defesa em camadas documentada em vez de
/// gateada, pelo precedente do ADR-0145.
///
/// ⚠️ **Ela é a MESMA para as dez tools.** O original não tem seletor de
/// falloff, e é por isso que este `fn` não recebe um enum: um parâmetro que só
/// pode ter um valor seria um controle morto na assinatura.
#[inline]
#[must_use]
pub fn falloff(dist: f64) -> f64 {
    let f = dist * dist;
    3.0 * f * f - 4.0 * f * dist + 1.0
}

/// De onde a distância é medida — a **única** coisa que o checkbox *Accumulate*
/// do original controla.
///
/// ⚠️ **Ele não toca o alvo nem a quantidade, e isso contradiz o nome.** O
/// rótulo dele é *"Accumulate (no limit per stroke)"* (`english.js`), e o
/// mecanismo é: com o proxy, a distância sai da posição em que o vértice estava
/// no **primeiro toque deste traço**; o centro do pincel, porém, é o ponto de
/// intersecção com a superfície **VIVA**, que sobe junto com a tinta. Então a
/// distância entre um e outro **CRESCE** enquanto o artista empurra, o vértice
/// passa de `dist >= 1` e sai da pegada — *é daí que vem o limite por traço*.
/// Lendo a posição viva, os dois sobem juntos e o limite não existe.
///
/// Guardar isto num tipo, e não num `bool`, é o que impede a próxima leitura de
/// achar que ele multiplica alguma coisa.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Origin {
    /// `accumulate = true`: a distância sai da posição **VIVA**. Sem limite.
    Live,
    /// `accumulate = false`: a distância sai do proxy congelado no primeiro
    /// toque do traço. O pincel se esgota sozinho.
    Proxy,
}

/// Lê a posição de origem de um vértice — a resolução do [`Origin`].
///
/// ⚠️ **`Live` lê o MESMO buffer que o kernel escreve**, e é por isso que ele é
/// um `Option` em vez de dois slices: com `&mut` e `&` do mesmo array o
/// compilador recusa, e a alternativa (clonar as posições por dab) faria o
/// caminho quente pagar uma cópia por uma pergunta.
#[inline]
fn origin(pos: &[f32], proxy: Option<&[f32]>, ind: usize) -> [f64; 3] {
    match proxy {
        Some(p) => [f64::from(p[ind]), f64::from(p[ind + 1]), f64::from(p[ind + 2])],
        None => [
            f64::from(pos[ind]),
            f64::from(pos[ind + 1]),
            f64::from(pos[ind + 2]),
        ],
    }
}

/// A distância normalizada do vértice ao centro, e `None` quando ele está fora.
#[inline]
fn normalised_dist(o: [f64; 3], center: [f64; 3], radius: f64) -> Option<f64> {
    let dx = o[0] - center[0];
    let dy = o[1] - center[1];
    let dz = o[2] - center[2];
    let dist = (dx * dx + dy * dy + dz * dz).sqrt() / radius;
    // `>= 1.0` e não `> 1.0`: é o `if (dist >= 1.0) continue` do original, e a
    // borda exata é onde a curva vale zero de qualquer jeito — o que muda é o
    // TRABALHO, não o desenho.
    if dist >= 1.0 { None } else { Some(dist) }
}

/// **BRUSH** (`Brush.js:57-91`) — empurra ao longo da normal de ÁREA.
///
/// É o `_clay = false` do original; com o Clay armado — que é o **default de
/// fábrica** — o `Brush.stroke` nem chama isto: ele chama o [`flatten`] contra
/// um plano deslocado. Ver o doc de [`flatten`].
#[allow(clippy::too_many_arguments)]
pub fn brush(
    pos: &mut [f32],
    free: &[f32],
    verts: &[u32],
    proxy: Option<&[f32]>,
    a_normal: [f64; 3],
    center: [f64; 3],
    radius_squared: f64,
    intensity: f64,
    negative: bool,
) {
    let radius = radius_squared.sqrt();
    let mut deform = intensity * radius * 0.1;
    if negative {
        deform = -deform;
    }
    for &v in verts {
        let ind = v as usize * 3;
        let o = origin(pos, proxy, ind);
        let Some(dist) = normalised_dist(o, center, radius) else {
            continue;
        };
        let f = falloff(dist) * f64::from(free[v as usize]) * deform;
        pos[ind] = (f64::from(pos[ind]) + a_normal[0] * f) as f32;
        pos[ind + 1] = (f64::from(pos[ind + 1]) + a_normal[1] * f) as f32;
        pos[ind + 2] = (f64::from(pos[ind + 2]) + a_normal[2] * f) as f32;
    }
}

/// **FLATTEN** (`Flatten.js:41-81`) — projeta no plano da área, e **serve o
/// CLAY**.
///
/// ⚠️ **O Clay não é um verbo próprio no original: ele é ISTO com o plano
/// deslocado.** O `Brush.stroke` calcula `aCenter += aNormal * (raio · 0.1)` e
/// chama `Flatten.prototype.flatten.call(this, ...)` — o mesmo laço, o mesmo
/// `continue`. Ver [`clay_plane_offset`].
///
/// ⚠️ **O `continue` do lado errado do plano é o que torna o verbo
/// AUTO-LIMITADO**, e é a metade que o nosso `add(project(...), n, reach)`
/// nunca teve: um vértice que já passou do plano **não é tocado**. Sem ele o
/// Clay empurraria para sempre, e com ele o barro sobe até o plano e para —
/// que é o que faz um Clay parecer barro em vez de um Draw.
///
/// ⚠️ **E o deslocamento é PROPORCIONAL à distância ao plano** (`fallOff *=
/// distToPlane * …`), não a uma constante do pincel: longe do plano ele anda
/// muito, perto ele anda pouco. É um ALVO, não um incremento — e é por isso que
/// ele converge em vez de oscilar.
#[allow(clippy::too_many_arguments)]
pub fn flatten(
    pos: &mut [f32],
    free: &[f32],
    verts: &[u32],
    proxy: Option<&[f32]>,
    a_normal: [f64; 3],
    a_center: [f64; 3],
    center: [f64; 3],
    radius_squared: f64,
    intensity: f64,
    negative: bool,
) {
    let radius = radius_squared.sqrt();
    let comp = if negative { -1.0 } else { 1.0 };
    for &v in verts {
        let ind = v as usize * 3;
        let vx = f64::from(pos[ind]);
        let vy = f64::from(pos[ind + 1]);
        let vz = f64::from(pos[ind + 2]);
        let dist_to_plane = (vx - a_center[0]) * a_normal[0]
            + (vy - a_center[1]) * a_normal[1]
            + (vz - a_center[2]) * a_normal[2];
        if dist_to_plane * comp > 0.0 {
            continue;
        }
        let o = origin(pos, proxy, ind);
        let Some(dist) = normalised_dist(o, center, radius) else {
            continue;
        };
        let f = falloff(dist) * dist_to_plane * intensity * f64::from(free[v as usize]);
        pos[ind] = (vx - a_normal[0] * f) as f32;
        pos[ind + 1] = (vy - a_normal[1] * f) as f32;
        pos[ind + 2] = (vz - a_normal[2] * f) as f32;
    }
}

/// O deslocamento do plano que faz do [`flatten`] um **Clay** — `Brush.js:52`.
///
/// ⚠️ **Uma constante da FERRAMENTA, não um knob.** No original ela é o literal
/// `0.1` dentro do `stroke`, sem slider; o nosso `plane_offset` é um controle do
/// artista, e os dois são compatíveis (o default dele passa a ser este número).
#[must_use]
pub fn clay_plane_offset(radius: f64) -> f64 {
    radius * 0.1
}

/// **INFLATE** (`Inflate.js:36-76`) — empurra cada vértice pela **PRÓPRIA**
/// normal.
///
/// ⚠️ **Ele DIVIDE pelo comprimento da normal** (`fallOff /= sqrt(nx²+ny²+nz²)`)
/// e depois multiplica pela normal crua — ou seja, ele normaliza *na hora*, em
/// vez de assumir que o array já está unitário. Numa malha cuja normal é
/// unitária isto é um no-op algébrico e **não** é um no-op em bits.
#[allow(clippy::too_many_arguments)]
pub fn inflate(
    pos: &mut [f32],
    normals: &[f32],
    free: &[f32],
    verts: &[u32],
    proxy: Option<&[f32]>,
    center: [f64; 3],
    radius_squared: f64,
    intensity: f64,
    negative: bool,
) {
    let radius = radius_squared.sqrt();
    let mut deform = intensity * radius * 0.1;
    if negative {
        deform = -deform;
    }
    for &v in verts {
        let ind = v as usize * 3;
        let o = origin(pos, proxy, ind);
        let Some(dist) = normalised_dist(o, center, radius) else {
            continue;
        };
        let mut f = deform * falloff(dist);
        let nx = f64::from(normals[ind]);
        let ny = f64::from(normals[ind + 1]);
        let nz = f64::from(normals[ind + 2]);
        f /= (nx * nx + ny * ny + nz * nz).sqrt();
        f *= f64::from(free[v as usize]);
        pos[ind] = (f64::from(pos[ind]) + nx * f) as f32;
        pos[ind + 1] = (f64::from(pos[ind + 1]) + ny * f) as f32;
        pos[ind + 2] = (f64::from(pos[ind + 2]) + nz * f) as f32;
    }
}

/// **CREASE** (`Crease.js:38-76`) — aperta lateralmente **e** cava.
///
/// ⚠️ **O `pow(fallOff, 5)` cai SÓ no termo da normal**, e é isso que faz um
/// vinco ser fino: o puxão lateral é linear no falloff (largo) e o afundamento é
/// quíntico (estreito). Trocar a ordem dá um Draw invertido com nome de vinco.
///
/// ⚠️ **A máscara entra ANTES do expoente** (`:67` roda antes do `:68`), então
/// um vértice meio-mascarado leva `0,5⁵ = 3%` do empurrão normal e `0,5` do
/// puxão lateral. A assimetria é da referência, não um descuido do porte.
///
/// ⚠️ **O expoente é uma CADEIA DE MULTIPLICAÇÕES, e não um `pow`** — e a
/// escolha é medida, não estética. O original chama `Math.pow(fallOff, 5)`;
/// medido sobre 2 M amostras em `[0,1]`, `Math.pow(x,5)` e `x²·x²·x` divergem em
/// **51,5%** das avaliações, por **≤ 4,66e-16 relativo** — onze ordens de
/// grandeza abaixo da resolução do `f32` em que o resultado pousa, e o gate de
/// paridade fica verde com qualquer uma das duas.
///
/// Sendo indistinguíveis, o desempate é o resto do repo: um `powf` é uma
/// chamada de **libm**, cuja última casa não é a mesma nos três OSes da matriz
/// (HR-5), e ele faria a cor deste gate depender da plataforma por uma
/// diferença que ninguém consegue observar. A cadeia é `+ − ×` puro, que o
/// IEEE-754 especifica exatamente.
#[allow(clippy::too_many_arguments)]
pub fn crease(
    pos: &mut [f32],
    free: &[f32],
    verts: &[u32],
    proxy: Option<&[f32]>,
    a_normal: [f64; 3],
    center: [f64; 3],
    radius_squared: f64,
    intensity: f64,
    negative: bool,
) {
    let radius = radius_squared.sqrt();
    let deform = intensity * 0.07;
    let mut brush_factor = deform * radius;
    if negative {
        brush_factor = -brush_factor;
    }
    for &v in verts {
        let ind = v as usize * 3;
        let o = origin(pos, proxy, ind);
        // ⚠️ **O sinal do delta é o OPOSTO do dos outros kernels** (`cx - vProxy`
        // e não `vProxy - cx`, `Crease.js:59-61`): é ele que aponta para o
        // centro, e é o vetor que o puxão lateral usa depois.
        let dx = center[0] - o[0];
        let dy = center[1] - o[1];
        let dz = center[2] - o[2];
        let dist = (dx * dx + dy * dy + dz * dz).sqrt() / radius;
        if dist >= 1.0 {
            continue;
        }
        let f = falloff(dist) * f64::from(free[v as usize]);
        let f2 = f * f;
        let brush_modifier = f2 * f2 * f * brush_factor;
        let f = f * deform;
        pos[ind] = (f64::from(pos[ind]) + dx * f + a_normal[0] * brush_modifier) as f32;
        pos[ind + 1] = (f64::from(pos[ind + 1]) + dy * f + a_normal[1] * brush_modifier) as f32;
        pos[ind + 2] = (f64::from(pos[ind + 2]) + dz * f + a_normal[2] * brush_modifier) as f32;
    }
}

/// **PINCH** (`Pinch.js:34-66`) — os vértices se juntam em volta do ponto de
/// intersecção.
///
/// ⚠️ **A distância sai da posição VIVA, e ele não tem proxy nenhum** — o
/// original lê `vAr` para o delta *e* para a distância, sem consultar
/// `_accumulate`. É a única tool desta lista sem essa pergunta.
///
/// ⚠️ **Ele puxa em 3D, sem projetar na tangente** — o que significa que ele
/// também ACHATA um pouco. É divergência conhecida do nosso Pinch, que remove a
/// componente normal; aqui manda a referência.
#[allow(clippy::too_many_arguments)]
pub fn pinch(
    pos: &mut [f32],
    free: &[f32],
    verts: &[u32],
    center: [f64; 3],
    radius_squared: f64,
    intensity: f64,
    negative: bool,
) {
    let radius = radius_squared.sqrt();
    let mut deform = intensity * 0.05;
    if negative {
        deform = -deform;
    }
    for &v in verts {
        let ind = v as usize * 3;
        let vx = f64::from(pos[ind]);
        let vy = f64::from(pos[ind + 1]);
        let vz = f64::from(pos[ind + 2]);
        let dx = center[0] - vx;
        let dy = center[1] - vy;
        let dz = center[2] - vz;
        let dist = (dx * dx + dy * dy + dz * dz).sqrt() / radius;
        // ⚠️ **SEM `if dist >= 1.0 continue`**, e a ausência é da referência
        // (`Pinch.js:52-58` não tem a guarda). Fora do raio a curva **CRESCE**
        // (ver o doc do [`falloff`]: `0,31` em `d = 1,2`, `17` em `d = 2`),
        // então um vértice fora da pegada seria puxado com FORÇA MAIOR que o do
        // centro — e a `deform` de `0,05` deixaria de ser um freio.
        //
        // ⚠️ **O que o segura é a PEGADA, e é por isso que ela não é um
        // detalhe do chamador aqui.** No original quem entrega a lista é o
        // `pickVerticesInSphere`, que corta no raio; entregar a este kernel uma
        // lista mais larga que a esfera não o deixa mais forte de leve — deixa
        // a ferramenta divergente, e sem nenhum sintoma perto do centro.
        let f = falloff(dist) * deform * f64::from(free[v as usize]);
        pos[ind] = (vx + dx * f) as f32;
        pos[ind + 1] = (vy + dy * f) as f32;
        pos[ind + 2] = (vz + dz * f) as f32;
    }
}

/// **DRAG** (`Drag.js:98-125`) — o *snake hook*: soma um deslocamento comum,
/// pesado pelo falloff, sobre a posição **VIVA**.
///
/// ⚠️ **A pegada ANDA com o cursor** (`updateDragDir` reposiciona o centro a
/// cada passo), e é por isso que um dab entrega ao seguinte e a ponta cresce.
/// Aqui só o laço é portado; quem move o centro é o chamador.
pub fn drag(
    pos: &mut [f32],
    free: &[f32],
    verts: &[u32],
    dir: [f64; 3],
    center: [f64; 3],
    radius_squared: f64,
) {
    let radius = radius_squared.sqrt();
    for &v in verts {
        let ind = v as usize * 3;
        let vx = f64::from(pos[ind]);
        let vy = f64::from(pos[ind + 1]);
        let vz = f64::from(pos[ind + 2]);
        let dx = vx - center[0];
        let dy = vy - center[1];
        let dz = vz - center[2];
        let dist = (dx * dx + dy * dy + dz * dz).sqrt() / radius;
        let f = falloff(dist) * f64::from(free[v as usize]);
        pos[ind] = (vx + dir[0] * f) as f32;
        pos[ind + 1] = (vy + dir[1] * f) as f32;
        pos[ind + 2] = (vz + dir[2] * f) as f32;
    }
}

/// **MOVE** (`Move.js:110-138`) — o *grab*: `proxy + dir · falloff`.
///
/// ⚠️ **A distância sai do PROXY e o resultado é somado à posição VIVA**
/// (`vAr[ind] += dirx * fallOff`), o que faz dele o único desta lista em que as
/// duas leituras vêm de arrays diferentes de propósito. O proxy é congelado no
/// pen-down (`initMoveData`) e o conjunto **nunca é reconsultado** — é o que
/// impede o barro puxado para fora do raio de congelar a meio caminho.
///
/// ⚠️ **O proxy aqui é indexado pela POSIÇÃO NA LISTA** (`j = i * 3`), não pelo
/// id do vértice: ele é um array do tamanho da pegada, não da malha. Trocar os
/// dois lê a vizinhança errada em silêncio, e o gate contra o oráculo é o que
/// pega — os números não se parecem com nada.
pub fn r#move(
    pos: &mut [f32],
    free: &[f32],
    verts: &[u32],
    proxy_packed: &[f32],
    dir: [f64; 3],
    center: [f64; 3],
    radius_squared: f64,
) {
    let radius = radius_squared.sqrt();
    for (i, &v) in verts.iter().enumerate() {
        let ind = v as usize * 3;
        let j = i * 3;
        let vx = f64::from(proxy_packed[j]);
        let vy = f64::from(proxy_packed[j + 1]);
        let vz = f64::from(proxy_packed[j + 2]);
        let dx = vx - center[0];
        let dy = vy - center[1];
        let dz = vz - center[2];
        let dist = (dx * dx + dy * dy + dz * dz).sqrt() / radius;
        let f = falloff(dist) * f64::from(free[v as usize]);
        pos[ind] = (f64::from(pos[ind]) + dir[0] * f) as f32;
        pos[ind + 1] = (f64::from(pos[ind + 1]) + dir[1] * f) as f32;
        pos[ind + 2] = (f64::from(pos[ind + 2]) + dir[2] * f) as f32;
    }
}

/// **LOCAL SCALE** (`LocalScale.js:65-88`) — afasta ou aproxima da âncora.
///
/// ⚠️ **A `intensity` aqui é o DELTA DO GESTO em pixels**, não um `[0,1]`: o
/// `sculptStroke` passa `main._mouseX - main._lastMouseX` direto, e o `0.01` é
/// o que o converte em fração. É a única tool cujo parâmetro de intensidade não
/// é o slider — e é por isso que um `intensity` de `0.5` aqui quase não move
/// nada, enquanto nos outros kernels ele é meio pincel.
pub fn scale(
    pos: &mut [f32],
    free: &[f32],
    verts: &[u32],
    center: [f64; 3],
    radius_squared: f64,
    intensity: f64,
) {
    let delta_scale = intensity * 0.01;
    let radius = radius_squared.sqrt();
    for &v in verts {
        let ind = v as usize * 3;
        let vx = f64::from(pos[ind]);
        let vy = f64::from(pos[ind + 1]);
        let vz = f64::from(pos[ind + 2]);
        let dx = vx - center[0];
        let dy = vy - center[1];
        let dz = vz - center[2];
        let dist = (dx * dx + dy * dy + dz * dz).sqrt() / radius;
        let f = falloff(dist) * delta_scale * f64::from(free[v as usize]);
        pos[ind] = (vx + dx * f) as f32;
        pos[ind + 1] = (vy + dy * f) as f32;
        pos[ind + 2] = (vz + dz * f) as f32;
    }
}

/// **A NORMAL DE ÁREA** (`SculptBase.js:224-243`) — a direção que o Draw empurra
/// e a normal do plano que o Flatten/Clay ajusta.
///
/// ⚠️ **A ponderação é a MÁSCARA, não o falloff**, e essa é uma divergência
/// nossa que vale um desenho inteiro: o nosso `fit_plane_over` pesa por
/// `falloff.weight(...)`, o que faz o centro da pegada mandar no plano; aqui
/// todo vértice **livre** pesa igual, e um vértice travado não vota. São planos
/// diferentes sobre a mesma pegada.
///
/// Devolve `None` quando a soma degenera (`len == 0`) — o original faz
/// `return;`, e o `Brush.stroke` **sai do dab inteiro** nesse caso.
#[must_use]
pub fn area_normal(normals: &[f32], free: &[f32], verts: &[u32]) -> Option<[f64; 3]> {
    let (mut anx, mut any, mut anz) = (0.0f64, 0.0f64, 0.0f64);
    for &v in verts {
        let ind = v as usize * 3;
        let f = f64::from(free[v as usize]);
        anx += f64::from(normals[ind]) * f;
        any += f64::from(normals[ind + 1]) * f;
        anz += f64::from(normals[ind + 2]) * f;
    }
    let len = (anx * anx + any * any + anz * anz).sqrt();
    if len == 0.0 {
        return None;
    }
    let len = 1.0 / len;
    Some([anx * len, any * len, anz * len])
}

/// **O CENTRO DE ÁREA** (`SculptBase.js:246-261`) — o ponto do plano.
///
/// ⚠️ **O divisor é a soma das MÁSCARAS, não a contagem** (`acc += f`), então
/// uma pegada meio travada tem o centro puxado para o lado livre. E com a
/// pegada inteiramente travada ele divide por zero e devolve `NaN` — o original
/// não se defende disso, e nós **não** copiamos o buraco: devolvemos `None`,
/// porque um `NaN` aqui envenena a malha inteira pelo plano.
#[must_use]
pub fn area_center(pos: &[f32], free: &[f32], verts: &[u32]) -> Option<[f64; 3]> {
    let (mut ax, mut ay, mut az) = (0.0f64, 0.0f64, 0.0f64);
    let mut acc = 0.0f64;
    for &v in verts {
        let ind = v as usize * 3;
        let f = f64::from(free[v as usize]);
        acc += f;
        ax += f64::from(pos[ind]) * f;
        ay += f64::from(pos[ind + 1]) * f;
        az += f64::from(pos[ind + 2]) * f;
    }
    if acc == 0.0 {
        return None;
    }
    Some([ax / acc, ay / acc, az / acc])
}

/// **OS VÉRTICES DA FRENTE** (`SculptBase.js:206-221`) — os que olham para o
/// olho, e os únicos que decidem a direção do Draw e o plano do Flatten.
///
/// ⚠️ **`<= 0.0` e não `< 0.0`**: um vértice cuja normal é exatamente
/// perpendicular ao olho CONTA. Na silhueta isso é metade dos vértices.
pub fn front_vertices(normals: &[f32], verts: &[u32], eye: [f64; 3], out: &mut Vec<u32>) {
    out.clear();
    for &v in verts {
        let ind = v as usize * 3;
        let d = f64::from(normals[ind]) * eye[0]
            + f64::from(normals[ind + 1]) * eye[1]
            + f64::from(normals[ind + 2]) * eye[2];
        if d <= 0.0 {
            out.push(v);
        }
    }
}

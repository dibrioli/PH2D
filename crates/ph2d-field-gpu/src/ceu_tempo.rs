//! ⭐⭐⭐⭐ **A OCLUSÃO NO TEMPO** — o céu de um ponto calcula-se UMA vez e sobrevive ao movimento
//! (`docs/Render3d/14` §6, o passo `3` da `F1`: *«o quadro anterior é informação; hoje é deitado
//! fora»*; ordem do dono de 2026-09-29: *«o render ainda não está em tempo real — somos uma game
//! engine»*).
//!
//! # A propriedade que torna isto EXACTO e não uma aproximação
//!
//! Os cones da oclusão são direcções FIXAS de MUNDO ([`crate::trace_wgsl`], `direccao_do_cone`), e
//! o céu de um ponto depende do ponto, da normal e do campo — **nunca da câmara**. Enquanto o artista
//! gira a vista a peça não muda, logo o que o quadro anterior somou para um ponto continua CERTO.
//!
//! ⇒ as SOMAS dos cones (`Σ c·vis` e `Σ c`) guardam-se NO MUNDO, numa tabela de hash de células do
//! tamanho de um pixel — a *cache em grelha de hash* dos traçadores de caminhos (a SHaRC e família),
//! aplicada à grandeza que aqui é independente da vista. Cada pixel acha a célula do SEU ponto (e da
//! sua normal, num de `16` baldes); se ela já tem as `CEU_FATIAS` fatias, lê; senão reclama as que
//! faltam com um `atomicAdd` cada e acrescenta-as a uma lista de trabalho — pixels vizinhos na mesma
//! célula repartem as fatias entre si, e a célula fica COMPLETA no mesmo quadro em que apareceu.
//!
//! ⛔⛔ **A 1.ª redacção guardava o histórico no ECRÃ e reprojectava-o**, e foi MEDIDA e recusada: com
//! a rotação a zero ela era exacta (`máx 1` nível), e bastava UM quadro de meio grau para `9 212`
//! canais passarem de `8` níveis — cada salto copia o valor do pixel vizinho, nas sombras de contacto
//! a oclusão muda depressa, e o erro ACUMULA a cada quadro (`16` saltos de `0,5°` erravam mais do que
//! `16` de `3°`). *Um valor recopiado de ecrã para ecrã deriva; um valor guardado no mundo não.* Na
//! tabela o erro está limitado ao tamanho de uma célula, e não cresce.
//!
//! O quadro ASSENTE continua a oclusão inteira e exacta (é ele que a paridade com a CPU mede) e
//! GRAVA as células com a MÉDIA dos seus pixels — logo a primeira rotação já as encontra cheias.
//!
//! # ⭐⭐⭐⭐ Tempo real também no ZOOM (2026-09-30)
//!
//! Três coisas tinham de ser verdade, e nenhuma era: **(1)** o alcance da oclusão vinha da câmara e
//! passou a MUNDO (`ph2d_field_render::occlusion_reach`); **(2)** a [`ChaveDoCeu`] não leva nada da
//! câmara; **(3)** a tabela tem [`ENTRADAS_POR_PIXEL`] entradas por pixel da vista, com o tecto do
//! dispositivo — a `2²¹` fixos ela ENCHIA e o que sobrava era sal. Relógio (`diag_o_ceu_no_tempo`,
//! release, `1920×1080`, carga `8`–`12`, mínimo das medianas de `3` corridas, `0` reinícios):
//!
//! | cena | girar `3°` antes → agora | aproximar `3 %` | afastar `3 %` | sem céu (girar · aproximar) |
//! |---|---|---|---|---|
//! | nó (`28`) | `28,5 → 18,3` | `66,3 → 29,5` | `27,2 → 16,0` | `12,4` · `21,0` |
//! | outras três | `8,0`–`9,2 → 5,8`–`7,2` | `12,3`–`15,7 → 8,1`–`10,6` | `7,8`–`9,8 → 6,6`–`7,3` | `4,4` · `5,9` (a `30`) |
//!
//! ⚠️ A coluna SEM CÉU é o chão: a aproximar, o nó passa de `16,7 ms` mesmo com a oclusão de graça
//! — o que lhe falta ao tempo real ali é a marcha da peça, não o céu.
//!
//! # ⚠️ Os armazéns
//!
//! A tabela vive num GRUPO próprio (`1`), ligado só por estes dois kernels: o grupo `0` tem seis
//! armazéns e aqui sobem para sete — abaixo do piso de oito da `wgpu` —, e o passe que pinta (que
//! já precisa de nove) não é tocado.

/// ⭐⭐⭐⭐ **O que o quadro faz com o histórico da oclusão.**
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CeuTempo {
    /// O quadro de sempre, **ao bit** — nenhum histórico lido nem escrito.
    #[default]
    Nao,
    /// O quadro ASSENTE: a oclusão inteira, e GRAVA-a no histórico com as fatias todas.
    Grava,
    /// O quadro de MOVIMENTO: herda do quadro anterior e marcha uma fatia só onde falta.
    Acumula,
}

impl CeuTempo {
    /// O código que o `Setup` do WGSL lê — ver `ceu_tempo` na `struct Setup`.
    #[must_use]
    pub(crate) fn codigo(self) -> u32 {
        match self {
            Self::Nao => 0,
            Self::Grava => 1,
            Self::Acumula => 2,
        }
    }
}

/// ⭐ **Em quantas fatias os cones se partem.** Com os `48` de fábrica são `6` cones por fatia — a
/// unidade de trabalho da lista: os pixels de uma célula nova repartem-nas, e cada item da lista
/// marcha a mesma quantidade de cones (sem divergência de célula dentro de um grupo).
pub const CEU_FATIAS: u32 = 8;

/// O que tem de ser IGUAL para o histórico ser válido: o campo e tudo o que os cones leem.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ChaveDoCeu {
    fonte: String,
    consts: Vec<u32>,
    marcha: Vec<u32>,
    longe: Option<crate::longe::Longe>,
    /// Quantas entradas a tabela tem — ver [`entradas_para`]. Mudar de janela muda-as, e a tabela
    /// recomeça pela MESMA porta que uma edição da peça.
    entradas: u32,
}

impl ChaveDoCeu {
    /// ⚠️ **Uma peça com ESCULTURA não é guardada** — a cerca das sondas e do chão: a grade dela
    /// sobe por outro caminho e não entra nesta chave.
    pub(crate) fn de(
        fita: &ph2d_field_eval::wgsl::TapeWgsl,
        sculpts: &[ph2d_field_eval::device::DeviceSculpt],
        setup: &crate::trace::MarchSetup,
        pixels: u64,
        limite: u64,
    ) -> Option<Self> {
        if !sculpts.is_empty() {
            return None;
        }
        let bits = |v: &[f32]| v.iter().map(|f| f.to_bits()).collect::<Vec<u32>>();
        // ⛔⛔ **NADA DA CÂMARA ENTRA AQUI** (2026-09-30). Os dois limiares de PIXEL (`hit_eps`,
        // `normal_eps`) escalam com o zoom e estavam na chave: cada quadro de aproximar recomeçava a
        // tabela e pagava a oclusão inteira — o zoom NÃO estava em tempo real, e o gate que só
        // girava à distância fixa não o podia ver. Eles só deslocam a origem de um cone e a
        // diferença finita da normal na ordem de UM pixel, que é o tamanho da própria célula — o
        // erro que a tabela já aceita. O alcance, que também vinha da câmara, passou a MUNDO
        // (`ph2d_field_render::occlusion_reach`).
        let mut marcha = bits(&[
            setup.step,
            setup.ao_reach,
            setup.ball_center[0],
            setup.ball_center[1],
            setup.ball_center[2],
            setup.ball_radius,
        ]);
        marcha.push(setup.budget);
        marcha.push(setup.ao_rays);
        Some(Self {
            fonte: fita.source.clone(),
            consts: bits(&fita.consts),
            marcha,
            longe: setup.longe,
            entradas: entradas_para(pixels, limite),
        })
    }
}

/// ⭐⭐⭐⭐ **Quantas entradas por PIXEL da vista a tabela tem** — o recurso é a MEMÓRIA da placa
/// (`20 B` por entrada ⇒ `80 B` por pixel: `166 MB` a `1920×1080`), e o tecto é o do dispositivo
/// ([`entradas_para`]).
///
/// ⛔⛔ **Até 2026-09-30 era uma constante, `2²¹` (`40 MB`), e ela era o SAL da imagem.** Uma célula
/// tem `1`–`2` píxeis e a sondagem é linear em `8` vagas, logo a `1920×1080` a tabela ENCHIA: um
/// pixel sem entrada marcha UMA fatia (`6` cones) só para si e pinta-a — pontos soltos no fundo das
/// fendas. Um zoom põe um nível inteiro de células novas ao lado do de antes, e ali era pior.
/// Medido (`diag_os_gestos_da_camara`, o nó e a rosca, `12` quadros de cada gesto, canais acima de
/// `8` níveis contra a oclusão exacta):
///
/// | entradas | por pixel | girar nó · rosca | aproximar nó · rosca | afastar nó · rosca |
/// |---:|---:|---|---|---|
/// | `2²¹` | `1` | `64`–`131` · `480`–`548` | `1 604`–`1 744` · `1 251`–`1 361` | `55`–`83` · `144`–`187` |
/// | `2²²` | `2` | `11`–`32` · `181`–`239` | `326`–`411` · `464`–`516` | `1`–`34` · `57`–`75` |
/// | **`2²³`** | **`4`** | `11`–`32` · `208`–`214` | `240`–`267` · `417`–`445` | `1` · `51`–`69` |
/// | `2²⁴` | `8` | `11`–`32` · `183`–`192` | `213`–`240` · `346`–`373` | `1` · `51` |
///
/// ⇒ o joelho é `4` por pixel: o dobro disso compra `10`–`15 %` a aproximar e nada a girar.
/// ⛔ Roubar as entradas velhas mais cedo (`VELHA = 16`) foi medido e não ajuda.
pub const ENTRADAS_POR_PIXEL: u64 = 4;

/// ⭐ **As entradas de uma vista de `pixels`**, com o tecto do DISPOSITIVO: o maior buffer que ele
/// deixa ligar a um shader (`limite`, em bytes). Numa placa no piso da `wgpu` (`128 MiB`) uma vista
/// `1920×1080` fica com `6,7 M` em vez de `8,3 M` — o que a tabela acima mede entre `2²²` e `2²³`.
#[must_use]
pub fn entradas_para(pixels: u64, limite: u64) -> u32 {
    let quer = pixels.saturating_mul(ENTRADAS_POR_PIXEL);
    let cabe = limite / (PALAVRAS * 4);
    u32::try_from(quer.min(cabe).max(1 << 16)).unwrap_or(u32::MAX)
}

/// ⭐ **Quanto as vizinhas de uma célula vazia podem discordar para ela as herdar** — em céu (`0..1`).
///
/// Medido (nó de toro a `3°` por quadro, `1920×1080`): herdar sem esta cerca levava o quadro de
/// `21,1` a `16,2 ms` e o pior erro de `13` a `56` níveis, num ponto claro no fundo de uma fenda; a
/// cerca de `0,03`/`0,06`/`0,12` quase não mexe no relógio (`17,4`/`17,0`/`16,4`) nem no pior erro —
/// quem o curou foi exigir DUAS vizinhas. Fica o meio da faixa.
const CONCORDANCIA: f32 = 0.06;

/// O tecto de grupos de um despacho numa dimensão (`maxComputeWorkgroupsPerDimension` do piso da `wgpu`).
const MAX_GRUPOS_1D: u32 = 65_535;

/// Quantos quadros sem ser lida uma entrada aguenta antes de outra célula a poder roubar.
pub const VELHA: u32 = 120;

/// As palavras de uma entrada: a impressão da chave, `Σ c·vis`, `Σ c` (as duas em ponto fixo), as
/// fatias reclamadas e o quadro em que foi lida pela última vez.
const PALAVRAS: u64 = 5;

/// ⭐ **A tabela guardada na placa entre quadros** — e a chave do campo que a encheu.
pub(crate) struct Tabela {
    chave: ChaveDoCeu,
    buffer: wgpu::Buffer,
    quadro: u32,
    /// A lista de trabalho do movimento e quantas vagas ela tem.
    lista: Option<(wgpu::Buffer, u32)>,
    /// Os argumentos do despacho indirecto — um buffer À PARTE, porque a `wgpu` não deixa o mesmo
    /// ser armazém e argumento no mesmo despacho: o `args` escreve-os na lista e uma CÓPIA traz-nos.
    args: Option<wgpu::Buffer>,
}

/// ⭐ **Quantas vezes a tabela recomeçou do zero** — a régua de que uma edição da peça a esquece.
impl crate::FieldPipelines {
    #[must_use]
    pub fn ceu_tempo_reinicios(&self) -> usize {
        self.ceu_tempo_reinicios
    }

    /// ⭐ **Esquece a tabela** — o quadro seguinte recomeça do zero, como depois de uma edição. É a
    /// porta das sondas que repetem uma rotação: sem ela a 2.ª volta encontra as células cheias.
    pub fn esquece_o_ceu(&mut self) {
        self.ceu_tempo = None;
    }
}

/// ⭐ **A tabela guardada serve a esta chave?** — sem ela o quadro não tem o que herdar.
pub(crate) fn herda(cache: &crate::FieldPipelines, chave: &ChaveDoCeu) -> bool {
    cache.ceu_tempo.as_ref().is_some_and(|t| &t.chave == chave)
}

/// O WGSL dos dois kernels — acrescenta-se ao molde da marcha, só para estas duas entradas.
pub(crate) const WGSL: &str = r"
struct Tabela {
    entradas: u32, fatias: u32, quadro: u32, velha: u32,
    unidade: f32, concordancia: f32, vagas: u32, _a: u32,
};
@group(1) @binding(0) var<uniform> tab: Tabela;
@group(1) @binding(1) var<storage, read_write> t: array<atomic<u32>>;
// A LISTA de trabalho do movimento: `[0]` o contador, `[1..4]` os argumentos do despacho indirecto,
// e depois pares `(pixel, célula << 8 | fatia)`.
@group(1) @binding(2) var<storage, read_write> w: array<atomic<u32>>;

// O ponto fixo das somas: `Σ c ≤ 48`, logo `48 · 65 536` cabe à vontade num `u32`.
const FIXO: f32 = 65536.0;

fn mistura(x: u32) -> u32 {
    var h = x * 747796405u + 2891336453u;
    h = ((h >> ((h >> 28u) + 4u)) ^ h) * 277803737u;
    return (h >> 22u) ^ h;
}

// ⭐ A CÉLULA do ponto: o nível sai do tamanho do pixel NESTE ponto (uma célula de `1`–`2` píxeis,
// logo o zoom escolhe outro nível e nunca lê células de outro tamanho), e a normal cai num de `16`
// baldes — as duas faces de uma parede fina nunca partilham uma célula.
fn nivel_da_celula(p: vec3<f32>, n: vec3<f32>) -> i32 {
    return i32(ceil(log2(max(pixel_no_mundo(p) / tab.unidade, 1.0))));
}

fn chave_da_celula(p: vec3<f32>, n: vec3<f32>) -> vec2<u32> {
    return chave_no_nivel(p, n, nivel_da_celula(p, n));
}

fn chave_no_nivel(p: vec3<f32>, n: vec3<f32>, nivel: i32) -> vec2<u32> {
    let lado = tab.unidade * exp2(f32(nivel));
    let g = vec3<i32>(floor(p / lado));
    var e = n.xy / (abs(n.x) + abs(n.y) + abs(n.z));
    if (n.z < 0.0) {
        e = (vec2<f32>(1.0) - abs(e.yx)) * select(vec2<f32>(-1.0), vec2<f32>(1.0), e >= vec2<f32>(0.0));
    }
    let balde = vec2<u32>(clamp((e + vec2<f32>(1.0)) * 2.0, vec2<f32>(0.0), vec2<f32>(3.0)));
    var h = mistura(bitcast<u32>(g.x));
    h = mistura(h ^ bitcast<u32>(g.y));
    h = mistura(h ^ bitcast<u32>(g.z));
    h = mistura(h ^ (bitcast<u32>(nivel) * 16u + balde.x * 4u + balde.y));
    return vec2<u32>(h, mistura(h ^ 0x9e3779b9u) | 1u);
}

// ⭐ A ENTRADA da célula: sondagem linear de `8`, reclamando uma vazia ou ROUBANDO uma velha.
// `0xffffffff` quando não há lugar — o pixel marcha então uma fatia só para si.
fn entrada_de(k: vec2<u32>) -> u32 {
    for (var i: u32 = 0u; i < 8u; i = i + 1u) {
        let e = (k.x + i) % tab.entradas;
        let b = e * 5u;
        let r = atomicCompareExchangeWeak(&t[b], 0u, k.y);
        if (r.exchanged || r.old_value == k.y) { return e; }
        let lida = atomicLoad(&t[b + 4u]);
        if (tab.quadro - lida > tab.velha) {
            let s = atomicCompareExchangeWeak(&t[b], r.old_value, k.y);
            if (s.exchanged) {
                atomicStore(&t[b + 1u], 0u);
                atomicStore(&t[b + 2u], 0u);
                atomicStore(&t[b + 3u], 0u);
                return e;
            }
        }
    }
    return 0xffffffffu;
}

// Só PROCURA a célula (sem a reclamar): a entrada, ou `0xffffffff` se ela não existe.
fn procura(k: vec2<u32>) -> u32 {
    for (var i: u32 = 0u; i < 8u; i = i + 1u) {
        let e = (k.x + i) % tab.entradas;
        let v = atomicLoad(&t[e * 5u]);
        if (v == k.y) { return e; }
        if (v == 0u) { return 0xffffffffu; }
    }
    return 0xffffffffu;
}

// ⭐ A MESMA soma do `ceu_por_cones`, só com os cones `j ≡ fatia (mod fatias)`.
fn ceu_fatia(erguido: vec3<f32>, n: vec3<f32>, fatia: u32, fatias: u32) -> vec2<f32> {
    var soma = 0.0;
    var peso = 0.0;
    for (var j: u32 = fatia; j < s.ao_rays; j = j + fatias) {
        let dd = direccao_do_cone(j, s.ao_rays);
        let c = dot(n, dd);
        if (c <= 0.0) { continue; }
        peso = peso + c;
        let ate = min(s.ao_reach, cerca_da_bola(erguido, dd, s.ao_reach));
        soma = soma + c * visivel_ceu(erguido, dd, ate, 1.0 / c);
    }
    return vec2<f32>(soma, peso);
}

// O peso de TODOS os cones — é o denominador do `ceu_por_cones`, sem marchar nada.
fn peso_total(n: vec3<f32>) -> f32 {
    var peso = 0.0;
    for (var j: u32 = 0u; j < s.ao_rays; j = j + 1u) {
        peso = peso + max(dot(n, direccao_do_cone(j, s.ao_rays)), 0.0);
    }
    return peso;
}

// O tamanho de um pixel da câmara ACTUAL no ponto `p`, em unidades de mundo.
fn pixel_no_mundo(p: vec3<f32>) -> f32 {
    var escala = 1.0;
    if (s.eye_distance != 0.0) {
        let eye = s.alvo + s.fwd * s.eye_distance;
        escala = max(dot(p - eye, -s.fwd), 0.0) / s.eye_distance;
    }
    return s.half_extent / s.half_px * escala;
}

// O ponto e a normal do pixel `g` do quadro ASSENTE (a câmara que o assentou) — os dois passes leem-no.
fn assente_do_pixel(g: vec3<u32>) -> array<vec3<f32>, 2> {
    let c = centro[g.y * s.w + g.x];
    let r = ray_at_plane(raio(f32(g.x) + 0.5, f32(g.y) + 0.5));
    return array<vec3<f32>, 2>(r.o + r.d * c.x, s.right * c.y + s.up * c.z + s.fwd * c.w);
}

// ⭐⭐⭐ O QUADRO ASSENTE, passe 1 — ZERAR: cada célula que um pixel toca é reclamada, esvaziada e
// dada por CHEIA. ⚠️ É um passe à parte porque o seguinte SOMA: zerar e somar no mesmo passe
// deixaria a ordem dos pixels decidir o que se apaga.
@compute @workgroup_size(8, 8, 1)
fn ceu_tempo_zera(@builtin(global_invocation_id) g: vec3<u32>) {
    if (g.x >= s.w || g.y >= s.h) { return; }
    if (centro[g.y * s.w + g.x].x < 0.0) { return; }
    let pn = assente_do_pixel(g);
    let e = entrada_de(chave_da_celula(pn[0], pn[1]));
    if (e == 0xffffffffu) { return; }
    let b = e * 5u;
    atomicStore(&t[b + 1u], 0u);
    atomicStore(&t[b + 2u], 0u);
    atomicStore(&t[b + 3u], tab.fatias);
    atomicMax(&t[b + 4u], tab.quadro);
}

// ⭐⭐⭐ O QUADRO ASSENTE: a oclusão já foi marchada inteira pela luz; aqui só se guarda na célula.
// ⛔ Escrever também nos quatro CANTOS da pegada do pixel (levados ao plano tangente), para cobrir as
// células que nenhum centro de pixel atravessa, foi medido e RECUSADO: o nó a `3°` não ficou mais
// barato e as curvas passaram de `358` para `1 023` canais acima de `8` níveis — o plano tangente
// erra numa superfície curva. Quem cobre essas células é a herança das vizinhas (`ceu_tempo_pede`).
//
// ⭐⭐⭐ Passe 2 — SOMAR: a célula guarda a MÉDIA dos píxeis que caem nela, pesada como os cones a
// pesam. ⛔ Guardar o céu do ÚLTIMO pixel a escrever (a 1.ª redacção) punha cada pixel parado a ler o
// céu de um vizinho sorteado pela ordem dos grupos — medido no nó: `203`–`253` canais acima de `8`
// níveis contra a exacta, pior que o quadro a passo `2` que a cache substitui (`31`).
@compute @workgroup_size(8, 8, 1)
fn ceu_tempo_grava(@builtin(global_invocation_id) g: vec3<u32>) {
    if (g.x >= s.w || g.y >= s.h) { return; }
    let i = g.y * s.w + g.x;
    if (centro[i].x < 0.0) { return; }
    let pn = assente_do_pixel(g);
    let e = procura(chave_da_celula(pn[0], pn[1]));
    if (e == 0xffffffffu) { return; }
    let peso = peso_total(pn[1]);
    atomicAdd(&t[e * 5u + 1u], u32(luz[i * passo_da_luz()] * peso * FIXO));
    atomicAdd(&t[e * 5u + 2u], u32(peso * FIXO));
}

// O ponto e a normal do pixel `i`, reconstruídos do `centro` — os três passes do movimento leem-no.
fn ponto_do_pixel(i: u32) -> array<vec3<f32>, 2> {
    let c = centro[i];
    let r = ray_at_plane(raio(f32(i % s.w) + 0.5, f32(i / s.w) + 0.5));
    return array<vec3<f32>, 2>(r.o + r.d * c.x, s.right * c.y + s.up * c.z + s.fwd * c.w);
}

// ⭐⭐⭐⭐ O QUADRO DE MOVIMENTO, passe 1 — PEDIR: cada pixel acha a célula do ponto e, se lhe faltam
// fatias, reclama-as e ACRESCENTA o trabalho a uma lista. ⚠️ Marchar aqui dentro seria pagar a
// divergência: uma célula a encher no meio de um grupo de `64` píxeis prende o grupo inteiro, e as
// células novas vêm espalhadas pela imagem — medido, o quadro ficava mais caro do que o de antes.
@compute @workgroup_size(8, 8, 1)
fn ceu_tempo_pede(@builtin(global_invocation_id) g: vec3<u32>) {
    if (g.x >= s.w || g.y >= s.h) { return; }
    let i = g.y * s.w + g.x;
    if (centro[i].x < 0.0) { return; }
    let pn = ponto_do_pixel(i);
    let e = entrada_de(chave_da_celula(pn[0], pn[1]));
    var cel = e;
    if (e != 0xffffffffu) {
        atomicMax(&t[e * 5u + 4u], tab.quadro);
        let tem = atomicLoad(&t[e * 5u + 3u]);
        if (tem >= tab.fatias) { return; }
        if (tem == 0u && herda_das_vizinhas(e, pn[0], pn[1])) { return; }
    }
    for (var q: u32 = 0u; q < tab.fatias; q = q + 1u) {
        // A vaga na lista vem ANTES da fatia: uma fatia reclamada sem vaga ficaria contada e nunca somada.
        let vaga = atomicAdd(&w[0], 1u);
        if (vaga >= tab.vagas) { return; }
        var k = 0u;
        if (e != 0xffffffffu) {
            k = atomicAdd(&t[e * 5u + 3u], 1u);
            if (k >= tab.fatias) { k = 0xffu; }
        }
        atomicStore(&w[4u + vaga * 2u], i);
        atomicStore(&w[5u + vaga * 2u], (cel << 8u) | k);
        if (k == 0xffu || e == 0xffffffffu) { return; }
    }
}

// ⭐⭐⭐⭐ Uma célula VAZIA herda a média das seis vizinhas cheias do mesmo nível e da mesma normal.
// ⚠️ A célula vazia a meio de uma superfície já vista não é superfície NOVA: é uma célula que nenhum
// centro de pixel do quadro assente atravessou (uma célula tem `1`–`2` píxeis e a superfície corta-a
// de raspão). Medido: um único quadro de `3°` encontrava `~10 %` dos píxeis da peça nestas células,
// espalhados como sal pela peça inteira, e cada um marchava os `48` cones. O erro de herdar é a
// distância de UMA célula.
fn herda_das_vizinhas(e: u32, p: vec3<f32>, n: vec3<f32>) -> bool {
    let nivel = nivel_da_celula(p, n);
    let lado = tab.unidade * exp2(f32(nivel));
    var soma = 0.0;
    var quantas = 0.0;
    var menor = 1.0;
    var maior = 0.0;
    for (var a: u32 = 0u; a < 6u; a = a + 1u) {
        var d = vec3<f32>(0.0);
        d[a / 2u] = select(-lado, lado, (a & 1u) != 0u);
        let v = procura(chave_no_nivel(p + d, n, nivel));
        if (v == 0xffffffffu || atomicLoad(&t[v * 5u + 3u]) < tab.fatias) { continue; }
        let pv = f32(atomicLoad(&t[v * 5u + 2u]));
        if (pv <= 0.0) { continue; }
        let ceu = f32(atomicLoad(&t[v * 5u + 1u])) / pv;
        soma = soma + ceu;
        quantas = quantas + 1.0;
        menor = min(menor, ceu);
        maior = max(maior, ceu);
    }
    // Só onde as vizinhas CONCORDAM: numa sombra de contacto o céu muda depressa de célula para
    // célula, e ali herdar é errar — a célula marcha os cones como qualquer outra.
    // ⚠️ E pelo menos DUAS: com uma vizinha só não há concordância a medir, e era dali que vinham os
    // pontos claros isolados no fundo das fendas.
    if (quantas < 2.0 || maior - menor > tab.concordancia) { return false; }
    let r = atomicCompareExchangeWeak(&t[e * 5u + 3u], 0u, tab.fatias);
    if (r.exchanged) {
        let peso = peso_total(n);
        atomicStore(&t[e * 5u + 1u], u32(soma / quantas * peso * FIXO));
        atomicStore(&t[e * 5u + 2u], u32(peso * FIXO));
    }
    return true;
}

// Os argumentos do despacho indirecto do passe 2: quantos grupos de `64` a lista pede.
@compute @workgroup_size(1, 1, 1)
fn ceu_tempo_args() {
    let n = min(atomicLoad(&w[0]), tab.vagas);
    atomicStore(&w[1], (n + 63u) / 64u);
    atomicStore(&w[2], 1u);
    atomicStore(&w[3], 1u);
}

// ⭐⭐⭐⭐ Passe 2 — MARCHAR: uma fatia por item da lista, lado a lado, sem divergência de célula.
@compute @workgroup_size(64, 1, 1)
fn ceu_tempo_marcha(@builtin(global_invocation_id) g: vec3<u32>) {
    if (g.x >= min(atomicLoad(&w[0]), tab.vagas)) { return; }
    let i = atomicLoad(&w[4u + g.x * 2u]);
    let ck = atomicLoad(&w[5u + g.x * 2u]);
    let k = ck & 0xffu;
    if (k == 0xffu) { return; }
    let cel = ck >> 8u;
    let pn = ponto_do_pixel(i);
    // ⭐ A fatia reclamada em `k`-ésimo lugar é a de índice INVERTIDO em bits: as primeiras que uma
    // célula recebe ficam espalhadas pela esfera, e uma célula ainda a encher lê um conjunto de cones
    // uniforme em vez de metade do céu.
    let ordem = array<u32, 8>(0u, 4u, 2u, 6u, 1u, 5u, 3u, 7u);
    var fatia = k;
    if (tab.fatias == 8u) { fatia = ordem[k]; }
    let sp = ceu_fatia(pn[0] + pn[1] * (s.hit_eps * 4.0), pn[1], fatia, tab.fatias);
    if (cel == 0xffffffu) {
        // A tabela estava cheia à volta da chave: uma fatia só para este pixel, e nada se guarda.
        var ceu = 1.0;
        if (sp.y > 0.0) { ceu = sp.x / sp.y; }
        luz[i * passo_da_luz()] = ceu;
        return;
    }
    atomicAdd(&t[cel * 5u + 1u], u32(sp.x * FIXO));
    atomicAdd(&t[cel * 5u + 2u], u32(sp.y * FIXO));
}

// ⭐⭐⭐⭐ Passe 3 — LER: cada pixel lê a célula, já com as fatias deste quadro somadas.
@compute @workgroup_size(8, 8, 1)
fn ceu_tempo_le(@builtin(global_invocation_id) g: vec3<u32>) {
    if (g.x >= s.w || g.y >= s.h) { return; }
    let i = g.y * s.w + g.x;
    // Quem não acerta a peça já tem o céu do CHÃO (ou `1`), escrito pela luz.
    if (centro[i].x < 0.0) { return; }
    let pn = ponto_do_pixel(i);
    let p = pn[0];
    let n = pn[1];
    let nivel = nivel_da_celula(p, n);
    let e = procura(chave_no_nivel(p, n, nivel));
    // Sem entrada, quem escreveu este pixel foi o passe 2.
    if (e == 0xffffffffu) { return; }
    let b = e * 5u;
    // ⛔⛔ **Aqui vivia um recurso ao nível VIZINHO para a célula «a encher», e saiu por MEDIÇÃO**
    // (2026-09-30): o `pede` reclama todas as fatias em falta no MESMO quadro, antes deste passe, logo
    // a contagem já está cheia quando se lê — o ramo só corria quando a lista transbordava. A prova
    // de mutação deixou-o viver (`M5`), e sem ele os gestos de câmara medem o mesmo dentro da
    // variância das corridas. *Uma linha que a mutação não consegue matar não é lei.*
    let soma = f32(atomicLoad(&t[b + 1u]));
    let peso = f32(atomicLoad(&t[b + 2u]));
    var ceu = 1.0;
    if (peso > 0.0) { ceu = soma / peso; }
    luz[i * passo_da_luz()] = ceu;
}
";

/// ⭐⭐⭐⭐ **Despacha o kernel da tabela neste quadro**, no encoder da marcha e depois da luz.
#[allow(clippy::too_many_arguments)]
pub(crate) fn despacha(
    device: &wgpu::Device,
    enc: &mut wgpu::CommandEncoder,
    cache: &mut crate::FieldPipelines,
    molde: &str,
    fita: &ph2d_field_eval::wgsl::TapeWgsl,
    bgl0: &wgpu::BindGroupLayout,
    bg0: &wgpu::BindGroup,
    chave: ChaveDoCeu,
    setup: &crate::trace::MarchSetup,
    width: u32,
    height: u32,
) {
    use wgpu::util::DeviceExt;
    // A tabela só vale com a MESMA chave — uma edição da peça esquece-a.
    let mut tabela = match cache.ceu_tempo.take() {
        Some(t) if t.chave == chave => t,
        antiga => {
            cache.ceu_tempo_reinicios += 1;
            match antiga {
                // ⚠️ O buffer serve se tiver o MESMO tamanho: limpá-lo no encoder custa menos do que
                // alocar `166 MB`.
                Some(t) if t.chave.entradas == chave.entradas => {
                    enc.clear_buffer(&t.buffer, 0, None);
                    Tabela {
                        chave,
                        buffer: t.buffer,
                        quadro: 0,
                        lista: t.lista,
                        args: t.args,
                    }
                }
                // Sem tabela, ou com outro tamanho: a velha (se houver) é largada aqui.
                _ => Tabela {
                    buffer: device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("ceu-tempo"),
                        size: u64::from(chave.entradas) * PALAVRAS * 4,
                        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    }),
                    chave,
                    quadro: 0,
                    lista: None,
                    args: None,
                },
            }
        }
    };
    // ⚠️ O quadro começa em `VELHA + 1`: com `0` toda entrada acabada de nascer lia-se velha.
    tabela.quadro = tabela.quadro.max(VELHA + 1).wrapping_add(1);
    // ⭐ A UNIDADE das células é uma fracção da bola da peça — um mundo sem escala própria.
    let unidade = (setup.ball_radius * 2f32.powi(-16)).max(f32::MIN_POSITIVE);
    let mut u = Vec::with_capacity(32);
    for v in [tabela.chave.entradas, CEU_FATIAS, tabela.quadro, VELHA] {
        u.extend_from_slice(&v.to_le_bytes());
    }
    // ⭐ As VAGAS da lista: duas fatias por pixel, e nunca mais do que um despacho indirecto alcança.
    let vagas = (width * height).saturating_mul(2).min(MAX_GRUPOS_1D * 64);
    let lista = match tabela.lista.take() {
        Some((b, n)) if n >= vagas => (b, n),
        _ => (
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("ceu-tempo-lista"),
                size: (4 + u64::from(vagas) * 2) * 4,
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            vagas,
        ),
    };
    for f in [unidade, CONCORDANCIA] {
        u.extend_from_slice(&f32::to_le_bytes(f));
    }
    for v in [vagas, 0] {
        u.extend_from_slice(&v.to_le_bytes());
    }
    let ub = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("ceu-tempo-tabela"),
        contents: &u,
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let bgl1 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("ceu-tempo"),
        entries: &[
            crate::trace::uniforme(0),
            crate::trace::armazem(1, false),
            crate::trace::armazem(2, false),
        ],
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("ceu-tempo"),
        bind_group_layouts: &[Some(bgl0), Some(&bgl1)],
        immediate_size: 0,
    });
    let fonte = format!("{molde}{WGSL}");
    let mut kernel = |nome: &str| {
        cache
            .entry_with_layout(device, &fonte, fita, nome, Some(&layout))
            .clone()
    };
    let bg1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("ceu-tempo"),
        layout: &bgl1,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: ub.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: tabela.buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: lista.0.as_entire_binding(),
            },
        ],
    });
    let grupos = (width.div_ceil(8), height.div_ceil(8));
    let indirecto = tabela.args.take().unwrap_or_else(|| {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ceu-tempo-args"),
            size: 12,
            usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    });
    if setup.ceu_tempo == CeuTempo::Acumula {
        let (pede, args, marcha, le) = (
            kernel("ceu_tempo_pede"),
            kernel("ceu_tempo_args"),
            kernel("ceu_tempo_marcha"),
            kernel("ceu_tempo_le"),
        );
        enc.clear_buffer(&lista.0, 0, Some(16));
        let passe = |enc: &mut wgpu::CommandEncoder,
                     p: &wgpu::ComputePipeline,
                     disp: &dyn Fn(&mut wgpu::ComputePass<'_>)| {
            let mut cp = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("ceu-tempo"),
                timestamp_writes: None,
            });
            cp.set_pipeline(p);
            cp.set_bind_group(0, bg0, &[]);
            cp.set_bind_group(1, &bg1, &[]);
            disp(&mut cp);
        };
        passe(enc, &pede, &|cp| {
            cp.dispatch_workgroups(grupos.0, grupos.1, 1)
        });
        passe(enc, &args, &|cp| cp.dispatch_workgroups(1, 1, 1));
        enc.copy_buffer_to_buffer(&lista.0, 4, &indirecto, 0, 12);
        passe(enc, &marcha, &|cp| {
            cp.dispatch_workgroups_indirect(&indirecto, 0)
        });
        passe(enc, &le, &|cp| {
            cp.dispatch_workgroups(grupos.0, grupos.1, 1)
        });
    } else {
        for nome in ["ceu_tempo_zera", "ceu_tempo_grava"] {
            let p = kernel(nome);
            let mut cp = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("ceu-tempo"),
                timestamp_writes: None,
            });
            cp.set_pipeline(&p);
            cp.set_bind_group(0, bg0, &[]);
            cp.set_bind_group(1, &bg1, &[]);
            cp.dispatch_workgroups(grupos.0, grupos.1, 1);
        }
    }
    tabela.lista = Some(lista);
    tabela.args = Some(indirecto);
    cache.ceu_tempo = Some(tabela);
}

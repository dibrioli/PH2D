//! ⭐⭐⭐ **O CORPO DO PINTOR, em WGSL** — o grupo `1`, as leis de leitura e as duas entradas.
//!
//! # Por que um ficheiro irmão
//!
//! O [`super::paint`] é a **montagem**: a `PaintSetup`, os buffers, o pipeline e o despacho. Este é
//! o **texto do shader**, que é outro assunto e outra língua — e o ficheiro passou o tecto de LOC
//! da workspace ao crescer com a subsuperfície (`docs/Render3d/10`).
//!
//! ⛔ **Split, nunca allowlist** (`CLAUDE.md` §5.0): a cura de um tecto vermelho é corte por
//! responsabilidade, e a fronteira aqui desenha-se sozinha — *o que se compila e o que se monta*.
//!
//! ⚠️ **As marcas `{...}` continuam a ser substituídas por quem MONTA**, no irmão: elas são lidas
//! dos ficheiros que declaram cada constante, nunca escritas aqui. *Uma constante transcrita é uma
//! divergência à espera de um dia em que alguém mexa na outra.*

/// ⭐⭐⭐ **A CURVATURA MÉDIA COM SINAL de um ponto**, em WGSL — `H = ∇²f/2`, o gémeo exacto do
/// [`ph2d_field_render::curvatura`]. Ver lá porque ela NÃO pode vir de `fwidth` e porque o passo
/// dela não é o da normal.
///
/// # ⚠️⚠️ Porque ela é uma const PÚBLICA e não uma linha do [`PINTOR`]
///
/// Ela tem **dois leitores**: o pintor (que a espeta na marca `{CURVATURA}`) e o **instrumento que
/// mede a curvatura nos dois motores** — a dívida que o `estilo_tests` deixou nomeada, e que só se
/// paga comparando a GRANDEZA em vez do pixel. ⛔ *Transcrever estas dezoito linhas num arnês de
/// teste seria medir uma cópia da lei e chamar-lhe paridade* — é a mesma razão que faz `{BLUR_COS}`
/// e `{PISO_LUZ}` serem lidos do ficheiro que os declara.
///
/// ⛔⛔ **O `ε` ERA lido de `pintor.knobs.z` e HOJE é ARGUMENTO — a premissa daquela recusa morreu
/// em 2026-09-19.** Ela dizia: *«passar o `ε` por argumento mudaria o texto do produto para servir o
/// instrumento»*, e era verdade enquanto o **produto** tivesse um `ε` só. A auditoria da camada de
/// estilo (`docs/Render3d/11` §10) trouxe o segundo: o material pede o **óptimo de PRECISÃO** e o
/// estilo pede uma **ESCALA ARTÍSTICA**, e elas não podem ser o mesmo número.
///
/// ⇒ *quem move o número que tornava algo inalcançável tem de reconferir a nota* (`CLAUDE.md` §0.0).
/// Hoje o argumento serve o PRODUTO, e o instrumento passa a ser o segundo beneficiário em vez do
/// único — que é a ordem certa.
///
/// ⚠️ **Ela deixou de viver dentro do `com_a_curvatura`**, e isso compra duas coisas: o SINAL passa
/// a ser legível por quem o queira (a `W8`), e numa fronteira entre dois materiais que a leem as
/// cinco amostras passam a ser pagas **uma vez** em vez de duas. *O valor é o mesmo `f32`.*
pub const CURVATURA: &str = r"
fn {NOME}(p: vec3<f32>) -> f32 {
    let e = {EPS};
    if (e <= 0.0) { return 0.0; }
    let o0 = vec3<f32>( 1.0, -1.0, -1.0);
    let o1 = vec3<f32>(-1.0, -1.0,  1.0);
    let o2 = vec3<f32>(-1.0,  1.0, -1.0);
    let o3 = vec3<f32>( 1.0,  1.0,  1.0);
    let soma = field(p + o0 * e) + field(p + o1 * e) + field(p + o2 * e) + field(p + o3 * e);
    let laplaciano = (soma - 4.0 * field(p)) / (2.0 * e * e);
    return laplaciano * 0.5;
}
";

/// O corpo do pintor — o grupo `1`, as leis de leitura e as duas entradas.
///
/// ⚠️ `{BLUR_COS}` e `{PISO_LUZ}` são **lidos do ficheiro** que os declara, nunca escritos aqui: uma
/// constante transcrita é uma divergência à espera de um dia em que alguém mexa na outra. O mesmo
/// vale para `{CURVATURA}`, que é a [`CURVATURA`] — ver lá porque ela tem dois leitores.
pub(crate) const PINTOR: &str = r"
// ── o grupo 1: o que só o pintor lê ───────────────────────────────────────────────────────────
struct Pintor {
    knobs: vec4<f32>,  // stops, pixel_world, curv_eps (o do MATERIAL), raio da peça
    fundo: vec4<f32>,  // o fundo em LINEAR pré-multiplicado, para a média da borda
    modo: vec4<u32>,   // view, bordas, fundo empacotado, materiais
    // ⭐ `x` = há gémeas foscas empacotadas (ver `ler_mat_fosca`). `0` é o caminho anterior ao
    // report do dono de 2026-09-17, ao bit.
    modo2: vec4<u32>,
    // ⭐⭐⭐ **O CAMPO DO CHÃO** (`ph2d_field_render::ground_bounce`): `xy` = o canto `(x, z)`,
    // `z` = o passo, `w` = a altura do plano. A contagem de células por aresta vive em `modo2.y`,
    // e `0` ali quer dizer «campo vazio» — o caminho de sempre, ao bit.
    chao_campo: vec4<f32>,
    // ⭐ **Uma radiância por lâmpada**, na MESMA ordem das posições do `Setup`.
    lamp: array<vec4<f32>, {MAX_LAMPS}>,
    // ⭐⭐⭐ **A CAMADA DE ESTILO** (`ph2d_style`, a `W8`) — e repare que ela é a PRÓPRIA struct do
    // gémeo, e não cinco `vec4` soltos: a arrumação dos vinte números vive numa função só
    // (`ph2d_style::wgsl::pack`) e este lado é o LEITOR dela. ⛔ Cinco campos nomeados aqui seriam a
    // segunda resposta à mesma tabela.
    estilo: Estilo,
};
@group(1) @binding(0) var<uniform> ceu: Ceu;
@group(1) @binding(1) var<uniform> pintor: Pintor;
@group(1) @binding(2) var<storage, read> tabela: array<f32>;
@group(1) @binding(3) var<storage, read> materiais: array<f32>;
@group(1) @binding(4) var<storage, read_write> saida: array<u32>;
// ⭐⭐⭐ **AS SONDAS** (`ph2d_field_render::probes`): `PROBE_GRID³` sondas × `SH_STRIDE` floats — os
// nove coeficientes esféricos por canal (27) e a bandeira «está dentro da peça» (o 28.º).
@group(1) @binding(5) var<storage, read_write> sondas: array<f32>;
// ⭐⭐⭐ **A COR QUE A PEÇA DEVOLVE AO CHÃO** (`ph2d_field_render::ground_bounce`): `n²` irradiâncias
// em ordem `z * n + x`, três floats cada. ⚠️ **Ela é assada na CPU e ENVIADA**, ao contrário das
// sondas — ver a medição no `PaintSetup::ground_bounce`.
@group(1) @binding(6) var<storage, read> chao_luz: array<f32>;
// ⭐⭐⭐ **O QUADRO EM CENA-LINEAR, que só o BRILHO lê** (`docs/Render3d/12`) — o gémeo do
// `ph2d_field_render::brilho::campo_de_cena`.
//
// ⚠️ **Com o brilho desligado ele tem UM texel** (o chamador liga uma rede de 1 elemento) e o
// `pinta` **não escreve**, porque `modo2.w` o diz. ⭐ O ramo é de graça: a bandeira é **uniforme em
// todo o despacho**, logo não há divergência de warp nenhuma — e sem ele o caminho de omissão
// pagaria uma escrita de `16 B` por pixel para lado nenhum.
@group(1) @binding(7) var<storage, read_write> cena_hdr: array<vec4<f32>>;

const BLUR_COS: f32 = {BLUR_COS};
const PISO_LUZ: f32 = {PISO_LUZ};
const PROBE_GRID: u32 = {PROBE_GRID}u;
const PROBE_DIRS: u32 = {PROBE_DIRS}u;
const PROBE_MARGIN: f32 = {PROBE_MARGIN};
const SH_STRIDE: u32 = 28u;
// `4π / PROBE_DIRS`, formatado do MESMO f32 que a CPU calcula.
const SH_PESO: f32 = {SH_PESO};
const SH_A1: f32 = {SH_A1};
const SH_A2: f32 = {SH_A2};
const SQRT3: f32 = {SQRT3};
const PACKED: u32 = {PACKED}u;

// ⚠️ **A rede é `materiais[0]`**, e ela não é decorativa: o dono pode vir de uma peça que já mudou
// entre a marcha e a pintura. Ler fora do buffer devolveria lixo; pintar com o primeiro material é
// o que o artista lê como «ainda não actualizou», que é o que de facto aconteceu.
fn ler_mat(i: u32) -> Mat {
    var j = i;
    if (j >= pintor.modo.w) { j = 0u; }
    return mat_em(j * PACKED);
}

// ⭐⭐⭐ **A GÉMEA FOSCA do material `i`** — ver `ph2d_material::Surface::matte` e o
// `PaintSetup::matte`. Com `pintor.modo2.x == 0` não há gémeas empacotadas e ela devolve o material
// de sempre, que é o caminho anterior ao report do dono, ao bit.
fn ler_mat_fosca(i: u32) -> Mat {
    var j = i;
    if (j >= pintor.modo.w) { j = 0u; }
    if (pintor.modo2.x == 0u) { return mat_em(j * PACKED); }
    return mat_em((pintor.modo.w + 1u + j) * PACKED);
}

// ⭐ **O material do CHÃO** — a difusa branca que vive depois dos da peça. Ver `PaintSetup::catcher`.
fn mat_do_chao() -> Mat {
    return mat_em(pintor.modo.w * PACKED);
}

fn mat_em(o: u32) -> Mat {
    var m: Mat;
    m.base_color_weight    = vec4<f32>(materiais[o +  0u], materiais[o +  1u], materiais[o +  2u], materiais[o +  3u]);
    m.specular_color_metal = vec4<f32>(materiais[o +  4u], materiais[o +  5u], materiais[o +  6u], materiais[o +  7u]);
    m.coat_color_diffrough = vec4<f32>(materiais[o +  8u], materiais[o +  9u], materiais[o + 10u], materiais[o + 11u]);
    m.emission_specweight  = vec4<f32>(materiais[o + 12u], materiais[o + 13u], materiais[o + 14u], materiais[o + 15u]);
    m.darkening_coatweight = vec4<f32>(materiais[o + 16u], materiais[o + 17u], materiais[o + 18u], materiais[o + 19u]);
    m.attenuation_coatior  = vec4<f32>(materiais[o + 20u], materiais[o + 21u], materiais[o + 22u], materiais[o + 23u]);
    m.prepared             = vec4<f32>(materiais[o + 24u], materiais[o + 25u], materiais[o + 26u], materiais[o + 27u]);
    m.emissive             = vec4<f32>(materiais[o + 28u], materiais[o + 29u], materiais[o + 30u], materiais[o + 31u]);
    m.ss_color_weight      = vec4<f32>(materiais[o + 32u], materiais[o + 33u], materiais[o + 34u], materiais[o + 35u]);
    m.ss_mfp_aniso         = vec4<f32>(materiais[o + 36u], materiais[o + 37u], materiais[o + 38u], materiais[o + 39u]);
    m.ss_brdf_thin         = vec4<f32>(materiais[o + 40u], materiais[o + 41u], materiais[o + 42u], materiais[o + 43u]);
    m.ss_btdf_curv         = vec4<f32>(materiais[o + 44u], materiais[o + 45u], materiais[o + 46u], materiais[o + 47u]);
    return m;
}

// ⭐⭐⭐ **A CURVATURA DESTE PONTO, escrita no material** — o gémeo exacto do
// `ph2d_material::Surface::at_curvature`. Ver `ph2d_field_render::curvatura` para a lei e para
// porque ela NAO pode vir de `fwidth`: aquilo e' por quad de 2x2 e a CPU e' por pixel, e as duas
// respostas partiriam a paridade que esta linha tem em 100,000 %.
//
// ⚠️ **Ela so' e' calculada quando alguem a le** — com a subsuperficie macica desligada (a
// omissao) o `if` sai antes de tocar no campo, e o quadro e' o de sempre ao bit.
fn com_a_curvatura(m_in: Mat, k: f32) -> Mat {
    var m = m_in;
    if (m.ss_color_weight.a <= 0.0 || m.ss_brdf_thin.a > 0.5) { return m; }
    // ⭐ **O MÓDULO é tomado aqui**, como na CPU: a lei do OpenPBR pede um comprimento e a tinta por
    // curvatura da `W8` pede o SINAL. Tomar o módulo antes ou depois de guardar dá o mesmo `f32`.
    m.ss_btdf_curv.a = abs(k);
    return m;
}

{CURVATURA}

// **Este material lê a curvatura?** — a metade que o `com_a_curvatura` já perguntava, com nome.
fn mat_le_curvatura(m: Mat) -> bool {
    return m.ss_color_weight.a > 0.0 && m.ss_brdf_thin.a <= 0.5;
}

// A base é ortonormal, logo a transposta é a inversa — a mesma `ViewBasis::world_to_view` da CPU.
fn mundo_para_vista(w: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(dot(w, s.right), dot(w, s.up), dot(w, s.fwd));
}

// ⭐ **A OCLUSÃO SUAVIZADA** — o `ph2d_field_render::blur_occlusion`, célula a célula.
//
// ⚠️ Os vizinhos leem a oclusão CRUA (a CPU suaviza para uma cópia), e um pixel que não acerta
// devolve o valor dele sem tocar em nada — ali `n0` é o vector zero e a cerca da normal fecha
// sozinha, que é exactamente o `continue` da CPU.
fn ceu_em(x: u32, y: u32, i: u32, n0: vec3<f32>) -> f32 {
    var soma = 0.0;
    var cont = 0u;
    for (var dy = -1; dy <= 1; dy = dy + 1) {
        for (var dx = -1; dx <= 1; dx = dx + 1) {
            let xx = i32(x) + dx;
            let yy = i32(y) + dy;
            if (xx < 0 || yy < 0 || xx >= i32(s.w) || yy >= i32(s.h)) { continue; }
            let j = u32(yy) * s.w + u32(xx);
            let c = centro[j];
            if (c.x < 0.0) { continue; }
            if (dot(n0, c.yzw) < BLUR_COS) { continue; }
            soma = soma + luz[j * passo_da_luz()];
            cont = cont + 1u;
        }
    }
    if (cont > 0u) { return soma / f32(cont); }
    return luz[i * passo_da_luz()];
}

// A base é ortonormal, logo a transposta é a inversa — a volta do `mundo_para_vista`.
fn vista_para_mundo(v: vec3<f32>) -> vec3<f32> {
    return s.right * v.x + s.up * v.y + s.fwd * v.z;
}

// ⭐⭐⭐ **A RADIÂNCIA QUE SAI DO PONTO ACERTADO, por luz DIRECTA** — e é isto que faz o ricochete
// ser **UM**: a luz que sai dali não traz o que lhe chegou por sua vez.
//
// O `ph2d_field_render::bounce_slice`, passo 4, linha a linha.
fn devolvida_de(q: vec3<f32>, nq_vista: vec3<f32>, veio_de: vec3<f32>) -> vec3<f32> {
    // ⚠️ **Sem fronteira suavizada, e o doc da `Surfaces::of` diz porquê:** *«um raio de ricochete
    // não tem pixel — ele acerta um ponto —, e inventar-lhe uma largura seria inventar a
    // resposta»*. Só o dono (`.a`) é lido; a largura não muda quem ele é.
    // ⭐⭐⭐ **A GÉMEA FOSCA** — ver `ph2d_material::Surface::matte`, que traz a medição na peça do
    // dono: o lóbulo especular é uma quase-delta que `48` direcções FIXAS não amostram, e carregá-lo
    // faz o estimador **deixar de convergir** (`96` direcções leem PIOR que `48`).
    let m = ler_mat_fosca(dono_mix(q, pintor.knobs.y).a);
    // ⭐ O observador daquele ponto é **quem lhe perguntou**: o raio veio de `-veio_de`.
    let v = mundo_para_vista(-veio_de);
    let nq = vista_para_mundo(nq_vista);
    let erguido = q + nq * (s.hit_eps * 4.0);
    let piso = PISO_LUZ * PISO_LUZ;
    var sai = vec3<f32>(0.0);
    for (var l: u32 = 0u; l < s.n_lamps; l = l + 1u) {
        let d = s.lamps[l].xyz - q;
        let cru = dot(d, d);
        // ⚠️ **O braço degenerado é o do sombreador**: abaixo do piso a direcção é a NORMAL, que é
        // o limite finito. A mesma lei da `shade_render::chega_da_lampada`.
        var to_light = nq_vista;
        var vis = 1.0;
        if (cru > piso) {
            let dist = sqrt(cru);
            let dir = d / dist;
            to_light = mundo_para_vista(dir);
            // ⭐ **Só quem VÊ a luz paga raio** — e isto não muda a resposta: com `N·L <= 0` o
            // `mx_direct` devolve zero seja qual for a visibilidade. É a mesma cerca do passe da
            // sombra, e o gate de paridade contra a referência de CPU é quem o prova.
            if (dot(nq, dir) > 0.0) {
                vis = visivel(erguido, dir, cerca_da_bola(erguido, dir, dist), 8.0);
            }
        }
        sai = sai + mx_direct(m, nq_vista, v, to_light, pintor.lamp[l].rgb * vis / max(cru, piso));
    }
    return sai;
}

";

/// ⭐⭐ **A EXTRACÇÃO NÃO PODE APODRECER EM SILÊNCIO — e este gate corre SEM PLACA.**
///
/// A [`CURVATURA`] saiu do corpo do [`PINTOR`] para ter um segundo leitor (ver o doc dela). Os dois
/// modos de falha dessa mudança são **de GPU**: sem a substituição o shader leva um `{CURVATURA}`
/// literal e o `naga` recusa-o; sem a const a função `curvatura_em` fica por declarar. ⛔⛔ **Os
/// gates que veriam qualquer um dos dois são `#[ignore]`, logo o CI NUNCA os corre** — é a lei do
/// `CLAUDE.md` §5.0 sobre *skip gracioso não é verde*.
///
/// ⇒ as duas metades, medidas no TEXTO, numa máquina qualquer:
/// a marca existe **uma** vez no corpo, e **alguém a substitui**.
#[cfg(test)]
mod extraccao_tests {
    #[test]
    fn a_marca_da_curvatura_existe_uma_vez_e_alguem_a_substitui() {
        let marcas = super::PINTOR.matches("{CURVATURA}").count();
        assert_eq!(
            marcas, 1,
            "o corpo do pintor tem {marcas} marcas `{{CURVATURA}}` — uma a menos e a função fica \
             por declarar; uma a mais e o shader declara-a duas vezes"
        );
        // ⚠️ **O FIO, e não só a porta.** Sem esta metade, apagar o `.replace` do irmão deixa o
        // gate verde e o shader com um `{CURVATURA}` literal lá dentro.
        let montagem = include_str!("paint_fonte.rs");
        assert!(
            montagem.contains(r#".replace("{CURVATURA}", &{"#),
            "ninguém substitui a marca `{{CURVATURA}}` na montagem do shader"
        );
        // ⭐⭐⭐ **E a montagem gera as DUAS funções do molde** — a do MATERIAL com o `ε` de sempre e
        // a do ESTILO com o dele. ⛔ Sem esta metade, uma montagem que preenchesse o molde uma vez
        // só deixava o shader sem `curvatura_do_estilo_em` e o gate acima verde.
        // ⚠️ **Procura o LITERAL do nome e não a chamada colada a ele:** o `cargo fmt` parte uma
        // chamada longa em várias linhas, e um censo que exija `molde("…"` casa zero sobre produto
        // certo — a mesma armadilha que um filtro de teste reformatado já custou a esta linha hoje.
        for nome in ["\"curvatura_em\"", "\"curvatura_do_estilo_em\""] {
            assert!(
                montagem.contains(nome),
                "a montagem não gera a função {nome} do molde"
            );
        }
        // E a const declara exactamente a função que o corpo chama.
        assert_eq!(
            super::CURVATURA
                .matches("fn {NOME}(p: vec3<f32>) -> f32")
                .count(),
            1,
            "o MOLDE não declara a função exactamente uma vez"
        );
        // ⭐⭐⭐ **E o molde tem de ter as DUAS marcas** — o nome E o passo. Sem a do passo ele
        // geraria duas funções idênticas, e a do estilo mediria a curvatura do MATERIAL: a borda
        // dura voltava sem uma linha de lei ter mudado.
        for marca in ["{NOME}", "{EPS}"] {
            assert!(
                super::CURVATURA.contains(marca),
                "o molde da curvatura perdeu a marca {marca}"
            );
        }
        // ⚠️⚠️ **Quem CHAMA está noutra metade** (`PINTOR_SONDAS`), e a 1.ª redacção deste gate
        // procurou-a aqui e reprovou — *as metades são UM shader* (são TRÊS desde 2026-09-19, com o
        // `PINTOR_MOLE`), como o cabeçalho deste ficheiro diz, e o corte entre elas foi sempre um
        // tecto de LOC e nunca um assunto. A declaração
        // entra por esta marca, a chamada mora no irmão, e o gate tem de olhar para os dois.
        assert!(
            crate::paint_wgsl_sondas::PINTOR_SONDAS.contains("curvatura_em(p)")
                && crate::paint_wgsl_sondas::PINTOR_SONDAS.contains("curvatura_do_estilo_em(p)"),
            "o shader deixou de CHAMAR uma das duas medições — a extracção ficou órfã"
        );
    }
}

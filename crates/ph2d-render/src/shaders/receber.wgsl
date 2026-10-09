// A luz 2D RECEBIDA (plano `docs/Luz2D/01_plano.md` §2.2–§2.4) — UMA função, concatenada à frente do
// shader de sprite E da colagem do vetor (`band_blit.wgsl`): a mesma conta nos dois.
//
// `luz_fluencia` é a fluência `F` por sonda da grade de luz (a c0 do solver, `Rgba16Float`), ancorada
// no mundo: o centro do texel `(i, j)` está em `origem + (i + ½, j + ½)·texel`. A luz que uma
// superfície DE FRENTE recebe é `F / 2π` (o forno branco, plano §2.3).
//
// ⛔ `ligada == 0` NÃO multiplica por 1: o ramo não é tomado e a cor sai intacta — é isso que faz o
// quadro sem luz byte-idêntico POR CONSTRUÇÃO (gate G6), como o `SEM_PELE` da pele.

struct LuzUniforme {
    // mundo → coordenada de textura: `uv = (mundo − origem) · escala`, `escala = 1/(texel·tamanho)`.
    origem: vec2<f32>,
    escala: vec2<f32>,
    // tela (px do alvo, centro do pixel) → mundo, para quem não tem a posição do mundo (o vetor):
    // `mundo = (tela_x.x·px + tela_x.y·py + tela_x.z, tela_y.x·px + tela_y.y·py + tela_y.z)`.
    tela_x: vec4<f32>,
    tela_y: vec4<f32>,
    ligada: u32,
    _p0: u32,
    _p1: u32,
    _p2: u32,
};

@group(3) @binding(0) var<uniform> luz: LuzUniforme;
@group(3) @binding(1) var luz_fluencia: texture_2d<f32>;
@group(3) @binding(2) var luz_amostra: sampler;

const LUZ_INV_TAU: f32 = 0.15915494309189535;

// A luz que chega ao ponto `mundo` (metros), por canal. Bilinear entre centros de texel.
fn luz_em(mundo: vec2<f32>) -> vec3<f32> {
    let uv = (mundo - luz.origem) * luz.escala;
    return textureSampleLevel(luz_fluencia, luz_amostra, uv, 0.0).rgb * LUZ_INV_TAU;
}

fn luz_tela_para_mundo(px: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(
        luz.tela_x.x * px.x + luz.tela_x.y * px.y + luz.tela_x.z,
        luz.tela_y.x * px.x + luz.tela_y.y * px.y + luz.tela_y.z,
    );
}

// `cor` linear (pré-multiplicada ou não: a luz é linear). `sem_luz != 0` = `NaoRecebeLuz`.
fn luz_receber(cor: vec4<f32>, mundo: vec2<f32>, sem_luz: u32) -> vec4<f32> {
    if (luz.ligada == 0u || sem_luz != 0u) {
        return cor;
    }
    return vec4<f32>(cor.rgb * luz_em(mundo), cor.a);
}

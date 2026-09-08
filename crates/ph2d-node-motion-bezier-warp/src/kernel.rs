//! O `motion.bezier_warp` **no dispositivo** (ciclo 3, W5a — doc 106 §3).
//!
//! ## Por que ele podia ir, e ninguém tinha ido
//!
//! O nó irmão — o `motion.four_point_warp` — já corre no dispositivo com **a mesma
//! forma**: quatro reduções que dão a caixa envolvente do layout, e um mapa por
//! elemento que a lê. Este é o mesmo `reduce → broadcast → map` com outra álgebra no
//! meio (um patch de Coons em vez de uma homografia), e a álgebra é **polinomial**
//! (HR-5: nem uma transcendental, nem sequer um `sqrt`).
//!
//! ⚠️ **O que faltava não era capacidade, era o TETO DO UNIFORME — e ele foi MEDIDO,
//! não suposto.** Os 24 offsets ocupam `24 × 4 = 96 B`, e o cabeçalho (`count`,
//! `playhead`) mais o bit de broadcast levam o total a **108 B** dos
//! [`ph2d_gpu_cook::UNIFORM_BYTES`] `128`. O portão `motion_gpu_kernel_budgets` do
//! shell mede exactamente isto sobre **todo** kernel do registry, então este não passa
//! por uma folga que ninguém vigia. ⛔ Um 25.º param neste nó não cabe, e a resposta
//! nesse dia é subir o slot (o comentário do `UNIFORM_BYTES` já diz porquê: *«isto é um
//! slot, não uma alocação por elemento — a folga é grátis»*), nunca cortar um controlo
//! que a referência tem.
//!
//! ## As duas identidades, escritas como ESTRUTURA
//!
//! O `eval` tem dois caminhos que devolvem o stream **verbatim**: os 24 offsets a zero
//! ([`super::Offsets::is_neutral`]) e a caixa degenerada (uma linha ou um ponto). Os
//! dois são escritos aqui como um `write_P(i, bw_p)` **próprio**, e não como o caso
//! `s == p` da mistura geral.
//!
//! ⚠️ **A diferença não é estilo: `p + (p − p) * f` NÃO é `p`.** Quando `p` é `-0.0` a
//! soma devolve `+0.0`, e o nó recém-largado — que é precisamente o caso neutro —
//! passaria a mexer bits que a CPU não mexe. A lei é a mesma que o cisalhamento da W3
//! pagou: *uma identidade que se quer AO BIT escreve-se num ramo separado, senão ela é
//! um facto de aritmética e a aritmética muda.*

use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel, ReduceOp, ReduceSpec};
use ph2d_nodegraph::port::Dim;

use crate::VALUE_COL;

/// A caixa envolvente do layout, pelas quatro reduções do canal
/// `reduce → broadcast → map` — **a mesma tabela que o `motion.four_point_warp`
/// declara**, porque é a mesma pergunta sobre o mesmo stream.
///
/// ⚠️ **`Min`/`Max` são exactos sobre floats** (associativos *e* sem arredondamento),
/// então a caixa que o dispositivo dobra tem os mesmos bits que a da CPU: o ε deste nó
/// mora todo na aritmética do patch, nunca na caixa.
///
/// ⚠️ A identidade `0.0` casa com o braço degenerado do `eval`: um stream sem `P` lê
/// origem nos dois caminhos, a caixa fica um ponto (`w = h = 0`), e o nó devolve o
/// layout intacto.
pub(crate) static REDUCES: &[ReduceSpec] = &[
    ReduceSpec {
        name: "xmin",
        column: "P",
        dim: Dim::Vec2,
        port: 0,
        op: ReduceOp::Min,
        value: "v.x",
        params: &[],
        identity: [0.0; 4],
    },
    ReduceSpec {
        name: "xmax",
        column: "P",
        dim: Dim::Vec2,
        port: 0,
        op: ReduceOp::Max,
        value: "v.x",
        params: &[],
        identity: [0.0; 4],
    },
    ReduceSpec {
        name: "ymin",
        column: "P",
        dim: Dim::Vec2,
        port: 0,
        op: ReduceOp::Min,
        value: "v.y",
        params: &[],
        identity: [0.0; 4],
    },
    ReduceSpec {
        name: "ymax",
        column: "P",
        dim: Dim::Vec2,
        port: 0,
        op: ReduceOp::Max,
        value: "v.y",
        params: &[],
        identity: [0.0; 4],
    },
];

/// A forma de dispositivo de [`super::coons`] + [`super::boundary_in_world`], operação
/// por operação.
///
/// ⚠️ **As tangentes dos terços são CALCULADAS, não escritas como literal**, e isto foi
/// medido antes de a linha existir: `1.0 − (1.0/3.0)` em `f32` dá `0,66666663` (o caso
/// de empate arredonda para par), enquanto o literal `0.6666667` é `0,66666669` — **bits
/// diferentes**. Escrever a tabela do quadrado unitário à mão daria um patch neutro que
/// já não é a identidade, e o defeito apareceria como um layout a tremer um ulp ao
/// largar o nó. *A porta é a EXPRESSÃO, não a tabela que ela produz.*
///
/// ⚠️ **A fronteira é reconstruída POR ELEMENTO**, como a homografia do irmão: ela só
/// depende da caixa e dos params, mas partilhá-la exigiria um segundo passe, e são ~150
/// flops contra um kernel limitado por memória. O preço está medido no doc 106 §4.
pub(crate) const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let bw_xmin = reduce_xmin();\n\
        let bw_xmax = reduce_xmax();\n\
        let bw_ymin = reduce_ymin();\n\
        let bw_ymax = reduce_ymax();\n\
        let bw_lo = vec2<f32>(bw_xmin, bw_ymin);\n\
        let bw_hi = vec2<f32>(bw_xmax, bw_ymax);\n\
        let bw_w = bw_xmax - bw_xmin;\n\
        let bw_h = bw_ymax - bw_ymin;\n\
        let bw_p = read_in_P(i);\n\
        if (bw_w < 1e-6 || bw_h < 1e-6 || bw_is_neutral()) {\n\
        // Os dois caminhos em que o `eval` devolve o stream VERBATIM. Ver o cabecalho:\n\
        // escrever `bw_p` aqui e a mistura no outro ramo e' o que torna a identidade\n\
        // um facto de ESTRUTURA, e nao de `p + 0.0` -- que troca `-0.0` por `+0.0`.\n\
        \x20   write_P(i, bw_p);\n\
        } else {\n\
        \x20   let bw_warp = read_warp_v(0u);\n\
        // O quadrado unitario, e as tangentes nos TERCOS -- a posicao em que a cubica\n\
        // degenera na recta por identidade polinomial.\n\
        \x20   let u_tl = vec2<f32>(0.0, 1.0);\n\
        \x20   let u_tr = vec2<f32>(1.0, 1.0);\n\
        \x20   let u_br = vec2<f32>(1.0, 0.0);\n\
        \x20   let u_bl = vec2<f32>(0.0, 0.0);\n\
        \x20   let c_tl = bw_world(u_tl, bw_lo, bw_hi) + bw_warp * vec2<f32>(params.tl_dx, params.tl_dy);\n\
        \x20   let c_tr = bw_world(u_tr, bw_lo, bw_hi) + bw_warp * vec2<f32>(params.tr_dx, params.tr_dy);\n\
        \x20   let c_br = bw_world(u_br, bw_lo, bw_hi) + bw_warp * vec2<f32>(params.br_dx, params.br_dy);\n\
        \x20   let c_bl = bw_world(u_bl, bw_lo, bw_hi) + bw_warp * vec2<f32>(params.bl_dx, params.bl_dy);\n\
        \x20   let g_top_a = bw_world(bw_third_a(u_tl, u_tr), bw_lo, bw_hi) + bw_warp * vec2<f32>(params.top_a_dx, params.top_a_dy);\n\
        \x20   let g_top_b = bw_world(bw_third_b(u_tl, u_tr), bw_lo, bw_hi) + bw_warp * vec2<f32>(params.top_b_dx, params.top_b_dy);\n\
        \x20   let g_rgt_a = bw_world(bw_third_a(u_tr, u_br), bw_lo, bw_hi) + bw_warp * vec2<f32>(params.right_a_dx, params.right_a_dy);\n\
        \x20   let g_rgt_b = bw_world(bw_third_b(u_tr, u_br), bw_lo, bw_hi) + bw_warp * vec2<f32>(params.right_b_dx, params.right_b_dy);\n\
        \x20   let g_bot_a = bw_world(bw_third_a(u_br, u_bl), bw_lo, bw_hi) + bw_warp * vec2<f32>(params.bottom_a_dx, params.bottom_a_dy);\n\
        \x20   let g_bot_b = bw_world(bw_third_b(u_br, u_bl), bw_lo, bw_hi) + bw_warp * vec2<f32>(params.bottom_b_dx, params.bottom_b_dy);\n\
        \x20   let g_lft_a = bw_world(bw_third_a(u_bl, u_tl), bw_lo, bw_hi) + bw_warp * vec2<f32>(params.left_a_dx, params.left_a_dy);\n\
        \x20   let g_lft_b = bw_world(bw_third_b(u_bl, u_tl), bw_lo, bw_hi) + bw_warp * vec2<f32>(params.left_b_dx, params.left_b_dy);\n\
        \x20   let bw_u = (bw_p.x - bw_xmin) / bw_w;\n\
        \x20   let bw_v = (bw_p.y - bw_ymin) / bw_h;\n\
        // As quatro bordas, ja' orientadas no sentido do patch (o BOTTOM corre BR->BL,\n\
        // logo as tangentes dele entram trocadas -- errar isto da' um patch que ainda\n\
        // interpola os cantos e se cruza no meio).\n\
        \x20   let e_top = bw_bezier(c_tl, g_top_a, g_top_b, c_tr, bw_u);\n\
        \x20   let e_bot = bw_bezier(c_bl, g_bot_b, g_bot_a, c_br, bw_u);\n\
        \x20   let e_lft = bw_bezier(c_bl, g_lft_a, g_lft_b, c_tl, bw_v);\n\
        \x20   let e_rgt = bw_bezier(c_tr, g_rgt_a, g_rgt_b, c_br, 1.0 - bw_v);\n\
        \x20   let ruled_v = (1.0 - bw_v) * e_bot + bw_v * e_top;\n\
        \x20   let ruled_u = (1.0 - bw_u) * e_lft + bw_u * e_rgt;\n\
        \x20   let bilinear = (1.0 - bw_u) * (1.0 - bw_v) * c_bl + bw_u * (1.0 - bw_v) * c_br + (1.0 - bw_u) * bw_v * c_tl + bw_u * bw_v * c_tr;\n\
        \x20   let bw_s = ruled_v + ruled_u - bilinear;\n\
        \x20   let bw_f = clamp(read_in_falloff(i), 0.0, 1.0);\n\
        \x20   write_P(i, bw_p + (bw_s - bw_p) * bw_f);\n\
        }\n",
    wgsl_lib: "\
        // O quadrado unitario mapeado na caixa envolvente -- componente a componente,\n\
        // como o `to_world` da CPU.\n\
        fn bw_world(q: vec2<f32>, lo: vec2<f32>, hi: vec2<f32>) -> vec2<f32> {\n\
            return vec2<f32>(lo.x + q.x * (hi.x - lo.x), lo.y + q.y * (hi.y - lo.y));\n\
        }\n\
        // As duas tangentes dos TERCOS de `a -> b`. Calculadas, nunca escritas como\n\
        // literal: ver o cabecalho deste modulo.\n\
        fn bw_third_a(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {\n\
            return a + (b - a) / 3.0;\n\
        }\n\
        fn bw_third_b(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {\n\
            return b - (b - a) / 3.0;\n\
        }\n\
        // Uma cubica de Bezier na base de Bernstein, com os quatro pesos explicitos --\n\
        // a forma que a CPU corre, termo a termo.\n\
        fn bw_bezier(p0: vec2<f32>, p1: vec2<f32>, p2: vec2<f32>, p3: vec2<f32>, t: f32) -> vec2<f32> {\n\
            let s = 1.0 - t;\n\
            let w0 = s * s * s;\n\
            let w1 = 3.0 * s * s * t;\n\
            let w2 = 3.0 * s * t * t;\n\
            let w3 = t * t * t;\n\
            return vec2<f32>(\n\
                w0 * p0.x + w1 * p1.x + w2 * p2.x + w3 * p3.x,\n\
                w0 * p0.y + w1 * p1.y + w2 * p2.y + w3 * p3.y);\n\
        }\n\
        // Os 24 offsets a zero -- o atalho do `Offsets::is_neutral`. Ele existe para o\n\
        // no' recem-largado nao pagar o patch, e NAO para o resultado ser outro.\n\
        fn bw_is_neutral() -> bool {\n\
            return params.tl_dx == 0.0 && params.tl_dy == 0.0\n\
                && params.tr_dx == 0.0 && params.tr_dy == 0.0\n\
                && params.br_dx == 0.0 && params.br_dy == 0.0\n\
                && params.bl_dx == 0.0 && params.bl_dy == 0.0\n\
                && params.top_a_dx == 0.0 && params.top_a_dy == 0.0\n\
                && params.top_b_dx == 0.0 && params.top_b_dy == 0.0\n\
                && params.right_a_dx == 0.0 && params.right_a_dy == 0.0\n\
                && params.right_b_dx == 0.0 && params.right_b_dy == 0.0\n\
                && params.bottom_a_dx == 0.0 && params.bottom_a_dy == 0.0\n\
                && params.bottom_b_dx == 0.0 && params.bottom_b_dy == 0.0\n\
                && params.left_a_dx == 0.0 && params.left_a_dy == 0.0\n\
                && params.left_b_dx == 0.0 && params.left_b_dy == 0.0;\n\
        }\n",
    bindings: &[
        ColumnBinding {
            column: "P",
            dim: Dim::Vec2,
            access: ColumnAccess::ReadWrite,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "falloff",
            dim: Dim::Scalar,
            access: ColumnAccess::Read,
            identity: [1.0, 0.0, 0.0, 0.0],
            port: 0,
        },
        ColumnBinding {
            column: VALUE_COL,
            dim: Dim::Scalar,
            // Desligada lê `1.0` — o `unwrap_or(1.0)` da CPU, e a mesma leitura do
            // elemento 0 (`first()`) que o irmão faz.
            access: ColumnAccess::ReadBroadcast,
            identity: [1.0, 0.0, 0.0, 0.0],
            port: 1,
        },
    ],
    // Os 24 offsets, na ordem em que o corpo os lê.
    params: &[
        "tl_dx",
        "tl_dy",
        "tr_dx",
        "tr_dy",
        "br_dx",
        "br_dy",
        "bl_dx",
        "bl_dy",
        "top_a_dx",
        "top_a_dy",
        "top_b_dx",
        "top_b_dy",
        "right_a_dx",
        "right_a_dy",
        "right_b_dx",
        "right_b_dy",
        "bottom_a_dx",
        "bottom_a_dy",
        "bottom_b_dx",
        "bottom_b_dy",
        "left_a_dx",
        "left_a_dy",
        "left_b_dx",
        "left_b_dy",
    ],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

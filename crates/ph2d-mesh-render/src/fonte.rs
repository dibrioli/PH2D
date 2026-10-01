//! ⭐⭐⭐ **A FONTE DO SHADER, composta** — o `mesh.wgsl` sozinho, ou ele mais
//! o bloco da tinta fina.
//!
//! ⛔⛔ **Porque é composição e não um `if` dentro do shader:** o
//! `@builtin(primitive_index)` só existe quando a placa anuncia
//! [`wgpu::Features::PRIMITIVE_INDEX`], e uma fonte que o mencione **falha a
//! VALIDAÇÃO INTEIRA** onde ele não está — a peça deixaria de desenhar de
//! todo, e não só a tinta fina. *Um caminho que degrada tem de degradar na
//! FONTE, porque a validação de um módulo é tudo-ou-nada.*
//!
//! ⚠️ **O bloco da LEI não precisa da capacidade** (ele recebe o índice do
//! triângulo como argumento) — quem precisa é a **entrada** `fs_main_tinta`.
//! É por isso que o corte é entre os dois e não no meio da lei: assim o gémeo
//! em WGSL é validado pelo `naga` **sem device e sem feature**, em toda
//! máquina que corra `cargo test`.

use std::borrow::Cow;

/// O corpo do shader de barro.
pub const MESH_WGSL: &str = include_str!("shaders/mesh.wgsl");

/// ⭐ **A LEI da tinta fina em WGSL** — o gémeo conferido contra a
/// `ph2d_mesh_colors`. Ela não menciona `primitive_index`.
pub const TINTA_WGSL: &str = include_str!("shaders/tinta.wgsl");

/// A entrada de fragmento que LÊ a tinta fina — a única peça que precisa da
/// capacidade, e por isso a única que fica de fora quando ela falta.
///
/// ⚠️ Ela delega no [`MESH_WGSL`] `fs_core`, que é o MESMO corpo de
/// sombreamento do `fs_main`: *duas leis de luz para a mesma peça divergem no
/// dia em que alguém afinar uma.*
pub const TINTA_ENTRADA_WGSL: &str = r#"
@fragment
fn fs_main_tinta(in: VsOut, @builtin(primitive_index) pi: u32) -> @location(0) vec4<f32> {
    // ⚠️ `armado == 0` é *nenhum plano ligado*, e aí esta entrada devolve o que
    // o `fs_main` devolveria — ao bit. Sem esta linha, uma peça sem plano leria
    // um buffer de um elemento e pintaria a peça inteira com ele.
    if (tinta_cfg.armado == 0u) {
        return fs_core(in, in.vcolor);
    }
    let t = tinta_no_ponto4(pi, in.opos);
    // ⚠️ Sem relevo, o caminho de sempre AO BIT: a normal nem é tocada.
    if ((tinta_cfg.armado & TINTA_RELEVO) == 0u) {
        return fs_core(in, t.c.xyz);
    }
    return fs_core_n(in, t.c.xyz, tinta_relevo_n(in, t.c.w, t.corpo));
}

// ⭐⭐⭐ **A NORMAL INCLINADA PELO RELEVO** — *bump mapping* sem parametrização
// (M. Mikkelsen, «Bump Mapping Unparametrized Surfaces on the GPU», 2010): a
// normal da superfície `p + h·n` sai das derivadas de ECRÃ da posição e da
// altura, sem tangentes e sem UV — que é o que uma malha esculpida não tem.
//
// ⚠️ Tudo em espaço de VISTA, o espaço do `n_view`. A altura vem em unidades
// de OBJECTO, logo é escalada pela escala do `obj.model` (a pose é uniforme).
//
// ⛔ As derivadas são chamadas aqui, fora de qualquer ramo divergente: o
// `tinta_no_ponto4` ramifica por `topo` (o valor que ele devolve pode vir de
// um ramo, a CHAMADA de `dpdx` não pode), e os dois `if` de cima leem um
// uniforme.
//
// ⭐⭐ **A inclinação é pesada pelo CORPO** (`docs/3D/29` §6) — a lei do passe
// de luz 2D do Painter (`impasto_light::paint_body`: *relevo sob cobertura
// zero não acende*). Sem ela a encosta que o alisamento do impasto espalha
// para fora da tinta acendia o barro nu: o anel do report do dono de 01/10.
// Com corpo `1` a normal é a de antes AO BIT (`1·grad` é `grad`).
fn tinta_relevo_n(in: VsOut, h: f32, corpo: f32) -> vec3<f32> {
    let escala = length(obj.model[0].xyz);
    let p = (cam.view * obj.model * vec4<f32>(in.opos, 1.0)).xyz;
    let hs = h * escala;
    let sx = dpdx(p);
    let sy = dpdy(p);
    let dhx = dpdx(hs);
    let dhy = dpdy(hs);
    let n = normalize(in.n_view);
    let r1 = cross(sy, n);
    let r2 = cross(n, sx);
    let det = dot(sx, r1);
    let grad = clamp(corpo, 0.0, 1.0) * (sign(det) * (dhx * r1 + dhy * r2));
    let nb = abs(det) * n - grad;
    let l = length(nb);
    // Um triângulo degenerado no ecrã (det = 0) não tem gradiente a ler.
    if (l <= 0.0) {
        return in.n_view;
    }
    return nb / l;
}
"#;

/// A fonte que o `create_shader_module` recebe.
///
/// `com_tinta` é [`wgpu::Features::PRIMITIVE_INDEX`] — e a decisão é do
/// chamador, que é quem tem o device.
///
/// ⭐⭐ **A base é a fonte COMPOSTA do [`crate::pbr::fonte`], nunca o
/// [`MESH_WGSL`] cru** (integração de 2026-09-25, `line/sculpt3d` +
/// `line/3DModeling`): desde o modo `Lighting::Pbr` o `mesh.wgsl` chama as
/// `mx_*` da `ph2d-material` e o tonemap da `ph2d-view-transform`, e sozinho
/// **não parsa**. As duas linhas compunham a fonte cada uma à sua maneira — a
/// tinta fina por fora, a lei de luz por dentro —, e *duas composições da
/// mesma fonte divergem no dia em que alguém mexer numa*: a tinta passa a
/// embrulhar a composição da luz, e os gates do `naga` leem esta porta.
#[must_use]
pub fn mesh_wgsl(com_tinta: bool) -> Cow<'static, str> {
    let base = crate::pbr::fonte();
    if com_tinta {
        // ⚠️ **A directiva `enable` tem de vir ANTES de toda declaração**, logo
        //   ela abre a fonte e não o bloco — foi o `naga` que o disse, em voz
        //   alta: *«the `primitive_index` enable extension is not enabled»*.
        Cow::Owned(format!(
            "enable primitive_index;\n{base}\n{TINTA_WGSL}\n{TINTA_ENTRADA_WGSL}"
        ))
    } else {
        Cow::Owned(base)
    }
}

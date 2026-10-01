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
    return fs_core_n(in, t.c.xyz, tinta_relevo_n(in, t.g, t.corpo));
}

// ⭐⭐⭐ **A NORMAL INCLINADA PELO RELEVO** — o *gradiente de superfície*
// (M. Mikkelsen, «Surface Gradient-Based Bump Mapping Framework», 2020): a
// normal da superfície `p + h·n` é `normalize(n − ∇ₛh)`, com `∇ₛh` o gradiente
// da altura projectado no plano tangente.
//
// ⭐⭐ **O gradiente é EXACTO e vem do OBJECTO** (`TintaLida::g`, report do dono
// de 01/10: *«de cima parece bom, inclinado aparece artefato de relevo»*). A
// 1.ª redacção tirava-o por diferenças de ECRÃ (`dpdx`/`dpdy`, Mikkelsen 2010),
// que são por bloco de `2×2` píxeis — de cima a encosta ocupa muitos píxeis e
// ninguém o via; inclinada ela cabe em um ou dois, e a luz acendia-se em
// tracinhos soltos ao longo da borda. ⇒ a altura é linear/bilinear em cada
// célula da retícula e a derivada dela não depende de onde se olha.
//
// ⚠️ Tudo em espaço de VISTA, o espaço do `n_view`. A altura vem em unidades
// de OBJECTO e o declive é ADIMENSIONAL, logo o gradiente só roda: a vista é
// rígida e a pose é uniforme, e `M·g` traz a escala uma vez — que se divide.
//
// ⭐⭐ **A inclinação é pesada pelo CORPO** (`docs/3D/29` §6) — a lei do passe
// de luz 2D do Painter (`impasto_light::paint_body`: *relevo sob cobertura
// zero não acende*). Sem ela a encosta que o alisamento do impasto espalha
// para fora da tinta acendia o barro nu: o anel do report do dono de 01/10.
fn tinta_relevo_n(in: VsOut, g: vec3<f32>, corpo: f32) -> vec3<f32> {
    let m = cam.view * obj.model;
    let escala = length(obj.model[0].xyz);
    let n = normalize(in.n_view);
    if (escala <= 0.0) {
        return n;
    }
    let gv = (m * vec4<f32>(g, 0.0)).xyz / escala;
    // ⚠️ A lei (gradiente de superfície + o HORIZONTE) mora no `tinta.wgsl`,
    // pura e sem recursos: é por isso que a placa a confere contra a CPU.
    return tinta_inclina(in.n_view, gv, corpo);
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

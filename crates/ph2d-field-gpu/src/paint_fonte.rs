//! ⭐⭐⭐ **A MONTAGEM DO TEXTO DO SHADER DO PINTOR** — o irmão dos três que guardam o texto.
//!
//! O [`super::paint_wgsl`], o [`super::paint_wgsl_sondas`] e o [`super::paint_wgsl_mole`] guardam o
//! corpo; este ficheiro **compõe-no** com as leis da marcha, o material, o olhar, o estilo e a lei
//! do dono, e preenche as ranhuras numéricas.
//!
//! ⚠️ **Ele saiu do [`super::paint`] por um TECTO DE LOC** (`720` contra `700`, 2026-09-22) — e a
//! fronteira que o tecto forçou é a certa: *montar um texto e despachar um passe são duas
//! responsabilidades*, e o `paint.rs` fica com a segunda. É o mesmo corte que o
//! `ph2d-app-field3d/src/device_probes.rs` pagou, e com a mesma leitura.

use super::{OwnersWgsl, PaintSetup};

/// ⛔ **A lei do dono numa peça de UMA folha** — a mesma resposta que o `Surfaces::owners: None` dá
/// na CPU, e não um caso especial: não perguntar é exactamente o custo zero.
const DONO_DE_UMA_FOLHA: &str = r"
struct Dono { a: u32, b: u32, t: f32 };
fn dono_mix(p: vec3<f32>, width: f32) -> Dono { return Dono(0u, 0u, 0.0); }
";

/// ⭐ **O corpo do shader vive no irmão** ([`super::paint_wgsl`]) — ver o cabeçalho dele.
use crate::paint_wgsl::PINTOR;
use crate::paint_wgsl_mole::PINTOR_MOLE;
use crate::paint_wgsl_sondas::PINTOR_SONDAS;

/// ⭐⭐⭐ **O texto do pintor, composto** — as quatro leis mais o corpo.
///
/// ⚠️ **A ordem é a que o WGSL precisa para o `struct Ceu` existir antes do binding que o nomeia.**
/// O material traz a ranhura do ambiente já preenchida pelo céu de quem chama.
pub(crate) fn fonte(
    pintor: &PaintSetup<'_>,
    lei_do_dono: Option<&OwnersWgsl>,
    leis: &str,
) -> String {
    let dono = lei_do_dono.map_or(DONO_DE_UMA_FOLHA, |l| l.source.as_str());
    let material = ph2d_material::wgsl::SOURCE
        .replace(ph2d_material::wgsl::ENV_SLOT, &ambiente(pintor.env_source));
    // ⚠️ **As TRÊS metades são UM shader** — o corte é o tecto de LOC, e a concatenação é onde ele
    // deixa de se ver. Ver o cabeçalho do [`crate::paint_wgsl`] e o do [`crate::paint_wgsl_mole`].
    // ⛔ Eram duas até 2026-09-19; a `W10` trouxe a terceira, e há gate a exigir que ela seja
    // JUNTA (`o_gemeo_da_borda_mole_esta_ligado_no_dispositivo`): *um fragmento declarado que o
    // `format!` não junta compila e não chega ao shader.*
    let corpo = format!("{PINTOR}{PINTOR_SONDAS}{PINTOR_MOLE}")
        .replace(
            "{BLUR_COS}",
            &formata(ph2d_field_render::OCCLUSION_BLUR_COS),
        )
        .replace(
            "{PISO_LUZ}",
            &formata(ph2d_field_render::POINT_LAMP_MIN_DISTANCE),
        )
        .replace("{PACKED}", &ph2d_material::wgsl::PACKED.to_string())
        // ⚠️ **Lida do sítio que a declara** ([`crate::paint_wgsl::CURVATURA`]), porque ela tem um
        // SEGUNDO leitor: o instrumento que mede a curvatura nos dois motores. *Uma lei com dois
        // leitores não se escreve duas vezes.*
        // ⛔⛔⛔ **DUAS funções do MESMO molde, e a razão é MEDIDA** (2026-09-19): quando o `ε`
        // passou a ser ARGUMENTO, a imagem do dispositivo divergiu da referência em **2–3 bytes
        // sobre `4 519` píxeis do MIOLO**. ⚠️ **Não era a lei nem o `ε`** — bissectado: a divergência
        // é **idêntica** com o ganho antigo e com a suavidade no piso. O que muda é o TEXTO da
        // função: a placa **contrai `a*b + c` num `fma`** de outra maneira, e a segunda diferença
        // amplifica essa última casa por `1/(4ε²)`.
        //
        // ⚠️⚠️ **A recusa que o doc daquela const escrevia tinha uma razão REAL e enunciava OUTRA:**
        // ela dizia *«mudaria o texto do produto para servir o instrumento»*, e o perigo não era o
        // instrumento — era **a aritmética da placa mudar com o texto**. ⇒ o molde dá à medição do
        // MATERIAL um corpo byte-idêntico ao de ontem (`let e = pintor.knobs.z;`) e ao ESTILO o
        // mesmo corpo com outra primeira linha. *Uma lei, dois leitores, e nenhum texto novo no
        // caminho que já shipava.*
        .replace("{CURVATURA}", &{
            let molde = |nome: &str, eps: &str| {
                crate::paint_wgsl::CURVATURA
                    .replace("{NOME}", nome)
                    .replace("{EPS}", eps)
            };
            format!(
                "{}{}",
                molde("curvatura_em", "pintor.knobs.z"),
                molde("curvatura_do_estilo_em", "pintor.estilo.extra.z"),
            )
        })
        // ⚠️ **Lido do ficheiro que o declara**, nunca transcrito — a mesma lei do `{BLUR_COS}`.
        .replace(
            "{FADE}",
            &formata(ph2d_field_render::ground_bounce::GROUND_BOUNCE_FADE),
        )
        .replace(
            "{PROBE_GRID}",
            &ph2d_field_render::probes::PROBE_GRID.to_string(),
        )
        .replace(
            "{PROBE_DIRS}",
            &ph2d_field_render::probes::PROBE_DIRS.to_string(),
        )
        .replace(
            "{PROBE_MARGIN}",
            &formata(ph2d_field_render::probes::PROBE_MARGIN),
        )
        // ⚠️ O MESMO f32 que a CPU calcula em tempo de execução — `4π/N` em `f32`.
        .replace(
            "{SH_PESO}",
            &formata(4.0 * std::f32::consts::PI / ph2d_field_render::probes::PROBE_DIRS as f32),
        )
        .replace(
            "{SH_A1}",
            &formata(ph2d_field_render::probes::SH_COSSENO[1]),
        )
        .replace(
            "{SH_A2}",
            &formata(ph2d_field_render::probes::SH_COSSENO[4]),
        )
        .replace("{SQRT3}", &formata(3.0f32.sqrt()))
        .replace("{MAX_LAMPS}", &crate::trace::MAX_LAMPS.to_string())
        .replace("{LUMA_R}", &formata(ph2d_field_render::GROUND_LUMA[0]))
        .replace("{LUMA_G}", &formata(ph2d_field_render::GROUND_LUMA[1]))
        .replace("{LUMA_B}", &formata(ph2d_field_render::GROUND_LUMA[2]))
        // ⭐ O MESMO tecto de raio que a CPU clampa — `ph2d_field_render::sss_shadow::MAX_RAIO_PX`.
        .replace(
            "{MAX_RAIO_MOLE}",
            &formata(ph2d_field_render::sss_shadow::MAX_RAIO_PX),
        );
    // ⚠️⚠️ **As LEIS da marcha entram aqui desde o ricochete** (`docs/Render3d/08` §12): ele marcha
    // a partir da superfície, logo o passe que PINTA precisa do campo, da marcha e da
    // visibilidade. ⛔ **E só as leis** — os dois kernels da marcha ficam de fora, senão este
    // módulo teria pontos de entrada que ninguém despacha.
    // ⭐⭐⭐ **A camada de ESTILO entra ANTES do corpo, e a ordem é load-bearing:** com a
    // `struct Estilo` já declarada, o `Pintor` pode tê-la como CAMPO — e a arrumação dos vinte
    // números deixa de estar escrita uma segunda vez neste ficheiro. *Um adaptador que copiasse
    // `array[0] → rim`, `array[1] → convexo` … seria a segunda resposta à tabela do
    // `ph2d_style::wgsl::pack`, e a que envelhece.*
    format!(
        "{}{leis}{material}\n{}\n{}\n{dono}\n{corpo}",
        crate::trace_wgsl::comum(),
        ph2d_view_transform::wgsl::SOURCE,
        ph2d_style::wgsl::source()
    )
}

/// ⭐⭐⭐ **O AMBIENTE QUE O MATERIAL VÊ — o céu de quem chama, OU o ricochete.**
///
/// A lei indirecta do OpenPBR pergunta duas coisas ao ambiente (`env_radiance` e `env_irradiance`)
/// e é **linear** nas duas: o termo difuso entra pela irradiância, o especular pela radiância, e o
/// empilhamento é linear nas respostas. ⇒ *chamar a lei duas vezes com dois ambientes é somar os
/// dois*, que é exactamente o que o [`ph2d_field_render::shade_render`] faz com o `SoIrradiancia`.
///
/// ⚠️⚠️ **E tinham de ser DUAS chamadas e não um ambiente somado**, porque a parcela do céu leva a
/// oclusão por cima (`* ceu_vis`) e a do ricochete não: *a oclusão é a sombra do CÉU, e a luz que
/// vem das superfícies não é céu.*
///
/// ⛔ **É por isso que o [`PaintSetup::env_source`] declara `ceu_*` e não `env_*`:** quem manda no
/// ambiente é este despacho, e o céu de quem chama é um dos dois braços dele. Com o
/// `ambiente_e_ricochete` a `false` — o valor de nascença — o texto gerado entrega o céu **ao
/// bit**.
fn ambiente(ceu: &str) -> String {
    format!(
        r"{ceu}
// ⭐ O estado deste pixel, por invocação: o WGSL dá `var<private>`, que é exactamente isso.
var<private> ricochete: vec3<f32> = vec3<f32>(0.0);
var<private> ambiente_e_ricochete: bool = false;

fn env_radiance(dir: vec3<f32>, alpha: f32, shrink: f32) -> vec3<f32> {{
    // ⚠️ **O ricochete NÃO tem direcção**: ele é uma irradiância por pixel, e o lóbulo especular
    // pergunta *«que luz vem DAQUELA direcção»*. A resposta honesta é zero — a mesma que o
    // `SoIrradiancia` da CPU dá.
    if (ambiente_e_ricochete) {{ return vec3<f32>(0.0); }}
    return ceu_radiance(dir, alpha, shrink);
}}

fn env_irradiance(n: vec3<f32>) -> vec3<f32> {{
    if (ambiente_e_ricochete) {{ return ricochete; }}
    return ceu_irradiance(n);
}}
"
    )
}

/// ⚠️ Um `f32` que o WGSL leia como `f32` — sem isto um `0.9` inteiro sairia `0.9` e um `2` sairia
/// `2`, que ali é um literal **inteiro**.
fn formata(v: f32) -> String {
    let s = format!("{v:?}");
    if s.contains('.') || s.contains('e') {
        s
    } else {
        format!("{s}.0")
    }
}

//! ⭐⭐⭐ **O REALCE DESLIGADO TEM DE ESTAR DESLIGADO NOS DOIS CAMINHOS** — a segunda lei desta crate
//! que DIVERGE da referência de propósito, e a primeira em que a divergência é uma CORRECÇÃO.
//!
//! # ⛔⛔⛔ O report do dono, e o que ele fotografou
//!
//! 2026-09-19: *«temos um tipo de rim sem que o rim esteja ligado. Isso é o normal? Veja que no
//! blender não há isso»* — uma banda clara ao longo da silhueta, na **barra escura** da cena `=36`.
//! Aquela barra é autorada com **`specular_weight: 0`**, e o comentário que a escreve diz porquê:
//! *«um brilho especular por cima confunde o que é halo com o que é reflexo»*.
//!
//! # ⭐⭐⭐ A causa, medida: o MESMO material devolve `0,0000` a uma LÂMPADA e `0,41` ao CÉU
//!
//! O `specular_weight` não multiplica o lobo: ele **modula o índice de refracção**
//! (`modulated_eta_s` no grafo do OpenPBR), e a zero o índice fica exactamente `1` — um interface
//! que **não existe**. A partir daí os dois caminhos da mesma superfície respondem coisas opostas:
//!
//! | caminho | o que ele usa | `η = 1`, `n·v = 0,405` |
//! |---|---|---:|
//! | LUZ directa | `fresnel_dielectric(v·h, η)`, a de Fresnel **exacta** | **`0,0000`** |
//! | CÉU indirecto | `ggx_dir_albedo(n·v, α, F0, F90)` com **`F90 = 1`** | **`0,0702`** |
//!
//! e no rasante o segundo vai a **`0,41`**. *Uma superfície que não reflecte uma lâmpada não pode
//! reflectir o céu.*
//!
//! # ⚠️ A porta é FIEL — o defeito é da lei de origem, e a divergência é DECLARADA
//!
//! Corrido o oráculo (MaterialX 1.39.5, Apache-2.0, instalado nesta máquina), o
//! `mx_dielectric_bsdf.glsl` escreve `mx_ggx_dir_albedo(NdotV, avgAlpha, F0, 1.0)` nos **dois**
//! ramos, e o `mx_ggx_dir_albedo(..., FresnelData)` do `mx_environment_prefilter` escreve
//! `mx_ggx_dir_albedo(NdotV, alpha, vec3(F0), vec3(1.0))` para o modelo dieléctrico. ⇒ a nossa porta
//! está linha a linha certa e **o `F90 = 1` é uma constante escrita à mão lá**.
//!
//! ⭐ O valor rasante de um dieléctrico é `1` para todo interface e **`0` quando não há interface**
//! ([`crate::bsdf::grazing_dielectric`]) — logo a cura é ler o `F90` do próprio `η` em vez de o
//! cravar. **Todo material com um índice a sério continua BYTE A BYTE o mesmo** (para `η ≠ 1` a
//! função devolve `1,0` exactamente), e só o realce desligado muda.

use crate::{Environment, OpenPbr};

/// Um céu de radiância `1` e irradiância `0`: o que sai é o **albedo direccional** e mais nada.
struct SoEspelho;

impl Environment for SoEspelho {
    fn radiance(&self, _dir: [f32; 3], _alpha: f32) -> [f32; 3] {
        [1.0; 3]
    }
    fn irradiance(&self, _n: [f32; 3]) -> [f32; 3] {
        [0.0; 3]
    }
}

/// A normal inclinada de `n·v` contra um observador em `+z`.
fn normal_a(ndv: f32) -> [f32; 3] {
    [(1.0 - ndv * ndv).max(0.0).sqrt(), 0.0, ndv]
}

/// A barra escura da cena `=36`, que é o sujeito do report.
fn barra(specular_weight: f32) -> crate::Surface {
    OpenPbr {
        base_color: [0.055, 0.055, 0.065],
        specular_weight,
        ..OpenPbr::default()
    }
    .prepare()
}

/// ⭐⭐⭐ **COM O REALCE A ZERO O CÉU NÃO SE REFLECTE, EM ÂNGULO NENHUM** — e o CONTROLO é o mesmo
/// material com o realce ligado, que continua a reflectir.
///
/// ⚠️ **As duas metades são obrigatórias.** Sem a primeira a lei não é afirmada; sem a segunda a
/// cura barata (zerar o lobo indirecto) ficava verde e apagava o reflexo de **toda** superfície.
#[test]
fn com_o_realce_a_zero_o_ceu_nao_se_reflecte() {
    let v = [0.0, 0.0, 1.0];
    let desligado = barra(0.0);
    let ligado = barra(1.0);
    let mut pior_desligado = 0.0f32;
    let mut menor_ligado = f32::INFINITY;
    // ⚠️ A varredura vai até ao RASANTE de propósito: é lá que o `F90` manda, e é lá que a banda
    // do report vive. O `n·v = 0,405` é o pixel que o dono fotografou.
    for k in 0..=100 {
        #[allow(clippy::cast_precision_loss)]
        let ndv = 1.0 - k as f32 / 100.0;
        let n = normal_a(ndv.max(1.0e-4));
        let d = desligado.indirect(n, v, &SoEspelho);
        let l = ligado.indirect(n, v, &SoEspelho);
        pior_desligado = pior_desligado.max(d[0]).max(d[1]).max(d[2]);
        menor_ligado = menor_ligado.min(l[0]);
    }
    assert!(
        pior_desligado <= 1.0e-6,
        "com `specular_weight = 0` o céu ainda se reflecte: pior {pior_desligado:.4} em 101 ângulos \
         — é o rim que o dono fotografou numa barra cujo realce está DESLIGADO"
    );
    // ⭐ O CONTROLO: o mesmo material com o realce ligado reflecte, e reflecte MUITO no rasante.
    assert!(
        menor_ligado > 1.0e-3,
        "o CONTROLO ficou mudo (menor {menor_ligado:.6}): com o realce LIGADO o céu tem de se \
         reflectir, senão a metade de cima passa por vácuo"
    );
}

/// ⭐⭐⭐ **OS DOIS CAMINHOS DO MESMO MATERIAL CONCORDAM SOBRE SE HÁ REFLEXO** — a lâmpada e o céu.
///
/// ⚠️ **A régua é a PRESENÇA e não o valor:** uma lâmpada é uma radiância numa direcção e o céu é
/// uma radiância em todas, logo os dois números não são comparáveis. O que tem de concordar é
/// *«esta superfície reflecte alguma coisa neste ângulo?»* — e era exactamente aí que eles
/// discordavam.
///
/// ⚠️⚠️ **O `base_weight` vai a ZERO, e a 1.ª redacção deste gate não o fazia:** a
/// [`crate::Surface::direct`] devolve o difuso **e** o especular numa chamada só, logo com a luz na
/// direcção espelhada e `n·v = 1` ela lia `0,017507` — que é `0,055/π`, o DIFUSO da barra, acusado
/// como reflexo. *Uma régua que soma dois lobos não pode afirmar que um deles está desligado.*
#[test]
fn a_lampada_e_o_ceu_concordam_sobre_haver_reflexo() {
    /// Só os lobos de realce: sem difuso, o que sai de qualquer dos caminhos é o reflexo e nada mais.
    fn so_realce(specular_weight: f32) -> crate::Surface {
        OpenPbr {
            base_weight: 0.0,
            specular_weight,
            ..OpenPbr::default()
        }
        .prepare()
    }
    let v = [0.0, 0.0, 1.0];
    let desligado = so_realce(0.0);
    let ligado = so_realce(1.0);
    let (mut pior_lampada, mut pior_ceu) = (0.0f32, 0.0f32);
    let (mut controlo_lampada, mut controlo_ceu) = (0.0f32, 0.0f32);
    for k in 0..=20 {
        #[allow(clippy::cast_precision_loss)]
        let ndv = (1.0 - k as f32 / 20.0).max(1.0e-4);
        let n = normal_a(ndv);
        // A luz na direcção ESPELHADA, que é onde o especular é máximo.
        let d = 2.0 * (n[0] * v[0] + n[1] * v[1] + n[2] * v[2]);
        let l = [d * n[0] - v[0], d * n[1] - v[1], d * n[2] - v[2]];
        pior_lampada = pior_lampada.max(desligado.direct(n, v, l, [1.0; 3])[0]);
        pior_ceu = pior_ceu.max(desligado.indirect(n, v, &SoEspelho)[0]);
        controlo_lampada = controlo_lampada.max(ligado.direct(n, v, l, [1.0; 3])[0]);
        controlo_ceu = controlo_ceu.max(ligado.indirect(n, v, &SoEspelho)[0]);
    }
    assert!(
        pior_lampada <= 1.0e-6 && pior_ceu <= 1.0e-6,
        "com o realce desligado a LÂMPADA devolve {pior_lampada:.6} e o CÉU devolve {pior_ceu:.6} — \
         a mesma superfície, os mesmos ângulos, duas respostas"
    );
    // ⭐ O CONTROLO: com o realce LIGADO os dois caminhos reflectem, senão a metade de cima passa
    // por vácuo sobre um material que não existe.
    assert!(
        controlo_lampada > 1.0e-3 && controlo_ceu > 1.0e-3,
        "o CONTROLO ficou mudo: lâmpada {controlo_lampada:.6}, céu {controlo_ceu:.6}"
    );
}

/// ⭐⭐⭐ **UM ÍNDICE A SÉRIO CONTINUA A TER RASANTE `1`, AO BIT** — a cerca que torna a cura
/// surgical.
///
/// ⛔ Sem ela, esta wave seria uma mudança de aparência em todo material do produto. A tabela
/// abaixo é a razão de o gate varrer a faixa inteira em vez de um valor: o `specular_weight` entra
/// pelo **índice**, e qualquer valor acima de zero produz um interface.
#[test]
fn todo_indice_a_serio_mantem_o_rasante_em_um() {
    #[allow(clippy::float_cmp)] // a promessa É a igualdade exacta
    for ior in [1.0001f32, 1.05, 1.2, 1.5, 1.6, 2.0, 2.4, 0.75, 0.5] {
        assert_eq!(
            crate::bsdf::grazing_dielectric(ior),
            1.0,
            "η = {ior} tem interface, logo o rasante é 1 — mudá-lo mexeria em todo material do \
             produto"
        );
    }
    #[allow(clippy::float_cmp)]
    {
        assert_eq!(
            crate::bsdf::grazing_dielectric(1.0),
            0.0,
            "η = 1 é a ausência de interface: o rasante tem de ser 0"
        );
    }
}

/// ⭐⭐⭐ **O REALCE DESLIGADO DEVOLVE O DIFUSO INTEIRO** — a outra metade do mesmo defeito, e ela
/// não estava no report.
///
/// O `throughput` de um lobo é *«o que ele deixa passar para o substrato»*. Com `F90 = 1` o lobo
/// desligado **comia** até `41 %` do difuso no rasante, sem nunca devolver nada em troca — energia
/// destruída por uma camada que não existe.
#[test]
fn o_realce_desligado_deixa_passar_o_difuso_inteiro() {
    struct SoIrradiancia;
    impl Environment for SoIrradiancia {
        fn radiance(&self, _dir: [f32; 3], _alpha: f32) -> [f32; 3] {
            [0.0; 3]
        }
        fn irradiance(&self, _n: [f32; 3]) -> [f32; 3] {
            [1.0; 3]
        }
    }
    let v = [0.0, 0.0, 1.0];
    let desligado = barra(0.0);
    // ⚠️⚠️ **AS DUAS METADES, e a segunda nasceu de uma MUTAÇÃO SOBREVIVENTE:** o `throughput` do
    // céu vive na [`crate::indirect::dielectric`] e o da lâmpada na
    // [`crate::bsdf::dielectric_reflection`] — são dois sítios, e uma régua que só use o céu deixa o
    // segundo sem ninguém a olhar. *Repor o `F90 = 1` lá ficava verde.*
    //
    // A lei difusa de omissão é lambertiana (`base_diffuse_roughness = 0`), logo com a luz ao longo
    // da NORMAL o difuso não depende do observador: tudo o que a razão abaixo mede é a camada.
    for (nome, frontal, rasante) in [
        (
            "o CÉU",
            desligado.indirect(normal_a(1.0), v, &SoIrradiancia)[0],
            desligado.indirect(normal_a(0.05), v, &SoIrradiancia)[0],
        ),
        (
            "a LÂMPADA",
            desligado.direct(normal_a(1.0), v, normal_a(1.0), [1.0; 3])[0],
            desligado.direct(normal_a(0.05), v, normal_a(0.05), [1.0; 3])[0],
        ),
    ] {
        assert!(
            frontal > 0.0 && rasante > 0.0,
            "{nome}: a fixtura ficou muda (frontal {frontal:.6}, rasante {rasante:.6})"
        );
        // Com o lobo a comer energia, o rasante lia `0,63×` o frontal; sem ele, a lei difusa sozinha.
        let razao = rasante / frontal;
        assert!(
            razao > 0.99,
            "{nome}: o difuso rasante é {razao:.4}× o frontal — a camada DESLIGADA ainda come \
             {:.1} % da luz que ela nunca devolve",
            (1.0 - razao) * 100.0
        );
    }
}

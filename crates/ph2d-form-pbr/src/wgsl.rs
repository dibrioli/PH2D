//! **O GÉMEO EM WGSL** do laço da [`super::acende_texel`].
//!
//! ⛔⛔ **Ele NÃO contém uma linha de óptica.** A lei do OpenPBR entra por
//! [`ph2d_material::wgsl::SOURCE`], que é o gerado pelo próprio MaterialX, e o que este ficheiro
//! acrescenta é **só o laço** — as mesmas quatro decisões do lado da CPU: normalizar a normal,
//! somar as lâmpadas, pesar o ambiente pela oclusão e misturar pela cobertura.
//!
//! ⚠️ **Uma segunda redacção da óptica aqui seria a divergência que a `ph2d-material` existe para
//! impedir**, e ela não apareceria num teste de valor — apareceria num screenshot, meses depois.
//!
//! # Como se monta
//!
//! ```ignore
//! let fonte = format!("{}\n{}", ph2d_material::wgsl::SOURCE, ph2d_form_pbr::wgsl::SOURCE);
//! ```
//!
//! ⚠️ **Por esta ordem**: o nosso laço chama `mx_direct`, que vem de lá.

/// Quantos `f32` uma [`super::Lampada`] ocupa no buffer: `dir.xyz` + `_pad` + `rgb` + `_pad`.
///
/// ⚠️ **`8` e não `6`** — um `vec3<f32>` num array de storage alinha a `16` bytes em WGSL, e
/// empacotar `6` faria a segunda lâmpada ser lida do meio da primeira. *O alinhamento é uma
/// propriedade da linguagem, não uma escolha nossa.*
pub const LAMPADA_FLOATS: usize = 8;

/// Empacota as lâmpadas para o buffer que o [`SOURCE`] lê.
///
/// ⚠️ A ordem é a mesma do `SOURCE`, e há gate a atá-las: *duas respostas à mesma pergunta
/// divergem no dia em que uma mudar.*
#[must_use]
pub fn pack_lampadas(lampadas: &[super::Lampada]) -> Vec<f32> {
    let mut v = Vec::with_capacity(lampadas.len() * LAMPADA_FLOATS);
    for l in lampadas {
        v.extend_from_slice(&[
            l.para_a_luz[0],
            l.para_a_luz[1],
            l.para_a_luz[2],
            0.0,
            l.radiancia[0],
            l.radiancia[1],
            l.radiancia[2],
            0.0,
        ]);
    }
    v
}

/// O laço, em WGSL. Ver o cabeçalho do módulo.
///
/// ⚠️ **A `VISTA` é escrita aqui como literal e há gate a prendê-la à [`super::VISTA`]** — uma
/// constante transcrita é uma divergência à espera de um dia em que alguém mexa na outra.
pub const SOURCE: &str = r#"
// ⛔ Este bloco NÃO tem óptica: ele chama `mx_direct`, que vem do SOURCE da `ph2d-material`.

struct LampadaGpu {
    para_a_luz: vec3<f32>,
    _pad0: f32,
    radiancia: vec3<f32>,
    _pad1: f32,
};

// A vista de um canvas 2D. MUST equal `ph2d_form_pbr::VISTA` — preso por
// `a_vista_do_shader_e_a_da_lei`.
const FORMA_VISTA: vec3<f32> = vec3<f32>(0.0, 0.0, 1.0);

// O quadrado do comprimento abaixo do qual uma normal não tem direcção. MUST equal o `1e-12` da
// `ph2d_form_pbr::normaliza` — preso pelo mesmo gate.
const FORMA_EPS_N: f32 = 1e-12;

// ⭐⭐⭐ O LAÇO. `m` é o material já empacotado (`ph2d_material::wgsl::pack`).
fn forma_acende_texel(
    m: Mat,
    normal: vec3<f32>,
    albedo: vec3<f32>,
    cobertura: f32,
    oclusao: f32,
    lampadas: ptr<storage, array<LampadaGpu>, read>,
    n_lampadas: u32,
    ambiente: vec3<f32>,
) -> vec3<f32> {
    let q = dot(normal, normal);
    // ⛔ Fora da silhueta devolve o albedo CRU, nunca preto — ver o doc da `acende_texel`.
    if (q < FORMA_EPS_N) {
        return albedo;
    }
    let n = normal * inverseSqrt(q);

    var luz = vec3<f32>(0.0);
    for (var i = 0u; i < n_lampadas; i = i + 1u) {
        let l = (*lampadas)[i];
        luz = luz + mx_direct(m, n, FORMA_VISTA, l.para_a_luz, l.radiancia);
    }

    // ⚠️ A oclusão pesa SÓ o ambiente. Ver o doc da `acende_texel`.
    let aceso = albedo * (luz + ambiente * oclusao);

    let c = clamp(cobertura, 0.0, 1.0);
    return albedo + (aceso - albedo) * c;
}
"#;

#[cfg(test)]
mod tests {
    /// ⭐ **As constantes do shader são as da lei.** Um `const` transcrito diverge em silêncio;
    /// este gate lê as DUAS do sítio que as declara.
    ///
    /// ⚠️ A agulha é montada em runtime e não escrita como literal: *um censo textual que se lê a
    /// si mesmo encontra sempre o que procura* — é a lição que a `ph2d-style` pagou.
    #[test]
    fn a_vista_do_shader_e_a_da_lei() {
        let v = super::super::VISTA;
        let agulha = format!("{}({:.1}, {:.1}, {:.1})", "vec3<f32>", v[0], v[1], v[2]);
        assert!(
            super::SOURCE.contains(&agulha),
            "o shader tem de declarar a VISTA da lei ({agulha})"
        );
        let eps = format!("{} = {}", "FORMA_EPS_N: f32", "1e-12");
        assert!(
            super::SOURCE.contains(&eps),
            "o shader tem de declarar o mesmo limiar de normal degenerada"
        );
    }

    /// ⚠️ **O empacotamento tem de caber no alinhamento do WGSL.** Ver o doc de
    /// [`super::LAMPADA_FLOATS`].
    #[test]
    fn o_empacotamento_respeita_o_alinhamento() {
        let l = super::super::Lampada {
            para_a_luz: [1.0, 2.0, 3.0],
            radiancia: [4.0, 5.0, 6.0],
        };
        let v = super::pack_lampadas(&[l, l]);
        assert_eq!(v.len(), 2 * super::LAMPADA_FLOATS);
        // A segunda lâmpada começa exactamente no `LAMPADA_FLOATS`-ésimo float.
        assert_eq!(
            v[super::LAMPADA_FLOATS],
            1.0,
            "a 2.ª lâmpada tem de começar alinhada"
        );
        assert_eq!(v[super::LAMPADA_FLOATS + 4], 4.0);
    }
}

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
//! ⚠️⚠️ **NÃO é uma concatenação, e foi o gate de `naga` que o descobriu** — a redacção anterior
//! deste bloco dizia que era, e estava errada nas duas metades: a fonte da lei traz um `{ENV}` por
//! preencher, e a nossa traz um `{MAX_LAMPADAS}`.
//!
//! ```ignore
//! let fonte = format!(
//!     "{}\n{}",
//!     ph2d_material::wgsl::SOURCE.replace(ph2d_material::wgsl::ENV_SLOT, o_ambiente_do_consumidor),
//!     ph2d_form_pbr::wgsl::SOURCE.replace(ph2d_form_pbr::wgsl::CAP_SLOT, "4u"),
//! );
//! ```
//!
//! ⚠️ **Por esta ordem**: o nosso laço chama `mx_direct`, que vem de lá.
//!
//! # ⏳ O que os gates desta crate NÃO afirmam, e é dívida NOMEADA
//!
//! Eles medem a **ESTRUTURA** do gémeo — que ele parsa, que valida, que chama a óptica de lá em vez
//! de a reescrever, e que as duas constantes são as da lei. ⛔ **Eles não medem um VALOR**: nada
//! aqui prova que o dispositivo calcula o mesmo número que a [`super::acende_texel`].
//!
//! ⚠️ E isso **não** se cura nesta crate: medir um valor pede um `Device`, e uma folha que ganhasse
//! `wgpu` deixava de ser folha. *A paridade é do PASSE* — ela nasce com o consumidor, com a barra
//! derivada do formato do alvo, como a `ph2d-flip` já faz. Até lá a promessa desta crate é
//! exactamente a que os gates escrevem, e nem uma linha a mais.

/// Quantos `f32` uma [`super::Lampada`] ocupa no buffer: `dir.xyz` + `_pad` + `rgb` + `_pad`.
///
/// ⚠️ **`8` e não `6`** — um `vec3<f32>` num array de storage alinha a `16` bytes em WGSL, e
/// empacotar `6` faria a segunda lâmpada ser lida do meio da primeira. *O alinhamento é uma
/// propriedade da linguagem, não uma escolha nossa.*
pub const LAMPADA_FLOATS: usize = 8;

/// ⛔⛔ **Onde o TECTO de lâmpadas entra no [`SOURCE`] — e porque ele NÃO é um número desta crate.**
///
/// O WGSL não tem array de tamanho dinâmico **por valor**, e a alternativa — receber um
/// `ptr<storage, …>` — **não é WGSL do núcleo**: a `naga` recusa-a com `InvalidArgumentPointerSpace`,
/// e foi assim que este tecto apareceu (o gate `o_gemeo_em_wgsl_parsa_e_valida` reprovou a 1.ª
/// redacção deste ficheiro, antes de haver um pixel).
///
/// ⚠️ **Quem sabe o número é o RIG, e esta crate não depende dele de propósito** — logo escrevê-lo
/// aqui seria a segunda cópia de um valor que vive noutro sítio, e as duas divergiriam na primeira
/// lâmpada nova. ⇒ ele entra como MARCA, que é o idioma que o [`ph2d_material::wgsl::ENV_SLOT`] já
/// paga para a mesma classe de pergunta: *o que o consumidor sabe, o consumidor escreve.*
///
/// O que substituir esta marca tem de ser um literal `u32` de WGSL (`"4u"`), e tem de ser **≥ o
/// número de lâmpadas que o buffer dele carrega**.
pub const CAP_SLOT: &str = "{MAX_LAMPADAS}";

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

// ⛔ As lâmpadas entram por VALOR e a contagem viaja COM elas: um `n` passado ao lado seria um
// segundo argumento que se pode esquecer, e o laço leria lixo do resto do array.
struct Lampadas {
    n: u32,
    l: array<LampadaGpu, {MAX_LAMPADAS}>,
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
    lampadas: Lampadas,
    ambiente: vec3<f32>,
) -> vec3<f32> {
    let q = dot(normal, normal);
    // ⛔ Fora da silhueta devolve o albedo CRU, nunca preto — ver o doc da `acende_texel`.
    if (q < FORMA_EPS_N) {
        return albedo;
    }
    let n = normal * inverseSqrt(q);

    var luz = vec3<f32>(0.0);
    for (var i = 0u; i < lampadas.n; i = i + 1u) {
        let l = lampadas.l[i];
        // ⛔ A cerca do meio-vector degenerado — ver o doc da `acende_texel`. Sem ela uma lâmpada
        // apontada de frente para trás pinta a peça INTEIRA de `NaN`.
        let h = FORMA_VISTA + l.para_a_luz;
        if (dot(h, h) >= FORMA_EPS_N) {
            luz = luz + mx_direct(m, n, FORMA_VISTA, l.para_a_luz, l.radiancia);
        }
    }

    // ⚠️ A oclusão pesa SÓ o ambiente. Ver o doc da `acende_texel`.
    let aceso = albedo * (luz + ambiente * oclusao);

    let c = clamp(cobertura, 0.0, 1.0);
    return albedo + (aceso - albedo) * c;
}
"#;

#[cfg(test)]
#[path = "wgsl_gate_tests.rs"]
mod gate_tests;

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

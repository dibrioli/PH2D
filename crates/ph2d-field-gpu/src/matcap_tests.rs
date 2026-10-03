//! Os gates do passe de matcap — as quatro propriedades que o cabeçalho do
//! [`super::matcap`] e o do [`crate::matcap_wgsl`] afirmam.

use super::{MatcapSetup, arruma, fonte};

fn um_setup<'a>(rgb: &'a [f32], side: u32) -> MatcapSetup<'a> {
    MatcapSetup {
        rgb_linear: rgb,
        side,
        chave: 7,
        stops: 0.0,
        view: 0,
        background: [10, 20, 30, 40],
        entrega: None,
    }
}

/// ⭐⭐⭐⭐ **O TEXTO DESTE PASSE NÃO CONTÉM A RANHURA DO CAMPO** — e é essa ausência que faz o
/// cache de pipelines acertar para sempre.
///
/// # ⚠️ Porque isto é um gate e não um comentário
///
/// A [`super::pinta`] **passa** a fita real à porta do cache, e ela chama
/// `molde.replace(FIELD_SLOT, …)`. Enquanto o texto não tiver a ranhura, o `replace` é um no-op e
/// a chave do cache — que é o TEXTO — **não muda quando o artista acrescenta uma forma**.
///
/// ⛔ No dia em que alguém escrever `{FIELD}` aqui (por exemplo para medir curvatura no matcap),
/// aquela propriedade morre **em silêncio**: nada estoura, e o modo de OMISSÃO do modelador passa
/// a recompilar um pipeline a cada edição da peça — o defeito de `1 406 ms` que o pintor de material
/// do Render traçado mediu (`le_o_campo`), no caminho que o artista toma sempre.
#[test]
fn o_passe_do_matcap_nao_le_o_campo_da_peca() {
    let t = fonte();
    assert!(
        !t.contains(crate::FIELD_SLOT),
        "o texto do matcap ganhou a ranhura do campo: o cache de pipelines passa a errar a cada \
         edição da peça, e o caminho de OMISSÃO do modelador é o que paga"
    );
    // ⚠️ **O CONTROLO**: sem ele este gate passaria sobre um texto vazio ou sobre a ranhura
    // renomeada. *Uma asserção de ausência precisa de provar que o sujeito existe.*
    assert!(
        t.contains("fn pinta_matcap(") && t.contains("fn pinta_matcap_bordas("),
        "o texto composto tem de conter os DOIS pontos de entrada que a `pinta` despacha"
    );
    // ⚠️⚠️ **O CONTROLO aponta para as LEIS DA MARCHA**, que é onde a ranhura vive.
    //
    // ⭐ É isso que diz porque este passe não a tem — ele compõe o
    // [`crate::trace_wgsl::comum`] (as declarações do grupo `0`) e **nunca as leis**, logo o
    // shader do matcap não tem uma chamada a `field()` para alimentar.
    assert!(
        crate::trace_wgsl::LEIS.contains(crate::FIELD_SLOT),
        "controlo: as LEIS da marcha têm a ranhura — se a perdessem, este gate deixaria de \
         distinguir os dois passes"
    );
    assert!(
        !fonte().contains("field("),
        "o texto do matcap ganhou uma chamada ao campo da peça — ver a nota acima: ele compõe o \
         `comum()` e não as leis, e é essa ausência que o mantém fora da fita"
    );
}

/// ⭐⭐⭐⭐ **OS ARMAZÉNS SÃO CONTADOS, E ESTE PASSE CABE NO PISO DO WEBGPU.**
///
/// # ⛔⛔⛔ A história deste gate em três formas, e porque a terceira é a certa
///
/// 1. **um `#[test]` sobre dois literais** — o clippy apanhou-o (`assertions_on_constants`): *um
///    `assert!` sobre duas constantes é dobrado pelo compilador*, logo ele nunca podia reprovar
///    numa árvore que compila;
/// 2. **um `const _: () = assert!(…)`** — melhor, porque falha a compilar… e ainda media o
///    **literal**;
/// 3. ⭐ **este**, que mede as **LISTAS DE LIGAÇÃO** que os passes de facto ligam.
///
/// ⚠️⚠️ **A forma 2 nunca teria apanhado o defeito real:** o `paint::armazens` do pintor de
/// material (retirado em 03/10) era o literal `9` e o passe ligava **`12`**. *Uma asserção sobre um
/// número escrito à mão confirma o que alguém escreveu, nunca o que o código faz.*
#[test]
fn os_armazens_sao_contados_e_cabem_no_piso() {
    let meu = super::armazens();
    let piso = super::PISO_DO_WEBGPU;
    assert!(
        meu <= piso,
        "o passe do matcap liga {meu} armazéns contra o piso de {piso}: o modo de OMISSÃO do \
         modelador deixa de correr na placa em toda placa conforme"
    );
    // ⚠️ **O CONTROLO da própria régua:** uma contagem que devolvesse `0` passaria a 1.ª metade.
    assert_eq!(
        crate::trace::conta_armazens(&crate::trace::entradas_da_marcha()),
        5,
        "o grupo `0` da marcha liga cinco armazéns — se esse número mudar, o passe do matcap muda"
    );
    eprintln!("[armazéns] matcap {meu} · piso {piso}");
}

/// ⭐⭐ **A ARRUMAÇÃO DO UNIFORME É A ORDEM QUE O SHADER LÊ** — `48` bytes, três `vec4`.
///
/// ⛔ **Um uniforme lido com a compensação errada não estoura: PINTA.**
#[test]
fn a_arrumacao_do_uniforme_tem_a_ordem_que_o_shader_le() {
    let mc = MatcapSetup {
        stops: 1.5,
        view: 1,
        side: 749,
        ..um_setup(&[], 749)
    };
    let u = arruma(&mc, 123);
    assert_eq!(u.len(), 48, "três `vec4` — o `struct Mat` do shader");
    let f = |i: usize| f32::from_le_bytes([u[i], u[i + 1], u[i + 2], u[i + 3]]);
    let w = |i: usize| u32::from_le_bytes([u[i], u[i + 1], u[i + 2], u[i + 3]]);
    assert!((f(0) - 1.5).abs() < 1e-9, "`olhar.x` é a exposição");
    for k in 1..4 {
        assert_eq!(f(k * 4), 0.0, "a reserva do `olhar` fica a ZERO");
    }
    // ⚠️ **O fundo é LINEAR e PRÉ-MULTIPLICADO** — é ele que entra na média de um pixel de borda.
    let a = 40.0 / 255.0;
    assert!(
        (f(16) - ph2d_color::srgb::srgb_to_linear_byte(10) * a).abs() < 1e-9,
        "`fundo_lin.x` é linear e pré-multiplicado"
    );
    assert!((f(28) - a).abs() < 1e-9, "`fundo_lin.w` é o alfa");
    assert_eq!(w(32), 1, "`modo.x` é o código da vista");
    assert_eq!(w(36), 123, "`modo.y` é a contagem de bordas");
    // ⚠️ Os bytes EXACTOS do fundo, para um pixel de fundo puro ser COPIADO.
    assert_eq!(w(40), 10 | (20 << 8) | (30 << 16) | (40 << 24));
    assert_eq!(w(44), 749, "`modo.w` é o lado do matcap");
}

//! **EM TORNO DE QUÊ** — o vocabulário do pivô, numa porta só (ciclo 3, W1 — doc 106 §2.3).
//!
//! ## O achado que este módulo existe para fechar
//!
//! A auditoria do grupo dos transformes e deformadores mediu **seis** respostas diferentes à
//! MESMA pergunta, lidas do `MANIFEST` e do `register_reduces` de cada crate:
//!
//! | como o nó respondia | nós |
//! |---|---|
//! | `pivot_x`/`pivot_y`, coordenada **absoluta de mundo** | `bend` · `twist` · `kaleidoscope` |
//! | **centroide + offset** (o offset é relativo, com cerca escrita) | `spherize` |
//! | a **bbox**, por quatro reduções, sem controlo nenhum | `four_point_warp` |
//! | a **bbox**, calculada no `eval`, sem reduções | `bezier_warp` |
//! | um **enum** `pivot_mode` | `transform` |
//! | a **linha do centroide** do que entrou, mais um `offset` escalar | `mirror` |
//! | **nada** — a origem do mundo, sempre | `move` · `rotate` · `scale` · `look_at` |
//!
//! ⚠️ *Uma lei escrita em seis sítios não é uma lei.* O artista que aprende o `pivot_x` do
//! `motion.twist` não sabe usar o `offset_x` do `motion.spherize`, e nenhuma das seis se
//! converte nas outras.
//!
//! ## E as TRÊS formas de toda a indústria, nenhuma delas a nossa
//!
//! | referência | como responde |
//! |---|---|
//! | Blender *Simple Deform* (Bend/Twist/Taper/Stretch) | um **objecto** `Origin` que se aponta |
//! | Blender GN *Rotate/Scale Instances* | sockets `Pivot Point` / `Center` (Vector) |
//! | After Effects *Transform* / *Twirl* / *Bulge* | **Anchor Point** / `…Center` — dois números |
//! | Illustrator / Figma | o **widget 3×3** das nove âncoras da bbox |
//! | Cinema 4D *Fields* | *Fit to Parent* — um botão que **CONGELA** a bbox do pai |
//! | Houdini *Bend* | uma **capture region** autorada à parte |
//!
//! ⇒ **um número que se digita · um objecto que se aponta · uma bbox que se congela.** Nenhuma
//! é *«o centro do que está a passar por aqui, AGORA»* — e é essa que o nosso substrato entrega
//! de graça e **no dispositivo**, porque o par `Sum(v.x)`/`Sum(v.y)` do canal
//! [`crate::reduce_meta`] corre antes do passe por elemento e o resultado chega ao corpo do
//! kernel como um número.
//!
//! ## ⚠️ O que este módulo NÃO faz
//!
//! Ele **não** decide por nenhum nó. Um nó adopta-o declarando um `ParamSpec` chamado
//! [`PARAM`], pintando-o com [`LABELS`] e chamando [`PivotMode::of`] — e continua a ser dele a
//! escolha do DEFAULT, porque o default é a lei da identidade byte-a-byte de todo documento já
//! autorado, e ela é diferente em cada nó (o `motion.transform` sempre escalou em torno da
//! ORIGEM ⇒ `WorldOrigin`; o `motion.kaleidoscope` sempre honrou o ponto digitado ⇒ `Point`).
//!
//! ⛔ **E há nós que ficam de fora por NATUREZA, não por esquecimento:** o `motion.rotate` e o
//! `motion.scale` escrevem `rot` e `size`, não `P` — «o centro de uma escala» não é uma
//! pergunta que aquele dado admita, e as duas recusas estão escritas nos doc-comments deles.
//! O `motion.spherize` fica de fora por uma **cerca medida** (o `offset` dele é relativo ao
//! centroide, e um centro absoluto arrancaria a lente do assunto em todo documento já gravado).

/// O nome do param que carrega o modo. Um nó que o declare com outro nome não é apanhado pelo
/// censo que ata a família — e é por isso que o nome vive aqui, e não em cada crate.
pub const PARAM: &str = "pivot_mode";

/// Os rótulos, na ordem dos valores. ⚠️ **A ordem é o contrato**: o valor é gravado no
/// documento, então trocar duas entradas muda o significado de toda cena já salva.
pub const LABELS: &[&str] = &["World Origin", "Point", "Centroid"];

/// **Em torno de quê a deformação acontece.**
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum PivotMode {
    /// `(0, 0)` — nenhuma redução.
    WorldOrigin,
    /// O `pivot_x`/`pivot_y` que o artista digitou.
    Point,
    /// A média do `P` que entrou — o número que o layout **é**, remedido a cada quadro.
    Centroid,
}

impl PivotMode {
    /// Do valor do param. ⚠️ **Fora de alcance cai em [`Self::WorldOrigin`]** e não num
    /// `panic`: um documento de uma versão futura tem de abrir com a arte no ecrã.
    ///
    /// ⚠️ O arredondamento é o do Rust (metade **para longe do zero**); um kernel que leia o
    /// mesmo param tem de portar essa convenção, porque o `round` do WGSL é metade-par.
    #[must_use]
    pub fn of(v: f32) -> Self {
        match v.round() as i32 {
            1 => Self::Point,
            2 => Self::Centroid,
            _ => Self::WorldOrigin,
        }
    }

    /// O ponto, resolvido. `p` é a coluna `P` que entrou (vazia ⇒ o centroide não existe e a
    /// resposta é a origem, que é a deformação que o artista ainda consegue ver — nunca um
    /// `NaN`, que apaga a arte).
    #[must_use]
    pub fn resolve(self, typed: [f32; 2], p: &[[f32; 2]]) -> [f32; 2] {
        match self {
            Self::WorldOrigin => [0.0, 0.0],
            Self::Point => typed,
            Self::Centroid => crate::reduce_meta::centroid_of(p).unwrap_or([0.0, 0.0]),
        }
    }
}

/// **O gémeo em WGSL de [`PivotMode::of`] + [`PivotMode::resolve`]** — o prólogo que o corpo
/// de um kernel cola tal e qual, com `concat!`.
///
/// Escreve **`pv_pivot`** (um `vec2<f32>`) e espera do módulo `params.pivot_mode`,
/// `params.pivot_x`, `params.pivot_y`, `params.count` e — porque o modo `Centroid` o pede —
/// as duas reduções `cx`/`cy` que o nó tem de declarar.
///
/// ⚠️ **É a segunda expressão da lei e é inevitável** (o dispositivo não chama Rust); o que a
/// mantém honesta é o gate de paridade de cada nó. ⛔ Mas é **uma** string, aqui, em vez de uma
/// por crate — e o preço de a copiar está medido: a `DRIVE_LIB_HSV` do `motion.drive` era uma
/// cópia à mão de metade da biblioteca do irmão, saiu **sem `drive_resolve`**, e três canais
/// daquele nó não compilavam no dispositivo desde que existem (curado em 2026-09-07).
///
/// ⚠️ **O arredondamento é metade-para-longe-do-zero**, a convenção do Rust — o `round` do
/// WGSL é metade-par, e `pivot_mode` escolhe um RAMO. Com um `Enum` os valores são inteiros e
/// as duas convenções coincidem; a linha existe para o dia em que um fio conduzir o param.
///
/// ⚠️ **É um `macro_rules!` e não um `const`** porque `concat!` só aceita literais, que é
/// exactamente o obstáculo que levou a cópia à mão do `motion.drive` a existir.
#[macro_export]
macro_rules! pivot_wgsl {
    () => {
        "\
        let pv_m = i32(select(ceil(params.pivot_mode - 0.5), \
                              floor(params.pivot_mode + 0.5), \
                              params.pivot_mode >= 0.0));\n\
        var pv_pivot = vec2<f32>(0.0, 0.0);\n\
        if (pv_m == 1) { pv_pivot = vec2<f32>(params.pivot_x, params.pivot_y); }\n\
        if (pv_m == 2) {\n\
        \x20   pv_pivot = vec2<f32>(reduce_cx(), reduce_cy()) / f32(params.count);\n\
        }\n"
    };
}

/// As duas reduções que o modo [`PivotMode::Centroid`] pede — `Sum` sobre `P.x` e `P.y`, com o
/// kernel a dividir por `params.count`. Um nó que adopte o [`PARAM`] declara-as com
/// `register_reduces`, e é isto que faz o centroide correr **no dispositivo** em vez de
/// derrubar a cadeia para a CPU.
///
/// ⚠️ **Elas correm SEMPRE, mesmo nos modos que não as leem** — o `ReduceSpec` não tem gate de
/// param. ⭐ E há uma segunda razão para ficarem incondicionais: o `Sum` sobre um `P` ausente
/// dobra a identidade `0` e devolve a origem, que é exactamente o que
/// [`PivotMode::resolve`] calcula, então os dois caminhos concordam sem o kernel ter de saber
/// se a coluna existe.
///
/// ⚠️⚠️ **NÃO conte com identidade de PONTEIRO para provar que um nó usa esta tabela.** Medido:
/// mesmo sendo um `static`, o `&[…]` que o inicializa é uma constante **promovida** e um leitor
/// noutra crate re-materializa-a — os dois endereços diferem, e um gate escrito com
/// `std::ptr::eq` reprova sobre código correcto. *Uma régua de identidade que a linguagem não
/// garante mede o compilador, não o código.* O gate que serve compara os CAMPOS.
pub static CENTROID_REDUCES: &[crate::reduce_meta::ReduceSpec] = &[
    crate::reduce_meta::ReduceSpec {
        name: "cx",
        column: "P",
        dim: crate::port::Dim::Vec2,
        port: 0,
        op: crate::reduce_meta::ReduceOp::Sum,
        value: "v.x",
        params: &[],
        identity: [0.0; 4],
    },
    crate::reduce_meta::ReduceSpec {
        name: "cy",
        column: "P",
        dim: crate::port::Dim::Vec2,
        port: 0,
        op: crate::reduce_meta::ReduceOp::Sum,
        value: "v.y",
        params: &[],
        identity: [0.0; 4],
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_scale_is_the_one_the_document_carries() {
        assert_eq!(PivotMode::of(0.0), PivotMode::WorldOrigin);
        assert_eq!(PivotMode::of(1.0), PivotMode::Point);
        assert_eq!(PivotMode::of(2.0), PivotMode::Centroid);
        // Fora de alcance abre com a arte no ecrã, nao com um panico.
        assert_eq!(PivotMode::of(7.0), PivotMode::WorldOrigin);
        assert_eq!(PivotMode::of(-3.0), PivotMode::WorldOrigin);
        // E o arredondamento e' o do Rust: 0,5 vai para LONGE do zero.
        assert_eq!(PivotMode::of(0.5), PivotMode::Point);
        // Os rotulos descrevem exactamente os valores que a escada aceita.
        assert_eq!(LABELS.len(), 3);
    }

    #[test]
    fn a_layout_with_no_positions_pivots_on_the_origin_instead_of_a_nan() {
        assert_eq!(PivotMode::Centroid.resolve([9.0, 9.0], &[]), [0.0, 0.0]);
        assert_eq!(PivotMode::WorldOrigin.resolve([9.0, 9.0], &[]), [0.0, 0.0]);
        assert_eq!(PivotMode::Point.resolve([9.0, 9.0], &[]), [9.0, 9.0]);
        assert_eq!(
            PivotMode::Centroid.resolve([9.0, 9.0], &[[4.0, 0.0], [6.0, 2.0]]),
            [5.0, 1.0],
            "o centroide VENCE o ponto digitado — o modo decide, nao a presenca de um numero"
        );
    }
}

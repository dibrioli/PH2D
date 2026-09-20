//! ⭐⭐⭐ **A FORMA ACESA COMO MATÉRIA** — o laço por texel que acende um objecto 2D com a lei do
//! OpenPBR, a partir das normais que uma malha 3D lhe doou.
//!
//! # Porque esta crate existe, com o número
//!
//! A rota A do [`02.2`](../../../docs/3D/02-Arquitetura/02.2-Sprite-com-malha-filha.md) está
//! construída: uma malha 3D assa-se num sprite e deixa lá `base` (o albedo), `form` (as normais) e
//! `form_occ` (a oclusão), e o objecto fica **re-iluminável**. ⛔ **Mas quem o acende é o
//! `ImpastoLightPass`** — o passe da TINTA do Painter, emprestado através de um adaptador que
//! *neutraliza* os planos dele (`baked_form::neutral_planes`).
//!
//! Medido no [`impasto_light.wgsl`](../../ph2d-render/src/shaders/impasto_light.wgsl), esse passe
//! é um modelo de tinta: difuso envolvido mais um especular lido de uma tabela, **sem GGX e sem
//! conservação de energia**. Ao lado, o modelador acende com o OpenPBR inteiro.
//!
//! ⇒ *o objecto 3D do jogo é aceso pela lei errada, e é essa a razão de ele não parecer o alvo.*
//!
//! # ⛔⛔ Porque NÃO se muda o passe que já existe
//!
//! O cabeçalho do `impasto_light.wgsl` declara duas propriedades **load-bearing**, e as duas são
//! sobre TINTA: *«tinta plana é byte-idêntica»* (o sombreamento é RELATIVO — divide pelo que uma
//! superfície plana do mesmo material devolve) e *«papel nu recebe exactamente nada»*. Mudar a lei
//! ali parte o Painter.
//!
//! ⇒ **passe NOVO, escolhido por objecto.** O antigo fica, porque é a lei certa para a tinta.
//!
//! # ⚠️ A diferença que o artista VAI ver, e ela é deliberada
//!
//! O passe da tinta é **relativo**; este é **absoluto**, como manda a física — é o que o
//! [`01` §1](../../../docs/Render3d/01_o_alvo_decomposto.md) chama de *«um pipeline fisicamente
//! correcto, com a direcção de arte a mentir por cima DE PROPÓSITO»*. Um objecto aceso por aqui
//! responde à luz como matéria: escurece onde a luz não chega e tem destaque especular com a forma
//! do GGX, em vez de um brilho tabelado.
//!
//! ⛔ **É por isso que a escolha é POR OBJECTO e não global**: um projecto gravado tem de continuar
//! a abrir com a aparência com que foi gravado (`baked_form` guarda o `rig` exactamente por essa
//! razão), e trocar a lei por baixo mudaria a arte em silêncio.
//!
//! # A vista é CONSTANTE, e isso não é uma simplificação
//!
//! Num objecto 2D o observador olha o canvas de frente: `v = (0, 0, 1)`. ⚠️ Não é uma aproximação
//! — é o que a projecção do canvas É. Uma vista por texel só faria sentido com câmera perspectiva,
//! e a engine é 2D **por desenho** ([`14` §5](../../../docs/Render3d/14_a_ordem_de_superar.md)).

#![forbid(unsafe_code)]

pub use ph2d_material::{OpenPbr, Rgb, Surface};

pub mod imagem;
pub mod wgsl;

#[cfg(test)]
mod tests;

/// **Uma lâmpada, já resolvida** — direcção e radiância, nada mais.
///
/// ⚠️ **Ela não é a `ph2d_light::Lamp`, e a diferença é a razão de esta crate ser folha.** Aquela
/// carrega o vocabulário do rig do artista (o `half` do modelo de tinta, o `tint` antes da
/// intensidade); esta carrega o que a **óptica** precisa. Quem converte é o chamador, que já tem o
/// rig — e é lá que a conversão tem de morar, porque ela é uma decisão sobre o RIG.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lampada {
    /// Da superfície PARA a luz, **normalizada**. ⚠️ O sentido é o que a
    /// [`Surface::direct`] espera; invertê-lo apaga a peça em vez de a acender.
    pub para_a_luz: [f32; 3],
    /// A radiância que ela entrega, já com intensidade e cor.
    pub radiancia: Rgb,
}

/// **O que um texel traz** — o que a forma doou, num ponto.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Texel {
    /// A normal doada pela malha, em espaço de vista. ⚠️ Pode vir **não normalizada** do
    /// `baked_form` (ela viaja quantizada em `rgba8`), e [`acende_texel`] normaliza-a — ver lá
    /// porque isso não é defensivo.
    pub normal: [f32; 3],
    /// O albedo — os pixels **antes** da luz.
    pub albedo: Rgb,
    /// Quanto deste texel é forma (`0` = nada, `1` = forma cheia). Fora da silhueta a lei é um
    /// **no-op exacto**, que é a metade do contrato que o `baked_form` já cumpre com a tinta.
    pub cobertura: f32,
    /// A oclusão que a forma doou (`1` = aberto, `0` = fechado).
    pub oclusao: f32,
}

/// A vista de um canvas 2D. Ver o cabeçalho do módulo.
pub const VISTA: [f32; 3] = [0.0, 0.0, 1.0];

/// ⭐⭐⭐ **A LEI, num texel.** Ver o cabeçalho do módulo.
///
/// # ⚠️ Porque a normal é normalizada AQUI
///
/// Não é uma guarda defensiva: o `baked_form` guarda a forma em `rgba8` (medido — em `f32` custaria
/// `4×` o disco), e a viagem por oito bits **encurta o vector**. Uma normal de comprimento `0,97`
/// num `N·L` dá uma peça `3 %` mais escura em todo lado, e isso lê-se como *«o material está
/// errado»*. ⛔ Normalizar no chamador poria a mesma conta em cada consumidor novo.
///
/// # ⚠️ A COBERTURA multiplica no fim, e a ordem importa
///
/// Ela mistura entre o albedo cru e o albedo aceso — **nunca entre preto e o aceso**. Misturar com
/// preto escureceria a borda da silhueta a cada re-acendida, que é exactamente o defeito que o
/// `baked_form` guarda `base` para impedir (*«re-acender a partir do que já está aceso compõe»*).
#[must_use]
pub fn acende_texel(s: &Surface, t: &Texel, lampadas: &[Lampada], ambiente: Rgb) -> Rgb {
    let n = normaliza(t.normal);
    // ⚠️ Uma normal degenerada (o texel fora da silhueta, onde a forma não escreveu nada) devolve o
    // albedo **cru**. ⛔ Devolver preto pintaria um halo no contorno de toda peça assada.
    let Some(n) = n else {
        return t.albedo;
    };

    let mut luz: Rgb = [0.0; 3];
    for l in lampadas {
        // ⛔⛔⛔ **A LÂMPADA ANTI-PARALELA À VISTA, e ela só é alcançável porque a vista é
        // CONSTANTE.** Medido: com `to_light = -VISTA` a lei devolve `[NaN, NaN, NaN]`, porque o
        // meio-vector `v + to_light` é o vector nulo e normalizá-lo não tem resposta.
        //
        // ⚠️ **No modelador isto tem medida nula** — ali a vista é a do raio e varia por pixel,
        // logo nenhuma lâmpada é anti-paralela em mais do que um ponto. **Num canvas 2D a vista é
        // `(0,0,1)` em TODO texel**, e uma lâmpada apontada de frente para trás cai exactamente
        // aqui, na peça inteira. *A constante que simplifica o 2D é a mesma que torna um caso
        // degenerado alcançável.*
        //
        // ⭐ E o limite é ZERO, não um valor escolhido: a `1e-3` de distância do caso a lei já lê
        // `2,6e-9` (medido). Devolver zero é continuar a curva, não inventar um número.
        //
        // ⛔ **A cura mora AQUI e não na [`ph2d_material`]**: aquela crate é o porte fiel do
        // MaterialX e o caso é do nosso CONSUMIDOR, não da óptica.
        if meio_vector_degenera(l.para_a_luz) {
            continue;
        }
        let r = s.direct(n, VISTA, l.para_a_luz, l.radiancia);
        luz = [luz[0] + r[0], luz[1] + r[1], luz[2] + r[2]];
    }

    // ⚠️ **A oclusão só pesa o AMBIENTE**, nunca a luz directa. Uma lâmpada que o artista apontou
    // tem de chegar onde ele a apontou; escurecer a directa com oclusão de forma é o que faz um
    // objecto parecer sujo em vez de ocluído — e nenhuma das cinco referências o faz.
    let amb = [
        ambiente[0] * t.oclusao,
        ambiente[1] * t.oclusao,
        ambiente[2] * t.oclusao,
    ];

    let aceso = [
        t.albedo[0] * (luz[0] + amb[0]),
        t.albedo[1] * (luz[1] + amb[1]),
        t.albedo[2] * (luz[2] + amb[2]),
    ];

    let c = t.cobertura.clamp(0.0, 1.0);
    let n_c = 1.0 - c;
    [
        t.albedo[0] * n_c + aceso[0] * c,
        t.albedo[1] * n_c + aceso[1] * c,
        t.albedo[2] * n_c + aceso[2] * c,
    ]
}

/// ⛔⛔ **O meio-vector degenera** — `v + to_light ≈ 0`, o caso em que a lei do OpenPBR devolve
/// `NaN` porque normalizar o vector nulo não tem resposta.
///
/// ⚠️ **Só é alcançável porque a [`VISTA`] é CONSTANTE**, logo basta olhar a lâmpada: no modelador a
/// vista é a do raio e varia por pixel, e este caso tem medida nula; aqui ele é uma configuração
/// que o artista escreve, e vale para a peça inteira de uma vez.
///
/// ⚠️ O limiar é o quadrado, como na [`normaliza`], e a `1e-3` de distância deste caso a lei já lê
/// `2,6e-9` (medido) — *devolver zero continua a curva em vez de inventar um número*.
fn meio_vector_degenera(para_a_luz: [f32; 3]) -> bool {
    let h = [
        VISTA[0] + para_a_luz[0],
        VISTA[1] + para_a_luz[1],
        VISTA[2] + para_a_luz[2],
    ];
    h[0] * h[0] + h[1] * h[1] + h[2] * h[2] < 1e-12
}

/// Normaliza, ou `None` se o vector não tem direcção.
///
/// ⚠️ O limiar é o quadrado do comprimento e não o comprimento: poupa a raiz no caso que sai, e
/// `1e-12` é o quadrado de `1e-6`, que é onde um `f32` deixa de ter dígitos para uma direcção.
fn normaliza(v: [f32; 3]) -> Option<[f32; 3]> {
    let q = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
    if q < 1e-12 {
        return None;
    }
    let inv = 1.0 / q.sqrt();
    Some([v[0] * inv, v[1] * inv, v[2] * inv])
}

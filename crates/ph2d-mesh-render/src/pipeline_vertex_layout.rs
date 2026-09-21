//! **A DISPOSIÇÃO POR VÉRTICE** — os nove buffers que o `mesh.wgsl` lê.
//!
//! ⚠️ Irmão (`#[path]`) do [`super::super`], cortado pelo tecto de LOC do
//! `pipeline_build.rs` (`719` contra `700`) e pelo ASSUNTO: *como o passe é
//! MONTADO* cresce quando o pipeline ganha um estado, e **esta** lista cresce
//! quando a MALHA ganha um canal — a última a crescer foi a COR, que é o 9.º e
//! o primeiro acima do piso do WebGPU.
//!
//! ⛔ **Um canal novo escreve-se AQUI e no shader, e em mais lado nenhum** — a
//! ordem desta lista **é** o `@location` que o WGSL declara, e uma segunda
//! cópia dela noutro ficheiro divergiria no dia do 10.º buffer.

// Os atributos têm de viver tanto quanto o descritor ⇒ `static`, e não
// `const`: um `&CONST` só chega a `'static` por PROMOÇÃO, que é uma regra
// sobre a expressão e não uma garantia da assinatura — e a assinatura que
// este módulo publica é `VertexBufferLayout<'static>`.
const fn vec3_attr(location: u32) -> [wgpu::VertexAttribute; 1] {
    [wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x3,
        offset: 0,
        shader_location: location,
    }]
}
static POS: [wgpu::VertexAttribute; 1] = vec3_attr(0);
static NRM: [wgpu::VertexAttribute; 1] = vec3_attr(1);
const fn f32_attr(location: u32) -> [wgpu::VertexAttribute; 1] {
    [wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32,
        offset: 0,
        shader_location: location,
    }]
}
static MASK: [wgpu::VertexAttribute; 1] = f32_attr(2);
/// A CURVATURA por vértice (`ph2d_mesh::curvature`) — buffer próprio, e
/// não um segundo canal empacotado com a máscara, porque as duas mudam em
/// momentos diferentes: a máscara quando o artista a pinta, a curvatura
/// em TODO dab. Juntá-las faria um upload incremental de forma reenviar
/// a autoria que ninguém tocou.
static CURV: [wgpu::VertexAttribute; 1] = f32_attr(3);
/// O AO ASSADO por vértice — buffer próprio pela razão da curvatura, e
/// por uma a mais: ele é o canal que muda MENOS de todos (só num bake
/// explícito), então empacotá-lo com qualquer vizinho faria o upload
/// dele viajar de carona em toda mudança de forma.
static AO: [wgpu::VertexAttribute; 1] = f32_attr(4);
/// A CURVATURA DE MUNDO por vértice (`1/comprimento`) — o eixo da tabela
/// do SSS. Buffer próprio pela razão da irmã adimensional ao lado, e não
/// empacotada COM ela apesar de mudarem no mesmo instante: um `vec2` num
/// buffer só é o layout certo se as duas forem sempre lidas juntas, e o
/// Cavity lê uma sem a outra em todo frame com o SSS desligado.
static CURVW: [wgpu::VertexAttribute; 1] = f32_attr(5);
/// A ESPESSURA assada por vértice — buffer próprio pela razão do AO, que
/// é a irmã dele em tudo: os dois nascem do MESMO bake, mudam só nele, e
/// empacotá-los juntos economizaria um buffer para pagar com um upload
/// de canal que ninguém mexeu em toda troca de forma.
static THICK: [wgpu::VertexAttribute; 1] = f32_attr(6);
/// O PREVIEW do padrão do pincel — o canal **transiente** que mostra,
/// no barro, o que o próximo traço vai depositar.
///
/// ⚠️ **Irmão da máscara e não dela:** os dois são `f32` por vértice e
/// pintam um tinto, mas a máscara é AUTORADA (ela protege) e este é
/// DERIVADO do pincel vivo. Colapsá-los faria o preview apagar a
/// proteção que o artista pintou — e restaurá-la depois seria uma
/// promessa que um `return` esquecido quebra em silêncio.
static PREVIEW: [wgpu::VertexAttribute; 1] = f32_attr(7);
/// A COR por vértice — o albedo que os pincéis de pintura escrevem.
///
/// ⚠️ **Buffer próprio e não empacotado com um vizinho**, pela mesma
/// razão que separa o AO da espessura: ele muda a CADA dab de pintura,
/// e o vizinho com quem coubesse pagaria um upload por dab de um canal
/// que ninguém tocou. ⭐ É o 9.º buffer, um acima do piso do WebGPU —
/// ver a medição em [`ph2d_gpu`].
static COLOR: [wgpu::VertexAttribute; 1] = vec3_attr(8);
// Irmão do `vec3_buffer`. Os dois são `const fn` do módulo e já não
// closures do construtor: o corte pelo tecto de LOC tirou-os de dentro da
// `new`, e nada aqui captura nada.
const fn f32_buffer(attrs: &'static [wgpu::VertexAttribute]) -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: 4,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: attrs,
    }
}
const fn vec3_buffer(attrs: &'static [wgpu::VertexAttribute]) -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: 12,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: attrs,
    }
}

/// **A lista que o pipeline recebe** — a ordem é o `@location` do shader.
///
/// ⚠️ Ela é devolvida por VALOR e os dois pipelines (a cena e o G-buffer) a
/// pedem ao mesmo sítio: *duas listas escritas à mão seriam duas leis sobre
/// onde cada canal entra, e o compilador não as cruzaria.*
pub(crate) const fn layout_por_vertice() -> [wgpu::VertexBufferLayout<'static>; 9] {
    [
        vec3_buffer(&POS),
        vec3_buffer(&NRM),
        f32_buffer(&MASK),
        f32_buffer(&CURV),
        f32_buffer(&AO),
        f32_buffer(&CURVW),
        f32_buffer(&THICK),
        f32_buffer(&PREVIEW),
        vec3_buffer(&COLOR),
    ]
}

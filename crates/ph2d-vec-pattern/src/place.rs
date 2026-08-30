//! **ONDE o ladrilho fica** — o afim que leva os pixels do assado ao espaço das ÂNCORAS.
//!
//! # A lei que este módulo herda inteira, e que decide tudo
//!
//! O cabeçalho da [`Paint`](../../ph2d-vec-scene/src/paint.rs) já a escreveu para os gradientes: a
//! geometria de um preenchimento vive no **espaço das âncoras** e **transforma junto com o path**.
//! Rodar a forma roda o preenchimento rigidamente, sem *"respirar"* — foi a cura de um bug real (o
//! gradiente relativo à bbox respirava a cada edição).
//!
//! O padrão herda-a **sem uma linha de código de acompanhamento**, e o mecanismo é do Vello: ele
//! compõe `transform * brush_transform` (`vello-0.8.0/src/scene.rs:329`), e o `transform` que a
//! `ph2d-vec-render` já passa é `câmara * Transform_da_entidade`. ⇒ pôr a colocação em espaço de
//! âncoras faz o padrão cavalgar a pose de graça.
//!
//! ⚠️⚠️ **E por isso o padrão ESMAGA sob escala não-uniforme, ao contrário da caneta do traço.**
//! Não confunda com o [bug #27](../../../docs/Vector%20Module/BUGS_vector.md): o traço é a
//! **ferramenta que desenha** a forma (o Enio decidiu em 23/08 que ela engrossa por igual nos dois
//! eixos, `√|det|`); o preenchimento está **colado** à forma — um gradiente radial já vira elipse
//! hoje, e ninguém chamou a isso um defeito.
//!
//! # A disposição do afim
//!
//! `[a, b, c, d, e, f]` na convenção do `kurbo::Affine` (`x' = a*x + c*y + e`, `y' = b*x + d*y + f`)
//! — a MESMA que o `xform_of` da `ph2d-vec-scene` já usa. É por isso que esta folha não precisa de
//! depender de `kurbo`.

/// `√3 / 2` — o único quociente de espaçamento que põe os **seis** vizinhos de um reticulado
/// desfasado a meio passo à mesma distância. É o que separa uma colmeia de um tijolo.
pub const HEX_ROW_RATIO: f64 = 0.866_025_403_784_438_6;

/// O período VERTICAL de uma colmeia: o passo da linha **apertado** por [`HEX_ROW_RATIO`].
///
/// ⭐ **A colmeia não tem assado próprio** — ela assa byte-a-byte como um tijolo por linha de meio
/// passo. O que a faz colmeia é esta função, e é por isso que a lei mora **num sítio só**: escrita
/// duas vezes, o desenho ficaria num instante e o espaçamento noutro.
///
/// ⛔⛔ **O ARGUMENTO É O PASSO DA LINHA, nunca o da coluna** (report do Enio, 2026-08-30). O
/// parâmetro chamava-se `col_period` e o único chamador passava-lhe o passo horizontal: numa célula
/// quadrada dá no mesmo, e numa célula alta o passo vertical colapsa para `0,866 ×` a **largura**,
/// com sobreposição medida de **56 % da altura** e a cópia vizinha a reescrever metade do motivo.
/// *Um nome de parâmetro é um contrato, e este estava a pedir o eixo errado.*
#[must_use]
pub fn hex_row_period(row_period: f64) -> f64 {
    row_period * HEX_ROW_RATIO
}

/// O vão do artista (**mundo**) convertido para pixels da arte.
///
/// A escala é a da própria arte: `pixels / unidades`. ⚠️ Um `size` degenerado (zero, infinito, NaN)
/// devolve zero em vez de dividir por zero — um vão que não se sabe medir é um vão que não existe.
#[must_use]
pub fn gap_px_from_world(gap: [f64; 2], size: [f64; 2], art_px: [u32; 2]) -> [i32; 2] {
    let one = |g: f64, s: f64, a: u32| -> i32 {
        if !g.is_finite() || !s.is_finite() || s.abs() <= f64::EPSILON {
            return 0;
        }
        let v = (g * f64::from(a) / s)
            .round()
            .clamp(f64::from(i32::MIN), f64::from(i32::MAX));
        #[allow(clippy::cast_possible_truncation)]
        let px = v as i32;
        px
    };
    [
        one(gap[0], size[0], art_px[0]),
        one(gap[1], size[1], art_px[1]),
    ]
}

/// O afim **pixels do ladrilho -> espaço das âncoras**.
///
/// - `period` — quanto mede UMA célula no mundo (o passo da repetição: arte + vão);
/// - `cells` — quantas células o ladrilho assado contém (`[1,1]` na grade, `[1,n]` no tijolo por
///   linha), ou seja quantos PERÍODOS o rectângulo cobre;
/// - `origin` / `angle` — a colocação autorada;
/// - `tile_px` — a resolução do assado.
///
/// ⭐ **A resolução do assado não aparece na resposta** a não ser como divisor: dois ladrilhos com o
/// mesmo período e contagens de pixels diferentes mapeiam para o **mesmo** rectângulo de mundo. É o
/// que permite re-assar em melhor qualidade (ou com outra lei) sem deslocar o desenho do artista.
#[must_use]
pub fn placement(
    period: [f64; 2],
    cells: [u32; 2],
    origin: [f64; 2],
    angle: f64,
    tile_px: [u32; 2],
) -> [f64; 6] {
    let tw = f64::from(tile_px[0].max(1));
    let th = f64::from(tile_px[1].max(1));
    let sx = period[0] * f64::from(cells[0].max(1)) / tw;
    let sy = period[1] * f64::from(cells[1].max(1)) / th;
    let (sin, cos) = angle.sin_cos();
    // ⛔⛔⛔ **O EIXO DAS LINHAS APONTA PARA BAIXO** (report do Enio, 2026-08-30: *"o padrão está de
    // cabeça para baixo"*).
    //
    // A linha `0` do assado é o **topo** do desenho — o assador põe lá o canto de cima, sob uma
    // câmara de Y invertido, e tem gate a prová-lo (`the_baked_tile_is_upright`). A âncora desta
    // colocação é o canto **INFERIOR** esquerdo da caixa da forma (`default_placement` devolve o
    // `lo`). ⇒ com `+sy` a linha 0 caía no fundo e as seguintes subiam: um espelho vertical exacto.
    //
    // ⚠️ **A caixa de mundo NÃO muda** — continua a ser `origem .. origem + período x células`. O
    // que se inverte é qual linha cai em cima, e é por isso que a cura não desloca padrão nenhum:
    // ela espelha-o **dentro** da mesma caixa. A base (`py = th`) assenta na âncora, e a linha `0`
    // fica uma altura de ladrilho acima dela.
    let hy = sy * th;
    [
        cos * sx,
        sin * sx,
        sin * sy,
        -cos * sy,
        origin[0] - sin * hy,
        origin[1] + cos * hy,
    ]
}

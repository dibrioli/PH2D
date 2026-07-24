//! **Os mapas track↔valor da seção Contour** — módulo irmão de [`super`], e a PORTA ÚNICA de cada
//! um deles.
//!
//! Cada mapa era preciso em três sítios — o `scale`/`offset` que o `populate` dá ao chip numérico,
//! a conversão do `event.rs` que alimenta o bus, e o inverso que o `paint` usa para pôr o slider no
//! lugar. Três cópias da mesma aritmética, em três arquivos, é a forma exata que diverge em
//! silêncio: digitar `8` e arrastar até 8 dariam valores diferentes, e nenhum dos dois pareceria
//! errado. Com uma porta, divergir deixa de ser possível em vez de ser gateado.
//!
//! É a lição que a Rotation do Pattern on Path pagou uma wave antes (`rotation_from_track`), agora
//! aplicada de saída aos quatro controles.

/// Quantos anéis o slider de **Steps** alcança.
///
/// ⚠️ **É a SEGUNDA cópia de um número cuja autoridade é `ph2d_ecs::MAX_CONTOUR_STEPS`**, e as duas
/// existem porque nem o painel nem a crate de ferramenta dependem do ECS (nem devem: um painel que
/// conhece componentes é um painel que conhece a cena). A cópia é pinada por um gate de shell —
/// que é quem vê os dois lados — em vez de ser mantida por prosa.
pub(crate) const CONTOUR_STEPS_MAX: f64 = 16.0; // LITERAL-PX-OK: faixa no domínio do documento
/// Com quantos anéis um contour NASCE — o valor com que os controles são registrados e o que o
/// `paint` mostra enquanto a shell não publicou nada.
///
/// ⚠️ **Const, e não o literal repetido:** este número era preciso em QUATRO sítios (o `populate`
/// regista o slider e o chip com ele, o `event` usa-o de default do track, o `state` de valor
/// inicial), e quatro cópias de um default é como o painel passa a mostrar `4` sobre um contour
/// que nasceu com outra coisa. A autoridade é `ph2d_ecs::VecContour::default()`, do outro lado de
/// uma fronteira que o painel não atravessa — a igualdade é pinada por um gate de shell.
pub(crate) const CONTOUR_STEPS_DEFAULT: f64 = 4.0; // LITERAL-PX-OK: default no domínio do documento
/// A meia-faixa do **Offset por passo**, em FRAÇÃO do tamanho da forma: o slider é bipolar
/// (`−CONTOUR_D_MAX..CONTOUR_D_MAX`, track `0..1`, `0.5` = anéis coincidentes com a fonte).
///
/// ⚠️ **Fração, não unidades de mundo** — a mesma razão do Offset da seção Expand: o mapa do store
/// é estático, então um rótulo em unidades de mundo mentiria sempre que a seleção mudasse de
/// tamanho. O `d` que o componente guarda É de mundo (tem de sobreviver à troca de seleção); a
/// conversão acontece na fronteira, com a MESMA `vec_expand::offset_scale` que o Offset usa.
///
/// ⚠️ O valor (25% do tamanho da forma, por passo) é escolhido contra o TETO de passos: a 16
/// passos o anel externo já fica a 4× o tamanho da forma, que é bem além do que se usa. Uma
/// meia-faixa maior poria todo o curso útil do slider nos primeiros 5% do trilho.
pub(crate) const CONTOUR_D_MAX: f64 = 0.25; // LITERAL-PX-OK: faixa no domínio do documento
/// O teto da **aceleração**; o piso é o recíproco (`1/CONTOUR_ACCEL_MAX`). A faixa é GEOMÉTRICA e
/// não linear, e é isso que põe o neutro `1.0` no CENTRO do trilho: linearmente, `0.25..4.0`
/// deixaria o `1.0` a 21% do curso, e o artista teria de o caçar.
pub(crate) const CONTOUR_ACCEL_MAX: f64 = 4.0; // LITERAL-PX-OK: faixa no domínio do documento

/// O track `0..1` do slider **para o número de anéis** (`1..=CONTOUR_STEPS_MAX`), arredondado.
///
/// Arredonda porque um contour tem um número INTEIRO de anéis: `4,5` anéis não existe, e um chip
/// que mostrasse `4,5` prometeria um estado que a cena não consegue ter.
pub(crate) fn steps_from_track(t: f64) -> f64 {
    t.mul_add(CONTOUR_STEPS_MAX - 1.0, 1.0).round()
}

/// O inverso da [`steps_from_track`]: o número de anéis para o track `0..1`.
pub fn steps_to_track(steps: f64) -> f32 {
    (((steps - 1.0) / (CONTOUR_STEPS_MAX - 1.0)) as f32).clamp(0.0, 1.0)
}

/// O track `0..1` do slider **para a fração de offset por passo** (bipolar).
pub(crate) fn d_from_track(t: f64) -> f64 {
    t.mul_add(2.0 * CONTOUR_D_MAX, -CONTOUR_D_MAX)
}

/// O inverso exato da [`d_from_track`].
pub fn d_to_track(frac: f64) -> f32 {
    (((frac / CONTOUR_D_MAX) * 0.5 + 0.5) as f32).clamp(0.0, 1.0)
}

/// O track `0..1` do slider **para a aceleração**, GEOMETRICAMENTE: `0.5` é exatamente `1.0`.
pub(crate) fn accel_from_track(t: f64) -> f64 {
    CONTOUR_ACCEL_MAX.powf(t.mul_add(2.0, -1.0))
}

/// O inverso exato da [`accel_from_track`].
pub fn accel_to_track(accel: f64) -> f32 {
    ((accel.max(f64::MIN_POSITIVE).log(CONTOUR_ACCEL_MAX) + 1.0) * 0.5).clamp(0.0, 1.0) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Os quatro mapas são inversos um do outro** — a propriedade que faz o slider e o chip
    /// numérico concordarem. Sem ela, arrastar até um valor e digitar o mesmo valor põem o slider
    /// em sítios diferentes, e nada na tela diz qual dos dois está certo.
    /// A tolerância é do TRILHO, que é `f32`: o epsilon relativo dele é ~1,2e-7, então exigir
    /// `1e-9` seria afirmar uma precisão que a representação não tem — e o gate falharia em
    /// alguns pontos e não noutros, por arredondamento, sobre um mapa correto (aconteceu).
    const TRACK_EPS: f64 = 1e-6;

    #[test]
    fn every_track_map_round_trips() {
        for i in 0..=40 {
            let t = f64::from(i) / 40.0;
            let d = d_from_track(t);
            assert!(
                (f64::from(d_to_track(d)) - t).abs() < TRACK_EPS,
                "offset: track {t} -> {d} -> {}",
                d_to_track(d)
            );
            let a = accel_from_track(t);
            assert!(
                (f64::from(accel_to_track(a)) - t).abs() < TRACK_EPS,
                "accel: track {t} -> {a} -> {}",
                accel_to_track(a)
            );
        }
        // Steps arredonda, então o round-trip é do VALOR (não do track): todo inteiro alcançável
        // tem de voltar a si mesmo, senão o slider salta ao mostrar o que o chip acabou de aceitar.
        for n in 1..=16 {
            let steps = f64::from(n);
            assert!(
                (steps_from_track(f64::from(steps_to_track(steps))) - steps).abs() < TRACK_EPS,
                "steps: {steps} não sobrevive ao round-trip"
            );
        }
    }

    /// **O neutro de cada controle cai onde o artista o encontra.** O offset bipolar tem o zero no
    /// meio do trilho, e a aceleração tem o `1.0` — e é POR ISSO que a faixa dela é geométrica: no
    /// mapa linear que a versão óbvia daria, `1.0` cairia a 21% do curso.
    #[test]
    fn the_neutral_of_each_control_sits_at_the_centre_of_the_track() {
        assert!(
            d_from_track(0.5).abs() < 1e-12,
            "offset zero fora do centro"
        );
        assert!(
            (accel_from_track(0.5) - 1.0).abs() < 1e-12,
            "aceleração linear ({}) fora do centro — a faixa deixou de ser geométrica",
            accel_from_track(0.5)
        );
        assert!(
            (accel_from_track(0.0) - 1.0 / CONTOUR_ACCEL_MAX).abs() < 1e-12,
            "o piso da aceleração não é o recíproco do teto"
        );
    }
}

//! **O EIXO e o AFUNILAMENTO** do `motion.spline_wrap` (doc 89, folha 04 — as duas últimas
//! células desta folha).
//!
//! ⚠️ **Este arquivo existe pelo mesmo teto de LOC que criou o [`super::ui`]** (HR-18, 700 para
//! `crates/`), e o corte é o mesmo: o `lib.rs` responde *como o enrolamento funciona* e este
//! responde *que eixo entra nele e o que sai afunilado*.

use super::P2;

/// **QUAL EIXO DO LAYOUT CORRE NA CURVA** (doc 89 folha 04 — C4D Spline Wrap ▸ **Axis**,
/// `±X / ±Y / ±Z`).
///
/// O nó lia **sempre o X**: `u` saía de `(x − xmin)/w` e o `y` era o desvio ao longo da normal.
/// A célula media a composição — a mesma do `motion.bend` (`motion.orbit ±θ` a envolver o nó) —
/// e o preço: **três nós por um knob**, e a `motion.orbit` é `Effect::Temporal`, então a
/// sub-árvore inteira mudava de classe de efeito por causa de uma escolha de eixo.
///
/// ⚠️ **Um ÂNGULO, e não quatro estados** — a mesma decisão que o `direction` do `motion.bend`
/// tomou, e pela mesma razão: um ângulo contínuo CONTÉM os `±X`/`±Y` da referência (`0`, `180`,
/// `90`, `−90`) e ainda entrega o que está entre eles. O `±Z` fica de fora por não existir: um
/// stream planar não tem terceiro eixo, e o `banking` da mesma referência já foi recusado por
/// essa razão.
///
/// ⚠️ **A rotação entra e NÃO sai** — e é aqui que ele diverge do irmão `motion.bend`. A dobra
/// produz geometria no quadro dela e tem de a trazer de volta; aqui a curva **já vive no
/// mundo**, então o ângulo só muda *como o layout é LIDO*. Girar a saída poria a curva
/// desenhada pelo artista noutro sítio que não onde ele a desenhou.
///
/// ⚠️ **O giro é em torno da ORIGEM, não de um pivô**, e isso é a continuação do que o nó já
/// fazia: o `u` é invariante a translação (sai de `min`/`max` do próprio layout), e o desvio
/// perpendicular **sempre** foi o `y` cru, medido da origem. Um pivô aqui seria um segundo
/// referencial para uma pergunta que já tinha um.
///
/// ⚠️ **`0` é literal**: `cos_sin_cycles(0)` dá `(1, 0)` ao bit e as duas projecções devolvem
/// `x`/`y` inalterados — mas o ramo literal fica escrito na mesma, pelo zero NEGATIVO
/// (`−0,0 + 0,0` é `+0,0`, e um golden vê a diferença). O mesmo precedente do `motion.bend`.
pub(super) const DIRECTION: &str = "direction";

/// **O AFUNILAMENTO AO LONGO DO ARCO** (doc 89 folha 04 — C4D Spline Wrap ▸ **Size** /
/// **Size Strength**, um *graph* sobre a spline).
///
/// ⚠️ **A célula dizia PARCIAL, e a medição corrigiu QUAL metade falta.** Ela dizia que
/// `motion.scale` + um `field.*` rampa o `size` *"em espaço de MUNDO"*, e que num S os dois
/// divergem. Reconferido contra o código: **uma rampa a MONTANTE não diverge** — o `u` deste nó
/// é `(x − xmin)/divisor`, que é afim no `x` do layout, então uma rampa sobre o `x` antes do
/// embrulho é a MESMA função. O que de facto não é alcançável é a rampa sobre o **`s`**, a
/// posição de ARCO, porque o `s` nasce dentro deste nó (`ArcMap::s_at`) e ninguém o publica.
///
/// ⇒ **O afunilamento corre sobre o `s`**, e a escolha compra duas coisas que a composição não
/// tem:
/// - o `s` **satura** nas pontas (o `s_at` clampa), então o que é empurrado para fora da curva
///   herda a espessura da ponta em vez de continuar a afinar num vazio;
/// - com o `offset` animado, o perfil fica **PREGADO NA CURVA** enquanto o layout desliza por
///   ele — uma região grossa do caminho, por onde as coisas engrossam ao PASSAR. É a leitura
///   da referência (*Spline* Size: a espessura é da spline), e nenhum nó a jusante a exprime,
///   porque a jusante já não há `s`.
///
/// A forma entre as duas pontas é a **mesma família de quatro** que todo `field.*` desta casa
/// oferece (Linear · Quad · Smooth · Smoother) — um artista que aprendeu a curva de uma leu
/// todas. ⛔ **Não é um *graph* arbitrário**: o nó já tem um canal para curva autorada (o
/// `Shape`), e um segundo, com outro widget, para uma pergunta diferente, seria o painel a
/// oferecer duas maneiras de desenhar.
///
/// ⚠️ **O default não multiplica por `1,0` — ele COPIA a coluna** (a lei do `follow_rotation`
/// deste mesmo nó): com `1, 1` o `size` atravessa como qualquer outra coluna, então um stream
/// que nunca teve `size` continua a não ter. Byte-idêntico por ESTRUTURA, não por aritmética —
/// e é a diferença entre *"o nó não mexeu"* e *"o nó mexeu e deu no mesmo"*.
pub(super) const SIZE_TAPER: (&str, &str, &str) = ("size_start", "size_end", "size_profile");

/// A curva de aresta sobre um `s ∈ [0,1]` já clampado — **a MESMA família dos `field.*`**
/// (HR-5, e o espelho verbatim da do `motion.twist`). `0` Linear · `1` Quad · `2` Smooth ·
/// `3` Smoother. Monótona, exacta nos extremos.
fn curve(kind: i32, s: f32) -> f32 {
    match kind {
        1 => s * s,
        2 => s * s * (3.0 - 2.0 * s),
        3 => s * s * s * (s * (s * 6.0 - 15.0) + 10.0),
        _ => s,
    }
}

/// O perfil de espessura ao longo do arco — ver [`SIZE_TAPER`].
#[derive(Clone, Copy)]
pub(super) struct Taper {
    pub(super) start: f32,
    pub(super) end: f32,
    pub(super) profile: i32,
}

impl Taper {
    /// **Nada a fazer** — as duas pontas em `1` ⇒ a coluna `size` é COPIADA, não escrita.
    ///
    /// ⚠️ A pergunta é sobre as PONTAS, nunca sobre o perfil: com `start == end` o perfil não
    /// tem nada que moldar (a curva devolve o mesmo valor nas duas), então mudar a forma sobre
    /// um afunilamento nulo não pode ser motivo para escrever a coluna.
    pub(super) fn is_identity(self) -> bool {
        self.start == 1.0 && self.end == 1.0
    }

    /// O multiplicador de espessura na posição de arco `s`.
    pub(super) fn at(self, s: f32) -> f32 {
        let t = curve(self.profile, s.clamp(0.0, 1.0));
        self.start + (self.end - self.start) * t
    }
}

/// O quadro LOCAL do embrulho — ver [`DIRECTION`]. Devolve `(cos, sin)` do ângulo.
pub(super) fn frame_of(direction_deg: f32) -> (f32, f32) {
    super::trig::cos_sin_cycles(direction_deg / 360.0)
}

/// Leva `p` ao quadro local: o eixo `x` do resultado é o que corre na curva.
pub(super) fn to_local(p: P2, direction_deg: f32, (c, s): (f32, f32)) -> P2 {
    if direction_deg == 0.0 {
        p
    } else {
        [p[0] * c + p[1] * s, -p[0] * s + p[1] * c]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **`0` não mexe no layout, AO BIT** — o caminho por que passa todo grafo já autorado.
    #[test]
    fn a_direction_of_zero_reads_the_layout_untouched() {
        let cs = frame_of(0.0);
        for p in [[1.5_f32, -2.25], [0.0, 0.0], [-0.0, 3.0], [7.0, -0.0]] {
            let q = to_local(p, 0.0, cs);
            assert_eq!(
                (q[0].to_bits(), q[1].to_bits()),
                (p[0].to_bits(), p[1].to_bits()),
                "{p:?} virou {q:?}"
            );
        }
    }

    /// **`90°` troca os eixos** — o `±Y` da referência, alcançado pelo ângulo.
    #[test]
    fn ninety_degrees_puts_the_layouts_y_on_the_curve() {
        let cs = frame_of(90.0);
        let q = to_local([3.0, 5.0], 90.0, cs);
        assert!(
            (q[0] - 5.0).abs() < 1e-3 && (q[1] + 3.0).abs() < 1e-3,
            "o eixo que corre na curva tinha de ser o y: {q:?}"
        );
    }

    /// **As duas pontas em `1` é identidade, seja qual for o perfil** — ver [`Taper::is_identity`].
    #[test]
    fn a_flat_taper_is_identity_under_every_profile() {
        for profile in 0..4 {
            let t = Taper {
                start: 1.0,
                end: 1.0,
                profile,
            };
            assert!(t.is_identity(), "perfil {profile}");
            for k in 0..=10 {
                let s = k as f32 / 10.0;
                assert_eq!(
                    t.at(s).to_bits(),
                    1.0_f32.to_bits(),
                    "perfil {profile}, s={s}"
                );
            }
        }
    }

    /// **O perfil molda o MEIO e nunca as pontas** — a lei da família de quatro.
    #[test]
    fn every_profile_pins_both_ends_and_only_bends_between() {
        for profile in 0..4 {
            let t = Taper {
                start: 2.0,
                end: 0.5,
                profile,
            };
            assert!((t.at(0.0) - 2.0).abs() < 1e-6, "perfil {profile} na origem");
            assert!((t.at(1.0) - 0.5).abs() < 1e-6, "perfil {profile} no fim");
            // E é monótono: uma cauda que afina não pode engrossar no caminho.
            let mut prev = t.at(0.0);
            for k in 1..=20 {
                let v = t.at(k as f32 / 20.0);
                assert!(v <= prev + 1e-6, "perfil {profile} subiu em {k}");
                prev = v;
            }
        }
    }

    /// **Fora de `[0,1]` o perfil SATURA** — é o que faz o que sai da curva herdar a espessura
    /// da ponta em vez de continuar a afinar (ver [`SIZE_TAPER`]).
    #[test]
    fn the_taper_saturates_past_the_ends_of_the_curve() {
        let t = Taper {
            start: 2.0,
            end: 0.5,
            profile: 1,
        };
        assert!((t.at(-3.0) - t.at(0.0)).abs() < 1e-6, "antes do inicio");
        assert!((t.at(4.0) - t.at(1.0)).abs() < 1e-6, "depois do fim");
    }
}

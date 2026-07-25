//! **A superfície de PARÂMETROS de [`PathEffect`]** — o que o painel desenha (`params`) e edita
//! (`get`/`set`). Split de [`crate::effect`] pelo teto de LOC; é `impl PathEffect` continuado,
//! então os três métodos seguem parte da API pública do tipo, apenas noutro arquivo.

use super::{FxParam, PathEffect};

impl PathEffect {
    /// Os parâmetros que este efeito oferece, na ordem em que o painel os desenha.
    ///
    /// ⚠️ Nunca mais de [`super::MAX_FX_PARAMS`] — há gate.
    #[must_use]
    pub fn params(&self) -> &'static [FxParam] {
        /// Um slider de fração `0..=1`, a forma mais comum.
        const fn frac(name: &'static str) -> FxParam {
            FxParam {
                name,
                min: 0.0,
                max: 1.0,
                toggle: false,
                integer: false,
            }
        }
        /// Uma caixinha (o valor só é 0 ou 1).
        const fn flag(name: &'static str) -> FxParam {
            FxParam {
                name,
                min: 0.0,
                max: 1.0,
                toggle: true,
                integer: false,
            }
        }
        const TRIM: &[FxParam] = &[frac("Start"), frac("End"), frac("Offset")];
        const ZIGZAG: &[FxParam] = &[
            // `Size` é PERCENTAGEM da forma: 100 = a média das dimensões dela. A faixa já era
            // `0..100`, mas em unidades de MUNDO — e era isso que a tornava inútil.
            FxParam {
                name: "Size",
                min: 0.0,
                max: 100.0,
                toggle: false,
                integer: false,
            },
            // **128, e o número é MEDIDO** (§0 do CLAUDE.md — um teto diz de que recurso é).
            // O recurso é o tempo de `cooked()`, que é chamado por vários consumidores por
            // frame; o custo é LINEAR nas cristas. Medido em release, círculo de 4 âncoras:
            //
            // | cristas |  8   |  32  |  64  | 128  | 256  |
            // |---------|------|------|------|------|------|
            // | ms/cook |0,019 |0,104 |0,219 |0,475 |0,902 |
            //
            // Enio pediu 128 (2026-07-18) e 128 custa 0,41 ms. Não há parede física antes de
            // ~2000 (o `MAX_SAMPLES` de guarda); quem quiser subir só tem de re-medir.
            FxParam {
                name: "Ridges",
                min: 1.0,
                max: 128.0,
                toggle: false,
                integer: true,
            },
            flag("Smooth"),
            flag("Rough"),
        ];
        /// Um eixo de cópia: quantas, e quanto anda cada uma.
        const fn axis(count: &'static str, mv: &'static str) -> [FxParam; 2] {
            [
                // **128 por eixo, e o número saiu da MEDIÇÃO** (CLAUDE.md §0): o custo de
                // `cooked()` é linear nas cópias, medido numa silhueta de 24 âncoras —
                //
                // | cópias  |  2   |  8   |  16  |  32  |  64  | 128  |
                // |---------|------|------|------|------|------|------|
                // | ms/cook |0,0008|0,0042|0,0086|0,0144|0,0290|0,0614|
                //
                // ⚠️ O cozimento NÃO é o recurso que limita isto — 0,06 ms não limita nada. O
                // custo por medir é o RENDER dos contornos. Há um teto separado no PRODUTO dos
                // dois eixos (`MAX_TOTAL`), porque o teto de um eixo não é o teto de uma grelha.
                FxParam {
                    name: count,
                    min: 1.0,
                    max: 128.0,
                    toggle: false,
                    integer: true,
                },
                // Distâncias em PERCENTAGEM e POR EIXO — o *Relative Offset* do Array do
                // Blender: `100` encaixa sem folga, porque x mede pela LARGURA e y pela ALTURA.
                FxParam {
                    name: mv,
                    min: -200.0,
                    max: 200.0,
                    toggle: false,
                    integer: false,
                },
            ]
        }
        const AX: [FxParam; 2] = axis("Copies X", "Move X");
        const AY: [FxParam; 2] = axis("Copies Y", "Move Y");
        /// Uma rotação por cópia, em graus.
        const fn turn(name: &'static str) -> FxParam {
            FxParam {
                name,
                min: -180.0,
                max: 180.0,
                toggle: false,
                integer: false,
            }
        }
        const REPEAT: &[FxParam] = &[
            AX[0],
            AX[1],
            AY[0],
            AY[1],
            // **Duas rotações, porque fazem coisas diferentes** (Enio, 2026-07-18). O `Spin`
            // roda cada cópia sobre o centro dela; o `Orbit` roda-a em torno do centro do
            // original — é o *Object Offset* do Blender, e é de onde saem as espirais.
            turn("Spin"),
            turn("Orbit"),
        ];
        // Um parametro cada. O Twist entrega o angulo na BORDA da forma; o Bloat e' uma
        // percentagem do raio de cada ponto (`-100` colapsa no centro, `100` duplica).
        const BLOAT: &[FxParam] = &[FxParam {
            name: "Amount",
            min: -100.0,
            max: 100.0,
            toggle: false,
            integer: false,
        }];
        // Os TRÊS sliders do diálogo Warp do Illustrator (o estilo já foi escolhido no Add, não é
        // parâmetro): a DOBRA e as duas distorções de perspectiva, cada uma em `-100..100`. As
        // perspectivas compõem com a dobra — ver [`crate::fx_warp_presets`].
        const fn pct(name: &'static str) -> FxParam {
            FxParam {
                name,
                min: -100.0,
                max: 100.0,
                toggle: false,
                integer: false,
            }
        }
        const WARP: &[FxParam] = &[pct("Bend"), pct("Horizontal"), pct("Vertical")];
        // O ângulo do Twist, em GRAUS, entregue na borda. A faixa `-360..360` (uma volta em cada
        // sentido) não é um teto físico — a matemática não tem cap: é onde a reamostragem
        // (`SAMPLES = 128`) ainda desenha o remoinho liso; além de ~1 volta ela faceta antes de a
        // curva importar. Quem quiser mais voltas re-mede a densidade de amostras.
        const TWIST: &[FxParam] = &[FxParam {
            name: "Angle",
            min: -360.0,
            max: 360.0,
            toggle: false,
            integer: false,
        }];
        match self {
            Self::Trim(_) => TRIM,
            Self::ZigZag(_) => ZIGZAG,
            Self::Repeat(_) => REPEAT,
            Self::Bloat(_) => BLOAT,
            Self::Warp(_) => WARP,
            Self::Twist(_) => TWIST,
            // O Falloff descreve os próprios params (a lista muda com a FORMA), então delega — a
            // porta única que impede o painel e o motor de discordarem sobre o layout por-forma.
            Self::Falloff(f) => f.params(),
        }
    }

    /// O valor do parâmetro `i`, ou `0.0` se ele não existe.
    #[must_use]
    pub fn get(&self, i: usize) -> f64 {
        // O Falloff indexa por FORMA (ver `FalloffSpec::get`) — delega antes do match plano.
        if let Self::Falloff(f) = self {
            return f.get(i);
        }
        match (self, i) {
            (Self::Trim(t), 0) => t.start,
            (Self::Trim(t), 1) => t.end,
            (Self::Trim(t), 2) => t.offset,
            (Self::ZigZag(z), 0) => z.amplitude,
            (Self::ZigZag(z), 1) => z.ridges,
            (Self::ZigZag(z), 2) => f64::from(u8::from(z.smooth)),
            (Self::ZigZag(z), 3) => f64::from(u8::from(z.rough_seed.is_some())),
            (Self::Repeat(r), 0) => r.copies_x,
            (Self::Repeat(r), 1) => r.move_x,
            (Self::Repeat(r), 2) => r.copies_y,
            (Self::Repeat(r), 3) => r.move_y,
            (Self::Repeat(r), 4) => r.spin,
            (Self::Repeat(r), 5) => r.orbit,
            (Self::Bloat(b), 0) => b.amount,
            (Self::Warp(w), 0) => w.bend,
            (Self::Warp(w), 1) => w.h_distort,
            (Self::Warp(w), 2) => w.v_distort,
            (Self::Twist(t), 0) => t.angle,
            _ => 0.0,
        }
    }

    /// Escreve o parâmetro `i`. Índice inexistente é no-op.
    ///
    /// Um parâmetro de CONTAGEM é arredondado **aqui**, na porta única de escrita — assim é o
    /// DOCUMENTO que guarda o inteiro, e o motor, o chip e o slider não podem discordar sobre
    /// que número está em uso. Arredondar só na exibição deixaria o chip a mostrar `37,42`
    /// enquanto a geometria desenha `37`.
    pub fn set(&mut self, i: usize, v: f64) {
        let v = if self.params().get(i).is_some_and(|p| p.integer) {
            v.round()
        } else {
            v
        };
        // O Falloff indexa por FORMA (ver `FalloffSpec::set`) — delega antes do match plano.
        if let Self::Falloff(f) = self {
            f.set(i, v);
            return;
        }
        match (self, i) {
            (Self::Trim(t), 0) => t.start = v,
            (Self::Trim(t), 1) => t.end = v,
            (Self::Trim(t), 2) => t.offset = v,
            (Self::ZigZag(z), 0) => z.amplitude = v,
            (Self::ZigZag(z), 1) => z.ridges = v,
            (Self::ZigZag(z), 2) => z.smooth = v >= 0.5,
            // A seed é FIXA por enquanto: o que o artista liga é o *modo* Roughen. Um knob de
            // seed entra quando alguém quiser duas rugosidades diferentes na mesma cena.
            (Self::ZigZag(z), 3) => z.rough_seed = (v >= 0.5).then_some(1),
            (Self::Repeat(r), 0) => r.copies_x = v,
            (Self::Repeat(r), 1) => r.move_x = v,
            (Self::Repeat(r), 2) => r.copies_y = v,
            (Self::Repeat(r), 3) => r.move_y = v,
            (Self::Repeat(r), 4) => r.spin = v,
            (Self::Repeat(r), 5) => r.orbit = v,
            (Self::Bloat(b), 0) => b.amount = v,
            (Self::Warp(w), 0) => w.bend = v,
            (Self::Warp(w), 1) => w.h_distort = v,
            (Self::Warp(w), 2) => w.v_distort = v,
            (Self::Twist(t), 0) => t.angle = v,
            _ => {}
        }
    }
}

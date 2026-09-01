//! **OS NÚMEROS DO PAINEL, lidos uma vez** — irmão de [`super`] pelo tecto de LOC (HR-18), e o
//! corte é por responsabilidade: lá fica *o que o nó faz*, aqui *o que ele lê antes de o fazer*.

use crate::{GEOMETRY_BRANCHES, MODE_GRAMMAR, param, shape};
use ph2d_nodegraph::cook::EvalCtx;

/// Os dez números do painel, lidos uma vez.
#[derive(Clone, Copy)]
pub(crate) struct Params {
    pub(crate) generations: f32,
    pub(crate) angle: f32,
    pub(crate) step: f32,
    pub(crate) width: f32,
    pub(crate) width_scale: f32,
    pub(crate) length_scale: f32,
    pub(crate) root_angle: f32,
    pub(crate) tropism: f32,
    pub(crate) tropism_angle: f32,
    pub(crate) seed: f32,
    pub(crate) orient: f32,
    pub(crate) mode: f32,
    pub(crate) branches: f32,
    pub(crate) segments: f32,
    pub(crate) variation: f32,
    pub(crate) bend: f32,
    pub(crate) continuous_length: f32,
    pub(crate) continuous_angle: f32,
    pub(crate) step_scale: f32,
    pub(crate) growth: f32,
    pub(crate) geometry: f32,
    pub(crate) tip_taper: f32,
    /// ⭐ O primeiro NÍVEL de ramo que ganha folha — ver [`param::LEAF_FIRST_LEVEL`].
    pub(crate) leaf_first_level: f32,
    /// A viragem acrescentada à direcção do ramo, em graus.
    pub(crate) leaf_angle: f32,
    /// A abertura aleatória à volta dela, em graus (`±spread/2`).
    pub(crate) leaf_spread: f32,
    /// A fracção desenhada à frente dos galhos — lida pela SHELL, não pela tartaruga.
    pub(crate) leaf_front: f32,
    /// `0` = os efeitos a jusante não alcançam a folha.
    pub(crate) leaf_effects: f32,
}

impl Params {
    /// **Os sliders mandam?** — a pergunta que decide de onde vem a gramática.
    pub(crate) fn guided(&self) -> bool {
        self.mode.round() as i32 != MODE_GRAMMAR
    }

    /// Os números de forma, na cara que o [`shape`] pede.
    pub(crate) fn shape(&self) -> shape::Shape {
        shape::Shape {
            branches: self.branches,
            segments: self.segments,
            variation: self.variation,
            bend: self.bend,
        }
    }

    pub(crate) fn read(ctx: &EvalCtx<'_>) -> Self {
        Self::read_with(|n| ctx.param(n))
    }

    /// **A MESMA leitura, por GETTER** — a porta que a shell usa para construir as fitas
    /// com exactamente os números que o nó vai cozinhar.
    ///
    /// ⚠️ Extraída em vez de copiada: a shell tem de resolver a escada inteira
    /// (conduzido → override → default) e uma segunda leitura ao lado desta poria a
    /// geometria num conjunto de números e o cozimento noutro — *uma lei escrita duas vezes
    /// ainda não é uma lei*. É o mesmo movimento que o `source.shape` já pagou
    /// (`ShapeParams::read` sobre a mesma escada).
    pub(crate) fn read_with(get: impl Fn(&str) -> f32) -> Self {
        Self {
            generations: get(param::GENERATIONS),
            angle: get(param::ANGLE),
            step: get(param::STEP),
            width: get(param::WIDTH),
            width_scale: get(param::WIDTH_SCALE),
            length_scale: get(param::LENGTH_SCALE),
            root_angle: get(param::ROOT_ANGLE),
            tropism: get(param::TROPISM),
            tropism_angle: get(param::TROPISM_ANGLE),
            seed: get(param::SEED),
            orient: get(param::ORIENT),
            mode: get(param::MODE),
            branches: get(param::BRANCHES),
            segments: get(param::SEGMENTS),
            variation: get(param::VARIATION),
            bend: get(param::BEND),
            continuous_length: get(param::CONTINUOUS_LENGTH),
            continuous_angle: get(param::CONTINUOUS_ANGLE),
            step_scale: get(param::STEP_SCALE),
            growth: get(param::GROWTH),
            geometry: get(param::GEOMETRY),
            tip_taper: get(param::TIP_TAPER),
            leaf_first_level: get(param::LEAF_FIRST_LEVEL),
            leaf_angle: get(param::LEAF_ANGLE),
            leaf_spread: get(param::LEAF_SPREAD),
            leaf_front: get(param::LEAF_FRONT),
            leaf_effects: get(param::LEAF_EFFECTS),
        }
    }

    /// ⭐⭐⭐ **A PORTA DOS NÚMEROS** — o que entra estranho sai utilizável (doc 96 §3.4).
    ///
    /// ⚠️ **Um `f32` de painel é coagido; um `f32` de FIO não é.** O `EvalCtx::param` entrega o
    /// que o nó a montante produziu, `F(s/0)` é uma gramática que o parser ACEITA, e o `eval`
    /// de um `NodeOp` **não pode devolver `Result`**. Medido antes desta função
    /// (`examples/probe_finite.rs`, molde `Tree` a `g = 4`): **8 de 23** knobs punham `NaN`/`inf`
    /// na corrente, `112` valores num só.
    ///
    /// Duas leis, e a segunda não é a primeira:
    ///
    /// 1. **Não-finito ⇒ o DEFAULT do manifesto.** ⛔ Não `0`: num `step` isso é uma planta de
    ///    tamanho zero, e num `width` uma sem espessura — *degradar para um número definido não
    ///    é degradar para o número neutro*. O default é o que o artista vê no painel.
    /// 2. **Um ângulo é reduzido MÓDULO A VOLTA.** ⚠️ A linha que ensina é `angle = 1e30`, que é
    ///    **finito** e mesmo assim produzia `56` não-finitos: o heading acumula até `inf` e a
    ///    conversão para direcção devolve `NaN`. *Guardar contra não-finitos não chega quando a
    ///    grandeza é periódica* — e em `f32` acima de `2^24` a soma de `30°` já nem se vê.
    ///
    /// ⚠️ **Chamada no [`crate::build`]**, que é a porta ÚNICA do nó: o `eval` e a bancada de
    /// sonda entram os dois por lá, e uma cura em `read_with` deixaria a sonda a medir outro
    /// programa ([`crate::probe::probe_params`] constrói a struct à mão).
    pub(crate) fn sanitized(&self) -> Self {
        use ph2d_nodegraph::node::ParamSpec;
        let default_of = |name: &str| {
            crate::MANIFEST
                .params
                .iter()
                .find(|p: &&ParamSpec| p.name == name)
                .map_or(0.0, |p| p.default)
        };
        // ⚠️ **A FAIXA DECLARADA é a do painel**, e a lei é a mesma do `generation_plan`: *o fio
        // não conduz o que a caixa recusa*. A `max` do hint é o tecto arrastável; quando existe
        // um [`crate::ui::PARAM_HARD_MAX`] (hoje só o `generations`) é ele que manda, porque é o
        // que a caixa de facto aceita digitar.
        //
        // ⛔ **Não é um tecto inventado** (§0.0): é o número que o painel já escreve. Ele existe
        // aqui porque um valor **finito** também parte a conta — medido, `width_scale = 1e30`
        // punha `48` não-finitos na corrente e `tropism = 1e30` punha `80`, porque os dois
        // COMPÕEM ao longo das gerações e o produto estoura o `f32`.
        let range_of = |name: &str| {
            crate::ui::PARAM_HINTS
                .iter()
                .find(|h| h.param == name)
                .map(|h| (h.min, h.max))
        };
        let hard_max_of = |name: &str| {
            crate::ui::PARAM_HARD_MAX
                .iter()
                .find(|l| l.param == name)
                .map(|l| l.max)
        };
        let mut out = *self;
        let fix = |v: &mut f32, name: &str| {
            if !v.is_finite() {
                *v = default_of(name);
                return;
            }
            if let Some((lo, hi)) = range_of(name) {
                *v = v.clamp(lo, hard_max_of(name).unwrap_or(hi));
            }
        };
        // ⛔ **O `generations` NÃO entra aqui, e a ausência é a DECISÃO.** O
        // [`crate::generation_plan`] já responde a um não-finito com uma cadeia **vazia**, e há
        // gate a exigi-lo (`untrusted_generations_never_costs_and_never_empties`): substituí-lo
        // pelo default faria um `NaN` **desenhar uma planta**, que é o contrário do que aquele
        // gate defende. O tecto dele vive no `generation_plan`, com a medição ao lado.
        fix(&mut out.angle, param::ANGLE);
        fix(&mut out.step, param::STEP);
        fix(&mut out.width, param::WIDTH);
        fix(&mut out.width_scale, param::WIDTH_SCALE);
        fix(&mut out.length_scale, param::LENGTH_SCALE);
        fix(&mut out.root_angle, param::ROOT_ANGLE);
        fix(&mut out.tropism, param::TROPISM);
        fix(&mut out.tropism_angle, param::TROPISM_ANGLE);
        fix(&mut out.seed, param::SEED);
        fix(&mut out.orient, param::ORIENT);
        fix(&mut out.mode, param::MODE);
        fix(&mut out.branches, param::BRANCHES);
        fix(&mut out.segments, param::SEGMENTS);
        fix(&mut out.variation, param::VARIATION);
        fix(&mut out.bend, param::BEND);
        fix(&mut out.continuous_length, param::CONTINUOUS_LENGTH);
        fix(&mut out.continuous_angle, param::CONTINUOUS_ANGLE);
        fix(&mut out.step_scale, param::STEP_SCALE);
        fix(&mut out.growth, param::GROWTH);
        fix(&mut out.geometry, param::GEOMETRY);
        fix(&mut out.tip_taper, param::TIP_TAPER);
        fix(&mut out.leaf_first_level, param::LEAF_FIRST_LEVEL);
        fix(&mut out.leaf_angle, param::LEAF_ANGLE);
        fix(&mut out.leaf_spread, param::LEAF_SPREAD);
        fix(&mut out.leaf_front, param::LEAF_FRONT);
        fix(&mut out.leaf_effects, param::LEAF_EFFECTS);
        // ⛔⛔ **A DOBRA MÓDULO A VOLTA foi construída aqui e RETIRADA, por mutação.**
        //
        // A 1.ª redacção reduzia os cinco ângulos com `% 360`, contra o caso `angle = 1e30`
        // (finito, e mesmo assim `56` não-finitos na corrente). ⚠️ **A mutação que a apagava
        // SOBREVIVEU**: com a faixa declarada a valer para o fio, os cinco já entram dentro de
        // uma volta (`ANGLE [0, 180]` · `LEAF_ANGLE [−180, 180]` · `LEAF_SPREAD [0, 180]` ·
        // `ROOT_ANGLE` e `TROPISM_ANGLE [−180, 360]`) e a dobra não tem sujeito.
        //
        // ⛔ E ela não era só inerte, era **arriscada**: `root_angle = 360` — uma posição
        // legítima do slider — dobraria para `0`, mudando os bits de uma planta que ninguém
        // tocou. *Código morto que ainda pode escrever é pior que código morto.*
        //
        // ⚠️ Se um param angular novo declarar faixa mais larga que uma volta, a dobra volta —
        // **com um gate que lhe dê sujeito**, que é o que faltou desta vez.
        out
    }

    /// O valor de um param pelo NOME — a ponte que deixa uma expressão da gramática ler o
    /// painel (`F(step*0.5)`). Um nome desconhecido é `0`, como em toda expressão da casa.
    pub(crate) fn by_name(&self, n: &str) -> f32 {
        match n {
            param::GENERATIONS => self.generations,
            param::ANGLE => self.angle,
            param::STEP => self.step,
            param::WIDTH => self.width,
            param::WIDTH_SCALE => self.width_scale,
            param::LENGTH_SCALE => self.length_scale,
            param::ROOT_ANGLE => self.root_angle,
            param::TROPISM => self.tropism,
            param::TROPISM_ANGLE => self.tropism_angle,
            param::SEED => self.seed,
            param::ORIENT => self.orient,
            param::MODE => self.mode,
            param::BRANCHES => self.branches,
            param::SEGMENTS => self.segments,
            param::VARIATION => self.variation,
            param::BEND => self.bend,
            param::CONTINUOUS_LENGTH => self.continuous_length,
            param::CONTINUOUS_ANGLE => self.continuous_angle,
            param::STEP_SCALE => self.step_scale,
            param::GROWTH => self.growth,
            param::GEOMETRY => self.geometry,
            param::TIP_TAPER => self.tip_taper,
            _ => 0.0,
        }
    }

    /// **Desenha em FITAS?** — a pergunta que decide de onde vem a geometria.
    pub(crate) fn ribbons(&self) -> bool {
        self.geometry.round() as i32 == GEOMETRY_BRANCHES
    }
}

//! ⭐⭐ **A VIAGEM ENTRE VISTAS** — a câmera vai suavemente, em vez de saltar (W51).
//!
//! > Enio, 2026-08-23: *"falta um Lerp() rápido para mudança suave das views como no blender."*
//!
//! # ⚠️ A curva e a duração NÃO são minhas
//!
//! A casa tem um sistema de movimento por **mola**, com carácter (`Discrete`/`Expressive`), com
//! *reduced motion*, e com papéis que dizem o que uma coisa **é**
//! ([`ph2d_editor::motion::Role`]). Inventar aqui uma duração e uma curva seria uma segunda ideia de
//! *como as coisas se mexem neste app* — a doença que este módulo já nomeou meia dúzia de vezes.
//!
//! ⭐ **O papel é [`Role::Surface`]**, e o doc dele descreve este caso à letra: *"viaja (o reduced
//! motion mata-a) e **nunca ultrapassa**, nos DOIS carácteres… uma roda nomeia um **destino**, e
//! passar dele e voltar não lê como peso — lê como a régua a mentir"*. **Uma vista nomeada é um
//! destino.** Com `Role::Travel` (que ultrapassa no Expressivo) a peça passaria da frente e voltava
//! — a janela inteira a balançar.
//!
//! ⚠️ E o *reduced motion* sai de graça: ali a lei é `None`, o progresso chega a `1` no primeiro
//! quadro, e a viagem vira o salto que existia antes desta wave. *Uma animação que ignora essa
//! preferência é um defeito de acessibilidade, e a casa já tem a preferência.*
//!
//! # ⭐ Chegar EXATAMENTE ao destino é uma exigência, não um detalhe
//!
//! O chip da vista acende por [`crate::field3d_views::named_view`], que reconhece a orientação com
//! uma barra de **0,16°**. Uma viagem que se aproximasse assintoticamente pousaria *perto* e o botão
//! nunca acenderia. O sistema da casa já resolve isto — *"assentar põe o valor EXACTO e larga o
//! voo"* — e esta metade honra-o: em `t >= 1` a câmera é o destino, escrito, não interpolado.

use ph2d_field_render::Orbit;

/// ⭐⭐ **O PAPEL desta viagem** — e é ele que decide a curva, a duração e o *reduced motion*.
///
/// ⚠️ **Nomeado aqui, e não no laço de quadro**, para um gate o poder alcançar: a alegação que
/// interessa ao artista é *"a viagem acontece mesmo com o movimento reduzido ligado"*, e ela só é
/// gateável se o papel tiver um nome deste lado.
///
/// ⚠️ **Mudou na W52.** Ele era [`Role::Surface`] — que morre no *reduced motion* —, e o smoke da
/// W51 leu como *"não funcionou, está como antes"*: a preferência do Enio estava ligada, e o código
/// fazia exatamente o que ela manda. Decisão dele, com os dois comportamentos vistos: *"o lerp não
/// deve estar vinculado ao Reduced Motion. Mas deve ser o único modo."*
///
/// ⭐ O [`Role::Viewpoint`] existe por causa disto, e o critério dele é estreito: *o que substitui
/// esta animação é um CORTE que desorienta mais do que ela*. Ver o doc do papel.
pub(crate) const ROLE: ph2d_editor::motion::Role = ph2d_editor::motion::Role::Viewpoint;

/// **Uma viagem em curso**: de onde, para onde.
///
/// ⚠️ O `from` é **congelado na partida**. Interpolar a partir da câmera de agora faria cada quadro
/// medir contra o resultado do anterior — o gesto a perseguir a própria cauda, que é a nota que o
/// `Smoke::drag_grip` já carrega para o arrasto do gizmo.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Flight {
    pub(crate) from: Orbit,
    pub(crate) to: Orbit,
}

impl Flight {
    /// ⭐ **A câmera em `t`** — `0` na partida, `1` no destino.
    ///
    /// | grandeza | como se interpola | porquê |
    /// |---|---|---|
    /// | orientação | **slerp**, pelo caminho curto | é uma rotação; interpolar componentes daria uma trajetória que acelera e desacelera sozinha |
    /// | alvo | linear | é um ponto |
    /// | enquadramento | **geométrico** | o zoom é multiplicativo neste módulo (`ZOOM_PER_STEP`), *"para que cada passo aproxime a mesma **fração**"* — linear daria uma aproximação que corre depressa longe e para perto |
    ///
    /// ⚠️ **A lente vem do destino desde o primeiro quadro:** ela não é uma grandeza contínua
    /// (convergente ou paralela), e não há meia-lente.
    pub(crate) fn at(self, t: f32) -> Orbit {
        if t >= 1.0 {
            return self.to;
        }
        let t = t.max(0.0);
        Orbit {
            rotation: slerp(self.from.rotation, self.to.rotation, t),
            half_extent: geometric(self.from.half_extent, self.to.half_extent, t),
            target: [0, 1, 2]
                .map(|i| (self.to.target[i] - self.from.target[i]).mul_add(t, self.from.target[i])),
            lens: self.to.lens,
        }
    }
}

/// ⚠️ **Pelo caminho CURTO.** `q` e `−q` são a mesma orientação, e sem o `dot < 0` metade das
/// viagens dá a volta pelo lado comprido — a peça gira 300° para chegar a um sítio a 60°. É a irmã
/// da nota do [`crate::field3d_views::named_view`], que já paga o mesmo sinal.
fn slerp(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    let mut d = dot(a, b);
    let b = if d < 0.0 {
        d = -d;
        [-b[0], -b[1], -b[2], -b[3]]
    } else {
        b
    };
    // Quase paralelos: o `sin` do denominador colapsa e a conta explode. Aí a reta **é** o arco.
    let (s0, s1) = if d > 0.9995 {
        (1.0 - t, t)
    } else {
        let theta = d.clamp(-1.0, 1.0).acos();
        let sin = theta.sin();
        (((1.0 - t) * theta).sin() / sin, (t * theta).sin() / sin)
    };
    let q = [
        s0.mul_add(a[0], s1 * b[0]),
        s0.mul_add(a[1], s1 * b[1]),
        s0.mul_add(a[2], s1 * b[2]),
        s0.mul_add(a[3], s1 * b[3]),
    ];
    let n = dot(q, q).sqrt();
    if n > f32::EPSILON {
        q.map(|c| c / n)
    } else {
        b
    }
}

/// Interpolação **geométrica** — ver a tabela do [`Flight::at`].
fn geometric(a: f32, b: f32, t: f32) -> f32 {
    let (a, b) = (a.max(f32::EPSILON), b.max(f32::EPSILON));
    a * (b / a).powf(t)
}

fn dot(a: [f32; 4], b: [f32; 4]) -> f32 {
    a[0].mul_add(b[0], a[1].mul_add(b[1], a[2].mul_add(b[2], a[3] * b[3])))
}

#[cfg(test)]
#[path = "field3d_flight_tests.rs"]
mod tests;

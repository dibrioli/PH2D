//! **OS VERBOS QUE PUXAM** — o barro que vem com o dedo.
//!
//! Módulo FILHO de [`super`] (`#[path]`), e o corte é entre *o que a cena É* (lá: a lista, a
//! câmera, o passe) e *o que um gesto de PUXÃO faz com ela* (aqui). São os quatro da W5 — Grab,
//! Snake Hook, Twist, Local Scale — mais as duas portas que os três primeiros compartilham: o
//! ponto que o dedo segura ([`Sculpt3dScene::take_hold`]) e para onde ele o levou
//! ([`Sculpt3dScene::finger_world`]).
//!
//! ⚠️ Eles moram juntos porque obedecem a uma LEI que os verbos de carimbo não obedecem: a pegada
//! é presa no pen-down e o alvo é função do puxão TOTAL, não da soma dos passos. Espalhá-los pelo
//! arquivo da cena foi o que aproximou os dois do teto de LOC ao mesmo tempo.

use ph2d_sculpt3d::{Amount, Dab};

use super::{SCALE_PER_PX, Sculpt3dScene, TWIST_DEADZONE_PX, TwistSweep};

/// A direção unitária de `center` até o pixel — `None` dentro da zona morta.
///
/// ⚠️ **Porta única da zona morta**, e ela tem dois consumidores desde que a
/// varredura passou a ser armada no pen-down: quem ARMA e quem ACUMULA têm de
/// concordar sobre *quando não há direção*, senão um gesto que começa em cima do
/// pivô arma um `last` que o outro lado consideraria inexistente.
fn sweep_dir(center: (f32, f32), x: f32, y: f32) -> Option<[f32; 2]> {
    let d = [x - center.0, center.1 - y];
    let len = (d[0] * d[0] + d[1] * d[1]).sqrt();
    (len >= TWIST_DEADZONE_PX).then(|| [d[0] / len, d[1] / len])
}

impl Sculpt3dScene {
    /// **PEGA o barro** — o pen-down dos dois grips que puxam. Devolve `false`
    /// se o raio errou a malha (e aí o botão vira órbita, como em todo gesto).
    ///
    /// ⚠️ **Nem o Grab nem o Snake Hook re-picam depois disto**, e o Hook é o
    /// caso surpreendente: ele arrasta uma ESFERA pelo espaço, não um acerto de
    /// superfície (`Drag.js:59` consulta `pickVerticesInSphere` num centro
    /// deslocado, sem raycast, e o `makeStroke` dele devolve `true` sempre).
    /// Sair do modelo no meio do gesto não interrompe um espinho — é assim que
    /// se puxa uma ponta para fora da silhueta.
    pub(super) fn take_hold(&mut self, x: f32, y: f32) -> bool {
        // ⚠️ **Na peça ATIVA, e ela já foi escolhida** — pelo `aim` do pen-down,
        // antes de o traço começar. Repicar aqui trocaria o objeto no meio de um
        // gesto cujos planos por-vértice já estão dimensionados na malha antiga.
        let Some(hit) = self.pick_active(x, y) else {
            return false;
        };
        // ⚠️ **A âncora é guardada em MUNDO**, e não em local, porque quem a
        // consome é a câmera (`screen_delta_to_world`): o dedo anda na TELA. A
        // descida para o espaço da malha acontece no dab, uma vez, na porta.
        self.grab = Some((self.pose().point_to_world(hit.point), (x, y)));
        true
    }

    /// Onde o dedo está, em MUNDO, na profundidade da pegada.
    ///
    /// Porta única dos dois grips que puxam, e é ela que os liga: o
    /// [`Grip::Hold`] pede o vetor da âncora até aqui (o puxão TOTAL) e o
    /// [`Grip::Hook`] pede a diferença entre dois destes (o INCREMENTO). Duas
    /// aritméticas para *"onde o dedo está"* divergiriam no dia em que uma delas
    /// ganhasse a perspectiva e a outra não.
    fn finger_world(&self, at: [f32; 3], from: (f32, f32), x: f32, y: f32) -> [f32; 3] {
        let d = self
            .camera
            .screen_delta_to_world(at, x - from.0, y - from.1, self.viewport);
        [at[0] + d[0], at[1] + d[1], at[2] + d[2]]
    }

    /// **O gesto de quem SEGURA** ([`Grip::Hold`]): a pegada fica onde foi
    /// presa e o que cresce é o puxão.
    pub(super) fn grab_at(&mut self, x: f32, y: f32) {
        let Some((at, from)) = self.grab else {
            return;
        };
        let f = self.finger_world(at, from, x, y);
        let pull = [f[0] - at[0], f[1] - at[1], f[2] - at[2]];
        // A âncora e o puxão descem juntos: o CENTRO é um ponto, o puxão é um
        // deslocamento, e a [`Pose`] os trata diferente de propósito.
        let center = self.pose().point_to_local(at);
        let pull = self.pose().vector_to_local(pull);
        let brush = self.armed_brush(center);
        let eye = self.dir_to_local(self.ray_at(x, y).dir());
        self.stroke.dab(
            self.objects[self.active].stack.mesh_mut(),
            &brush,
            &Dab::pulling(center, brush.radius, eye, pull),
            self.symmetry,
        );
        Self::mesh_changed(
            &mut self.objects[self.active].dirty,
            &mut self.edits,
            self.stroke.last_gpu_dirty(),
        );
    }

    /// **Um passo de quem ARRASTA** ([`Grip::Hook`]): a pegada anda de `from`
    /// até `to` (pixels) e o dab recebe o INCREMENTO daquele trecho.
    ///
    /// ⚠️ **Os dois centros saem da ÂNCORA, não um do outro.** Somar
    /// incrementos passo a passo acumularia o erro de cada conversão ao longo
    /// de um arrasto inteiro, e a pegada iria escorregando para longe do
    /// cursor. Derivando os dois da âncora, o incremento é a diferença de dois
    /// absolutos e o centro está sempre onde o dedo está.
    pub(super) fn hook_step(&mut self, from: [f32; 2], to: [f32; 2]) {
        let Some((at, origin)) = self.grab else {
            return;
        };
        let c0 = self.finger_world(at, origin, from[0], from[1]);
        let c1 = self.finger_world(at, origin, to[0], to[1]);
        let step = [c1[0] - c0[0], c1[1] - c0[1], c1[2] - c0[2]];
        let center = self.pose().point_to_local(c1);
        let step = self.pose().vector_to_local(step);
        let brush = self.armed_brush(center);
        let eye = self.dir_to_local(self.ray_at(to[0], to[1]).dir());
        self.stroke.dab(
            self.objects[self.active].stack.mesh_mut(),
            &brush,
            &Dab::hooking(center, brush.radius, eye, step),
            self.symmetry,
        );
        Self::mesh_changed(
            &mut self.objects[self.active].dirty,
            &mut self.edits,
            self.stroke.last_gpu_dirty(),
        );
    }

    /// **O ângulo varrido em torno de `center`**, do pen-down até aqui.
    ///
    /// `None` quando o gesto ainda não começou: dentro da zona morta e com nada
    /// varrido, não há torção a aplicar — e um dab de ângulo zero pagaria a
    /// pegada inteira (consulta, refit, upload) para não mover um vértice.
    ///
    /// ⚠️ **O CENTRO é parâmetro, e é ele que decide o que o gesto significa.**
    /// O [`Grip::Turn`] gira em torno da ÂNCORA — cujo pixel *é* o pen-down, e
    /// por isso os dois eram a mesma coisa enquanto este era o único chamador.
    /// O transform da máscara gira em torno do **PIVÔ**, que é o centroide
    /// ponderado do que se move e quase nunca cai debaixo do dedo: medir a
    /// varredura a partir do pen-down fazia o dedo dar uma volta inteira em
    /// torno do pivô e a peça girar **meia** (teorema do ângulo inscrito), com
    /// a razão a variar conforme o percurso — medido em
    /// `sculpt3d_transform_tests`, 0,31× a 0,45× para o mesmo gesto em três
    /// amplitudes. *Uma razão que muda com a amplitude é o que o artista sente
    /// como "impreciso".*
    ///
    /// ⚠️ **O sinal é o da TELA, e a conversão mora aqui:** `y` de janela cresce
    /// para BAIXO e o eixo `up` da câmera aponta para CIMA, então a componente
    /// vertical entra NEGADA — a mesma negação que o `screen_delta_to_world`
    /// faz, e pelo mesmo motivo. Com ela, varrer no sentido anti-horário na tela
    /// dá ângulo positivo em torno do eixo que aponta para o observador, que é
    /// o que o artista vê o barro fazer.
    pub(super) fn swept_angle_about(&mut self, center: (f32, f32), x: f32, y: f32) -> Option<f32> {
        let dir = sweep_dir(center, x, y);
        let g = self.twist.get_or_insert(TwistSweep {
            last: None,
            total: 0.0,
        });
        let Some(dir) = dir else {
            // O total FICA: o barro não desfaz a torção porque o dedo passou
            // perto do pivô. O que se perde é a referência de direção.
            g.last = None;
            return if g.total == 0.0 { None } else { Some(g.total) };
        };
        if let Some(p) = g.last {
            g.total += (p[0] * dir[1] - p[1] * dir[0]).atan2(p[0] * dir[0] + p[1] * dir[1]);
        }
        g.last = Some(dir);
        Some(g.total)
    }

    /// **ARMA a varredura no PEN-DOWN** — a referência de direção nasce onde o
    /// dedo desceu, não onde ele foi visto a mover-se pela primeira vez.
    ///
    /// ⚠️ **Sem ela o gesto perde o PRIMEIRO incremento, e nunca o recupera.**
    /// A acumulação só soma quando já existe um `last`, então o trecho entre o
    /// pen-down e o primeiro `move` some — um deslocamento CONSTANTE que o
    /// artista lê como *"o barro não acompanha a mão"*. Medido antes de existir
    /// (`sculpt3d_transform_tests`, amostrando de 5° em 5°): 25° de giro para
    /// 30° de dedo, 85° para 90°, 175° para 180° — o déficit é exatamente um
    /// passo, sempre.
    ///
    /// ⚠️ **E ela NÃO serve ao [`Grip::Turn`], que é por isso que ele continua
    /// a perder o primeiro passo:** lá o centro do gesto é a âncora, cujo pixel
    /// *é* o pen-down — a direção do centro até o dedo no instante zero é o
    /// vetor nulo, e não existe. Só um gesto que gira em torno de um ponto
    /// LONGE do dedo tem essa referência para armar.
    pub(super) fn arm_sweep_at(&mut self, center: (f32, f32), x: f32, y: f32) {
        self.twist = Some(TwistSweep {
            last: sweep_dir(center, x, y),
            total: 0.0,
        });
    }

    /// **O gesto de quem GIRA** ([`Grip::Turn`]): a âncora fica onde foi presa e
    /// o que cresce é o ângulo varrido (ou a fração de escala).
    ///
    /// ⚠️ **O [`Amount`] chega por PARÂMETRO, e não é cerimônia:** ele vem do
    /// `match` sobre o grip que o chamador já faz, então as duas espécies de
    /// gesto são exaustivas aqui dentro — um terceiro `Amount` **não compila**
    /// até alguém dizer que gesto o produz. Derivá-lo do verbo aqui abriria a
    /// segunda porta para a mesma pergunta.
    ///
    /// ⚠️ **O EIXO é o raio que PEGOU o barro, não o raio de agora.** O cursor
    /// varre um círculo em volta da âncora, então o raio que passa por ele
    /// aponta para uma direção que muda alguns graus ao longo da varredura — e a
    /// torção passaria a ser em torno de um eixo que bamboleia. O original
    /// congela o dele no `startSculpt` (`Twist.js:31`) pelo mesmo motivo. Aqui
    /// ele nem precisa ser guardado: a câmera não se mexe durante um arrasto de
    /// escultura, então o raio do pixel de pen-down **é** o eixo congelado.
    pub(super) fn turn_at(&mut self, kind: Amount, x: f32, y: f32) {
        let Some((at, from)) = self.grab else {
            return;
        };
        let center = self.pose().point_to_local(at);
        let brush = self.armed_brush(center);
        let eye = self.dir_to_local(self.ray_at(from.0, from.1).dir());
        let dab = match kind {
            Amount::Angle => {
                // O centro é a ÂNCORA, e o pixel dela é o próprio pen-down —
                // ver o doc de [`Self::swept_angle_about`].
                let Some(radians) = self.swept_angle_about(from, x, y) else {
                    return;
                };
                Dab::turning(center, brush.radius, eye, radians)
            }
            // Arrastar para a DIREITA cresce. É o `mouseX - lastMouseX` do
            // original, com o total no lugar do incremento. ⚠️ A fração é
            // ADIMENSIONAL, então ela NÃO atravessa a pose — descê-la seria o
            // erro simétrico de esquecer a descida de um deslocamento.
            Amount::Fraction => {
                Dab::scaling(center, brush.radius, eye, (x - from.0) * SCALE_PER_PX)
            }
        };
        self.stroke.dab(
            self.objects[self.active].stack.mesh_mut(),
            &brush,
            &dab,
            self.symmetry,
        );
        Self::mesh_changed(
            &mut self.objects[self.active].dirty,
            &mut self.edits,
            self.stroke.last_gpu_dirty(),
        );
    }
}

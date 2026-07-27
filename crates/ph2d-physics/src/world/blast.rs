//! **A EXPLOSÃO e o CAMPO DE ATRAÇÃO** (W-Hand) — as duas ferramentas de
//! interação que não seguram nada: elas empurram o que está por perto.
//!
//! A mão ([`super::grab`]) precisa de um CORPO sob o cursor. Estas duas precisam
//! só de um PONTO, e é essa diferença que decide tudo o resto — quem despacha o
//! gesto, o que o overlay desenha, e por que a família tem uma seção própria no
//! painel em vez de ser mais um knob da mão.
//!
//! # Duas leis, e a diferença não é o sinal
//!
//! - **Explosão** = um **IMPULSO**, uma vez. `N·s`, resistido pela massa, e
//!   acabou no mesmo tick — é um evento.
//! - **Atração** = uma **FORÇA**, sustentada enquanto o botão está apertado.
//!   `N`, aplicada por sub-passo como o `drag` e os efetores de zona. Negativa
//!   REPELE, e isso não duplica a explosão: um empurrão contínuo e um estalo são
//!   coisas diferentes (o primeiro segura um corpo no ar, o segundo o arremessa).
//!
//! Por isso a atração é **estado da sessão** (mora no mundo, aplicada pelo
//! `step`, exatamente como a mão) e a explosão é uma **chamada**.
//!
//! # A régua é UMA
//!
//! As duas pesam pela MESMA [`blast_falloff`] — linear, `1` no centro, **zero
//! exatamente na borda**. Chegar a zero na borda é o que evita o degrau que o
//! W-AreaFalloff descreve (um corpo que atravessa a fronteira passando de força
//! cheia a nada dentro de um sub-passo), e ter uma função para as duas é o que
//! impede a explosão e a atração de discordarem sobre onde o alcance termina —
//! elas compartilham o ANEL que o overlay desenha.
//!
//! # Determinismo
//!
//! São entradas NÃO-REPRODUZÍVEIS, como a mão: não estão no documento e não
//! estarão. As mesmas duas regras valem, e o dono delas é a ponte
//! (`bridge::grab`): pegar/explodir/atrair **descarta o ring de checkpoints** e
//! nada é gravado enquanto uma atração está em voo; um rewind **solta**.
//!
//! ⚠️ Cada corpo recebe um impulso **independente** dos outros, então a ordem de
//! iteração não entra em nenhuma soma de `f32` — o que dispensa a tabela
//! ordenada por handle que os efetores de zona precisam (HR-5).

use rapier2d::dynamics::{RigidBodySet, RigidBodyType};
use rapier2d::na::Vector2;

use super::PhysicsWorld;

/// Um campo de atração em voo: onde, até onde, e com que força.
///
/// `force` negativa REPELE. Copy porque é três números — o mundo guarda um
/// `Option` dele, do mesmo jeito que guarda a mão.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Attract {
    /// O cursor, em mundo.
    pub center: [f32; 2],
    /// Onde o peso chega a zero.
    pub radius: f32,
    /// Newtons no CENTRO (o falloff pesa daí para fora). Negativo = repele.
    pub force: f32,
    /// **Resistência dentro do campo** — sem ela isto não é uma ferramenta.
    ///
    /// ⚠️ MEDIDO, e é a mesma descoberta que o **W-AreaDrag** fez do outro lado
    /// da cerca (*"zona com força e sem resistência é um vácuo que sopra"*): uma
    /// atração puramente conservativa é um **oscilador harmônico**, então o corpo
    /// atravessa o foco e volta para sempre. A tabela em
    /// [`PhysicsWorld::ATTRACT_DAMPING`] mostra a distância final não-monotônica
    /// em `damping = 0` (força 10 → 0,68 m, força 20 → 1,61 m, força 100 →
    /// 2,47 m: mais força chegando MAIS LONGE) e a convergência com ela.
    ///
    /// A lei é a do rapier e a do resto do módulo — `v /= 1 + d·dt` — **pesada
    /// pelo mesmo falloff**, senão um corpo cruzando a fronteira seria frenado de
    /// uma vez (o degrau que a régua única existe para não ter).
    pub damping: f32,
}

/// **O peso de um corpo a `dist` do centro de um alcance `radius`.**
///
/// `1` no centro, **0 exatamente em `radius`**, linear entre os dois. A porta
/// ÚNICA: a explosão, a atração e o anel do overlay perguntam a ela, senão o
/// desenho descreveria um alcance que o solver não usa (o argumento do
/// `scaled_shape`).
///
/// `radius <= 0` devolve 0 — um alcance nulo não alcança nada, e é o que faz o
/// slider no mínimo ser inerte em vez de dividir por zero.
#[must_use]
pub fn blast_falloff(dist: f32, radius: f32) -> f32 {
    if radius <= 0.0 {
        return 0.0;
    }
    (1.0 - dist / radius).clamp(0.0, 1.0)
}

impl PhysicsWorld {
    /// **A resistência que faz da atração uma ferramenta**, e não um estilingue.
    ///
    /// MEDIDO (`world::tests::sweep_the_attract_damping`, corpo de 1 kg a 2 m do
    /// foco num campo de raio 3 m e força 50 N, gravidade ligada — a distância ao
    /// foco depois de 1 s e de 2 s, e a velocidade que sobra):
    ///
    /// | damping | dist @1 s | dist @2 s | v @2 s |
    /// |---|---|---|---|
    /// | 0 | 0,064 | **2,342** | 0,06 |
    /// | 1 | 0,408 | 0,110 | 4,17 |
    /// | 2 | 0,404 | 0,037 | 2,67 |
    /// | **4** | 0,241 | **0,012** | 0,21 |
    /// | 8 | 0,052 | 0,001 | 0,42 |
    ///
    /// A linha de `0` é o achado: o corpo chega ao foco em 1 s e está a **2,3 m
    /// dele** um segundo depois — ele atravessou e voltou, porque uma atração
    /// puramente conservativa é um oscilador harmônico. Com `4` ele chega e FICA
    /// (12 mm, 0,21 m/s de resíduo) sem parecer colado.
    ///
    /// Não é knob: é a lei do campo. O que o artista regula é **quão forte** ele
    /// puxa, e um segundo número que precisa concordar com o primeiro para a
    /// ferramenta convergir é a falha de duas-portas que esta linha já pagou.
    pub const ATTRACT_DAMPING: f32 = 4.0;

    /// **A EXPLOSÃO** — um impulso radial, uma vez, a todo corpo dinâmico dentro
    /// de `radius` de `center`. Devolve quantos corpos foram atingidos (o número
    /// que o toast e os gates leem).
    ///
    /// `impulse` é o valor NO CENTRO, em `N·s`: resistido pela massa, então a
    /// folha voa e o caixote resiste — a mesma escolha que o W-Area fez pela
    /// força de zona, e o oposto de uma zona de *aceleração*, que seria a segunda
    /// resposta ao que o `GravityScale` já diz por-corpo.
    ///
    /// ⚠️ **A distância é medida ao CENTRO DE MASSA, e o impulso é aplicado ali**
    /// (`apply_impulse`, não `apply_impulse_at_point`) ⇒ **a explosão não impõe
    /// torque**. É o que o `AddExplosionForce` da Unity faz, e é deliberado: o
    /// ponto de aplicação "certo" seria o ponto do collider mais próximo do
    /// estouro, o que é uma consulta de geometria a mais para produzir um giro
    /// que a cena já produz sozinha (corpos a distâncias diferentes recebem
    /// impulsos diferentes e tombam nas colisões). Uma parede enorme cujo centro
    /// está longe do estouro **não** é atingida — a mesma limitação honesta.
    ///
    /// ⚠️ **Acorda o corpo** (`wake_up = true`): cutucar uma pilha assentada é o
    /// caso de uso inteiro, e um corpo dormindo não é integrado.
    pub fn explode(&mut self, center: [f32; 2], radius: f32, impulse: f32) -> usize {
        let mut hit = 0usize;
        for (_, body) in self.bodies.iter_mut() {
            if body.body_type() != RigidBodyType::Dynamic {
                continue;
            }
            let com = body.center_of_mass();
            let d = Vector2::new(com.x - center[0], com.y - center[1]);
            let dist = d.norm();
            let w = blast_falloff(dist, radius);
            if w <= 0.0 {
                continue;
            }
            // ⚠️ A direção de um corpo EXATAMENTE no centro é indefinida.
            // `normalize` de um vetor nulo é NaN, e um NaN na velocidade envenena
            // a pose, o `Transform` e o hash determinista (a lição do `clamped()`
            // do W3). Um corpo no olho do estouro não é empurrado — não há para
            // onde, e "para cima" seria inventar um eixo.
            if dist <= f32::EPSILON {
                continue;
            }
            let dir = d / dist;
            body.apply_impulse(dir * (impulse * w), true);
            hit += 1;
        }
        hit
    }

    /// Arma (ou desarma) o campo de atração. `None` é o release.
    ///
    /// Como a mão: o mundo guarda a sessão e o `step` a aplica, então o chamador
    /// só precisa mover o centro e soltar no fim.
    pub fn set_attract(&mut self, attract: Option<Attract>) {
        self.attract = attract;
    }

    /// O campo em voo, se houver — a fonte ÚNICA do fato (o overlay pergunta a
    /// ela em vez de guardar uma cópia).
    #[must_use]
    pub fn attracting(&self) -> Option<Attract> {
        self.attract
    }
}

/// **Um sub-passo de atração.** Chamada de dentro do laço do `step`, ao lado do
/// `drag` e do `effector` e pela mesma razão: uma força aplicada uma vez por TICK
/// erraria pelo número de sub-passos.
///
/// No-op sem campo armado, o que a mantém byte-neutra em toda cena que não está
/// sendo cutucada.
pub(super) fn apply_attract(bodies: &mut RigidBodySet, attract: &Option<Attract>, dt: f32) {
    let Some(a) = attract else {
        return;
    };
    for (_, body) in bodies.iter_mut() {
        if body.body_type() != RigidBodyType::Dynamic {
            continue;
        }
        let com = body.center_of_mass();
        let d = Vector2::new(a.center[0] - com.x, a.center[1] - com.y);
        let dist = d.norm();
        let w = blast_falloff(dist, a.radius);
        if w <= 0.0 || dist <= f32::EPSILON {
            continue;
        }
        // Impulso = força × dt, o primitivo certo: `add_force` do rapier é
        // CONSTANTE até um `reset_forces` que este pipeline nunca chama, e foi
        // isso que deixou os terminais do W2b não-monotônicos.
        let dir = d / dist;
        body.apply_impulse(dir * (a.force * w * dt), true);
        // E a resistência, pesada pelo MESMO `w` — ver [`Attract::damping`].
        if a.damping > 0.0 {
            let k = 1.0 / (1.0 + a.damping * w * dt);
            let v = *body.linvel();
            body.set_linvel(v * k, true);
        }
    }
}

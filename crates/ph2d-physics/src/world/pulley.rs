//! **A POLIA** — a corda que passa por duas roldanas (ADR-0131, W-Pulley).
//!
//! Dois corpos, dois pontos fixos de mundo (as roldanas) e uma corda que os une:
//! `l1 + razão·l2 ≤ L0`. Puxar um lado ergue o outro; com `razão ≠ 1` é uma
//! talha, e o lado de razão maior anda menos e levanta mais.
//!
//! # Por que isto NÃO é um joint do rapier, e não foi escolha
//!
//! `grep -rin pulley` sobre `rapier2d-0.28.0/src` devolve **nada** — o conjunto
//! é Fixed · Generic · Prismatic · Revolute · Rope · Spherical · Spring, e o
//! `b2PulleyJoint` do Box2D não foi portado. E **não há gancho de restrição do
//! usuário**: `PhysicsHooks` tem exatamente três métodos e os três falam de
//! CONTATOS (`filter_contact_pair` · `filter_intersection_pair` ·
//! `modify_solver_contacts`), então não existe rota para injetar uma linha
//! própria no solver dele.
//!
//! Logo a polia é imposta **de fora**, no mesmo lugar e pelo mesmo mecanismo que
//! o arrasto, as zonas de força e o campo de atração: um passe de impulso por
//! **sub-passo**. O que ela NÃO é: um laço de força PD. A `world::grab` já mediu
//! e escreveu o preço disso — *"um controlador PD explícito explode no ganho
//! equivalente a 1/60 s"* — e a razão é que uma força é integrada, então o ganho
//! que segura um peso é o mesmo que faz o passo seguinte passar do alvo.
//!
//! # É uma PROJEÇÃO de velocidade, e é isso que a torna estável
//!
//! O impulso sai da formulação de impulso sequencial, com a **massa efetiva
//! exata** do Jacobiano — não de um ganho afinado à mão:
//!
//! ```text
//! C     = l1 + razão·l2 − L0                        (a violação, em metros)
//! Ċ     = u1·v(p1) + razão·u2·v(p2)                  (a taxa dela)
//! k     = uᵀ M⁻¹ u  +  razão²·(uᵀ M⁻¹ u)  +  I⁻¹(r×u)² …
//! λ     = (Ċ + β·C/dt) / k                           , λ ≥ 0
//! ```
//!
//! e o impulso é `−λ·u1` em `p1` e `−razão·λ·u2` em `p2`. Como `λ` é dividido
//! pela massa efetiva **verdadeira**, um passo o zera: não há ganho a estourar,
//! e a corda segura igual com um corpo de 0,1 kg ou de 100 kg. É a mesma razão
//! pela qual a mão é um joint e não um laço de força — só que aqui o solver não
//! nos deixa entrar, então trazemos a álgebra dele para fora.
//!
//! ⚠️ **`effective_inv_mass` é um VETOR (por eixo)**, porque é ele que carrega o
//! Freeze X/Y do W-LockPos — então o termo é a forma quadrática `u.x²·invM.x +
//! u.y²·invM.y`, nunca `invM·|u|²`. Escrito assim, um corpo com um eixo travado
//! entra na conta com a massa infinita que ele de fato tem naquele eixo, e a
//! polia o respeita **de graça**.
//!
//! # Uma corda PUXA e não empurra — a desigualdade é a física, não um atalho
//!
//! `C ≤ 0` (folga) é o estado normal de uma corda frouxa: ela não faz nada. Só
//! `C > 0` age, e `λ` é clampado em zero, então a polia **nunca empurra** um
//! corpo para longe da roldana. É a mesma semântica do `RopeJoint` que o kit já
//! tem (um TETO de distância), estendida à SOMA dos dois ramos — e é o que faz
//! um contrapeso pousar no chão sem a corda o arrancar de volta.
//!
//! ⚠️ **A corda ESTICA sob carga, e isso tem de estar dito na UI**: o passe roda
//! uma vez por sub-passo, fora do solver, então ele disputa com os contatos em
//! vez de ser resolvido junto com eles. O erro residual é medido em
//! `tests/measure_pulley.rs` e nomeado no `PULLEY_BIAS`.

use rapier2d::dynamics::{RigidBodyHandle, RigidBodySet};
use rapier2d::na::{Point2, Vector2};

/// Uma polia viva: os dois corpos, onde a corda se prende em cada um, onde estão
/// as duas roldanas, a vantagem mecânica e o comprimento da corda.
///
/// Os pontos de amarração são **body-local** pela mesma razão que as âncoras de
/// joint são (W-AnchorFollow): guardar mundo faria o nó caminhar pelo corpo
/// conforme ele gira. As **roldanas são mundo** porque é isso que uma roldana é —
/// um ponto pregado no cenário.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PulleyDesc {
    /// O corpo do primeiro ramo.
    pub body_a: RigidBodyHandle,
    /// O corpo do segundo.
    pub body_b: RigidBodyHandle,
    /// Onde a corda se amarra em A, no frame local dele.
    pub local_a: [f32; 2],
    /// Onde a corda se amarra em B, no frame local dele.
    pub local_b: [f32; 2],
    /// A roldana do ramo A, em **mundo**.
    pub wheel_a: [f32; 2],
    /// A roldana do ramo B, em **mundo**.
    pub wheel_b: [f32; 2],
    /// **A vantagem mecânica da talha:** `l1 + razão·l2 ≤ L0`.
    ///
    /// A regra em uma frase: **o lado B anda `1/razão` do que A anda, e por isso
    /// precisa pesar `razão` vezes mais para equilibrá-lo.** A vantagem de B é
    /// portanto `1/razão`, e ela vem de uma razão **menor** que 1 —
    /// `0,25` faz um contrapeso de 1 kg erguer 3 kg descendo quatro vezes mais
    /// caminho, que é exatamente o que uma talha troca.
    ///
    /// ⚠️ **Escrevi isto ao contrário na primeira versão** (*"2 é a talha, o lado A
    /// anda o dobro e ergue o dobro"*) e a cena de smoke mediu o oposto: com razão 2
    /// a carga pesada não só ganhava como caía o DOBRO. `1` é a polia simples.
    pub ratio: f32,
    /// `L0` — o comprimento total da corda, em metros de `l1 + razão·l2`.
    pub total_length: f32,
}

/// Fração do erro de posição corrigida por sub-passo (Baumgarte).
///
/// **MEDIDO** em `tests/measure_pulley.rs`, numa balança de Atwood de 1 kg × 1 kg
/// (massas iguais de propósito: é a única configuração com equilíbrio estático,
/// logo a única em que *"quanto a corda estica em regime"* é pergunta com
/// resposta — com massas diferentes o sistema acelera para sempre, que é o que
/// uma máquina de Atwood faz):
///
/// | β | esticamento em regime | tremor (pico-a-pico, 0,5 s) |
/// |---|---|---|
/// | 0,05 | 0,0043 m | 0,00000 |
/// | 0,10 | 0,0021 m | 0,00000 |
/// | **0,20** | **0,0011 m** | **0,00000** |
/// | 0,40 | 0,0005 m | 0,00000 |
/// | 0,80 | 0,0003 m | 0,00000 |
/// | 1,00 | 0,0002 m | 0,00000 |
/// | 2,00 | 0,0001 m | 0,00000 |
///
/// E contra um CONTATO — a carga de 4 kg apoiada no chão com a corda esticada,
/// um contrapeso de 1 kg puxando para cima o tempo todo (o caso que de fato
/// limitaria uma correção explícita, porque é onde ela disputa com o solver):
/// `y` final **0,3990 m em toda a faixa**, tremor **0,00000**, e a velocidade
/// residual vai de 0,0153 a 0,0155 m/s entre β 0,05 e 2,00.
///
/// ⚠️ **As duas varreduras dizem que β NÃO é um knob de estabilidade** — não há
/// oscilação medida em lugar nenhum da faixa, e o erro cai exatamente como
/// `1/β`. É a assinatura de uma **projeção de velocidade com a massa efetiva
/// exata**, e não de um ganho: a cada sub-passo a gravidade injeta `g·dt` e a
/// correção devolve β do erro, então o regime é onde os dois se igualam. Um laço
/// de força PD teria a outra assinatura, que é a que a `world::grab` mediu.
///
/// **Então por que 0,20, e não o mais preciso que a tabela oferece?** Porque
/// 0,0011 m já é **menor que a tolerância de repouso do próprio rapier** — o
/// `normalized_allowed_linear_error`, 1,3 mm, medido nesta mesma linha quando a
/// interpenetração foi investigada. Uma corda seis vezes mais exata que todo
/// contato da cena é precisão que ninguém pode ver, comprada com uma correção
/// explícita mais agressiva. O teto legítimo é o da engine, não o da tabela.
///
/// ⚠️ **A primeira versão desta tabela foi ESCRITA ANTES DE MEDIR e estava
/// errada em todos os números e na conclusão** (ela afirmava 0,1462 m em β=0,05,
/// 34× o real, e que β=1,00 oscilava — não oscila). Fica registrado porque o
/// modo de falha é o pior possível: uma tabela plausível, com casas decimais,
/// que ninguém re-mede.
pub const PULLEY_BIAS: f32 = 0.2;

/// Abaixo disto um ramo não tem direção definida (o corpo está EM CIMA da
/// roldana), e normalizar produziria `NaN` que envenenaria a pose e o hash C9.
/// O ramo é simplesmente pulado — uma corda de comprimento zero não puxa para
/// lado nenhum, o que é a resposta certa e não um caso especial.
const MIN_BRANCH: f32 = 1.0e-4;

/// **Impor as polias**, uma vez por sub-passo.
///
/// No-op para todo mundo sem polia (o laço não roda), que é o que mantém isto
/// byte-neutro para toda cena que já existia.
pub fn apply(bodies: &mut RigidBodySet, pulleys: &[PulleyDesc], dt: f32, bias: f32) {
    if pulleys.is_empty() || dt <= 0.0 {
        return;
    }
    for p in pulleys {
        let Some(branch_a) = branch(bodies, p.body_a, p.local_a, p.wheel_a) else {
            continue;
        };
        let Some(branch_b) = branch(bodies, p.body_b, p.local_b, p.wheel_b) else {
            continue;
        };
        let ratio = p.ratio;
        // A violação. `C ≤ 0` é corda frouxa: uma corda não empurra, então não
        // há nada a fazer — e sair aqui é o que deixa um contrapeso pousado no
        // chão em paz.
        let c = branch_a.length + ratio * branch_b.length - p.total_length;
        if c <= 0.0 {
            continue;
        }
        // A massa efetiva do Jacobiano. `ratio²` no segundo ramo porque é assim
        // que ele entra em `C`.
        let k = branch_a.k + ratio * ratio * branch_b.k;
        if k <= f32::EPSILON {
            continue;
        }
        let c_dot = branch_a.rate + ratio * branch_b.rate;
        // Só PUXA: `λ` negativo seria a corda empurrando o corpo para longe da
        // roldana, que é a única coisa que uma corda não sabe fazer.
        let lambda = (c_dot + bias * c / dt) / k;
        if lambda <= 0.0 {
            continue;
        }
        push(bodies, p.body_a, branch_a.point, -lambda, branch_a.dir);
        push(
            bodies,
            p.body_b,
            branch_b.point,
            -lambda * ratio,
            branch_b.dir,
        );
    }
}

/// Um ramo da corda, resolvido contra a pose VIVA do corpo.
struct Branch {
    /// Onde a corda se prende, em mundo.
    point: Point2<f32>,
    /// Da roldana para o ponto de amarração, unitário.
    dir: Vector2<f32>,
    /// `|point − roldana|`.
    length: f32,
    /// A contribuição deste ramo para `Ċ`.
    rate: f32,
    /// A contribuição deste ramo para a massa efetiva.
    k: f32,
}

/// Resolver um ramo. `None` quando o corpo sumiu ou quando o ramo é curto demais
/// para ter direção.
fn branch(
    bodies: &RigidBodySet,
    handle: RigidBodyHandle,
    local: [f32; 2],
    wheel: [f32; 2],
) -> Option<Branch> {
    let body = bodies.get(handle)?;
    let point = body.position() * Point2::new(local[0], local[1]);
    let d = point - Point2::new(wheel[0], wheel[1]);
    let length = d.norm();
    if length < MIN_BRANCH {
        return None;
    }
    let dir = d / length;
    // `r` é medido do CENTRO DE MASSA, não da origem do corpo: é em torno dele
    // que o corpo gira, e usar a origem daria o braço de alavanca errado em todo
    // corpo cujo collider tem offset (W-Offset).
    let com = body.center_of_mass();
    let r = point - com;
    let angvel = body.angvel();
    // A velocidade do PONTO, não a do corpo: `v + ω × r` (em 2D, `ω × r` é
    // `(−ω·r.y, ω·r.x)`).
    let v_point = body.linvel() + Vector2::new(-angvel * r.y, angvel * r.x);
    let rate = v_point.dot(&dir);
    // ⚠️ **Um corpo não-dinâmico é massa INFINITA para a corda, e o rapier não
    // diz isso pelas mass properties.** Um corpo `Fixed` guarda o
    // `effective_inv_mass` que os colliders dele deram — medido, uma parede de
    // 1 kg reporta `1.0` — porque quem honra o TIPO é o solver (que simplesmente
    // não integra o corpo), não a tabela de massas; só o `LockedAxes` zera a
    // entrada. Sem esta linha uma corda amarrada a uma parede dividia a correção
    // com ela e ficava 5,8× mais frouxa (`C` 0,003087 contra 0,000532), e o gate
    // do eixo congelado nasceu VERMELHO exatamente por isso — o caso `lock_y`
    // estava certo e o caso PAREDE, errado.
    //
    // A velocidade do ponto (`rate`) NÃO é zerada junto, e a diferença é uma
    // feature: um corpo **kinematic** é massa infinita (a corda não o move) mas
    // TEM movimento próprio, então uma bobina animada por curva vira um guincho
    // de graça.
    let k = if body.is_dynamic() {
        let mp = body.mass_properties();
        // ⚠️ Forma QUADRÁTICA sobre a inversa de massa por-eixo, porque é o
        // `effective_inv_mass` que carrega o Freeze X/Y (cabeçalho do módulo).
        let inv_m = mp.effective_inv_mass;
        let inv_i = mp.effective_world_inv_inertia_sqrt * mp.effective_world_inv_inertia_sqrt;
        let cross = r.x * dir.y - r.y * dir.x;
        dir.x * dir.x * inv_m.x + dir.y * dir.y * inv_m.y + inv_i * cross * cross
    } else {
        0.0
    };
    Some(Branch {
        point,
        dir,
        length,
        rate,
        k,
    })
}

/// Aplicar o impulso de um ramo, acordando o corpo.
///
/// ⚠️ **Acordar não é higiene:** um corpo pendurado numa polia ADORMECE, e um
/// corpo dormindo não é integrado — sem isto, o contrapeso que assenta trava a
/// polia inteira e o outro lado fica pendurado no ar. É a mesma lição que o
/// `move_grab` pagou.
fn push(
    bodies: &mut RigidBodySet,
    handle: RigidBodyHandle,
    at: Point2<f32>,
    magnitude: f32,
    dir: Vector2<f32>,
) {
    if let Some(body) = bodies.get_mut(handle)
        && body.is_dynamic()
    {
        body.apply_impulse_at_point(dir * magnitude, at, true);
    }
}

impl super::PhysicsWorld {
    /// **Install the pulley table**, replacing whatever was there.
    ///
    /// Wholesale rather than per-pulley because a pulley is not owned by a body:
    /// the bridge re-derives the whole set from the authored components every
    /// dispatch (the same shape the joint reconcile has), so an incremental API
    /// would need a removal door whose only caller would be a diff nobody keeps.
    pub fn set_pulleys(&mut self, pulleys: Vec<PulleyDesc>) {
        self.pulleys = pulleys;
    }

    /// Sweep door for the table on [`PULLEY_BIAS`] — see the field's own note.
    pub fn set_pulley_bias(&mut self, bias: f32) {
        self.pulley_bias = bias;
    }

    /// **Trocar a tabela pela do chamador**, devolvendo-lhe a anterior.
    ///
    /// É por aqui que a ponte instala as polias todo dispatch: ela reconstrói a
    /// lista num scratch próprio e troca, então o caso comum não aloca nada — o
    /// que o gate de zero-alloc do caminho quente exige. `set_pulleys` fica para
    /// fixtures, onde a alocação não importa e a leitura é mais direta.
    pub fn swap_pulleys(&mut self, other: &mut Vec<PulleyDesc>) {
        std::mem::swap(&mut self.pulleys, other);
    }

    /// The live pulleys — what the overlay draws the rope from.
    #[must_use]
    pub fn pulleys(&self) -> &[PulleyDesc] {
        &self.pulleys
    }

    /// A massa efetiva de cada ramo — diagnóstico para as tabelas de
    /// `measure_pulley.rs`, que precisam distinguir *quanto* cada lado absorve
    /// do *onde* ele está.
    #[must_use]
    pub fn pulley_branch_k(&self, desc: &PulleyDesc) -> Option<(f32, f32)> {
        let a = branch(&self.bodies, desc.body_a, desc.local_a, desc.wheel_a)?;
        let b = branch(&self.bodies, desc.body_b, desc.local_b, desc.wheel_b)?;
        Some((a.k, b.k))
    }

    /// `l1 + ratio·l2` for a pulley **as it stands now**, so a caller can seed
    /// `total_length` from the rest pose instead of asking the artist to measure
    /// a rope with a ruler.
    ///
    /// `None` when either branch is degenerate — the same refusal `apply` makes,
    /// asked through the same door so a seed can never name a length the pass
    /// would then decline to hold.
    #[must_use]
    pub fn pulley_span(&self, desc: &PulleyDesc) -> Option<f32> {
        let a = branch(&self.bodies, desc.body_a, desc.local_a, desc.wheel_a)?;
        let b = branch(&self.bodies, desc.body_b, desc.local_b, desc.wheel_b)?;
        Some(a.length + desc.ratio * b.length)
    }
}

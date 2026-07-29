//! **A POLIA** — a corda que passa por N roldanas (ADR-0131, W-Pulley).
//!
//! Dois corpos, uma corda, e no meio dela qualquer número de roldanas com raio
//! próprio: `L(rota) ≤ L0`. Puxar um lado ergue o outro. A geometria da rota —
//! por onde a corda passa, quanto dela existe — mora inteira em
//! [`super::rope_route`]; aqui fica só a **dinâmica**.
//!
//! # Não há `ratio`, e a razão é física
//!
//! A v1 desta wave carregava um `ratio` (`l1 + razão·l2 ≤ L0`) vendido como
//! vantagem mecânica. **Ele descrevia uma corda que não existe:** numa corda
//! única sobre roldanas livres a tensão é **uniforme**, então os dois corpos
//! sentem a MESMA força e a vantagem é **1**, quaisquer que sejam os diâmetros.
//! Aquela lei é, de fato, a de uma talha **diferencial** (dois tambores de raios
//! diferentes no mesmo eixo) *com os tambores invisíveis* — um número sem peça
//! na cena. A vantagem verdadeira volta por onde ela vem no mundo: uma roldana
//! **montada num corpo que se move** (W3) ou um **tambor dirigido** (W2).
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
//! C     = L(rota) − L0                              (a violação, em metros)
//! Ċ     = u1·v(p1) + u2·v(p2)                        (a taxa dela)
//! k     = uᵀ M⁻¹ u  +  uᵀ M⁻¹ u  +  I⁻¹(r×u)² …
//! λ     = (Ċ + β·C/dt) / k                           , λ ≥ 0
//! ```
//!
//! ⚠️ **`u` é o versor do TRECHO que chega ao corpo, e o ARCO não entra no
//! Jacobiano** — o ponto de tangência desliza quando a âncora se move, e a
//! variação do arco cancela a do trecho (teorema do envelope; a derivação está
//! no cabeçalho da [`super::rope_route`]). É por isso que N roldanas COM raio
//! custam ao kernel exatamente o que duas de raio zero custavam.
//!
//! e o impulso é `−λ·u1` em `p1` e `−λ·u2` em `p2`. Como `λ` é dividido
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
//!
//! # O MOTOR (W2): um tambor dirigido muda o COMPRIMENTO DE REPOUSO
//!
//! Uma roldana com motor é um **guincho**, e a lei é uma linha: `L0` encolhe a
//! `Σ ω·r` (o [`PulleyDesc::motor_rate`]). Recolher encurta a corda e ergue o que
//! estiver pendurado; pagar corda alonga, e a carga desce **pela gravidade**, com
//! a corda ainda a segurando.
//!
//! ⚠️ **Um alvo de VELOCIDADE não serviria, e a desigualdade é o motivo.** A
//! forma óbvia seria `λ = (Ċ − v_alvo + β·C/dt)/k`: com a corda esticada e o alvo
//! recolhendo, `λ > 0` e o guincho ERGUE — mas com o alvo pagando corda `λ` fica
//! negativo, é clampado em zero, e **o comprimento nunca mudou**: a restrição
//! `L ≤ L0` continua segurando a carga no ar. Um guincho que sobe e não desce.
//! Mexer em `L0` não tem esse buraco, porque quem baixa a carga é a gravidade — e
//! `λ ≥ 0` fica intacta, então nada é EMPURRADO em momento nenhum.
//!
//! ⚠️ **O que foi recolhido é ESTADO VIVO, e mora no [`super::PhysicsWorld`]** —
//! não aqui, e não no componente autorado. Não no componente porque um número
//! reescrito por frame é um passo de undo por frame (a lei do W1); não na tabela
//! porque a ponte a reinstala por dispatch. E ele entra no
//! [`super::checkpoint::PhysicsCheckpoint`] pelo motivo que decide tudo nesta
//! linha: **o mundo é função do TICK**, e um guincho cujo recolhido não fosse
//! restaurado faria um scrub com acerto de ring discordar de um replay do zero.
//!
//! ⚠️ **Por SUB-PASSO, não por tick** — a mesma lei do arrasto e das zonas. Por
//! tick, `L0` daria degraus de `rate·dt` que o passe seguinte corrigiria de uma
//! vez; por sub-passo o degrau é `rate·dt/substeps` e a carga sobe lisa.

use super::rope_load::{PulleyBreak, PulleyLedger, ledger_axles};
use super::rope_route::{self, RopeWheel, Tangent};
use rapier2d::dynamics::{RigidBodyHandle, RigidBodySet};
use rapier2d::na::{Point2, Vector2};

/// A metade da polia que fala de roldanas MONTADAS num corpo (W3).
///
/// Módulo FILHO e não irmão: ele precisa do [`End`] e do [`end`], que são
/// privados daqui — um eixo montado é **mais uma ponta da mesma restrição**, e
/// dar-lhe uma resposta própria para *"qual é a massa efetiva neste ponto?"* seria
/// a segunda porta que esta linha passou a wave inteira fechando.
#[path = "pulley_mount.rs"]
mod mount;

/// A metade da polia que fala do TAMBOR (W2): a que taxa ele recolhe, quanto já
/// recolheu, e o único guarda que o guincho precisou.
///
/// Filho pelo mesmo motivo do irmão acima — ele lê o [`PulleyDesc`] e escreve no
/// livro-razão do mundo, e o corte é por assunto: *o que a corda FAZ com os
/// corpos* aqui, *o que um tambor faz com a corda* ali.
#[path = "pulley_winch.rs"]
mod winch;

pub use winch::PULLEY_CORRECTION_LAG;
use winch::reel;

pub use mount::refresh_mounts;
use mount::{mount_point, mounted_axle};

/// Uma polia viva: os dois corpos, onde a corda se prende em cada um, quais
/// roldanas ela atravessa e quanto de corda existe.
///
/// Os pontos de amarração são **body-local** pela mesma razão que as âncoras de
/// joint são (W-AnchorFollow): guardar mundo faria o nó caminhar pelo corpo
/// conforme ele gira. As **roldanas são mundo** porque é isso que uma roldana é —
/// um ponto pregado no cenário.
///
/// ⚠️ **As roldanas vivem numa ARENA à parte** (`wheel_start`/`wheel_count`
/// indexam o `&[RopeWheel]` que anda junto com a tabela), e não num `Vec` aqui
/// dentro: o passe roda por sub-passo sobre uma tabela que a ponte re-instala
/// todo dispatch, e um `Vec` por polia seria uma alocação por corda por frame —
/// além de tirar o `Copy`, que é o que deixa a tabela ser trocada com um `swap`.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PulleyDesc {
    /// **Quem esta corda É, através das trocas de tabela.**
    ///
    /// A tabela é reinstalada por dispatch e um índice não sobrevive a acrescentar
    /// uma corda, então o que a corda RECOLHEU (o `payout` do motor) não pode ser
    /// guardado por posição. A ponte passa aqui o `stable_name_id` do nome da
    /// corda — a MESMA chave por que ela aponta os corpos e as roldanas.
    ///
    /// `0` é *"sem nome"*, e uma corda sem nome não pode ter motor: as roldanas
    /// são encontradas pelo nome dela, então ela não tem roldana nenhuma, logo não
    /// tem tambor. O modo de falha de duas cordas anônimas dividirem uma chave é,
    /// por construção, duas cordas que recolhem zero.
    pub id: u64,
    /// O corpo do primeiro ramo.
    pub body_a: RigidBodyHandle,
    /// O corpo do segundo.
    pub body_b: RigidBodyHandle,
    /// Onde a corda se amarra em A, no frame local dele.
    pub local_a: [f32; 2],
    /// Onde a corda se amarra em B, no frame local dele.
    pub local_b: [f32; 2],
    /// Índice da primeira roldana desta corda na arena.
    pub wheel_start: u32,
    /// Quantas roldanas ela atravessa, na ordem A → B. **Zero é legítimo** — é
    /// uma corda reta entre os dois corpos, que é o que uma polia sem roldana
    /// nenhuma de fato é.
    pub wheel_count: u32,
    /// `L0` — o comprimento total da corda, em metros de rota (trechos + arcos),
    /// **como o artista o autorou**. O que o motor já recolheu vive à parte, no
    /// [`super::PhysicsWorld`]; ver [`PulleyDesc::motor_rate`].
    pub total_length: f32,
    /// **A que taxa os TAMBORES desta corda a recolhem**, em metros por segundo —
    /// a SOMA de `ω·r` sobre as roldanas dela.
    ///
    /// Positivo encurta a corda. A soma é a lei certa e degenera sozinha: um
    /// tambor só é a própria taxa, dois puxando juntos recolhem o dobro, e dois em
    /// sentidos opostos se anulam — que é o que dois guinchos brigando pela mesma
    /// corda fazem.
    pub motor_rate: f32,
    /// **O que a CORDA aguenta**, newtons — `∞` é uma corda que não parte (W2).
    ///
    /// ⚠️ **UM número para as DUAS pontas, e isso não é economia:** a corda é
    /// inextensível, então a tensão é **uniforme** ao longo dela — as duas
    /// amarrações sentem exatamente a mesma carga. Dois limites contra a mesma
    /// carga são um limite (o menor) e um controle inerte ao lado dele, e o §7 do
    /// plano pedia dois antes de esta wave medir a uniformidade.
    ///
    /// O que difere de ponto para ponto é o **EIXO de cada roldana**, que carrega
    /// a RESULTANTE do desvio — e esse número mora em [`RopeWheel::break_force`].
    pub break_force: f32,
}

impl PulleyDesc {
    /// As roldanas desta corda dentro da arena.
    ///
    /// Uma faixa fora do fim devolve vazio em vez de entrar em pânico: a tabela e
    /// a arena são instaladas juntas pela ponte, então isto não deveria acontecer
    /// — e *deveria* é a palavra que precede um índice fora de faixa no caminho
    /// quente.
    #[must_use]
    pub fn wheels<'a>(&self, arena: &'a [RopeWheel]) -> &'a [RopeWheel] {
        let start = self.wheel_start as usize;
        arena
            .get(start..start + self.wheel_count as usize)
            .unwrap_or(&[])
    }
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

/// **Impor as polias**, uma vez por sub-passo.
///
/// No-op para todo mundo sem polia (o laço não roda), que é o que mantém isto
/// byte-neutro para toda cena que já existia. `scratch` é do chamador porque a
/// rota escreve `N+1` trechos por corda por sub-passo — alocá-los aqui apareceria
/// no gate de zero-alloc do caminho quente.
#[allow(clippy::too_many_arguments)] // a tabela, a arena, o livro-razão, o scratch e os dois números medidos
pub fn apply(
    bodies: &mut RigidBodySet,
    pulleys: &[PulleyDesc],
    arena: &[RopeWheel],
    led: &mut PulleyLedger<'_>,
    live: &mut Vec<RopeWheel>,
    scratch: &mut Vec<Tangent>,
    dt: f32,
    bias: f32,
    lag: f32,
) {
    if pulleys.is_empty() || dt <= 0.0 {
        return;
    }
    for p in pulleys {
        // Uma corda que já rompeu não segura mais nada, e não volta sem um
        // rebuild — como um joint desabilitado (W-J7), e pelo mesmo motivo: a
        // ruptura nunca é escrita no estado AUTORADO.
        if led.broken_ropes.contains(&p.id) {
            continue;
        }
        // As roldanas VIVAS desta corda. Um eixo que cedeu **sai da rota**, o que
        // encurta o caminho ⇒ `C < 0` ⇒ folga, e a carga cai. Sem estouro, por
        // construção. O scratch é do chamador porque isto roda por sub-passo.
        live.clear();
        live.extend(
            p.wheels(arena)
                .iter()
                .filter(|w| !led.broken_wheels.contains(&w.id))
                .copied(),
        );
        let Some(a) = end(bodies, p.body_a, p.local_a) else {
            continue;
        };
        let Some(b) = end(bodies, p.body_b, p.local_b) else {
            continue;
        };
        // A rota inteira, contra a pose VIVA dos dois corpos. `None` = algum
        // trecho é degenerado (o corpo está em cima de uma roldana, duas rodas se
        // tocam) — a corda inteira é pulada, e essa é a MESMA recusa que o modelo
        // de ponto fazia, pela mesma razão: meia rota é pior que nenhuma.
        let Some(route) = rope_route::route(
            [a.point.x, a.point.y],
            [b.point.x, b.point.y],
            live,
            scratch,
        ) else {
            continue;
        };
        let dir_a = Vector2::new(route.dir_a[0], route.dir_a[1]);
        // ⚠️ **A ponta B entra ENGRENADA** (W4). `weight_b` é `1.0` em toda corda
        // sem tambor diferencial — e `x * 1.0 == x` é exato —, então isto é
        // byte-neutro para tudo que já existia.
        //
        // Ele não é um versor, e não precisa ser: [`End::k`] é a forma quadrática
        // `vᵀM⁻¹v` e [`End::rate`] é uma projeção — nenhuma das duas pede norma 1.
        // É a MESMA razão pela qual o eixo montado do W3 entrou sem caminho de
        // código próprio, e é o que faz a vantagem mecânica contínua sair de
        // graça: a tensão em B é `weight_b` vezes a de A, porque o impulso lá é
        // `−λ·weight_b·u_b`.
        let dir_b = Vector2::new(route.dir_b[0], route.dir_b[1]) * route.weight_b;
        // Os tambores. Avança DEPOIS da rota — ela não depende do recolhido — e
        // antes da conta, de modo que o sub-passo que recolhe é o mesmo que
        // corrige; um passo de atraso aqui seria o guincho subindo um sub-passo
        // depois de mandado.
        let reeled = reel(led.payout, p, dt);
        // A violação. `C ≤ 0` é corda frouxa: uma corda não empurra, então não
        // há nada a fazer — e sair aqui é o que deixa um contrapeso pousado no
        // chão em paz.
        // ⚠️ Sem tambor, sem teto — é o que mantém byte-idêntica toda cena
        // que já existia (ver [`PULLEY_CORRECTION_LAG`]).
        let cap = if p.motor_rate == 0.0 {
            f32::INFINITY
        } else {
            lag * p.motor_rate.abs() * dt
        };
        let c = route.length - (p.total_length - reeled);
        if c <= 0.0 {
            continue;
        }
        let mut k = a.k(dir_a) + b.k(dir_b);
        let mut c_dot = a.rate(dir_a) + b.rate(dir_b);
        // **W3 — cada eixo MONTADO é mais uma ponta da mesma restrição.** A
        // roldana pregada no cenário não contribui com nada (o `None` sai fora), e
        // é isso que mantém toda cena anterior byte-idêntica.
        for i in 0..live.len() {
            let Some((_, e, j)) = mounted_axle(bodies, live, scratch, i) else {
                continue;
            };
            k += e.k(j);
            c_dot += e.rate(j);
        }
        if k <= f32::EPSILON {
            continue;
        }
        // Só PUXA: `λ` negativo seria a corda empurrando o corpo para longe da
        // roldana, que é a única coisa que uma corda não sabe fazer.
        let lambda = (c_dot + bias * c.min(cap) / dt) / k;
        if lambda <= 0.0 {
            continue;
        }
        push(bodies, p.body_a, a.point, -lambda, dir_a);
        push(bodies, p.body_b, b.point, -lambda, dir_b);
        // O impulso no eixo é `−λ·∂L/∂C`, a MESMA forma das duas pontas — e é aqui
        // que a vantagem mecânica aparece sozinha: num enlace de 180° o Jacobiano
        // tem magnitude **2**, então a cadernal móvel recebe o DOBRO da tensão da
        // corda. Ninguém escreveu um "2" em lugar nenhum; ele é a geometria.
        for i in 0..live.len() {
            let Some((handle, e, j)) = mounted_axle(bodies, live, scratch, i) else {
                continue;
            };
            push(bodies, handle, e.point, -lambda, j);
        }
        // A TENSÃO. `λ` é um impulso sobre o sub-passo, então a carga em newtons
        // é `λ/dt` — a mesma conversão que o `joint_impulse_to_force` faz para os
        // joints do rapier, e pela mesma razão: um limiar escrito em impulsos
        // mudaria de significado a cada troca da contagem de sub-passos.
        let tension = lambda / dt;
        // ⚠️ **A corda tem UMA tensão — até um tambor diferencial entrar nela**
        // (W4). O `break_force` do [`PulleyDesc`] documenta a uniformidade como a
        // razão de haver um número só, e essa premissa vale enquanto a corda
        // DESLIZA; o diferencial é precisamente onde ela não desliza, e os dois
        // lados carregam `T` e `T·gear`. O limiar tem de olhar o lado MAIS
        // carregado, senão uma corda com engrenagem 5 aguentaria cinco vezes o que
        // o artista dimensionou, em silêncio.
        //
        // Sem tambor `weight_max` é `1.0` e `peak == tension` **ao bit** — a
        // ruptura de toda cena anterior não se move.
        let peak = tension * route.weight_max;
        let slot = led.tension.entry(p.id).or_insert(0.0);
        *slot = slot.max(peak);
        // ⚠️ **O eixo recebe a tensão BASE, não o pico**, e a diferença é
        // load-bearing: o Jacobiano de cada roldana JÁ carrega os pesos dos dois
        // lados dela ([`wheel_jacobian`]), então multiplicar pelo pico contaria a
        // engrenagem duas vezes.
        ledger_axles(led, live, scratch, tension);
        if peak > p.break_force {
            led.broken_ropes.insert(p.id);
            led.breaks.push(PulleyBreak {
                id: p.id,
                is_wheel: false,
                // O meio da rota: o ponto que está SOBRE a corda desenhada em
                // qualquer montagem, que é onde o artista está olhando.
                point: [(a.point.x + b.point.x) * 0.5, (a.point.y + b.point.y) * 0.5],
                // A carga que de fato a partiu — a mesma que o limiar comparou.
                load: peak,
            });
        }
    }
}

/// Uma ponta da corda, resolvida contra a pose VIVA do corpo.
///
/// ⚠️ **Ela não conhece direção nenhuma**, e é isso que a separa do `Branch` do
/// modelo de ponto: com raio, quem decide a direção do trecho é a ROTA (o ponto
/// de tangência, não o centro da roldana), então a ponta guarda só o que é fato
/// do CORPO e responde `rate`/`k` para o versor que a rota entregar.
struct End {
    /// Onde a corda se prende, em mundo.
    point: Point2<f32>,
    /// Do centro de massa para o ponto de amarração — o braço de alavanca.
    r: Vector2<f32>,
    /// A velocidade DO PONTO (`v + ω × r`), não a do corpo.
    v: Vector2<f32>,
    /// A inversa de massa efetiva, por eixo. Zero para corpo não-dinâmico.
    inv_m: Vector2<f32>,
    /// A inversa de inércia efetiva. Zero para corpo não-dinâmico.
    inv_i: f32,
}

impl End {
    /// A contribuição desta ponta para `Ċ`.
    fn rate(&self, dir: Vector2<f32>) -> f32 {
        self.v.dot(&dir)
    }

    /// A contribuição desta ponta para a massa efetiva.
    ///
    /// ⚠️ Forma QUADRÁTICA sobre a inversa de massa por-eixo, porque é o
    /// `effective_inv_mass` que carrega o Freeze X/Y (cabeçalho do módulo).
    fn k(&self, dir: Vector2<f32>) -> f32 {
        let cross = self.r.x * dir.y - self.r.y * dir.x;
        dir.x * dir.x * self.inv_m.x + dir.y * dir.y * self.inv_m.y + self.inv_i * cross * cross
    }
}

/// Resolver uma ponta. `None` quando o corpo sumiu.
fn end(bodies: &RigidBodySet, handle: RigidBodyHandle, local: [f32; 2]) -> Option<End> {
    let point = mount_point(bodies, handle, local)?;
    let body = bodies.get(handle)?;
    // `r` é medido do CENTRO DE MASSA, não da origem do corpo: é em torno dele
    // que o corpo gira, e usar a origem daria o braço de alavanca errado em todo
    // corpo cujo collider tem offset (W-Offset).
    let r = point - body.center_of_mass();
    let angvel = body.angvel();
    // A velocidade do PONTO, não a do corpo: `v + ω × r` (em 2D, `ω × r` é
    // `(−ω·r.y, ω·r.x)`).
    let v = body.linvel() + Vector2::new(-angvel * r.y, angvel * r.x);
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
    // A velocidade do ponto (`v`) NÃO é zerada junto, e a diferença é uma
    // feature: um corpo **kinematic** é massa infinita (a corda não o move) mas
    // TEM movimento próprio, então uma bobina animada por curva vira um guincho
    // de graça.
    //
    // ⚠️ **E essa frase vale para a PONTA da corda, não para um eixo MONTADO** —
    // a distinção foi medida em 2026-07-29 (`measure_pulley_kinematic_sheave`),
    // porque o plano a estendia a *"uma roldana montada num corpo kinematic vira
    // um guincho de graça (o `end` não zera a velocidade do ponto, só a massa)"*.
    // A conclusão é certa e o mecanismo citado, não: **zerar `v` aqui não para o
    // içamento** de uma cadernal dirigida (razão 1,907 → 1,881, e o controle cai
    // junto ⇒ a vantagem fica 2), enquanto **neutralizar o `refresh_mounts` o
    // mata** (1,907 → 0,012, com o controle intocado em 0,952). Quem iça uma
    // cadernal montada é a GEOMETRIA — o eixo andando na arena faz a rota crescer,
    // e o termo posicional cobra. Este `v` é um refino da TAXA (o guincho
    // *antecipado* do gate irmão), ~1,4% do alcance, não a causa.
    let (inv_m, inv_i) = if body.is_dynamic() {
        let mp = body.mass_properties();
        (
            mp.effective_inv_mass,
            mp.effective_world_inv_inertia_sqrt * mp.effective_world_inv_inertia_sqrt,
        )
    } else {
        (Vector2::zeros(), 0.0)
    };
    Some(End {
        point,
        r,
        v,
        inv_m,
        inv_i,
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
    pub fn set_pulleys(&mut self, pulleys: Vec<PulleyDesc>, wheels: Vec<RopeWheel>) {
        self.pulleys = pulleys;
        self.pulley_wheels = wheels;
    }

    /// Sweep door for the table on [`PULLEY_BIAS`] — see the field's own note.
    pub fn set_pulley_bias(&mut self, bias: f32) {
        self.pulley_bias = bias;
    }

    /// A porta de varredura de [`PULLEY_CORRECTION_LAG`], irmã da de cima e pelo
    /// mesmo motivo: a tabela do teto é medida contra o PRODUTO.
    pub fn set_pulley_correction_lag(&mut self, lag: f32) {
        self.pulley_lag = lag;
    }

    /// **Trocar a tabela pela do chamador**, devolvendo-lhe a anterior.
    ///
    /// É por aqui que a ponte instala as polias todo dispatch: ela reconstrói a
    /// lista num scratch próprio e troca, então o caso comum não aloca nada — o
    /// que o gate de zero-alloc do caminho quente exige. `set_pulleys` fica para
    /// fixtures, onde a alocação não importa e a leitura é mais direta.
    pub fn swap_pulleys(&mut self, other: &mut Vec<PulleyDesc>, wheels: &mut Vec<RopeWheel>) {
        std::mem::swap(&mut self.pulleys, other);
        std::mem::swap(&mut self.pulley_wheels, wheels);
    }

    /// **Pôr os eixos MONTADOS onde os corpos deles estão AGORA** (W-Pulley W3).
    ///
    /// A [`mount::refresh_mounts`] tem DOIS chamadores e eles pedem coisas
    /// diferentes, então nenhum dos dois é redundante:
    ///
    /// - o `step` a roda por SUB-PASSO, para o SOLVER: geometria de um sub-passo
    ///   atrás puxa numa direção que já não é a da corda;
    /// - a PONTE a roda uma vez no fim de todo dispatch, para o DESENHO — e essa
    ///   é a metade que faltava. A arena é reinstalada a cada dispatch com o
    ///   centro derivado da pose de REPOUSO (é o que a colheita do ECS sabe), e
    ///   um quadro mais rápido que o tique não dá passo nenhum ⇒ ele desenhava a
    ///   roldana **onde ela foi autorada**. Medido num bloco que viaja: salto de
    ///   **1,27 m** entre um quadro e o seguinte, com a simulação correta o tempo
    ///   todo — o tremor que o smoke da talha reportou.
    ///
    /// No-op para toda roldana pregada no cenário, que é o que toda roldana era
    /// antes do W3.
    pub fn refresh_mounted_wheels(&mut self) {
        mount::refresh_mounts(&self.bodies, &mut self.pulley_wheels);
    }

    /// The live pulleys — what the overlay draws the rope from.
    #[must_use]
    pub fn pulleys(&self) -> &[PulleyDesc] {
        &self.pulleys
    }

    /// A arena de roldanas que as faixas de [`PulleyDesc`] indexam.
    #[must_use]
    pub fn pulley_wheels(&self) -> &[RopeWheel] {
        &self.pulley_wheels
    }

    /// **Quanto de corda os tambores desta corda já recolheram**, em metros —
    /// zero para uma corda sem motor, que é o estado de toda corda que ninguém
    /// dirigiu.
    ///
    /// É o número que o readout mostra e o que os gates comparam contra `ω·r·t`;
    /// o comprimento que a restrição de fato segura é
    /// `total_length − pulley_reeled`.
    ///
    /// ⚠️ **O mapa NÃO é podado quando uma corda sai da tabela**, e isso é
    /// deliberado: uma corda que pisca fora por um dispatch (um `active`
    /// desmarcado, um rename a caminho) mantém o guincho onde ele estava, em vez
    /// de o rebobinar em silêncio. O preço, nomeado: apagar uma corda e criar
    /// outra **com o mesmo nome** no MESMO run herda o recolhido — a mesma
    /// exposição de toda ligação por nome deste editor, e o Reset a cura, porque
    /// ele constrói um mundo novo.
    #[must_use]
    pub fn pulley_reeled(&self, desc: &PulleyDesc) -> f32 {
        self.pulley_payout.get(&desc.id).copied().unwrap_or(0.0)
    }

    /// A massa efetiva de cada ponta — diagnóstico para as tabelas de
    /// `measure_pulley.rs`, que precisam distinguir *quanto* cada lado absorve
    /// do *onde* ele está.
    #[must_use]
    pub fn pulley_branch_k(&self, desc: &PulleyDesc) -> Option<(f32, f32)> {
        let a = end(&self.bodies, desc.body_a, desc.local_a)?;
        let b = end(&self.bodies, desc.body_b, desc.local_b)?;
        let mut scratch = Vec::new();
        let r = rope_route::route(
            [a.point.x, a.point.y],
            [b.point.x, b.point.y],
            desc.wheels(&self.pulley_wheels),
            &mut scratch,
        )?;
        Some((
            a.k(Vector2::new(r.dir_a[0], r.dir_a[1])),
            b.k(Vector2::new(r.dir_b[0], r.dir_b[1])),
        ))
    }

    /// **A que velocidade a corda CORRE**, em m/s, positiva no sentido A → B.
    ///
    /// É o que faz uma roldana girar (`ω = s·lado/r`), e é UM número por corda e
    /// não um por roda: a corda é inextensível, então ela passa por todas as
    /// roldanas na mesma taxa. Derivar por roda seria N respostas para um fato.
    ///
    /// Sai do ramo A: se ele se ALONGA, a corda está sendo puxada de B para A,
    /// logo ela corre no sentido B → A — daí o sinal trocado.
    #[must_use]
    pub fn pulley_rope_speed(&self, desc: &PulleyDesc) -> Option<f32> {
        let a = end(&self.bodies, desc.body_a, desc.local_a)?;
        let b = end(&self.bodies, desc.body_b, desc.local_b)?;
        let mut scratch = Vec::new();
        let r = rope_route::route(
            [a.point.x, a.point.y],
            [b.point.x, b.point.y],
            desc.wheels(&self.pulley_wheels),
            &mut scratch,
        )?;
        Some(-a.rate(Vector2::new(r.dir_a[0], r.dir_a[1])))
    }

    /// O comprimento de rota de uma polia **como ela está agora**, para o
    /// chamador semear `total_length` da pose de repouso em vez de pedir ao
    /// artista que meça uma corda com régua.
    ///
    /// `None` quando a rota é degenerada — a mesma recusa que o `apply` faz,
    /// perguntada pela mesma porta, para que um semeio nunca nomeie um
    /// comprimento que o passe depois se recuse a segurar.
    #[must_use]
    pub fn pulley_span(&self, desc: &PulleyDesc) -> Option<f32> {
        let a = end(&self.bodies, desc.body_a, desc.local_a)?;
        let b = end(&self.bodies, desc.body_b, desc.local_b)?;
        let mut scratch = Vec::new();
        Some(
            rope_route::route(
                [a.point.x, a.point.y],
                [b.point.x, b.point.y],
                desc.wheels(&self.pulley_wheels),
                &mut scratch,
            )?
            .length,
        )
    }
}

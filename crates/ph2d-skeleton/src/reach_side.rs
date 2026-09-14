//! ⭐⭐⭐ **COMO UMA CORRENTE GANHA E GUARDA UM LADO** — irmão do [`super::reach`] pelo tecto de 700
//! LOC (`architecture_workspace_file_loc_cap`), cortado por RESPONSABILIDADE.
//!
//! Ali mora *onde a corrente chega*; aqui *para que lado ela dobra a caminho*. São três passos, e
//! a ordem entre eles é load-bearing (o [`super::reach`] chama-os por ela):
//!
//! 1. [`seed_arc`] — com um lado AUTORADO, a pose de partida é canónica, e é isso que torna o
//!    resultado uma função dos dados em vez de um resto do quadro anterior;
//! 2. [`break_collinearity`] — dá ao FABRIK *por onde cair* quando a corrente chega recta;
//! 3. [`mirror_to_side`] — o dono ÚNICO da escolha do lado. ⚠️ Uma mutação que dava sinal próprio
//!    ao bojo do arco **sobreviveu** a toda a suíte, porque este espelho já corrigia: *o lado tem
//!    um dono*, e daí sai a lei de que `Ccw` e `Cw` são espelhos exactos um do outro.

use super::reach::{BOW, BendSide, dominant_side, unit};

/// ⭐⭐⭐ **A POSE DE PARTIDA CANÓNICA de uma corrente com lado AUTORADO** — um arco de seno da raiz
/// ao alvo, a bojar para o lado pedido.
///
/// ⛔⛔ **Report do dono** (2026-09-14): *«IK Bend não está consistente para maior que 2. Muda o
/// ângulo de lado.»* O FABRIK é sensível à pose inicial e cada quadro partia do resultado do
/// anterior ⇒ a MESMA restrição resolvida de quatro poses de partida diferentes dava quatro poses
/// finais: `0,52` de desvio num alcance de `3` (**17 %**), `1,15` em `4`, `1,58` em `5` (**32 %**).
/// O lado saía certo e o ÂNGULO mudava. ⚠️ A dois ossos isto nunca aconteceu — ali a lei é
/// **fechada** e não olha para a pose.
///
/// ⇒ com um lado autorado a corrente parte sempre daqui, e o resultado passa a ser uma **função de
/// `(raiz, comprimentos, alvo, lado)`**.
///
/// ⛔⛔ **E o arco tem de ser de VERDADE, não um `BOW` de `1e-3`:** a 1.ª cura deitou a corrente
/// recta e deixou o [`break_collinearity`] dar-lhe o lado — e a `8` ossos a corrente **caiu para o
/// lado errado** (`dominant_side` `−1,87` com `Ccw` pedido). Aquele arqueamento existe para dar ao
/// FABRIK *por onde cair*, não para escolher a pose: uma perturbação de um milésimo do alcance não
/// sobrevive a quarenta passagens numa corrente longa.
///
/// A amplitude é a **folga** — a altura do triângulo isósceles de lados `total/2` sobre a base `d`
/// —, que é zero quando o alvo está no limite do alcance (ali a recta É a resposta) e máxima quando
/// a corrente está dobrada em dois. ⚠️ **É uma SEMENTE, não a resposta:** os comprimentos dos ossos
/// não são respeitados aqui, e são as passagens do FABRIK que os repõem.
///
/// ⭐⭐ **O arco boja SEMPRE para o mesmo lado, e quem escolhe é o [`mirror_to_side`]** — uma
/// mutação que multiplicava o bojo pelo lado pedido **sobreviveu** a toda a suíte, porque o espelho
/// a seguir já corrige. ⇒ *o lado tem UM dono*, e a consequência é uma lei mais forte: `Ccw` e `Cw`
/// saem **espelhos exactos** um do outro sobre a recta `raiz → alvo`, o que um bojo com sinal
/// próprio não garantiria.
pub(crate) fn seed_arc(
    joints: &mut [[f64; 2]],
    lengths: &[f64],
    total: f64,
    goal: [f64; 2],
    u: [f64; 2],
) {
    let root = joints[0];
    let d = (goal[0] - root[0]).hypot(goal[1] - root[1]).min(total);
    let meia = total * 0.5;
    let h = (meia * meia - d * d * 0.25).max(0.0).sqrt();
    let perp = [-u[1], u[0]];
    let mut acc = 0.0;
    for (i, len) in lengths.iter().enumerate() {
        acc += len;
        let t = acc / total;
        let bojo = h * (std::f64::consts::PI * t).sin();
        joints[i + 1] = [
            root[0] + u[0] * d * t + perp[0] * bojo,
            root[1] + u[1] * d * t + perp[1] * bojo,
        ];
    }
}

/// Arqueia uma corrente RECTA para o FABRIK ter um lado para onde cair — para o lado PEDIDO, se
/// houver um.
pub(crate) fn break_collinearity(p: &mut [[f64; 2]], reach: f64, goal: [f64; 2], side: BendSide) {
    let n = p.len();
    let to_goal = [goal[0] - p[0][0], goal[1] - p[0][1]];
    let d = to_goal[0].hypot(to_goal[1]);
    if reach - d < BOW * reach {
        return; // o alvo está na extensão máxima (ou além): a recta É a resposta
    }
    if dominant_side(p, goal).abs() > BOW * reach {
        return; // já está fora da recta — a iteração tem para onde cair
    }
    let u = unit(to_goal, [1.0, 0.0]);
    // `u × perp = +1`, então `+perp` é o lado anti-horário de [`side_of`].
    let perp = [-u[1], u[0]];
    let arco = BOW * reach * side.forced().unwrap_or(1.0);
    for q in p.iter_mut().take(n - 1).skip(1) {
        q[0] += perp[0] * arco;
        q[1] += perp[1] * arco;
    }
}

/// ⭐⭐⭐ **ESPELHA a corrente para o lado pedido** — a metade que o arqueamento não faz.
///
/// ⚠️ **Sem isto o lado autorado não morde numa corrente de 3+ ossos**, e a razão é que o
/// [`break_collinearity`] só age sobre uma pose **recta**: com a corrente já dobrada para o lado
/// errado ele devolve cedo, e o FABRIK parte da pose que encontra — ele **preserva** o lado, que é
/// exactamente o que aqui se quer mudar.
///
/// ⭐ A operação é uma **reflexão sobre a recta `raiz → alvo`**, e por ser uma isometria ela não
/// toca em nenhum comprimento — o invariante das duas leis desta crate sobrevive por construção,
/// não por uma guarda escrita à mão. A raiz fica onde está porque a recta passa por ela.
pub(crate) fn mirror_to_side(p: &mut [[f64; 2]], goal: [f64; 2], side: BendSide) {
    let Some(quero) = side.forced() else {
        return;
    };
    let actual = dominant_side(p, goal);
    if actual == 0.0 || actual.signum() == quero.signum() {
        return;
    }
    let root = p[0];
    let u = unit([goal[0] - root[0], goal[1] - root[1]], [1.0, 0.0]);
    for q in &mut p[1..] {
        let v = [q[0] - root[0], q[1] - root[1]];
        let ao_longo = v[0] * u[0] + v[1] * u[1];
        // `q' = raiz + 2·(v·u)·u − v` — a reflexão de `v` sobre a direcção `u`.
        q[0] = root[0] + 2.0 * ao_longo * u[0] - v[0];
        q[1] = root[1] + 2.0 * ao_longo * u[1] - v[1];
    }
}

//! **A cena da MOLA** — `PH2D_BUILD_SMOKE=65` (plano UI/UX W7m).
//!
//! # A pergunta desta cena é de olho, e ela é sobre a REVERSÃO
//!
//! *Eu interrompo o gesto a meio caminho — a forma continua um instante para onde ia, e só depois
//! volta. Ela não PARA e recomeça.*
//!
//! Quatro faixas com **a mesma geometria, a mesma viagem e as mesmas duas poses**. A única coisa
//! que difere entre elas é **como** viajam, e é por isso que a cena é um A/B e não um catálogo:
//!
//! 1. **Spring** — mola sub-amortecida. Passa da marca e volta; e ao reverter, **carrega o
//!    momento**.
//! 2. **Curve** — `Cubic In-Out`, o regime que a medição nomeia a **0,00×**: revertido a meio, ele
//!    para e arranca do zero. É o A/B da mola, e a razão de ela existir.
//! 3. **Back** — `Back Out`. ⚠️ **É a mudança de comportamento desta wave**, e é o que o Enio tem
//!    de julgar: até aqui o clamp era GLOBAL e ele apenas *chegava*; agora pica em **1,100** e
//!    passa a marca, que é o que o nome da curva promete.
//! 4. **Control** — `Cubic Out`, o default de fábrica. Curva contida em `[0, 1]` ⇒ o clamp nunca
//!    mordia ⇒ ela tem de estar **byte-idêntica** ao que já shipava. É a metade que responde à
//!    ordem *"não prejudique nada do sistema de easing"*.
//!
//! # ⚠️ A MARCA DE ALVO é o que torna a cena legível
//!
//! *Passou do alvo* não se vê sem uma referência. Cada faixa tem um traço fino no `x` de destino,
//! e ele **não é hospedeiro** — não se mexe, e é contra ele que o olho mede o overshoot.
//!
//! ⚠️ **E ela imprime o número que a torna válida:** o pico de cada faixa, medido a andar as
//! máquinas. Se o Back não passar da marca, o clamp por canal não landou e o resto do roteiro não
//! diz nada.

use ph2d_anim::{Easing, EasingFamily, EasingMode};
use ph2d_ui_state::{Machine, Spring, StateRole};
use ph2d_vec_scene::{Paint, Rgba8, VecPathId, rectangle};

use crate::smoke_script::Step;

/// Onde cada faixa vive, e o que a dirige.
///
/// ⚠️ A ordem é a do roteiro: a entrega primeiro, o A/B logo abaixo dela (para o artista comparar
/// sem procurar), a decisão a seguir e o controle por último.
const LANES: [(&str, f64); 4] = [
    ("Spring", 2.4),
    ("Curve", 0.8),
    ("Back", -0.8),
    ("Control", -2.4),
];

const SPRING: usize = 0;
const CURVE: usize = 1;
const BACK: usize = 2;

/// A viagem. ⚠️ **Longa de propósito:** um overshoot de 10% sobre meia unidade é ruído de sub-pixel
/// — sobre seis unidades ele é meia largura da própria forma.
const X_REST: f64 = -5.4;
const X_HOVER: f64 = 1.2;
/// Quanto cada faixa viaja — o alvo, e o número contra o qual todo pico é lido.
const TRAVEL: f64 = X_HOVER - X_REST;
const BOX_W: f64 = 1.6;
const BOX_H: f64 = 1.0;

/// A marca de alvo: fina, alta e **imóvel**.
const MARK_W: f64 = 0.07;
const MARK_H: f64 = 1.5;

/// A mola: ⚠️ **`ζ = 0,4` é sub-amortecido de propósito** (o default de produto é o crítico
/// `1,0`, que chega sem passar) — uma cena que abrisse no crítico não mostraria a metade que se vê.
/// E `ω = 8` é o compromisso medido: o voo dura o suficiente para a mão o interromper, sem
/// arrastar.
const SPRING_CFG: Spring = Spring {
    stiffness: 8.0,
    damping: 0.4,
};

/// A duração das três faixas de CURVA. ⚠️ **Igual nas três**, senão o A/B compararia dois números
/// ao mesmo tempo; e maior que os 150 ms de fábrica porque a reversão a meio é um gesto de mão.
const CURVE_SECONDS: f64 = 0.6;

const REST: [[u8; 3]; 4] = [[46, 62, 96], [96, 56, 60], [46, 84, 66], [70, 70, 78]];
const HOVER: [[u8; 3]; 4] = [
    [92, 154, 236],
    [232, 128, 132],
    [96, 200, 150],
    [176, 176, 190],
];
const MARK_RGB: [u8; 3] = [150, 150, 160];

pub(crate) fn frame(app: &mut crate::App, f: u32) {
    match f {
        3 => build(app),
        // Num frame POSTERIOR: a entidade de uma forma nasce no `vec_entities::sync`, que corre no
        // frame do desenho. Nomear antes seria escrever num objeto que ainda não existe.
        5 => name_them(app),
        7 => record(app, StateRole::Default),
        9 => pose_hover(app),
        11 => record(app, StateRole::Hover),
        13 => back_to_rest(app),
        // ⚠️ O timing vai DEPOIS do repouso: com a mola já armada, a volta ao Default seria ela
        // própria uma animação de dois segundos, e a cena abriria a mexer-se sozinha.
        15 => set_timing(app),
        17 => announce(app),
        _ => {}
    }
}

fn build(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    for (i, (_, y)) in LANES.iter().enumerate() {
        let mut p = rectangle([X_REST, y - BOX_H * 0.5], [X_REST + BOX_W, y + BOX_H * 0.5]);
        let c = REST[i];
        p.fill = Some(Paint::Solid(Rgba8::new(c[0], c[1], c[2], 255)));
        gfx.vec_scene.push_path(p);
    }
    // As marcas vêm DEPOIS das formas: os primeiros `LANES.len()` ids são os hospedeiros, e é
    // isso que deixa `hosts()` ser uma fatia em vez de um filtro.
    for (_, y) in LANES {
        let mut m = rectangle(
            [X_HOVER, y - MARK_H * 0.5],
            [X_HOVER + MARK_W, y + MARK_H * 0.5],
        );
        m.fill = Some(Paint::Solid(Rgba8::new(
            MARK_RGB[0],
            MARK_RGB[1],
            MARK_RGB[2],
            255,
        )));
        gfx.vec_scene.push_path(m);
    }
}

fn path_ids(app: &crate::App) -> Vec<VecPathId> {
    app.gfx
        .as_ref()
        .map(|g| g.vec_scene.paths().iter().map(|p| p.id).collect())
        .unwrap_or_default()
}

/// Os ids dos quatro hospedeiros — as marcas ficam de fora, e é isso que as mantém imóveis.
fn hosts(app: &crate::App) -> Vec<VecPathId> {
    let ids = path_ids(app);
    if ids.len() < LANES.len() * 2 {
        return Vec::new();
    }
    ids[..LANES.len()].to_vec()
}

fn name_them(app: &mut crate::App) {
    let ids = path_ids(app);
    if ids.len() < LANES.len() * 2 {
        return;
    }
    let ents: Vec<_> = ids
        .iter()
        .map(|id| {
            app.vec_entities
                .get(id)
                .map(|&b| ph2d_ecs::Entity::from_bits(b))
        })
        .collect();
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    for (i, (name, _)) in LANES.iter().enumerate() {
        if let Some(e) = ents[i]
            && let Ok(mut ent) = gfx.sim.world_mut().get_entity_mut(e)
        {
            ent.insert(ph2d_ecs::Name::new(*name));
        }
    }
    for (i, (name, _)) in LANES.iter().enumerate() {
        if let Some(e) = ents[LANES.len() + i]
            && let Ok(mut ent) = gfx.sim.world_mut().get_entity_mut(e)
        {
            ent.insert(ph2d_ecs::Name::new(format!("{name} target")));
        }
    }
}

/// ⚠️ Pela porta do PRODUTO (`vec_ui_state_edit::apply`), e não escrevendo a tabela à mão: uma
/// cena que semeia estado por baixo pula exactamente a costura que ela existe para provar.
fn record(app: &mut crate::App, role: StateRole) {
    let hosts = hosts(app);
    if hosts.is_empty() {
        return;
    }
    let map = &app.vec_entities;
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    for h in hosts {
        crate::vec_ui_state_edit::apply(
            &mut gfx.sim,
            &mut gfx.vec_scene,
            map,
            &[h],
            &mut gfx.ui_states,
            crate::vec_ui_state_edit::UiStateEdit::Record(role),
        );
    }
}

/// Põe as quatro faixas na pose de hover — exactamente o que o artista faria com a mão.
fn pose_hover(app: &mut crate::App) {
    let hosts = hosts(app);
    if hosts.is_empty() {
        return;
    }
    let ents: Vec<_> = hosts
        .iter()
        .map(|id| {
            app.vec_entities
                .get(id)
                .map(|&b| ph2d_ecs::Entity::from_bits(b))
        })
        .collect();
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    #[allow(clippy::cast_possible_truncation)]
    let travel = (X_HOVER - X_REST) as f32;
    for (i, e) in ents.iter().enumerate() {
        if let Some(e) = *e
            && let Some(mut t) = gfx.sim.world_mut().get_mut::<ph2d_ecs::Transform>(e)
        {
            t.translation.x += travel;
        }
        if let Some(p) = gfx.vec_scene.path_mut(hosts[i]) {
            let c = HOVER[i];
            p.fill = Some(Paint::Solid(Rgba8::new(c[0], c[1], c[2], 255)));
        }
    }
}

/// Devolve a cena ao repouso pela porta do produto — a pose que o artista vê ao abrir.
fn back_to_rest(app: &mut crate::App) {
    let hosts = hosts(app);
    for h in hosts {
        let Some(gfx) = app.gfx.as_mut() else { return };
        crate::render_loop::ui_state_bridge::request(
            &mut gfx.ui_machines,
            &gfx.ui_states,
            h,
            StateRole::Default,
        );
    }
}

/// A curva de cada faixa. `None` = mola.
fn drive_of(lane: usize) -> Option<Easing> {
    match lane {
        SPRING => None,
        CURVE => Some(Easing::new(EasingFamily::Cubic, EasingMode::InOut)),
        BACK => Some(Easing::new(EasingFamily::Back, EasingMode::Out)),
        _ => Some(Easing::new(EasingFamily::Cubic, EasingMode::Out)),
    }
}

/// Arma o motor de cada faixa. ⚠️ Pelas portas do produto (`set_spring`/`set_easing`), que são as
/// mesmas que os controles do painel chamam.
fn set_timing(app: &mut crate::App) {
    let hosts = hosts(app);
    if hosts.is_empty() {
        return;
    }
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    for (i, h) in hosts.iter().enumerate() {
        match drive_of(i) {
            None => gfx.ui_states.set_spring(*h, Some(SPRING_CFG)),
            Some(e) => {
                gfx.ui_states.set_easing(*h, e);
                gfx.ui_states.set_duration(*h, CURVE_SECONDS);
            }
        }
    }
}

/// **Quão longe cada faixa vai** — medido a andar uma máquina própria, sem tocar na cena.
///
/// ⚠️ Máquina PRÓPRIA e não a viva: medir com a do produto escreveria o mundo, e a cena abriria
/// numa pose que ninguém pediu. O que se lê aqui é a mesma tabela que a ponte lê.
fn peak_x(app: &crate::App, host: VecPathId, lane: usize) -> f64 {
    let Some(gfx) = app.gfx.as_ref() else {
        return f64::NAN;
    };
    let Some(mut m) = Machine::new(gfx.ui_states.get(host).to_vec()) else {
        return f64::NAN;
    };
    match drive_of(lane) {
        None => m.go_to_role_spring(StateRole::Hover, SPRING_CFG),
        Some(e) => m.go_to_role(StateRole::Hover, CURVE_SECONDS, e),
    }
    let mut hi = f64::NEG_INFINITY;
    for _ in 0..600 {
        m.advance(1.0 / 60.0);
        for p in m.pose() {
            hi = hi.max(p.translation[0]);
        }
        if !m.is_animating() {
            break;
        }
    }
    hi
}

fn announce(app: &mut crate::App) {
    let hosts = hosts(app);
    if hosts.len() < LANES.len() {
        eprintln!("[spring] !! a cena NAO montou — PARE.");
        return;
    }
    let Some(gfx) = app.gfx.as_ref() else { return };
    let poses: usize = hosts.iter().map(|h| gfx.ui_states.get(*h).len()).sum();
    let peaks: Vec<f64> = hosts
        .iter()
        .enumerate()
        .map(|(i, h)| peak_x(app, *h, i))
        .collect();
    // O alvo é a translação que o Hover gravou — o mesmo número para as quatro faixas.
    let target = TRAVEL;
    eprintln!(
        "[spring] {poses} poses gravadas (4 faixas x Default+Hover); o alvo e' x = {target:.2}."
    );
    for (i, (name, _)) in LANES.iter().enumerate() {
        let over = (peaks[i] - target) / target * 100.0;
        eprintln!(
            "[spring]   {name:<8} pico x = {:.3}  ({over:+.1}%)",
            peaks[i]
        );
    }
    // ⚠️ Os DOIS números que tornam a cena válida, e eles afirmam coisas opostas.
    let mut bad = false;
    if peaks[SPRING] <= target * 1.02 {
        eprintln!("[spring] !! a MOLA nao passou da marca — o clamp por canal nao landou. PARE.");
        bad = true;
    }
    if peaks[BACK] <= target * 1.02 {
        eprintln!(
            "[spring] !! o BACK nao passou da marca — a mudanca desta wave nao landou. PARE."
        );
        bad = true;
    }
    if (peaks[LANES.len() - 1] - target).abs() > 1.0e-6 {
        eprintln!(
            "[spring] !! o CONTROLE passou da marca: uma curva contida em [0,1] tem de ficar \
             BYTE-IDENTICA ao que ja' shipava. PARE."
        );
        bad = true;
    }
    if bad {
        return;
    }
    crate::smoke_script::script(
        "spring",
        "as quatro faixas já estão gravadas e armadas",
        STEPS,
    );
}

const STEPS: &[Step] = &[
    Step {
        verb: "LIGUE A PREVIEW",
        lines: &[
            "Na seção UI States do painel, marque Preview.",
            "É o modo em que o rato dirige — e só nele.",
            "A cena entra em repouso: cada faixa vai para o Default.",
        ],
    },
    Step {
        verb: "⭐ A MOLA PASSA DA MARCA",
        lines: &[
            "Passe o cursor pela faixa SPRING (a de cima).",
            "Ela viaja, PASSA o traço cinza e volta para ele.",
            "O traço é o alvo, e ele não se mexe: é contra ele",
            "que o olho mede o overshoot.",
        ],
    },
    Step {
        verb: "⭐⭐ O QUE A MOLA COMPRA — reverta a meio",
        lines: &[
            "Entre na faixa SPRING e SAIA antes de ela chegar.",
            "Ela continua um instante para onde ia, e só então volta:",
            "é o momento a ser carregado para dentro do movimento novo.",
            "",
            "Agora o mesmo na faixa CURVE (Cubic In-Out): ela PARA",
            "e arranca do zero. É o mesmo gesto e a outra resposta.",
        ],
    },
    Step {
        verb: "A DECISÃO — o Back agora passa do alvo",
        lines: &[
            "Passe o cursor pela faixa BACK. Ela recua um pouco,",
            "vai, passa a marca e assenta.",
            "Até esta wave o clamp era global e ela apenas CHEGAVA:",
            "escolher Back ou Elastic desenhava o mesmo que Cubic.",
            "Se você preferir o comportamento antigo, diga — é",
            "o único ponto em que esta wave mexe no easing.",
        ],
    },
    Step {
        verb: "⚠️ O CONTROLE — nada mais mudou",
        lines: &[
            "Passe pela faixa CONTROL (Cubic Out, o default).",
            "Ela chega e para, sem passar da marca — exactamente",
            "como sempre fez. Toda curva contida em [0,1] é",
            "byte-idêntica: o clamp nunca mordia nelas.",
        ],
    },
    Step {
        verb: "AS LINHAS TROCAM, NÃO SOMAM",
        lines: &[
            "Saia da Preview. Selecione a faixa SPRING e olhe a",
            "seção States: há Spring marcado, Stiffness e Damping —",
            "e NÃO há Duration nem Curve. Uma mola não tem duração.",
            "Desmarque Spring: Duration e Curve VOLTAM, com o que",
            "você tinha afinado. Remarcar devolve os dois knobs.",
        ],
    },
    Step {
        verb: "OS DOIS KNOBS FAZEM O QUE O NOME DIZ",
        lines: &[
            "Com Spring marcado, leve Damping ao máximo: entre na",
            "Preview e ela deixa de passar da marca — chega e para.",
            "Devolva-o e leve Stiffness ao máximo: ela chega quase",
            "instantânea, e ainda assim passa.",
        ],
    },
];

#[cfg(test)]
#[path = "ui_spring_smoke_tests.rs"]
mod tests;

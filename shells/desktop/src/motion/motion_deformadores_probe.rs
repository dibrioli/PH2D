//! ⭐⭐ **A AUDITORIA DO GRUPO DOS TRANSFORMES & DEFORMADORES** (ciclo 3, passo 2 — doc 106).
//!
//! Mesma régua dos ciclos 1 e 2: a auditoria começa por **medir o que existe**, nunca por uma
//! lista do que eu acho que falta.
//!
//! ⚠️ **A coluna do dispositivo lê-se do `register_gpu_kernel`, NUNCA do `lowerings`** — o
//! ciclo 2 pagou essa: o `lowerings` é o que o nó declara saber baixar **sozinho**, e imprimir
//! essa coluna fabrica uma tabela de dívida com oito kernels que já existem.
//!
//! ```text
//! cargo test -p ph2d-host-desktop --bins --release -- --ignored --nocapture audit_the_deformer_group
//! ```

use crate::motion::motion_state::MotionState;

/// Os treze do ciclo 3 (doc 103 §5).
pub(crate) const GRUPO: [&str; 13] = [
    "motion.move",
    "motion.rotate",
    "motion.scale",
    "motion.transform",
    "motion.mirror",
    "motion.look_at",
    "motion.bend",
    "motion.twist",
    "motion.spherize",
    "motion.four_point_warp",
    "motion.bezier_warp",
    "motion.kaleidoscope",
    "motion.spline_wrap",
];

#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn audit_the_deformer_group() {
    crate::motion::motion_ciclo_probe::retrato(&GRUPO);
}

/// **OS PARAMS DE CADA NÓ DO GRUPO, um a um** — o que a auditoria compara contra as
/// referências. ⚠️ Sem esta lista, «falta X» é um palpite.
///
/// ```text
/// cargo test -p ph2d-host-desktop --bins -- --ignored --nocapture what_each_deformer_offers
/// ```
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn what_each_deformer_offers() {
    crate::motion::motion_ciclo_probe::params_de(&GRUPO);
}

/// ⭐⭐ **AS ROWS QUE O CARTÃO DE FACTO PINTA, com o RÓTULO que aparece na tela** — ver a porta.
///
/// ```text
/// cargo test -p ph2d-host-desktop --bins -- --ignored --nocapture what_the_card_shows
/// ```
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn what_the_card_shows() {
    crate::motion::motion_ciclo_probe::cartao(&GRUPO);
}

// ---------------------------------------------------------------------------------------------
// ⛔⛔ ONDE UMA CENA ESTÁ — e se o artista consegue lá CHEGAR.
// ---------------------------------------------------------------------------------------------

/// ⛔⛔ **O ALCANCE DA CÂMARA, lido do código.** `ph2d_render::Camera2d` abre em
/// `height_world = 10` e o zoom-out **pára** em `ZOOM_MAX_HEIGHT_WORLD = 100` — metade disso é a
/// distância máxima a que um objecto ainda pode ser trazido ao ecrã, e ⚠️ **só com o zoom no
/// batente**. A barra fica em `50`, que é esse batente, e não num número escolhido.
const ALCANCE_DA_CAMARA: f32 = ph2d_render::Camera2d::ZOOM_MAX_HEIGHT_WORLD * 0.5;

/// A cena publica legenda? Construir é barato; **cozinhar não é** (a cena `=1` tem 2 M
/// elementos), e é por isso que a pergunta vem antes.
fn scene_has_legend(level: u32) -> bool {
    // ⛔⛔ **LIMPA ANTES DE MONTAR, e a trava é FINA.**
    //
    // A legenda é um global do processo e só é reescrita por uma cena que **publique**: sem o
    // `publish(vec![])`, uma cena muda deixava a legenda da ANTERIOR e esta função respondia
    // `true` sobre ela. ⚠️ E a trava fecha só a janela `limpar → montar → ler`: a 1.ª redacção
    // tomava-a à volta da varredura inteira de 112 níveis, e a suíte do shell passou de **72 s
    // para 1 796 s** — *uma trava que protege mais do que o estado partilhado paga o preço de
    // toda a gente*.
    let mut m = MotionState::new();
    let (_, legenda) =
        crate::motion::motion_demo_legend::monta(&level.to_string(), &mut m.doc, &m.registry);
    !legenda.is_empty()
}

/// A caixa que os objectos de uma cena ocupam, em unidades de mundo.
fn scene_bounds(level: u32) -> Option<([f32; 2], [f32; 2], usize)> {
    let mut m = MotionState::new();
    let (sinks, _) =
        crate::motion::motion_demo_legend::monta(&level.to_string(), &mut m.doc, &m.registry);
    let sink = *sinks.first()?;
    crate::render_loop::motion_shape_gen::publish(&mut m, 0.0);
    let out = m
        .pump
        .cook
        .cook(&m.doc.graph, &m.registry, sink, 0.0)
        .ok()?;
    let stream = out.first()?.as_stream();
    let ph2d_nodegraph::attr::Column::Vec2(p) = stream.get("P")? else {
        return None;
    };
    if p.is_empty() {
        return None;
    }
    let mut lo = [f32::INFINITY; 2];
    let mut hi = [f32::NEG_INFINITY; 2];
    for q in p {
        for k in 0..2 {
            lo[k] = lo[k].min(q[k]);
            hi[k] = hi[k].max(q[k]);
        }
    }
    Some((lo, hi, p.len()))
}

/// ⭐⭐⭐ **A CÂMARA NÃO CHEGA A TODA A PARTE, E O NÚMERO É DO CÓDIGO.**
///
/// `ph2d_render::Camera2d` abre em `height_world = 10` e o zoom-out **pára em 100**
/// (`ZOOM_MAX_HEIGHT_WORLD`). ⇒ uma cena cujos objectos vivam para lá disso não está «fora do
/// ecrã»: está **fora do alcance**, e o artista não tem gesto que a encontre.
///
/// ⚠️ **Isto não é uma regra para TODA cena.** As cenas de perf (`=12` é uma grelha de
/// `700 × 700` a `1` unidade de passo) existem para carregar o dispositivo, não para serem
/// lidas — e a esse tamanho o zoom máximo mostra um sétimo delas, de propósito.
///
/// ```text
/// cargo test -p ph2d-host-desktop --bins --release -- --ignored --nocapture where_each_demo_scene_lives
/// ```
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn where_each_demo_scene_lives() {
    eprintln!(
        "\n  cena | objectos | x                  | y                  | legenda | cabe em 100?"
    );
    eprintln!(
        "  -----|----------|--------------------|--------------------|---------|-------------"
    );
    for level in 1..=crate::motion::motion_state::demo_router::MAX_DEMO_LEVEL {
        let Some((lo, hi, n)) = scene_bounds(level) else {
            continue;
        };
        let legenda = !crate::motion::motion_demo_legend::captions().is_empty();
        let alcance = hi[0]
            .abs()
            .max(lo[0].abs())
            .max(hi[1].abs())
            .max(lo[1].abs());
        let cabe = alcance <= 50.0;
        if legenda || !cabe {
            eprintln!(
                "  {level:>4} | {n:>8} | {:>8.1}..{:<8.1} | {:>8.1}..{:<8.1} | {:^7} | {}",
                lo[0],
                hi[0],
                lo[1],
                hi[1],
                if legenda { "sim" } else { "-" },
                if cabe { "sim" } else { "NAO" },
            );
        }
    }
    eprintln!();
}

/// ⭐⭐⭐ **UMA CENA QUE PÕE LEGENDA TEM DE CABER NO ALCANCE DA CÂMARA** (ciclo 3 — doc 106).
///
/// ⛔⛔ **O report que a fez existir:** *«a simulação funciona nos nós mas não aparece no canvas»*
/// (Enio, 2026-09-08). Os cartões cozinhavam `180 000` objectos, cada pré-visualização de nó
/// tinha pontos, e o canvas estava vazio — porque a cena `=111` nascia com o pano entre `300` e
/// `600` unidades de mundo, e **a câmara não chega lá**: ela abre em `10` e o zoom-out pára em
/// `100`. *Não estava fora do ecrã; estava fora do ALCANCE, e nenhum gesto a encontrava.*
///
/// ⚠️⚠️ **E o HUD não ajudava — dizia `0 inst`, o que quase mandou a investigação para o sítio
/// errado.** Aquele contador conta **entidades `RenderInstance` do ECS**, e uma cena de motion
/// residente no dispositivo não cria nenhuma: ele lê `0` em toda cena de motion que funciona.
///
/// ## A régua é DERIVADA, não uma lista
///
/// ⚠️ Isto **não** vale para toda cena, e a distinção não é de gosto: as cenas de perf
/// (`=1`, `=2`, `=6`, `=7`, `=12`..`=16`) vivem a `180`–`800` unidades **de propósito** — elas
/// existem para carregar o dispositivo, e ninguém as lê. O que separa umas das outras é a
/// **LEGENDA**: uma cena que pousa uma ficha no canvas está, por construção, a dizer *«alguém vai
/// ler isto»*.
///
/// Medido no dia em que este gate nasceu: **29 cenas com legenda, todas dentro de `±11`**
/// unidades; **9 sem legenda, todas entre `180` e `800`**; e a `=111` — esta — era a única com
/// legenda do lado errado. *A partição não foi escolhida: ela estava lá.*
///
/// A tabela inteira sai de `where_each_demo_scene_lives`.
#[test]
fn a_scene_with_a_legend_fits_inside_what_the_camera_can_reach() {
    let mut lidas = 0usize;
    let mut fora: Vec<String> = Vec::new();
    for level in 1..=crate::motion::motion_state::demo_router::MAX_DEMO_LEVEL {
        if !scene_has_legend(level) {
            continue;
        }
        lidas += 1;
        let Some((lo, hi, _)) = scene_bounds(level) else {
            continue;
        };
        let alcance = hi[0]
            .abs()
            .max(lo[0].abs())
            .max(hi[1].abs())
            .max(lo[1].abs());
        if alcance > ALCANCE_DA_CAMARA {
            fora.push(format!(
                "cena {level}: x {:.1}..{:.1} · y {:.1}..{:.1} (alcance {alcance:.1} >                  {ALCANCE_DA_CAMARA})",
                lo[0], hi[0], lo[1], hi[1]
            ));
        }
    }
    // Controlo positivo: um censo que casasse zero passaria vaziamente, e é exactamente o que
    // aconteceria se o `build_level` deixasse de publicar legendas.
    //
    // ⛔⛔ **O piso era `25`, e esse número estava CALIBRADO SOBRE A CONTAMINAÇÃO** (medido
    // 2026-09-09). A legenda é um global do processo e só é reescrita por uma cena que
    // **publique**: sem limpar antes de montar, uma cena **muda** herdava a legenda da anterior
    // e era contada como tendo uma. Quando a [`crate::motion::motion_demo_legend::monta`] passou a
    // limpar, a contagem caiu de `25+` para **`21`** — e o gate acusou, correctamente, a sua
    // própria calibração.
    //
    // ⚠️ **Baixar o piso não é afrouxar a barra aqui:** `21` é o número de cenas que de facto
    // publicam uma legenda; os outros quatro eram ecos. *Um controlo positivo calibrado enquanto
    // o leitor estava contaminado encoda a contaminação.*
    assert!(
        lidas >= 21,
        "so' {lidas} cena(s) com legenda foram vistas — a varredura foi as cegas"
    );
    assert!(
        fora.is_empty(),
        "{} cena(s) com legenda vivem fora do alcance da camara — o artista NAO tem gesto que \
         as encontre:\n  {}",
        fora.len(),
        fora.join("\n  ")
    );
}

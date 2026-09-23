//! **A ponte do HUD** (TOP-20 #20) — a raiz de um placar cola-se à vista da câmera do jogo.
//!
//! Irmã do [`crate::script_bridge`] e pela mesma lei: o que um motor escreve agora é
//! **pré-visualização**. A pose de um [`UiCanvas`] é reescrita a cada quadro (é isso que o cola à
//! vista) e passa pelo ledger do `ph2d-preview-drive` ⇒ **não entra no ficheiro nem no `Ctrl+Z`**.
//!
//! # ⛔ Sem câmera de jogo, NADA é conduzido — e isso é a lei, não uma guarda
//!
//! A vista é a da **câmera do jogo**, nunca a do editor: uma corrida que dependesse de onde o
//! artista rolou o ecrã seria outra corrida em cada máquina (é a razão escrita no
//! `fase_game_camera`, e o `DestroyOutside` do #12 já a herdou). Sem câmera na cena não há vista,
//! e então o canvas fica **onde o artista o pôs**, e o painel diz porquê.
//!
//! ⛔⛔ **E quem o devolve é esta ponte, nunca o `settle`** (auditoria 26 da paralaxe, §1.2, que
//! achou a mesma frase falsa aqui): o `settle` só ESQUECE, e a pose conduzida ficava no mundo e
//! entrava no documento na captura seguinte. Hoje quem esta ponte deixa de conduzir é LARGADO por
//! `release_to_authored`.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, SimWorld, Transform, UiCanvas};
use ph2d_hud::{Canvas, place};

/// ⭐ **A vista re-exportada**, para quem compõe não precisar de declarar a folha só para
/// nomear um rectângulo. *Uma segunda dependência por um nome de tipo é ruído no manifesto.*
pub use ph2d_hud::{Gesto, View, clique};
use ph2d_preview_drive::{Driven, Driver, PreviewDrive};

/// **Quantos canvas há na cena** — o número que o painel mostra quando quer dizer *«isto não está
/// a ser conduzido por ninguém»*.
#[must_use]
pub fn canvas_count(sim: &mut SimWorld) -> usize {
    let world = sim.world_mut();
    world.query::<&UiCanvas>().iter(world).count()
}

/// ⭐ **Um quadro do HUD.** Devolve quantas raízes foram conduzidas.
///
/// ⚠️ **A rotação e o *skew* autorados SOBREVIVEM** — só a translação e a escala são derivadas.
/// Um HUD inclinado é uma decisão de desenho que o artista pode tomar, e apagá-la por arrasto
/// tornaria o controlo inalcançável; a posição e o tamanho, esses, são a razão de este passe
/// existir.
pub fn drive_canvases(sim: &mut SimWorld, vista: Option<View>, drive: &mut PreviewDrive) -> usize {
    let antes: Vec<(Entity, Transform, UiCanvas)> = {
        let world = sim.world_mut();
        world
            .query::<(Entity, &Transform, &UiCanvas)>()
            .iter(world)
            .map(|(e, t, c)| (e, *t, *c))
            .collect()
    };
    let Some(vista) = vista else {
        // ⛔ Sem câmera não há condução — e quem já era conduzido é DEVOLVIDO (ver o cabeçalho).
        largar_os_nao_declarados(sim, drive, &[]);
        return 0;
    };
    let mut n = 0;
    let mut declarados: Vec<Entity> = Vec::new();
    for (entity, era, cfg) in antes {
        let Some(caixa) = Canvas::new(cfg.ref_w, cfg.ref_h, cfg.fit) else {
            // Uma caixa impossível (lado zero ou não-finito) não conduz nada — ver o doc do
            // `Canvas::new`. O painel é que a acusa.
            continue;
        };
        let p = place(&caixa, vista);
        let agora = Transform {
            translation: Vec2::new(p.translate[0], p.translate[1]),
            scale: Vec2::new(p.scale[0], p.scale[1]),
            ..era
        };
        if agora != era {
            if let Some(mut t) = sim.world_mut().get_mut::<Transform>(entity) {
                *t = agora;
            }
            drive.driven(entity, Driven::CanvasPose(era), Driven::CanvasPose(agora));
            declarados.push(entity);
        } else if drive.still_driving(entity, Driver::CanvasPose) {
            // ⚠️ A linha do meio da tabela do ledger: o motor continua a conduzir, e o facto de a
            // pose não ter mudado neste quadro não pode fazê-la voltar a ser documento.
            drive.driven(entity, Driven::CanvasPose(agora), Driven::CanvasPose(agora));
            declarados.push(entity);
        }
        n += 1;
    }
    largar_os_nao_declarados(sim, drive, &declarados);
    n
}

/// ⭐⭐ **Devolve o autorado a quem esta ponte deixou de conduzir** — a varredura é do LEDGER, porque
/// um canvas cujo componente saiu já não aparece na consulta.
fn largar_os_nao_declarados(sim: &mut SimWorld, drive: &mut PreviewDrive, declarados: &[Entity]) {
    for e in drive.driven_by(Driver::CanvasPose) {
        if !declarados.contains(&e) {
            drive.release_to_authored(sim, e, Driver::CanvasPose);
        }
    }
}

/// ⭐⭐⭐ **A MOLDURA que um canvas de HUD oferece às âncoras** — a caixa efectiva em unidades
/// LOCAIS dele, e a escala que leva essas unidades ao mundo.
///
/// Um valor só porque as duas nunca fazem sentido separadas — é a mesma razão que o `Measured` do
/// passe das âncoras escreve para o par dele.
///
/// # ⛔⛔ Porque isto existe: a âncora estava INERTE, e não por ligar
///
/// O [handoff do #20](../../../docs/Components/handoffs/HANDOFF_INTEGRACAO_line_components_HUD_2026-09-17.md)
/// §7 escreve *«as quatro âncoras do `VecAnchors` não estão LIGADAS ao canvas»*, e a sonda do §5.0
/// corrige a redacção: **elas estão ligadas e são inertes.** O `delta_local` pergunta *«a moldura
/// mudou de tamanho?»* e a caixa de um canvas é a MESMA em toda janela — o que muda é a ESCALA da
/// raiz ⇒ o delta saía `0,0` por subtracção de iguais. *A régua media uma grandeza que não se
/// mexe.*
///
/// ⚠️ **A escala é a que CONDUZIU a raiz** ([`place`]), e não uma segunda medição: é a mesma porta
/// que o [`drive_canvases`] usou para escrever o `Transform` **neste quadro**.
#[must_use]
pub fn anchor_frame_of(cfg: UiCanvas, vista: View) -> Option<([f64; 4], [f64; 2])> {
    let caixa = Canvas::new(cfg.ref_w, cfg.ref_h, cfg.fit)?;
    let e = ph2d_hud::effective_box(&caixa, vista);
    let p = place(&caixa, vista);
    Some((
        [
            f64::from(e[0]),
            f64::from(e[1]),
            f64::from(e[2]),
            f64::from(e[3]),
        ],
        [f64::from(p.scale[0]), f64::from(p.scale[1])],
    ))
}

#[cfg(test)]
#[path = "hud_bridge_tests.rs"]
mod tests;

/// ⭐⭐⭐ **QUAL botão o dedo tocou** — a lei que o ramo do clique da shell consulta.
///
/// # ⛔⛔ Ela recebe CANDIDATOS e não um ponto, e é isso que a torna testável
///
/// Quem está sob o cursor sai de duas varreduras que precisam de uma `wgpu::Surface` e de uma cena
/// vectorial construída (`path_at` · `pick_sprite_at_world`), logo a shell não é alcançável de um
/// teste. ⇒ a shell colhe os candidatos e a LEI vive aqui, onde um `World` basta — a mesma forma
/// que o [`clique`] da folha já tem.
///
/// # ⭐⭐ A lei: **o PRIMEIRO candidato que SOBE até um botão elegível ganha**
///
/// ⚠️ **E não *«o primeiro candidato»*.** A diferença é o caso que fez esta porta existir: o
/// `path_at` devolve *a forma mais ao topo que contém o ponto* entre as VECTORIAIS, e ela pode não
/// ter nada que ver com um botão (um cenário desenhado, o rótulo de outra coisa). Com a regra
/// ingénua, um botão feito de **sprite** por baixo de qualquer forma vectorial ficava inalcançável
/// — e o dono leria isso como *«o botão não funciona»*, sem nada na tela a distingui-lo de um
/// botão sem nome.
///
/// ⚠️ **A ordem dos candidatos é da SHELL** (primeiro o vectorial, que é o meio próprio do HUD),
/// e essa escolha está escrita lá.
///
/// # ⚠️ A ELEGIBILIDADE mora aqui, no mesmo sítio
///
/// `disabled` e o nome em branco são as **duas maneiras de um botão não ser um botão**, e separá-las
/// de quem o encontra daria dois lugares para decidir. ⛔ E note-se que um candidato que sobe até um
/// botão **inelegível** não bloqueia o seguinte: *um botão desligado é decoração, e decoração não
/// engole um clique que era de outra coisa*.
#[must_use]
pub fn botao_sob_o_cursor(
    world: &bevy_ecs::world::World,
    candidatos: impl IntoIterator<Item = u64>,
) -> Option<Entity> {
    for bits in candidatos {
        let Some(e) = ph2d_ecs::hud::botao_de(world, Entity::from_bits(bits)) else {
            continue;
        };
        if world
            .get::<ph2d_ecs::UiButton>(e)
            .is_some_and(|b| !b.disabled && b.name().is_some())
        {
            return Some(e);
        }
    }
    None
}

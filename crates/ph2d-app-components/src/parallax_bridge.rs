//! **A ponte da PARALAXE** (plano [24](../../../docs/Components/24_plano_paralaxe.md), W1) — um
//! objecto guarda uma FRACÇÃO do movimento do mundo.
//!
//! Irmã do [`crate::hud_bridge`] e pela mesma lei: o que um motor escreve agora é
//! **pré-visualização**. A pose de quem tem [`ScrollFactor`] é recolocada a cada quadro e passa
//! pelo ledger ⇒ **não entra no ficheiro nem no `Ctrl+Z`**.

use ph2d_core::Vec2;
use ph2d_ecs::{
    Entity, ScrollFactor, ScrollLimits, ScrollMotion, ScrollRepeat, SimWorld, Transform, UiCanvas,
};
use ph2d_preview_drive::{Driven, Driver, PreviewDrive};

/// **Quantos objectos da cena têm paralaxe** — o número que o painel mostra.
#[must_use]
pub fn parallax_count(sim: &mut SimWorld) -> usize {
    let world = sim.world_mut();
    world.query::<&ScrollFactor>().iter(world).count()
}

/// ⭐ **Um quadro da paralaxe.** Devolve quantos objectos foram conduzidos.
///
/// `vista` é o rectângulo da câmera do JOGO (centro e meia-janela em metros) — o que a
/// `fase_game_camera` devolveu, ou `None` quando não há câmera de jogo na cena. `playhead` é o
/// tempo da régua em segundos, e ele é a ÚNICA entrada da deriva da W4.
///
/// ⚠️ **Um objecto com deriva é conduzido mesmo com `k` neutro**, e isso é a lei: `k = 1` diz *«não
/// guardo nada do movimento da câmera»*, e a deriva não é movimento da câmera. ⛔ Sem essa metade
/// uma nuvem que anda sozinha num plano normal ficaria parada, e o salto do neutro esconderia-a.
///
/// # ⚠️⚠️ A meia-janela atravessa, e a premissa da W1 MORREU aqui
///
/// A W1 passava **só o centro**, e o gate dela afirmava que o rectângulo não chegava à lei — com a
/// razão certa: *o deslocamento não depende do zoom* (medido no alvo: declive `0,5000` com zoom
/// `1,0`, `2,0` e `0,5`). ⛔ **Isso continua verdade e deixou de descrever esta fronteira:** o
/// CONFINAMENTO da W3 tem o joelho em `(região − ecrã)/2`, logo ele precisa de saber quanto a vista
/// mede.
///
/// ⇒ *o zoom não entra no DESLOCAMENTO; ele entra no CONFINAMENTO*, e são duas perguntas. A
/// primeira continua gateada pela **ASSINATURA** ([`ScrollFactor::deslocamento`] recebe um centro e
/// mais nada); a segunda tem gate próprio.
pub fn drive_parallax(
    sim: &mut SimWorld,
    vista: Option<([f32; 2], [f32; 2])>,
    playhead: f64,
    drive: &mut PreviewDrive,
) -> usize {
    // ⭐⭐⭐ **O DOLLY sai da câmera ACTIVA, e a escolha dela NÃO é reimplementada aqui** — a lei da
    // prioridade com desempate pelo `StableId` vive numa porta (`active_camera_of`), e escrevê-la
    // outra vez seria a segunda resposta a *«qual câmera manda?»*.
    //
    // ⚠️ **Sem câmera activa o dolly é `0`, que é a identidade** — e não uma recusa: a `vista` já
    // respondeu a essa pergunta uma linha abaixo, e duas recusas para o mesmo facto divergem.
    let dolly = ph2d_ecs::active_camera_of(sim.world_mut())
        .and_then(|c| sim.world().get::<ph2d_ecs::GameCamera>(c).map(|g| g.dolly))
        .unwrap_or(0.0);
    type Linha = (
        Entity,
        Transform,
        ScrollFactor,
        Option<ScrollRepeat>,
        Option<ScrollLimits>,
        Option<ScrollMotion>,
    );
    let antes: Vec<Linha> = {
        let world = sim.world_mut();
        world
            .query_filtered::<(
                Entity,
                &Transform,
                &ScrollFactor,
                Option<&ScrollRepeat>,
                Option<&ScrollLimits>,
                Option<&ScrollMotion>,
            ), bevy_ecs::prelude::Without<UiCanvas>>()
            .iter(world)
            .map(|(e, t, k, r, l, m)| (e, *t, *k, r.copied(), l.copied(), m.copied()))
            .collect()
    };
    let Some((centro, meia)) = vista else {
        // ⛔ Sem câmera não há condução — e quem já era conduzido é DEVOLVIDO (ver o cabeçalho).
        largar_os_nao_declarados(sim, drive, &[]);
        return 0;
    };
    let mut n = 0;
    let mut declarados: Vec<Entity> = Vec::with_capacity(antes.len());
    for (entity, era, cfg, rep, lim, mov) in antes {
        if cfg.e_neutro() && mov.is_none_or(|m| m.e_inerte()) {
            // ⛔ `k = 1` é o objecto do mundo: não se escreve um bit e **não se declara**, senão
            // toda cena com o componente anexado passaria a ter uma entrada viva no ledger. Quem
            // era conduzido e voltou ao neutro é largado pela varredura do fim.
            continue;
        }
        let autorada = base_autorada(drive, entity, era);
        // ⭐⭐ **A REPETIÇÃO envolve a posição RELATIVA À VISTA** (auditoria 26, §1.1): a correcção
        // é um número INTEIRO de ladrilhos, e o que ela mantém limitado é onde a camada aparece no
        // ECRÃ — não a pose no mundo. ⚠️ A 1.ª redacção envolvia o deslocamento `c·(1−k)`, e isso
        // prendia a camada a meio ladrilho da pose autorada no MUNDO: a fileira saía da vista.
        //
        // ⚠️ **Sem o componente a composição é a IDENTIDADE**, e não um caso à parte: a ausência é
        // «não repete», que é o que quase todo objecto com paralaxe faz.
        // ⭐⭐ **A ORDEM é CONFINAR e depois ENVOLVER:** o confinamento escolhe o centro que a
        // camada vê, e a repetição reduz o resultado por um inteiro de ladrilhos. ⚠️ Os dois juntos
        // são uma cena que se contradiz — um fundo que repete não tem borda para esconder —, e a
        // repetição, que vem por último, ganha.
        // ⭐⭐⭐ **O DOLLY é uma ESCALA à volta do centro da vista** (W5, corrigido na auditoria
        // 26, §1.4). ⛔ Com a câmera a ATRAVESSAR a camada (ou no plano focal) não há resposta, e
        // o objecto é LARGADO na pose autorada pela varredura do fim — *um clamp ali entregaria um
        // número plausível para uma cena impossível*.
        let Some(esc) = cfg.escala(dolly) else {
            continue;
        };
        // ⭐⭐ **A DERIVA é um SOMANDO e nunca um segundo condutor** — medido no
        // [`super::w4_probe`]: dois motores sobre o mesmo `Transform` entram no ledger com chaves
        // diferentes, e esta ponte leria a escrita do outro como um arrasto do artista.
        let deriva = mov.map_or([0.0, 0.0], |m| m.deslocamento(playhead));
        // ⭐⭐⭐ **A lei inteira vive numa função por eixo** ([`ph2d_ecs::scroll_factor::saida_eixo`])
        // — o confinamento com a meia-vista que a camada VÊ através do dolly, a repetição presa à
        // VISTA e o dolly à volta do centro da vista (auditoria 26, §1.1, §1.4 e §2.3).
        // ⭐ O ladrilho é LOCAL (a arte antes da escala): no mundo ele mede `tile · |escala|`, a
        // autorada vezes o dolly — sem isto uma Background Layer escalada envolvia por meio ladrilho
        // e a costura saltava (plano GEF 11 «aberto», curado na 19).
        let escala = [autorada.scale.x * esc[0], autorada.scale.y * esc[1]];
        let eixo = |i: usize| {
            let conf = lim.map_or(centro[i], |l| {
                ph2d_ecs::confina_eixo(
                    centro[i],
                    ph2d_ecs::meia_da_camada(meia[i], cfg.k[i], esc[i]),
                    l.min[i],
                    l.max[i],
                )
            });
            ph2d_ecs::scroll_factor::saida_eixo(
                cfg.k[i],
                if i == 0 {
                    autorada.translation.x
                } else {
                    autorada.translation.y
                },
                centro[i],
                conf,
                deriva[i],
                esc[i],
                rep.map(|r| r.tile[i] * escala[i].abs()),
            )
        };
        let (x, y) = (eixo(0), eixo(1));
        let agora = Transform {
            translation: Vec2::new(x, y),
            // ⚠️⚠️ **A ESCALA multiplica SEMPRE a AUTORADA** (auditoria 26, §1.3). A 1.ª redacção
            // escrevia o VIVO com `esc = 1`, e o vivo depois de um dolly ainda é a escala do
            // dolly: voltar o dolly a `0` deixava os fundos encolhidos para sempre. Com `esc = 1`
            // a multiplicação é a identidade ao bit, logo a W1..W4 não se mexem.
            scale: Vec2::new(escala[0], escala[1]),
            // ⚠️ O resto vem do VIVO e não do autorado: esta ponte não escreve a rotação nem o
            // skew, e roubá-los ao autorado apagaria o que outro motor tivesse escrito.
            ..era
        };
        let escreveu = agora != era;
        if escreveu && let Some(mut t) = sim.world_mut().get_mut::<Transform>(entity) {
            *t = agora;
        }
        // ⚠️⚠️ **O `before` é o AUTORADO e nunca o vivo.** Esta é a diferença de fundo para a
        // ponte do HUD, que passa `era`: ali a pose é função PURA da vista e o autorado nunca é
        // lido, aqui ele é a ENTRADA da lei — declarar o vivo escreveria a pose deslocada no
        // documento, e no quadro seguinte o fundo saltaria o deslocamento outra vez.
        //
        // ⚠️ E a condição é `escreveu || still_driving`, não `if/else`: o `still_driving` NUNCA
        // cria uma entrada (ver o doc dele), logo com a câmera na origem e nada escrito não nasce
        // condução nenhuma — o fundo está exactamente onde o artista o pôs. Mas se já havia
        // condução, ela tem de ser MANTIDA mesmo num quadro em que a vista não andou: esta ponte é
        // um condutor PERSISTENTE, e para ele *«não mudou»* e *«acabou»* são factos diferentes com
        // a mesma forma.
        if escreveu || drive.still_driving(entity, Driver::ParallaxPose) {
            drive.driven(
                entity,
                Driven::ParallaxPose(autorada),
                Driven::ParallaxPose(agora),
            );
            declarados.push(entity);
        }
        n += 1;
    }
    largar_os_nao_declarados(sim, drive, &declarados);
    n
}

/// ⭐⭐⭐ **Devolve o autorado a quem esta ponte deixou de conduzir** (auditoria 26, §1.2).
///
/// ⚠️ A varredura é do LEDGER e não da consulta, e isso é a lei: um objecto a quem tiraram o
/// `ScrollFactor` (ou que ganhou um `UiCanvas`) já não aparece na consulta — só o ledger se lembra
/// de que ele foi deslocado.
fn largar_os_nao_declarados(sim: &mut SimWorld, drive: &mut PreviewDrive, declarados: &[Entity]) {
    for e in drive.driven_by(Driver::ParallaxPose) {
        if !declarados.contains(&e) {
            drive.release_to_authored(sim, e, Driver::ParallaxPose);
        }
    }
}

/// **De que pose se parte** — ver o ⚠️ do cabeçalho sobre o arrasto.
///
/// # ⭐⭐⭐ O deslocamento é MEDIDO (`escrito − autorado`), nunca re-derivado da vista
///
/// A 1.ª redacção comparava o vivo com `desloca(memo, centro)` — o que esta função ESCREVERIA
/// agora — e recuperava o autorado subtraindo esse mesmo número. ⛔ **Está errado e falha mudo:**
/// o que está no mundo foi escrito com a vista do quadro ANTERIOR, logo com a câmera a andar as
/// duas nunca coincidem, toda leitura cai no ramo do arrasto, e o «autorado recuperado» anda com
/// a câmera na direcção oposta. Medido: o declive lia **`0,000`** para `k = 0` (o fundo parado no
/// ecrã, que é o caso que a paralaxe existe para produzir) com seis dos oito gates VERDES.
///
/// ⇒ o deslocamento que ESTA ponte aplicou é `last_written − authored`, e o ledger tem os dois.
/// Ele não depende de vista nenhuma, logo a recuperação é exacta mesmo entre duas vistas
/// diferentes.
///
/// ⛔⛔ **«Nada mexeu» é um RAMO, e a 1.ª redacção dizia o contrário** (auditoria 26, §2.1): ela
/// argumentava que a subtracção *«devolve o memo ao bit»* — e em `f32` `x − (x − m) ≠ m` em geral.
/// Medido pela ponte real: com a câmera a andar o autorado saía do valor original ao 5.º quadro e
/// ficava diferente em `5 995` de `6 000` ⇒ um passo de undo espúrio por quadro com input, e a
/// pose no ficheiro a derivar. ⇒ com o vivo IGUAL ao que a ponte escreveu, o autorado é o memo,
/// sem aritmética nenhuma; a recuperação só corre quando outra mão mexeu.
fn base_autorada(drive: &PreviewDrive, entity: Entity, era: Transform) -> Transform {
    let bits = entity.to_bits();
    let (Some(Driven::ParallaxPose(memo)), Some(Driven::ParallaxPose(escrito))) = (
        drive.authored(bits, Driver::ParallaxPose),
        drive.last_written(bits, Driver::ParallaxPose),
    ) else {
        // 1.º quadro deste objecto: o que está no mundo É o autorado.
        return era;
    };
    if era == escrito {
        return memo;
    }
    Transform {
        translation: Vec2::new(
            era.translation.x - (escrito.translation.x - memo.translation.x),
            era.translation.y - (escrito.translation.y - memo.translation.y),
        ),
        // ⚠️ **A escala recupera-se por RAZÃO e não por diferença**, porque a lei do dolly a
        // MULTIPLICA. ⛔ Uma escala escrita a zero não tem volta — ali o autorado é o vivo, que é o
        // valor conservador: *dividir por zero devolveria um infinito com cara de pose*.
        scale: Vec2::new(
            razao(era.scale.x, escrito.scale.x, memo.scale.x),
            razao(era.scale.y, escrito.scale.y, memo.scale.y),
        ),
        ..era
    }
}

/// O inverso multiplicativo do «subtrair o deslocamento» — ver o ⚠️ acima.
fn razao(vivo: f32, escrito: f32, memo: f32) -> f32 {
    if escrito == 0.0 || !escrito.is_finite() {
        return vivo;
    }
    vivo * (memo / escrito)
}

#[cfg(test)]
#[path = "parallax_bridge_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "parallax_w4_probe_tests.rs"]
mod w4_probe;

#[cfg(test)]
#[path = "parallax_custo_tests.rs"]
mod custo;

#[cfg(test)]
#[path = "parallax_w6_tests.rs"]
mod w6;

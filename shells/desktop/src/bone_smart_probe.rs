//! ⭐⭐⭐ **A SONDA DO OSSO INTELIGENTE** (`PH2D_BONE_SMART_PROBE=1`, sobre a cena
//! `PH2D_VEC_BONE_SMOKE=1`) — o aparelho que o report *«tudo configurado e a animação não rodou ao
//! rotacionar o bone»* (Enio, 2026-09-08) pede.
//!
//! # Por que ela existe, e por que os gates não bastaram
//!
//! A cadeia inteira já está medida **fora** do app: `the_smoke_chain_moves_the_leaf_end_to_end`
//! monta a acção da cena pela porta da cena (`seed_demo_action`), põe o `SmartBone` com o default do
//! painel, gira o osso um quarto de volta e vê a folha subir. ⇒ **o motor está ilibado**, e o que
//! sobra é o **QUADRO** — a ordem dos passes, o que os escreve por cima, e o que resolve (ou não)
//! nesta sessão.
//!
//! ⚠️ Nada disso é alcançável de um teste: a `App` segura uma surface de janela real.
//!
//! # O que ela faz
//!
//! Salta o painel de propósito — ele já foi aprovado pelo dono (*«Painel funcionou OK»*) — e escreve
//! o componente à mão sobre o osso do tentáculo, exactamente como o painel o escreveria. Depois
//! **gira o osso** um pouco por quadro e imprime, por quadro, a única linha que separa as causas.
//!
//! # Como se lê
//!
//! ```text
//! [probe-smart] f=NN rot=+12.3° | accao "Leaf Rises" (clip 1, activo 0, 1 track) |
//!               alvo bits=4294967298 vivo=true y=+0.412
//! ```
//!
//! - `accao … clip N` ausente ⇒ **o clip foi PURGADO** do documento (a binding perdeu o objecto e a
//!   purga levou a track — e, sendo a última, o documento inteiro).
//! - `vivo=false` ⇒ a binding aponta para bits que já não existem: o objecto foi re-criado com bits
//!   novos e o heal por identidade não o alcançou.
//! - `y` **parado** com `rot` a subir ⇒ o passe corre e alguém escreve por cima **depois** dele.
//! - `y` a subir aqui e parado no gesto do dono ⇒ a diferença está na CONFIGURAÇÃO dele, não no
//!   quadro, e a linha diz qual (a faixa, o clip, o alvo).

use std::sync::atomic::{AtomicU32, Ordering};

/// O quadro corrente do roteiro — a sonda não pode acrescentar campo à `App`.
static FRAME: AtomicU32 = AtomicU32::new(0);

/// Quanto o osso gira por quadro, em radianos — a faixa inteira (`0..90°`) em ~60 quadros.
const PASSO: f32 = 0.08; // LITERAL-PX-OK: ângulo do documento (rad), não medida de UI

impl crate::App {
    /// No prólogo do quadro, ao lado das outras sondas. No-op sem a env.
    pub(crate) fn bone_smart_probe(&mut self) {
        if std::env::var_os("PH2D_BONE_SMART_PROBE").is_none() || self.gfx.is_none() {
            return;
        }
        // ⚠️⚠️ **NÃO se contam QUADROS aqui, pergunta-se o FATO** — a mesma lei que a cena ao lado
        // já pagou, e que esta sonda pagou OUTRA vez: a 1.ª versão armava no quadro `30` e nunca
        // disparou, porque **uma janela em segundo plano redesenha poucas vezes** (medido: ~11
        // quadros em 25 s). *Um roteiro por contagem de quadros mede a taxa de redesenho, não o
        // programa.*
        let armado = self.gfx.as_ref().is_some_and(|g| {
            g.sim
                .world()
                .iter_entities()
                .any(|er| er.get::<ph2d_skeleton_ecs::SmartBone>().is_some())
        });
        if !armado {
            let pronto = self
                .timeline
                .doc
                .clips()
                .iter()
                .any(|c| c.name == crate::vec_bone_smoke::DEMO_ACTION);
            if pronto {
                self.smart_probe_arm();
            }
            return;
        }
        let f = FRAME.fetch_add(1, Ordering::Relaxed);
        self.smart_probe_turn_and_read(f);
    }

    /// Liga um osso da CORRENTE MAIS LONGA da cena à acção que ela traz, com o default do painel.
    ///
    /// ⛔⛔ **A 1.ª versão apanhava `ossos.last()` e mediu OUTRA COISA** (medido 2026-09-08): ela caiu
    /// num osso do **braço**, que está sob a âncora de IK — e uma corrente governada tem a pose
    /// reescrita **depois** deste passe. O sintoma foi a rotação a ficar presa (`rot` constante a
    /// somar `+4,58°` por quadro), e a leitura ingénua disso seria *«o osso inteligente não
    /// funciona»*. *Uma sonda que arma no sujeito errado mede o passe do vizinho.*
    ///
    /// ⇒ a corrente mais longa da cena é o **tentáculo** (6 ossos, sem âncora), e o osso escolhido é
    /// a PONTA dela — abaixo do que tem limite de ângulo, para a parede não aparar o giro.
    fn smart_probe_arm(&mut self) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let e_osso = |e: ph2d_ecs::Entity, w: &ph2d_ecs::World| {
            w.get::<ph2d_skeleton_ecs::Bone>(e).is_some()
        };
        let ossos: Vec<ph2d_ecs::Entity> = gfx
            .sim
            .world()
            .iter_entities()
            .filter(|er| er.get::<ph2d_skeleton_ecs::Bone>().is_some())
            .map(|er| er.id())
            .collect();
        // As RAÍZES: ossos cujo pai não é osso. De cada uma desce-se pelo 1.º filho-osso.
        let mut melhor: Option<(usize, ph2d_ecs::Entity)> = None;
        for &r in &ossos {
            let pai_osso = gfx
                .sim
                .world()
                .get::<ph2d_ecs::ChildOf>(r)
                .is_some_and(|c| e_osso(c.0, gfx.sim.world()));
            if pai_osso {
                continue;
            }
            let (mut e, mut n) = (r, 1usize);
            while let Some(f) = gfx
                .sim
                .world()
                .get::<ph2d_ecs::Children>(e)
                .and_then(|c| c.iter().find(|c| e_osso(**c, gfx.sim.world())).copied())
            {
                e = f;
                n += 1;
            }
            if melhor.is_none_or(|(m, _)| n > m) {
                melhor = Some((n, e));
            }
        }
        let Some((n, osso)) = melhor else {
            eprintln!(
                "[probe-smart] ⛔ a cena nao tem osso nenhum -- corra com PH2D_VEC_BONE_SMOKE=1"
            );
            return;
        };
        eprintln!("[probe-smart] corrente mais longa: {n} osso(s); o controlo e' a ponta dela");
        gfx.sim
            .world_mut()
            .entity_mut(osso)
            .insert(ph2d_skeleton_ecs::SmartBone {
                clip: crate::vec_bone_smoke::DEMO_ACTION.to_string(),
                ..ph2d_skeleton_ecs::SmartBone::default()
            });
        eprintln!(
            "[probe-smart] armado em {osso:?} -- accao \"{}\", faixa 0..90 graus, {} osso(s) na cena",
            crate::vec_bone_smoke::DEMO_ACTION,
            ossos.len()
        );
    }

    /// Gira o controlo um passo e imprime a linha que separa as causas.
    fn smart_probe_turn_and_read(&mut self, f: u32) {
        let doc_clip = self
            .timeline
            .doc
            .clips()
            .iter()
            .position(|c| c.name == crate::vec_bone_smoke::DEMO_ACTION);
        let activo = self.timeline.doc.active_index();
        let tracks = doc_clip.map_or(0, |i| self.timeline.doc.clips()[i].clip.tracks().len());
        let alvo = self.timeline.doc.bindings().first().map(|b| b.entity);
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let Some(osso) = gfx
            .sim
            .world()
            .iter_entities()
            .find(|er| er.get::<ph2d_skeleton_ecs::SmartBone>().is_some())
            .map(|er| er.id())
        else {
            return;
        };
        let rot = if let Some(mut t) = gfx.sim.world_mut().get_mut::<ph2d_ecs::Transform>(osso) {
            t.rotation += PASSO;
            t.rotation
        } else {
            return;
        };
        let (vivo, y) =
            alvo.and_then(ph2d_ecs::Entity::try_from_bits)
                .map_or((false, f32::NAN), |e| {
                    gfx.sim
                        .world()
                        .get::<ph2d_ecs::Transform>(e)
                        .map_or((false, f32::NAN), |t| (true, t.translation.y))
                });
        // ⚠️ Uma linha a cada 4 quadros: por quadro ela afogaria o log do próprio passe
        // (`PH2D_BONE_LOG=1`), que é quem diz *porquê* quando esta diz *que não*. ⛔ E não mais
        // esparsa que isso: uma janela em segundo plano redesenha ~11 vezes em 25 s.
        if f.is_multiple_of(4) {
            eprintln!(
                "[probe-smart] f={f} rot={:+.1}° | accao {} (activo {activo}, {tracks} track(s)) | \
                 alvo bits={:?} vivo={vivo} y={y:+.3}",
                rot.to_degrees(),
                doc_clip.map_or("AUSENTE".to_string(), |i| format!("clip {i}")),
                alvo,
            );
        }
    }
}

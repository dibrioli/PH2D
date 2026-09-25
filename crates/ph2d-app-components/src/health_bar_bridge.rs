//! ⭐⭐⭐ **A PONTE da BARRA DE VIDA** (plano 28, W4) — anda o rasto de cada barra e deixa em
//! [`HealthBarsState::instances`] as três faixas que o passo de sprites desenha.
//!
//! # ⚠️ Porque o estado vive FORA do mundo
//!
//! O rasto muda a cada quadro e é apresentação, não documento: no mundo, cada quadro seria um passo
//! de `Ctrl+Z`. ⇒ um `BTreeMap` chaveado pelos bits da entidade, como os emissores de partículas
//! (`particles_bridge`), com a mesma porta de rebobinar — *rebobinar é renascer*, e um rasto do
//! passado mostraria um golpe que ainda não aconteceu.
//!
//! # ⚠️ De onde vem a vida
//!
//! Do `HealthNow` que a ponte da física publica no fim de cada dispatch; antes do 1.º tique (a
//! corrida parada no zero) ele ainda não existe, e a barra lê a vida com que o objecto NASCE
//! (`Health::start`). *Uma barra vazia antes do Play leria-se como um inimigo morto.*
//!
//! ⚠️ **`max = 0` quer dizer «sem máximo»** na lei da vida. Aí a barra mede contra a vida de
//! nascença, e sem nenhuma das duas não se desenha — *uma fracção sem denominador não é uma barra*.
//!
//! # ⛔ Quem NÃO desenha
//!
//! Um MOLDE (`MasterPiece`: a receita escondida de uma fábrica não é um inimigo da cena), um objecto
//! escondido, e uma barra com vida cheia quando o artista pediu `hide_when_full`.

use std::collections::{BTreeMap, BTreeSet};

use ph2d_ecs::{Entity, MasterPiece, Name, SimWorld, StableId, Visibility, World};
use ph2d_hud::barra::{self, Rasto};
use ph2d_physics_ecs::{Health, HealthBar, HealthNow};
use ph2d_render::RenderInstance;

/// **As barras a correr** — o rasto de cada uma, fora do mundo.
#[derive(Debug, Default)]
pub struct HealthBarsState {
    rastos: BTreeMap<u64, Rasto>,
    /// As instâncias deste quadro, no MUNDO, prontas para o passo de sprites.
    pub instances: Vec<RenderInstance>,
    /// O recorte do átlas que uma faixa desenha — o ladrilho BRANCO (a cor vem do `tint`).
    pub uv: [f32; 4],
    /// ⭐ **O IMPACTO** (plano 28, W5) — a pausa no golpe e os números de dano.
    ///
    /// ⚠️ **Mora AQUI e não ao lado, de propósito:** este estado é *o que a família da vida
    /// apresenta por cima do jogo* — a barra, a pausa, os números —, e os três RENASCEM juntos.
    /// Um terceiro campo solto na shell seria um terceiro sítio a lembrar no rebobinar, que é
    /// exactamente a porta onde as irmãs desta família esqueceram metades (§5 do roteador).
    pub impacto: crate::impacto::ImpactoState,
}

/// **A vida de quem a barra mostra**, como `(agora, máximo)` — `None` se não há vida a ler.
///
/// ⭐ **Porta PÚBLICA com dois leitores** — a ponte ao desenhar e o Inspector ao dizer o que a barra
/// encontrou. *Escrita duas vezes, o painel diria «mostra 30 de 30» sobre uma barra que a tela
/// desenha vazia.*
#[must_use]
pub fn vida_da_barra(w: &World, alvo: Entity) -> Option<(f32, f32)> {
    let h = w.get::<Health>(alvo)?;
    #[allow(clippy::cast_possible_truncation)]
    let agora = w
        .get::<HealthNow>(alvo)
        .map_or(h.start, |n| n.pontos as f32);
    let max = if h.max > 0.0 { h.max } else { h.start };
    (max > 0.0 && max.is_finite()).then_some((agora, max))
}

/// **De quem é a vida** — o próprio objecto (nome vazio), ou o objecto com esse NOME.
///
/// ⚠️ **Entre homónimos, o de menor identidade**, e nunca um molde: a ordem de iteração do mundo não
/// é prometida entre arquétipos, e *«o primeiro»* mudaria com a ordem de criação.
fn alvo_de(
    w: &World,
    dono: Entity,
    nome: &str,
    memo: &mut BTreeMap<String, Option<Entity>>,
) -> Option<Entity> {
    let nome = nome.trim();
    if nome.is_empty() {
        return Some(dono);
    }
    // ⚠️ UMA varredura por NOME por quadro, não uma por barra: mil inimigos com a barra do mesmo
    // herói no placar eram mil varreduras do mundo (a sonda `mede_o_custo_de_n_barras`).
    if let Some(e) = memo.get(nome) {
        return *e;
    }
    let achado = procura_por_nome(w, nome);
    memo.insert(nome.to_owned(), achado);
    achado
}

/// ⭐ **De quem esta barra mostra a vida** — a mesma porta para a ponte e para o Inspector.
/// `None` = ninguém com esse nome (um molde não conta).
#[must_use]
pub fn alvo_da_barra(w: &World, dono: Entity, nome: &str) -> Option<Entity> {
    alvo_de(w, dono, nome, &mut BTreeMap::new())
}

/// O não-molde com este nome e a menor identidade — a varredura que o memo do quadro poupa.
fn procura_por_nome(w: &World, nome: &str) -> Option<Entity> {
    // ⚠️ **A identidade e o molde lêem-se por ACERTO, fora da consulta:** o `try_query` devolve
    // `None` quando QUALQUER componente dela é desconhecido do mundo, e um `Option<&StableId>` conta
    // — num mundo que nunca viu uma identidade, a barra não acharia ninguém e nada o diria (a lição
    // do `tagged`).
    let mut q = w.try_query::<(Entity, &Name)>()?;
    q.iter(w)
        .filter(|(e, n)| {
            n.as_str() == nome
                && w.get::<MasterPiece>(*e).is_none()
                && w.get::<ph2d_ecs::MasterRoot>(*e).is_none()
        })
        .min_by_key(|(e, _)| (w.get::<StableId>(*e).map_or(u64::MAX, |s| s.0), e.to_bits()))
        .map(|(e, _)| e)
}

/// **A pose da barra**: a posição e a ESCALA de quem a carrega, nunca a rotação (ver o componente).
fn pose_de(sim: &SimWorld, dono: Entity) -> Option<([f32; 2], f32)> {
    let t = ph2d_ecs::transform_inverse::world_transform(sim.world(), dono)?;
    let m = ph2d_ecs::GlobalTransform::from_transform(t).matrix;
    let det = m.x_axis.x * m.y_axis.y - m.x_axis.y * m.y_axis.x;
    let s = det.abs().sqrt();
    s.is_finite().then_some(([m.z_axis.x, m.z_axis.y], s))
}

/// Uma faixa chapada do ladrilho branco.
fn faixa(
    centro: [f32; 2],
    tamanho: [f32; 2],
    cor: [f32; 4],
    uv: [f32; 4],
    ordem: u32,
) -> RenderInstance {
    RenderInstance {
        world_pos: centro,
        size: tamanho,
        atlas_uv: uv,
        tint: cor,
        basis: [1.0, 0.0, 0.0, 1.0],
        premultiplied: 0.0,
        anchor: [0.0, 0.0],
        per_corner_tint: [[1.0; 4]; 4],
        opacity: 1.0,
        flip_uv: 0,
        texture_id: RenderInstance::ATLAS_TEXTURE_ID,
        // ⚠️ **À frente de tudo** — uma barra por trás do inimigo que ela descreve não se lê.
        z_order: u32::MAX,
        sampling: 0,
        uv_xform: RenderInstance::IDENTITY_UV_XFORM,
        clip_group: RenderInstance::CLIP_GROUP_NONE,
        clip_meta: 0,
        // Fundo → rasto → vida, pela ordem em que se sobrepõem.
        sub_order: ordem,
    }
}

impl HealthBarsState {
    /// Um estado vazio.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Quantas barras têm rasto vivo.
    #[must_use]
    pub fn live_count(&self) -> usize {
        self.rastos.len()
    }

    /// **O rasto de uma barra agora** (em fracção) — `None` se ela não correu.
    #[must_use]
    pub fn rasto_of(&self, bits: u64) -> Option<f32> {
        self.rastos.get(&bits).map(|r| r.valor)
    }

    /// **O quadro.** `dt_s` é o tempo de JOGO do quadro (zero com o relógio parado).
    pub fn frame(&mut self, sim: &mut SimWorld, dt_s: f32) {
        self.impacto.anda(dt_s);
        self.instances.clear();
        let barras: Vec<(Entity, HealthBar)> = {
            let w = sim.world_mut();
            // ⚠️ **O molde fica de fora pelas DUAS marcas**: o `MasterPiece` é DERIVADO por um passe
            // da shell, e antes do 1.º passe a raiz só tem o `MasterRoot` — sem ela aqui, a barra
            // do molde piscava no meio do ecrã no quadro em que a cena monta.
            let mut q = w.query_filtered::<(Entity, &HealthBar, Option<&StableId>), (
                bevy_ecs::query::Without<MasterPiece>,
                bevy_ecs::query::Without<ph2d_ecs::MasterRoot>,
            )>();
            let mut v: Vec<(u64, Entity, HealthBar)> = q
                .iter(w)
                .map(|(e, b, s)| (s.map_or(u64::MAX, |s| s.0), e, b.clone()))
                .collect();
            // ⚠️ Pela ordem da IDENTIDADE — a mesma cena desenha as barras na mesma ordem.
            v.sort_by_key(|(s, e, _)| (*s, e.to_bits()));
            v.into_iter().map(|(_, e, b)| (e, b)).collect()
        };
        // ⚠️ Por CONJUNTO e não por `any` sobre a lista: a varredura linear dentro do `retain` era
        // `O(n²)` e a sonda mediu-a — 10 000 barras custavam 59× as 1 000.
        let vivas: BTreeSet<u64> = barras.iter().map(|(e, _)| e.to_bits()).collect();
        self.rastos.retain(|bits, _| vivas.contains(bits));
        let mut memo: BTreeMap<String, Option<Entity>> = BTreeMap::new();
        for (dono, cfg) in barras {
            if sim
                .world()
                .get::<Visibility>(dono)
                .is_some_and(|v| v.hidden)
            {
                continue;
            }
            let Some(alvo) = alvo_de(sim.world(), dono, &cfg.target, &mut memo) else {
                continue;
            };
            let Some((agora, max)) = vida_da_barra(sim.world(), alvo) else {
                continue;
            };
            let f = barra::fraccao(agora / max);
            let r = self
                .rastos
                .get(&dono.to_bits())
                .copied()
                .unwrap_or_else(|| Rasto::nasce(f));
            let r = barra::avanca(r, f, dt_s, cfg.trail_delay_s, cfg.trail_speed);
            self.rastos.insert(dono.to_bits(), r);
            if cfg.hide_when_full && f >= 1.0 && r.valor >= 1.0 {
                continue;
            }
            let Some((pos, s)) = pose_de(sim, dono) else {
                continue;
            };
            let centro = [pos[0] + cfg.offset_x * s, pos[1] + cfg.offset_y * s];
            let altura = cfg.height.max(0.0) * s;
            let cores = [cfg.back, cfg.trail, cfg.fill];
            for (i, (cx, w)) in barra::faixas(cfg.width * s, r.valor, f)
                .into_iter()
                .enumerate()
            {
                if w <= 0.0 {
                    continue;
                }
                #[allow(clippy::cast_possible_truncation)]
                self.instances.push(faixa(
                    [centro[0] + cx, centro[1]],
                    [w, altura],
                    cores[i],
                    self.uv,
                    i as u32,
                ));
            }
        }
    }

    /// **Rebobinar é renascer** — todo rasto volta a nascer colado à vida no próximo quadro.
    /// Devolve quantos havia.
    pub fn rewind(&mut self) -> usize {
        let n = self.rastos.len() + self.impacto.rewind();
        self.rastos.clear();
        self.instances.clear();
        n
    }
}

#[cfg(test)]
#[path = "health_bar_bridge_tests.rs"]
mod tests;

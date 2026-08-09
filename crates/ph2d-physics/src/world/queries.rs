//! **As consultas de LEITURA do mundo** — o que o overlay pergunta, e nada mais.
//!
//! Irmão de `sensors` e `contacts`, e pelo mesmo motivo: uma pergunta que a shell faz ao
//! mundo sem nunca escrever nele. Nasceu quando a linha d'água levou `world.rs` a 703 dos
//! seus 700 — e o corte é o que o arquivo já vinha fazendo, porque `spawn`/`step`/`rewind`
//! (o que MOVE) e "onde está a superfície desta poça?" (o que se OLHA) não são a mesma
//! responsabilidade.

use rapier2d::dynamics::RigidBodyHandle;

use super::PhysicsWorld;
use super::buoyancy;

impl PhysicsWorld {
    /// **A linha d'água de cada zona com empuxo** — o segmento onde a superfície corta
    /// o collider dela, em unidades de mundo.
    ///
    /// A metade VISÍVEL do empuxo. Uma zona de força ganha uma seta porque *para que
    /// lado sopra* não é inferível; um arrasto não tem direção para desenhar; mas uma
    /// poça **tem um lugar**, e até aqui o artista posicionava o tronco no olho.
    ///
    /// Sai da MESMA função que o empuxo usa (`buoyancy::waterline` → `surface_level`),
    /// nunca de uma re-derivação: duas respostas para *"onde está a água?"* divergiriam
    /// numa poça rotacionada ou sob gravidade lateral, que é precisamente onde ninguém
    /// confere. Vazio numa cena sem empuxo, e vazio sem gravidade.
    #[must_use]
    pub fn waterlines(&self) -> Vec<([f32; 2], [f32; 2])> {
        self.effectors
            .iter()
            .filter(|(_, e, _)| e.density > 0.0)
            .filter_map(|(handle, _, _)| {
                let collider = self
                    .bodies
                    .get(*handle)
                    .and_then(|b| b.colliders().first().copied())
                    .and_then(|h| self.colliders.get(h))?;
                buoyancy::waterline(collider.shape(), collider.position(), self.gravity)
            })
            .collect()
    }

    /// **A caixa que envolve TODAS as formas de um corpo**, em mundo —
    /// `(mínimos, máximos)`. `None` se o handle morreu ou se o corpo não tem
    /// forma nenhuma.
    ///
    /// ⚠️ **Todas as formas, e é a lição de 02/08 escrita como código:** a
    /// W-Compound deu a um corpo várias, e a frase *"um corpo tem exatamente um
    /// collider"* — que estava escrita em quatro lugares — virou quatro defeitos
    /// de classes diferentes. Uma caixa é sempre UMA, seja o corpo simples ou
    /// composto, e por isso é a forma certa de perguntar *"qual é a largura da
    /// cabeça dele?"* (o sensor de quina do player, W10).
    ///
    /// ⚠️ E ela é **conservadora**: a caixa de uma cápsula é mais larga que os
    /// ombros dela perto do topo, então uma assistência que a use dispara um
    /// pouco antes do estritamente necessário — o lado seguro de errar, e o
    /// preço de não precisar de um shapecast por forma.
    #[must_use]
    pub fn body_aabb(&self, handle: RigidBodyHandle) -> Option<([f32; 2], [f32; 2])> {
        use rapier2d::parry::bounding_volume::{Aabb, BoundingVolume};
        let rb = self.bodies.get(handle)?;
        let mut merged: Option<Aabb> = None;
        for &ch in rb.colliders() {
            let Some(c) = self.colliders.get(ch) else {
                continue;
            };
            let box_ = c.shape().compute_aabb(c.position());
            merged = Some(merged.map_or(box_, |a| a.merged(&box_)));
        }
        let a = merged?;
        Some(([a.mins.x, a.mins.y], [a.maxs.x, a.maxs.y]))
    }

    /// **QUANTO DO PESO deste corpo o fluido está a carregar**, em `[0, 1]` — a
    /// razão entre o empuxo que ele recebe agora e o peso dele.
    ///
    /// `0` no ar seco. **`1` exactamente à tona**, porque *boiar em repouso* É a
    /// definição de o empuxo igualar o peso. Entre os dois, a faixa de poucos
    /// centímetros em que o corpo está a entrar na água.
    ///
    /// # ⚠️ Por que uma LEI de personagem precisa disto
    ///
    /// A modelagem do arco de um pulo (leve no ápice, pesada na queda) descreve um
    /// corpo em **voo balístico**, onde a gravidade é a única força e o arco é o
    /// produto dela. Quando é o EMPUXO quem o segura, os mesmos multiplicadores
    /// viram **amplificação paramétrica**: pesado ao descer injeta mais energia do
    /// que leve ao subir devolve, ciclo após ciclo, e o personagem largado numa
    /// poça sai da cena (medido: `−1,05 / +4,71 / +12,08 / −20,31`).
    ///
    /// # ⚠️ É o PESO, não a submersão — e a diferença foi MEDIDA
    ///
    /// A primeira versão desta consulta devolvia a fração da ÁREA submersa, que é
    /// a resposta intuitiva e é a errada: à tona, a cápsula de controle desta
    /// linha submerge **`0,26`**, então uma lei que desvanecesse por `1 − área`
    /// deixaria **74%** da bomba ligada exactamente onde o personagem passa a vida.
    /// A razão empuxo÷peso vale `1` ali por construção — e ela também acerta os
    /// casos que a área erra: um corpo de densidade neutra, todo submerso, está em
    /// queda-zero e lê `1`; uma pedra afundando lê `ρ_fluido/ρ_pedra < 1`, que é
    /// quanto da gravidade dela o fluido de facto compensa.
    ///
    /// # ⚠️ A sobreposição é PERGUNTADA, não inferida da altura
    ///
    /// [`buoyancy::buoyant_force`] recorta contra o **nível** da superfície e mais
    /// nada — no solver, quem restringe *a que zona isto se aplica* é o passeio do
    /// grafo de interseção. Sem a mesma pergunta aqui, um personagem numa vala ao
    /// LADO da poça leria `1`, porque ele está de facto abaixo do nível dela. A
    /// pergunta parte das formas DELE, então o custo é o número de sobreposições
    /// do corpo, não o de zonas do mundo.
    ///
    /// ⚠️ **Só matéria conta** (`shapes::displaces`, a mesma porta do empuxo): um
    /// pé-sensor não desloca fluido. E o resultado é **somado sobre zonas** — duas
    /// poças sobrepostas somam força, exactamente como o solver as soma.
    ///
    /// ⚠️ **NÃO é mais capado em `1`, e a mudança é de 2026-08-09.** O teto existia
    /// enquanto o único consumidor perguntava `> 0` (a trava do fluido do
    /// W-Submerged), e para ele *"o fluido carrega-te inteiro"* era de facto o fim
    /// da escala. A lei cinemática (W-KinFluid) lê a MAGNITUDE, e ali o teto era
    /// fatal: uma cápsula 4× menos densa que a água satura em `1` já a **`y = 0,2`**
    /// (medido), com mais de metade do corpo ainda fora — e uma gravidade efetiva
    /// `g·(1 − 1)` é **zero**, então o personagem pararia no meio da poça e nunca
    /// subiria. O número honesto é a razão, e ela passa de `1` para tudo que boia.
    #[must_use]
    pub fn buoyed(&self, handle: RigidBodyHandle) -> f32 {
        self.fluid_at(handle).buoyed
    }

    /// **O que o MEIO faz a este corpo** — as duas grandezas do fluido, respondidas
    /// numa varredura só.
    ///
    /// # ⚠️ Por que exactamente estas duas, e não a força da zona
    ///
    /// A fronteira não foi escolhida aqui: o **W-AreaFalloff** já a desenhou, e o
    /// `effector::apply` a carrega num gate. O falloff pesa os dois **EMPURRÕES**
    /// (força e torque) e **deixa o MEIO em paz** — `drag`, `density` e `form_drag`
    /// descrevem uma substância, e uma substância não fica mais rala perto da
    /// própria margem. É precisamente isso que torna o meio respondível por uma
    /// consulta: ele não depende do frame da zona, nem do espelho dela, nem da
    /// posição do corpo dentro dela.
    ///
    /// ⛔ **A força FICA de fora, e não por escopo:** ela precisa do frame da zona
    /// (`zone_force_world`), do espelho (`zone_spin_sign`) e do falloff
    /// (`zone_falloff_scale`) — re-derivá-los aqui seria uma **segunda resposta**
    /// para *"que empurrão esta zona dá neste ponto?"*, que é o defeito recorrente
    /// desta linha. Uma corrente não leva um personagem cinemático, e isso está
    /// medido e nomeado, não esquecido.
    ///
    /// ⚠️ **O arrasto é o MÁXIMO sobre as zonas, e a escolha apaga uma dedup:** um
    /// corpo COMPOSTO sobrepõe a mesma poça com cada uma das formas dele (a lição
    /// da W-CompoundZone), então uma soma sobre PARES faria a água resistir `N`
    /// vezes mais a uma jangada de `N` peças. O máximo é idempotente sob pares
    /// repetidos ⇒ não há conjunto de zonas-já-vistas a manter, nem alocação, nem
    /// teto. Duas poças sobrepostas dão a mais viscosa das duas, que é a leitura
    /// honesta de *"em que substância estou?"* — o empuxo soma porque forças somam;
    /// um coeficiente não é uma força.
    ///
    /// ⚠️ **`form_drag` fica de fora**: ele é um kernel por-aresta sobre o polígono
    /// do collider, não um escalar do meio, e um personagem cinemático não tem
    /// velocidade que o solver possa integrar contra cada normal.
    #[must_use]
    pub fn fluid_at(&self, handle: RigidBodyHandle) -> FluidAt {
        let g = self.gravity.norm();
        if g <= 0.0 || self.effectors.is_empty() {
            return FluidAt::DRY;
        }
        let Some(rb) = self.bodies.get(handle) else {
            return FluidAt::DRY;
        };
        // O peso REAL que o solver tem — inclusive o `MassOverride` do W-Mass, que
        // é precisamente o caso em que uma massa re-derivada da densidade mentiria.
        //
        // ⚠️ **E ele responde para TODA espécie de corpo, o que não era óbvio:** um
        // corpo cinemático tem massa INFINITA para o solver, mas `rb.mass()` devolve
        // a massa dos colliders na mesma (medido: `1,0000` em Dynamic, Kinematic e
        // Fixed) — o rapier zera a inversa-massa EFETIVA, não esta. Uma versão desta
        // wave carregava um `authored_weight` que somava as massas dos colliders
        // quando este número era zero, com um doc a dizer que era ele que fazia a
        // água existir no modo cinemático. **Era falso e a mutação o provou:**
        // removê-lo deixou tudo verde, porque o zero que eu tinha medido vinha
        // inteiramente do par que não existia no grafo (ver `collider_build`).
        let weight = rb.mass() * g;
        if weight <= 0.0 {
            return FluidAt::DRY;
        }
        let mut lift = 0.0f32;
        let mut drag = 0.0f32;
        for &ch in rb.colliders() {
            let Some(mine) = self
                .colliders
                .get(ch)
                .filter(|c| super::shapes::displaces(c))
            else {
                continue;
            };
            for (c1, c2, intersecting) in self.narrow_phase.intersection_pairs_with(ch) {
                // O par existe assim que as CAIXAS se tocam, que é uma região maior
                // que a forma — a mesma distinção que o `effector` faz.
                if !intersecting {
                    continue;
                }
                let other = if c1 == ch { c2 } else { c1 };
                let Some(zone) = self.colliders.get(other) else {
                    continue;
                };
                let Some(zone_body) = zone.parent() else {
                    continue;
                };
                let Some(effect) = self
                    .effectors
                    .binary_search_by_key(&zone_body.into_raw_parts(), |(h, _, _)| {
                        h.into_raw_parts()
                    })
                    .ok()
                    .map(|i| &self.effectors[i].1)
                else {
                    continue;
                };
                // O arrasto é do MEIO: ele conta por ESTAR dentro, mesmo numa zona
                // que não tem empuxo nenhum (uma corrente de ar viscosa).
                drag = drag.max(effect.drag);
                if effect.density <= 0.0 {
                    continue;
                }
                if let Some((force, _)) = buoyancy::buoyant_force(
                    mine.shape(),
                    mine.position(),
                    zone.shape(),
                    zone.position(),
                    self.gravity,
                    effect.density,
                ) {
                    lift += force.norm();
                }
            }
        }
        FluidAt {
            // `max`, não `clamp`: com `NaN` o `clamp` PROPAGA e o `max` devolve o
            // outro operando — um piso que também sanitiza, que é estritamente
            // mais forte que o teto que saiu.
            buoyed: (lift / weight).max(0.0),
            drag,
        }
    }
}

/// **O que o fluido faz a um corpo** — a resposta de [`PhysicsWorld::fluid_at`].
///
/// Duas grandezas e não uma porque são feitas na MESMA varredura do grafo de
/// interseção: perguntá-las em duas consultas pagaria o passeio duas vezes por
/// tique de player, e as duas descrevem o mesmo fato (*em que meio estou?*).
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct FluidAt {
    /// **Quantos pesos deste corpo o fluido carrega.** `0` no ar seco, `1`
    /// exactamente à tona (boiar em repouso É o empuxo igualar o peso), e **maior
    /// que 1** para o que sobe — uma rolha submersa lê a razão das densidades.
    pub buoyed: f32,
    /// **O coeficiente de resistência do meio**, por segundo — o `AreaDrag` da zona
    /// mais viscosa que contém este corpo. `0` no ar seco.
    pub drag: f32,
}

impl FluidAt {
    /// Ar seco — o neutro, e o que uma cena sem zona nenhuma produz.
    pub const DRY: Self = Self {
        buoyed: 0.0,
        drag: 0.0,
    };
}

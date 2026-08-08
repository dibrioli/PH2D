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
    /// pé-sensor não desloca fluido. E o resultado é **somado sobre zonas e
    /// CLAMPADO** — duas poças sobrepostas somam força, e passar de `1` não
    /// significa nada para quem lê isto: *"o fluido carrega-te inteiro"* é o fim
    /// da escala.
    #[must_use]
    pub fn buoyed(&self, handle: RigidBodyHandle) -> f32 {
        let g = self.gravity.norm();
        if g <= 0.0 || self.effectors.is_empty() {
            return 0.0;
        }
        let Some(rb) = self.bodies.get(handle) else {
            return 0.0;
        };
        // O peso REAL que o solver tem — inclusive o `MassOverride` do W-Mass, que
        // é precisamente o caso em que uma massa re-derivada da densidade mentiria.
        let weight = rb.mass() * g;
        if weight <= 0.0 {
            return 0.0;
        }
        let mut lift = 0.0f32;
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
                let fluid = self
                    .effectors
                    .binary_search_by_key(&zone_body.into_raw_parts(), |(h, _, _)| {
                        h.into_raw_parts()
                    })
                    .ok()
                    .map_or(0.0, |i| self.effectors[i].1.density);
                if fluid <= 0.0 {
                    continue;
                }
                if let Some((force, _)) = buoyancy::buoyant_force(
                    mine.shape(),
                    mine.position(),
                    zone.shape(),
                    zone.position(),
                    self.gravity,
                    fluid,
                ) {
                    lift += force.norm();
                }
            }
        }
        (lift / weight).clamp(0.0, 1.0)
    }
}

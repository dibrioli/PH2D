//! ⭐⭐⭐ **O TRAÇO DO PINCEL DE PESO** — a pincelada e a voz da recusa.
//!
//! # Porque ele é um ficheiro e não mais um ramo do press
//!
//! O irmão [`super::despacho_clique_vetor_premido`] responde *«qual dos modos do vector reclama
//! este clique?»* — uma pergunta sobre ROTEAMENTO. Isto responde *«o que uma pincelada de peso
//! faz»*, que é uma pergunta sobre a FERRAMENTA, e tem duas metades que o roteamento não tem: a
//! conversão de unidades (o painel fala ECRÃ, a lei fala MUNDO) e a tradução do resultado da lei
//! numa recusa com voz.
//!
//! ⚠️ **O nome começa por `despacho_` de propósito:** o oráculo textual das leis de ordem desta
//! shell (`input_text::dispatch`) colhe **só** os ficheiros com esse prefixo, e um nome fora dele
//! tiraria estas linhas de toda régua de despacho — **em silêncio**, que é a armadilha §2.7 do
//! HOWTO de partir uma família.

impl crate::App {
    /// ⭐⭐⭐ **UMA PINCELADA DE PESO** — a porta que o press e o movimento partilham.
    ///
    /// ⚠️ **Ela é UMA e não duas** porque a lei do pen-down e a do arrasto são a mesma: *duas
    /// cópias divergiriam no primeiro ajuste, e o sintoma seria o clique a pintar diferente do
    /// arrasto*. O que difere entre os dois é **quem escolhe a arte** — o press, e só ele.
    ///
    /// ⚠️ **A recusa SOBE À TELA com a entrada que falta** (a família que o `CLAUDE.md` §5.0 nomeia:
    /// *um pincel que não faz nada e não diz porquê é indistinguível de um pincel partido*), e ⛔
    /// **só no press** — uma queixa por evento de ponteiro é ruído que o artista aprende a ignorar.
    ///
    /// ⛔⛔ **Elas eram `eprintln!` e isso era um botão mudo** (report do dono, 2026-09-19): a
    /// superfície já existia e é a que os OUTROS verbos deste mesmo painel usam desde 18/09
    /// ([`ph2d_skeleton_live::recusa_do_osso`] + a [`ph2d_editor_core::ToastQueue`]) — *uma recusa
    /// que só o terminal vê é um botão mudo*, e o pincel nasceu a violar a lei que a família ao
    /// lado dele tinha acabado de pagar. ⚠️ O terminal FICA ao lado do aviso: um smoke headless não
    /// tem tela, e é ali que a sonda lê.
    pub(super) fn vec_pinta_peso(&mut self, world: [f64; 2]) {
        let Some(alvo) = self.skeleton.weight_drag else {
            // ⛔ Sem arte sob o dedo o traço nem começou — a queixa é do pen-down, e sai UMA vez.
            self.recusa_do_pincel_de_peso(
                ph2d_skeleton_live::recusa_do_osso::RecusaDoOsso::PincelForaDaArte,
            );
            return;
        };
        let Some(osso) = self.selected_bone_bits() else {
            self.recusa_do_pincel_de_peso(
                ph2d_skeleton_live::recusa_do_osso::RecusaDoOsso::PincelOssoDeFora,
            );
            self.skeleton.weight_drag = None;
            return;
        };
        // ⚠️ Ver o pen-down: o raio do painel e' de ECRA, a lei fala MUNDO.
        let raio = self.vec.draw_config.weight_radius * self.vec_px_to_world();
        // ⭐⭐⭐ **A MAGNITUDE compoe-se com a DIRECCAO numa PORTA, nunca num `if` aqui**
        // ([`ph2d_tool_vector::WeightDirection::delta`], ordem do dono de 2026-09-19).
        //
        // ⛔⛔ Ate' esse dia o sinal vivia dentro do numero e esta linha lia-o cru. Escrever a
        // composicao aqui — `if soma { q } else { -q }` — poria a lei num laco de input, onde
        // teste nenhum lhe chega: *e' exactamente assim que a escolha do alvo do pincel viveu ate'
        // 19/09*, e foi preciso um report do dono para a descobrir.
        let quanto = self
            .vec
            .draw_config
            .weight_direction
            .delta(self.vec.draw_config.weight_amount);
        let ppm = self
            .gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .map_or(crate::EPS_PIXELS_PER_METER, |h| h.project.pixels_per_meter);
        let Some(gfx) = self.gfx.as_mut() else { return };
        let r = ph2d_skeleton_live::peso_a_mao::pinta(
            &mut gfx.sim,
            ph2d_ecs::Entity::from_bits(alvo),
            ph2d_ecs::Entity::from_bits(osso),
            ppm,
            world,
            raio,
            quanto,
        );
        // ⭐ A LEI devolve o facto; a tradução para uma recusa com voz é da shell — é ela que tem
        // a fila de avisos. ⚠️ As três que falam são as que têm CURA pela mão do artista.
        let recusa = match r {
            ph2d_skeleton_live::peso_a_mao::Pincelada::OssoDeFora => {
                Some(ph2d_skeleton_live::recusa_do_osso::RecusaDoOsso::PincelOssoDeFora)
            }
            ph2d_skeleton_live::peso_a_mao::Pincelada::SemPele => {
                Some(ph2d_skeleton_live::recusa_do_osso::RecusaDoOsso::PincelSemPele)
            }
            ph2d_skeleton_live::peso_a_mao::Pincelada::ForaDaArte => {
                Some(ph2d_skeleton_live::recusa_do_osso::RecusaDoOsso::PincelForaDaArte)
            }
            ph2d_skeleton_live::peso_a_mao::Pincelada::Pintada { .. } => None,
        };
        if let Some(recusa) = recusa {
            self.recusa_do_pincel_de_peso(recusa);
            self.skeleton.weight_drag = None;
        }
    }

    /// **A recusa do pincel de peso sobe à tela** — a mesma porta dos outros verbos do osso.
    fn recusa_do_pincel_de_peso(&mut self, r: ph2d_skeleton_live::recusa_do_osso::RecusaDoOsso) {
        let texto = ph2d_i18n::tr(r.chave());
        eprintln!("[peso] {texto}");
        if let Some(gfx) = self.gfx.as_mut() {
            gfx.toasts.push(ph2d_editor_core::Toast::warning(texto));
        }
    }
}

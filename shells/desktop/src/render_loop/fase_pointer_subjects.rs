//! **Fase do quadro: O QUE ESTÁ SOB O CURSOR** — a 1.ª do `run_render_frame` (OBRA 2 da
//! `line/render-loop`, 2026-09-12).
//!
//! Os sujeitos que o ponteiro aponta, resolvidos UMA vez por quadro e antes de qualquer consumidor:
//! o som de UI pendente, a escuta do comando do mapa de entrada, o dedo do jogador, o objecto sob o
//! cursor e o contorno dele, o pedaço do Trim, a face do Balde e a metade de osso. As razões de cada
//! um estar AQUI em cima viajaram com o código, verbatim.
//!
//! ⚠️ **A ordem é o contrato.** Esta fase corre antes do temporizador do quadro e de tudo o resto,
//! e devolve o `PlayerInput` porque o quadro o entrega mais abaixo à câmera de smoke e à física.

impl crate::App {
    /// Ver o cabeçalho do módulo. Devolve o dedo do jogador, resolvido do mapa do projecto.
    pub(super) fn fase_pointer_subjects(&mut self) -> ph2d_physics_ecs::PlayerInput {
        // ⭐ **O REALCE DE PROVENIÊNCIA, resolvido UMA vez por quadro** (estudo de UI viva, C2).
        //
        // ⚠️ **Aqui em cima, e não em cada consumidor.** A Hierarquia publica cedo no quadro e o
        // contorno do canvas desenha tarde: dois picks separados dariam duas respostas assim que o
        // mapa vivo mudasse entre eles, e a linha acesa deixaria de ser a forma contornada.
        // ⭐ **O SOM DE UI** (estudo de UI viva, D1) — o canal diferido que os gestos enchem,
        // drenado UMA vez por quadro. ⚠️ O `take` é o que garante *um som por gesto*: sem ele, um
        // gesto que não limpasse o canal tocaria em todos os quadros seguintes.
        if let Some(what) = self.pending_ui_sound.take() {
            self.ui_sound(what);
        }
        // ⭐ **O dedo do jogador, resolvido do mapa** (plano 30 W5) — no TOPO do quadro, e num
        // local, porque ele lê o `HeroScreen` (onde o mapa mora) e escreve no estado resolvido da
        // `App`: fazê-lo no sítio do consumo emprestaria `self` duas vezes.
        // ⚠️ **A escuta do comando ANTES da resolução**: se o botão que o artista acabou de ligar
        // fosse resolvido no mesmo quadro, ele dispararia a acção que acabou de nascer — o gesto de
        // LIGAR viraria também o gesto de ACCIONAR.
        self.poll_input_map_pad_binding();
        // ⭐ A porta é da CRATE desde a Fase C, e leva os três campos em vez de `self`: o
        // empréstimo disjunto é feito AQUI, que é onde os campos vivem.
        let player_input = ph2d_app_physics::player_input::resolve_player_input(
            &mut self.input_actions,
            self.gfx.as_ref().and_then(|g| g.hero_screen.as_ref()),
            &self.input,
        );
        let pointer = self.last_pointer;
        self.hovered_object = self.pick_hovered_object(pointer);
        // ⭐ **A geometria do contorno é resolvida AQUI, com o objecto.** Ela precisa da
        // `PresentWorld` mutável (a caixa de uma sprite sai de uma query), que o sítio de desenho
        // já não tem — e resolvê-la aqui garante que o que se desenha é o objecto que se escolheu.
        self.hover_outline = match self.hovered_object {
            Some(bits) => self.resolve_hover_outline(bits),
            None => Vec::new(),
        };
        // ⭐⭐⭐ **O pedaço que o Trim aponta** (plano 38) — ao lado do realce de proveniência, que
        // responde à mesma pergunta noutra tinta. Fora do modo Trim ele LIMPA-SE.
        self.refresh_trim_hover(pointer);
        // ⭐⭐⭐ **A face que o Balde vai preencher** (plano 40) — ao lado do realce do Trim porque
        // os dois respondem à mesma pergunta (*"o que está sob o cursor?"*) para ferramentas que
        // apontam em vez de autorar. ⚠️ A rede é GUARDADA lá dentro: montá-la custa `3,8 ms` a 20
        // traços, e só se refaz quando a geometria muda.
        // ⭐⭐⭐ **O upkeep corre ANTES do realce e em QUALQUER ferramenta** (plano 40): ele refaz a
        // rede quando a geometria muda e RE-COZE os preenchimentos vivos — o artista arrasta um nó
        // com a seta branca, e a área tem de acompanhar. O realce só lê a rede já guardada.
        self.bucket_upkeep();
        self.refresh_bucket_hover(pointer);
        // ⭐⭐ **A metade de OSSO sob o ponteiro** (Enio, 2026-09-06) — ao lado dos dois de cima, e
        // pela mesma razão: é a pergunta *"o que está sob o cursor?"*, resolvida UMA vez por quadro
        // e num sítio só. ⚠️ Ela NÃO se gateia pelo modo, ao contrário do Trim e do Balde: os ossos
        // desenham-se em TODO modo da ferramenta de vetor (`vec_overlay::bones`), então o realce
        // deles tem de existir onde eles existem.
        self.refresh_bone_hover(pointer);
        player_input
    }
}

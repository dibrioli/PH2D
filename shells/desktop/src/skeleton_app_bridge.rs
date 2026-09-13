//! ⭐ **As PONTES da família `skeleton` que ainda tocam a `App`** — e só elas.
//!
//! O corpo dos módulos [`ph2d_app_skeleton::bone_gesture`] e [`ph2d_app_skeleton::bone_pick`] é lei
//! pura sobre o `SimWorld` e mudou-se para a crate na Fase C da W2. O que sobrou são **três
//! métodos de `App`** que o `input_dispatch` e o laço do quadro chamam, e que perguntam a campos
//! que as **cinco portas** do [`ph2d_app_host::AppHost`] não cobrem:
//!
//! | método | o que ele precisa da `App` |
//! |---|---|
//! | `selected_bone_bits` | `gfx.hero_screen.gizmo` — a selecção do gizmo deste quadro |
//! | `bone_handle_at` | `vec_world_at` / `vec_px_to_world` — a câmara, que é da shell |
//! | `refresh_bone_hover` | mais `vec_draw_config.bone_action`, `bone_hover`, `bone_preview`, `vec_bone_drag` |
//!
//! ⛔⛔ **Isto NÃO é um pedido de sexto método de host.** O HOWTO §1.5 é explícito: *«se a sua
//! família precisa de um método por campo da `App` que hoje toca, ela não precisa de um trait
//! maior: precisa de tirar o campo da `App`»* — e ⛔ **nenhum método do trait devolve um handle**
//! (`&AppGfx`, `&HeroScreen`), o que é precisamente o que os três precisariam. ⇒ eles ficam aqui,
//! **juntos e nomeados**, em vez de manterem dois ficheiros inteiros na shell.
//!
//! ⚠️ **Os sítios de chamada ficam byte a byte iguais** — é um `impl crate::App`, logo
//! `self.refresh_bone_hover(p)` continua a escrever-se assim (HOWTO §1.5).
//!
//! ⚠️ E as leis que estes três chamam são as da crate: `selected_bone`, `hover`,
//! `grabbable_outside_bone_mode`, `drag_now`. Elas atravessam por NOME, nunca por handle.

use ph2d_app_skeleton::bone_gesture::selected_bone;
use ph2d_app_skeleton::bone_pick::{grabbable_outside_bone_mode, hover};

impl crate::App {
    /// [`ph2d_app_skeleton::bone_pick::selected_bone`] pela selecção do gizmo deste quadro.
    ///
    /// ⚠️ **Ela existe SÓ para o caminho do gesto**, onde `self` está inteiro na mão. No laço de
    /// desenho o `gfx` já está emprestado mutável de ponta a ponta, e ali chama-se a função livre
    /// acima — a lei é a mesma, e é por isso que ela vive numa função só.
    pub fn selected_bone_bits(&self) -> Option<u64> {
        let gfx = self.gfx.as_ref()?;
        selected_bone(&gfx.sim, gfx.hero_screen.as_ref()?.gizmo.iter_selected())
    }
}

impl crate::App {
    /// **Resolve a metade de osso sob o ponteiro**, uma vez por quadro
    /// ([`crate::App::bone_hover`]).
    ///
    /// ⚠️ **Sem ponteiro no canvas ⇒ LIMPA**, como o realce do Trim e o do Balde: um realce que
    /// sobrevive ao cursor sair da tela é uma alça que finge estar apontada.
    /// ⭐⭐⭐ **A alça de osso sob este ponto, se o VERBO dela não existir noutra ferramenta.**
    ///
    /// ⚠️ **A MESMA porta do realce** ([`ph2d_app_skeleton::bone_pick::hover`]) — o que o artista vê aceso é, por construção, o
    /// que ele vai pegar. Uma segunda varredura seria a segunda resposta à mesma pergunta.
    pub fn bone_handle_at(&self, pointer: (f32, f32)) -> Option<ph2d_skeleton_render::BoneHover> {
        let world = self.vec_world_at(pointer)?;
        let px = self.vec_px_to_world();
        let foco = self.selected_bone_bits();
        // ⚠️ **Sempre *Transformar*, e não o verbo armado**: esta porta é a de FORA do modo Osso, e
        // ali o que existe são os verbos que nenhuma outra ferramenta sabe exprimir
        // ([`ph2d_app_skeleton::bone_pick::grabbable_outside_bone_mode`]). O verbo *Criar* não é um deles.
        let h = hover(
            &self.gfx.as_ref()?.sim,
            world,
            px,
            foco,
            ph2d_tool_vector::BoneAction::Transform,
        )?;
        grabbable_outside_bone_mode(h.part).then_some(h)
    }

    pub fn refresh_bone_hover(&mut self, pointer: (f32, f32)) {
        // ⚠️⚠️ **OS DOIS SLOTS SAEM DA MESMA LEITURA, e a saída antecipada tem de limpar OS DOIS.**
        // A 1.ª redacção desta função limpava só o realce e deixava a pré-visualização congelada:
        // o cursor sai do canvas e um osso fantasma fica desenhado na tela até ao gesto seguinte.
        // *Um par de slots resolvido no mesmo sítio esquece-se meio a meio.*
        let Some(world) = self.vec_world_at(pointer) else {
            self.skeleton.bone_hover = None;
            self.skeleton.bone_preview = None;
            return;
        };
        let px_to_world = self.vec_px_to_world();
        // ⚠️ O osso em FOCO entra: a alça da força só existe onde ela se desenha, e o que a desenha
        // é a selecção. Sem ele o dedo procuraria uma alça que não está na tela.
        let foco = self.selected_bone_bits();
        // ⚠️ **O verbo só manda DENTRO do modo Osso.** Fora dele o `bone_action` continua guardado
        // (ele é estado da ferramenta, não do quadro), e lê-lo aqui deixaria o realce das outras 13
        // ferramentas preso no que o artista armou da última vez que passou pelo esqueleto.
        let acao = if self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::Bone {
            self.vec.draw_config.bone_action
        } else {
            ph2d_tool_vector::BoneAction::Transform
        };
        // ⭐⭐⭐ **O QUE O ARRASTO SIGNIFICA AGORA** — a porta ÚNICA que o release também lê
        // ([`ph2d_app_skeleton::bone_gesture::drag_now`]). ⛔ É ela que faz o que o artista VÊ ser o que ele
        // RECEBE: a emenda, a ponta encaixada e o limiar saem todos da mesma leitura.
        let agora = self.skeleton.bone_drag.and_then(|n| {
            let gfx = self.gfx.as_ref()?;
            Some(crate::bone_gesture::drag_now(
                &gfx.sim,
                n,
                world,
                px_to_world,
            ))
        });
        // ⭐⭐⭐ **COM UM ARRASTO VIVO, o realce responde ao RELEASE e não ao press.**
        //
        // ⚠️ São perguntas diferentes — *o que um press aqui faria?* contra *o que este soltar vai
        // fazer?* — e durante um arrasto só a segunda tem sentido: o press já aconteceu. A bolinha
        // que acende é a que o osso novo vai agarrar.
        self.skeleton.bone_hover = match agora.and_then(|a| a.splice) {
            Some((alvo, _)) => Some(ph2d_skeleton_render::BoneHover {
                bone: alvo,
                part: ph2d_skeleton_render::BonePart::Joint,
            }),
            None => self
                .gfx
                .as_ref()
                .and_then(|gfx| hover(&gfx.sim, world, px_to_world, foco, acao)),
        };
        // ⭐ E o osso que está a NASCER, da MESMA leitura.
        self.skeleton.bone_preview = self
            .skeleton
            .bone_drag
            .zip(agora)
            .map(|(n, a)| (n.origin, a.tip, a.armed));
    }
}

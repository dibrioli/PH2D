//! ⭐⭐⭐ **OS MOTORES DE POSE DO ESQUELETO, na metade da SIMULAÇÃO** — o osso inteligente e a
//! cinemática inversa que persiste.
//!
//! # ⛔⛔ O defeito que trouxe estes dois para aqui (report do dono, 2026-09-14, com duas fotos)
//!
//! *«Ao acrescentar o IK, de algum modo o osso perde influência sobre a ponta da malha»* — nas
//! fotos o **gizmo** do osso está na pose resolvida e a **arte** não o acompanha.
//!
//! Eles viviam na `fase_vector_view_and_drives`, imediatamente antes do `skeleton_live::recook` (a
//! pele **VECTORIAL**), e o comentário que os punha ali escreve a lei à letra: *«ela escreve a pose
//! dos ossos, e o recook é quem transforma a pose em geometria. Ao contrário, a pele mostraria a
//! pose do quadro anterior.»*
//!
//! ⛔⛔ **A SEGUNDA MÍDIA chegou depois e não herdou a arrumação.** A pele de **IMAGEM**
//! (`attach_skin_meshes`) é posta na [`super::fase_sim_extract`], que corre na metade da
//! **simulação** — antes de toda esta fase. ⇒ a malha era construída da pose de ANTES do solver.
//!
//! ⚠️ **E não é um atraso de um quadro, é permanente:** o apply da timeline
//! ([`super::fase_timeline_drain`]) corre na mesma metade e reescreve todo osso keyado pela curva,
//! logo cada quadro repunha a pose autorada mesmo a tempo de a malha a ler. O gizmo mostrava a pose
//! da IK; a arte mostrava a curva. *Duas leituras da mesma pose, e a que o artista vê era a errada.*
//!
//! # A lei, agora numa ordem só
//!
//! *Todo motor que ESCREVE pose corre antes de qualquer consumidor que a LEIA* — e nesta metade os
//! consumidores são dois (a malha da imagem, aqui; a pele vectorial, na fase de canvas, que
//! continua a correr depois). A ordem interna entre os dois motores é a de sempre e continua
//! load-bearing: **o osso inteligente primeiro** (ele é a correcção autorada, a pose de BASE), a
//! **âncora depois** (a restrição persegue um alvo e tem de ver a pose já corrigida).

impl crate::App {
    /// Ver o cabeçalho deste ficheiro. Corre entre o apply da timeline e o extract das sprites.
    pub(super) fn fase_skeleton_drives(&mut self) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let sim = &mut gfx.sim;
        crate::skeleton_smart::drive(sim, &self.timeline.doc, &mut self.preview_drive);
        crate::skeleton_goal::solve(sim, &mut self.preview_drive);
    }
}

//! **A cena 96 — A CORRIDA SOBREVIVE AO ARQUIVO** (W17), irmã de
//! `physics_smoke_player_bake.rs`.
//!
//! O mesmo percurso curto da cena 95, e de propósito: a W16 provou que o bake
//! replaya a corrida GRAVADA, e esta prova que a gravação continua lá **depois de
//! fechar o app**. As duas juntas são o par que o §4 do plano 06 pedia — *o bake
//! é o caminho "torne durável", a fita persistida é o caminho "mantenha
//! editável"*.
//!
//! # ⚠️ O que só se julga fechando o app
//!
//! Um Ctrl+S seguido de Ctrl+O na MESMA sessão não prova nada: a fita que o load
//! instala é a que a sessão já tinha. O roteiro manda **fechar e reabrir** —
//! é a única forma de a promessa (*sobrevive a fechar o app*) ser observada.
//!
//! # ⚠️ E a metade que o smoke tem de encontrar de propósito
//!
//! Com a corrida carregada, o botão **Clear Recorded Run (N.N s)** aparece na
//! §14 com os segundos no rótulo. **A ausência dele é o outro readout:** num
//! documento onde ninguém correu ele não existe, porque não há o que descartar.
//! O roteiro pede as duas leituras.

use ph2d_core::Vec2;

use crate::App;
use crate::physics_smoke_player::{slab, spawn_player};

impl App {
    /// **A corrida sobrevive ao arquivo** — jogar, salvar, fechar, reabrir, assar.
    pub(crate) fn physics_smoke_recorded_run(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let world = gfx.sim.world_mut();

        slab(
            world,
            "Ground",
            Vec2::new(4.0, -0.5),
            [10.0, 0.5],
            0.0,
            [0.35, 0.35, 0.4, 1.0],
        );
        slab(
            world,
            "Step",
            Vec2::new(15.0, 0.0),
            [3.0, 0.5],
            0.0,
            [0.38, 0.36, 0.44, 1.0],
        );
        slab(
            world,
            "Backstop",
            Vec2::new(19.0, 2.0),
            [0.5, 2.0],
            0.0,
            [0.30, 0.34, 0.42, 1.0],
        );

        spawn_player(world, Vec2::new(-4.0, 1.4));
        eprintln!("{RECORDED_RUN_SMOKE_MESSAGE}");
    }
}

/// O roteiro da cena 96 — o gesto é JOGAR, SALVAR, **FECHAR**, e reabrir.
pub(crate) const RECORDED_RUN_SMOKE_MESSAGE: &str = concat!(
    "[physics-smoke 96] A CORRIDA SOBREVIVE AO ARQUIVO (W17). Um chao, um degrau\n",
    "e um encosto -- o percurso curto da cena 95, para a corrida ser reconhecivel.\n",
    "\n",
    "⚠️ Se a linha acima nao aparecer, pare: a cena nao montou.\n",
    "\n",
    "CONTROLE: setas <- / -> (ou A / D) andam. SETA PARA CIMA (ou Z) pula.\n",
    "\n",
    "O QUE JULGAR, nesta ordem:\n",
    " 1. Selecione o personagem na Hierarquia e olhe a secao Platform Player:\n",
    "    ⚠️ NAO deve haver botao 'Clear Recorded Run'. Ninguem correu ainda, e a\n",
    "    AUSENCIA dele e' o readout de que o documento nao tem corrida.\n",
    " 2. Marque Physics no transporte e de Play. JOGUE por uns 3 segundos: ande\n",
    "    para a direita, suba o degrau, pule. Faca uma corrida que se reconheca.\n",
    " 3. Pause. Selecione o personagem: agora o botao 'Clear Recorded Run (N.N s)'\n",
    "    APARECE, e o numero de segundos bate com o que voce jogou.\n",
    " 4. Ctrl+S. O log diz 'salvo: ... (N bytes)'.\n",
    " 5. ⚠️ FECHE O APP e abra de novo com a MESMA cena (env 96). Sem fechar,\n",
    "    este passo nao prova nada: um Ctrl+O na mesma sessao devolveria a fita\n",
    "    que a sessao ja' tinha.\n",
    " 6. Ctrl+O. Selecione o personagem: o botao 'Clear Recorded Run (N.N s)' esta'\n",
    "    la', com os MESMOS segundos. A corrida sobreviveu a fechar o app.\n",
    " 7. Aperte Bake (secao Physics Body). Desmarque Physics, de Play do inicio:\n",
    "    o personagem REFAZ a corrida que voce jogou ONTEM.\n",
    " 8. Ctrl+Z (duas vezes: as curvas e o kind sao duas filas). Depois clique\n",
    "    'Clear Recorded Run': o botao SOME, porque nao ha mais o que descartar.\n",
    "\n",
    "⚠️ E o defeito que a wave removeu, se quiser ve-lo pelo avesso: antes dela a\n",
    "fita gravava TODO tique que o relogio andasse -- ate' sem personagem na cena e\n",
    "com o Physics desmarcado. Como o Physics nasce DESMARCADO, o passo 5 teria\n",
    "apagado a corrida so' por voce assistir a timeline.\n",
);

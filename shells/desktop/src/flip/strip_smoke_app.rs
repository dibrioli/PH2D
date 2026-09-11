//! **A metade que precisa da `App`** do `strip_smoke` (W2/L5, 2026-09-11).
//!
//! O resto do módulo vive em [`ph2d_app_flip::strip_smoke`] — as leis e a cena, que não
//! precisam da shell. Aqui fica só o que toca os agregados DELA: `AppGfx` (o `FlipDoc`
//! vivo, o registo de ferramentas), o `HeroScreen` (o barramento dos painéis) e o
//! relógio. É a lista que o substrato da `ph2d-app-host` tem de cobrir para isto
//! também sair (W2 Fase B).

#[allow(unused_imports)]
use ph2d_app_flip::strip_smoke::*;
use std::sync::atomic::Ordering;

impl crate::App {
    /// Roda no prólogo do frame (ao lado dos outros smokes). No-op sem a env.
    pub(crate) fn flip_strip_smoke(&mut self) {
        if !enabled() || self.gfx.is_none() {
            return;
        }
        if FRAME.fetch_add(1, Ordering::Relaxed) != 3 {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        let tool_ok = gfx.tools.set_active(&ph2d_editor::ToolId::new("flip"));

        let oid = gfx.flip.push_object("Strip Smoke");
        let obj = gfx.flip.object_mut(oid).expect("objeto recém-criado");
        stage(obj);

        self.playhead.seek(0.0);
        self.playhead.pause();

        eprintln!(
            "\n[strip-smoke] cena montada: a bola quicando em 4 chaves (0, 4, 5, 11; \
             exposicoes 4, 1, 6, 2). Ferramenta flip ativa: {}.",
            if tool_ok {
                "sim"
            } else {
                "NAO (PARE: sem ela a faixa Frames nao aparece)"
            }
        );
        eprintln!(
            "\n\
             ============================================================\n\
             ANTES DE TUDO, confira DUAS coisas, nesta ordem:\n\
             ============================================================\n\
             1. Este terminal imprimiu, logo acima, a linha comecando com\n\
                '[strip-smoke] cena montada'. Se NAO imprimiu, PARE: o\n\
                smoke nao rodou (arvore ou variavel de ambiente errada).\n\
             2. Na parte de BAIXO da janela do app existe uma faixa com o\n\
                titulo 'Frames - Bola'. Dentro dela, abaixo da fileira de\n\
                botoes, ha QUATRO CAIXAS lado a lado com os numeros\n\
                4, 1, 6 e 2 dentro. Se essa faixa nao existir, PARE e me\n\
                diga o que voce ve no lugar.\n\
             \n\
             O que esta na tela: UMA bola quicando, desenhada em 4\n\
             momentos -- um desenho por caixa: vermelha no alto a\n\
             esquerda, amarela caindo, ciano ESMAGADA no chao, verde no\n\
             alto a direita. Voce esta no primeiro desenho (a bola\n\
             VERMELHA, na cor real). Perto dela ha um VULTO azul-escuro\n\
             meio transparente: e' o desenho SEGUINTE, mostrado como\n\
             referencia (o 'papel vegetal' do animador). Vulto\n\
             esverdeado = desenho anterior; azulado = seguinte. So o\n\
             desenho em que voce esta tem a cor verdadeira.\n\
             \n\
             ------------------------------------------------------------\n\
             AQUECIMENTO (10 s): clique nas quatro caixas, uma por uma\n\
             ------------------------------------------------------------\n\
             A cada clique a bola PULA para a pose daquele desenho (e\n\
             muda de cor), e os vultos mudam junto. Se isso funciona,\n\
             va aos seis testes.\n\
             \n\
             ------------------------------------------------------------\n\
             TESTE 1 -- ARRASTAR UMA CAIXA muda o desenho de lugar no tempo\n\
             ------------------------------------------------------------\n\
             Segure o MEIO de uma caixa e arraste para o lado.\n\
             Enquanto arrasta, um CONTORNO mostra onde ela vai parar;\n\
             ela so muda de lugar quando voce SOLTA. Ao chegar na\n\
             vizinha, ENCOSTA e para -- nao passa por cima, nao troca de\n\
             lugar, nao some. (So clicar, sem arrastar, continua pulando\n\
             para aquele desenho; tremidinha de mao no clique nao move.)\n\
             \n\
             ------------------------------------------------------------\n\
             TESTE 2 -- ARRASTAR A BEIRADA DIREITA muda quanto tempo dura\n\
             ------------------------------------------------------------\n\
             Va com o mouse na BEIRADA DIREITA da caixa mais larga (a de\n\
             numero 6). Uma barrinha clara aparece ali; arraste-a.\n\
             A caixa estica ou encolhe, o numero acompanha, e as caixas\n\
             SEGUINTES sao empurradas junto.\n\
             Na caixa mais fina (a de numero 1) a barrinha NAO aparece --\n\
             de proposito: estreita demais para os dois gestos, ela\n\
             continua servindo para arrastar; a duracao dela muda pela\n\
             caixa 'Hold' na fileira de botoes.\n\
             \n\
             ------------------------------------------------------------\n\
             TESTE 3 -- o botao 'Pin' deixa um desenho visivel de longe\n\
             ------------------------------------------------------------\n\
             Clique na PRIMEIRA caixa: a bola verde (a ultima) nao\n\
             aparece nem como vulto -- esta longe demais.\n\
             Agora clique na ULTIMA caixa, aperte 'Pin' na fileira de\n\
             botoes, e volte a primeira caixa.\n\
             O vulto da bola verde agora aparece, mesmo de longe -- e o\n\
             vulto do vizinho continua la. A caixa fixada ganha um\n\
             pontinho no canto de baixo. Apertar 'Pin' de novo desfaz.\n\
             \n\
             ------------------------------------------------------------\n\
             TESTE 4 -- VARIAS CAIXAS de uma vez (marcar e arrastar)\n\
             ------------------------------------------------------------\n\
             Segure SHIFT e clique na PRIMEIRA e na ULTIMA caixa: as\n\
             duas ficam marcadas (cor de destaque). Agora segure o MEIO\n\
             de uma das marcadas e arraste: aparecem DOIS contornos --\n\
             um para cada marcada -- e, ao soltar, as duas mudam de\n\
             lugar JUNTAS, a mesma distancia. O destaque acompanha as\n\
             caixas movidas. As caixas NAO marcadas ficam onde estao, e\n\
             o grupo ENCOSTA nelas e para, como no Teste 1. Arrastar uma\n\
             caixa nao marcada move so ela. Para desmarcar, clique numa\n\
             caixa sem Shift.\n\
             \n\
             ------------------------------------------------------------\n\
             TESTE 5 -- 'Trace': deslizar o VULTO para calcar\n\
             ------------------------------------------------------------\n\
             No painel do Flip (coluna DIREITA), clique no botao\n\
             'Trace'. Agora arraste o VULTO azul no canvas: ele desliza\n\
             com o mouse -- e' so a referencia; va ate a caixa daquele\n\
             desenho e confira que a bola dele esta onde sempre esteve.\n\
             Segure CTRL e arraste para GIRAR o vulto em torno do\n\
             proprio centro. Volte a 'Draw': o vulto fica onde voce o\n\
             deixou (a folha de papel deslizada do animador -- e' para\n\
             desenhar por cima dela). De volta em 'Trace', o botao\n\
             'Reset Shifts' devolve todos os vultos ao lugar.\n\
             \n\
             ------------------------------------------------------------\n\
             TESTE 6 -- F1/F2/F3: folhear os desenhos sem sair do lugar\n\
             ------------------------------------------------------------\n\
             Clique numa caixa do MEIO (a amarela ou a ciano). Agora\n\
             SEGURE F1: a tela mostra SO o desenho anterior, na cor\n\
             real, sem vulto nenhum -- e' voce levantando a folha de\n\
             baixo para conferir o arco. Solte: tudo volta. F3 mostra o\n\
             desenho SEGUINTE; F2 mostra o desenho ATUAL sozinho (os\n\
             vultos somem enquanto segura). O playhead nao se move; na\n\
             primeira caixa, F1 nao tem para onde folhear e fica.\n\
             \n\
             ============================================================\n\
             Se algo nao fizer o que esta escrito, me diga O QUE\n\
             ACONTECEU -- e' so isso que eu preciso.\n\
             ============================================================\n"
        );
    }
}

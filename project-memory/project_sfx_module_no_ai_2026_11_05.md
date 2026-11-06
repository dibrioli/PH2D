---
name: project-sfx-module-no-ai-2026-11-05
description: "Dono 05/11 — o módulo de criação de efeitos sonoros para games sai SEM IA; a proposta é só síntese, exploração de ouvido e UI"
metadata:
  node_type: memory
  type: project
  originSessionId: 11886d09-1a04-4fd6-9e34-86d20a83a832
  modified: 2026-11-05T23:15:50.503Z
---

O dono decidiu em 2026-11-05: «vamos esquecer a AI» no módulo de criação de efeitos sonoros (SFX) para games. Vale tanto para a IA que gera áudio quanto para a IA pequena de busca, nomes e «descreva, receba uma receita».

**Why:** a pesquisa de 05/11 (`docs/SfxLab/pesquisa/relatorio.md`, na `line/SfxLab`) mostrou que o valor do módulo está na receita viva e na cena do jogo; o dono prefere concentrar a análise na ideia e na UI.

**How to apply:** não proponha IA neste módulo salvo pedido dele. As partes de IA do relatório ficam só como registro histórico.

**Dono 05/11, depois da crítica (`docs/SfxLab/pesquisa/critica.md`):** «faça tudo como o agente achar melhor» aprova as recomendações da crítica: consertar antes o caminho até o jogo (vários sons por objeto, sorteio, Autoplay, marcador no quadro); PH2D primeiro, com exportação excelente e sem ligação direta com Wwise/FMOD; biblioteca própria gravada; modelo de nome escrito pelo usuário não conta como IA; AI Denoise fora do módulo, com a ponte para o Editor; sem pintura de som nem metáfora de planta; microfone desde o começo; som assado antes do ao vivo; profissional testa às cegas antes de congelar a tela. «SFX Lab» é nome provisório. Gravar a biblioteca e convidar o profissional dependem do dono.

**Dono 06/11, smoke do corpo vibrante:** aprovado, W3 liberada. O Drum Thump (base 95 Hz) «quase não se ouve» nas caixas pequenas dele; a receita não muda sem pedido dele. Em smoke de som grave, lembre que ele pode estar ouvindo em caixas pequenas.

**Dono 06/11, smoke da tela (W3):** tudo OK. Duas ordens: com Play on change ligado, mudar o S toca a faixa em solo; o Use in Game abre um modal bem desenhado, com campo de busca e um eyedropper que escolhe o objeto na Hierarchy (a lista simples sai). Ctrl+Z fica como está: na aba de som desfaz a edição do som, e a ligação com o objeto só na aba Scene.

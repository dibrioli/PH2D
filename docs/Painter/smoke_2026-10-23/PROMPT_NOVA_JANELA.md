# Prompt da janela seguinte — line/PainterWatercolor (reaberta 23/10)

> Smoke do dono 23/10: **OK para tudo** o que a linha entregou (45 pincéis, navegador por categoria, Paint Mode
> no topo, Filbert, Grain Colors nos 4 meios, Voronoi da Aquarela, preview pelo caminho da tela, pino da preview,
> Same as Paper esconde o grão, o vazamento do miolo branco). A linha está no main (`972f411a6f`, CI verde) e foi
> reaberta em fast-forward sobre ele.

```
Linha line/PainterWatercolor (Modo L), reaberta depois da integração de 23/10.
FASE 0: cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && pwd && git branch --show-current
Leia: CLAUDE.md (§0, §2), o handoff docs/Painter/handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-10-22_PINCEIS.md
(o que ficou aberto) e project-memory/MEMORY.md. Delegue leitura e verificação a agentes (explorador, verificador).

TAREFA 1 — BUG do smoke 23/10, modo WATERCOLOR (não Wet Paint):
no PRIMEIRO traço sobre uma poça SECA, com Smudge > 0, a borda da poça anterior fica PIXELADA e ESCURECE, como
uma sombra (degraus escuros ao longo da orla onde o traço novo a cruza). Depois de outras pinceladas o aspecto
melhora, mas a primeira fica estranha. Capturas do dono:
  docs/Painter/smoke_2026-10-23/smudge_poca_seca_antes.png (a poça amarela seca)
  docs/Painter/smoke_2026-10-23/smudge_poca_seca_primeiro_traco.png (o 1.º traço amarelo por cima: setas verdes
  na orla escura e em degraus)
Avalie e corrija na raiz:
- Reproduza em teste pelo caminho real do PainterTool (poça Watercolor → secar → 1.º traço com Smudge > 0
  cruzando a orla) e MEÇA: tamanho dos degraus (px) e o escurecimento da orla face à poça antes do traço.
  Compare com o 2.º/3.º traço (o dono diz que melhora) — a diferença entre o 1.º e os seguintes é a pista:
  estado/caches inicializados só depois do 1.º traço (amostra do smudge lida de uma grelha grossa, a camada
  seca ainda não promovida/convertida, o buffer de pickup vazio/zerado no 1.º dab, a orla seca reacesa sem
  filtro). Onde o defeito ATERRA ≠ o que o PROVOCA.
- Lição da linha: a Aquarela já teve um defeito de resolução por ler a cor num ponto DESLOCADO e por vizinho
  mais próximo (`93c454c37a`, pigment::texel_do_deposito) e um de estado que vazava entre traços
  (`b58532d9ce`, marcas_da_aguada). Confira se é da mesma família antes de inventar outra lei.
- Cura aditiva: com Smudge 0, e nos outros meios, tudo ao bit (fingerprints a_lei_da_tinta, ph2d-wet-paint,
  os_outros_meios_nao_mudam, midias_de_fabrica, presets_de_fabrica). Portão red-first (o 1.º traço = os
  seguintes; degrau ≤ 1 px; a orla não escurece além da medida sem smudge) + prova de mutação. Foto
  antes/depois pelo teste, olhada.
- Clippy -D warnings --all-targets com PH2D_SUITE=fecho e fmt só no fecho; testes por nome no meio.
- Resposta ao Enio: curta, sem jargão, smoke em passos numerados com o cd.
```

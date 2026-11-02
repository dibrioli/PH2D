---
name: project_wetpaint_refactor_and_illustrated_help_2026_10_31
description: "Dono 31/10 noite — refatorar o Wet Paint por áreas (parâmetros com grandeza física, uma lei de camada) + ajuda ilustrada estilo Fusion 360 por parâmetro"
metadata:
  node_type: memory
  type: project
  originSessionId: b289fefd-0e44-4e9a-852a-6f0399faba0a
  modified: 2026-10-31T21:32:01.597Z
---

Dono 31/10 (noite), depois de `dono_6.png` (Thick Impasto com pigmento ↓ e água ↑: azul concentrado e ferrugem): propôs
refatoração profunda do Wet Paint (e do Painter se preciso) — parâmetros agrupados por área, cada um entendido e
comparado com a literatura, simplificação que dê resultados consistentes — e um sistema de ajuda como o do Fusion 360:
cada parâmetro mostra em texto E imagem o que faz.

**Why:** as curas sintoma-a-sintoma do óleo (branco, caroços, orla ferrugem) abriam o defeito vizinho; aquarela está
estável, o óleo de alto relevo não.

**How to apply:** prompt em `docs/Painter/49_realshader/PROMPT_REFATORACAO_WETPAINT_E_AJUDA.md` (worktree
line-PainterWatercolor). Imagem da ajuda gerada pelo próprio motor (não mente); maquete do cartão aprovada pelo dono
antes da 1.ª tela. Dono 31/10 (mesma noite): *«não meça esforços … cada parâmetro plenamente funcional, sendo o
comportamento estável, reprodutível, confiável»* — escopo = TODO o Painter nos 4 modos, docs em `docs/Painter/Arquitetura/`
com fotos geradas pelo motor; critério de pronto = cada controlo vivo em cada meio + varredura do painel inteira verde +
reprodutível (mesmo bit por placa). Dono 31/10: as imagens dos cartões de ajuda pintam-se sobre o PAPEL TEXTURIZADO
PADRÃO do app (não liso) — e as réguas/sondas da refatoração também, porque é onde o artista pinta. ⭐ PRIORIDADE Nº 1 do dono (31/10): óleo GROSSO
sobre óleo grosso, mesmo com pouca água, escorre como líquido — ondas e crateras ao sobrepor; tem de ser pasta de
Bingham (τ_y: só se move onde o pincel empurra), régua em `docs/Painter/Arquitetura/RELEVO_oleo_ondas_e_crateras.md`.
Texto do cartão
em Plain English ([[app-ui-english-only]]). Ver [[project_superar_rebelle_autonomous_2026_10_31]].


**Respostas do dono (31/10, janela 2):** (1) a foto F0 (`fotos/f0_varredura/F0_para_o_dono.png`) É o defeito que ele viu (ferrugem + azul amontoado) — a régua-mãe F0 está confirmada; (2) cartão de ajuda = variante **C (animado)**: pincelada grande que passa sozinha low→medium→high + 3 miniaturas; (3) QUER o opcional por pincel «Pigment mix» (azul+amarelo = verde, WGM), desligado por omissão — a mistura robusta (Oklab pelo volume) continua a ser a padrão.

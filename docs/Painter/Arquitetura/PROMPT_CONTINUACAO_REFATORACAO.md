# PROMPT — continuação da refatoração do Painter (janela 5)

`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor` · `pwd` · `git branch --show-current`
(= `line/PainterWatercolor`) ANTES de ler. Base: o HEAD com o commit «handoff da janela 4».

Leia, nesta ordem: [HANDOFF_01_11_janela4.md](HANDOFF_01_11_janela4.md) (estado, a grelha a correr, aberto) ·
[F3_controlos.md](F3_controlos.md) **§8 primeiro** (o smoke do dono manda sobre o §0–§7) e §2–§4 · a memória
`project-memory/project_painter_panel_e10_owner_decisions_2026_11_01.md` · [F3b_medicoes.md](F3b_medicoes.md) ·
[E10b_pigment_mix.md](E10b_pigment_mix.md) §5–§6 · [ferramentas/briefing_comum_agentes.md](ferramentas/briefing_comum_agentes.md)
(o briefing de todo agente; acrescente-lhe: o meio vive em `Soma::m`, o ecrã decompõe em sRGB por `decompoe`, o
Pigment mix vive no `Espectro` lateral `Grid::pigmento`).

**Missão do dono (31/10):** refatoração profunda, completa e brutal de TODO o Painter nos 4 modos (Digital ·
Watercolor · Impasto · Wet Paint) — cada parâmetro plenamente funcional, estável, reprodutível, confiável; docs com
FOTOS do motor nesta pasta. «Não meça esforços.» Já respondido (não repergunte): foto F0 = o defeito ✓ · cartão de
ajuda = **C animado** · «Pigment mix» por pincel ✓ · as 9 perguntas da E10 (F3 §8).

**Ordem desta janela:**
1. A grelha E9 (handoff §2): confirme que acabou (ou relance), resuma, escreva o E9 §8 com fotos.
2. **E10 no painel de HOJE** (F3 §8, handoff §4.2): os controlos de lei nas secções existentes, no padrão do
   Inspector; nenhuma secção comum aos 4 modos sai; todo knob vivo fica; Tilt e Spacing «Continuous» intocados; tools
   na barra vertical. Antes da 1.ª linha, leia o painel de hoje (`tp/wetpaint_settings.rs`, o registo de ids, o i18n)
   e escreva no F3 a tabela «controlo → secção de hoje → id».
3. Camada fina presa ao matiz (Q8) · Body como tensão de cedência (Q9, cura o Bug #56) · o aberto da E10c.
4. Depois: E11 GPU · E12 · E13 · os outros 3 modos · ajuda (F5, C) · sondas para a crate dev-only · crateras da costura.

**Regras desta linha** (além do CLAUDE.md): coordenador autónomo, sem perguntas salvo aprovação de UI; a UI nova é
mostrada ao dono NO APP (smoke em passos numerados), nunca numa maquete fora do padrão; agentes ≤ 3 em paralelo, 45
min, ficheiros disjuntos; linha ⛔ em todo briefing. Testes POR NOME; `--profile smoke`; comando pesado por
`bash scripts/ph2d-run.sh`; NVIDIA proibida (`PH2D_WET_GPU=1`/testes `ph2d-wet-gpu` com a RTX — travou o PC 2× em
31/10; o app corre na Radeon com `VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/radeon_icd.json`); só a janela principal
regera miniaturas. O guarda do commit corre ANTES do comando: `rustfmt` num passo separado; no zsh, listas de paths em
array `P=(…)`; ficheiros ≤ 700 linhas; `sed -i` só com `&& grep -c` na mesma linha. `cargo-check-narrow` NÃO apanha
erros nos módulos de teste da sonda — compile com `cargo test … --no-run`. Bisecção = árvore inteira por commit
(handoff §4.7). Não integrar no main nem push. Respostas ao dono: curtas, sem jargão, smoke em passos numerados; o app
é SÓ em inglês.

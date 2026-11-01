# PROMPT — continuação da refatoração do Painter (janela 4)

`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor` · `pwd` · `git branch --show-current`
(= `line/PainterWatercolor`) ANTES de ler. Base: o HEAD com o commit «handoff da janela 3».

Leia, nesta ordem: [HANDOFF_31_10_janela3.md](HANDOFF_31_10_janela3.md) (estado, a grelha que estava a correr, aberto)
· [E9_portao_mae.md](E9_portao_mae.md) §7 · [F3_controlos.md](F3_controlos.md) (rascunho da E10, por rever) ·
[ferramentas/briefing_comum_agentes.md](ferramentas/briefing_comum_agentes.md) (o briefing de todo agente; acrescente-lhe:
o meio vive em `Soma::m`, o ecrã decompõe em sRGB por `decompoe`).

**Missão do dono (31/10):** refatoração profunda, completa e brutal de TODO o Painter nos 4 modos (Digital ·
Watercolor · Impasto · Wet Paint) — cada parâmetro plenamente funcional, estável, reprodutível, confiável; docs com
FOTOS do motor nesta pasta. «Não meça esforços.» Respostas já dadas (não repergunte): foto F0 = o defeito ✓ · cartão
de ajuda = **C animado** · **«Pigment mix»** por pincel = sim.

**Ordem desta janela:**
1. A grelha E9 (handoff §2): confirme que acabou, resuma, escreva o E9 §8 com fotos.
2. Thick Impasto `mix_p99 9,5` (handoff §3.2): o desvio de cor entra pela ferramenta — meça e cure.
3. **E10**: reveja o F3 como coordenador (decida o técnico); leve ao dono, num só smoke com a maquete e a foto da borda
   fina, as perguntas do handoff §3.3–§3.4; implemente o painel quando a lei nova puder ser ligada nele.
4. Depois: «Pigment mix» (WGM 10 bandas, F2b §4.2) · E11 GPU · E12 · E13 · os outros 3 modos · ajuda (F5, C) · sondas
   para a crate dev-only · crateras da costura.

**Regras desta linha** (além do CLAUDE.md): coordenador autónomo, sem perguntas salvo aprovação do cartão/painel;
agentes ≤ 3 em paralelo, 45 min, ficheiros disjuntos; linha ⛔ em todo briefing. Testes POR NOME; `--profile smoke`;
comando pesado por `bash scripts/ph2d-run.sh`; NVIDIA proibida (`PH2D_WET_GPU=1`/testes `ph2d-wet-gpu` com a RTX — travou o
PC 2× em 31/10; o app corre na Radeon com `VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/radeon_icd.json`); só a janela
principal regera miniaturas. O guarda do commit corre ANTES do comando: `rustfmt` num passo separado; no zsh, listas
de paths em array `P=(…)`; ficheiros ≤ 700 linhas. `cargo-check-narrow` NÃO apanha erros nos módulos de teste da
sonda — compile com `cargo test … --no-run`. Não integrar no main nem push. Respostas ao dono: curtas, sem jargão,
smoke em passos numerados; o app é SÓ em inglês.

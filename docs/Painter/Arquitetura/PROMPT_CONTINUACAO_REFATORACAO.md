# PROMPT — continuação da refatoração do Painter (janela 3)

`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor` · `pwd` · `git branch --show-current`
(= `line/PainterWatercolor`) ANTES de ler. Base: o HEAD com o commit «handoff da janela 2».

Leia, nesta ordem: [HANDOFF_31_10_janela2.md](HANDOFF_31_10_janela2.md) (estado, respostas do dono, o portão que
reprova e porquê, aberto) · [E9_portao_mae.md](E9_portao_mae.md) §5–§6 · [F4_2_plano_wet_paint.md](F4_2_plano_wet_paint.md)
§7 (E10) e §7.2 · [ferramentas/briefing_comum_agentes.md](ferramentas/briefing_comum_agentes.md) (o briefing de todo agente).
O ADR-0197 e o resto da pasta só quando a etapa pedir.

**Missão do dono (31/10):** refatoração profunda, completa e brutal de TODO o Painter nos 4 modos (Digital ·
Watercolor · Impasto · Wet Paint) — cada parâmetro plenamente funcional, estável, reprodutível, confiável; docs com
FOTOS do motor nesta pasta. «Não meça esforços.» Respostas do dono já dadas (não repergunte): foto F0 = o defeito ✓ ·
cartão de ajuda = **C animado** · **«Pigment mix»** por pincel = sim.

**Ordem desta janela:**
1. **E9 fecho** (HANDOFF §4.1): a régua do portão lê a tinta SECA; o depósito escreve `m` pelo meio do pincel (óleo 1,
   aquarela 0 — o mínimo da E10); a franja da orla (`mix_max 157`); as manchas do azul. Célula da E9 perto dos
   limiares ⇒ a grelha inteira (2 leis × 2 papéis) por `scripts/fila-pesada.sh`, UMA corrida.
2. **E10**: o painel do Wet Paint por áreas (PAINT · BRUSH · PAPER · TIME · LIGHT, em inglês), um controlo = uma
   grandeza, 177 controlos → N com a tabela de migração; presets mapeados por função. Mostrar ao dono (smoke) só
   quando a lei nova puder ser ligada no painel.
3. «Pigment mix» (WGM 10 bandas, F2b §4.2) · E11 GPU · E12 a velha sai num bloco · E13.
4. Depois: os outros 3 modos na mesma lei, a ajuda ilustrada (F5, variante C), as sondas para a crate dev-only, e por
   fim as crateras na costura entre pinceladas.

**Regras desta linha** (além do CLAUDE.md): coordenador autónomo, sem perguntas salvo aprovação do cartão/painel;
agentes ≤ 3 em paralelo, 45 min, ficheiros disjuntos; linha ⛔ em todo briefing (está no briefing comum). Testes POR
NOME; `--profile smoke`; comando pesado por `bash scripts/ph2d-run.sh`; NVIDIA proibida (`PH2D_WET_GPU=1`/testes
`ph2d-wet-gpu` com a RTX — travou o PC 2× em 31/10; o app corre na Radeon com
`VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/radeon_icd.json`); só a janela principal regera miniaturas. O guarda do commit
corre ANTES do comando: `rustfmt` num passo separado; no zsh, listas de paths em array `P=(…)`. Não integrar no main
nem push. Respostas ao dono: curtas, sem jargão, smoke em passos numerados; o app é SÓ em inglês.

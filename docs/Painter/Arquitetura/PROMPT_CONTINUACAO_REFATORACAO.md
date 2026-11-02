# PROMPT — continuação da refatoração do Painter (janela 13)

`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor` · `pwd` · `git branch --show-current`
(= `line/PainterWatercolor`) ANTES de ler. Base: o HEAD com o commit «handoff da janela 12». Se o integrador avisar que
o main novo saiu (com ou sem o Painter), `git rebase main` primeiro.

Leia, nesta ordem: [HANDOFF_02_11_janela12.md](HANDOFF_02_11_janela12.md) (estado; o §3 é a ordem) · o
[handoff de integração de 02/11](../handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-02_VERMELHOS.md) (os 4
vermelhos conhecidos e o que cada cura mudou) · [J11](J11_agua_no_oleo.md) §4 (o FPS) · [J9](J9_meio_no_liquido.md) ·
[F2_grandezas_fisicas.md](F2_grandezas_fisicas.md) · [F3_controlos.md](F3_controlos.md) §8–§9 ·
[ferramentas/briefing_comum_agentes.md](ferramentas/briefing_comum_agentes.md) (o briefing de todo agente; acrescente:
o chão da água vive em `Grid::chao` (`wp/solver/chao_forca.rs`), o meio em `Soma::m`, o Pigment mix no `Espectro`
`Grid::pigmento`, o Real Oil na célula, o piso do espalhar vale sobre o ACUMULADO do traço (`Espalha::erguido`), o
undo do motor guarda o cursor do passo em voo (`PassoEmVoo`)).

**Missão do dono (31/10):** refatoração profunda, completa e brutal de TODO o Painter nos 4 modos (Digital ·
Watercolor · Impasto · Wet Paint) — cada parâmetro plenamente funcional, estável, reprodutível, confiável; docs com
FOTOS do motor nesta pasta. «Não meça esforços.» O Rebelle NÃO é referência (31/10): a lei é a do próprio Painter.
Já respondido (não repergunte): cartão de ajuda = **C animado** · «Pigment mix» por pincel · as 9 perguntas da E10
(F3 §8) · UMA lei, sem segundo fluido (Medium = «Paint type: Water · Oil») · Wet Liner sem fio escuro · lápis marcam
no toque leve · cena macro 1 (Splash) fica.

**Smoke da janela 12:** «smoke OK». **Ordem desta janela** = o §3 do handoff da janela 12: (1) o FPS com o Real Oil,
medido NO APP (tela real, FPS, Radeon) antes de mexer — quem paga por passo onde a água corre sobre o óleo; (2) o
rectângulo velho na shell; (3) os vermelhos conhecidos (Transmission Blender na malha real, Relevo do papel, Smear
Wet); (4) a lista do §4 da janela 7. Smoke do dono no fim da onda, em passos numerados, com o app recompilado
(`--profile smoke`, Radeon, NO WORKTREE).

**Regras desta linha** (além do CLAUDE.md): coordenador autónomo, sem perguntas salvo aprovação de UI; a UI nova é
mostrada ao dono NO APP (smoke em passos numerados), nunca numa maquete fora do padrão; agentes ≤ 3 em paralelo, 45
min, ficheiros disjuntos; linha ⛔ em todo briefing. Testes POR NOME; `--profile smoke`; comando pesado por
`bash scripts/ph2d-run.sh`; NVIDIA proibida (`PH2D_WET_GPU=1`/testes `ph2d-wet-gpu` com a RTX — travou o PC 2× em
31/10; o app corre na Radeon com `VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/radeon_icd.json`); só a janela principal
regera miniaturas. O guarda do commit corre ANTES do comando: `rustfmt` num passo separado; no zsh, listas de paths em
array `P=(…)`; ficheiros ≤ 700 linhas; `sed -i` só com `&& grep -c` na mesma linha. `cargo-check-narrow` NÃO apanha
erros nos módulos de teste da sonda — compile com `cargo test … --no-run`. Bisecção = árvore inteira por commit.
**Lições de 01/11:** o `cargo-test-narrow.sh` engole o `--nocapture` (sonda que imprime: `cargo test -p … --profile smoke --lib -- --ignored <nome> --nocapture` por `ph2d-run.sh`) · um filtro de nome largo (`diag_o_`) corre sondas alheias em paralelo e baralha os interruptores globais — nome exato e `--test-threads=1` · uma força multiplicada pela água EMPILHA a poça (J10 §4) · o Size do pincel é o RAIO (cena que põe o pé do pincel a N px usa o raio) · a advecção PUXA (corte de fluxo vai no destino) · o campo `Grid::oleo` existe sem Real Oil (o meio vive lá; a óptica lê `Grid::oleo_real`) · `/tmp` é RAM (62 GB) — builds, worktrees, bisecções e grelhas em `target/` do worktree, em `/home`,
um worktree de bisecção de cada vez · vigie a SAÍDA a crescer, nunca `pgrep -f <texto>` (casa a si mesmo) · todo
«vermelho depois da minha mudança» leva A/B no commit pai (árvore inteira) ANTES de regravar ou afrouxar — 5 de 6 já
estavam vermelhos · a grelha E9 inteira = 45 min direto por `ph2d-run.sh` com a saída em `/home`. **Lições de 02/11:** o gerador de presets pede `PH2D_MEM_MAX=64G` (pico 33–35 GB; OOM a 24G) e um preset de cada vez · no zsh `set -- $var` não parte palavras (use arrays) · régua retirada por paridade com o Rebelle leva junto a lei própria que partilhava o teste — separe-as e reponha a nossa com tecto MEDIDO e mutação · um `99`/sentinela que «passa» esconde o mudo · cura de outra frente entra na linha ANTES de um teste verde noutra worktree: re-meça na linha. Não integrar no main
nem push. Respostas ao dono: curtas, sem jargão, smoke em passos numerados; o app é SÓ em inglês.

# PROMPT — continuação da refatoração do Painter (janela 11)

`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor` · `pwd` · `git branch --show-current`
(= `line/PainterWatercolor`) ANTES de ler. Base: o HEAD com o commit «handoff da janela 10».

Leia, nesta ordem: [HANDOFF_01_11_janela10.md](HANDOFF_01_11_janela10.md) (estado; o §2–§3 é por onde começar) · [J10](J10_chao_como_forca.md) · [bisseção](J10_bissecao_vermelhos.md) · [J9](J9_meio_no_liquido.md) · [J8](J8_vermelhos_antigos.md) · [J7](J7_pasta_do_oleo.md) · [E10g](E10g_oleo_diluido.md) · [E10i](E10i_ferrugem_velha.md) · a memória
`project-memory/project_paint_type_water_oil_one_law_2026_11_01.md` (repo principal) ·
[E10e2_km_e_remolha.md](E10e2_km_e_remolha.md) §E3 · [E10a2_meio_e_corpo.md](E10a2_meio_e_corpo.md) ·
[F2_grandezas_fisicas.md](F2_grandezas_fisicas.md) (leis de viscosidade e secagem) · [F3_controlos.md](F3_controlos.md)
§8–§9 · [ferramentas/briefing_comum_agentes.md](ferramentas/briefing_comum_agentes.md) (o briefing de todo agente;
acrescente-lhe: o chão da água vive em `Grid::chao` (`wp/solver/chao_forca.rs`), o meio vive em `Soma::m`, o ecrã decompõe em sRGB por `decompoe`, o Pigment mix vive no `Espectro` lateral
`Grid::pigmento`, o Real Oil vive na célula).

**Missão do dono (31/10):** refatoração profunda, completa e brutal de TODO o Painter nos 4 modos (Digital ·
Watercolor · Impasto · Wet Paint) — cada parâmetro plenamente funcional, estável, reprodutível, confiável; docs com
FOTOS do motor nesta pasta. «Não meça esforços.» Já respondido (não repergunte): foto F0 = o defeito ✓ · cartão de
ajuda = **C animado** · «Pigment mix» por pincel ✓ · as 9 perguntas da E10 (F3 §8) · **UMA lei, sem segundo fluido: o
Medium vira o seletor «Paint type: Water · Oil»; o solvente é sempre líquido** (01/11).

**Smoke da janela 10** (fotos e palavras no §2 do handoff): o empoçar ficou certo. **Ordem desta janela** = o §3 do
handoff da janela 10: (1) num bloco, a família água × óleo seco: a água entra no corpo do óleo mais depressa que escorre
no papel (irreal com o relevo), a mistura que DISSOLVE e BAIXA o relevo (o dono: «deve ser configuração, já vi
funcionando» — `Knob::Solubility`, fábrica 0) e o pixelado duro (suspeito: a grade de fluxo grossa) — medir primeiro;
(2) o FPS com Real Oil: achar quem marca a caixa molhada inteira a cada tique, depois religar o filtro «só o que se vê»
(J10 §6); (3) os vermelhos antigos (N e A com mecanismo; P e S por isolar). Smoke do dono no fim da onda, em passos
numerados, com o app recompilado (`--profile smoke`, Radeon).

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
estavam vermelhos · a grelha E9 inteira = 45 min direto por `ph2d-run.sh` com a saída em `/home`. Não integrar no main
nem push. Respostas ao dono: curtas, sem jargão, smoke em passos numerados; o app é SÓ em inglês.

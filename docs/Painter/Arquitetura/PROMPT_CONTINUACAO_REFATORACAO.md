# PROMPT — continuação da refatoração do Painter (janela 2)

`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor` · `pwd` · `git branch --show-current`
(= `line/PainterWatercolor`) ANTES de ler. Base: `541d84c785` (ou o HEAD seguinte, só docs).

Leia, nesta ordem: [HANDOFF_31_10_noite.md](HANDOFF_31_10_noite.md) (estado, decisões do dono, aberto) ·
[README.md](README.md) (a pasta) · [ADR-0197](../../architecture/decisions/0197-o-painter-e-uma-lei-de-pigmento-em-volumes-e-os-meios-sao-pontos-dela.md) ·
[F4_2_plano_wet_paint.md](F4_2_plano_wet_paint.md) (as etapas). O resto da pasta só quando a etapa pedir.

**Missão do dono (31/10):** refatoração profunda, completa e brutal de TODO o Painter nos 4 modos (Digital ·
Watercolor · Impasto · Wet Paint) — cada parâmetro plenamente funcional, comportamento estável, reprodutível e
confiável; documentação técnica com FOTOS geradas pelo motor nesta pasta, para a LLM estudar. «Não meça esforços.»
A esperança do dono: a simplificação deixa os defeitos evidentes e fáceis de resolver.

**Ordem desta janela:**
1. Faça as 3 perguntas pendentes do HANDOFF §4 ao dono logo no início (curtas, sem jargão).
2. E5 → E9 do F4_2 (a lei nova a pintar atrás do interruptor `Sim::tinta`, a varredura F0 nas DUAS leis, sobre o
   papel texturizado padrão) — a velha só sai (E12) quando o portão-mãe passar inteiro.
3. E10: o painel do Wet Paint por áreas (PAINT · BRUSH · PAPER · TIME · LIGHT, em inglês), um controlo = uma grandeza,
   177 controlos → N com a tabela de migração; presets mapeados por função.
4. Depois: os outros 3 modos na mesma lei (HANDOFF §3.3), a ajuda ilustrada (F5, com a variante que o dono escolher),
   as sondas para a crate dev-only, e por fim as crateras na costura entre pinceladas (HANDOFF §3.1).

**Regras desta linha** (além do CLAUDE.md): coordenador autónomo, sem perguntas salvo as do §4 e a aprovação do
cartão; agentes ≤ 3 em paralelo, 45 min; linha ⛔ em todo briefing: «`cd X && …` · `bash -c`/`sh -c` · `rm -rf` com
variável/glob · `kill` de PID não gravado · escrever em `.git/`/`.claude/` · AskUserQuestion · push · commit (o
coordenador comita) · tocar fora dos seus paths · spectacle · NVIDIA (`PH2D_WET_GPU=1`/testes `ph2d-wet-gpu` que
exijam a RTX — travou o PC 2× em 31/10; o app corre na Radeon com
`VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/radeon_icd.json`)». Testes POR NOME; `--profile smoke`; comando pesado por
`bash scripts/ph2d-run.sh`; só a janela principal regera miniaturas (`gera_os_presets_de_fabrica` com
`PH2D_REGERA_PRESETS=1 PH2D_REGERA_SO_MINIATURA=1 PH2D_REGERA_SO="…"`, conferir o JSON igual e a foto antes ×
depois). O guarda do commit corre ANTES do comando: `rustfmt` num passo separado. Não integrar no main nem push.
Respostas ao dono: curtas, sem jargão, smoke em passos numerados; o app é SÓ em inglês.

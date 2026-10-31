# PROMPT — Refatoração do Wet Paint por áreas + ajuda ilustrada por parâmetro (dono 31/10, noite)

`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor` · `pwd` · `git branch --show-current`
(= `line/PainterWatercolor`) ANTES de ler. Base: `b8265acf42` (fim da auditoria do óleo sobre óleo).

## 0. A queixa e a proposta (verbatim)

Captura [`smoke_31_10_oleo/dono_6.png`](smoke_31_10_oleo/dono_6.png) — Thick Impasto com **pigmento reduzido e água
aumentada**, amarelo sobre azul: manchas de azul ESCURO concentrado dentro do azul (seta do meio), pontos FERRUGEM na
orla (setas das pontas), verde em degraus por passada.

> «Cada vez mais e mais problemas aparecendo. Tudo indica que relacionados aos novos pincéis de óleo que nas últimas
> rodadas vem apresentando comportamento bizarro quanto a assentamento da tinta e mescla quando temos camadas de relevo
> altas. Na imagem os problemas aparecem quando reduzi pigmento e aumentei água de pincéis como thick impasto. Pincéis
> aquarela parecem continuar com bom comportamento. Pincéis de óleo de alto relevo apresentam mais inconsistências e
> erros.
> Considere a proposta: Criar nova arquitetura com profunda refatoração do módulo Wet Painter (e se necessário do Painter
> em geral) buscando organização mais sólida dos parâmetros, estudo e compreensão do que cada um faz, agrupamento dos
> parâmetros conforme a área de atuação, comparação com os docs científicos e técnicos da área e descoberta de meios de
> simplificação que torne os resultados mais consistentes e facilmente encontrados.
> Além disso criar um sistema de ajuda/dicas como do Fusion 360 que para cada parâmetro mostra em texto e imagem como
> aquele parâmetro atua.»

## 1. Leitura do coordenador anterior (porquê a proposta é a cura certa)

As últimas rodadas curaram SINTOMAS do óleo um a um (branco, caroços, orla ferrugem — [handoff](HANDOFF_AUDITORIA_OLEO_SOBRE_OLEO.md))
e cada cura abriu a vizinha: a lei da cor do óleo é a soma de modos sobrepostos (`Ecra::Cobre` com `filme · esconde ·
molhada · vela · oleo · opaca`, o sentinela `PINCEL_DE_OLEO = -1`, a rampa `OLEO_PLENO`, o tapar extra, a mistura RGB
da aguada sobre o óleo, a ladeira com 8 rotas). Os portões medem pontos isolados do espaço de parâmetros; o dono mexe
em DOIS botões (pigmento ↓, água ↑) e cai num canto que nenhum mede. ⇒ O defeito é de ARQUITETURA: parâmetros sem
grandeza física única, leis paralelas por meio, e nenhuma régua que varre o espaço que o painel deixa o artista
alcançar.

## 2. Fases (cada uma fecha com doc + commit; a F0 e a F5-maquete param para o dono)

**F0 — Reproduzir `dono_6` no app** (driver em [app_oleo/](app_oleo/), Radeon, `PH2D_OLEO_APP_PRESSAO`; acrescentar
pigmento/água ao driver). Uma **régua de VARREDURA** do espaço do painel: para cada pincel de óleo, grelha pigmento ×
água × pressão × N passadas, amarelo sobre azul assentado; mede distância à curva K-M (a `fora_da_mistura` de
`assenta/oleo_sobre_oleo_tests.rs`), manchas de concentração (azul mais escuro que o azul puro = pigmento que se
AMONTOOU), furos, rugosidade. Este é o portão-mãe da refatoração: a lei nova tem de o passar inteiro.

**F1 — Censo de cada parâmetro** (explorador ×2, partir por motor). Para cada controlo do Wet Paint e de cada preset:
quem escreve · quem lê · o leitor DECIDE? · faixa · em que meios age · unidades · duplicados (dois knobs, uma grandeza)
· mortos. Partir de [45_censo_dos_controlos.md](../45_censo_dos_controlos.md) e [46_plano_cada_controlo_em_cada_meio.md](../46_plano_cada_controlo_em_cada_meio.md)
— conferir no CÓDIGO, não acreditar no doc.

**F2 — Literatura → grandezas físicas.** Kubelka–Munk (mistura e camada fina sobre fundo), Curtis et al. 1997
(aquarela: água, pigmento em suspensão/depositado, granulação), Baxter IMPaSTo 2004 e dAb (óleo: volume, empasto,
pick-up/deposit), Rebelle como ORÁCULO que se corre (`reference_rebelle_trial_installed_under_wine`). Saída: a tabela
«controlo ↔ grandeza ↔ equação ↔ fonte», e a lista do que NÃO tem grandeza (candidato a sair ou a fundir).

**F3 — ADR da arquitetura nova.** Proposta de partida (a validar pela F1/F2):
- **Tinta** (o que está no pincel): pigmento (concentração), meio (água ↔ óleo, UMA escala contínua, não modos), carga.
- **Pincel** (como deposita): forma, cerdas, pressão→carga/área, pick-up.
- **Superfície**: papel/tela, absorção, relevo.
- **Tempo**: secagem, abertura (quanto o assentado volta a misturar).
- **Luz**: relevo visível, brilho.
Uma ÚNICA lei de camada (K-M sobre o assentado, com cobertura derivada de concentração × espessura) para todos os
meios — aquarela e óleo são PONTOS da mesma escala de meio, não ramos. Os sentinelas e modos (`PINCEL_DE_OLEO`,
`com_opaca`, mistura RGB da aguada) somem ou viram casos da lei. Migração: cada preset de fábrica mapeia para a lei
nova; as miniaturas são o golden (só mudam onde mostravam um defeito, e o doc diz qual).

**F4 — Refatorar por etapas** (Modo L, drop-in atrás de flag até a varredura da F0 passar inteira; depois a lei velha
sai no mesmo bloco — nada de duas leis vivas). Testes por nome; a régua de varredura e as miniaturas a cada etapa.

**F5 — Ajuda ilustrada estilo Fusion 360.** Pairar sobre o controlo → cartão com título, 1–2 frases do que faz, e uma
IMAGEM: a mesma pincelada a 3 valores do parâmetro (baixo · meio · alto), **gerada pelo próprio motor** como as
miniaturas (portão: a imagem da ajuda é a que o motor pinta hoje — a ajuda nunca mente). Texto por `ph2d-i18n`, sem
hex/literais (HR-15). ⛔ **Maquete do cartão aprovada pelo dono ANTES da primeira tela** (precedente Kanban 27/10).
Reaproveitável para o resto do app depois (Inspector inteiro), mas o 1.º consumidor é o Wet Paint.

## 3. Restrições (as da auditoria anterior valem)

- Coordenador autónomo, sem perguntas, salvo as duas paragens acima (F0 confirmada em foto · maquete da F5).
- Agentes ≤ 3 em paralelo, 45 min cada. Linha ⛔ em todo briefing: «`cd X && …` · `bash -c`/`sh -c` · `rm -rf` com
  variável/glob · `kill` de PID não gravado · escrever em `.git/`/`.claude/` · AskUserQuestion · push · tocar fora dos
  seus paths · spectacle · NVIDIA (`PH2D_WET_GPU=1` / testes `ph2d-wet-gpu`)».
- ⛔ A RTX travou o PC duas vezes em 31/10 (Xid 109): nada na NVIDIA; o app corre na Radeon (`VK_ICD_FILENAMES`).
- Comando pesado por `bash scripts/ph2d-run.sh`; testes POR NOME; `--profile smoke`; só a janela principal regera
  miniaturas. NÃO integrar no main nem push.
- Vermelhos ANTERIORES conhecidos (não são desta onda): ver §4 do [handoff](HANDOFF_AUDITORIA_OLEO_SOBRE_OLEO.md).
- Respostas ao dono: curtas, sem jargão, smoke em passos numerados.

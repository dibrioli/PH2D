# Comissão de velocidade (29/10/2026) — reduzir o tempo de implementar, testar e publicar

> Leitor: a janela que implementa as propostas aprovadas pelo dono (30/10). Origem: 4 pesquisadores
> (compilação · testes · integração e publicação · processo dos agentes) + debate, a pedido do dono
> depois da J6 da linha PainterWatercolor (28–29/10). Os relatórios brutos estavam em `/tmp` e perderam-se
> no desligar; este ficheiro guarda os números e as propostas. **Medido** = de logs/transcrições/`git log`/`gh run`;
> **estimado** = conta sobre o medido, a confirmar com A×B antes de adoptar.

## Decisão do dono (30/10)

**FAZER: A, B, D, E, F, G, H, I, K, L, M. NÃO fazer: C (CI verde só com Linux) e J (fila global de 2 linhas pesadas).**
O item «filtro de prazo do nextest morto» já foi feito em 29/10 (`scripts/nextest-filtros-casam.py`, no `ship.sh`).

## Onde vai o tempo (medido)

- **Integração em série:** 5 linhas (GameEngineFuncional, BlenderConstraints, UIUX, TrelloClone, PixelArt) levaram
  ~9 h, ~1 h 50 por linha; ~11 suítes completas de 21–31 min (~4,5–5 h, ~50 %). A 1.ª suíte de cada linha deu
  SEMPRE vermelho (1–38 falhas); a 2.ª, inteira, só confirmou 1–7 curas pequenas. A 5.ª linha esperou ~7 h.
- **Ship:** o `ship.sh` corre outra `nextest --workspace` sobre a MESMA árvore que o integrador acabou de passar
  (+25–35 min). CI: 84–101 min nas corridas verdes (caminho crítico: lint 10–12 min → Windows 62–75 min → C9);
  em 29/10 deu 1 h 00. O CI cobre só 26 das 313 crates.
- **Suíte:** 30 551 testes, 2 166 s (integração 29/10). `nextest-impacted` do fecho: 19 469 testes em 1 544 s,
  16 632 s de teste somados; **30 testes ≥ 120 s = 46 % do total; 53 ≥ 60 s = 58 %; 214 ≥ 10 s = 84 %**.
  Maiores: `ph2d-tool-painter` 9 545 s (57 %); `macro_track::tutorial` 1 862 s; `ph2d-panel-painter-layers`
  3 022 s (censo 390 s); `a_largura_e_a_prometida` 599 s; sonda de monotonia 1 669 s; `ph2d-skeleton-live`
  835–1 043 s; Luz 2D 769 + 248 s. TIMEOUTs sob load 50–146 queimaram ~3 060 s.
- **Regerar presets:** 1 preset de ferramenta 27–80 min (a tabela de largura pinta dezenas de raios × 3 sementes;
  os 13 raios > 300 de 64 são **93–98 %** do custo; no portão, o raio 300 sozinho é 83–93 % — custo ∝ r²–r³).
- **Agentes (desde 28/10):** janelas principais 54,8 h activas, **90 % à espera de ajudantes**; 179 subagentes,
  92,3 h, mediana 14 min; **29 (16 %) passaram do prazo de 45 min e somam 61,5 h (67 %)** — o pior 7,2 h, 6,5 h num
  laço de espera; por dentro 26 % laços de espera, 33 % gerar, ~19 % testes. Fecho da linha PainterWatercolor
  ≥ 15,5 h (~45 commits; ~10 só para partir ficheiros acima de 700 linhas; regerou Blender/Smear 2× porque a régua
  mudou a meio). O tecto de 700 linhas e o fmt só correm na suíte ⇒ o excesso aparece no fim. 1,15 chamadas por
  passo (alvo 1,5); `test`:`check` 2,4× (alvo ≤ 1); janelas coordenadoras a 448–600 mil tokens por passo.
- **Compilação:** na janela principal pesa pouco (build 2,1 %, check 0,8 %, clippy 0,5 % de 251,8 h/semana);
  os comandos `cargo test` somam **71 h/semana** (53,6 h nos subagentes) e **não se sabe quanto é compilar e quanto
  é correr** (a auditoria de 10/09 mediu ~80 % compilar). 442 crates (eram 331), profundidade 16; mexer em
  `ph2d-nodegraph` recompila 246. **`ph2d-app-motion` tem 193 k linhas (a shell 150 k) e depende de 292 crates**;
  a catraca `the_shell_only_shrinks` não o vê. 68 esperas pelo cadeado da pasta de build numa semana.

## As propostas aprovadas

| # | O quê | Ganho | Custo | Risco / nota |
|---|---|---|---|---|
| A | **Integração em lote:** rebase em cadeia das linhas num `integ/<rodada>` (check + censos a cada linha, ~2 min), UMA suíte completa no fim, curar tudo num bloco, depois B2, e `--ff-only` no main. Já previsto na DIRETRIZ §1.5.3 (13/09), nunca usado; «fusão de ensaio» ficou «não medida» (anatomia 17/09 §5.4) | −3 a −3,5 h por rodada de 5 linhas (estim.) | ½ dia | atribuir cada vermelho à linha certa |
| B | **B1:** o `ship.sh` salta a suíte quando o `HEAD^{tree}` é igual ao da última suíte verde registada (o integrador regista-a). **B2:** depois de curar, re-correr SÓ as falhas + `nextest-impacted` desde o commit vermelho + os censos/arch-gates (~20 s), não a 2.ª suíte inteira | B1 −25–35 min por envio; B2 ~−2 h por rodada | pouco | B1 risco zero (mesma árvore); B2: portões que varrem a árvore TÊM de entrar |
| D | **Recontagens automáticas** além do `PROJECT_SCHEMA` (`scripts/schema-recount.py`, que já precisou de conserto `049da68afb`): nº de ADR, registos de componentes ecs/render/script, cenas de smoke | menos retrabalho e erro na integração | pouco | — |
| E | **Tirar a suíte completa do `/pd-linha-fechar`** (contradiz a ordem do dono de 23/10: suíte só antes da integração); alinhar `.claude/commands/pd-linha-fechar.md`, DIRETRIZ §1.5.9 e o que mais a mandar | −25–35 min a −4 h por linha | 1 h | — |
| F | **Verificações baratas no commit** (hook local de pre-commit ou `scripts/` chamado pelo commit das linhas): tecto de 700 linhas por ficheiro e de funções/painéis, `cargo fmt --check` nos ficheiros tocados, zero hex/f32 literal de UI/string fixa (HR-15), filtros do nextest. Fora das recusas (clippy a meio e suíte na linha foram recusados) | −3–4 h por fecho (estim.) | ½ dia | MEDIR quanto demora por commit (alvo < 5 s); nada de compilar no hook |
| G | **Dois níveis de teste:** os de ≥ 60 s (lista MEDIDA, não à mão) saem da suíte da LINHA (`nextest-impacted`) e correm na integração e no ship | `nextest-impacted` 1 544 → ~600 s em linhas fora do Painter (estim.) | ½ dia | um vermelho lento só aparece na integração; a lista regenera-se pela medição |
| H | **Régua da largura mais barata:** medir numa janela de comprimento fixo (o custo deixa de crescer com r²) e/ou raios > 300 só no gerador e no nível lento | portão 600 → 60–100 s; regerar 18–65 → 3–10 min por preset (estim.) | ½ dia | perder um defeito só de raio grande (o Blender Soft que caía a 0 a r > 723) — guardar 1–2 raios grandes |
| I | **Prazo de 45 min IMPOSTO** por um vigia (não só escrito): o subagente recebe o aviso aos 40 min e o comando pesado é recusado/cortado no prazo; o pesado de verdade (regerar presets, bancada inteira) vai a uma fila única por onda, com a régua congelada antes (§METODOLOGIA 4 do `docs/Painter/Pinceis/REGRAS_DE_CUSTO.md`) | até ~40 h de agente/dia (61,5 → ~21,8 h, estim.) | ½ dia | medir um dia contra hoje antes de impor |
| K | **`cargo-test-narrow.sh` (e o `nextest-impacted`) imprimem compilar × correr × load** (e gravam num log agregável) | decide as próximas alavancas | 1–2 h | nenhum — FAZER PRIMEIRO |
| L | **Subir o Rust para 1.99** (pendente desde 10/09; −2,4 % instruções, −4,5 % ciclos pela auditoria §5) | 3–5 % em toda compilação | horas | `bash scripts/stack-audit.sh --tetos` primeiro (STACK_VERSOES) |
| M | **Partir `ph2d-app-motion`** (193 k linhas) em crates por família e estender a catraca «só encolhe» a toda `ph2d-app-*` | ~−50 % na unidade dele (estim.; precedente medido: corte da shell, gate 93 → 60 s) | 1–3 dias | médio: gates partem calados quando o código muda de crate (HOWTO §2) |

## ⛔ Medido e recusado — não reabrir

Subir o tecto de CPU de 50 % por linha (custa ~3 %, `project_agent_time_split_2026_10_17`) · Cranelift ·
`cargo-hakari`/workspace-hack (sem cascata) · `build-override` (145 → 146 s) · `-j` baixo · trocar o mold ·
partilhar `CARGO_TARGET_DIR` entre worktrees (troca os `.rlib`) · `-Zshare-generics` · sccache no CI (HTTP 400,
run 25840452466) · clippy a meio da linha · suíte completa na linha · baixar o `opt-level` do `ci-test`
(«baixar é silenciar o portão»). Recusado pelo DONO em 30/10: C e J.

## Implementado (janela «Velocidade 2», 29/10)

> Cada item: o que mudou · SHA · antes → depois MEDIDO · gate e mutação. As provas de processo vivem em
> `scripts/processo.prova.d/<item>.sh` e correm juntas em `bash scripts/processo.prova.sh` (< 5 s, sem
> compilar), que o `ship.sh` corre.

### K — compilar × correr × load por corrida (`39c4570912`)

- `cargo-test-narrow.sh` e `nextest-impacted.sh` (e o `suite-completa.sh` do B1) gravam cada corrida em
  `$(git rev-parse --git-common-dir)/ph2d-tempos-teste.tsv`: check, compilar (soma dos `Finished … in` do
  cargo), correr (o resto do relógio), `loadavg` no início e no fim, exit. Agregar:
  `python3 scripts/tempos-teste.py [--desde AAAA-MM-DD]` (separa as corridas com load > 5).
- **Medido, 5 corridas reais por nome, load 0,7–3,5:** compilar **106 s** (check 46 s + build de teste 60 s),
  correr **3 s** ⇒ **97 % compilar**. Detalhe: num crate recém-mexido o `check` da porta e o build de teste
  NÃO partilham artefactos (`ph2d-timeline`: check 24,4 s + build 25,1 s; `ph2d-ecs`: 18,1 + 30,6 s).
- **Histórico (transcrições 22–29/10, 7 322 corridas, 74 h):** só 4 % das saídas trazem o `Finished` (a
  porta filtra-o) ⇒ a divisão de 22–29/10 não se reconstrói: piso 12 % compilar (809 corridas = 8,0 h
  nunca passaram da compilação), central estimada ~35–40 %, confiança baixa. O log novo responde daqui
  para a frente.
- **Decide L e M:** nas corridas por nome — que são o grosso das 71 h — o relógio é quase todo compilar ⇒
  L (toolchain) e M (partir a maior unidade de compilação) atacam a parte que pesa.
- Prova `K.sh` (5 linhas); mutação «os minutos do `1m 05s` ignorados» vista vermelha (17,5 em vez de 77,5).

### B1 — o `ship.sh` salta a suíte com a MESMA árvore (`b92bccfe62`)

- `scripts/suite-completa.sh` = filtro-casa do `nextest.toml` + `nextest run --workspace` (ci-test) + registo
  da CHAVE verde em `$(git-common-dir)/ph2d-suites-verdes.tsv`. Chave = árvore inteira (commitada + por
  commitar + por rastrear, índice temporário) + `rustc -vV` + argumentos + `PH2D_*`/`CARGO_*` (fora: os botões
  de recurso). A árvore é relida no fim: mudou durante a corrida ⇒ não regista. O `ship.sh` usa-o.
- **Antes → depois:** o envio corria outra suíte sobre a árvore que o integrador acabara de passar
  (+25–35 min); com a mesma árvore registada, **0,07 s** (o cálculo da chave).
- Prova `B1.sh` (8 linhas: mesmo conteúdo com outro SHA salta; um byte por comitar, uma `PH2D_*` nova ou
  outra árvore correm; `PH2D_PORTAO_FORCAR` fora da chave). Mutação «chave sem a árvore» vista vermelha. A
  prova apanhou um defeito real da 1.ª versão (o caminho do registo dentro da chave ⇒ nunca saltava).

### E — a suíte (e o impactado) saem do fecho da LINHA (`a2dd377341`)

- `/pd-linha-fechar`, `/pd-onda-fechar`, `/pd-gate-fechamento`, agente `verificador`, MODELO_ABERTURA_LINHA
  item E, DIRETRIZ §6.1/§6.2/§6.6.A: na linha, até no fecho, testes do item e portões tocados POR NOME,
  mutação, fmt/clippy das crates tocadas 1×, auditoria; `nextest-impacted` e `suite-completa.sh` são do
  integrador (ordem do dono 23/10 e 25/10, que o `/pd-linha-fechar` ainda contradizia).
- **Antes → depois:** o fecho corria o `nextest-impacted` (1 544 s no fecho da PainterWatercolor) e às vezes a
  suíte de novo com `motivo:`; depois, 0 s de suíte no fecho.
- Prova `E.sh`: toda linha desses protocolos que nomeia o impactado/a suíte diz que é do integrador ou
  proíbe-a. Mutação «o `pd-gate-fechamento.md` antigo» vista vermelha.

### D — recontagens automáticas (`b877f0d7d1`)

- `python3 scripts/recontar.py [--proximo]`: ADR duplicado (+ `adr-index --check`), os `reg.register*` do
  `register_ecs_components` contra o `assert` da `ph2d-ecs` e os `+N` de render/script, o `max_level` de cada
  `SmokeRouter` das `ph2d-app-*` contra o CLAUDE.md §5.1, e o `PROJECT_SCHEMA` pelo `schema-recount.py`
  (importado). 1,5 s, sem compilar. No `ship.sh` (reprova), no `/pd-integracao` passo 4 e no agente integrador.
- **Achado na 1.ª corrida:** o CLAUDE.md dizia `PH2D_CODE_SMOKE=1..9`; o roteador tem `CENAS = 10` — corrigido.
- Prova `D.sh` (fixturas: ADR duplicado, registo +1, cena 31 sobre 30 = vermelho); mutação «detecção de
  duplicado desligada» vista vermelha.

### G — dois níveis de teste (`8e47af360e`)

- `scripts/testes-lentos.py` gera `.config/testes-lentos.tsv` do `junit.xml` do nextest (máximo por teste;
  < 60 s sai; timeout conta); `--filtro` = a expressão do nível lento; `--confere` reprova um teste ≥ 60 s sem
  nível (linha do `ship.sh`). `nextest-impacted`: `PH2D_NIVEL=rapido` (omissão) subtrai o nível lento, as 5
  redes obrigatórias nunca saem; `PH2D_NIVEL=tudo` não corta. `suite-completa.sh` regera a lista a cada verde.
- Lista real (junit da integração 29/10): **55 testes, 9 335 s de 16 616 s** somados.
- **Antes → depois** (segundos de teste somados no junit, diffs reais): tool-painter+wet-paint 16 215 → 6 880
  (−58 %); pixel/kanban 14 887 → 5 803; ecs+timeline 16 276 → 6 941; app-constraints 192 → 192.
- Prova `G.sh` (13); mutação «limiar 60 → 6000» vista vermelha.

### F — verificações baratas no commit (`6a9583fef1`, fix `1e1a487362`)

- `scripts/commit-barato.sh` + hook `.claude/hooks/commit-barato.sh` (PreToolUse): tectos de ficheiro
  (workspace/painel/widget/shell, congelados só encolhem), de função, `rustfmt --check`, HR-15 (cor, número,
  texto fixo), termos novos do `nextest.toml` — os números lidos dos `.rs` dos gates (o gate continua a lei;
  forma mudada ⇒ «espelho partido»). `PH2D_COMMIT_BARATO=0` desliga.
- **Medido:** mediana **48–67 ms**, máximo 103 ms por commit (alvo < 5 s); zero ✗ sobre 4 894 `.rs` reais.
- Prova `F.sh` (11: 701 linhas recusado, 700 aceite, hex recusado, o hook nega, traceback do espelho passa);
  mutação «tecto lido → 7000» vista vermelha. O 1.º dia achou um falso positivo (heredoc com `git -C "$v" …
  commit` ⇒ traceback ⇒ negava) — curado em `1e1a487362`.

### B1 (cont.) — o envio olha para o que ENVIA (`1e1a487362`)

- O primário vive com `project-memory/` por comitar ⇒ a chave da pasta nunca casava a árvore do integrador
  (achado no ensaio do A). `suite-completa.sh --enviar` (o `ship.sh`) aceita também a chave do `HEAD^{tree}`.
  Prova `B1b.sh`; mutação «`--enviar` desligado» vista vermelha.

### I — prazo de 45 min IMPOSTO + fila do pesado (`a0fcfc1f19`)

- `.claude/hooks/prazo-do-agente.sh` (SubagentStart · PostToolUse · PreToolUse, por `agent_id`; a janela
  principal fica de fora): aviso a cada chamada depois dos 40 min; comando pesado cujo prazo passa do que falta
  é recusado com o `PH2D_PRAZO=<resto>` a pôr à frente (o `ph2d-run.sh` corta-o aí); depois dos 45 todo pesado
  é recusado; `PH2D_REGERA_*`/bancada nunca correm num subagente ⇒ `scripts/fila-pesada.sh pôr` (fila POR
  LINHA — a global, J, foi recusada), que o coordenador corre 1× só com a régua congelada (sha256).
- **Visto a funcionar no mesmo dia:** os agentes de H e A pararam e devolveram «onde cheguei» no prazo.
- **Por medir:** um dia inteiro contra 28/10 (16 % estouravam, 67 % das horas) — `agent-time-split.py` amanhã.
- Prova `I.sh` (15); mutação «aviso desligado» vista vermelha.

### A + B2 — integração em lote (`958c906677`)

- `scripts/integrar-rodada.sh [-C wt] cadeia|suite|falhas [--bisecar]|b2|ff [--aplicar]`; `/pd-integracao` 3c e
  DIRETRIZ §1.5.3 reescritos. Atribuição de um vermelho: a linha cujo diff (tip anterior → tip) alcança o
  binário dele (`impacto-preciso`, `nextest list` sem compilar); vários candidatos ⇒ `--bisecar` (≤ k−1
  corridas só desses testes). B2 regista a árvore curada («B2 desde …») só com falhas + impactados
  (`PH2D_NIVEL=tudo`) + censos que provam ter corrido + `architecture_*` + recontar verdes.
- **Ensaio a seco** (worktree `integ-ensaio`, base `c3662cd8e4`, GameEngineFuncional + BlenderConstraints
  reconstruídas, load ~15): GEF check 76 s frio + censos 123 s; BC check 48 s + censos 93 s **vermelho** —
  `const assert PROJECT_SCHEMA == 198`, o mesmo que o integrador curou à mão em `4c8f7c6049`, atribuído à
  linha certa em ~2 min em vez de numa suíte de 25 min. Nota: o `check --all-targets` não avalia um `const {
  assert! }` dentro de fn; só o build de teste dos censos o apanha.
- **Estimativa da rodada de 5 linhas:** ~9 h → cadeia ~5 × 3 min + 1 suíte ~25 min + curas + B2 ~5 min.
  **Por medir:** a `suite`/`falhas`/`b2` reais do ensaio (comandos no handoff).
- Prova `A.sh` (14); mutação «atribui sempre à última linha» vista vermelha.

### H — régua da largura mais barata: janela fixa RECUSADA por medição (sem commit)

- Tentativa: medir numa janela de comprimento fixo de 200 px (patch em
  `docs/DevOps/patches/H_janela_fixa_200px_recusada_2026-10-29.patch`). Portão 873,9 s (antigo, PASS) → 310,1 s
  (janela, **FAIL**), load 4–32 (relógio sem valor de veredito). **20 raios fora da promessa, todos r ≥ 55**:
  misturadores a r = 300 leem 366–520 px para 600; Knife 98/110 a r = 55; Dabber 504/600; no raio 990,6 os
  misturadores leem ~metade (Blender Soft 717 × 1 398). Causa: o traço tem de ser mais longo que o diâmetro
  para o misturador e a água chegarem ao regime — a janela tem de crescer com o raio, o que tira o ganho.
- ⛔ **Medido e recusado:** janela de comprimento fixo menor que o diâmetro para a régua da largura.
- O que já tirou o custo da linha: o portão (599–874 s) está no nível lento do G ⇒ só corre na suíte.
- **Aberto:** (a) raios > 300 só no gerador e no nível lento (o r = 300 é 83–93 % do portão); (b) janela de
  2–3 diâmetros só nos misturadores/espátulas; ambos com a prova do Blender Soft (r > 723) vermelha.

### M — parte 1: catraca de toda `ph2d-app-*` + o plano (`99f00989c3`)

- `the_shell_only_shrinks_and_every_app_crate_too`: tecto por crate (19; medido + 2 %, piso 100), censo de
  obsolescência a 5× a folga, crate sem linha reprova. Motion 193 644 (tecto 197 516). 0,05 s. Mutações
  (tecto abaixo, linha apagada, tecto obsoleto) vistas vermelhas.
- `docs/DevOps/PLANO_PARTIR_APP_MOTION.md`: o núcleo tem CICLOS (`motion_state` ↔ `motion_bridge`, ↔ shape/
  objeto/macro_track/partículas/mundo) ⇒ a onda 0 corta o núcleo antes de mover; ondas 1–4 e critério verde.
- **Aberto:** as ondas 0–4 (1–3 dias, janelas seguintes); a suíte completa 1× no fim.
- ⚠️ Consequência a ter presente: uma família que cresça > 2 % reprova — a cura é uma crate nova
  (drop-crate, ADR-0075), nunca subir o número.

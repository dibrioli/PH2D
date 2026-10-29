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

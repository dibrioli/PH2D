# HANDOFF DE INTEGRAÇÃO — `line/Macros-Automação`, W6 (tutorial «FAZER») → `main` (2026-10-15)

> A linha fecha e PARA: integrar e shipar só por ordem do Enio, pelo agente integrador (DIRETRIZ §1.5.3).
> Mecanismo de cada wave, com as tabelas MEDIDAS (não copiadas aqui): [plano W6](../03_plano_W6.md) — §5 W6a ·
> §5.1 nós em fila · §6 W6b · §6.1 a fracção · §6.3 a régua da gravada (W6c) · §6.4 a base e o ensino fora do quadro (W6d).
> Antes: [handoff de integração W5](HANDOFF_INTEGRACAO_line_Macros_2026-10-15_W5.md) (a W5 NÃO está no `main`: este
> handoff cobre a linha inteira) · [continuação W6](HANDOFF_CONTINUACAO_W6_FAZER_2026-10-15.md).

## §1 — Base e HEAD

- Branch `line/Macros-Automação`, worktree `Worktrees/line-Macros-Automação/`. Merge-base com o `main`: `0aab2c1cb6`
  (o `main` NÃO andou; 0 atrás). HEAD de código `991dcd4649` (os commits de documentação deste handoff vêm por cima).
- `git diff --shortstat 0aab2c1cb6 HEAD`: 312 ficheiros, +21 471 / −749 (a linha inteira, W4+W5+W6). Só a W6
  (`b8571e3636..HEAD`, 23 commits): 89 ficheiros, +6 974 / −283.
- Linha única ⇒ ordem trivial; o conflito possível é com `line/PainterWatercolor` (§4).

## §2 — O que a W6 entrega (sobre a W5)

- **W6a — Motion «fazer»** (`6ec665ddf4` `ceffb4d5c8`): o modo `Do it` (`ModoDaMacro::Fazer`), critérios das etapas
  derivados da gravação, balão com anel nas portas, «Mostra-me», «Recomeçar», texto do autor (`EdicaoDeEtapa.dica`,
  etiqueta 8), cena `PH2D_MACRO_SMOKE=3`.
- **O grafo arruma-se ao abrir** (`bd161797a3`: só o objecto arrumado à mão e gravado fica como está) e **o balão traz à
  vista os nós da etapa** (`d1c5954303` `a3826bcdd7` `1d927891f9`; `request_frame_nodes`, piso legível 0,35).
- **Nós soltos em fila** (`6e4095966b`): decisão do dono 15/10, opção (a) — lei 4 de `ph2d-nodegraph/src/layout.rs`
  (`loose_row`): os cartões soltos partilham UMA banda; Drag, Box e Wind lado a lado.
- **W6b — Painter «fazer»** (`baf7fa3a17` `c545d65eb2`): região = caminho gravado + raio; só a parte NOVA conta;
  «pintado» = alguma camada difere da do estado inicial; área marcada TRANSLÚCIDA (o `AccentSoft` é opaco: a foto
  lia-o como tinta); «Mostra-me» pelas portas da ferramenta; «Recomeçar» = 1 passo de undo; cena `=4`.
- **W6c — a trilha do Painter grava os quatro meios** (`a7387f528d`, `677ac41fbf`): sai a recusa de meio
  (`MACRO_RECUSA_MEIO`, `panel.macros.refusal.wet_paint_only`); o estado inicial leva cada slot de pincel inteiro, o
  meio, o modo e o estado de tela; o meio é POR PINCELADA (`7d9c35a474`). Censo de controlos do painel de pincel:
  314 → **0** reprovados de 985. A régua passa a ser a fracção SOBRE a da gravada (`referencia`).
- **W6d** (`5e5dcd095f`): lei normalizada pela gravação COM a base de água (`Referencia{gravada, base}`),
  `FRACAO_DA_GRAVADA_QUE_CUMPRE = 0.703` (margem **2,8 pontos**), `DELTA_MINIMO 0.005`; o ensino e as medidas num fio
  (`macro-referencias`) fora do quadro; balão «a preparar» (`APreparar`). Detalhe e tabelas: plano §6.3–§6.4.

## §3 — Foundational / partilhado tocado, e porquê é ADITIVO

| Crate | O quê | Porquê não parte ninguém |
|---|---|---|
| `ph2d-nodegraph` | `layout.rs` lei 4 (`loose_row`) + 3 testes | só muda a POSIÇÃO do arrumar; `NodeOp`/`OpResolver`/`NodeManifest` intactos; o espaçamento contra a sobreposição (dono 20/09) igual |
| `ph2d-editor-core` | `macro_tutorial.rs` NOVO: `PedidoDoTutorial::{AbreParaPintar, Meio{chave_do_meio}, Pinta, APreparar}`, `AreaNaImagem`, `BalaoDoTutorial.area`; `MacroRequest::{MostraMe, Recomeca}`; `MACRO_MODOS` 3; sai `MACRO_RECUSA_MEIO` (W5, só a shell/família o liam) | variantes novas num enum SÓ desta linha; chrome `tutorial_balao` (+ `tests.rs` numa pasta) e `chrome/mod.rs` (gerado pelo `ph2d-chrome-sync`) |
| `ph2d-i18n` | + `panel.inspector.macro.mode.2`, `…mode_hint.2`, `panel.macros.hint`, `panel.macros.error.other_size`, `panel.macros.tutorial.{step,before,done_title,done,edit,add,connect,disconnect,value,delete,open_paint,medium,paint,preparing,show_me,restart}` · − `panel.macros.refusal.wet_paint_only` | chaves novas; a única removida só tinha leitor na recusa que saiu |
| `ph2d-panel-motion-graph` | `enquadra_nos.rs` NOVO (o `paint_enquadra_nos` passou a `enquadra_nos`), `request_frame_nodes`, `hits::alvo` público | API nova + renomeação interna; testes `fit` migrados |
| `ph2d-macro` | `EdicaoDeEtapa.dica` (etiqueta 8, SEM `exige`: não muda o que toca) | um leitor 1.1 antigo salta-a |
| `ph2d-motion-doc` | `import_owner`; `copy_owner` partilha `Atributos`; `arrumados_a_mao` (linhas `oh <id>` só quando há) | ficheiro sem elas é igual ao byte |
| `ph2d-ecs` | `ModoDaMacro::Fazer` | variante nova |
| `ph2d-tool-painter` / `ph2d-app-painter` / `ph2d-app-motion` / `ph2d-app-macros` / `ph2d-panel-macros` / `ph2d-panel-inspector` / shell | tutorial, estado inicial rico (W6c), `fase_macros`, `macro_smoke` `=3` `=4` | — |
| `.config/nextest.toml` | override de 6 × 60 s para `macro_track::tutorial::tests::pular_a_pincelada_e_esperar_nao_cumpre_nenhuma` e `…a_mao_torta_cumpre_o_pincel_fino_e_o_nada_nao` (24,0 s / 10,4 s sozinhos a load 16; MORTO aos 180 s / 106 s sob a suíte a load ~200) | só alarga tecto de dois testes |
| `.typos.toml` | palavra `instantaneos` (nome do módulo `macro_track::instantaneos`) | — |

**Formato gravado.** O JSON do estado inicial do Painter ganhou as chaves `meio`, `modo`, `pinceis`, `tela`… (sete).
Uma gravação de ANTES da W6c lê-se como Wet Paint e toca a mesma tela — gate
`ph2d-tool-painter --lib macro_track::tests::um_estado_inicial_de_antes_da_w6c_reproduz_o_mesmo_splash`.

**Contratos congelados (§6) — NÃO tocados.** Prova (corrida em `991dcd4649`):

```
git diff --name-only 0aab2c1cb6 HEAD | grep -E 'ph2d-editor-core/src/tool|ph2d-vec|ph2d-vector|ph2d-nodegraph/src/(op|manifest|cook)|schema'   # vazio
git diff 0aab2c1cb6 HEAD -U0 | grep -E '^[+-].*(PROJECT_SCHEMA|MAX_SPIRAL_TURNS|MAX_POLYGON_SIDES|MAX_VERTICES_PER_LLM_GEN)'   # só prosa do handoff W5 («PROJECT_SCHEMA: não muda»)
git diff --name-only 0aab2c1cb6 HEAD | grep ph2d-nodegraph    # só layout.rs e layout_tests.rs
```

Os `NodeOp`/`Tool`/`PanelEvent` que o diff cita são `use` e chamadas em TESTES. Nenhum gate `*_contract_surface`
muda; `PROJECT_SCHEMA` não muda.

## §4 — O que um merge pode partir

- `ph2d-tool-painter` `tool/runtime.rs` e `wetpaint*` com `line/PainterWatercolor` (superfície herdada da W5, §4 do
  handoff W5) — rode `collision-surface.sh` pelo caminho absoluto em cada worktree.
- O estado inicial do Painter: um campo novo de pincel/slot noutra linha só entra no estado inicial se o censo o
  alcançar (`exploration` do painel de pincel; 0 reprovados de 985 agora) — recontar após o merge.
- `ph2d-i18n`: ficheiros de chaves somam entre linhas; uma linha que reintroduza `wet_paint_only` reprova o censo.
- `chrome/mod.rs` é GERADO: regenerar com `ph2d-chrome-sync`, nunca resolver à mão.
- Tectos de LOC somam: `tutorial_base.rs` nasceu para cortar `tutorial.rs` (705 → 579).
- nextest: o override é uma só linha `filter` — dois acrescentos de linhas diferentes conflitam no mesmo texto.

## §5 — Contadores (DELTA sobre o merge-base)

Crates novas: 2 (W5d, `ph2d-app-macros`, `ph2d-panel-macros`); **W6: 0**. Chaves i18n W6: +19 / −1. Famílias no registo
`FAMILY`: +1 (W5). Schema, ADR, contratos: 0. Cenas de smoke das Macros: `=1..4` (a próxima é `=5`).
Recontar contra o `main` do dia — o número é o do código.

## §6 — Prova de fecho (15/10)

- `BASE=main PH2D_SUITE=fecho bash scripts/nextest-impacted.sh`: **22 114 passed, 1 failed, 1 timeout de 22 116**,
  7 134 skipped. O vermelho: tecto de LOC (`workspace_src_files_under_loc_cap`) — curado por corte (`46ea84135f`,
  `tutorial_base.rs`). O timeout: o tutorial pesado do Painter a load ~200 — override de nextest com o medido sozinho
  (24 s). ⚠️ Os commits de cura DEPOIS da suíte foram re-corridos **POR NOME**, não numa suíte nova.
- clippy `--all-targets -D warnings` nas 24 crates tocadas: ✓ depois de 5 lints (`89aeb3bedc`, `991dcd4649`: 3 tipos
  complexos, um assert de bool, um literal que era 1,2, `type VistaCumprida`). `cargo fmt` ✓ · typos ✓ · índices de
  ADR/docs ✓.
- **Mutação:** 1.ª prova da W6 **20/20** depois de fechar 3 sobreviventes (`f788876c15`: vaga por tipo+ordem, o Painter
  só na imagem do tutorial, a pilha do Composite no estado inicial). 2.ª prova das leis novas (W6c/W6d) **27/27**
  depois de 3 sobreviventes (`7bd6ddb13f`: a base no `por_cumprir`, a versão velha que se larga, a outra tela por dentro).
- **Auditoria (2 lentes):** meio por pincelada (já não único da trilha), pincelada de Δ vazio sem critério, outra
  tela diz o tamanho, estado igual não se reavalia, «Recomeçar» a gravar grava recusa (`7d9c35a474`); vermelhos do
  fecho: balão pela porta do tema e do ritmo, área marcada dormente (`8f5d6408ca`).

## §7 — Smoke do dono (PENDENTE) e fotos

Binário `--profile smoke` na worktree, 2.ª compilação:

```
    Finished `smoke` profile [optimized] target(s) in 0.23s
```

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Macros-Automação && PH2D_MACRO_SMOKE=3 ./target/smoke/ph2d-host-desktop`
   — Motion: Drag, Box e Wind numa fila, balão «Step 1 of 4» com anel na 1.ª porta. Fazer as 3 ligações e o valor;
   cada ligação certa avança o balão; «Show me» faz o passo; «Restart» volta ao início; Ctrl+Z desfaz.
2. `… PH2D_MACRO_SMOKE=4 ./target/smoke/ph2d-host-desktop` — Painter: tela branca com faixa translúcida marcada; pintar
   por cima enche-a e o balão avança. Errado: pintar longe ou fino não avança; a faixa opaca como tinta seria o defeito
   `c545d65eb2`.

Fotos (`fotografa_cena.sh`): `=3` tirada. `=4`: a 1.ª foto mostrou a área opaca (curado em `c545d65eb2`); a foto do
binário FINAL (`991dcd4649`, 1930×1040, smoke) mostra a faixa translúcida do acento com o caminho tracejado, o balão
«Step 1 of 4» com o texto do autor, «Restart» e «Show me»; 59 fps.

## §8 — ABERTO

1. A margem de **2,8 pontos** da `0.703`: a água continua a espalhar depois do instante em que a base é amostrada;
   «pular e esperar» chega a 67,5 % (k = 22 das 30 pinceladas Wet Paint) contra 73,1 % mínimo da torta ½ r em Digital.
   Cura proposta: amostrar base e referência no MESMO instante já assentado (plano §6.4).
2. `ph2d-tool-painter --test it macro_agua` depende do relógio: vermelho sozinho a load ~4 no código da W5 E no HEAD
   (provado trocando as crates da W5 de volta); verde dentro do nextest sob carga. NÃO é regressão desta linha —
   família inversa de [FLAKES_DE_CARGA](../../DevOps/FLAKES_DE_CARGA.md), pede olhar próprio.
3. A medição inteira custa 2–3× uma reprodução (22,6 s para 30 pinceladas a 1024², progressiva, fora do quadro).
4. Do dono, sem tratar: depois do Parar, pintar SEM gravar na mesma imagem fica escondido enquanto o clipe toca.
5. O estado das ferramentas do trilho (fonte do Clone, Máscara/Selecção) não viaja no estado inicial (o censo só
   explora o painel de pincel).
6. Smoke do dono de `=3` e `=4` por fazer. A dica do Motion ainda descodifica a trilha por quadro (plano §6.4).

## Retomar

`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Macros-Automação && pwd && git branch --show-current`.

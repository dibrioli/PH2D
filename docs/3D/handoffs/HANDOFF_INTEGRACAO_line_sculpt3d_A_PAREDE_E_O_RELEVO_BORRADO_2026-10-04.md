# HANDOFF de INTEGRAÇÃO — `line/sculpt3d` · a PAREDE DO IMPASTO como no 2D e o RELEVO BORRADO pelos efeitos de vizinhança (2026-10-04)

> **Para quem é:** o agente INTEGRADOR (só por ordem do dono — CLAUDE.md §0.7). Superfície de colisão,
> contadores como DELTA, o que um merge pode partir, a prova de fecho e o que fica aberto. O mecanismo
> vive no [doc 3D 30 §19 e §20](../30_plano_camadas_e_efeitos_na_peca.md).
>
> ⚠️ **SUPERSEDE** o [handoff O_RELEVO_POR_CAMADA (04/10)](HANDOFF_INTEGRACAO_line_sculpt3d_O_RELEVO_POR_CAMADA_2026-10-04.md)
> como documento de integração desta linha. Ele **ainda NÃO está no `main`**: os commits dele (e dos que
> ele supersede — OS_EFEITOS_DE_VIZINHANCA, CAMADAS_EM_ECRA, A_INCLINACAO) vêm nesta mesma série. Tudo o
> que eles declaram continua válido e **soma-se** ao que está abaixo, que é só o que esta janela acrescenta.

> ⛔ **O DONO PAROU O DESENVOLVIMENTO DOS MÓDULOS 3D (Sculpt) a 04/10**, a meio do fecho desta janela, e
> pediu este handoff para integrar o que existe. Nada do que vem abaixo foi smoke-testado pelo dono; o
> que ficou por fechar está nomeado no §5 e no §10 (⚠️ um SIGSEGV intermitente na suíte da peça).

## §0. Estado em uma tabela

| | |
|---|---|
| ramo · worktree | `line/sculpt3d` · `Worktrees/line-sculpt3d` |
| merge-base | `1ad60a1ce` (= `main` a 04/10; rebase desnecessário à data) |
| commits | **87** por integrar, o deste handoff incluído (`git log --oneline 1ad60a1ce..HEAD`); esta janela são os **12** de `6c62a8a70` até ele · a linha inteira `275` ficheiros `+26 596 −2 483` · esta janela `49` ficheiros `+2 988 −177` |
| smoke do dono | **não feito** (o 3D foi parado): (A) a pincelada grossa sem a orla, (B) a cena `=56` — os passos no §7 para quem retomar |
| contratos §6 | **intocados** (`Tool=12`, `PanelEvent=4`, `NodeOp`) |
| `shells/desktop/src` | **0** linhas nesta janela |

### Contadores — DELTA desta janela (a fonte é o código; reconte contra o `main` do dia)

| contador | delta | valor aqui |
|---|---|---|
| `PROJECT_SCHEMA` · `SCULPT_DOC_VERSION` | **0** | `7` (o relevo composto é derivado, não se grava) |
| cenas do roteador sculpt3d (`scenes::CENAS`) | **+1** (55→56) | `=56` o relevo borrado — conta-se no roteador desta família |
| ADR | **0** | a lei está no doc 30 §19–§20 |
| `Cargo.lock` | **0** | |
| chaves i18n | **+1** | `panel.painter_layers.adjust.tone_not_relief` |
| memória | **+1** caso na família | `reference_topic_code_gotchas.md` (o índice: «Gotchas de código (29)») |

## §1. O que esta janela traz

| bloco | commit |
|---|---|
| **A — a parede do impasto lê-se como no 2D** (o oráculo: o Painter 2D a pintar a mesma pincelada): a cor pintada entra DESCODIFICADA (`cor_em_luz`; era código sRGB a multiplicar a luz, `230,30,30` saía `213,81,79`); o corpo pesa a LUZ e não a inclinação (`fs_main_tinta` mistura o liso e o relevo inteiro pelo corpo); o ambiente (o céu do PBR, o piso com direcção) lê a peça sem o relevo | `6c62a8a70` · `79d394ea1` |
| **B — o Gaussiano e a Nitidez borram o relevo por baixo deles**, no 2D e na peça, pela porta única `fold_relief_through`; na peça o calor corre NA PLACA (`surface_heat_field`) | `8132f126c` |
| o Brilho e as Sombras/Realces dizem no painel que o relevo não muda | `8bc8361bb` |
| a cena `=56` e a sonda que a fotografa | `9cc30c66e` |
| os gates que faltavam (máscara invertida, o traço por baixo do desfoque, os leitores da CPU) e o arnês `muta_o_relevo_atraves.sh` | `acbc97e30` |
| duas linhas redundantes que a mutação provou inertes saem; doc 30 §20; memória | `da704a372` |
| R13/R24 e P2 re-ancoradas | `9a14a7745` |

## §2. Foundational tocado e a superfície de API (o que um merge pode partir)

| item | mudança | quem parte |
|---|---|---|
| `ph2d-mesh-render` `shaders/mesh.wgsl` | `cor_em_luz` NOVA; o `fs_core_n` descodifica a cor (o parâmetro chama-se `codigo`) e lê o ambiente com `na = canvas_normal(in.n_view)` (`mx_indirect`, `ambient_floor`) | **toda peça pintada desenha mais escura e saturada** (a cor certa); o 3D Modeling e a doação não alimentam cor por vértice (conferido) |
| `ph2d-mesh-render` `fonte.rs` · `shaders/tinta.wgsl` · `relevo_normal.rs` | `tinta_relevo_n(in, g)` e `tinta_inclina(n, gv)` / `relevo_normal::inclina(n, gv)` perdem o `corpo`; o `fs_main_tinta` mistura pelo corpo | quem chame `inclina` com três argumentos |
| `ph2d-render` `layer_compositor/surface.rs` | `LayerCompositor::surface_heat_field` NOVA (pública); o `run_surface_heat` passa por um `heat_core` com duas pontas; os buffers de rascunho do calor ganham `COPY_DST | COPY_SRC` | quem edite o despacho do calor |
| `ph2d-painter-effects` `Neighbourhood` | `blur2` NOVO com implementação por omissão (dois `blur1`) | — (aditivo) |
| `ph2d-tool-painter` `layers/relief_through.rs` (NOVO) | `ReliefStep` · `ReliefFilter` · `ReliefEffect` · `ReliefSamples` · `fold_relief_through` · `relief_effect` · `relief_plan_filters` · `LayerStack::relief_plan` (reexportados no `lib.rs`) | — (aditivo) |
| `ph2d-tool-painter` `tool/paint/{impasto_light.rs, relief_fields.rs}` | `ReliefLayer.id` NOVO; `ReliefFields.folded`; o `height_at`/`cover_at` leem a dobra materializada; `fold_at`/`own_at` (o corpo de sempre, dividido); `Amostras` | **a `line/PainterWatercolor` (aberta) mexe no impasto 2D**: se ela redigiu o `height_at`, a resolução é ela chamar o `fold_at` e ler o `folded` |
| `ph2d-panel-painter-layers` `paint_adjust.rs` | `paint_nota_do_relevo` no fim do `paint_adjustment_params` | quem edite o fim dessa função |
| `ph2d-app-sculpt3d` | `composto_na_placa_relevo.rs` NOVO (`VizinhancaNaPlaca`, `CompostosDaCena::dobra_o_relevo`); `AssinaturaDoRelevo = Vec<ReliefStep>`; `Dobrado` guarda «por dobrar»; `relevo_por_dobrar`/`relevo_dobrado`/`relevo_em_dia`/`dobra_o_relevo_com`/`relevo_atraves`; `com_vizinhos`; o `sync_mesh` dobra o relevo por dobrar; o `em_dia` também | — |

### Consumidores conferidos

| porta | consumidores | conferido |
|---|---|---|
| `fold_relief_through` | o 2D (`impasto_fields`), a peça (`relevo_composto`, `dobra_o_relevo_com`) | sem ajuste que aja, as duas dobram por amostra AO BIT (gate) |
| `cor_em_luz` | os quatro modos de luz (plano, matcap, barro, PBR) | o barro por pintar `1 → 1` ao bit (os 114 + 90 gates do `ph2d-mesh-render`) |
| `surface_heat_field` | a peça (o relevo) | paridade com a CPU `1,0e-5` relativo |

## §3. Para o integrador — linhas abertas

- `line/PainterWatercolor` (aberta): mexe no Painter e no impasto 2D. Colisões possíveis: `tool/paint/{relief_fields.rs, impasto_light.rs}`, `layers/mod.rs`, `lib.rs`. Não negociei; nada re-pinado nela.
- Qualquer linha que pinte cor na peça e afirme bytes de ECRÃ com cor (não com barro branco) vê a cor mudar (a descodificação); nas linhas abertas não achei nenhuma.

## §4. Gates novos

| | |
|---|---|
| A (sem placa) | `wgsl_gate_tests::{a_cor_pintada_entra_na_luz_descodificada, o_ambiente_le_a_peca_sem_o_relevo}` · `relevo_normal_tests` re-escritos sem o corpo |
| A (placa) | `parede_do_impasto_tests::a_parede_do_impasto_na_peca_le_como_no_2d` (base e camada nova, PBR · barro · matcap; pior desvio na orla `0,074`, a lei velha `0,30`) |
| B (sem placa) | `relief_through_tests` (7) · `impasto_body::blur` (2) · `o_relevo_por_dobrar_chega_aos_leitores_da_cpu` · `scenes_relevo_borrado_tests` · `nota_do_relevo_tests` |
| B (placa) | `relevo_atraves` (3): **o DoD** — o arrasto REAL do raio pelo painel muda o relevo LIDO DA PLACA, paridade com a CPU, inclinações, `Ctrl+Z` ao bit · o traço por baixo do desfoque · o Brilho não mexe |
| mutação | `muta_a_parede_do_impasto.sh` **6/6** + C1 · `muta_o_relevo_atraves.sh` **12/12** + C1 · `muta_a_normal_do_relevo.sh` **12/12** + C1 (N6/N7 re-ancoradas) · `muta_o_horizonte_do_relevo.sh` **6/6** + C0 (H1/H4 re-ancoradas) · os irmãos: §5 |

## §5. Prova de fecho (batched sobre o diff acumulado)

Gate batched corrido sobre o diff acumulado em `9a14a7745`; as reprovações curadas em `475f5d311` e
re-conferidas onde indicado.

| passo | resultado |
|---|---|
| `nextest-impacted.sh` (load `3,6`) | **19 469 / 19 471** — os 2 vermelhos eram os tectos de linhas (`architecture_panel_loc_cap`: `paint_adjust.rs` 632/600; `architecture_workspace_file_loc_cap`: `relevo_painel_no_produto_tests.rs` 729/700, **já 729 em `b77d5cefb`**). Curados (`paint_adjust_nota.rs` irmão; as sondas da orla foram para `parede_do_impasto_tests.rs`) |
| `architecture_*` + memória + `claude_md` + `docs_` | depois da cura **106 / 106** (load `1,4`) |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | ✓ |
| `clippy --workspace --all-targets -D warnings` | ✗ em `9a14a7745` (`ReliefSamples` sem `is_empty`; um laço por índice num teste) — curado e re-corrido só nas crates tocadas (`ph2d-tool-painter`, `ph2d-app-sculpt3d`, `ph2d-panel-painter-layers`, `ph2d-render`, `ph2d-mesh-render`, `ph2d-painter-effects`) ✓. ⚠️ **O clippy do workspace inteiro não foi repetido** (a 1.ª corrida parou na `ph2d-tool-painter`; as crates que dependem dela não foram lintadas) — o `ship.sh` apanha o que sobrar |
| `fmt --all --check` · `machete` · `check-standalone-optional` · `check-workflow-packages` | ✓ · ✓ · ✓ · ✓ |
| placa | `ph2d-render` 52/52 · a luz do impasto 2D 6/6 · `ph2d-flip-render composite_` 19/19 · preview do Painter 3/3 · peça `tinta_no_produto --skip diag_` **45/45** · `ph2d-mesh-render` 90/90 |
| CPU das crates tocadas | `ph2d-tool-painter` 1 461 · `ph2d-sculpt3d` 693 · `ph2d-mesh-colors` 72 · `ph2d-panel-painter-layers` 212 · `ph2d-painter-effects` 109 · `ph2d-mesh-render` 121 · `ph2d-render` 404 · `ph2d-app-sculpt3d` **411** — ⚠️ ver abaixo |
| mutação (esta janela) | parede **6/6** + C1 · relevo através **12/12** + C1 · normal **12/12** + C1 · horizonte **6/6** + C0 |
| mutação (irmãos re-corridos depois das re-âncoras) | relevo por camada **25/25** · pilha da peça **43/43** · pilha na placa **19/19** · efeitos de vizinhança **interrompido** pela paragem a ~30 de 49, zero sobreviventes até ali · camadas em ecrã e relevo na peça **não re-corridos** (pré-voo 50/50 e 17/17 ✓) |

⚠️⚠️ **SIGSEGV intermitente na suíte sem placa da `ph2d-app-sculpt3d`** (o binário de testes morre a
meio, `signal: 11`): **2 em ~8** corridas na cabeça, a load `16–37`; **0 em 14** na base `b77d5cefb`, a
load `39–89`; **0 em 3** com o `nextest` (um processo por teste: **nenhum teste sozinho cai**) e 0 em 2 com
`--test-threads=1`. ⇒ é uma interacção entre testes no MESMO processo, muito provavelmente desta janela;
**a causa não foi achada**. A bissecção por fazer: correr a cabeça sem os dois testes novos sem placa
(`scenes::relevo_borrado::tests::*` e `tinta_da_peca::pilha::tests::o_relevo_por_dobrar_chega_aos_leitores_da_cpu`,
que correm o calor da retícula na CPU com `rayon`). O CI corre os testes pelo `nextest`, onde não se
reproduziu; mesmo assim é um vermelho que o integrador tem de conhecer.

## §6. Perfil de agente (`bash scripts/agent-loop-profile.sh`)

```
  ✗ paralelismo de ferramenta              1.13/passo   alvo: >= 1,5  (9% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                233   alvo: <= 800  (uma janela nova por onda de trabalho)
  ✗ cargo test : cargo check                726 : 247   alvo: <= 1,0  razao 2.9x (baseline: 4,3x)
  ✗ edicoes pela ferramenta Edit                  33%   alvo: >= 80%  (1742 por script; baseline: 48%)
  ✗ contexto relido por passo (media)         486 mil   alvo: <= 250 mil  (set/2026: 606 mil — 82% do custo)
  ✓ contexto no inicio da sessao               63 mil   alvo: <= 80 mil  (02/10: 380 mil, CLAUDE.md a 710 KB)
```

## §7. Smoke (do dono)

**Não feito** — o dono parou o 3D antes. A cena `=56` está fotografada (`kwin --virtual` + `import
-window`: a bola com as riscas e a faixa, nítida, a raio `0`), e a sonda
`diag_a_cena_56_antes_e_depois_do_raio` mostra-a a `0`/`50`/`100 %` do curso (as riscas e a borda da faixa
amaciam juntas). Para quem retomar:

**(A) a parede como no 2D**
1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-sculpt3d && env PH2D_SCULPT3D_SMOKE=55 cargo run -p ph2d-host-desktop --profile smoke`
2. Troque `IMG` por `PNTR`, escolha o meio `Impasto`, um pincel grosso (~48) e pinte uma pincelada vermelha na bola.
3. A pincelada é vermelha até à borda, com a parede de cima clara e a de baixo escura — sem um contorno cinzento à volta.
4. Deu errado se a pincelada tem uma orla cinzenta, ou a cor sai rosada e desbotada.

**(B) o desfoque amacia o relevo** — o roteiro da cena sai no terminal ao abrir:
1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-sculpt3d && env PH2D_SCULPT3D_SMOKE=56 cargo run -p ph2d-host-desktop --profile smoke`
2. Troque `IMG` por `PNTR`, separador `Layers`: no topo a linha `Gaussian Blur`.
3. Arraste a barra do raio para a direita: a borda da faixa E as riscas e a lomba amaciam ao vivo; `Ctrl+Z` devolve-as.
4. Deu errado se só a cor amacia e as riscas ficam riscadas, ou o `Ctrl+Z` não as devolve.

Binário compilado nesta worktree no HEAD, depois de `rm -rf target/*/incremental` (`19 G` + `4,7 G`); a 1.ª
build `33,0 s`, a 2.ª:

```
$ bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke
    Finished `smoke` profile [optimized] target(s) in 0.27s
```

## §8. `CLAUDE.md` §5.1

A linha do módulo **3D / Sculpt** troca o link do «Último» para este handoff (a natureza do módulo não
mudou).

## §9. Premissas do briefing que a medição/o código derrubou

1. *«cura candidata: o peso da inclinação pelo corpo como no 2D»* → o 2D não pesa a inclinação; pesá-la
   ERA o defeito.
2. *«é a luz da peça a desenhar a parede»* → um terço do cinzento era a COR (códigos sRGB multiplicados
   como luz, em todos os modos, até no plano) e, no PBR, o resto era o AMBIENTE.
3. *«o brilho na parede»* → aparece sozinho no modo barro quando a cor entra descodificada.
4. *«o relevo por baixo de um ajuste = a dobra até ao z dele»* → dentro de um grupo, o que o grupo fez.
5. *«decida se o relevo borrado se calcula na placa e como as inclinações o seguem»* → só o passa-baixo
   vai à placa e volta (a CPU pagava `103`/`256 ms` a `32x`; a placa `33`/`62 ms`); as inclinações seguem
   pela porta de sempre.
6. *«o Bloom e Sombras/Realces borram o relevo»* (a lista do dono) → são operações de TOM (limiar e
   quantidades em luminância); não agem e o painel diz porquê (a alternativa deixada pelo briefing).
7. *«a assinatura passa a incluir os ajustes»* → a assinatura passou a SER o plano.

## §10. ABERTO (nomeado)

- ⚠️⚠️ **O SIGSEGV intermitente da suíte sem placa da peça** (§5) — causa por achar; o 1.º passo é a
  bissecção pelos dois testes novos sem placa.
- O clippy do workspace inteiro e os arneses `muta_os_efeitos_de_vizinhanca.sh` (até ao fim),
  `muta_as_camadas_em_ecra.sh` e `muta_o_relevo_na_peca.sh` não foram re-corridos depois da última onda.

- O CONTRASTE das paredes sólidas é o da luz da cena (decisão de 24/09): a parede de sombra desce a `0,65`
  na peça contra `0,35` no 2D. Dar-lhe o contraste do 2D seria pôr a luz relativa do Painter na peça —
  pergunta de produto, não feita.
- `64x` com um desfoque por cima do relevo: `369`/`870 ms` por passo do raio (a família do limite já
  nomeado da cor); `128x`/`256x` recusam os efeitos de vizinhança.
- Um traço por baixo de um desfoque paga a dobra inteira por quadro (`~33 ms` a `32x`).
- O Motion Blur do 2D (que também arrasta tinta) não age no relevo — a decisão listou os quatro de vizinhança.
- Herdado: o §10 do handoff superseded (pintar a máscara de uma camada na peça, grupos, camadas de
  textura, Lock/Ref na peça; W5, W7) e as 3 âncoras de `muta_a_silhueta_do_contacto.sh` partidas do `main`.

# HANDOFF de INTEGRAÇÃO — `line/sculpt3d` · o RELEVO POR CAMADA na peça 3D (W4) + os efeitos de vizinhança, as camadas em tons de ecrã e a pilha na peça (2026-10-04)

> **Para quem é:** o agente INTEGRADOR (só por ordem do dono — CLAUDE.md §0.7). Superfície de colisão,
> contadores como DELTA, o que um merge pode partir, a prova de fecho e o que fica aberto. O mecanismo
> da W4 vive no [doc 3D 30 §15](../30_plano_camadas_e_efeitos_na_peca.md) (o desenho, as premissas
> derrubadas, a medição).
>
> ⚠️ **SUPERSEDE** o [handoff OS_EFEITOS_DE_VIZINHANCA (03/10)](HANDOFF_INTEGRACAO_line_sculpt3d_OS_EFEITOS_DE_VIZINHANCA_2026-10-03.md)
> como documento de integração desta linha. Ele **ainda NÃO está no `main`**: os commits dele (e dos que
> ele supersede — CAMADAS_EM_ECRA, A_INCLINACAO) vêm nesta mesma série. Tudo o que eles declaram
> continua válido e **soma-se** ao que está abaixo, que é só o que a W4 acrescenta.

## §0. Estado em uma tabela

| | |
|---|---|
| ramo · worktree | `line/sculpt3d` · `Worktrees/line-sculpt3d` |
| merge-base | `1ad60a1ce` (= `main` a 04/10; rebase desnecessário à data) |
| commits | **67** + o commit deste handoff (`git log --oneline 1ad60a1ce..HEAD`) — a W4 são os últimos **8** (`767f2d9b4`…`efd72c67a`, mais os 2 de docs) · a linha inteira **254** ficheiros `+22 714 −2 391` · só a W4 **43** ficheiros `+1 903 −168` |
| smoke do dono | W4: ✅ **04/10** («smoke OK»), com 2 observações — a cor que não entrava numa camada nova (curada, `efd72c67a`, re-smoke no §7) e o desfoque que não borra o relevo (decisão do dono: borrar também — a próxima onda, §10) · W6: ✅ 04/10 · os anteriores: ver os handoffs superseded |
| contratos §6 | **intocados** (`Tool=12`, `PanelEvent=4`, `NodeOp`) |
| `shells/desktop/src` | **0** linhas (a linha inteira toca só `shells/desktop/tests/it/the_sculpt_mesh_edits_are_wired.rs`, da W1b) |

### Contadores — DELTA da W4 (a fonte é o código; reconte contra o `main` do dia)

| contador | W4 | valor aqui |
|---|---|---|
| `PROJECT_SCHEMA` e família · `SCULPT_DOC_VERSION` | **0** | `SCULPT_DOC_VERSION` continua `7`: o v7 já grava o relevo por camada (W1) e a `LayerStack` serializada já leva `impasto_depth`/`impasto_composite`/`has_relief` |
| cenas do roteador sculpt3d (`scenes::CENAS`) | **+1** (54→55) | `=55` o relevo por camada — conta-se no roteador desta família |
| ADR | **0** | a lei está no doc 30 §15 |
| `Cargo.lock` | **0** | (`rayon` já era dependência da `ph2d-app-sculpt3d`) |
| chaves i18n | **0** novas · **2** textos | `panel.painter_layers.piece.off_here` (sai «relief per layer») · `app.sculpt3d.camadas.recusa.a_base` (o porquê novo) |

## §1. O que a W4 traz

Cada camada do Painter na peça leva o SEU relevo de impasto, e o da peça é a dobra da pilha com a lei
do 2D: `Add` soma, `Level` enterra pela cobertura da própria camada, profundidade com sinal (`0` muda,
negativo cava), as escondidas não entram. No painel de Layers, as linhas da peça com relevo mostram a
barra da profundidade e o chip `Add`/`Level` (um arrasto = um `Ctrl+Z`). O traço de impasto pinta só a
camada activa; duplicar leva o relevo; a base continua permanente e no fundo (a lei do 2D). Cena `=55`.

| bloco | commit |
|---|---|
| W4a: a dobra única (`fold_relief_step`, `relief_layers_bottom_up` na `LayerStack`; o 2D chama-as) | `767f2d9b4` |
| W4b+c: a peça dobra a pilha, o traço só na activa, a redobra e a subida só do relevo, o painel, os gates, a cena `=55`, o arnês | `2a3e3ca63` |
| o arrasto cabe num quadro a `32x` (`Inclinacoes::refaz`, a dobra em `rayon`), doc 30 §15, memória | `05b20a284` |
| a cena `=55` fotografada (a lomba visível) | `951777490` |
| a 1.ª redobra da sessão só sinaliza se o relevo mudou (a P10 que sobreviveu no fecho) | `dce8d150c` |
| o que o fecho apanhou (1 `type_complexity`, 1 literal lido como π) | `fa9a7e224` |
| **report do smoke:** o impasto (toda tela semeada) pinta a COR numa camada nova — `Mistura::Camada` (diferença em cor pré-multiplicada e opacidade), doc 30 §16 | `efd72c67a` |
| **2.ª foto do dono:** a encosta fora da tinta de uma camada de cima não acende (`relief_share`, na dobra única — 2D e peça) e o fantasma ao esconder uma camada (a assinatura renova-se em `relevo_nas`), doc 30 §18 | `6da772700` |

## §2. Foundational tocado e a superfície de API (o que um merge pode partir)

| item | mudança | quem parte |
|---|---|---|
| `ph2d_tool_painter::layers` | ficheiro NOVO `relief_fold.rs`: `pub fn fold_relief_step(h, own, depth, composite, cover, cover_max)` (a cobertura deixou de ser um fecho) · `pub fn relief_share` (lê `ph2d_painter_brush::height_film::W_SOLID`) · `pub const RELIEF_FOLD_SEED = -0.0` · `LayerStack::{effectively_visible, relief_layers_bottom_up, set_impasto_depth_norm, toggle_impasto_composite}` (reexportados no `lib.rs`) | — (aditivo) |
| `ph2d_tool_painter` `tool/paint/relief_fields.rs` · `impasto_light.rs` | `height_at` chama `fold_relief_step` e começa em `RELIEF_FOLD_SEED`; o `layer_effectively_visible` privado SAIU (é o da `LayerStack`); `impasto_fields` itera `relief_layers_bottom_up` | uma linha que edite o corpo do `height_at` ou do `impasto_fields` — **a `line/PainterWatercolor` (aberta) mexe no impasto 2D** |
| `ph2d_tool_painter` `tool/layers/mutate.rs` · `layer_edit.rs` | `set_layer_impasto_depth_norm` delega na `LayerStack`; `toggle_layer_impasto_composite` NOVO; o braço `ImpastoLevel` do `apply_layer_edit` chama-o (o `use ReliefComposite` saiu de `layer_edit.rs`) | idem |
| `ph2d_tool_painter` `tool/piece_layers.rs` | `ImpastoDepth`/`ImpastoLevel` deixam de ser «não oferecidos» e pedem `Metadata` | — |
| `ph2d_tool_painter` `tool/paint/relief_fold_probe.rs` (NOVO) | `#[doc(hidden)] PainterTool::composed_relief_of_planes` + `ReliefPlaneProbe` — a sonda do gate cruzado | — |
| `ph2d_mesh_colors::inclinacao` | `Inclinacoes::refaz` NOVO (a escolha incremental/do zero que a `nova` tinha por dentro) | quem edite a `com_threads` |
| `ph2d_sculpt3d` `tela_na_malha.rs` · `tela_na_malha_pousa.rs` | `Mistura::Camada { dpm, da }` NOVA (privada); `leitura(.., camada)` ganhou um parâmetro; numa camada a borracha passa a tirar opacidade | quem edite a pousada |
| `ph2d_mesh_render` | `tinta_gpu_relevo.rs` NOVO (`upload_tinta_relevo_at`, `le_relevo_at`); os buffers `alturas`/`inclinacoes` ganham `COPY_SRC` | quem crie esses buffers noutro sítio |
| `ph2d-panel-painter-layers` `paint_rows.rs` | a linha 3 (profundidade/`Add`) pinta-se também na peça | — |
| `ph2d-app-sculpt3d` | `pilha_da_peca_relevo.rs` NOVO (a dobra da pilha, a assinatura, `troca_relevo(id, …)`, `pinta_camada`); `relevo_composto` saiu do `pilha_da_peca.rs` (686/700); `SceneObject::relevo_sujo`; `slots::sync_mesh` sobe o relevo sozinho; `mesma_estrutura` aceita profundidade e modo; `scenes_relevo_camadas*` | — |

### Consumidores conferidos (a pergunta é «a lei da porta tem de concordar com alguma lei dele?»)

| porta partilhada | consumidores | conferido |
|---|---|---|
| `fold_relief_step` / `RELIEF_FOLD_SEED` / `relief_layers_bottom_up` | o `ReliefFields::height_at` do 2D → a luz de CPU do impasto, o `impasto_gpu_planes(_in)` (o preview de placa do `ph2d-app-painter`, `painter_gpu_preview.rs`) e os gates da `ph2d-render` (`tests/it/impasto_light_gpu.rs`); a peça | a mesma conta, a mesma ordem; a semente `-0` só troca o sinal de um zero (a luz lê gradientes) — os gates de placa do impasto 2D verdes (§5) |
| `Inclinacoes::refaz` | a `nova` (subida inteira) e a subida só do relevo | a `nova` tem a escolha de antes, ao bit (os 72 gates da crate) |

## §3. Para o integrador — linhas abertas

- `line/PainterWatercolor` (aberta): mexe no Painter e no impasto 2D. Colisões possíveis nesta onda:
  `ph2d-tool-painter/src/tool/paint/{relief_fields.rs, impasto_light.rs, paint.rs}`,
  `tool/{layer_edit.rs, layers/mutate.rs}`, `lib.rs`, `layers/mod.rs`. Não negociei com ela; nada
  re-pinado nela. Se ela tiver redigido de novo o `height_at`, a resolução é a dela chamar
  `crate::layers::fold_relief_step` (o gate cruzado `a_dobra_do_relevo_da_peca_e_a_do_2d_ao_bit` diz se
  ficou uma só).

## §4. Gates novos (W4)

| | |
|---|---|
| dobra | `ph2d-tool-painter` `layers::relief_fold::tests` (3) · `piece_layers::tests::a_profundidade_e_o_level_na_peca_sao_os_do_2d` (o «não oferecido» perdeu a profundidade) |
| pilha | `pilha_da_peca::relevo::tests` (5): **a dobra da peça = a do 2D ao bit** (`Add`, `Level`, profundidade negativa, uma escondida; CONTROLO de ordem) · o neutro e a profundidade `0` (a altura ao bit; o corpo pela lei do 2D) · o traço só na activa e o desfazer · duplicar leva o relevo · só a forma da dobra redobra |
| re-escritos | `o_traco_desce_a_camada_activa_e_so_a_ela` · `a_base_fica_e_a_copia_leva_o_relevo` · `as_trocas_do_desfazer_sao_involucoes` (afirmavam «o relevo é da base») |
| cena | `scenes::relevo_camadas::tests` (1) |
| 2.ª foto | `relief_fold::tests::a_parte_do_relevo_de_uma_camada_e_a_sua_tinta_sobre_a_maior` · `pilha_da_peca::relevo::tests::{a_encosta_fora_da_tinta_de_uma_camada_de_cima_nao_conta, esconder_uma_camada_que_ganhou_relevo_num_traco_tira_o_relevo_dela}` · o neutro re-escrito (profundidade `0` = o relevo dela a zero) · R24/R25 |
| report do smoke | núcleo `tela_na_malha::pousa::alfa_tests::a_tela_semeada_com_uma_camada_poe_e_tira_opacidade` · com placa `relevo_painel::o_impasto_numa_camada_nova_pinta_a_cor_e_o_relevo` (CONTROLO a base) · R22/R23 no arnês (sangram, corridas dirigidas) |
| com placa | `tinta_no_produto_tests::relevo_painel` (3): **o seam test do DoD** — o arrasto REAL da profundidade muda o relevo só onde a de cima pinta, a placa tem os bits da peça, as inclinações da placa são as da CPU, o `Ctrl+Z` devolve ao bit (peça e placa), o `Level` enterra · a camada que cobre a peça refaz TODAS as inclinações |
| mutação | `docs/3D/ferramentas/muta_o_relevo_por_camada.sh` NOVO: **25/25** na corrida inteira depois da 2.ª foto (1.ª corrida 20/20; R21 o ramo «refaz todas»; R22–R25 as curas dos reports do smoke) · `muta_a_pilha_da_peca.sh` **43/43** (M5 e P10 re-ancorados: o relevo é a dobra; a cópia LEVA o relevo) · `muta_a_pilha_na_placa.sh` **19/19** (P10 re-ancorada; **sobreviveu na corrida de fecho** — a 1.ª redobra da sessão sinalizava sempre e a subida inteira compunha a placa por ela; curada em `dce8d150c`, sangra numa corrida dirigida: 2 gates de placa) · `muta_os_efeitos_de_vizinhanca.sh` **49/49** · `muta_as_camadas_em_ecra.sh` **50/50** · `muta_o_relevo_na_peca.sh` **16/16** + o controlo inerte C1 · `muta_a_normal_do_relevo.sh` **12/12** + C1 |

## §5. Prova de fecho (batched sobre o diff acumulado)

| passo | resultado |
|---|---|
| `nextest-impacted.sh` | **19 452 / 19 453** — o vermelho é `ph2d-timeline … the_cost_of_sampling_a_path_is_flat_in_its_anchors`, membro da família de [flakes de carga](../../DevOps/FLAKES_DE_CARGA.md) (razão de relógios, `load 12`); passa sozinho. A Timeline não foi tocada |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | ✓ |
| `clippy --workspace --all-targets -D warnings` · `fmt --all --check` | ✓ (depois de 1 `type_complexity` e 1 `approx_constant`, `fa9a7e224`) · ✓ |
| `machete` · `check-standalone-optional` (10/10) · `check-workflow-packages` | ✓ · ✓ · ✓ |
| `architecture_*` + memória `o_indice_da_memoria_conta_o_que_aponta` + `claude_md` | **105 / 105** |
| placa | `ph2d-render` **52/52** (`layer_compositor` + `fx_stack_adjust` + `blend_mode_regression` + `layers_no_alloc`) · a luz do impasto 2D (`ph2d-render --ignored impasto`) **6/6** · `ph2d-flip-render composite_` **19/19** · o preview de placa do Painter (`ph2d-app-painter painter_preview_handoff_tests`) **3/3** · sculpt3d `--ignored tinta_no_produto --skip diag_` **40/40** (+2); depois da cura da cor **41/41** |
| CPU das crates tocadas | `ph2d-tool-painter` (os dois de custo do `mask` — família de carga — passam sozinhos) · `ph2d-panel-painter-layers` · `ph2d-mesh-render` · `ph2d-mesh-colors` (72) · `ph2d-sculpt3d` · `ph2d-app-sculpt3d` ✓ |

⚠️ A 2.ª foto (`6da772700`): `ph2d-tool-painter` **1 452** · `ph2d-app-sculpt3d` **418** · placa: `ph2d-render` 52/52, impasto 2D 6/6, Flip 19/19, preview 3/3, peça **41/41** · `fmt` ✓.

⚠️ A cura da cor (`efd72c67a`, depois do fecho): `ph2d-sculpt3d` **563** · `ph2d-app-sculpt3d` **416** · placa **41/41** · `clippy -D warnings` e `deny warnings` das duas crates · `fmt --all --check` ✓; o `nextest-impacted` não foi repetido sobre ela.

⚠️ As duas curas do `fa9a7e224` (um alias e um literal de teste) entraram depois do `nextest-impacted`; o clippy e os testes da dobra correram verdes sobre elas.

## §6. Perfil de agente (`bash scripts/agent-loop-profile.sh`)

```
  ✗ paralelismo de ferramenta              1.15/passo   alvo: >= 1,5  (9% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                189   alvo: <= 800  (uma janela nova por onda de trabalho)
  ✗ test : check                            519 : 256   alvo: <= 1,0  razao 2.0x (baseline: 4,3x)
  ✗ edicoes pela ferramenta Edit                  37%   alvo: >= 80%  (1595 por script; baseline: 48%)
  ✗ contexto relido por passo (media)         490 mil   alvo: <= 250 mil  (set/2026: 606 mil — 82% do custo)
  ✓ contexto no inicio da sessao               63 mil   alvo: <= 80 mil  (02/10: 380 mil, CLAUDE.md a 710 KB)
```

## §7. Smoke (do dono: a cena `=55`)

Binário compilado nesta worktree no HEAD, depois de `rm -rf target/*/incremental` (`19 G` + `6,8 G`; e de novo depois da cura da cor, `1,8 G` + `1,9 G`); a última, sobre a 2.ª foto (`3b8258734`), foi de `16,9 s`, a 2.ª:

```
$ bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke
    Finished `smoke` profile [optimized] target(s) in 0.21s
```

A foto da cena tal como abre está conferida (`kwin --virtual` + `import -window`): a bola clara com
riscas horizontais em relevo e a faixa vertical cor de terra com a lomba (luz à esquerda, sombra à
direita), `16x` aceso. O clique prova-se no seam test (§4), não na foto.

**Passos (texto para o Enio):**

1. No terminal: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-sculpt3d && env PH2D_SCULPT3D_SMOKE=55 cargo run -p ph2d-host-desktop --profile smoke`
2. Abre a bola pintada em duas camadas: clara com RISCAS horizontais em relevo e, por cima, uma FAIXA
   vertical cor de terra com uma lomba lisa. Na barra de cima troque `IMG` por `PNTR` (o Painter) e
   abra o separador `Layers`: `Layer 2` em cima, `Layer 1` em baixo, cada uma com uma terceira linha —
   a barra da profundidade (`100%`) e o botão `Add`.
3. Na linha da `Layer 2`, arraste a barra da profundidade devagar para a esquerda até `0%`: **a lomba da
   faixa achata enquanto arrasta, e as riscas da camada de baixo ficam iguais**, também por baixo da
   faixa. A cor não muda. Passando de `0%` para a esquerda, a lomba vira um sulco.
4. `Ctrl+Z`: a lomba volta inteira.
5. Carregue no `Add` da `Layer 2` (passa a `Level`): **dentro da faixa as riscas somem** — a lomba lisa
   enterra a textura de baixo. Fora da faixa as riscas ficam.
6. (A cura do report) Carregue no `+` das camadas (nasce a `Layer 3`), escolha no Painter o meio
   `Impasto` e pinte na bola: **a pincelada tem a cor do pincel E o relevo**, e as riscas chegam até à
   borda da tinta (sem a orla lisa à volta). Esconda a `Layer 3` (o olho): **o relevo dela sai todo**.
7. Na linha da `Layer 1`, baixe a profundidade: as riscas achatam fora da faixa; dentro dela (com
   `Level`) nada muda. `Ctrl+Z` várias vezes: tudo volta ao que abriu.
8. **Deu errado se:** a pincelada na `Layer 3` sai sem cor; a lomba não achata ao arrastar; mexer na `Layer 2` muda as riscas fora da faixa; o
   `Level` não apaga as riscas dentro da faixa; a terceira linha (barra e `Add`) não aparece nas camadas;
   ou o `Ctrl+Z` não devolve a lomba.

## §8. `CLAUDE.md` §5.1

A linha do módulo **3D / Sculpt** troca o link do «Último» para este handoff (a natureza do módulo não
mudou).

## §9. Premissas do briefing que a medição/o código derrubou

1. *«a dobra extraída para UMA função pura»* → DUAS portas na `LayerStack` (quem entra, como entra); o
   **tecto de vidro fica no 2D** (aparência em píxeis; a peça nunca o teve).
2. *«profundidade 0 / neutro = no-op ao bit»* → ao bit na ALTURA; o corpo é o máximo dos corpos (a
   cobertura do 2D não escala com a profundidade) e pesa a inclinação — a lei do 2D.
3. *«o que a BASE deixa de ter de especial»* → o relevo; a permanência e o fundo ficam, pela lei do 2D
   (`delete_layer`).
4. *«`PilhaDaPeca::atrasa` não toca o relevo»* → continua a não tocar, agora de propósito: o relevo
   nunca fica atrás na CPU; redobra-se inteiro só quando a assinatura da dobra muda.
5. *«decida CPU incremental vs. placa»* → CPU (medido: `~5 ms` a `32x`, `~19 ms` a `64x`, doc 30 §15);
   a 1.ª redacção custava `34,9 ms` a `32x` (o incremental das inclinações com o plano todo sujo).
6. *«a W4 não pede degrau de `SCULPT_DOC_VERSION`?»* → não pede (ver §0).
7. *«os efeitos de vizinhança da W6 mexem no relevo?»* → não: no 2D a dobra lê só as camadas com relevo.

## §10. ABERTO (nomeado)

- ⏳ **A parede do impasto na peça com pincéis grossos** (3.ª foto do dono, 04/10, doc 30 §18.1): uma orla
  cinzenta à volta da tinta — também numa camada só (medido: igual na base). NÃO é das camadas nem da
  altura: o tecto do 2D não actuaria (joelho `384 px` contra `~60 px`), e a inclinação é a do 2D (bisel
  `~45°`). É a luz física da peça a desenhar a parede. O dono escolheu «parecer como no 2D».

## §11. A PRÓXIMA JANELA (decidido pelo dono a 04/10)

1. **Os efeitos de vizinhança borram também o relevo** (Gaussiano, Nitidez, Bloom, Sombras/Realces), no 2D
   e na peça, a mesma regra — doc 30 §17 (o desenho técnico por decidir, o critério `100 ms` a `32x`).
2. **A parede do impasto na peça a parecer a do 2D**: primeiro o ORÁCULO — a mesma pincelada (pincel
   `48 px`, `Impasto`) renderizada pelo Painter 2D e pela peça, lado a lado, com as sondas
   `relevo_painel::diag_a_orla_sobre_as_riscas` (`PH2D_SONDA_PINCEL`, `PH2D_SONDA_NA_BASE`,
   `PH2D_SONDA_DIR`) — e só depois mexer na luz da peça (`tinta_inclina`, horizonte, brilho). ⛔ O tecto
   de vidro do 2D já foi medido e não é a alavanca (doc 30 §18.1).
- ✅ **Decisão do dono (04/10): os efeitos de vizinhança borram TAMBÉM o relevo** (Gaussiano, Nitidez,
  Bloom, Sombras/Realces), no 2D e na peça, com a mesma regra — é a PRÓXIMA onda, janela nova (doc 30
  §17: o desenho técnico por decidir e o critério de custo).
- **`64x`+ com uma camada de relevo que cobre a peça**: um passo da profundidade `~19 ms` (as
  inclinações refeitas na CPU). Cura medida possível: as inclinações num compute da placa a partir das
  alturas que já lá estão. `128x`/`256x` não medidos neste gesto.
- Pintar a MÁSCARA de uma camada na peça, grupos, camadas de textura, Lock/Ref na peça (herdados da W3).
- Herdado e ainda aberto: o §9 dos handoffs superseded (W5 os ajustes ponto a ponto, W7 remesh, `64x`
  com raio grande nos efeitos de vizinhança, a cor por vértice um gesto atrás com vizinhos).
- ⚠️ 3 âncoras de `docs/Skeleton/ferramentas/muta_a_silhueta_do_contacto.sh` (B1, B5, B14) **vêm partidas
  do `main`** — não são desta linha.

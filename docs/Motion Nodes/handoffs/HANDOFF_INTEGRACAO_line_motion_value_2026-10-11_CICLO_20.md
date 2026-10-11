# HANDOFF de integração — `line/motion-value`, CICLO 20 FECHADO (W4–W5) (2026-10-11)

> Ciclo 20 «O ACABAMENTO» ([doc 129](../129_ciclo_20_o_acabamento.md): desenho §3, medições §5,
> lições §6, abertos §7). Este handoff cobre o que veio DEPOIS do
> [anterior](HANDOFF_INTEGRACAO_line_motion_value_2026-10-11_CICLO_20_W3b.md) (W0–W3b, já no `main`):
> **W4**, **W4b** e **W5**. O mecanismo de cada wave está no doc 129 (§5.7, §5.8, §5.9), não aqui.

## §1 — Identidade

- Branch `line/motion-value`, HEAD do código `a03d0a938` (o commit seguinte só preenche este handoff e
  a memória).
- Merge-base com o `main`: **`8f20c60de`**. **10 commits** (mais velho → mais novo): `ce01e3114`,
  `6c391500c`, `21e5e9fc2`, `2508cc6cf` (W4) · `876f9b624`, `1c5be0d12`, `53578474c` (W4b) ·
  `7ec680a8b` (correcções do portão de fecho) · `1cc498b6f`, `d521d1852` (W5, mutação).
- 67 ficheiros, +2 916 / −283 (a maior parte: fotos, PDF do tutorial 13, testes).

## §2 — O que o ciclo entrega (W4–W5)

- **W4** (doc 129 §5.7): a pilha de filtros só no retângulo da CENA (4K: 2,01 → 0,77 ms); as variantes
  de foto `PH2D_ACABAMENTO_PASSO`; o tutorial 13; a frase do cartão.
- **W4b** (doc 129 §3.5 e §5.8): a mistura do `Output` atravessa a camada filtrada.
  - `ph2d_render::Pousa` + `BandBlit::blit_pousa`: mistura de HARDWARE (`Soma` One/One, `Tela`
    One/OneMinusSrc, `Multiplica` em 2 passes dentro de UM passe de desenho); os pipelines extra são
    preguiçosos (`OnceLock`). `FxStackPass::run_straight`.
  - A cena vectorial do Motion vai PARA DENTRO da camada filtrada (`MotionFiltro::cena_da_camada` /
    `alvo_da_cena`; na shell, UMA linha em `fase_vector_overlays.rs`); o Vello renderiza sem o mundo
    por baixo e cola na camada; `Each Shape` corre sobre o conjunto Vello como UMA peça.
  - A camada pousa com `pousa_do_sink` (`Scene`/`Everything` = o modo; `Copies` = Normal).
    `MisturaDoSink::pousa_na_camada` (o `Scene` deixa de agrupar com a camada ligada) é perguntada
    pelas DUAS rotas: o pump (`isola_na_camada`, `ph2d-eval-motion`) e a placa
    (`sink_mistura_em_grupo(graph, sink, isolada)` + `estilos_das_saidas`). `isola_na_camada` suja o
    pump quando vira (achado da auditoria, §5).
  - Item 4 decidido: UMA camada por Motion; saídas com modos diferentes pousam com a da primeira e o
    cartão di-lo (`outputs_land_differently`).
  - i18n: SAIU `app.motion.motion_filter.shapes_off_the_layer`; ENTRARAM
    `each_shape_sees_the_vector_path_as_one` e `outputs_land_differently`.
  - Cena `=135`: o passo `9` (`Output` em `Add`) substitui o `mistura`; tutorial 13, secção
    «10 · Misturar com a cena», PDF regenerado, figura `fig/acabamento_mistura.png`; fotos
    `fotos_W4b/passo_0.png`, `passo_9.png`.
- **W5**: provas de mutação (doc 129 §5.9) — §6 abaixo.
- **MEDIDO** (doc 129 §5.8): a pousa contra a lei ≤ 1 nível (fundos meio-transparentes incluídos);
  contra o Vello 0 níveis (18 células opacas); a tabela do doc 118 de ponta a ponta pelo caminho do
  produto com pilha neutra = o quadro SEM filtro **byte-idêntico** em 9 células, nos dois ramos de
  colagem do Vello (`Each Shape` e `Whole Layer`). Preço do Vello na camada (release, 12 blocos
  intercalados, mínimo, `load` 12–23): 1600×972 com 256: 0,10 → 0,49 ms; 1600×972 com 4096: 0,10 →
  1,85; 4K 256: 0,29 → 1,54; 4K 4096: 0,28 → 2,25.

## §3 — Superfície de colisão (FOUNDATIONAL e partilhado)

Saída COLADA de `bash /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value/scripts/collision-surface.sh`
(11/10, HEAD `d521d1852`) — REFERÊNCIA, não evidência (§1.5.9 item 3): a coluna `base:` é o merge-base;
o integrador lê o valor do `main` no ficheiro e conta o delta.

```
  merge-base 8f20c60de   ·   10 commit(s)   ·   67 arquivo(s)
▸ SCHEMAS
    PROJECT_SCHEMA                        188   (base: 188)
      └ tripla do gate               (188, 13, 22)   (base: (188, 13, 22))
    VEC_SCENE_SCHEMA 22 (base: 22) · FLIP_SCHEMA 13 (base: 13) · DOC_VERSION (timeline) 20 (base: 20)
▸ REGISTRO DE COMPONENTES
    ph2d-ecs 107 (base: 107) · ph2d-render (espelho) 108 (base: 108) · ph2d-script (espelho) 108 (base: 108)
▸ CONTRATO CONGELADO (§6)
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado
▸ ADR: esta linha não cria ADR (último no disco 0181)
▸ Cargo.lock: nenhum '+name' novo
▸ MARCADORES DE CONFLITO: nenhum · TETOS DE LOC: nenhum arquivo da linha passa do teto
```

Contadores, como DELTA sobre a base: `PROJECT_SCHEMA` **0**; registos de componentes **0/0/0**; ADR
**0**; contratos congelados **0**. `Cargo.lock`: `+1` linha = a aresta INTERNA `ph2d-fx-cells →
ph2d-lens` (a lente do relógio), nenhum pacote externo.

**Contratos congelados NÃO tocados** — evidência: `git diff 8f20c60de..HEAD --name-only | grep -Ei
"nodegraph|ph2d-vec|tool|contract|schema"` só devolve `Cargo.lock` (o `grep` não casa com
`NodeOp`/`OpResolver`/`NodeManifest`, `Tool`, nem o contrato Vector); `git diff 8f20c60de..HEAD | grep
-ic schema` = **0**.

**Foundational tocado (todo ADITIVO):**

| crate | o quê | porquê |
|---|---|---|
| `ph2d-render` | `Pousa`, `BandBlit::blit_pousa`, `FxStackPass::run_straight` (`band_blit.rs`, `fx_stack_camada.rs`, `fx_stack.rs`, `lib.rs`); `blit_rect` mantém a assinatura e chama `blit_pousa(.., Pousa::Normal)` — byte-idêntico | a mistura de hardware da camada |
| `ph2d-eval-motion` | `MisturaDoSink::pousa_na_camada` (`sink_style.rs`); `MotionCookPump::isola_na_camada` / `camada_isolada` (`lib.rs`); o ramo do pump só ACTIVO com a camada isolada | as duas rotas respondem à mesma pergunta |
| `ph2d-i18n` | `app_motion.rs`: +2 chaves, −1 | cartão |
| `ph2d-fx-cells` | `Cargo.toml` + `ph2d-lens` (gates) | relógio |
| shell | `shells/desktop/src/render_loop/fase_vector_overlays.rs` (**1 linha**) e o teste `shells/desktop/tests/it/the_sink_style_is_one_answer_for_both_routes.rs` (+3 gates de texto) | o alvo da cena |

Símbolos novos que podem COLIDIR (grepar no `main`): `Pousa`, `blit_pousa`, `run_straight`,
`pousa_na_camada`, `isola_na_camada`, `camada_isolada`, `cena_da_camada`, `alvo_da_cena`,
`quadro_da_cena` (`motion_lens.rs`), `sink_mistura_em_grupo` (assinatura ganhou `isolada`),
`estilos_das_saidas`; as chaves i18n acima; a cena `PH2D_GPU_COOK_DEMO=135` (passo `9`).

**O que um merge pode partir:**

- **`ph2d-render/src/band_blit.rs`** é o ficheiro provável de colisão com outra linha (a Luz 2D e o
  Painter usam o `BandBlit`): rode `collision-surface.sh` PELO CAMINHO ABSOLUTO em cada worktree e leia
  o `main` no ficheiro. A chamada de `blit_rect` tem de continuar a ser o caso `Pousa::Normal`.
- `sink_mistura_em_grupo` mudou de assinatura: um chamador novo vindo de outra linha não compila.
- Chaves i18n: o censo de texto reprova se `shapes_off_the_layer` for reintroduzida por um merge.
- Tectos de LOC da shell somam entre linhas: `fase_vector_overlays.rs` está no tecto de função
  (200) — a escolha vive em `alvo_da_cena`, não ali.
- Esta linha NÃO mexe em esquema, registo de componentes, `LIVE_SECTIONS`, nem ADR.

## §4 — Gates e provas de fecho

Portão batched sobre o diff acumulado:

- `nextest-impacted`: **17 500 / 17 501**; o 1 vermelho era `fn_loc_caps` (`fase_vector_overlays`
  201/200) — curado movendo a escolha para `alvo_da_cena`.
- `clippy --all-targets -D warnings`: 2 `type_complexity` nos testes da pousa, curados com um alias;
  `fmt` corrigido.
- Gates da shell `the_shell_only_shrinks`, `arch_safe_clamp_only`, `panel_loc_cap` verdes; censos de
  i18n verdes; censos da árvore combinada **12/12** e **117/117**.
- Suítes de placa: `ph2d-render` **5/5**, `ph2d-app-motion` **58/58**.
- **re-corrida final (sobre `a03d0a938`)**: nextest-impacted 17 507/17 509 — os 2 vermelhos são
  gates de RELÓGIO da `ph2d-light2d-gpu` (fora do diff: `g4_o_custo_do_rc_nao_depende_do_numero_de_luzes`,
  `g4_o_custo_da_holografica_nao_depende_do_numero_de_luzes`) a `load 75`, verdes sozinhos a
  `load 52→41`; clippy `-D warnings` limpo (7 crates); fmt limpo; shell 23/23 + editor-core 7/7;
  placa: ph2d-render 107/107 (o `the_pointwise_op_costs_much_less_than_a_blur` caiu uma vez na suíte e
  passou 3/3 sozinho a `load 12`), ph2d-app-motion 68/68, `mistura_no_pump` 4/4, ph2d-node-fx-filter
  16/16; censos 12/12 · 117/117; `architecture_claude_md_cabe_no_orcamento` 3/3.

## §5 — Auditoria (duas lentes: correcção e fiação)

- «Camada codificada e não desenhada» — **FALSO POSITIVO**: o parâmetro `motion_active` de
  `present_world_sprites` recebe de facto `motion_draws` (o MESMO predicado de `cena_da_camada`).
- «A inversão do isolamento não suja o pump» — **REAL**; curado em `isola_na_camada`, com o gate
  `turning_the_layer_on_in_the_same_tick_relowers` (vermelho → verde).
- Nomes velhos em doc-comments corrigidos.

## §6 — Provas de mutação (W5; tabela completa no doc 129 §5.9)

- **W4b: 21 linhas** — 15 sangraram à primeira; as 6 sobreviventes (P9, P11, P12, P14, P18, P19)
  viraram 6 gates novos → **21/21 sangram**.
- **W0–W4: 41 linhas** — **36/40 sangravam**; Q23 (a shell) não tinha arnês → o gate de texto
  `com_filtros_o_motion_sai_do_passe_fundido`. Sobreviventes Q10, Q17, Q25, Q28 → gates novos, cada um
  visto VERMELHO sob a sua mutação → **41/41**. Q29–Q31 e Q34 (passe do campo) pareciam sobreviver
  porque a 1.ª corrida filtrou só a tabela — a suíte `fx_stack` inteira já as apanhava; ganharam gates
  DIRECTOS (`fx_stack_campo_gpu.rs`, `fx_stack_origem_mapa_gpu.rs`; números no doc 129 §5.9).
- Q7 (a ordem fixa da lente) só é apanhada pelo gate de placa `the_four_together_match_the_cpu`.

## §7 — Smokes (do Enio) e a binária

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && env PH2D_GPU_COOK_DEMO=135 cargo run -p ph2d-host-desktop --profile smoke`
2. No cartão do `Output`, ponha **Blend = Add**.
3. Tem de acontecer: os discos ficam mais claros com o relevo; a sombra escura desaparece.
4. Deu errado se: a sombra escura continua, ou os discos ficam chapados sem relevo.
5. Fotos de cada passo (14): `env PH2D_ACABAMENTO_PASSO=<p> …` com `p` em `0 1 2 3 4 5 5b 6 7a 7b 7c 7d 8 9`
   (mesmo `cd` e comando do passo 1). Tutorial:
   `docs/Motion Nodes/tutoriais/13_o_acabamento.pdf`.
- **binário do smoke**: `bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`,
  2.ª corrida: `Finished \`smoke\` profile [optimized] target(s) in 0.28s` (zero `Compiling`).

## §8 — ABERTO

Doc 129 §7, mais o que a W4b deixou:

- Os filtros de TODOS os fios aplicam-se à camada do Motion inteira (`from_graph` concatena); a cura
  (uma camada por fio) é a que o item 4 recusou — nenhuma cena o tem.
- `Multiplica` dentro da camada por hardware pediria `DUAL_SOURCE_BLENDING`; hoje `Everything`/`Copies`
  vão ao Vello (+0,4–1,9 ms).
- O retângulo da camada usa `scene_viewport`, não `cena_visivel` (a área que o artista VÊ) — por medir.
- [BUGS #19](../BUGS_motion_nodes.md) (o ângulo do `fx.glow` gateado ao contrário).
- A lei de mistura do `fx.filter` sem ficha; a rampa com 2 stops.
- (W3b) A sensibilidade da rota de PLACA do `SpriteRenderer` na silhueta; 4 096 formas = 4 páginas; U2.
- Do ciclo 19, no [continuação da W4b](HANDOFF_CONTINUACAO_line_motion_value_2026-10-11_CICLO_20_W4b.md)
  («ABERTOS NOMEADOS»): a fita acima de 32 MiB · uma linha de COR inteira num gesto · vida e dano pedem
  dois objectos · `Bezier`/`Tint` OKLCH caros na CPU de reserva · gate de gesto pelo caminho REAL do rato
  (BUGS #18).
- Aberto de processo: o smoke do Enio da W4b ainda NÃO foi feito (integrar não é aprovar).

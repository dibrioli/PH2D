---
name: reference-topic-code-gotchas
description: Gotchas silenciosos de código do PH2D — IconId · registry-init · node-sync · companion allowlist · inject · pixel center · exact-pin · ISPC · zero-alloc · Arc::from · áudio mudo · OS-green · low-res (13)
metadata: 
  node_type: memory
  type: reference
  originSessionId: d2f2dbec-7784-4b38-bcf8-424045e2fd3c
  modified: 2026-09-13T23:23:27.365Z
---

- [[feedback_new_tool_icon_needs_iconid]] — tool nova exige `IconId` (gate `enum_order_matches_svgs`)
- [[feedback_fanout_registry_init_friction]] — fan-out registry-init: 2 testes à mão
- [[feedback_node_sync_glob_prefix_gotcha]] — node-sync glob prefix: crate de nó ≠ `ph2d-node-`
- [[feedback_hier_companion_dispatch_allowlist]] — hier companion allowlist: 2 sites em `pointer.rs`
- [[feedback_pipeline_inject_dont_cap]] — inject, don't cap
- [[feedback_pixel_center_vs_edge_coord]] — pixel center vs edge: subtraia 0.5
- [[feedback_exact_pin_needs_substring_gate]] — exact-pin exige gate substring
- [[feedback_ispc_cross_process_concurrency]] — ISPC crasha com cargo concorrente entre processos
- [[feedback_zero_alloc_gate_capacity_not_global_counter]] — zero-alloc gate mede CAPACIDADE, não contador global
- [[reference_arc_from_vec_always_copies]] — `Arc::from(Vec)` SEMPRE copia; `collect` TrustedLen não
- [[project_audio_multichannel_silence]] — áudio: meter vivo, sem som = mute do WirePlumber
- [[project_painter_t19_latent_red_macos_2026_05_28]] — claimed-green ≠ seu-OS-green
- [[project_painter_canvas_res_64_not_sim_scale]] — Painter "low-res" = canvas 64px, não escala da sim
- ⛔ **Um serviço opcional lido de forma OBRIGATÓRIA vira requisito** — `with_registry_ref` (que faz
  `panic!`) num caminho que corre dentro de `HeroScreen::new` matou **12 testes de chrome** que nada
  têm com painéis; a variante `_opt` existe exactamente para isso. *Quem paga é quem nunca pediu o
  serviço* (`line/UIUX`, 2026-08-30, entrega 21)
- [[feedback_a_clamp_before_a_range_test_deletes_the_test]] — `clamp` antes de `contains` apaga o `if`; teste o valor CRU
- ⛔⛔ **Uma chave de cache que hasheia a SPEC mas não o CÓDIGO GERADO À VOLTA dela colide quando a
  spec é PARTILHADA.** Ao dar às reduções um canal de WGSL por-nó (`wgsl_shared`, 08/09), a chave
  do pipeline de `map` continuava a ser `(value, name, column, dim, op, present, params,
  antecessoras)` — e a `pivot::CENTROID_CX` é **literalmente a mesma `ReduceSpec`** em vários nós.
  O primeiro a declarar um canal emprestaria o módulo dele a todos os outros: um pipeline com
  funções que a expressão do vizinho não pede, ou **sem as que ela pede**. ⇒ *tudo o que entra no
  texto do módulo entra na chave*, e uma constante partilhada entre crates é exactamente onde isso
  morde. (O cache do KERNEL estava ileso: a chave dele já é o `NodeTypeId`.)
- ⭐ **Uma recusa cuja razão é «o substrato não tem onde pôr isto» mede o substrato, não a feature
  — e um ponto de extensão APPEND-ONLY costuma custar menos que a recusa.** O
  `motion.bend ▸ direction` ficou fora do dispositivo por o módulo de uma redução não ter onde pôr
  uma função auxiliar (o polinómio HR-5 teria de ser escrito uma 2.ª vez dentro da string). O
  canal novo é **um método de trait com default vazio**: nenhum implementador muda, todo módulo
  que não o declare sai byte a byte igual, e a recusa desaparece. ⚠️ Ao reconferir uma nota
  dessas, vá **ao gerador** ver o que ele de facto cola — não deduza do doc-comment.

- ⛔ **Uma `const` do módulo lida DENTRO de uma função chamada enquanto um objeto de topo nasce é ReferenceError (zona morta), e um `try/catch` em volta a engole** (Pixel Lab W28, 13/09): `readSessionMode()` rodava no literal do `app`, ANTES da linha `const SESSION_MODE_KEY`; o `catch` devolvia o padrão "sempre" para quem tinha escolhido outra coisa, sem erro nenhum. Declarações de `function` sobem; `const`/`let` não. ⇒ o que um inicializador de topo lê tem de estar ACIMA dele (ou escrito na função), e um `catch` que devolve um padrão precisa de um gate que prove que o caminho feliz é o que roda.
- ⛔⛔ **Um ponto de contacto derivado do CENTRO tem braço ZERO e nunca roda nada** (`line/motion-value`, 13/09, doc 109 §6): a resposta do plano usava `centro − n · suporte` como ponto de aplicação, que cai sempre debaixo do centro ⇒ `(ponto − centro) × n = 0`, e uma caixa a `20°` nunca se endireitava (giro medido `0,000`, com a lei de rotação inteira escrita e ligada). O ponto certo é o do SUPORTE (o vértice extremo em `−n`) — ⚠️ **e com a face PARALELA à parede ele tem de ser o MEIO dela**: ali os dois cantos tocam à mesma profundidade, e escolher um deles faz uma pilha PARADA tombar sozinha. *Um suporte é um conjunto; o representante honesto dele é o meio.*
- ⚠️ **«A coluna nem nasce» é a pergunta errada quando a coluna já vinha na ENTRADA** (mesma jornada): o gate do `sim.collide` afirmava que uma caixa de chapa não escreve `rot` — e o nó COPIA as colunas que recebe, então ela estava lá a `0`. A pergunta que mede a lei é *«o valor MUDOU?»*; a da ausência só vale onde o produtor é o único a poder criar a coluna (o `sim.step`, que a cria do nada).
- ⛔ **Um passeio de grafo do Motion que segue a PRIMEIRA aresta de saída dá a volta a um laço de simulação** (`line/motion-value`, 13/09, doc 109 §5): o `warp_gizmo::sink_of` fazia `edges().find(from == cur)`, e a `sim.zone` tem duas saídas — a primeira é a entrada ATRASADA do laço (`zone → wind`). O passeio girava `zone → wind → step → collide → zone` até ao limite de passos e devolvia `None`, sem erro. ⇒ *quem procura o sink anda em largura e não segue arestas `delayed`* (o `collider_gizmo::sink_of`).
- ⛔ **`Math.max(...lista)` tem teto de ARGUMENTOS, e um teto em BYTES deixa a lista passar dele** (Pixel Lab W28, 13/09, medido): o Node estoura a pilha a **125 408** argumentos; o Firefox recusou **500 592** (*too many function arguments*) e aceitou 524 288 na mesma página depois — nem determinístico. A fila de desfazer de 64 MB cabe **524 288** passos mínimos (128 bytes), e a abertura da sessão fazia `Math.max(0, ...ids)`. ⇒ *toda redução sobre uma lista cujo tamanho é limitado por bytes (ou por nada) é um laço*; espalhar em argumentos só com contagem pequena PROVADA.
- ⛔⛔ **Uma tabela `&[…]` dentro de uma FUNÇÃO só é `'static` se tudo nela for promovível — uma
  chamada `const fn` NÃO é** (E0515). Ao trocar `&str` por `TextKey::new("…")` nos menus
  (2026-09-16), o `menu_rows()` deixou de compilar; `const { &[…] }` por braço resolve e custa
  indentação — o `rustfmt` partiu cada linha e o ficheiro foi de 659 a **1234**. ⇒ a forma certa é
  **um item `const` com nome por tabela** (o desenho que os menus da timeline já tinham), e o
  `match` fica com uma linha por braço. ⚠️ E todo gate que lia a tabela **como texto** muda de
  ficheiro com ela — um que exclui «a tabela que pinta» das fontes de despacho, sem a exclusão
  nova, lê **verde sobre nada** (mutação provada).
- ⛔ **Substituir marcadores em SEQUÊNCIA deixa um VALOR virar marcador** — `tr_with` com
  `[("name", "{follows}"), ("follows", …)]` reescrevia o nome do artista. Um modelo com peças
  lê-se **numa passagem só**; o risco só aparece quando o valor é texto do UTILIZADOR, que foi
  exactamente o que a migração da moldura fez passar por ali (`line/UIUX`, 2026-09-16).
- ⛔⛔ **Um NOME que também é IDENTIDADE não se traduz no sítio** — o rack do Audio Editor gravava o
  nome inglês do efeito nos presets do utilizador, e a tradução mudaria o efeito que um ficheiro
  carrega. Antes de pôr um rótulo na tabela, pergunte *quem mais o LÊ como chave* (formato de
  ficheiro, procura por nome em cenas de smoke, agrupamentos): separe um `id` estável da chave de
  texto, e leia os nomes antigos como PADRÕES de `match` (que a régua lexical isenta).
  (`line/UIUX`, 2026-09-16, formato `v2` com alias `v1`.)

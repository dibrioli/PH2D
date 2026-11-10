# Diagnóstico por `perf` no app real — o quadro de ~100 ms a 4K e o app que redesenha parado

- **Pergunta** ([`ANTES.md`](ANTES.md), `2bf12269c1`): (1) um quadro de ~100 ms IGUAL nos 15 pincéis a 4K com papel — no
  toque, ~0,5 s depois do Up e ~0,5 s depois do Ctrl+Z (~35 ms a 1080p, ausente a 1024² blocos); (2) parado, 48–70 % de um
  núcleo.
- **Binário:** `target/smoke/ph2d-host-desktop` de 07:17 (`cd02ac77ef`, `CARGO_PROFILE_SMOKE_STRIP=none`; `file`: *with
  debug_info, not stripped*) — o mesmo da matriz «antes». Só documentos mudaram até `2bf12269c1`.
- **Placa:** a Radeon 610M (RADV, `MESA_VK_WSI_DEBUG=sw`), a mesma do `ANTES.md`; `.placa` = `libgallium libvulkan_radeon`,
  sem `nvidia`. Fatia `ph2d-run.sh` (CPU ≤ 1600 %). Carga: `load 0,52–1,4` no começo; outros agentes em paralelo — o que
  vale são as PROPORÇÕES.
- **Roteiro:** `mede_pen_up_4k.sh perf <WxH> <pincel:painel>` (modo novo, este commit): o da matriz para UM pincel, mas o
  `perf record -k CLOCK_MONOTONIC -F 997` ABRE o app (sem `-g`: `perf_event_paranoid=2` recusa a cadeia, medido de novo
  hoje) e cada marca leva os dois relógios. Fases: parado 5 s · mousedown parado 0,4 s · traço 1 s · Up · 3 s · Ctrl+Z ·
  1,5 s · parado 5 s. `PH2D_QUADRO_LENTO_MS=60`. Resumo: `mede_pen_up_4k.py --perf <pasta>/<rotulo>` — cada quadro lento
  recebe as funções FOLHA das amostras no seu intervalo, **pesadas pelos ciclos** (`period`: com `-F` o perf muda o
  período por amostra; contar amostras pesava errado entre janelas).
- **Corridas:** `omissão:digital` e `omissão:water` a 3840×2160 com papel. Dados mínimos em
  [`diagnostico_100ms/`](diagnostico_100ms/) (marcas, `[quadro-lento]`/`[frame]`, os dois resumos completos); o
  `perf.data` não vai ao git (refaz-se com o comando do fim). **Todo `file:line` é do HEAD `2bf12269c1`** (= o código do
  binário); a árvore de trabalho tem edições de outros agentes em curso (ex.: `main.rs` já anda 7 linhas).

## Fenómeno 1 — o quadro de ~100 ms é a FOLHA AO VIVO a reler a tela inteira (luz do relevo + dente do papel)

### Quando (digital 4K)

| quadro lento | intervalo | onde acaba |
|---|---|---|
| 1 | 96,1 ms | ~0,45 s depois de a cena armar (antes de qualquer gesto) |
| 2 | 96,8 ms | **+446 ms depois do mousedown** — ANTES do 1.º movimento (+491 ms) |
| 3 | 98,9 ms | **+473 ms depois do Up** |
| 4 | 99,0 ms | **+451 ms depois do Ctrl+Z** |

Os quatro são o MESMO quadro: sempre ~0,45 s depois de a revisão do conteúdo parar de mudar = `QUADROS_PARADOS = 30`
quadros de ~11–15 ms. **O «toque» da matriz não é o 1.º dab:** o mousedown já muda a revisão; o roteiro segura o botão
0,4 s parado e a leitura dispara aos 30 quadros parados. Na mão do artista: dispara em **toda pausa ≥ ~0,4 s, também a
meio do traço**, e depois de cada Up e cada Ctrl+Z.

### O perfil (as quatro janelas dão o mesmo; aqui a do Up; ≈ 4,0 G ciclos em 99 ms)

**Fios:** 43 com amostras; o fio principal só 2–3 % — está PARADO à espera do pool. ~16 fios × 6 % = o pool do rayon a
1600 % (o tecto da fatia). ≈ 1 s de CPU por leitura, dividido por 16 núcleos.

| % | função folha | de onde |
|---|---|---|
| 15,8 | `ReliefFields::material_at` | luz: material por píxel (`relief_fields.rs:183`) |
| 14,5 | `PainterTool::apply_impasto_light::{closure#0}` | luz: a faixa (`impasto_light.rs:327-399`) |
| 11,5 | `texture::sample::{closure#0}` | dente do papel: `height_at` → `tooth_loads` (`relief_fields.rs:80,117`; `substrate_relief.rs:245`) |
| 9,6 | `texture::sample` | idem |
| 8,0 | `impasto_shade::Rig::shade` | luz |
| 7,6 | `texture::patterns::paper_tile::sample` | dente do papel |
| 5,7 | `ReliefFields::height_at` | luz: altura (moldura de 1 px por faixa) |
| 4,1 | `compositor::compose::encode::{closure#1}` | composição |
| 3,4 | `Drain<u8,…f32::from…>::call_mut` | material u8→f32 (`material_at`) |
| 2,7 | `texture::patterns::sample_kind_t` | dente do papel |
| 2,7 | `ph2d_blend_mode::apply` | composição |
| 1,9 | `<f32 as SlicePartialEq>::equal_same_length` | ? (comparação de planos f32) |
| 1,9 | `texture::patterns::sample_kind_px` | dente do papel |
| 1,6 | `PainterTool::pilha_aplicada` | a leitura (`trait_impls_raster.rs:162`) |
| 1,5 | `compositor::compose::composite_into` | composição |

**Por natureza:** luz do relevo ≈ 50 % (`ph2d_tool_painter`) · dente do papel amostrado por píxel dentro do `height_at`
≈ 35 % (`ph2d_painter_brush::texture`) · conversões do material ≈ 5 % · composição Full ≈ 8–10 %. **É trabalho de tela
inteira:** composição Full + papel + luz do relevo sobre 3840×2160 — não é upload, `regen_mips`, painel nem o pincel (o
`[paint-perf] WORST split` do ANTES: despacho ≤ 2,6 ms).

### O caminho (lido no código)

1. `shells/desktop/src/render_loop/fase_snapshots_publish.rs:95` chama `animation_editor` **todo quadro, sem
   condição** → `:247` → `:262` `revisao_viva(tools)` (`crates/ph2d-app-painter/src/painter_bridge_queries.rs:28`: há
   documento e tinta por assar) → `:264` `follow_live(…, || imagem_por_assar(tools, vivo?.0))`.
2. `crates/ph2d-anim-sheets/src/sheet_cache.rs:222` `follow_live`: revisão igual há `QUADROS_PARADOS` (`:65`, = 30) e
   `devida` ⇒ `read()`. **Não pergunta se alguma folha/editor de animação está À VISTA.**
3. `painter_bridge_queries.rs:22` → `PainterTool::imagem_por_assar` (`crates/ph2d-tool-painter/src/tool/trait_impls_raster.rs:184`)
   → `imagem_aplicada` (`:194`) → pilha não trivial ⇒ `pilha_aplicada` (`:162`): `composite` da tela inteira +
   `compoe_sobre_o_papel` (`tool/papel.rs:236`) + `apply_impasto_light(rgba, full)` (`tool/paint/impasto_light.rs:327`,
   em faixas no pool — o fio principal espera).
4. Dentro da luz, por píxel: `height_at` (`relief_fields.rs:80`) → `s.tooth_loads(cx, cy)` (`:117`;
   `substrate_relief.rs:245`) → `texture::sample` do papel procedural — o dente ESTÁTICO reamostrado a cada leitura.

Escala com os píxeis (4K ≈ 4× 1080p: 98 ms × 35 ms) e some sem papel (1024² blocos: `impasto_visible` falso e pilha
trivial). O 1.º quadro (cena a armar) é a mesma leitura.

### O bloco que cura e a cura mais barata

| ordem | cura | bloco | ganho estimado (4K, Radeon, fatia 16 núcleos) |
|---|---|---|---|
| **1 (a mais barata, P)** | `follow_live` só lê com a folha do objeto **à vista** (editor de animação aberto / cartão ou miniatura a mostrar a folha); escondida, fica `devida` e lê ao abrir | **B10.3** | o quadro de ~98 ms → o de base (~11 ms) nos três instantes e nas pausas, **nos 15 pincéis**: toque 97–104 → ~25 ms (o traço), +1 s 97–109 → ~11, Ctrl+Z 99–105 → ~11 (fora a Water, abaixo) |
| 2 (folha à vista) | plano f32 do dente por `SubstrateKey` + tamanho (estático) no lugar do `texture::sample` por píxel | **B10.2** (e B15.1 Q2) | −~35 % da leitura: ~98 → ~65 ms; também desconta o mesmo dente de TODO quadro de luz no traço |
| 3 | leitura INCREMENTAL (caixa suja ⊕ 1 px, `imagem_aplicada_na_caixa` já existe) | **B10.5** | proporcional à caixa: um traço curto → poucos ms |

Gate red-first da 1: `sem_a_folha_a_vista_imagem_aplicada_nao_e_chamada` (contador) já está no B10.3; a medição é este
modo `perf` (o quadro lento tem de sumir das quatro janelas).

## Fenómeno 2 — parado = o laço em `Poll` + redraw incondicional: a UI inteira refeita ~95×/s

**Medido (digital 4K):** 50–51 % de um núcleo nos dois parados de 5 s; 475 quadros em 5 s (95 q/s, WSI de software sem
vsync); **25,5–26,0 M ciclos por quadro**, **100 % no fio principal** (Water: 56 %, 85 q/s, 31,8–32,6 M ciclos/quadro).

| % | função folha (parado0, digital) |
|---|---|
| 8,9 | `[libc.so.6]` (memcpy/memset/free) |
| 8,6 | `BTreeMap::Iter<a11y::NodeId, editor_core::interaction::state::InteractiveState>::next` |
| 6,6 | `btree search_tree<BoardId, u32>` ⚠ alias provável (abaixo) |
| 4,4 | `BTreeMap::Iter<a11y::NodeId, f32>::next` |
| 3,0 | `btree NodeRef<a11y::NodeId, InteractiveState…>` |
| 2,2 | `[libvulkan_radeon.so]` |
| 2,0 | `BTreeMap::Values<u64, ecs::scene::registry::ComponentTypeEntry>::next` |
| 1,9 | `texture::patterns::preview::render_texture_preview` |
| 1,8 | `BTreeMap<a11y::NodeId, f32>::insert` |
| 1,3 | `VacantEntry<a11y::NodeId, property_box::key_mark::KeyMark>::insert` |
| 1,2 | `btree NodeRef<(u64, ecs::canal::FieldKey), (f32, f32)>` |
| 1,1 | `malloc` |
| 0,8 | `btree NodeRef<text::layout_cache::LayoutCacheKey, parley…>` |
| 0,8 | `BTreeMap::Range<(u64, ecs::canal::FieldKey), f32>::next` |
| 0,8 | `read_fonts::FontRef::table_data` |

**Por crate:** `ph2d_a11y`/estado de interação 22–24 % · `ph2d_ecs` 9–11 % · libc 9 % · buscas em BTreeMap 8 % · `alloc`
5 % · `ph2d_editor_core` 4 % · Vulkan 2–3 % · Vello encode ~2 % · prévia da textura 1,5–2 %. ⚠ O `rustc` funde funções
idênticas (`merge-functions`): uma busca genérica de BTreeMap com chave `u64`-newtype sai com o nome de UMA das
instâncias — o `BoardId, u32` (`documents_fac.rs:43`, `live_round`) é provavelmente outra árvore; a categoria vale, o
nome não.

**Caminho:** `shells/desktop/src/main.rs:983` `about_to_wait` → `:1021` `event_loop.set_control_flow(ControlFlow::Poll)`
+ `shells/desktop/src/render_loop/present_title.rs:66` `host.request_redraw()` incondicional («Continuous redraw»). Sem
entrada nenhuma, cada quadro reconstrói a cena da UI inteira (árvore a11y, estados de interação, registo de componentes,
canais, texto, a faixa da prévia de textura — `crates/ph2d-panel-painter-layers/src/paint_texture.rs:466`
`previa_do_padrao`, sem cache) e codifica no Vello. Não é a tela 4K (a leitura da folha não corre parado: `devida` já
falso).

| cura | bloco | ganho estimado |
|---|---|---|
| **`Wait`/`WaitUntil` por produtor**; os contínuos do documento (traço aberto, papel molhado, `map_async`, Play) seguram o laço | **B23.2** (G; gates G1/G2/G3/G9) | parado 50–56 % → ≈ 0–2 % de um núcleo |
| cache por painel / não refazer a11y e estados de interação por quadro | B23.4 | o custo de TODO quadro (~25 M ciclos ≈ 6 ms do fio principal, também a pintar) |

Não há meia-cura barata e correta antes da B23.2: um tecto de 60 q/s (Fifo) só corta ~37 % aqui (95 → 60) e nada no
monitor do dono, que já apresenta com vsync.

## Achados à parte (Water a 4K, a mesma corrida)

| quadro | perfil | bloco |
|---|---|---|
| **161 ms a +177 ms do mousedown** (o «toque 155 ms» da matriz) | composição pelo vidro `composite_region_sobre_o_papel` 6,6 % + `composite_into` 4,3 % + luz (material 12,9 %, faixa 8,5 %) + dente (18 %) + `rayon collect [u8; 7]` 5,4 % / `RepeatN [f32; 3]` 3,2 % (inits da tela inteira) | B16.9 / B16.12 (pen-down da aguada) |
| 80 ms a +92 ms do Ctrl+Z | `impasto_gpu_planes_in` (planos do relevo para a placa: `material_at` 34,5 %, libc 14 %, `Drain` 11 %, dente 20 %) | B15.6 (dobra das duas pistas) / B17 (undo) |
| 100–108 ms a +527 ms do Up e +537 ms do Ctrl+Z | a mesma folha ao vivo do fenómeno 1 | B10.3 |

## Refazer

```
W=/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor; S=$W/target/mede-pen-up
SAIDA=$S bash $W/scripts/ph2d-run.sh bash $W/docs/Painter/ferramentas/mede_pen_up_4k.sh perf 3840x2160 "omissão:digital" $W/target/smoke/ph2d-host-desktop
python3 $W/docs/Painter/ferramentas/mede_pen_up_4k.py --perf $S/perf/omisso_digital_3840x2160
```

(Bash com o sandbox desligado, por causa do `perf`.) Para o «depois» da B10.3: o quadro lento tem de sumir das janelas
do mousedown, Up e Ctrl+Z; da B23.2: «CPU do processo» parado ≈ 0.

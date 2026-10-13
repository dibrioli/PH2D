# Velocidade dos portões — onde o relógio do fecho e da integração vai (medido 12–13/10/2026)

> Porta única para «por que o fecho/integração demora?». Nasceu do pedido do dono (12/10) depois de um
> «Batched closing gate F13» de **39 min** e de um «Fix the 13 Luz2D red tests» de **2 h 50**. Cada número
> abaixo tem a medição ao lado; o que foi medido e recusado está na tabela `⛔ Recusas MEDIDAS`.

## 1. Onde o tempo foi (12/10)

| medida | número | fonte |
|---|---|---|
| gate F13: espera de comando / total | 37,6 / 38,9 min | transcrição do verificador |
| `nextest-impacted` do F13: testes · binários · parede | 19 950 · 566 · 1 190 s (compilar 1 min 44) | log da corrida |
| grupo `luz2d-placa` (2 por vez) no F13 | ≈ 1 157 s de 1 190 s — **o caminho crítico** | idem |
| núcleos usados durante esse grupo | ~5 de 32; a placa a ~5 % | `top`, `nvidia-smi` |
| 31 testes ≥ 60 s | 51 % dos test-segundos | idem |
| vermelho aceito no `main` (Luz 2D W5m) | pago 3×: MiroClone e UIUX re-provaram «herdado», cura de 2 h 50 | handoffs de 12/10 |
| 14 dias de transcrições: reruns de `nextest-impacted` na mesma sessão | 123 sessões, 8,3 h | `scratchpad` do estudo |
| idem: rebase/merge · `ship.sh` | 4,2 h · 0,5 h — **não são gargalo** | idem |
| tempo por passo do modelo | mediana ~3 s; o custo é o NÚMERO de passos e as esperas | idem |

⚠️ Os «laços de espera» dos agentes (38 h em 14 dias) eram quase todos a espera pelos PRÓPRIOS testes: proibi-los
não poupa relógio. O relógio poupa-se com testes mais rápidos e com não correr de novo o que já passou.

## 2. O que mudou (linha `line/Velocidade`, 13/10)

### 2.0 ⭐⭐⭐ O cache de shaders do driver estava CHEIO (`.cargo/config.toml` `[env]`)
O gate `o_oclusor_fino_faz_a_sombra_do_gabarito` levava **~640 s sozinho** (643 s com o gabarito antigo, 645 s
com o novo — não era o gabarito). O processo usava um núcleo cheio; quatro amostras de pilha pelo `gdb` (o teste
como FILHO do `gdb`: o `ptrace_scope = 1` só deixa traçar descendentes, e o `samply` pede `perf_event_paranoid ≤ 1`)
caíram as quatro em `wgpu_hal::vulkan::Device::create_compute_pipeline` → `_nv002nvvm`, o compilador do driver.
Cada `luz()` do teste monta um `LuzDoQuadro` novo (24 por corrida), e o `~/.cache/nvidia/GLCache` estava em
**1 023 MB — o tecto de fábrica de 1 GiB**: o driver despejava e recompilava as mesmas pipelines a cada processo.

| o mesmo gate (`um_emissor_de_duas_cores_emite_cada_cor_no_seu_lado`), três corridas seguidas | parede |
|---|---|
| tecto de fábrica (1 GiB, cheio) | 52,7 s |
| `__GL_SHADER_DISK_CACHE_SIZE` 20 GiB, 1.ª (enche) | 13,7 s |
| idem, 2.ª (quente) | **0,6 s** |

A cura é a variável no `[env]` do `.cargo/config.toml` (16 GiB; o recurso é o disco de `~/.cache`, 483 GB livres):
vale para os testes, o `nextest` e o `cargo run`, e fora da NVIDIA ninguém a lê. ⚠️ Antes deste achado, a integração
de 12/10 deu aos gates da Luz 2D MAIS prazo (6 × 60 s) por serem lentos «na suíte inteira» — o sintoma era este.
Quem medir velocidade de um gate de placa diz se o cache estava QUENTE: a 1.ª corrida depois de um shader novo
paga a compilação.

### 2.1 O gabarito da Luz 2D ao mesmo bit, 3× mais rápido
Sonda de fases no gate `as_formas_redondas_nao_saem_em_facetas`: **95 % gabarito, 4 % placa**; dentro do gabarito,
os gradientes conjugados do miolo (767 mil nós, ~1 300 iterações por canal, um canal de cada vez, um núcleo).
`crates/ph2d-light2d-ref/src/miolo.rs::gradientes_conjugados`: os três canais em paralelo, a vizinhança compacta
(`[u32; 4]` por nó em vez do `Viz` inteiro) e três passagens por iteração em vez de seis, com as somas fundidas
na MESMA ordem (o zero de partida é o do `Sum` de `f64`).

| variante (o mesmo gate, `ci-test`, sozinho) | tudo | gabarito | impressão dos valores do gabarito |
|---|---|---|---|
| antes | 61,1 s | 57,9 s | `9bdb5b2ed184a704` |
| canais em paralelo + vizinhança compacta | 23,7 s | 20,6 s | `9bdb5b2ed184a704` |
| + produto `A x` em 4 fios por canal | 22,4 s | 19,3 s | `9bdb5b2ed184a704` — **recusado** (6 %, não paga a complexidade) |
| + passagens fundidas (o que ficou) | 20,2 s | 17,1 s | `9bdb5b2ed184a704` |

O limite agora é a banda de memória (~55 GB/s nas três threads). Ir além exige MENOS iterações (multigrelha como
pré-condicionador) — muda o bit, logo só com a prova de que nenhum veredito muda.

Na suíte da Luz 2D (155 testes, `ci-test`, `load` 1–3): `as_formas_redondas` 82 → 21 s, `o_miolo_das_formas`
98 → 38 s, `o_interior_do_opaco` 335 → 188 s, os três `c3_grade_fina` de ~120 → 50–60 s.

### 2.2 O seletor de impacto PRECISO (`scripts/impacto-preciso.py`)
O `rdeps()` do nextest segue as dev-arestas **em cadeia**. O `ph2d-render` tem a pintura e o `ph2d-editor-core`
como dev-deps (gates de paridade); por isso todo dependente do renderizador — a Luz 2D na placa, o esqueleto —
corria em qualquer mudança de UI sem um byte do seu binário mudar. A lei nova é a do grafo que o cargo compila:
os testes de `P` mudam se um mudado está no fecho normal+build de `P` ou no de uma dev-dep DIRECTA de `P`. Mais os
leitores de fonte alheia (`CARGO_MANIFEST_DIR` + `..`): quem varre a workspace entra a cada diff; quem nomeia
pacotes entra quando um deles muda. Auto-verificação a cada corrida (`--auto-teste`); `PH2D_IMPACTO=rdeps` compara.

Junto, a aresta `ph2d-ecs → ph2d-i18n` saiu (as duas `label()` inglesas, só usadas em testes, apagadas como as
irmãs de 19/09; a palavra resolve-se em quem pinta). Sem as duas mudanças juntas nada muda: só a aresta não
mexia no `rdeps` (151 → 151 pacotes no caso F13); as duas juntas: **151 → 127**, e a `ph2d-light2d-gpu` (metade
da fila de 2 por vez) sai de toda mudança de UI.

### 2.3 O selo do verde (`nextest-impacted.sh`)
A mesma árvore (commitada + por commitar + por rastrear, por um índice temporário) com o mesmo filtro, os
mesmos argumentos e as mesmas `PH2D_*` já passou ⇒ não corre de novo. O fecho da linha e o
`foundational-integrate.sh` sobre um `main` parado são a mesma árvore. `PH2D_PORTAO_FORCAR=1` corre na mesma.

### 2.4 O `main` só recebe verde (DIRETRIZ §1.5.3, `/pd-integracao` 5, `/pd-linha-fechar` 2)

## 3. ⛔ Recusas MEDIDAS

| proposta | por que não |
|---|---|
| proibir os laços de espera dos agentes por hook | o tempo deles era a espera pelos testes; não poupa relógio |
| produto `A x` paralelo dentro de cada canal | 20,6 → 19,3 s (6 %) com `thread::scope` por iteração |
| cache do gabarito em DISCO entre corridas | um gabarito velho depois de mudar a lei é um oráculo que mente; a chave teria de incluir o código |
| gabarito por multigrelha | muda o bit; só com prova de veredito por veredito |
| `__GL_SHADER_DISK_CACHE_SKIP_CLEANUP=1` | o cache cresceria sem tecto; o tecto grande com a limpeza do driver basta |
| comparar com o `junit.xml` do ship de 12/10 | os gates de placa ali (6 s para o `o_oclusor_fino`) correram com o cache do driver ainda a acertar; sozinho em 13/10, cheio, 643 s — régua de placa sem o estado do cache não é base |

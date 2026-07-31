# HANDOFF DE INTEGRAÇÃO — `line/FLIP`: O MOTOR NOVO DE TRAÇO (2026-07-30)

> **Para o agente integrador.** Este é o handoff **mestre** da linha: ele supersede os dois
> handoffs de integração parciais que estão dentro dela (`..._crossing_curves_2026-07-28.md` e
> `..._neighbour_cap_2026-07-28.md` — aquelas waves **nunca foram integradas**, então elas viajam
> nesta mesma entrega e o conteúdo delas segue válido como detalhe).
>
> **Smoke APROVADO pelo Enio em 2026-07-30.** A ordem de integração é dele e já foi dada.

---

## 0. Identificação

| | |
|---|---|
| branch | `line/FLIP` |
| worktree | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-FLIP` |
| tip | `7dafc8194` |
| commits | **57** |
| base do fork | `7ec917506` |
| diff **da linha** | **57 arquivos, +20 146 / −410** |

### ⚠️ O PRIMEIRO FATO: o `main` andou 186 commits desde o fork — **rebase obrigatório**

`git diff main..HEAD` mostra **502 arquivos** e **58 mil deleções**. Isso é **mentira de leitura**:
são as 186 integrações que o `main` ganhou depois do fork (Painter, Vector, physics, anim) aparecendo
invertidas. O diff verdadeiro da linha sai contra a **base**:

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-FLIP
git diff --stat $(git merge-base HEAD main)..HEAD    # 57 arquivos
```

**Rota:** `git rebase main` na worktree, e **NÃO** um merge de `main..HEAD`.

---

## 1. O que a linha entrega

Duas frentes, ambas sob a ordem permanente do Enio de 2026-07-28 (*"encontre um método
completamente novo de renderizar o traço, descarte o atual, alcance o look de um pintor digital
normal sem artefatos, pesquise o estado da arte"*).

### (A) O MOTOR — o traço deixa de ser rasterizado e passa a ser PERCORRIDO

A pergunta *"dado um `FlipStroke`, quais pixels ele acende?"* tem resposta nova. A antiga era um
rasterizador de quads com união global de cobertura; a nova é a **lei aditiva da tinta**, integrada
ao longo do percurso:

```
τ = ∫ f(dn) ds / pitch      f = −ln(1−w)      α = 1 − exp(−τ)
```

⚠️ **Isto não é um candidato entre outros — é o LIMITE dos outros** (doc 12 §22.1). A hierarquia
medida: Beer-Lambert sobre densidade varrida → `τ` exato (o percurso) → soma finita de um buffer de
dabs (GIMP/Krita/Procreate/o nosso Painter) → união global com eleição de profundidade (o raster).
Os buffers de dab da indústria **aproximam** a integral que o percurso computa.

**O percurso é o DEFAULT desde `9a4bdd07b`**; `PH2D_FLIP_NEW_ENGINE=0` é a **escape hatch** para o
rasterizador antigo (que continua vivo e testado — útil para bissecar).

**Dois motores, uma lei:** `binning.rs::walk_pixel` (referência CPU) e `shaders/walk.wgsl` (o compute
que SHIPA), unidos pelo gate `walk_gpu_parity`. ⚠️ **A barra de paridade é DERIVADA do formato**:
`rgba16float` tem 10 bits de mantissa ⇒ `2⁻¹¹ = 4,883e-4`. Ela não foi escolhida.

Fechado dentro de (A): anti-aliasing por **área exata do pixel** (recorte de Sutherland-Hodgman +
sapateiro, `pixel_area.rs`) · airbrush · tip pontilhado · fade sub-pixel · tampa chata · self
overlap · fill · o cap como **termo de fronteira** · o Pass A que **pergunta antes de rasterizar**.

### (B) A AUTORIA — a precisão do traço desenhado (as quatro rodadas de report do Enio)

1. `3ba66539a` — a simplificação cobra contra **a curva que será desenhada**, não contra a corda
   reta (gancho: **8,46 % → 2,86 %** da espessura).
2. `0c7d48bb1` — **o traço acaba onde a mão soltou** (a amostra recusada pelo `MIN_SAMPLE_PX` é
   promovida no pen-up): **8,33 % → 1,85 %**.
3. `c68037d0f` — **o triplo de pontos** (`STROKE_SIMPLIFY_FRACTION` 0,02 → **0,0025**), e o ajuste
   teve de virar **local** antes (reconstrução da vizinhança: 27-64 ms → 1,6-2,0 ms/frame).
4. `fff6177db` — **o ajuste anda da esquerda para a direita**. Era divide-e-conquista com fila por
   pior erro **global**, então cada amostra nova re-decidia o começo — medido, **762 de 1001 frames**
   mexiam em pontos já postos. Agora **zero, por construção**, e 2,5× mais barato.

---

## 2. Schema, contratos, dependências

| item | estado |
|---|---|
| `PROJECT_SCHEMA` | **NÃO tocado** pela linha |
| `FLIP_SCHEMA_VERSION` | **NÃO tocado** pela linha |
| contrato congelado (§6) | **INTACTO** — conferido por grep no diff, zero linha de `Tool`/`RasterEditTool`/`CanvasPaintTool`/`NodeOp`/`OpResolver`/`NodeManifest` |
| ADR novo | **nenhum** |
| crate nova | **nenhuma** |
| dep externa nova | **nenhuma** |
| id / token / i18n | **nenhum** |

⚠️ **UMA nota de rebase sobre schema:** os docs desta linha afirmam `PROJECT_SCHEMA 37 · FLIP_SCHEMA 12`
(o valor no fork). No `main` de hoje o `PROJECT_SCHEMA` é **38** (o `VecFilter` da `line/Vector`).
A linha **não escreve** essa constante, então **não há conflito** — mas as frases nos docs ficam
stale. *Um número falso é pior que ausente*: o integrador deve corrigir as quatro ocorrências em
`docs/Flip/12_novo_motor_pesquisa.md` e nos dois handoffs parciais para o valor do `main` do dia.

**Única mudança de `Cargo.toml`:** `ph2d-painter-brush` entra em **`[dev-dependencies]`** da
`ph2d-flip-render`. ⚠️ Ela é dev-only de propósito — o gate `hardness_law` precisa da função REAL do
Painter como oráculo (uma reescrita local seria a segunda porta da mesma pergunta), e o `src/` não a
toca ⇒ **machete-safe**, o mesmo padrão das crates-nó do gate de paridade da `line/gpu-nodes`.
No `Cargo.lock`, **uma aresta**.

---

## 3. Superfície pública nova (`ph2d-flip-render`)

```rust
pub use binning::{BinSeg, DEFAULT_TILE, MIN_WIDTH_PX, ScreenSpace, TileBins, bin_segments, walk_pixel};
pub use tau::{PAINTER_SPACING, SUB, dab_weight, f_of, sub_pixel_fade};
pub use walk_gpu::{TARGET_FORMAT as WALK_TARGET_FORMAT, WalkJob, WalkPass};
```

Mais um método **aditivo** no compositor de camadas (`ph2d-render`):

```rust
impl LayerCompositor { pub fn has_slice(&self, key: u64) -> bool }
```

⚠️ Ele é **read-only de propósito** (não toca `last_used`): um produtor que pula o re-render deixa
suas camadas estáveis esfriarem, então numa cena com mais camadas que o `cache_cap` elas podem ser
despejadas e re-renderizadas. Isso degrada para *fazer o trabalho*, **nunca** para mostrar pixel
velho — e uma consulta que contasse como uso seria mentira sobre ser read-only. É a lição do
ADR-0124 (pergunte ao DONO, nunca à sua própria cópia) no nível da fatia.

---

## 4. Colisão com o `main` — **4 arquivos**, todos aditivos

```bash
comm -12 <(git diff --name-only $(git merge-base HEAD main)..HEAD | sort) \
         <(git diff --name-only $(git merge-base HEAD main)..main | sort)
```

| arquivo | a LINHA fez | o MAIN fez | risco |
|---|---|---|---|
| `Cargo.lock` | +1 aresta | muitas | baixo — regenerar |
| `ph2d-render/…/compositor/mod.rs` | **+19** (o `has_slice`) | 1 linha, noutro lugar | baixo |
| `shells/desktop/src/main.rs` | **+1** (`mod flip_hardness_smoke;`) | +35 | baixo — lista de módulos |
| `shells/desktop/src/render_loop/mod.rs` | **+2** (`mod flip_pass_stage;` + `self.flip_hardness_smoke();`) | **+452 / −45** | ⚠️ **o único a conferir à mão** |

⚠️ **O `render_loop/mod.rs` é onde olhar.** A linha acrescenta duas linhas; o `main` reescreveu 497.
As duas são aditivas, mas o **sítio da chamada do smoke** pode ter se movido — confira que
`self.flip_hardness_smoke()` continua no mesmo bloco de smokes dos irmãos, e que o `mod` está na
lista.

---

## 5. Estado verde (rodado no tip, nesta máquina)

| suíte | release | debug |
|---|---|---|
| `ph2d-host-desktop --bins` | **1303 ok** | **1303 ok** |
| `ph2d-flip-render` (lib + 15 alvos) | **70 + 12 ok** | — |
| clippy `--all-targets` (shell + flip-render) | **0 avisos** | |
| `cargo fmt --all -- --check` | limpo | |
| `file_loc_caps` (shell, HR-18) | ok | |

### ⚠️ Os gates de GPU são `#[ignore]` — o `ship.sh` NÃO os roda

Rodados **no adapter (RTX)** neste tip, **114 passam**:

```bash
cargo test -p ph2d-flip-render --release -- --ignored
```

`ph2d_flip_render` 9 · `architecture_toll` 2 · `composite_blend` 17 · `gpu_colorize_look` 1 ·
`gpu_fill_fit` 10 · `gpu_render` 34 · `hardness_law` 1 · `integral_law` 7 · `painter_look` 25 ·
`probe_bucket_vs_draw_filled` 1 · `probe_halo_under_soft_line` 1 · `sampling_invariance` 1 ·
**`walk_gpu_parity` 3** · `walk_perf` 2.

⚠️ **Sem adapter eles fazem *skip gracioso*, que NÃO é verde.** O integrador tem de rodá-los na
árvore combinada — o `walk_gpu_parity` é o que prova que o motor que SHIPA (o compute) concorda com
a referência de CPU.

⚠️ **Rode a suíte do shell em DEBUG e RELEASE.** Um gate desta linha nasceu como kill de wall-clock e
**reprovou só em debug** (21,65 ms contra 1,92 em release) — um bar de relógio mede o PERFIL do
build, não o código (ADR-0124). Ele virou razão, mas a política fica.

---

## 6. Smokes (todos `--release`, da worktree)

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-FLIP
env PH2D_FLIP_HARDNESS_SMOKE=1 cargo run -p ph2d-host-desktop --release
```

| env | o que julga |
|---|---|
| **`PH2D_FLIP_HARDNESS_SMOKE=1`** | ⭐ o smoke-mestre desta linha: o traço, a dureza, o cruzamento, e a **autoria** (desenhe devagar e olhe o começo — ele não pode tremer; e um traço longo não pode deformar no início) |
| `PH2D_FLIP_AIRBRUSH_SMOKE=1` | o airbrush no percurso |
| `PH2D_FLIP_TIP_SMOKE=1` · `PH2D_FLIP_RESAMPLE_SMOKE=1` · `PH2D_FLIP_PRESSURE_SMOKE=1` · `PH2D_FLIP_SELF_OVERLAP_SMOKE=1` · `PH2D_FLIP_MULTIPLANE_SMOKE=1` · `PH2D_FLIP_STRIP_SMOKE=1` · `PH2D_FLIP_COLORIZE_SMOKE=1` | as cenas herdadas — **têm de continuar iguais** sob o motor novo |

**Diagnóstico:** `PH2D_FLIP_NEW_ENGINE=0` volta ao rasterizador (o A/B) · `PH2D_FLIP_STATS=1` ·
`PH2D_WALK_DUMP=<dir>` despeja o campo de alfa do percurso.

---

## 7. Mudanças de comportamento que o integrador deve saber

1. **O motor de traço do Flip mudou de default.** Toda arte do Flip é acesa por outro caminho. O
   raster continua no build como escape.
2. **A borda do traço mudou onde a lei antiga estava errada** (medido, não estimado): quina
   **63,75/255** e ângulo **9,72/255 a 45°** — a rampa `½ − sd` é o filtro-caixa 1-D e só é exata em
   borda paralela a um eixo. O miolo **não se move** (0 px acima de 1/255).
3. **O traço guardado tem ~3× mais pontos** (decisão de produto do Enio) e o pen-up **promove** a
   amostra pendente — o traço acaba um pouco mais adiante do que acabava.
4. **Não há mais teto de pontos no ajuste** (`MAX_FIT_POINTS` removido). Quem limita é o piso
   anti-redundância, que é **local**. 9000 amostras → 919 pontos, 1,92 ms.

---

## 8. Aberto — com número, nada como "a fazer"

| # | item | número |
|---|---|---|
| **2b** | cache em **tiles de MUNDO** (sobreviver ao pan, não só à câmera parada) | o (2) já entrega o caso que domina: arte commitada e fantasma de onion custam **0,0001 ms/camada** |
| **3c** | o resíduo de QUINA que a lei de área expôs | **13 px de 1115, ≤ 14,94/255** — **não é regressão**; duas curas candidatas, cada uma com preço |
| **4** | joins & caps como **estilo** | ⛔ a premissa de correção foi **REFUTADA** (§22.9): o `−64` era do rasterizador, e no percurso decompõe em duas causas conhecidas. Sobra pergunta de PRODUTO |
| **5** | a **terceira lei** (`Soft` do Krita) | ⚠️ **funciona exato** e a ressalva do §2.4 não a alcança — mas muda a borda de UMA passada em **+69 %**: decisão de LOOK, do Enio |
| **novo** | cache **incremental** do ajuste | a lei prefixo-estável o torna seguro; o frame do preview custa **0,33 ms a 1200 amostras e 2,42 ms a 9000** (95 % é o ajuste). 2,42 ms = 15 % de um quadro, num traço de 18 000 px |

⚠️ **Os itens 3c, 4 e 5 são decisões do Enio**, já devolvidas a ele com os números. Não são dívida
de engenharia.

---

## 9. Documentação que viaja junto

- **`docs/Flip/12_novo_motor_pesquisa.md`** — o doc-mãe: a pesquisa dos 4 candidatos, a baseline
  medida, a hierarquia das leis (§22.1), a fila do padrão-ouro (§22.4) e as §22.5-§22.11 com cada
  passo medido. **É onde a próxima LLM começa.**
- `docs/Flip/03_traco_rasterizacao.md` — atualizado.
- `docs/HANDOFF_line_FLIP_NOVO_MOTOR_DE_TRACO_2026-07-28.md` — o briefing original do Enio.
- `docs/HANDOFF_INTEGRACAO_line_FLIP_crossing_curves_2026-07-28.md` e
  `..._neighbour_cap_2026-07-28.md` — os dois handoffs parciais, **nunca integrados**, cujo conteúdo
  vale como detalhe das waves anteriores desta mesma entrega.

---

## 10. Checklist do integrador

- [ ] `git rebase main` na worktree (**não** merge)
- [ ] conferir à mão o `render_loop/mod.rs` (§4)
- [ ] `cargo test --workspace` (ou `scripts/nextest-impacted.sh`) em **debug E release**
- [ ] `cargo test -p ph2d-flip-render --release -- --ignored` **no adapter** — 114 esperados
- [ ] `cargo clippy --all-targets` · `cargo fmt --all -- --check` · `file_loc_caps` · machete/deny
- [ ] corrigir as menções stale a `PROJECT_SCHEMA 37` nos docs para o valor do `main` do dia (§2)
- [ ] `./scripts/ship.sh` até verde
- [ ] a §5 do `CLAUDE.md` ganha a entrada desta jornada
- [ ] **push só por ordem explícita do Enio**

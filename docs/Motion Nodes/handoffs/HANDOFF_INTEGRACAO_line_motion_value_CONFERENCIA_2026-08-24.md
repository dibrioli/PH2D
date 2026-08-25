# Handoff de integração — `line/motion-value` (conferência dos nós), 2026-08-24

> DIRETRIZ §1.5.9. **A linha está FECHADA e PARADA.** Não integrou, não pushou, não fez ship.
> Aguarda ordem explícita do Enio (`CLAUDE.md` §0.7).

## 1. Identidade

| | |
|---|---|
| branch | `line/motion-value` |
| último commit de **código** | `520ef02b8` |
| merge-base com `main` | `5038249c6` |
| commits | **11 de código + 1 deste handoff = 12** |
| ficheiros | **161** |
| worktree | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value` |

Os 11, do mais antigo ao mais novo:

```
084d5ab2c  folha 04 (deformadores) FECHA — o deformador aprende a ESCOLHER
cd828288d  folhas 05 e 14 FECHAM — «o irmão sabe e ele não»
3f9816137  cena =91 — o smoke das folhas 05 e 14
c7d9998ed  folhas 03 e 07 FECHAM — metade das oito células não era código
b9842478e  cena =92 — o smoke das folhas 03 e 07, e ela ANDA
a70d755fe  folha 01 FECHA — região, densidade graduada, métrica do Voronoi, variância de vida
b2235721c  o editor de curva era oferecido numa onda que NÃO o lê (3 nós + motion.randomize)
623f9b735  as partículas da cena =94 NÃO simulavam — a força estava FORA do laço
165b7167d  folha 02 FECHA — o PERFIL de um atrator, a força que SATURA, o mar com espectro
ffe5da981  o mar não parecia mar — dois defeitos com a mesma assinatura (Bug #6)
520ef02b8  o teto de LOC do shell é 600 e o meu filtro --bins nunca alcançava o gate
```

## 2. Foundational / partilhado tocado, e porquê

| ficheiro | natureza | porquê |
|---|---|---|
| `crates/ph2d-editor-core/src/interaction/state/chrome_ops.rs` → **`collapse_ops.rs`** (novo) + `mod.rs` (+1 linha) | **MOVE + aditivo** | `collapsed_choice(id) -> Option<bool>` (saber se o utilizador já decidiu sobre uma secção, para uma secção nascer DOBRADA sem esquecer a escolha dele). O `chrome_ops.rs` estava a **707** linhas ⇒ a família do colapso saiu para o irmão; ficou em **602**. ⚠️ **É um MOVE de 90 linhas** — um merge textual que não perceba isso duplica as funções. |
| `crates/ph2d-node-registry/src/ui.rs` (+26) e `src/lib.rs` (+16) | **aditivo** | `ParamGroup::folded` + `const fn folded(self)` + `param_groups_folded()`. Side-metadata do registry, nunca o contrato (§6). |
| `crates/ph2d-node-registry-init/src/lib.rs` (+1) e `Cargo.toml` (+1) | **aditivo, alvo de codegen** | regista `motion.randomize`. ⚠️ **Ponto de colisão clássico**: outra linha que acrescente um nó edita a MESMA lista, no mesmo sítio. |
| `shells/desktop/Cargo.toml` (+7 deps de path) | **aditivo** | as cenas de smoke passam a citar chaves de param pelo símbolo (`ph2d_node_force_wind::MODE`) em vez de as escrever à mão. |
| `shells/desktop/src/render_loop/mod.rs` (**1 hunk**) | substituição | `publish_objects` passa a receber `Appearance { atlas, cooked }`. ⚠️ Ficheiro de **10 875** linhas com allowlist — é o de maior probabilidade de conflito da linha, mas o hunk é pequeno e localizado. |
| `.typos.toml` (+8) | **aditivo** | duas entradas ancoradas (`^dispersa(m\|-se)?$`, `^excepcional$`). ⚠️ **Colisão silenciosa provável** — toda linha apende no fim da mesma lista. |
| `crates/ph2d-gpu-cook/tests/gpu_cpu_parity_sim.rs` | aditivo | um gate `#[ignore]` de paridade do perfil de distância do atrator. |
| `crates/ph2d-panel-motion-params/*` (10 ficheiros) | do módulo | secções dobráveis + o seed a correr ANTES do `paint_rows`. |

**Crates NOVAS (2):** `ph2d-motion-region` (folha, a REGIÃO de uma distribuição) e
`ph2d-node-motion-randomize`. As duas entram no `Cargo.lock` como **path** — nenhuma
dependência externa nova.

## 3. Símbolos que podem COLIDIR — saída do `collision-surface.sh`

⚠️ **Referência, não evidência** (§1.5.9): medida contra o `main` de 2026-08-24. **Re-rode o
script na worktree imediatamente antes de fundir.**

```
SUPERFÍCIE DE COLISÃO — line/motion-value contra main
  merge-base 5038249c6   ·   11 commit(s)   ·   161 arquivo(s)

▸ SCHEMAS
    PROJECT_SCHEMA                         95   (base: 95)
      └ tripla do gate               (95, 13, 14)   (base: (95, 13, 14))
    VEC_SCENE_SCHEMA                       14   (base: 14)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)

▸ REGISTRO DE COMPONENTES
    ph2d-ecs 69 (base 69) · ph2d-render 70 (base 70) · ph2d-script 70 (base 70)

▸ CONTRATO CONGELADO (§6)
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado

▸ ADR
    último no disco: 0163   próximo livre: 0164
    esta linha não cria ADR ⇒ fora de toda disputa de número

▸ Cargo.lock — 2 pacote(s) '+name' novo(s), os DOIS internos (path):
      "ph2d-motion-region"
      "ph2d-node-motion-randomize"

▸ MARCADORES DE CONFLITO — nenhum nos arquivos da linha

▸ TETOS DE LOC — nenhum arquivo da linha passa do teto
```

**⚠️ NÚMEROS QUE SOMAM, e que o git funde MUDO se a outra linha escrever o mesmo literal:**

- **`MAX_DEMO_LEVEL: 89 → 95`** em [`motion_state_demo_router.rs`](../../../shells/desktop/src/motion_state_demo_router.rs),
  com **seis** níveis novos reclamados: **`90 91 92 93 94 95`**. ⚠️ Se outra linha reclamou um
  destes, o valor certo **conta-se** (não se escolhe um dos lados) e o gate
  `no_two_*_scenes_claim_the_same_level` diz.
- **`motion.randomize`** — nome de tipo de nó novo, e uma linha na lista do `registry-init`.
- **7 chaves de param novas** em nós de **força**, todas **APENDADAS no fim do manifesto**
  (o gate append-only exige-o): `force.attractor` → `inner`/`peak`/`reverse` ·
  `force.wind` e `force.vortex` → `mode`/`air_resist` (⚠️ **a MESMA chave e os MESMOS rótulos
  nos dois**, com censo `both_forces_speak_the_same_target_velocity_vocabulary` a prová-lo) ·
  `force.buoyancy` → `waves`.
- **Params novos noutros 15 nós** do módulo (contagem por crate no fim deste doc).
- **Duas funções públicas novas** em `ph2d-node-force-buoyancy`: `surface_at` e
  `finest_wavelength`. Comportamento do nó **intocado** — a cena `=4` é byte-idêntica.
- **`.typos.toml`** — duas entradas no fim da lista.

## 4. Contratos congelados encostados

**NENHUM.** `ph2d-nodegraph/src/node.rs` e `ph2d-editor-core/src/tool.rs` intocados
(confirmado pelo `collision-surface.sh`). Todo canal novo é **side-metadata no registry**.
⇒ **não é preciso ADR**, e a linha não cria nenhum (o `0164` fica livre para quem o quiser).

## 5. O que só o `ship.sh` apanha (o gate de integração NÃO roda)

- **`machete`** — as 2 crates novas (`ph2d-motion-region`, `ph2d-node-motion-randomize`) e os
  7 deps de path novos no `shells/desktop/Cargo.toml` nunca passaram por ele nesta linha.
- **`typos` sobre a árvore INTEIRA** — corri-o só sobre os ficheiros do diff. E as duas
  entradas novas no `.typos.toml` são exactamente o tipo de coisa que a UNIÃO de duas linhas
  duplica.
- **`fmt` pré-fork / `deny` / `audit` (RUSTSEC)** — não corridos aqui; sem deps externas
  novas, o risco de RUSTSEC é o do `main`.
- **clippy latente noutras crates** — o clippy de fecho cobriu as **36** crates derivadas do
  diff, não a workspace.

## 6. Ordem, dependências e o que smokar

**Ordem:** os 11 commits são sequenciais e **não** rebasáveis fora de ordem — o `623f9b735`
conserta a cena que o `b2235721c` criou, e o `520ef02b8` corta o ficheiro que o `ffe5da981`
engordou. Funda a linha inteira ou nenhuma parte dela.

**Clippy de fecho:** alvo **derivado do diff** (36 crates, `-p` por pacote tocado),
`--all-targets` ⇒ **0 avisos**, exit `0`. ⚠️ Não é a workspace inteira — ver §5.

**Gate batched (1×, sobre o diff acumulado):**
`CARGO_INCREMENTAL=0 bash scripts/nextest-impacted.sh --no-fail-fast` ⇒ **13 083 / 13 083
verde**, 1 306 skipped, exit `0`.
⚠️ **A primeira corrida saiu vermelha e escondeu 11 247 testes** (o nextest cancela no 1.º ✗)
— era o teto de LOC, já curado no `520ef02b8`.

**Smokado pelo Enio, com veredito:**

| cena | o que mostra | veredito |
|---|---|---|
| `=91` | folhas 05 e 14 | ✅ ok |
| `=92` | folhas 03 e 07 | ✅ ok |
| `=93` | folha 01 | ✅ ok |
| `=94` | curva `Custom` + `motion.randomize` | ✅ ok |
| `=95` | folha 02 (as quatro forças) | ⚠️ **parcial** — ver abaixo |

**⏳ NÃO smokado / smoke com pendência:**

- **`=95`, o par do mar**, depois da cura do Bug #6: o Enio viu a versão corrigida e devolveu
  *«no 8 as cristas não parecem diferentes»* ⇒ **[Bug #7](../BUGS_motion_nodes.md), ABERTO por
  decisão dele** (*«deixe isso para amanhã»*). ⚠️ **Não é regressão nem bloqueia a integração**:
  a fileira já não é «partículas ao vento» (o que ela era é que a banda de 4 ondas mexe-se
  MENOS — `0,228` contra `0,377` —, porque a boia é um passa-baixo e apaga as camadas finas).
  O mecanismo, a alavanca (**o calado**, não a densidade) e as três saídas a medir estão no
  Bug #7.
- **`=90`** (folha 04, deformadores) nunca teve veredito escrito.
- O gate de paridade GPU do perfil do atrator é `#[ignore]` — **verde na RTX**, não no CI.

**Comando de smoke** (⚠️ o caminho é o da **worktree**, não o do primário):

```
env PH2D_GPU_COOK_DEMO=95 cargo run -p ph2d-host-desktop --release
```

## 7. `incremental/` reclamado

`rm -rf "$(git rev-parse --show-toplevel)"/target/*/incremental` — corrido depois do gate
batched e deste handoff.

## 8. A UMA LINHA para o `CLAUDE.md` §5 (⚠️ **o INTEGRADOR aplica, no primário** — §1.5.6)

Na entrada **Motion Nodes**, a linha `**Aberto:**` começa hoje por
`⭐ **68 P2 e ZERO P1** na conferência`. Substituir por:

> ⭐ **33 P2, ZERO P1 e ZERO P0** na conferência (placar DERIVADO por
> `placar_conferencia.py`, nunca escrito à mão); **onze das dezassete folhas sem P2** — as
> **01, 02, 03, 04, 05, 07, 09, 10, 12, 14 e 16**. Sobram **06** (7), **15** (9), **17** (6),
> **08** (5), **13** (5) e **11** (1). Cenas `=90..=95`.
> ⚠️ **[Bug #7](docs/Motion%20Nodes/BUGS_motion_nodes.md) ABERTO** (adiado pelo Enio): no par do
> mar da cena `=95` a fileira de 4 ondas não mostra cristas diferentes — **a boia é um
> passa-baixo** e apaga as camadas finas (excursão `0,228` contra `0,377`); a alavanca medida é
> o **calado**, e ⛔ **não** a densidade, que reabre a armadilha do Bug #6.

E acrescentar `PH2D_GPU_COOK_DEMO=90..95` já está coberto pela linha de **Smokes** existente
(`PH2D_GPU_COOK_DEMO=<n>`) ⇒ **nada a mexer ali**.

⚠️ **Não acrescente parágrafo de jornada ao §5** — o mecanismo de cada wave vive nos handoffs
desta pasta e nas 17 folhas de `89_conferencia/`.

---

## Apêndice — params apendados por crate (derivado do diff, não escrito de memória)

```
force-attractor 3 · force-buoyancy 1 · force-vortex 2 · force-wind 2
motion-bend 3 · motion-clone 4 · motion-distribute-poisson 3 · motion-emitter 1
motion-grid 2 · motion-lattice 2 · motion-mirror 1 · motion-randomize 7 (nó novo)
motion-scatter 3 · motion-shape 6 · motion-spline-wrap 4 · motion-step 3
motion-trail 1 · motion-transform 2 · motion-verlet-rope 3 · motion-voronoi 1
```

⚠️ **Todos APENDADOS no fim do manifesto** — há gate append-only, e uma inserção a meio da
lista muda o índice de porta de toda cena salva. Se outra linha apendou ao mesmo nó, a UNIÃO
tem de manter as duas e **a ordem relativa não importa**, mas nenhuma pode ficar antes das
portas nomeadas (`anchor_x`/`anchor_y`/`state` do `motion.soft_body` são o caso conhecido).

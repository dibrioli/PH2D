# HANDOFF DE INTEGRAÇÃO — `line/3DModeling` (2026-08-22)

> DIRETRIZ §1.5.9. A linha está **fechada e parada**. ⛔ **Não integrei nem pushei** — nem farei sem
> ordem explícita do Enio (CLAUDE.md §0.7).

---

## 1. Identidade

| | |
|---|---|
| branch | `line/3DModeling` |
| worktree | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-3DModeling` |
| HEAD | o **tip** de `line/3DModeling` — ⚠️ **este handoff É o último commit da linha**, então ele não pode citar o próprio sha sem mentir a cada `amend`. A âncora estável é o commit anterior, `e524dab1c`; leia o tip com `git rev-parse --short line/3DModeling` |
| merge-base com `main` | `ee1432203` |
| commits | **74** |
| arquivos tocados | **145** |
| `main` à frente do fork | **1 commit** (`899a8c18e`, *fix(disco)*) |

⭐ **O rebase deve ser limpo.** O único commit que o `main` ganhou depois do fork toca
`CLAUDE.md`, `docs/DevOps/`, `docs/IntegracaoMultiAgente/`, `docs/architecture/decisions/0104-*` e
`scripts/` — **zero interseção** com os 145 arquivos desta linha. ⚠️ Isto vale para o `main` de
**hoje**; se outra linha integrar antes, reconfira (o item 3 explica porquê).

**O que a linha É:** o módulo de **modelagem 3D por campo implícito / SDF**
([ADR-0161](../../architecture/decisions/0161-3d-modeling-is-an-implicit-field-tree-and-what-the-artist-sees-is-the-traced-field.md)),
34 waves. Crates novas `ph2d-field` · `-field-eval` · `-field-ecs` · `-field-mesh` · `-field-render`
· `-field-profile` · `ph2d-panel-model3d`, mais o host `field3d_*` no shell. Todas as 34 waves foram
**smokadas pelo Enio**, uma a uma.

---

## 2. Foundational / compartilhado tocado, e porquê

⭐ **Quase tudo é ADITIVO** (`+N/-0`). As três exceções estão marcadas ⚠️ e são o risco real de
conflito.

| arquivo | churn | o quê, e porquê |
|---|---|---|
| `crates/ph2d-editor-core/src/ids/chrome/model3d.rs` | +95/-0 | **arquivo NOVO** — os `NodeId` do painel. Isolado de propósito (§1.5.2.1) |
| `crates/ph2d-editor-core/src/ids/chrome/mod.rs` | +3/-0 | uma linha `pub mod model3d;` + re-export |
| `crates/ph2d-editor-core/src/ids/chrome/topbar.rs` | +12/-0 | o id do *pill* `MODEL` no topo |
| `crates/ph2d-editor-core/src/screens/hero/chrome/model3d_toggle.rs` | +42/-0 | **arquivo NOVO** — o pill |
| `crates/ph2d-editor-core/src/screens/hero/chrome/mod.rs` | +13/-0 | declara o irmão novo |
| ⚠️ `crates/ph2d-editor-core/src/screens/hero/paint.rs` | **+91/-70** | o passeio de z-order deixou de ser uma lista solta e virou **uma const com nome + gate** (`every_registered_panel_is_reachable_by_the_z_order_walk`). ⚠️ **É reescrita, não adição** — ver §2.1 |
| ⚠️ `crates/ph2d-editor-core/src/screens/hero/topbar/mod.rs` | **+5/-84** | a tabela de tooltips **saiu** para o irmão novo `topbar/tooltips.rs` (+101/-0) |
| `crates/ph2d-editor-core/src/screens/hero/topbar/tooltips.rs` | +101/-0 | **arquivo NOVO** (o destino da tabela acima) |
| `crates/ph2d-editor-core/src/screens/hero.rs` · `hero/fixture.rs` · `topbar/chip_name.rs` | +8 · +6 · +1 | fio do pill |
| `crates/ph2d-editor-core/tests/architecture_every_panel_is_painted.rs` | +41/-0 | o gate novo do z-order |
| `crates/ph2d-i18n/src/model3d.rs` · `lib.rs` | +114 · +2 | **tabela nova** + a linha que a liga |
| `crates/ph2d-tokens/src/color.rs` · `docs/design/tokens.json` | +11 · +9 | tokens novos do módulo (HR-15) |
| `crates/ph2d-panel-registry-init/{Cargo.toml,src/lib.rs}` | +3 · +2 | registrar o painel |
| ⚠️ `shells/desktop/src/render_loop/hierarchy.rs` | **+94/-1** | o módulo entra na Hierarquia |
| `shells/desktop/src/render_loop/mod.rs` · `input_dispatch.rs` · `input_dispatch/keyboard.rs` · `main.rs` · `init.rs` · `undo.rs` · `hero_intents/hierarchy.rs` | +73 · +21 · +37 · +19 · +4 · +5 · +10 | despacho, teclas, boot, undo |
| ⚠️ `shells/desktop/src/sculpt3d.rs` · `sculpt3d_export.rs` · `sculpt3d_import.rs` | +11/-1 · +6/-1 · +1/-1 | ⚠️ **fronteira com a `line/sculpt3d`** — ver §2.2 |
| `shells/desktop/Cargo.toml` · `crates/ph2d-editor-core/Cargo.toml` | +35 · +7 | deps das crates novas |
| `.typos.toml` | +11/-0 | ver §5 |
| `Cargo.lock` | +487/-42 | ver §3 |

### 2.1 ⚠️ `hero/paint.rs` — a reescrita que vale a pena preservar

O passeio de z-order do `paint_hero_screen` percorria uma lista de `NodeId` escrita à mão, e um
painel **registado, visível e fora dela** nunca era pintado — nada quebra, nada avisa. O arquivo
carregava **seis** comentários a dizer exatamente isso, um por cada vez que o defeito foi pago
(motion, timeline, physics, wet-tuning, tokens, authored, sculpt3d). *Uma regra escrita seis vezes
em comentário é uma regra que ninguém está a aplicar.*

A lista virou uma **const com nome** e ganhou gate
(`every_registered_panel_is_reachable_by_the_z_order_walk`, compara o **registro real** com ela).
⛔ **Numa colisão aqui, não volte à lista solta** — outra linha que tenha acrescentado um painel
deve acabar como uma entrada na const, e o gate diz se ficou de fora.

### 2.2 ⚠️ `sculpt3d*` — dois módulos 3D, e a adjacência é só de NOME

`line/sculpt3d` é o módulo de **escultura**; este é o de **modelagem**. Os três toques aqui são de
uma linha cada: a ponte que importa uma malha esculpida para dentro de uma peça (W21–W23). São
**adjacentes por nome e por assunto, mas não partilham tipo nenhum** — os prefixos de id
(`MODEL3D_*` vs `SCULPT3D_*`) e os painéis (`ph2d-panel-model3d` vs `ph2d-panel-sculpt3d`) nunca se
cruzam. ⚠️ Se a `line/sculpt3d` estiver na mesma leva, **funda-a primeiro** e reconfira estes três.

---

## 3. Símbolos que podem colidir — saída de `scripts/collision-surface.sh`

⚠️ **Colada, não escrita de memória** (rodada em `e524dab1c`, dois commits antes do HEAD — os dois seguintes tocam só docs e dois gates). ⚠️ **É REFERÊNCIA, não
evidência:** ela mede esta linha contra o `main` de **hoje**. Re-rode-a na worktree imediatamente
antes de fundir — a divergência entre as duas leituras é ela própria um achado.

```
SUPERFÍCIE DE COLISÃO — line/3DModeling contra main
  merge-base ee1432203   ·   73 commit(s)   ·   143 arquivo(s)
───────────────────────────────────────────────────────────────────────────────
▸ SCHEMAS — ⚠️ o valor se CONTA contra o main do dia; confira nos TRÊS sítios
    PROJECT_SCHEMA                         84   (base: 84)
      └ tripla do gate               (84, 13, 14)   (base: (84, 13, 14))
    VEC_SCENE_SCHEMA                       14   (base: 14)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)

▸ REGISTRO DE COMPONENTES — o contador é TRÊS, cada um roda só na suíte da própria crate
    ph2d-ecs                               57   (base: 57)
    ph2d-render (espelho)                  58   (base: 58)
    ph2d-script (espelho)                  58   (base: 58)

▸ CONTRATO CONGELADO (§6) — deve ser INTOCADO; se não, exige ADR
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado

▸ ADR — número escolhido numa linha paralela é PROVISÓRIO
    último no disco: 0161   próximo livre: 0162
  ⚠ esta linha cria ADR: 0161   — reconte contra o main do dia

▸ Cargo.lock — pacote EXTERNO novo é o que importa; aresta interna não
  ⚠ 40 pacote(s) '+name' novo(s):
      chacha20 · const-fnv1a-hash · dynasm · dynasmrt · facet · facet-core ·
      facet-macro-parse · facet-macro-types · facet-macros · facet-macros-impl ·
      facet-path · facet-reflect · fidget · fidget-core · fidget-jit · glam ·
      iddqd · impls · mutants · nalgebra · ph2d-field · ph2d-field-ecs ·
      ph2d-field-eval · ph2d-field-mesh · ph2d-field-profile · ph2d-field-render ·
      ph2d-panel-model3d · proc-macro-error-attr3 · proc-macro-error3 · rand ·
      rand_core · safe_arch · simba · smallvec · strum · strum_macros · syn ·
      unsynn · wide · workspace-hack

▸ MARCADORES DE CONFLITO — inclui '|||||||' (diff3)
    nenhum nos arquivos da linha

▸ TETOS DE LOC nos arquivos que a linha tocou
    nenhum arquivo da linha passa do teto
```

### 3.1 O que ler dessa tabela

- ⭐ **Zero schema mexido.** `PROJECT_SCHEMA` fica em 84 e a tripla do gate idem — o documento do
  módulo (`FieldDoc`) **não** entra no `ProjectFile` ainda (ver §6, item aberto). Nenhum degrau para
  contar contra outra linha.
- ⭐ **Zero registro de componente mexido** (57/58/58) — as crates novas registram os seus
  componentes na **própria** crate, não no `ph2d-ecs`.
- ⭐ **Contratos congelados intocados** (§4 abaixo).
- ⚠️ **ADR-0161 é PROVISÓRIO.** Se outra linha desta leva também reivindicar 0161, o número **se
  conta**, não se escolhe — e há **8 referências** a `0161` no código e nos docs desta linha
  (`git grep -n 0161` na worktree) mais o nome do arquivo.
- ⚠️ **`fidget` é a dependência que carrega a maior sub-árvore** (`dynasm`/`dynasmrt` = JIT). Ela é o
  avaliador de campo de referência; `nalgebra`/`simba`/`safe_arch`/`wide` vêm atrás dela.

---

## 4. Contratos congelados encostados

**Nenhum.** `NodeOp`/`OpResolver`/`NodeManifest` e `Tool`/`RasterEditTool`/`CanvasPaintTool`/
`PanelEvent` estão **intocados** (confirmado pelo `collision-surface.sh` acima e pelos arch-gates).

⭐ **E é por desenho:** este módulo **não é uma `Tool`** — ele é um host com painel próprio, e a
navegação orbital vive no **shell**. Foi a mesma decisão que manteve a `line/sculpt3d` fora do
`Tool=12`.

---

## 5. O que só o `ship.sh` pega — **rodado, e o resultado está aqui**

| gate | resultado |
|---|---|
| `cargo fmt --check` | ✓ limpo |
| `cargo clippy --workspace --all-targets --features ph2d-spike/bevy_ecs -- -D warnings` | ✓ limpo |
| `cargo machete` | ✓ limpo |
| `cargo deny check` | ✓ `advisories ok, bans ok, licenses ok, sources ok` — **com os 40 pacotes novos** |
| `cargo audit` | ✓ 3 avisos **allowed** (`memmap2` RUSTSEC-2026-0186/0221, `unsound`) — **pré-existentes, não desta linha** |
| `typos` (project-wide) | ✓ limpo — ⚠️ **estava VERMELHO**, ver abaixo |
| `bash scripts/doc-index.sh --check` | ✓ 14 índices em dia — ⚠️ ver §5.2 |
| `shells/desktop/tests/file_loc_caps.rs` | ✓ — ⚠️ **estava VERMELHO**, ver abaixo |
| `ph2d-editor-core/tests/architecture_workspace_file_loc_cap.rs` | ✓ — ⚠️ **estava VERMELHO** |
| `CARGO_INCREMENTAL=0 bash scripts/nextest-impacted.sh` | ✓ **10.197 testes, 10.197 verdes** (1171 skipped) — ⚠️ **dois estavam VERMELHOS**, ver 5.1.1 e 5.1.2 |

### 5.1 ⚠️ QUATRO vermelhos latentes, encontrados ao preparar ESTE handoff — todos curados

⛔ **Nenhum dos quatro aparece no `cargo test -p <crate>` do inner loop**, e os quatro teriam
chegado ao integrador como `✗` do `ship.sh`. Dois foram apanhados pelo `collision-surface.sh` e dois
pelo `nextest-impacted.sh`.

| # | vermelho | porque estava invisível | cura |
|---|---|---|---|
| 1 | `typos` — 6× `regresso` | scan é **project-wide**, ninguém o roda por crate | `.typos.toml` |
| 2 | teto de LOC — 4 arquivos | gate vive noutra crate (`ph2d-editor-core`) e no `shells/tests/` | **split** |
| 3 | `every_registered_panel_is_reachable_by_the_z_order_walk` | ⭐ **só reprova com as features da workspace ligadas** | ver 5.1.1 |
| 4 | `build_typed_registry_matches_enabled_features` | idem — a mesma causa | ver 5.1.2 |

1. **`typos`** — `regresso` é **português correto** (W23). Curado no formato já estabelecido pelo
   `^pilar$`/`^PILAR$`: **duas entradas ancoradas** (`^regresso$`, `^REGRESSO$`) em vez de um
   `(?i)`, porque a lista é **compartilhada** e um `(?i)` na linha existente arriscaria perder num
   merge o que outra linha lhe tenha acrescentado.
2. **Teto de LOC** — `edit.rs` 1003/700; `field3d_input.rs` 672, `field3d_scene.rs` 670,
   `field3d_smoke.rs` 622 / 600. Curados por **split**, nunca pelo marcador `// ph2d-loc-cap:` nem
   pela allowlist. **Zero mudança de comportamento.**

#### 5.1.1 O gate do z-order media **metade** das rotas — e era meu

O passeio real é `hero.store.panel_z_order() ∪ PANEL_Z_ORDER_FALLBACK`, e o gate da W4 comparava o
registro **só com o fallback**. A rota A não é apenas «o utilizador clicou»: uma **ponte** pode
levantar o painel dela com um `bump_panel_z` explícito quando a tool acorda — é o que o
`painter_bridge` faz com o `painter_layers`. Resultado: o gate declarava *"nunca pintado"* um painel
que é pintado em todo quadro.

⭐ **A exceção agora é explícita E PROVADA:** `RAISED_BY_A_BRIDGE` nomeia a const, e um segundo teste
(`every_bridge_raise_exception_has_a_real_call_site`) **procura a chamada na árvore** — uma ponte que
deixe de levantar o painel torna a entrada obsoleta e o gate diz.

⚠️ **E a primeira versão desse segundo teste era TAUTOLÓGICA:** ele varria a árvore inteira, e a
árvore inteira inclui o próprio arquivo do gate — que carrega a agulha dentro da const. Encontrava a
própria declaração e passava sempre. **Uma prova de mutação apanhou-o** (tirar a chamada real da
ponte deixava-o verde). A cura é a regra certa por si só: *uma ponte vive no `src`, nunca num
`tests/`*.

#### 5.1.2 O painel estava **registado e não contado** desde a W4

`EXPECTED_TYPED` em `ph2d-panel-registry-init` é um `const` que soma `+1` por feature de painel
ligada — um espelho **mantido à mão** do bloco de `push`. A W4 deu o `#[cfg]` ao `push` e **não** ao
contador: 24 painéis contra 23 esperados. ⚠️ A nota ao lado do contador **já dizia exatamente isto**
(*"a new panel feature must be counted here too"*) — *uma regra escrita no sítio certo não é lida
por quem chega pelo outro lado do arquivo.*

⭐ **A causa comum de #3 e #4 é uma só, e é a lição que sai desta linha:**

> ⚠️ **Um gate cuja resposta depende do conjunto de FEATURES tem de ser lido no conjunto que o CI
> usa.** `cargo test -p <crate>` não liga `ph2d-panel-painter-layers` nem `ph2d-panel-model3d`; a
> unificação de features da workspace liga. Os dois passaram verdes durante **quinze waves** e só
> reprovariam no `ship.sh` do integrador.

⭐ *A lição de processo:* **rode `collision-surface.sh` E `nextest-impacted.sh` ao FECHAR, não só ao
integrar.** Os quatro foram apanhados assim, e **nenhum** era visível de dentro do módulo.

### 5.2 ⏸️ `docs/3DModeling` **não** está no `DIRS` do `doc-index.sh` — e é uma decisão, não um buraco

O `--check` passa porque o script indexa **14 diretórios nomeados** e este não é um deles. ⛔ **Não o
acrescente sem ler primeiro** [`docs/3DModeling/README.md`](../README.md): ele é escrito à mão, com
uma tabela que diz *o que cada doc é* e o estado por wave — pôr o diretório no `DIRS` **reescreve-o**
com a tabela derivada e perde isso. O [`handoffs/README.md`](README.md) desta pasta foi escrito à
mão na forma do `docs/Physics/handoffs/`, que é a que **nove de nove** adotaram e que o próprio
script cita como referência. *A decisão é do Enio; a linha registou o custo dos dois lados.*

---

## 6. Ordem, dependências, e o que smokar

- **Ordem:** histórico **linear**, 74 commits, sem dependências fora de si. `rebase --onto main` +
  `--ff-only` deve bastar.
- **Fusão:** nenhuma lista ordenada compartilhada foi **reordenada** — todas as entradas desta linha
  são acréscimos no fim (ids, tokens, i18n, registro de painel). A exceção é `hero/paint.rs` (§2.1).
- **Ship do integrador:** o `ship.sh` pode ainda drenar latentes de OUTRAS linhas fundidas na mesma
  leva (2–4 iterações é o normal registado). Os desta linha estão em §5.

### 6.1 Smoke — o comando, e as cenas

```
cd /home/enio/Documentos/Projetos/PH2D && cargo run -p ph2d-host-desktop --release
```

O módulo abre pelo **pill `MODEL`** no topo (é a porta que a W4 abriu). As cenas de diagnóstico
continuam a existir por env var: `PH2D_FIELD_SMOKE=<n>` (o roteador é
[`field3d_smoke_scenes.rs`](../../../shells/desktop/src/field3d_smoke_scenes.rs) — ⚠️ **o número da
próxima cena se CONTA lendo o roteador**, nunca uma nota).

**Todas as 34 waves foram smokadas pelo Enio.** ⏸️ O que **não** foi smokado, e porquê:

- ⏸️ **O ESPELHO (`Mirror`) não se consegue demonstrar** — ele dobra em torno do centro do nó, e
  toda peça das cenas é simétrica. O verbo está correto e gateado; falta um alvo descentrado.
  **Adiado por decisão do Enio.**
- ⏸️ **Um arquivo de escultura que MUDOU DE SÍTIO** não se reencontra (religar pede UI — é uma
  pergunta que o app ainda não faz por asset nenhum).
- ⏸️ **A ligação à escultura VIVA** do módulo 3D: hoje o vínculo passa pelo **disco** e acorda ao
  **abrir** o projeto.
- ⏸️ **O `FieldDoc` não persiste no `ProjectFile`** — Ctrl+S não guarda a peça de modelagem. É o
  item que, quando fechar, **move o `PROJECT_SCHEMA`** (hoje 84) e passa a ser um número que
  **se conta** contra as outras linhas.

### 6.2 Gates `#[ignore]` — nenhum é um vermelho suprimido

São **21**, e **todos** são instrumentos (`measure_*` / `probe_*` / `dump_*`): custo do pick, custo
do regresso, resolução de exportação, qualidade da malha, convergência da esfera, custo do traçado.
⭐ **Nenhum é um gate desligado por reprovar.** Eles existem para responder «quanto custa?» com um
número, e a leitura de relógio desta workstation não vale nada acima de `load ~5`.

---

## 7. Higiene da worktree

- `rm -rf target/*/incremental` **feito** ao fechar (DIRETRIZ §1.5.9 item 7).
- Working tree **limpa**, tudo commitado, **nada pushado**.
- ⚠️ **Não apague a worktree antes de integrar** — o integrador precisa de re-rodar o
  `collision-surface.sh` dentro dela (§3).

---

## 8. A linha para o `CLAUDE.md §5` — **UMA**, e o resto fica aqui

⚠️ A narrativa das 34 waves vive em
[`docs/3DModeling/06_resultados_cena_e_gizmo.md`](../06_resultados_cena_e_gizmo.md) (§1–§35) e nos
outros cinco docs do módulo. **Não acrescente parágrafo de jornada ao §5.** Bullet a inserir:

⚠️ **Os caminhos dentro da citação são relativos à RAIZ do repo** (onde o `CLAUDE.md` vive), e por
isso **não resolvem a partir desta pasta** — um verificador de links reporta cinco quebrados aqui e
os cinco estão certos no destino. *Cole o bloco como está; não o "conserte".*

> - **3D Modeling (campo implícito)** — modelador **SDF** editável para sempre
>   ([ADR-0161](docs/architecture/decisions/0161-3d-modeling-is-an-implicit-field-tree-and-what-the-artist-sees-is-the-traced-field.md)):
>   `ph2d-field` (documento + primitivas + modificadores) · `-field-eval` (avaliador híbrido, bordo
>   da peça) · `-field-ecs` (a árvore de modelagem **é** a hierarquia da cena) · `-field-mesh`
>   (Surface Nets) · `-field-render` (traçado) · `ph2d-panel-model3d`. Abre pelo pill **MODEL**.
>   ⚠️ **A hierarquia da cena É o documento** — o `FieldDoc` é **cozido** dela a cada quadro, e é por
>   isso que o undo, o olho, o cadeado e o reparentar da casa valem aqui sem código próprio.
>   ⚠️ **Só uma OPERAÇÃO pode ter filhos**, e a lei impõe-se na **derivação** (`promote_leaf_hosts`),
>   nunca em cada gesto. ⚠️ **O painel oferece EXATAMENTE o que o gesto faz** (W34) — a lei está em
>   [`field3d_reach_tests.rs`](shells/desktop/src/field3d_reach_tests.rs), e ela apanha os dois
>   lados (botão mudo · gesto inalcançável).
>   **Aberto:** o `FieldDoc` **não persiste** no `ProjectFile` (fechá-lo move o `PROJECT_SCHEMA`) ·
>   religar uma escultura que mudou de sítio (pede UI) · o vínculo à escultura **viva** do módulo 3D
>   (hoje passa pelo disco) · ⏸️ o `Mirror` não se consegue demonstrar (adiado pelo Enio) ·
>   a exportação não diz o **tamanho** da peça.
>   **Smokes:** pill **MODEL** · `PH2D_FIELD_SMOKE=<n>` (o roteador é
>   [`field3d_smoke_scenes.rs`](shells/desktop/src/field3d_smoke_scenes.rs)).
>   **Ler:** [`docs/3DModeling/`](docs/3DModeling/) ·
>   [handoffs](docs/3DModeling/handoffs/README.md)

---

## 9. As três leis desta linha que valem fora dela

Escritas aqui porque o integrador é quem as leva para o resto do repo:

1. ⭐ **Empurrar a intenção prova o TRATADOR, nunca a ALCANÇABILIDADE** (W34). Um gate que encena o
   clique passa com o botão inexistente — cinco waves seguidas deste módulo curaram costuras mudas, e
   a quinta tinha o buraco **no gate**. A forma certa é ler o **retrato publicado** e exigir
   *«oferecido == age»* para toda fileira e toda seleção.
2. ⭐ **Um predicado que o painel consome tem de ser o MESMO que o gesto guarda** (`can_wrap`,
   `can_detach`). Duas cópias da regra divergem, e a divergência é silenciosa nos dois sentidos.
3. ⭐ **Uma prova de mutação mede o BINÁRIO, não o arquivo** — duas vezes esta linha produziu VERDE
   falso por bug do arranjo (`shutil.copy2` preserva o mtime e o cargo serve a build mutada; um
   `if "subida" in label` contra um rótulo escrito `SUBIDA`). O arranjo tem de exigir
   `"Compiling <pkg>"` na saída e `assert` de contagem em **cada** agulha.

---

**Resumo:** linha `3DModeling` pronta (tip de `line/3DModeling`, 74 commits, merge-base `ee1432203`).
Foundational tocado é aditivo exceto `hero/paint.rs`, `hero/topbar/mod.rs` e
`render_loop/hierarchy.rs`; zero schema, zero registro de componente, zero contrato congelado;
ADR-0161 provisório; 40 pacotes externos novos (`deny`/`audit` verdes); os gates que só o `ship.sh`
pega foram **rodados e estão verdes** (dois estavam vermelhos e foram curados). **Aguardo ordem de
integração.**

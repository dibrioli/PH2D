# HANDOFF DE INTEGRAÇÃO — `line/3DModeling`, 2026-08-26

> Para o **agente integrador**. A linha está fechada; **não integrei e não pushei** (CLAUDE.md §0.7).
> ⚠️ **Re-rode `bash /home/enio/Documentos/Projetos/PH2D/scripts/collision-surface.sh` nesta worktree
> imediatamente antes de fundir** — a tabela do §3 mede o `main` do dia em que fechei, e uma linha
> fundida no meio muda toda a coluna «base». A divergência entre as duas leituras é ela própria um
> achado.

---

## §1 — Identidade

| | |
|---|---|
| branch | `line/3DModeling` |
| HEAD | `4e359b17d` |
| merge-base com `main` | `0f5ce8040` |
| commits | **25** |
| arquivos | **71** (`+9 195 / −586`) |

Waves `w59`–`w80`. As três primeiras (`w59`–`w61b`) são de antes desta jornada e já estavam
commitadas na linha; as restantes são de 2026-08-26.

---

## §2 — Foundational / compartilhado tocado, e por quê

⚠️ **Nada fora do módulo foi tocado por conveniência.** A lista completa do que sai de
`crates/ph2d-field*` e `shells/desktop/src/field3d_*`:

| arquivo | o que mudou | aditivo? |
|---|---|---|
| `crates/ph2d-i18n/src/model3d.rs` | 2 chaves novas (`mod.mirror_y`, `mod.mirror_z`) e o rótulo de `mod.mirror` passou de `"Mirror"` para `"Mirror X"` | ⚠️ **quase** — a 3.ª linha é uma **edição** de rótulo |
| `crates/ph2d-panel-hierarchy/src/row.rs` | `"ISO" => TagTone::Warn` no mapa de tons do selo | ✅ aditivo |
| `crates/ph2d-quadchain/*` | crate **nova** desta linha (`phase_zero`, `quads_or_keep_from`, `ChainTiming`) — a cadeia de quads usada pela exportação | ✅ nova |
| `shells/desktop/src/main.rs` | registo dos módulos novos (`field3d_export_job`, `field3d_scene_acts`, …) | ✅ aditivo |
| `shells/desktop/src/modal.rs` | `pick_file` ganhou o irmão que a exportação usa (a porta que declara o congelamento) | ✅ aditivo |
| `shells/desktop/src/render_loop/mod.rs` | **+12 linhas**: o dreno da bancada de exportação e o pedido de religar, ao lado dos pedidos que já lá estavam | ✅ aditivo |
| `Cargo.lock` · `shells/desktop/Cargo.toml` | a dependência da crate nova | ✅ aditivo |
| `CLAUDE.md` | ⚠️ **§5, a linha do módulo** — ver §8 |
| `project-memory/*` | 9 memórias novas + 2 linhas no índice | ✅ aditivo |

⭐ **O `render_loop/mod.rs` é o arquivo de maior risco de conflito** (10 918 linhas, toda linha viva
lhe toca). As minhas 12 linhas estão **num bloco só**, dentro do `if` que já drenava
`take_export_request` / `take_import_request` — se houver conflito, é textual e a resolução é manter
**os dois** lados.

---

## §3 — Superfície de colisão (saída do `collision-surface.sh`, **não escrita de memória**)

```text
SUPERFÍCIE DE COLISÃO — line/3DModeling contra main
  merge-base 0f5ce8040   ·   25 commit(s)   ·   71 arquivo(s)
───────────────────────────────────────────────────────────────────────────────
▸ SCHEMAS — ⚠️ o valor se CONTA contra o main do dia; confira nos TRÊS sítios
    PROJECT_SCHEMA                         97   (base: 97)
      └ tripla do gate               (97, 13, 14)   (base: (97, 13, 14))
    VEC_SCENE_SCHEMA                       14   (base: 14)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)

▸ REGISTRO DE COMPONENTES — o contador é TRÊS, cada um roda só na suíte da própria crate
    ph2d-ecs                              —   (base: —)
    ph2d-render (espelho)                  71   (base: 71)
    ph2d-script (espelho)                  71   (base: 71)

▸ CONTRATO CONGELADO (§6) — deve ser INTOCADO; se não, exige ADR
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado

▸ ADR — número escolhido numa linha paralela é PROVISÓRIO
    último no disco: 0167   próximo livre: 0168
    esta linha não cria ADR ⇒ fora de toda disputa de número

▸ Cargo.lock — pacote EXTERNO novo é o que importa; aresta interna não
  ⚠ 1 pacote(s) '+name' novo(s):
      "ph2d-quadchain"

▸ MARCADORES DE CONFLITO — inclui '|||||||' (diff3)
    nenhum nos arquivos da linha

▸ TETOS DE LOC nos arquivos que a linha tocou
    nenhum arquivo da linha passa do teto
```

⭐ **Nenhum schema se mexeu** — e não por sorte: a extensão do vocabulário de modificadores
(`Unary::MirrorY`/`MirrorZ`) foi **append-only, no fim do `enum`**, precisamente porque o documento
serializa por posição. Um ficheiro `.ph2dproj` de ontem abre igual.

### Símbolos NOVOS que outra linha pode ter escolhido

| símbolo | valor | onde |
|---|---|---|
| `Unary::MirrorY` / `MirrorZ` | variantes **no fim** do `enum` | `crates/ph2d-field/src/mods.rs` |
| `UnaryKind::MirrorY` / `MirrorZ` · `ALL` passa de `[_; 6]` para `[_; 8]` | — | idem |
| `PrimitiveKind` (enum novo) + `Primitive::kind()` | `ALL` = 6 | `crates/ph2d-field/src/lib.rs` |
| `"panel.model3d.mod.mirror_y"` · `"panel.model3d.mod.mirror_z"` | chaves i18n | `crates/ph2d-i18n/src/model3d.rs` |
| `"panel.model3d.act.relink"` | chave de acção | `shells/desktop/src/field3d_scene_acts.rs` |
| `ISOLATE_BADGE = "ISO"` | selo da Hierarquia | idem (⚠️ o mapa de tons vive em `ph2d-panel-hierarchy`) |
| `ph2d_field_render::{SPECIALISED, SPECIALISE_NS, STEP_SAMPLES, EDGE_CHUNK}` | contadores `#[doc(hidden)]` | `crates/ph2d-field-render/src/` |
| `ph2d_field_eval::hybrid::{FLOAT_TAPES, GRAD_TAPES}` | idem | `crates/ph2d-field-eval/src/hybrid.rs` |
| crate `ph2d-quadchain` | nome de pacote | workspace |

⚠️ **`ph2d-quadchain` é o único nome que atravessa a workspace.** Se a `line/sculpt3d` (que trabalha
na cadeia de quads) tiver criado uma crate com o mesmo propósito, isto é **mesmo-símbolo** e a
resolução é de produto, não textual — **pare e reporte** (DIRETRIZ §1.5.5).

---

## §4 — Contratos congelados

**Nenhum.** `NodeOp`/`OpResolver`/`NodeManifest` e `Tool`/`RasterEditTool`/`CanvasPaintTool`/
`PanelEvent` estão **intocados** (confirmado pelo script acima). Esta linha não cria ADR.

---

## §5 — O que só o `ship.sh` apanha (o gate de integração não roda)

1. **`typos` da árvore inteira** — corri só nos meus diretórios. ⚠️ Um typo **pré-fork** noutro sítio
   aparece no ship e não é meu; e eu corrigi um do `w59` (`directos` → `directamente`) que estava na
   minha própria linha.
2. **`cargo machete`** — a linha acrescenta uma dependência interna (`ph2d-quadchain` no
   `shells/desktop`), que **é** usada; nenhuma dependência externa nova.
3. **`cargo deny` / `audit`** — nenhum pacote externo novo ⇒ sem RUSTSEC novo esperado.
4. **`clippy --all-targets` da workspace** — corri por crate tocada (0 avisos); o clippy **latente**
   de crates que não toquei é do ship.
5. **`nextest` da workspace** — corri as suítes das crates tocadas + os `field3d*` do shell.
   ⚠️ **Não corri a suíte inteira do shell** (3 800+ testes): é o que o gate de integração faz.
6. ⚠️ **`physics_ecs_c9`** — o `CLAUDE.md` avisa que ele está por re-capturar desde o snapshot v2 da
   `line/components`. **Não é desta linha**, mas é o candidato nº 1 a vermelho na matriz 3-OS.

---

## §6 — Ordem, dependências e o que smokar

**Ordem:** os 25 commits são sequenciais e cada um compila; ⚠️ **não os reordene** — a `w75` corrige
uma lei que a `w70`/`w71` medem, e a `w80` deriva um gate que a `w79` mostrou estar a mentir.

### Já smokado pelo Enio (aprovado)

| wave | o que ele viu |
|---|---|
| `w62`–`w63` | exportação de `8 min 17 s` → `6,4 s`, e a mensagem aparece; sem congelar a janela |
| `w65`–`w66` | o selo `ISO`; a exportação diz onde a peça está |
| `w67`–`w69` | teto de `Resolution` a 64; girar deixou de pesar |
| `w70`–`w72` | girar `2,6×` mais rápido |
| `w73` | ao parar, a peça fica lisa `3,8×` mais cedo |
| `w74` | duas formas escolhidas → duas peças, cada uma ligada ao seu desenho |
| `w75` | superfície contínua com filetes encadeados |
| `w79` | `Mirror X/Y/Z` |

### ⚠️ O que NÃO foi smokado

- **`w76` (religar escultura)** — o Enio recebeu os passos e respondeu *«ok»* sem confirmar o
  percurso completo (gravar → renomear o ficheiro → reabrir → `Relink Sculpture…`). **Vale um smoke
  na árvore integrada.**
- **`w77`, `w78`, `w80`** — não têm produto visível (medição, auditoria e um gate derivado).
- ⚠️ **A caixa de diálogo do religar não é alcançável por teste** (precisa de um app); o que os gates
  provam é o pedido e a escrita da chave.

### Smokes do módulo (o comando é o mesmo; muda a env)

```
cd /home/enio/Documentos/Projetos/PH2D && cargo run -p ph2d-host-desktop --release
```

pill **MODEL** · `PH2D_FIELD_SMOKE=<n>` · `PH2D_RETOPO_EXTRACT=1` (da linha irmã).
⚠️ **`~/.ph2d/prefs.txt`:** um `reduced_motion=1` esquecido reprova smokes sobre produto correto.

---

## §7 — Riscos conhecidos que o integrador deve olhar

1. ⚠️ **`ph2d-field-eval/src/lib.rs` foi PARTIDO** (`lib.rs` 642 + `step.rs` 143). Se outra linha
   tocou nesse arquivo, o merge textual vai parecer maior do que é: o conteúdo movido é o bloco
   contíguo do `safe_march_step`/`inflation_depth`.
2. ⚠️ **`shells/desktop/src/field3d_scene_panel.rs` foi PARTIDO** (`panel.rs` 461 +
   `field3d_scene_acts.rs` 160) — o mesmo aviso, e os consumidores mudaram de caminho
   (`panel::acts_for` → `acts::acts_for`).
3. ⚠️ **`shells/desktop/src/field3d_preview_tests.rs` foi PARTIDO** (+ `field3d_preview_cost_tests.rs`).
4. ⛔ **Dois `panic` do `ph2d-gridmap` continuam abertos e são da `line/quadextract`** (reprodutores
   no `CLAUDE.md` §5): `solve.rs:336` com alvo grosso sobre `uv_sphere(48,32)`, e o do cubo
   subdividido. **Não os toquei.**
5. ⚠️ **Um gate de RAZÃO desta crate é sensível a carga:**
   `an_abandoned_march_returns_nothing_and_returns_fast` (ph2d-field-render) reprovou a `load 16` e
   passou **3 de 3** sozinho. É a família documentada no `CLAUDE.md` §5.0 — *re-rode sozinho antes de
   suspeitar do merge*.

---

## §8 — A linha para o `CLAUDE.md §5` (**UMA**, e a narrativa fica aqui)

⚠️ Eu **já editei** a linha `Aberto:` do módulo nesta worktree (o `CLAUDE.md` está no diff). Se a
fusão der conflito ali, a versão a manter é a que contém, na entrada **3D Modeling**:

> ⭐⭐ **A jornada de 26/08 (w70–w80)**: girar ficou **2,6×** mais rápido (três em cada quatro fitas
> montadas por quadro não eram avaliadas por ninguém; `SLABS` 2→4; o anti-serrilhado sai do quadro de
> movimento) e o assentar virou uma **escada** de dois degraus — o alisamento chega `3,8×` mais cedo.
> ⛔ **E o passo da marcha estava ERRADO**: arredondamentos exactos **encadeados** compõem o factor
> (e um nó de `n` filhos já é uma corrente de `n−1`) — a cena 1 do smoke marchava acima do seguro
> desde que existe; o passo passa a ser `1/√2^k`, e a escultura foi medida (`1,0852`, `30 %` de
> folga). ⭐ `Mirror` passa a ter **três eixos**; duas formas escolhidas viram **duas peças**, cada
> uma ligada ao seu desenho; e uma escultura que perdeu o ficheiro tem **`Relink Sculpture…`**.
> ⚠️ **Seis notas deste módulo estavam desactualizadas** contra o código, e **dois gates prometiam
> «erro de compilação» sobre listas escritas à mão** — os dois estão derivados agora
> ([handoff de 26/08](docs/3DModeling/handoffs/HANDOFF_INTEGRACAO_line_3DModeling_2026-08-26.md)).

---

## §9 — Cinco coisas que uma leitura rápida do diff entende ao contrário

1. **O `MAX_PROFILE_RESOLUTION` mexeu-se três vezes** (`16 → 64 → 32 → 64`) e **cada passo estava
   certo com o que se sabia**: o 1.º media o relógio errado (o quadro assente, pago uma vez, em vez
   do de movimento), o 2.º media o certo **sem** a cura, o 3.º é o mesmo relógio **com** ela. A razão
   de cada degrau está no doc da constante.
2. **A `wants_antialias` nasceu na `w72` e MORREU na `w73`** — 24 horas. Não é indecisão: a pergunta
   dela (*«o tamanho pedido é o cheio?»*) **não sabe exprimir a escada** que o report do Enio
   obrigou a construir. A resposta passou a viajar com o pedido.
3. **A `w72` acrescentou um parâmetro a `march_slabs` e a `w72`-revert tirou-o.** O que ficou no
   `march.rs` é a cura que **paga** (não forkar a fita que se acabou de montar); o que saiu é a
   tentativa de reaproveitar o avaliador na 2.ª passagem, **medida neutra** (`0,97×`–`1,01×`) em duas
   formas. As duas recusas estão no doc do `EDGE_CHUNK`.
4. **`SLABS = 4` não é um palpite:** o `2` era o óptimo *quando montar custava o dobro*. A varredura
   nova é intercalada e está no doc da constante — junto com a antiga, que **não estava errada**.
5. **A `w74` faz N peças e não uma peça de N contornos**, e isso foi decidido pelo **vínculo vivo**:
   o `FieldProfileSource` aponta para **um** desenho. Uma peça de N contornos perderia o vínculo de
   `N−1` deles, ou obrigaria o componente — que viaja no arquivo — a mudar de forma.

---

## §10 — Três premissas minhas que a implementação REFUTOU

1. *«a montagem é `79 %` do quadro»* — era uma **divisão**, não uma medição; medida, é **`20 %`**
   (§72.1). Ela sustentava duas direcções de trabalho que ficaram com tecto de `20 %`.
2. *«a sobre-relaxação da marcha compra alguma coisa»* — a marcha já dá **`8,7` amostras por pixel**,
   quase o mínimo de uma esfera-marcha com normal. O custo é **por aresta tocada**, não por passo.
3. *«a escultura não tem medição do gradiente»* — **tinha**, num gate estreito (uma esfera, uma
   banda). O que faltava era a generalização; o pior caso real é o **cubo**, `1,0852`.

---

## §11 — Estado da worktree

- `git status` limpo, HEAD `4e359b17d`.
- ⚠️ **`rm -rf target/*/incremental` corrido** (DIRETRIZ §1.5.9 item 7).
- **Não integrei, não fiz rebase, não pushei.** Aguardo ordem.

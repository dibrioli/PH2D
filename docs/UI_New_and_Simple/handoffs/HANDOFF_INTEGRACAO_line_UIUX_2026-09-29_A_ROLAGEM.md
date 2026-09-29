# `line/UIUX` · 2026-09-29 · **HANDOFF DO INTEGRADOR — a rolagem única, com inércia**

> Ordem do dono (29/09): *«vamos para Rolagem com inércia. O Scroll e a barra de scroll não estão
> padronizados e centralizados e deste modo cada painel tem um comportamento diferente do outro.
> Alguns painéis o scroll nem funciona. vamos corrigir e padronizar para todos»*.
>
> O desenho, o censo e os doze defeitos com endereço vivem na spec
> [`04_a_rolagem_unica.md`](../spec/04_a_rolagem_unica.md) (§2 os defeitos, §3 o desenho, §4 as
> waves com o estado, §4.1 o que a implementação mudou). Aqui fica só o que a fusão precisa.
>
> ⛔ Integrar e enviar são **ordem do dono** (§0.7). Esta linha fechou e **parou**.

---

## §0 — Em uma página

| | |
|---|---|
| ramo | `line/UIUX` · worktree `Worktrees/line-UIUX` |
| merge-base | `912a9652e` (= o `main` de hoje: **o `main` não andou** desde que a linha nasceu) |
| commits | **5** (`git log --oneline main..line/UIUX`: W1, W2 ×2, o fecho W2–W6 e este handoff) · `122` ficheiros |
| contadores partilhados | **ZERO** (`PROJECT_SCHEMA`, os três registos, os schemas — §1) |
| **número CONTADO que soma entre linhas** | ⚠️ **dois ids de barra de rolagem**: `SKELETON_SCROLLBAR_ID = NodeId(848)` e `CMD_PALETTE_SCROLLBAR_ID = NodeId(849)` — **próximo livre `850`** (§2.3) |
| contrato congelado (§6) | **intocado** |
| ADR | **nenhum** |
| pacote externo novo | **nenhum** |
| a fusão | `--ff-only` enquanto o `main` não andar |

**O que a linha entrega:**

1. **Uma porta para toda lista que rola** — `widget::scroll_area` (partes soltas: `open_with` ·
   `close_with` · `close_parts` → `Pending`) e `panel::scroll_area` (as formas com `PaintCtx`:
   `open` · `close` · `close_showcase_body`). Ela recorta o desenho **e o clique** pelo corpo
   visível, publica as duas alturas, pinta a barra, regista a TRILHA e **publica o dono da barra**.
   **28** chamadas: 25 painéis (a galeria incluída; o Painter com as duas vistas), a coluna de
   catálogos do navegador, a janela do Input Map e a paleta de comandos.
2. **A barra arrasta à velocidade do dedo e salta onde se clica na trilha** (D1: a altura lida era
   a do polegar, não a da trilha).
3. **A roda não tem lista**: `cursor_over_hero_panel` é `panel_at(x, y).is_some()`, e `panel_at`
   respeita a **ordem z** (os rects fora dela — overlays — primeiro). Uma lei só:
   `WidgetStore::wheel_panel`.
4. **Inércia**: arrastar o corpo é **1:1** e soltar com velocidade **desliza** (constantes do AOSP
   e do `UIScrollView`, módulo `interaction::fling`).
5. **A tabela à mão `scrollbar_panel_for_id` foi APAGADA** — o dono de uma barra é o que a porta
   publicou.
6. **Painéis que não rolavam passam a rolar**: Ossos (pintava com o id do Vector e não estava na
   lista da roda), Hierarquia (a barra não arrastava), Input Map e paleta (barras sem dono).

---

## §1 — Superfície de colisão (colada do script, não escrita à mão)

```text

SUPERFÍCIE DE COLISÃO — line/UIUX contra main
  merge-base 912a9652e   ·   4 commit(s)   ·   122 arquivo(s)
───────────────────────────────────────────────────────────────────────────────
▸ SCHEMAS — ⚠️ o valor se CONTA contra o main do dia; confira nos TRÊS sítios
    PROJECT_SCHEMA                        176   (base: 176)
      └ tripla do gate               (176, 13, 22)   (base: (176, 13, 22))
    VEC_SCENE_SCHEMA                       22   (base: 22)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)
    FIELD_DOC_VERSION                      23   (base: 23)
  ⚠️  esta linha TOCA project*.rs — a escada e a tripla moram em arquivos IRMÃOS;
      um degrau escrito no arquivo errado funde LIMPO e evapora.

▸ REGISTRO DE COMPONENTES — o contador é TRÊS, cada um roda só na suíte da própria crate
    ph2d-ecs                              108   (base: 108)
    ph2d-render (espelho)                 109   (base: 109)
    ph2d-script (espelho)                 109   (base: 109)

▸ CONTRATO CONGELADO (§6) — deve ser INTOCADO; se não, exige ADR
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado

▸ ADR — número escolhido numa linha paralela é PROVISÓRIO
    último no disco: 0175   próximo livre: 0176
    esta linha não cria ADR ⇒ fora de toda disputa de número

▸ Cargo.lock — pacote EXTERNO novo é o que importa; aresta interna não
    nenhum '+name' novo

▸ MARCADORES DE CONFLITO — inclui '|||||||' (diff3), que uma varredura de 3 marcadores NÃO vê
    nenhum nos arquivos da linha

▸ TETOS DE LOC nos arquivos que a linha tocou (700 workspace · 600 painel/shell · 500 widget · 650 tool-runtime)
    nenhum arquivo da linha passa do teto
───────────────────────────────────────────────────────────────────────────────
  ⚠️ Isto é o MAPA, não o gate. O gate mecânico é scripts/foundational-integrate.sh;
     o que exige julgamento (mesmo-símbolo, decisão de produto) continua leitura humana.
```

Nenhum contador partilhado mexe. ⚠️ A coluna `base:` é o MERGE-BASE; se outra linha fundir antes, releia os valores no ficheiro.

---

## §2 — O que PARTE noutra linha ao fundir (leia antes de fundir a segunda)

### §2.1 — Assinaturas públicas que MUDARAM (erro de compilação noutra linha)

| item (`ph2d-editor-core`) | antes | agora |
|---|---|---|
| `screens::hero::chrome::paint_input_map_window` | `-> Option<Rect>` | `-> Option<(Rect, widget::scroll_area::Pending)>` — quem chama **publica** o `Pending` e o rect com `ids::INPUT_MAP_SURFACE` |
| `screens::hero::chrome::paint_command_palette` | `()` | `#[must_use] -> Option<Pending>` — quem chama publica |
| `widget::command_palette::paint` | sem `store` | `+ store: &WidgetStore`, `-> Pending` |
| `widget::showcase::paint_showcase_body` | pintava tudo | `-> ShowcaseBody`, fechado por `panel::scroll_area::close_showcase_body` (ou `close_showcase_body_with`) |
| **REMOVIDOS** | `command_palette_max_scroll` · `WidgetStore::{command_palette_scroll, scroll_command_palette, input_map_scroll, scroll_input_map}` · `scrollbar_panel_for_id` | a rolagem destes vive nas tabelas de todo painel, com a chave `CMD_PALETTE_CARD` / `INPUT_MAP_SURFACE` |

⚠️ Uma linha que chame os removidos **não compila** depois da fusão — a cura é a porta, nunca
repor o campo.

### §2.2 — A regra nova que um PAINEL NOVO de outra linha tem de seguir

⛔⛔ **Uma barra de rolagem registada à mão (`register(<X>_SCROLLBAR_ID, thumb)`) deixou de
arrastar**: o despachante já não tem tabela à mão, só o dono **publicado**
(`WidgetStore::publish_scroll_bar`, que a porta chama). Um painel que outra linha escreveu com a
forma antiga **compila, pinta a barra e ela fica morta sob o dedo** — o defeito mudo. Censo que o
apanha: `every_panel_that_scrolls_publishes_the_rect_the_wheel_reads` (`shells/desktop/tests/it/`,
piso de `20` crates que abrem a porta). Cura: `ph2d_editor_core::panel::scroll_area::open`/`close`.

### §2.3 — Números CONTADOS

`SKELETON_SCROLLBAR_ID = 848` e `CMD_PALETTE_SCROLLBAR_ID = 849`. Se outra linha escreveu `848` ou
`849`, os dois **compilam** — a colisão passa MUDA, e quem a apanha é a lista de unicidade em
`widget/scrollbar_ids.rs`. Reconte contra o `main` do dia; o próximo livre desta linha é `850`.

### §2.4 — Réguas cujo número MUDOU (medido, só encolhe)

| régua | antes | agora | porquê |
|---|---|---|---|
| `the_foundation_modules_form_a_dag`, `widget → interaction` | `49` | **`46`** | a paleta passou a receber o `WidgetStore`; os testes dela deixaram de nomear o `HitIndex` que já vinha pelo `use super::*` |
| janelas dos censos de `ph2d-panel-registry-init` | `4000`–`12000 px` | **`16000`** (a curta `8000`) | a porta recorta o CLIQUE: um censo que conta o índice de acerto tem de pintar o painel inteiro |
| `onde_comeca_o_valor` | pintava uma vez | **puxa o canto de toda janela que transborda** e pinta outra vez | a galeria é flutuante e pára em `720 px`; lia `1` coluna onde há `3` |
| `the_painted_control_reaches_a_consumer` | — | `set_sub_scroll_region` / `clear_sub_scroll_region` em `PANEL_CALLS` | a sub-região é lida por POSIÇÃO; quem a nomeava era a tabela apagada |

⚠️ **Se outra linha fundir antes desta**, as duas primeiras SOMAM: a catraca do DAG conta
referências de todo `widget/**`, e um painel novo pintado pela porta com um censo numa janela baixa
lê-se como controlos a menos.

---

## §3 — Fundação tocada, e porque é ADITIVA (ou onde não é)

- **Aditivo:** `interaction::fling` (módulo novo), `ScrollState` (tabelas novas no `WidgetStore`),
  `widget::scroll_area`, `panel::scroll_area`, `wheel_panel`, `publish_scroll_bar`.
- **Não aditivo:** a semântica de `panel_at` (agora pela ordem z, overlays primeiro) — um painel que
  dependesse da ordem do `BTreeMap` por id passa a ver o de cima; os removidos do §2.1.
- **A timeline fica FORA de propósito:** a roda nela é zoom do eixo do tempo, um gesto de canvas.

---

## §4 — Smoke (comandos da WORKTREE)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && cargo run -p ph2d-host-desktop --profile smoke
```

O que olhar: arrastar o corpo de qualquer painel e soltar depressa (desliza) · arrastar a barra
(segue o dedo) · clicar na trilha (salta) · roda sobre o painel de Ossos (rola; a câmera não faz
zoom) · barra da Hierarquia · *Settings → Input Map…* (barra e roda) · paleta de comandos (tem
barra).

---

## §5 — O portão do fecho (MEDIDO, sobre o diff acumulado)

Tudo na worktree, pela porta `scripts/ph2d-run.sh`, com a máquina a `load 38`–`73` (outras linhas a correr):

| portão | resultado |
|---|---|
| `nextest-impacted.sh` sobre o diff acumulado | **16 373 / 16 373** verdes |
| `cargo clippy -p <as 28 crates tocadas> --all-targets -- -D warnings` | **verde** (apanhou dois avisos desta linha — um `#[must_use]` duplo e um `&format!` —, curados) |
| `censos-da-arvore-combinada.sh` | **127 / 127**, controlo do filtro **12 de 12** |
| `cargo fmt --all` | limpo |

⚠️ **O `ship.sh` NÃO foi corrido** — é do integrador/ship, por ordem do dono (§0.7).

---

## §6 — Rebase e árvore combinada

- O `main` **não andou** desde o merge-base (`912a9652e`) ⇒ nada a rebasear.
- `bash scripts/censos-da-arvore-combinada.sh`: **127 / 127 verdes**, controlo do filtro **12 de 12**.

---

## §7 — A UMA linha proposta para o `CLAUDE.md` §5 (o integrador aplica)

Na entrada **UI/UX**, a seguir à linha de 25/09:

> ⭐⭐⭐ **A ROLAGEM É UMA** (29/09, [handoff](docs/UI_New_and_Simple/handoffs/HANDOFF_INTEGRACAO_line_UIUX_2026-09-29_A_ROLAGEM.md),
> [spec](docs/UI_New_and_Simple/spec/04_a_rolagem_unica.md)): toda lista que rola passa pela porta
> `panel::scroll_area` (recorta desenho E clique, publica as alturas e o DONO da barra), a roda não
> tem lista (`panel_at` pela ordem z) e o corpo desliza com inércia. ⛔ Uma barra registada à mão
> **fica morta sob o dedo** — a tabela `scrollbar_panel_for_id` foi APAGADA. ⏳ D9 (alturas velhas).

---

## §8 — O que fica ABERTO

- **D9:** um painel que publica o rect sem `content_h` deixa alturas velhas (o caminho de cena vazia
  do Sculpt3d).
- **O painel da escultura** é território da `line/sculpt3d`: a migração dele foi **uma chamada**
  (a porta), e se aquela linha reescrever o `paint` ela tem de continuar a passar por lá (§2.2).
- **A timeline** fica fora por desenho (§3).

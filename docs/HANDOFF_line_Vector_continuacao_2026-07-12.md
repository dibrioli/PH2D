# HANDOFF de CONTINUAÇÃO — linha `line/Vector` (Live Shapes: fechar o ciclo paramétrico)

> **Para o agente que assume a linha.** A rodada anterior **foi integrada no `main`** e a linha foi
> **re-preparada** (branch = tip do `main`, worktree limpa). Este doc é o teu ponto de partida:
> como operar (Modo L), o que já landou, e as **etapas planejadas** na ordem.
>
> Handoff da rodada integrada (contexto histórico):
> [`HANDOFF_line_Vector_integracao_2026-07-11.md`](HANDOFF_line_Vector_integracao_2026-07-11.md).

---

## 0. Estado — a linha está PRONTA, é só começar

| | |
|---|---|
| **Worktree** | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector` |
| **Branch** | `line/Vector` |
| **HEAD** | `3805f650` = **tip do `main`** (a rodada anterior já está lá, rebasada pelo integrador) |
| **Árvore** | **limpa** (0 arquivos pendentes) · `main..line = 0` · `line..main = 0` |
| **Tier** | `workstation` ⇒ **Modo L** (confirmado por `bash scripts/hw-profile.sh`) |
| **Baseline** | `cargo check --workspace` **verde** · testes vetoriais **55/55** · `ComponentRegistry` = **26** |

⚠️ **Nada de `git worktree add`** — a worktree **já existe** (rota "linha reaberta" do
[MODELO_ABERTURA_LINHA §4](IntegracaoMultiAgente/MODELO_ABERTURA_LINHA.md)). Só entre nela e trabalhe.

---

## 1. Como operar — Modo L (MODELO_ABERTURA_LINHA + DIRETRIZ §1.5)

**Regras permanentes da sessão** (do bloco de abertura — valem até o fim, sem exceção):

- **A. TUDO dentro da worktree.** `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector` em
  **todo** comando. A raiz do repo é o checkout primário (`main`) e **o mesmo path relativo existe
  nas duas árvores** — editar `crates/…` a partir da raiz é editar a árvore **ERRADA**.
  **Isto morde de verdade:** o `cwd` do Bash *reseta pra raiz* depois de um `/compact`, e o
  `target/` da raiz é um symlink quebrado (o cargo falha com *"Not a directory"* — é o sinal).
  ⇒ **`cd` absoluto em TODA chamada de shell**, e **caminho absoluto em toda mutação** (`sed -i`, `>`, `mv`)
  — ver [[feedback_sed_relative_path_hits_primary_cwd]].
- **B. Foundational você PODE e DEVE tocar** (ph2d-ecs / editor-core / shells / …) sob o protocolo
  testado (ADR-0107). **PARE e reporte ao Enio SÓ se:** (a) for **contrato congelado**
  (CLAUDE.md §6 — exige ADR), ou (b) o rebase conflitar **fora dos teus arquivos** (colisão de
  mesmo-símbolo). **Nunca** negocie com outra linha.
- **B'. Ao CRIAR foundational novo, projete para ISOLAMENTO** — módulo/arquivo **irmão** novo em vez
  de engordar arquivo compartilhado; ponto de extensão **append-only**. Todo id/const/variant novo:
  **anote no handoff** (regra H) pro integrador detectar colisão. *(Foi assim que o `VecShape` entrou:
  arquivo novo `ph2d-ecs/src/vec_shape.rs`, só primitivos, registrado numa lista append-only.)*
- **C. Commits locais frequentes:** `git commit --no-verify -m "msg" -- <paths>`.
  **NUNCA** `push` · **NUNCA** `--force` · **NUNCA** `git add -A`.
- **D. `git rebase main`** no início da jornada e antes de integrar. Conflito em `Cargo.lock` ou
  arquivo **GERADO**: **refaça pela geração**, não resolva à mão.
- **E/F. Você NÃO integra e NÃO faz ship.** Fecha o módulo com o **gate batched**, escreve o
  **handoff (§1.5.9)** e **PARA**. Integração/ship = **ordem EXPLÍCITA do Enio**, por um **agente
  integrador dedicado**.
- **G. UI canônica:** zero hex, zero `f32` literal de UI, zero string hardcoded — tokens/i18n
  (CLAUDE.md §0.3). **Labels da UI em inglês**; comentários pt-BR ok.
- **H. Handoff de integração** é entregável **obrigatório** no fechamento.

**Ritmo (o que funcionou):** inner loop = **só `cargo check -p <crate>`**. Teste/clippy/gates
**1× no fechamento** de cada incremento. **Não commite antes do smoke** — o Enio smoka cada
incremento e diz "smoke ok"; **só então** faça o commit ([[feedback_smoke_at_end]]).

**A cada passo, releia** [`DIRETIVA_IMPLEMENTACAO.md`](IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md).
Regra-mãe: **verde-de-compilação é velocidade; no audit vale ZERO.**

---

## 2. O que já landou (o alicerce que você vai estender)

**Live Shapes** — o modelo Figma/Illustrator: toda forma é um **objeto paramétrico vivo** até virar
curva. Leia estes 4 arquivos antes de codar; são o coração:

| Arquivo | O quê |
|---|---|
| `crates/ph2d-ecs/src/vec_shape.rs` | **`VecShape`** — enum com os PARÂMETROS (`Rectangle{w,h}` · `RoundRect{w,h,radius}` · `Ellipse` · `Polygon{…,sides}` · `Star{…,points,inner_ratio}` · `Spiral{…,turns}` · `Line` · `Arc{…,degrees}` · `Text(VecTextParams)`). Só primitivos; registrado no `ComponentRegistry` ⇒ **undo/save de graça**. |
| `shells/desktop/src/vec_shape_live.rs` | `recook_shape` (params → geometria **centrada**) · `recook_into` (troca a geometria in-place, preserva id+estilo) · `make_committed_shape_live` (a forma **nasce viva** ao soltar o gesto) · `drop_shape_params` (Convert de paramétrica = descartar o `VecShape`). |
| `shells/desktop/src/vec_text_object.rs` | O lado OBJETO do texto: `upsert_text_shape` · **`panel_text_target`** (o ALVO do painel) · **`edit_selected_text`** (edita+re-cook o objeto selecionado) · `convert_text_selection_to_curves` (explode em grupo por-letra). |
| `shells/desktop/src/vec_text.rs` | A SESSÃO de digitação (estado + `regen_into` + os `apply_text_*`). |

### As 3 invariantes que você NÃO pode quebrar

1. **Cook-centered.** A geometria de uma forma viva **nasce centrada no local 0** (= o **pivô**), e o
   `Transform` da entidade guarda a pose. ⇒ o pivô fica **no centro do objeto** (rotaciona em torno
   dele) e o **re-cook é idempotente**: preserva a pose e o *move* do usuário. Por isso
   `settle_origins` **pula entidades com `VecShape`** (`vec_transform.rs`).
   *No texto:* `Transform = origin + center` (a baseline fica no clique **e** o pivô no centro; o
   `center` cancela no caret — não "conserte" isso).
2. **Re-cook IN-PLACE** (`scene.path_mut(id)`, id preservado). Se você remover+repush o path, a
   entidade despawna/respawna e **o `VecShape`, o gizmo e a seleção piscam a cada tecla**.
3. **O ALVO do painel.** Uma config do painel age sobre: a **sessão viva**; sem ela, o **objeto
   selecionado**. Ver `panel_text_target` + `edit_selected_text`. **É o padrão que a Etapa 1 replica.**

### Gotchas que já custaram caro (não repita)

- **Undo:** uma sessão viva **reescreveria o `VecShape`/pose a cada frame** ⇒ *desfaria o undo no
  frame seguinte*. Por isso `undo.rs::apply_project` (restore) **encerra a sessão de texto**.
  Qualquer estado "por-frame que reescreve o mundo" tem esse risco — **teste Ctrl+Z**.
- **Semente de slider:** o painel semeia os sliders **one-shot, só quando o ALVO muda**
  (`set_current_text_seed` / `take_text_seed`). Semear todo frame **briga com o arrasto** do slider.
- **HR-18 (600 LOC/arquivo):** `vec_glyph`/`vec_text` estouraram e foram **divididos** (`vec_glyph_build`,
  `vec_text_object`), **não allowlist**. `fmt` **re-expande** ⇒ **rode fmt ANTES de medir** LOC.
- **`typos` reprova pt-BR em COMENTÁRIO DE CÓDIGO** (`.rs`) — ex.: `Descritor`→"Descriptor",
  `Prefere`→"Prefer". Reformule a frase **ou** adicione a palavra ao `.typos.toml`
  (`[default.extend-words]`, que já tem dezenas de palavras pt-BR). ⚠️ **`docs/**/*.md` é EXCLUÍDO**
  pela config — então **doc em pt-BR passa liso**. E rode `typos` **sem argumento** (project-wide,
  como o `ship.sh` faz): `typos <arquivo>` explícito **ignora o exclude** e dá falso-positivo.
- **`cargo nextest ... | grep`** mascara o exit code ⇒ **verifique o ESTADO**, não o `$?`
  ([[feedback_pipe_masks_script_exit_code]]).

---

## 3. ETAPAS PLANEJADAS — siga nesta ordem

> Cada etapa: implementa → **gate** (`cargo check -p` no loop; no fim `nextest` + `clippy --all-targets`
> + `fmt` pin 1.95 + `typos`) → **apresenta pro smoke do Enio** → **só faz o commit após "smoke ok"**.

### Etapa 1 — Sliders de forma editam a shape VIVA selecionada  ⭐ *(o mais valioso — comece por aqui)*

**Problema:** os sliders **Sides / Points / Inner / Radius / Turns / Degrees** ainda agem só no
*default de desenho* (`VectorTool`), **não** na shape viva selecionada. Ou seja: você desenha um
polígono de 5 lados e **não consegue mudar pra 7 depois**. **É o que falta pra "live shape" fazer
sentido pleno** — hoje o texto já faz isso, as formas não.

**Como (o padrão já existe — espelhe o texto):**
1. **Alvo:** um `panel_shape_target(sim, map, selection) -> Option<(VecPathId, Entity, VecShape)>`
   em `vec_shape_live.rs` (espelho de `vec_text_object::panel_text_target`).
2. **Publicar** ao painel: os params da forma selecionada + **qual variante** ela é (pra seção certa
   aparecer, mesmo na ferramenta **Select** — hoje as seções são keyed por `snap.mode`). Espelhe
   `set_current_text_visible` / `set_current_text_seed`, com a mesma **semente one-shot** quando o
   alvo muda.
3. **Aplicar:** no drain do `render_loop`, quando **não** há gesto de desenho e há shape viva
   selecionada, o `SetValue` do slider muta o `VecShape` e chama **`recook_into`** (que já preserva
   id/estilo/pose). Espelhe `edit_selected_text`.
4. **Cuidado:** o `RoundRect.radius` é guardado em **mundo** (foi autorado em px × `px_to_world`) —
   o slider é em **px**; converta na fronteira.
5. **Testes:** re-cook de `Polygon{sides:5→7}` troca a contagem de âncoras e **mantém a bbox
   centrada** + o `Transform` intacto.

**Smoke:** desenhe um Polygon → Select → clique nele → mexa em **Sides** → a forma re-cozinha **no
lugar** (não pula, não perde o pivô).

---

### Etapa 2 — Resize pelo gizmo reescreve os PARÂMETROS (`w`/`h`), não o `Transform.scale`

**Problema:** hoje o gizmo escala a **pose** (`gizmo_drag.rs` escreve `Transform.scale`). Numa live
shape isso está **errado**: o `RoundRect.radius` escala junto (o correto, à la Figma, é o raio ficar
**constante em px**), e os campos W/H do painel passam a mentir (mostram a base, não o efetivo).

**Como:** **bake-on-release** (espelhando a filosofia do `settle_origins`): ao **soltar** um resize de
entidade com `VecShape`, multiplique o `scale` nos params (`w *= scale.x`, `h *= scale.y`), **resete
`scale` para 1** e **re-cozinhe**. Durante o arrasto, o `scale` do `Transform` dá o preview barato.
O `radius`/`inner_ratio`/`sides` **não** escalam ⇒ é exatamente o comportamento Figma.
Reuse `vec_gizmo_view::anchor_half` (já dá o retângulo local exato) e o snap de escala vetorial que
já existe (`vec_scale_ids` em `gizmo_drag.rs`).

**Smoke:** redimensione um RoundRect → o **raio dos cantos NÃO deforma**; os campos W/H batem.

---

### Etapa 3 — Reabrir a STRING de um texto finalizado (duplo-clique)

Hoje as **propriedades** de um texto já são editáveis na seleção (Etapa 0, feita), mas a **string**
não. Falta: duplo-clique (ou Enter no modo Text com o objeto selecionado) **reabrir a sessão**
ligada àquele objeto — carregar o `VecShape::Text` de volta num `VecTextEdit` (o `id` do compound, o
`origin`, os params) e voltar a digitar/mostrar o caret.

**Cuidado:** o `origin` guardado é a **baseline em mundo**; a sessão recria o `Transform = origin +
center` a cada frame ⇒ se o usuário **moveu** o texto, reabrir precisa **respeitar a pose atual**
(deduzir o `origin` do `Transform` atual em vez de sobrescrever a pose). É a parte sutil.

---

### Etapa 4 — Virtualizar as previews do dropdown de fonte (perf)

A **1ª abertura** do dropdown carrega+parseia **todas** as fontes do sistema de uma vez (hitch único;
as seguintes são cacheadas). Virtualize: o painel publica a **faixa visível** e a shell constrói
preview **só dessas linhas** (handshake de range, 1 frame de lag — mostre o nome no fallback).
O handshake já existe em embrião: `take_want_font_previews`.

---

### Etapa 5 — (menor) Limpezas nomeadas

- `vec_history` está **morto** (a geometria vive no undo global) — ainda é populado e não lido. Limpe.
- `vec_save` não serializa pose/nome/parentesco (gap **pré-existente**, CLAUDE.md §5) — a persistência
  real é do `project.rs`; só confirme que a **forma viva** (`VecShape`) sobrevive a Ctrl+S/Ctrl+O.

---

## 4. Referência rápida (linha de comando)

```bash
W=/home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector

# inner loop
cd $W && cargo check -p ph2d-host-desktop

# gate de fechamento de incremento
cd $W && cargo nextest run -p ph2d-ecs -p ph2d-vec-scene -p ph2d-vec-edit -p ph2d-vec-render \
  -p ph2d-tool-vector -p ph2d-panel-vector -p ph2d-editor-core -p ph2d-host-desktop
cd $W && cargo clippy -p ph2d-panel-vector -p ph2d-host-desktop --all-targets
cd $W && rustup run 1.95 rustfmt --edition 2024 <arquivos>   # fmt ANTES de medir LOC
cd $W && cargo fmt -p ph2d-host-desktop --check && typos <arquivos>

# smoke (SEMPRE com o cd junto)
cd $W && cargo run -p ph2d-host-desktop
```

**Arch-gates que essa feature costuma acender:** `file_loc_caps` (HR-18) · `node_id_collisions`
(lista `CHROME_IDS` hand-maintained) · `architecture_panel_{wiring_parity,loc_cap}` ·
`no_{magic_numeric,literal_color,tofu_glyphs}` (escaneiam `ph2d-panel-*`).

---

*Linha `Vector` re-preparada em `3805f650` (= tip do main). Aguardo a tarefa — ou comece pela
**Etapa 1**, que é a que fecha o ciclo paramétrico.*

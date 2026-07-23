# HANDOFF DE INTEGRAÇÃO — `line/Vector` · Texto em caminho (W0–W5)

> **Regra H.** Entregável de fechamento. A linha está **FECHADA**; NÃO integra nem faz ship
> sozinha. Aguarda ordem EXPLÍCITA do Enio, via agente integrador dedicado.

## 1. Branch / HEAD / base

- **Branch:** `line/Vector`
- **HEAD:** `94dcc8c46`
- **Base (merge-base com `main`):** `13a04c7aa`
- **13 commits** (o 1º é a pesquisa+plano; os demais W0–W5 + docs). Árvore limpa.

## 2. O que a linha entrega

Texto que **cavalga uma curva** (SVG `<textPath>` / *Type on a Path* do Illustrator, caso A da
pesquisa — glyphs **rígidos**, não deformados). Motor (arco→afim por glyph) + vínculo (componente
ECS) + seção de painel + alça de canvas. Plano e racional completos:
[`docs/Vector Module/22_plano_texto_em_caminho.md`](Vector%20Module/22_plano_texto_em_caminho.md).

**Bônus da W0:** um bug VIVO, independente da feature — o re-cook de texto apagava a pilha de
efeitos (e a geometria de envelope) em silêncio ao editar uma letra. Curado pela porta única
`VecPath::replace_cooked`.

## 3. Foundational tocado, e por quê

| Crate | O quê | Isolamento |
|---|---|---|
| `ph2d-vec-scene` | módulos NOVOS `recook.rs`, `arc_path.rs`, `text_path.rs`; `+closest_arc` em `arc_path.rs` | arquivos próprios; `fx_zigzag` passou a delegar ao `ArcPath` (byte-idêntico, fingerprint pinado) |
| `ph2d-ecs` | componente NOVO `VecTextPath` (`vec_text_path.rs`) | arquivo próprio; registro append-only |
| `ph2d-editor-core` | arquivo NOVO `ids/chrome/vector_textpath.rs`; entrada em `VECTOR_SECTIONS` | ⚠️ `VECTOR_SECTIONS` é lista compartilhada — a entrada foi ao **FIM** (só ADICIONAR) |
| `ph2d-i18n` | 1 chave `panel.vector.section.textpath` | tabela compartilhada, só ADICIONAR |
| `ph2d-panel-vector` | arquivos NOVOS `paint_textpath.rs`, `state_textpath.rs`; `+track_slider_event` em `event.rs` (extração de 5 braços do `apply_event`, teto de LOC) | seção nova, isolada |
| `ph2d-vec-render` | arquivo NOVO `text_handle.rs` (ficha grande + colorida) + re-export | isolado |
| `ph2d-render`, `ph2d-script` | **só o contador de componentes** (`34→35`) | ver §5 |
| `shells/desktop` | arquivos NOVOS `vec_text_ride.rs`, `text_path_smoke.rs`, `text_path_gesture_smoke.rs`; edições em `vec_glyph*.rs`, `vec_text*.rs`, `render_loop/mod.rs`, `input_dispatch.rs`, `vec_overlay.rs`, `app_state.rs`, `main.rs`, `build_smoke.rs` | a shell é da linha |

## 4. IDs / consts / variants novos (regra H — colisão)

Todos os ids de UI são **hash de string** (colisão é de *nome*, não de número). As strings:

```
vector.section.textpath      vector.textpath.link       vector.textpath.detach
vector.textpath.flip         vector.textpath.flip.off   vector.textpath.offset
vector.textpath.offset.num
```

- Componente ECS: `ph2d::vec_text_path` (nome do `SimComponent`; cunha `stable_type_id` própria).
- Const de interação: `vec_text_ride::HANDLE_R_PX = 7.0` (px, medida de INTERAÇÃO, `LITERAL-PX-OK`).
- Smoke levels: **21, 22, 23** (lista compartilhada `build_smoke.rs`).
- i18n: `panel.vector.section.textpath`.

## 5. O que o `ship.sh`/árvore combinada pega e o `cargo test -p` NÃO

⚠️ **O contador de componentes do ECS é TRÊS e os três subiram** (`ecs 33→34`, `render 34→35`,
`script 34→35`). Os de `ph2d-render`/`ph2d-script` só rodam nas suítes deles — verifiquei os três
localmente, mas o **gate da árvore combinada** é a rede final. Se outra linha registrar um
componente antes desta na integração, **os três números se CONTAM, não se escolhem**
(`feedback_numbers_that_sum_across_lines_count_dont_pick`): o valor certo é `base + nº de
componentes das linhas fundidas`, e não está em nenhum lado do conflito.

⚠️ **`VECTOR_SECTIONS`** (lista compartilhada em `ids/chrome/vector.rs`) — a entrada nova
(`VECTOR_SECTION_TEXTPATH`) foi ao FIM. Numa fusão com outra linha que a tenha tocado, **só
ADICIONE, nunca reordene**.

⚠️ **Níveis de smoke 21/22/23** — se outra linha os tomou, renumere os desta (o valor se conta).

## 6. Contratos congelados encostados

**NENHUM.** `VectorOp`/`Vertex`/`Segment`/… (gate `architecture_vector_contract_surface`,
escaneia só `ph2d-vector-doc`+`-traits`) intactos. `NodeOp`/`OpResolver`/`NodeManifest` idem.

**Schema: NENHUM bump.** `VEC_SCENE_SCHEMA_VERSION` e `PROJECT_SCHEMA` intactos — o vínculo é um
componente OPCIONAL (blob-key própria), não campo apendado. (O plano §5.2 previa apender ao
`VecTextParams`, que teria bumpado `PROJECT_SCHEMA` e recusado todo projeto salvo — corrigido.)

## 7. O que smoke-testar

Todos `--release`, na worktree:

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector
env PH2D_BUILD_SMOKE=21 cargo run -p ph2d-host-desktop --release   # W0: efeito sobrevive ao re-cook
env PH2D_BUILD_SMOKE=22 cargo run -p ph2d-host-desktop --release   # W2/W3: o MOTOR (onda + 2 círculos)
env PH2D_BUILD_SMOKE=23 cargo run -p ph2d-host-desktop --release   # W4/W5: o GESTO (prender + alça no SELECT)
```

- **21/22/23 já foram smokados e APROVADOS pelo Enio** nesta jornada.
- A cena 23 **verifica a própria premissa**: se imprimir `PARE e reporte`, a mesa não está posta.

## 8. Verificação local (não é ship — o integrador roda o `ship.sh`)

- Suítes das crates tocadas: **verdes** (ph2d-host-desktop, -vec-scene, -vec-render, -editor-core,
  -panel-vector, -ecs, -render, -script, -i18n).
- `clippy --all-targets` limpo nas crates tocadas.
- Gates de LOC (workspace + painel + shell) verdes.
- **NÃO rodei o `ship.sh` completo** (é do integrador): fmt-skew, machete, deny, audit, typos e a
  **árvore combinada** (o contador de componentes cross-crate) são a rede que só ele fecha.

## 9. Estado: linha PRONTA + handoff. **PARO e espero.**

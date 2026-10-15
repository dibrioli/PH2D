# HANDOFF DE CONTINUAÇÃO — `line/UIUX`, 15/10 — F16 a/b (a ② vira a barra de controle do Illustrator, o traço) feita; próximo: F16c

- Branch `line/UIUX`, worktree `Worktrees/line-UIUX`, base `main` `0aab2c1cb6`. Anterior: [F15](HANDOFF_CONTINUACAO_line_UIUX_2026-10-15_F15_BARRA_DE_CONTROLE.md).
- Plano, respostas do dono verbatim e o mapa da F16c: [`26_plano_F16_barra_de_controle.md`](../vetor_nova_ui/26_plano_F16_barra_de_controle.md) (§4b = F16c).

## 1 — Commits desta janela

| commit | o quê |
|---|---|
| `762e3373ed` | doc 26: plano + 4 perguntas ao dono (sem código) |
| `c35be32de1` | F16 a/b: lista completa na mensagem do commit (`git show -s c35be32de1`) |

Respostas do dono (15/10): só o Vetor perde os botões genéricos · Snap volta ao painel · popover do traço: «faça o melhor
tendo como referência o Illustrator» · Style (graphic styles) é etapa FUTURA · o Duplicate sai da ④ · o «Live: Off/On» da
secção Boolean do painel fica por ora.

## 2 — Decisões de desenho (o que um merge/leitor pode partir)

- O popover do traço é pintado pelo passe diferido do painel do vetor (`bars_control.rs::paint_bar_popovers`, também no
  ramo painel-escondido com largura `DOCK_W_MIN`), ancorado em `WidgetStore::tool_row_slot` (quadro anterior, publicado por
  `tool_bar::publish_overflow`). É um Dropdown do store (`VECTOR_BAR_STROKE`) com light-dismiss genérico.
- Aberto o popover, a secção Stroke do painel pinta só uma dica: um id, um sítio por quadro (`HitIndex::rect_for` devolve o
  ÚLTIMO registo; um id duplicado faria o arrasto de um slider do painel seguir o rect do popover).
- Profile ▾ mostra o perfil da FORMA SELECIONADA (`vector_bridge_publish.rs`: sem `VecStrokeProfile` ⇒ Uniform), não os
  sliders de autoria do Power Stroke (bug apanhado pela foto: «Custom» numa estrela uniforme).
- Gates novos: ver a mensagem do commit. Catraca de altura de abertura do vetor 880 → 916 (cabeçalho do Snap recolhido). O
  fixture `the_app_never_reshapes_a_still_screen` agora semeia uma forma selecionada (sem ela dava 1005 < teto 1024).

## 3 — Smoke (por fazer pelo dono)

Cena 23: `PH2D_OBJECT_MODE_SMOKE=23` (`crates/ph2d-app-vec/src/vector_mode_controle.rs`, estágios 230–234), foto
[`F16_cena23_barra_de_controle.png`](../vetor_nova_ui/fotos/F16_cena23_barra_de_controle.png). A cena 10 passa a ensinar o
«Stroke ▾» (era o Snap). O `.log` irmão da foto fica não rastreado.

## 4 — ABERTO

1. **F16c**: 3 pares tracejado/vão + escala da seta por ponta (modelo + formato + render + painel + ferramenta + ponte +
   export SVG). ~60 sítios (doc 26 §4b); postcard é posicional ⇒ subir `VEC_SCENE_SCHEMA_VERSION` (hoje 22) e a escada do
   `PROJECT_SCHEMA` (hoje 189); receita no commit `60db072e7b`.
2. Smoke do dono da cena 23.
3. Suíte e clippy NÃO correram desde a última verde — no fecho, `PH2D_SUITE=fecho`.
4. No fecho, CLAUDE.md §5.1 UI/UX: lista de smoke `6..23` e o «Último:».
5. Por medir: `frame_margin()` (F8) conta UMA linha da ④ e a ④ agora tem muitas vezes duas.
6. Órfão pré-existente: `VECTOR_RULERS_OFF/ON` são registados e tratados mas pintados em lado nenhum (as réguas vivem no
   menu View) — decidir a remoção no fecho.

# HANDOFF DE CONTINUAÇÃO — `line/Macros-Automação`, W6 (tutorial «fazer») — 2026-10-15

> Plano e o que se mediu: [03_plano_W6.md](../03_plano_W6.md) (§2 a lei, §4 as ondas, §5 a W6a feita com os gates por
> nome). Antes: [handoff W5](HANDOFF_INTEGRACAO_line_Macros_2026-10-15_W5.md).

## Estado

- HEAD `ceffb4d5c8` sobre o merge-base `0aab2c1cb6` (o `main` não andou). W6a feita e fotografada; **smoke do dono
  pendente** (cena `PH2D_MACRO_SMOKE=3`). Binário `--profile smoke` compilado na worktree.
- Não se reconstrói: a lei do progresso (estado puro), os critérios por Δ (`ph2d-app-motion/macro_track/tutorial.rs`),
  `etapas::percorre` (uma porta para nomes e critérios), `MotionDoc::import_owner`, o balão
  (`ph2d-editor-core/.../chrome/tutorial_balao.rs`, dados por `macro_tutorial::set_balao`), `TutorialDaFamilia`
  (`ph2d-editor-core/src/macro_tutorial.rs`), a família (`ph2d-app-macros/src/tutorial.rs`).
- ⛔ Recusado (plano §2): o tutorial aplicar sozinho as etapas sem portão (laço com o Ctrl+Z); um cursor guardado
  (duas verdades com o documento).

## Partilhado tocado (para o integrador)

`ph2d-ecs` (`ModoDaMacro::Fazer`) · `ph2d-editor-core` (`macro_tutorial.rs` NOVO, `MacroRequest::{MostraMe,Recomeca}`,
`CampoDaEtapa::Dica`, `EtapaView.dica`, `MACRO_MODOS` 3, chrome `tutorial_balao` + `chrome/mod.rs` gerado pelo
`ph2d-chrome-sync`, `hero/paint.rs`) · `ph2d-macro` (`EdicaoDeEtapa.dica`, etiqueta 8) · `ph2d-motion-doc`
(`import_owner`, `copy_owner` partilha `Atributos`) · `ph2d-panel-motion-graph` (`hits::alvo` público) ·
`ph2d-panel-inspector` (3.º modo) · `ph2d-panel-macros` (`MACROS_HINT`) · `ph2d-i18n` · `ph2d-app-components` ·
`ph2d-app-painter` (`palco_de` em `Fazer`) · `ph2d-app-motion` · `ph2d-app-macros` (`Familias::motion` passa a
`GravadoraQueEnsina`, `motion_na_mao` NOVO) · shell (`fase_macros`, `macro_smoke` `=3`).

## Próximo

1. Smoke do dono da `=3`; o que ele disser pode mudar o balão antes da W6b.
2. W6b (Painter), W6c (os outros meios), prova de mutação W6a+b, fecho (`/pd-linha-fechar`).

## Depois (15/10, a pedido do dono ao ver a `=3`): o grafo arruma-se ao abrir

> Dono: *«favor abrir o app e dar o auto arranje pois os nós ficam todos bagunçados. Só não dar o auto arrange se o
> usuário mover os nós e salvar o projeto»*.

| commit | o quê |
|---|---|
| `bd161797a3` | `MotionDoc::arrumados_a_mao` (linhas `oh <id>` no `[owner]`, só quando há; ficheiro sem elas igual ao byte); o arrasto (`MoveNodes` com dx/dy ≠ 0) marca o dono, o undo desmarca; `motion_arrumar::arrumar_ao_abrir` (a MESMA `arrumar` do botão, e devolve as posições dos marcados); chamada em `load_text` (só o abrir projecto) e no «Recomeçar» do tutorial |
| `d1c5954303` · `a3826bcdd7` · `1d927891f9` | o tutorial traz à vista os nós da etapa: `ph2d_panel_motion_graph::request_frame_nodes` (uma vez por (dono, etapa, nós) — o aluno rola à vontade depois); espera o grafo chegar; re-centra se o painel muda de tamanho; nunca abaixo do zoom legível 0,35 — se não cabe, centra o 1.º alvo (a porta de onde se arrasta) |

Gates (por nome, verdes): `ph2d-motion-doc owner` (15) · `ph2d-app-motion motion_arrumar` (3) · `macro_track::tutorial` (8) ·
os de enquadrar do `ph2d-panel-motion-graph` (5) e `fit_tests` (4). `cargo check -p ph2d-host-desktop` limpo.

### ⛔ ABERTO — decisão do dono pedida (não implementar antes de ele responder)

Na `=3` arrumada, Box está à vista (anel + balão) e **Wind fica abaixo** do editor do grafo (~205 px de altura, a
Timeline come o resto). Causa medida: cada nó SOLTO é uma banda própria, empilhada com `BAND_GAP` 200 (não `GAP_Y`
40), e o espaçamento reserva a altura do cartão ABERTO — decisão do dono de 20/09 («nós estão se interpenetrando»):
⛔ não espaçar pela pílula. Box→Wind = 650 unidades; a 0,35 não cabem 117 px.
Opções postas ao dono: **(a, recomendada)** o arrumar põe os nós soltos lado a lado numa FILA; **(b)** fica — o
aluno rola. ⚠️ Tentado e desfeito: `hero.set_panel_visible("timeline", false)` na cena `=3` não esconde a Timeline.

HEAD de código `1d927891f9`. Binário `--profile smoke` compilado na worktree.

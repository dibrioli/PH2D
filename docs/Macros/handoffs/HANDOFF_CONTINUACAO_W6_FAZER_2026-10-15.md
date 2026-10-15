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

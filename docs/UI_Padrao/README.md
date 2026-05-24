# UI Padrão — fonte única da verdade

**Status:** ativo desde 2026-05-24. Sucessor de [DIRETRIZ §5.2 "Widget Gallery é a fonte de verdade"](../IntegracaoMultiAgente/DIRETRIZ.md), com escopo ampliado (3 painéis-canon) e granularidade aumentada (um doc por componente).

---

## O que este doc é

A **definição operacional** do padrão de UI da PH2D. Especifica:

- **Quem é o canon** (quais artefatos vivos têm autoridade).
- **Como o canon se organiza** (estrutura desta pasta).
- **Quem pode editar e quando atualizar**.
- **Como agentes (humanos ou LLM) consomem o canon** ao criar/modificar UI.

Tudo o que cai sob "como esse widget se comporta?", "qual o padrão visual?", "como compor isso com aquilo?" se resolve aqui (ou nos artefatos vivos referenciados aqui).

## O que este doc NÃO é

- **NÃO é a descrição dos componentes.** Cada componente tem (ou terá) seu próprio doc em `docs/UI_Padrao/components/<componente>.md`, populado conforme o canon estabilizar (vide plano [`docs/plans/2026-05-ui-source-of-truth.md`](../plans/2026-05-ui-source-of-truth.md)).
- **NÃO é especificação de design abstrato.** Para tokens / temas / tipografia, ver [`docs/design/`](../design/). Este doc parte do princípio que tokens são consumidos via `ph2d-tokens` e não rediscute essa camada.
- **NÃO é catálogo de bugs.** Bugs vivos ficam em [`docs/UI_Bugs/`](../UI_Bugs/) e [`docs/Image Tools Bugs/`](../Image%20Tools%20Bugs/). Este doc define o **comportamento esperado**; quando um bug é regularizado, vira regra aqui.
- **NÃO é arquitetura.** Para decisões estruturais (panel host, action bus, freeze contracts), ver [`docs/architecture/decisions/`](../architecture/decisions/).

## As 3 fontes vivas (canon executável)

A definição última do padrão **roda no app**, não em prosa. Estes 3 painéis SÃO o canon:

| Painel | Crate | Cobre |
|---|---|---|
| **Widget Gallery** | [`ph2d-panel-widget-gallery`](../../crates/ph2d-panel-widget-gallery/) | Componentes isolados — cada widget primitive em sua forma canônica. |
| **Inspector** | [`ph2d-panel-inspector`](../../crates/ph2d-panel-inspector/) | Componentes em contexto de painel docado — composição (sliders+chip, dropdowns, color picker, vector editor em painel real). |
| **Hierarchy** | [`ph2d-panel-hierarchy`](../../crates/ph2d-panel-hierarchy/) | Componentes de navegação — tree view, DnD reparent, multi-select, context menus. |

Seed central: [`crates/ph2d-editor-core/src/screens/hero/pre_populate.rs`](../../crates/ph2d-editor-core/src/screens/hero/pre_populate.rs).
Showcase auxiliar: [`crates/ph2d-editor-core/src/widget/showcase/`](../../crates/ph2d-editor-core/src/widget/showcase/).

Se este doc divergir dos 3 painéis, **confie nos painéis** e atualize este doc.

## Estrutura desta pasta

```
docs/UI_Padrao/
├── README.md              ← este doc (definição operacional)
└── components/            ← um arquivo por widget primitive (a popular)
    ├── number_chip.md
    ├── slider_with_chip.md
    ├── button.md
    ├── dropdown.md
    └── ...
```

Cada `components/<slug>.md` (formato a definir na Fase 2 do plano) descreverá: nome canônico, path do widget, onde aparece nos 3 painéis, estados visuais, comportamento de input, regras de composição, tokens consumidos, a11y, gates ativos. **Esta pasta começa vazia** — populada conforme a Fase 1 do plano fecha cada categoria de componente.

## Quem edita / quando atualizar

- **Coord-A** (vide [DIRETRIZ §1](../IntegracaoMultiAgente/DIRETRIZ.md)) é o único papel que edita este doc + os artefatos vivos canon (Gallery / Inspector / Hierarchy / showcase / pre_populate).
- **Sempre que mudar comportamento de um widget**: atualize o doc do componente afetado **antes** ou **junto** do commit. Atualizar depois → débito que sangra produção (vide §5.3 da DIRETRIZ — "regras herdadas do Gallery, cada uma já queimou ≥1×").
- **Adicionou componente novo no Gallery?** Crie `components/<slug>.md` no mesmo commit.
- **Removeu componente?** Marque o doc como deprecated com prazo de remoção; remova depois de uma wave de back-compat.

## Como agentes consomem

Implementador / Coord-B abrindo painel novo ou modificando existente:

1. Leia este README pra orientação.
2. Identifique cada componente que vai usar; abra `components/<slug>.md` correspondente.
3. **Copie literalmente** o setup que o doc descreve (ou que os 3 painéis canon mostram). Sem "minha variação compacta".
4. Se o doc falar uma coisa e os painéis canon mostrarem outra: **confie nos painéis**, reporte ao Coord-A pra atualizar o doc.
5. Se o componente não tem doc ainda: **pare e reporte ao Coord-A**. Não invente — significa que esse componente ainda não foi padronizado.

## Anti-padrões

- **Não fragmente o canon.** Se está em dúvida entre "abro um doc de componente novo" ou "estendo um existente": estender. Doc novo só pra widget primitive realmente distinto.
- **Não documente comportamento que não existe nos 3 painéis canon.** Doc de componente que descreve afordância não-implementada vira ficção; agentes copiam a ficção e quebra.
- **Não use este doc pra justificar exceção.** "O doc diz X mas no meu painel Y porque ...": se a justificativa procede, vira regra; se não, é divergência. Sem zona cinza.

## Referências

- Plano vivo de standardização: [`docs/plans/2026-05-ui-source-of-truth.md`](../plans/2026-05-ui-source-of-truth.md).
- Regra antecessora (sucedida por este doc): [DIRETRIZ §5.2](../IntegracaoMultiAgente/DIRETRIZ.md).
- ADR base de UI: [ADR-0023 — UI/UX baseline](../architecture/decisions/0023-ui-ux-baseline.md).
- ADR de painel típado: [ADR-0029 — trait-driven panel host](../architecture/decisions/0029-trait-driven-panel-host.md).
- Bugs históricos (lições estruturais embutidas aqui): [`docs/UI_Bugs/README.md`](../UI_Bugs/README.md), [`docs/Image Tools Bugs/README.md`](../Image%20Tools%20Bugs/README.md).

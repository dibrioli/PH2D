# Luz 2D

Iluminação 2D de cena: tudo o que brilha ilumina, objetos marcados fazem sombra suave física, a luz
rebate — com custo independente do número de luzes (Radiance Cascades). Linha `line/Luz2D`.

| Doc | O quê |
|---|---|
| [01_plano.md](01_plano.md) | o desenho (uma porta por pergunta), contratos, UI, gates, smokes, ondas |
| [pesquisa/reports/Luz 2D estado da arte.md](pesquisa/reports/Luz%202D%20estado%20da%20arte.md) | o estado da arte com fontes, a arquitetura em camadas, o plano de medição e os kill-criteria |
| [pesquisa/research_notes/](pesquisa/research_notes/) | as sete notas de pesquisa por trás do relatório |
| `handoffs/` | registro de sessão — o último é o de continuação da onda mais recente |
| `crates/ph2d-light2d-ecs` | componentes `Luz`/`BloqueiaLuz`/`NaoRecebeLuz`, a porta de registo e a coleta `World → CenaDeLuz` (W1) |
| `crates/ph2d-light2d-ref` | o gabarito: `fluencia` exata por varredura angular, `queda` (janela de Karis) — G1/G2/G10/G12 (W1) |
| `crates/ph2d-light2d-gpu` | o solver na GPU: C0 captura (cobertura + material do dono), C1 JFA, C2 Radiance Cascades (bilinear fix, pré-média), `PassList` e bytes, carimbos por passe — G3/G4/G5/G10/G12 (W2) |

⚠️ A crate `ph2d-light` é o rig de normais do **impasto** — não é esta luz.

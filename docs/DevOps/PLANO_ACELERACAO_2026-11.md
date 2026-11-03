# PLANO_ACELERACAO_2026-11 — medir menos, integrar mais cedo, reler menos

> Decisão do dono, 03/11/2026. Fonte e evidência: [PESQUISA_DESENVOLVIMENTO_RAPIDO_2026-11-03.md](PESQUISA_DESENVOLVIMENTO_RAPIDO_2026-11-03.md)
> (anexo técnico P1–P8, I1–I5, C1–C5, L1–L4, cada passo com a medição A×B). Memória:
> [teste de pincel = alarme barato](../../project-memory/project_brush_tests_cheap_alarm_owner_decides_2026_11_03.md).

## Lei dos pincéis — DECIDIDA (não «a medir»)

- O teste de pincel é um **alarme barato que aponta o que mudou**; quem **decide é o dono** (gizmo × traço, a olho).
  A aprovação do dono vira a **referência** (o «bless» do P5).
- Tabelas de largura de ~40 tamanhos e testes de largura de horas **deixam de ser o caminho**.
  ⛔ **Não regerar tabelas de 40 tamanhos.** As existentes ficam só como gabarito para provar a troca (P1).

## Estado dos itens: ADOTADO — a confirmar por medição (A×B do anexo)

Só **depois da medição** cada item muda a DIRETRIZ e os comandos `/pd-*`; até lá nada daqui é regra, gate ou texto de comando.

| # | Recomendação | Passos do anexo | Estado |
|---|---|---|---|
| 1 | ~5 medições + reta por pincel, provado contra as tabelas existentes | P1, P3 | ADOTADO — a confirmar por medição (A×B do anexo) |
| 2 | Teste rápido de largura (3–4 tamanhos) no lugar do lento; relatório só do que mudou | P2, P4, P5, P6 | ADOTADO — a confirmar por medição (A×B do anexo) |
| 3 | Integrar uma linha por vez, assim que verde | I1 | ADOTADO — a confirmar por medição (A×B do anexo) |
| 4 | Nível lento à noite sobre o `main` | I2, I4 | ADOTADO — a confirmar por medição (A×B do anexo) |
| 5 | Amostra do nível lento no fecho da linha | I3 | ADOTADO — a confirmar por medição (A×B do anexo) |
| 6 | Teste afrouxado na integração só marcado e explicado | I5 | ADOTADO — a confirmar por medição (A×B do anexo) |
| 7 | Sessões curtas, mecânico a modelo barato | L1–L4 | ADOTADO — a confirmar por medição (A×B do anexo) |
| 8 | Compilação: polo longo, hakari, binários de teste, cache por árvore | C1–C4 (C5 = não fazer) | ADOTADO — a confirmar por medição (A×B do anexo) |
| 9 | Segundo anel no cursor (tinta molhada) | P7 (P8 opcional) | ADOTADO — a confirmar por medição; o dono decide no smoke |

## Ordem

1. **P1** — a reta contra as tabelas existentes (custo zero de simulação; é o gabarito de tudo o resto).
2. **P4 / P2** — teste rápido de largura com **prova de mutação** + relatório antes/depois/diferença **só dos pincéis que
   mudaram**, para o dono aprovar (P5).
3. **I1** — integrar uma linha por vez, assim que verde. Linha grande ou de contrato (§6) integra **sempre sozinha**.
4. Depois: I2–I5, C1–C4, L1–L4, P3, P6–P8, na ordem que a medição do passo anterior ditar. Medição segundo
   [MEDIR_VELOCIDADE.md](MEDIR_VELOCIDADE.md) (mesmo processo, intercalado, o mínimo) e CLAUDE.md §0.10 (um lote).

# PLANO_ACELERACAO_2026-11 — medir menos, integrar mais cedo, reler menos

> Decisão do dono, 03/11/2026. Fonte e evidência: [PESQUISA_DESENVOLVIMENTO_RAPIDO_2026-11-03.md](PESQUISA_DESENVOLVIMENTO_RAPIDO_2026-11-03.md)
> (anexo técnico P1–P8, I1–I5, C1–C5, L1–L4, cada passo com a medição A×B). Memória:
> [teste de pincel = alarme barato](../../project-memory/project_brush_tests_cheap_alarm_owner_decides_2026_11_03.md).

## Lei dos pincéis — DECIDIDA (não «a medir»)

- O teste de pincel é um **alarme barato que aponta o que mudou**; quem **decide é o dono** (gizmo × traço, a olho).
  A aprovação do dono vira a **referência** (o «bless» do P5).
- Tabelas de largura de ~40 tamanhos e testes de largura de horas **deixam de ser o caminho**.
  ⛔ **Não regerar tabelas de 40 tamanhos.** As existentes ficam só como gabarito para provar a troca (P1).

### Como se usa (em vigor desde 03/11)

Mudou motor, preset ou pincel:

1. Corra os alarmes por nome, no fecho da onda ou da linha, não a cada edição:
   `bash scripts/cargo-test-narrow.sh ph2d-tool-painter a_largura_de_fabrica_e_a_aprovada` e
   `bash scripts/cargo-test-narrow.sh ph2d-tool-painter a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje`.
   Mexeu num misturador: também `a_area_do_blend_cabe_no_circulo`.
2. Vermelho aqui quer dizer «estes pincéis mudaram», não «está errado». ⛔ Não regrave para ficar verde.
3. Leve ao smoke do dono o relatório `target/relatorio-pinceis/index.html` (na worktree onde o alarme correu): só os pincéis
   que mudaram.
4. Ele aprovou? Regrave com `PH2D_PINCEIS_APROVA=1`, no mesmo comando do alarme, com `cargo test -p ph2d-tool-painter
   --profile smoke --test it <nome>`. Comite o resultado. Não aprovou? O defeito é do motor ou do preset.

Custo e prova: 55 presets, raios 2·7·26, 26,6 s, determinístico 3/3; os 3 testes juntos, 85–99 s. As mutações mostram que o
alarme apanha um superconjunto dos 8 testes de horas que substituiu
([medição §C](MEDICAO_P1_P4_LARGURA_2026-11-03.md)). `.config/testes-lentos.tsv` pode pô-lo no nível lento, por isso o
`nextest-impacted` não substitui o passo 1.

## Estado dos itens

Cada item só muda a DIRETRIZ e os comandos `/pd-*` **depois da sua medição**. Já mudaram: 1, 2 e 3. Os outros ainda não são regra,
gate nem texto de comando.

| # | Recomendação | Passos do anexo | Estado |
|---|---|---|---|
| 1 | ~5 medições + reta por pincel, provado contra as tabelas existentes | P1, P3 | MEDIDO 03/11 — [medição](MEDICAO_P1_P4_LARGURA_2026-11-03.md); reta+5 pontos passa em 34/55, só; ferramentas e aguadas precisam de mais pontos. ⇒ A reta **não** substitui as tabelas gravadas |
| 2 | Teste rápido de largura (3–4 tamanhos) no lugar do lento; relatório só do que mudou | P2, P4, P5, P6 | **EM VIGOR 03/11** — [medição](MEDICAO_P1_P4_LARGURA_2026-11-03.md), usado e aprovado no smoke do dono; ver «Como se usa» |
| 3 | Integrar uma linha por vez, assim que verde | I1 | **EM USO 03/11** — DIRETRIZ §1.5.3. Uma amostra: handoff às 19:18, curas no fim às 20:19, uma suíte de 31 min. A rodada anterior (4 linhas, 02/11c) levou >36 h, a maior parte a regerar tabelas: não é A×B. Dono: «muito sucesso» |
| 4 | Nível lento à noite sobre o `main` | I2, I4 | ADOTADO — a confirmar por medição (A×B do anexo) |
| 5 | Amostra do nível lento no fecho da linha | I3 | ADOTADO — a confirmar por medição (A×B do anexo) |
| 6 | Teste afrouxado na integração só marcado e explicado | I5 | ADOTADO — a confirmar por medição (A×B do anexo) |
| 7 | Sessões curtas, mecânico a modelo barato | L1–L4 | ADOTADO — a confirmar por medição (A×B do anexo) |
| 8 | Compilação: polo longo, hakari, binários de teste, cache por árvore | C1–C4 (C5 = não fazer) | ADOTADO — a confirmar por medição (A×B do anexo) |
| 9 | Segundo anel no cursor (tinta molhada) | P7 (P8 opcional) | ADOTADO — a confirmar por medição; o dono decide no smoke |

## Ordem

1. ✓ **P1** — a reta contra as tabelas existentes (custo zero de simulação; é o gabarito de tudo o resto).
2. ✓ **P4 / P2** — teste rápido de largura com **prova de mutação** + relatório antes/depois/diferença **só dos pincéis que
   mudaram**, para o dono aprovar (P5).
3. ✓ **I1** — integrar uma linha por vez, assim que verde. Linha grande ou de contrato (§6) integra **sempre sozinha**.
4. **A seguir:** I2–I5, C1–C4, L1–L4, P3, P6–P8, na ordem que a medição do passo anterior ditar. Medição segundo
   [MEDIR_VELOCIDADE.md](MEDIR_VELOCIDADE.md) (mesmo processo, intercalado, o mínimo) e CLAUDE.md §0.10 (um lote).

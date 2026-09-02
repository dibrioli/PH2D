---
name: an-absence-gate-that-names-a-file-is-disarmed-in-silence
description: Um gate que varre o FONTE e nomeia um ficheiro reage a um corte de LOC em direcções opostas conforme afirme presença ou ausência
metadata:
  type: feedback
---

Um gate de varredura de fonte que **nomeia um ficheiro** (`include_str!("../src/a/b.rs")`,
`join("src/a/b.rs")`) é partido por qualquer corte por responsabilidade — e as **duas espécies
reagem ao contrário**:

| o gate afirma | o que o corte lhe faz |
|---|---|
| **presença** (*«alguém chama isto»*) | reprova **ALTO**, com uma acusação FALSA |
| **ausência** (*«isto NÃO voltou»*) | fica **VERDE E VAZIO** — a prova mudou-se para fora do alcance dele |

⛔ **A segunda é a perigosa**, e não se nota: a ausência que ele exige passa a ser **de graça**, e
o defeito pode voltar no ficheiro ao lado com o gate a passar.

**Why:** medido em 2026-08-30 (`line/UIUX`): o tecto de 700 LOC obrigou a cortar o bloco da
geometria do `hero/paint.rs` para um irmão. Dois gates reprovaram alto (acusação falsa) e um gate
de ausência — `the_side_columns_are_anchored`, que exige que o offset de arrasto **não** esteja no
`paint.rs` — ficou verde sobre nada.

**How to apply:** pergunte ao **MÓDULO**, não ao ficheiro: varra o directório recursivamente e
procure o ficheiro que contém (presença) ou afirme sobre todos (ausência). Molde:
`crates/ph2d-editor-core/tests/common/hero_sources.rs`. ⚠️ E um gate pode ser desarmado por um
**RAMO**, não só por um corte: `the_chrome_reads_the_ui_clock` continua a afirmar a verdade sobre
código que agora só corre dentro de `if legacy_chrome` — sem rename e sem ficheiro movido, nada
podia falhar alto. Ver [[feedback_a_source_parsing_gate_must_know_every_shape_of_what_it_parses]]. Irmão de mecanismo
diferente e família igual: [[feedback_absence_gate_needs_a_presence_sibling]] — ali a ausência é
falso-zero porque a coisa medida não existe; aqui é falso-zero porque a PROVA se mudou de ficheiro.

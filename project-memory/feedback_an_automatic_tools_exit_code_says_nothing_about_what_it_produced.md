---
name: feedback-an-automatic-tools-exit-code-says-nothing-about-what-it-produced
description: "`exit 0` de uma ferramenta automática (cargo fix, clippy --fix, um script com /usr/bin/time) não é evidência sobre o trabalho: ela pode não ter corrido, não ter feito nada, ou ter escrito código partido. Só a medição independente depois responde."
metadata:
  type: feedback
---

**Regra:** o código de saída de uma ferramenta automática descreve **se ela terminou**, nunca **o que
ela produziu**. Toda corrida de ferramenta que escreve código ou mede algo precisa de uma verificação
**independente do próprio exit code** antes de eu afirmar o resultado.

**Why:** três ocorrências no MESMO dia (2026-08-29, bloco A da atualização de stack —
`docs/Atualizar Stack/`), todas com `exit 0` e todas mentindo:

| ferramenta | disse | era |
|---|---|---|
| script com `/usr/bin/time -v` (inexistente no CachyOS) | `0` — porque quem fechava o cano era um `tail` | a suíte **nunca correu**; a «fotografia do antes» ficou vazia |
| `cargo clippy --fix` | `0` | escreveu `&mut x.fill(v);` — linha partida, apanhada só pelo portão |
| `cargo clippy --fix` (2.ª e 3.ª corridas) | `0` | **não corrigiu nada**: o cargo só reanalisa o que RECOMPILA, e estava tudo em cache |

⚠️ A terceira é a mais traiçoeira, porque nada parece errado: a ferramenta corre, imprime, sai limpa,
e o número de problemas não desceu. *Uma ferramenta que não teve trabalho a fazer e uma que se recusou
a fazê-lo imprimem a mesma coisa.*

**How to apply:**
- **Depois de toda corrida de `--fix`, meça de novo** e compare a contagem antes/depois. Se não desceu,
  ela não trabalhou — não é que já estava bom.
- Para forçar reanálise quando o cargo tem tudo em cache: `touch` nos ficheiros afectados (ou
  `cargo clean -p <crate>`). Sem isso o `--fix` é um no-op silencioso.
- **Um portão que para no primeiro erro não mediu o que o nome dele promete.** `cargo clippy -- -D warnings`
  compila em ordem de dependência e aborta no 1.º crate com erro: cada rodada revela só a camada
  seguinte. Para o TOTAL, corra **sem** `-D warnings` — os avisos não interrompem e a varredura
  atravessa a workspace inteira numa passagem. (Custou-me cinco rodadas e quatro números errados
  anunciados como totais: 5 → 12 → 251 → 139 → **375**.)
- ⛔ Não anuncie um número que veio de uma corrida interrompida. Rotule-o: «medido numa passagem, sem
  portão a interromper» ou não o diga.
- Depois de um `--fix`, **audite o diff por TIPO de mudança**, não só a contagem: ele aplica todas as
  regras maquináveis, não a que você tinha em mente. Aqui ele mexeu em 9 ficheiros por outras regras,
  um deles no caminho do determinismo da física (`.iter()` → `.keys()`, seguro, mas eu tinha dito ao
  Enio que era «só a migração as_chunks»).

Liga com [[feedback-pipe-masks-script-exit-code]] (o cano que engole o código do comando — é o
mecanismo da 1.ª linha da tabela), [[feedback-python-replace-silent-noop-after-fmt]] (o `str.replace()`
que não casa e imprime sucesso — a mesma família, noutra ferramenta), [[feedback-perfection-no-deferrals]]
(§SUPRESSÃO: o que fazer quando a ferramenta está partida de forma sistemática) e
[[feedback-counting-the-work-done-is-not-counting-the-work-delivered]].

# Atualizar Stack — a porta

> **Estado:** planejado, **não começado**. Aberto em 2026-08-29, no disco novo.
> **Autorização:** o Enio pediu *«atualizar totalmente o projeto para o stack mais recente possível»*
> (2026-08-29). Isto é o plano; a execução de cada bloco continua a pedir ordem dele (§0.7 do `CLAUDE.md`).

## O veredito, em uma linha

**É viável, não há impedimento técnico, e este é o melhor momento que vai existir** — as 6 linhas
estão todas integradas e limpas, a árvore parte **100% verde** (20 041 testes, zero falhas, medido
em 2026-08-29), e o disco novo tem 1,9 TB com 1% de uso.

> ⛔ **Correção ao que este documento afirmava na 1.ª redação:** *«o `target/` está vazio, então o
> maior custo escondido — jogar fora a cache de build — já foi pago pela troca de disco»*. **Falso.**
> Há **sccache** no `~/.cargo/config.toml` global, e o cache dele vive em `/home`, que não foi
> trocado: a primeira build devolveu **1 302 acertos contra 2 faltas**. O que a troca apagou foi a
> metade barata. A metade cara sobreviveu — e é ela que os blocos **C**, **D** e **E** vão invalidar
> de verdade, porque `wgpu 29`, `vello 0.10`, `bevy_ecs 0.19` e `rapier2d 0.35` nunca estiveram
> naquele cache. Detalhe e números: [`04_registro.md`](04_registro.md) §1.

## Por onde entrar

| Você quer | Leia |
|---|---|
| **O retrato medido** — o que temos, o que dá para ter, e as 7 amarras | [`01_inventario.md`](01_inventario.md) |
| **As tarefas** — a lista exaustiva, por bloco, cada uma com verificação e recuo | [`02_tarefas.md`](02_tarefas.md) |
| **O que vai ficar vermelho** e como separar «melhorou» de «quebrou» | [`03_riscos.md`](03_riscos.md) |
| **O que já foi feito** — preencher DURANTE a execução | [`04_registro.md`](04_registro.md) |

## ⚠️ Antes de acreditar em qualquer número deste plano

```
bash scripts/stack-audit.sh
```

**O inventário do `01` é uma FOTOGRAFIA de 2026-08-29; o comando acima é a fonte.** Ele deriva da
árvore quem declara o quê, consulta o índice do crates.io, classifica cada salto e — a parte que
decide o plano inteiro — **calcula os TETOS**, isto é, quando uma dependência nossa é segurada por
outra dependência.

Isto não é zelo decorativo: o §5.0 do `CLAUDE.md` já cobrou cinco vezes o preço de um número
escrito à mão numa seção de roteador. *A fonte de cada número é o código, não este documento.*

Modos: `--maior` (só os saltos que quebram API) · `--tetos` (só as amarras) · `--offline`.

## Os oito blocos, e por que são oito

A atualização **não é um evento**. São oito trabalhos com riscos e donos diferentes — **95 tarefas** —
e quatro deles (T, A, B, F) podem correr sem ninguém acompanhar.

| Bloco | O que é | Risco | Termina em |
|---|---|---|---|
| **T** | O terreno do disco novo | 🟢 nenhum | `stack-audit` + `btrfs-health` verdes |
| **A** | Rust 1.95 → 1.98 | 🟢 se compilar, está igual | suíte verde |
| **B** | 31 bibliotecas compatíveis (`cargo update`) | 🟢 | suíte verde |
| **C** | GPU e texto — vello, wgpu, naga, skrifa, parley, fontique | 🔴 **muda pixel** | **smoke do Enio** |
| **D** | `bevy_ecs` 0.18 → 0.19 | 🟡 185 ficheiros, mecânico | suíte verde |
| **E** | `rapier2d` 0.28 → 0.35 | 🔴 **muda o tato da física** | **smoke do Enio** |
| **F** | A cauda — 19 bibliotecas isoladas | 🟡 uma de cada vez | suíte verde |
| **G** | Fecho — ADR, ship, memória | 🟢 | CI verde |

**Ordem recomendada:** `T → A → B → F → D → C → E`.
A cauda (**F**) vem antes de **D** e **C** de propósito: são as tarefas mais baratas e mais numerosas,
e fechá-las cedo tira ruído do diff dos dois blocos caros.

⛔ **Os blocos C e E não fecham em teste verde.** Eles fecham quando o Enio olha a tela. Isso não é
falta de rigor: as mudanças que eles trazem **compilam perfeitamente e mudam o resultado** — nenhum
portão deste repo as vê. Ver [`03_riscos.md`](03_riscos.md).

## ⛔ O que este plano NÃO faz

- **wgpu 30.** Inalcançável: o `vello 0.10` — que é o mais novo que existe — pede `wgpu ^29.0.3`.
  Forçar dá duas cópias do wgpu, e aí o vello recusa o nosso `Device`.
- **skrifa 0.46, accesskit 0.25, pollster 1.0, core-graphics 0.25.** Os quatro estão presos por
  `parley`/`vello`/`usvg`/`rfd`/`winit`. A tabela com o dono de cada teto está no `01`.
- **`ndarray` 0.17.** Preso pelo `deep_filter` vendorizado (`^0.15`), que não policiamos de propósito
  (`exclude` do workspace raiz).
- **Tocar em contrato congelado** (§6 do `CLAUDE.md`). Nenhuma tarefa aqui mexe em `NodeOp`,
  `OpResolver`, `NodeManifest`, `Tool`, `RasterEditTool`, `CanvasPaintTool`, `PanelEvent` ou na
  superfície de `ph2d-vector-doc`. Se uma tarefa te levar lá, **pare e reporte** — vira ADR.

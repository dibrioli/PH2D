# Despacho da janela principal (missão Painter 4K) — o que já foi lançado

> A janela principal só decide e confere. Se a conversa compactar: este ficheiro + `git log` + os estados do
> [`BACKLOG.md`](BACKLOG.md) e do [`BACKLOG_AUDITORIA.md`](BACKLOG_AUDITORIA.md) são a verdade.

| hora | o quê | resultado |
|---|---|---|
| 07:10 | 11 extratores dos estudos → `fontes/` | feito (`5693d690a4`) |
| 07:15 | medição de base no app (B08 «antes») | feito (`2bf12269c1`), parcial: sem fundo branco (falta `PH2D_COMPOSITE_FUNDO`), blocos só 1024², Real Oil 1440 por medir, trocar de camada por medir. **Placa: forçar a Radeon integrada** (sob Xvfb o app abria a RTX) |
| 07:20 | BACKLOG único: 47 blocos, 313 itens, 12 ondas | feito (`5693d690a4`) |
| 07:22 | pente fino AU01–AU15 (só leitura) | feito (`7e11fbcb6f`) |
| 07:47 | achados AU → `BACKLOG_AUDITORIA.md` | em curso |
| 07:48 | **onda 1:** B01 camadas · B03 traço preso fora da janela · B07 réguas/lacunas · B11 Wet a tempo + espelho · diagnóstico do quadro de ~100 ms comum a 4K e do CPU parado 48–70 % | em curso |

## Fatos medidos que mudam a ordem

- 4K papel: quadro de ~100 ms igual em TODOS os pincéis no 1.º movimento, ~0,5 s depois de soltar e depois do Ctrl+Z
  (1080p ~35 ms) — fora do pincel. Traço 4K p95 21–26 ms (Filbert 37). Toque Water/Watercolor Basic ~155 ms.
- Parado: 48–70 % de um núcleo (o app redesenha sem entrada) → B23 sobe.
- Wet com Real Oil: a água cai de 11,9 para 0,6 img/s em 6 rodadas.

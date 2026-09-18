# INBOX — canal da janela I para o ledger do `blender-rake` (append cego)

> A janela I **nunca lê** o `LEDGER_blender-rake.md` nem a `VASSOURA_blender-rake.txt`.
> O canal é este ficheiro: `cat >> INBOX_blender-rake.md` (append que não lê).
> Um subagente E/R transcreve para o ledger.

## Formato

```
[<data>] <session-id da janela> · <tipo: ABERTURA | RETOMADA | INCIDENTE | DUVIDA | RECALL>
<uma linha por facto. ⛔ DESCREVA, nunca REPRODUZA — se for preciso identificar um trecho com
exactidão, escreva o sha256 dele, nunca o texto.>
```

## Entradas

*(vazio — a janela I ainda não abriu)*

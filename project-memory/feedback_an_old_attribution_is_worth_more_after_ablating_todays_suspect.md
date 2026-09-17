---
name: an-old-attribution-is-worth-more-after-ablating-todays-suspect
description: Um vermelho «já documentado como pré-existente» não isenta a tua mudança — abla o teu suspeito e confirma que o número volta AO BIT
metadata:
  type: feedback
---

Ao fechar a `line/motion-value` (2026-09-16) a suíte `--ignored` de GPU deu 8 vermelhos. Um deles,
`value_slope_kernel_matches_the_cpu_on_the_device`, trazia no **próprio doc-comment** a atribuição:
*«VERMELHO NESTA MÁQUINA, e NÃO é da linha que o encontrou (2026-08-11): mede 1,05023384e-4 contra
a barra de 1e-4, atribuído por ABLAÇÃO»*.

Era tentador parar ali. Mas a atribuição velha ablou o suspeito **daquele dia** (uma recusa no
`eligible`), e o meu era outro: a cura que fez a costura CPU→GPU não voltar a atravessar quando não
muda. ⇒ desliguei a reutilização e corri outra vez: **`1,05023384e-4`, ao bit**, o mesmo número. Só
então a atribuição passou a cobrir também a minha mudança.

**Why:** uma atribuição é sempre *«não é o suspeito X»*, nunca *«não é ninguém»*. Ela envelhece por
ADIÇÃO de suspeitos, e nada no ficheiro avisa — o doc-comment continua a ler-se como absolvição
geral. O caso oposto é pior: um vermelho real e novo, escondido atrás de uma nota que fala de
outra coisa.

**How to apply:** ao encontrar um vermelho com atribuição escrita, pergunte *que suspeito foi
ablado?* Se o seu não estiver na lista, **abla o seu** — e a prova é o número voltar **igual ao
bit**, não «parecido». Depois acrescente o seu suspeito à nota, para o próximo não repetir.
⚠️ E o corolário da mesma corrida: `| tail -12` numa suíte dá os NOMES e não os MOTIVOS
([[feedback_a_tail_is_a_window_not_a_verdict]]) — re-corra cada acusado sozinho, com o `loadavg` ao
lado, porque 7 dos 8 eram carga ([[reference_flip_fit_cache_ratio_is_a_load_flake]]) e um verde
**sob carga** é conclusivo enquanto um vermelho sob carga não prova nada.

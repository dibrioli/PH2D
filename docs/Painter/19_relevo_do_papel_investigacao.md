# 19 — O relevo do PAPEL: investigação antes do ADR (2026-07-18)

> **Status: NÃO construído. Barreira aberta pelo Enio, mas o item mudou de forma sob medição.**
>
> Este doc existe para que a ordem que o Enio der seja sobre a decisão **real** — e para que o ADR possa
> ser escrito sem refazer a investigação. Ele não propõe código; propõe a pergunta certa.

## 0. O que a barreira diz

[`16_impasto_plano_implementacao.md` §2](16_impasto_plano_implementacao.md):

> **O relevo do PAPEL** (a ideia do §4.1 do doc 15 — `paper_h` alimentando a mesma luz) **sai do escopo**.
> Ela leria `watercolor_noise::paper_height`, e isso acopla impasto a aquarela. **Deferido, requer ordem
> nova do Enio.**

A premissa foi conferida e **está viva**: `paper_height` existe
(`watercolor_noise.rs:147`, usado por `watercolor_field.rs:552`) e a barreira executável
`watercolor_is_untouched_by_impasto` existe (`tests.rs:16958`).

## 1. Três achados que mudam a decisão

### 1.1 `paper_height` é um FALLBACK, não "o papel"

O docstring dele diz: *"the built-in fallback when no Paper slot is set"*. Existem também tiles assados
(`PaperCold` / `Rough` / `Hot`, 256²) e uma história de tiling com **snap de costura** (`NoiseTile`, para o
padrão fechar na emenda do sprite).

Ou seja: **"o papel" já é um conceito com várias partes** — um procedural de dois octaves, um conjunto de
bitmaps, e uma política de repetição. Fiar só o `paper_height` na luz seria fiar a *menor* delas.

### 1.2 O papel não é da aquarela — é do SUBSTRATO

Ele mora hoje em 5 arquivos, todos `watercolor_*` — mas isso é acidente de onde nasceu, não uma verdade
sobre o que ele é. **Um papel não é uma propriedade de um modo de pintura**: ele é a superfície sobre a
qual *qualquer* mídia assenta. Fazer o impasto chamar `watercolor_noise` seria exatamente o acoplamento que
a barreira recusa — e continuaria sendo o desenho errado mesmo com a barreira aberta.

### 1.3 A luz não tem termo de substrato, e a regra dela contradiz o papel

`ReliefFields::height_at` dobra **camadas que carregam relevo** + o traço vivo. Não há termo para "a
superfície embaixo de tudo".

E há uma regra que o módulo inteiro afirma: **relevo sob cobertura zero não acende** — o sculpt escreve só
`h` e isso é deliberado (§5 do plano 18), e o `Supply` do Conserve (2026-07-18) se apoia nela para recusar
um fosso que cavaria o invisível. **O papel precisa acender SEM tinta.** Isso é uma exceção legítima (o
papel está em toda parte, sua "cobertura" é 1), mas é **semântica nova**, não um termo a mais numa soma.

## 2. A pergunta que o ADR tem de responder

Não é *"a luz deve ler `paper_h`?"*. É:

> **Onde mora o SUBSTRATO, e o que a luz sabe sobre ele?**

Com três sub-perguntas que têm resposta técnica, não de gosto:

1. **Extração.** O substrato (procedural + slots + tiling) sai de `watercolor_*` para um módulo neutro que
   os dois lados leem? Isso **dissolve** o acoplamento em vez de aceitá-lo — o padrão que este projeto
   prefere (uma porta, não duas). Custo: mexe em arquivos `watercolor_*`, que é literalmente o que a
   barreira do §2 proíbe ("nenhuma linha, zero"). Logo, **exige o ADR** e revoga aquela cláusula
   explicitamente.
2. **Composição.** Tinta espessa **preenche** o dente do papel — não soma a ele. Um `h_total = h_paper +
   h_tinta` está errado: uma pincelada carregada deveria *apagar* a textura embaixo dela. A lei provável é
   uma mistura pesada pela carga (`h = lerp(h_paper, h_tinta, f(carga))`), e ela precisa de um número
   medido, não escolhido.
3. **Cobertura.** A regra "sem tinta, sem luz" vira "sem tinta, **luz do papel**". Isso toca o `Supply` do
   Conserve e o early-out de tinta plana do passe de GPU — os dois leem cobertura para decidir se há o que
   iluminar.

## 3. Recomendação

**Extrair (1), com ADR próprio, e tratar (2) como a parte cara.** A extração é mecânica e verificável; a
composição é o que decide se o resultado parece papel ou parece ruído somado à tinta, e é onde o smoke
mandará.

**Não recomendado:** o atalho de chamar `watercolor_noise` do impasto. É barato, passa nos testes, e
codifica a mentira de que o papel pertence à aquarela — a próxima mídia (giz, guache) herdaria o
acoplamento.

## 4. Por que não foi construído nesta jornada

O Enio abriu a barreira, mas a investigação mostrou que o item **não é uma fiação, é uma extração
arquitetural com um ADR na frente** — e começá-la no fim de uma jornada longa e entregá-la pela metade é o
oposto do padrão-ouro que ele pediu. A fila desta linha fechou; esta é a única coisa que resta, e ela
merece uma sessão inteira com a cabeça fresca.

---
name: feedback_measuring_a_ceiling_can_confirm_it_and_that_is_the_result
description: "Medir um teto pode CONFIRMAR o número que já lá estava — o defeito era a ausência de derivação, não o valor; um teto certo sem razão e um teto errado leem igual no dia em que alguém precisa de o mover"
metadata:
  type: feedback
---

Bloco Z, folha 09 da conferência: `MAX_GRADIENT_STOPS = 8`, com a justificativa
*"o painel é estreito e a faixa tem de ficar legível"*, sobre um modelo que admite 32.
Fui medi-lo esperando subi-lo. A conta deu **8,36**:

`220` (painel mais estreito) `− 36` (recuo) `= 184 px` úteis, `÷ 22 px` por parada
(`GRAB_R × 2`, o alvo de ponteiro que o próprio editor declara, mais a folga da
célula) ⇒ **8**.

⭐ **O número nunca esteve errado. O que não existia era a razão.**

**Why:** a lei do `CLAUDE.md` §0.0 é *meça antes de limitar*, e é fácil lê-la como
*"todo teto está baixo demais"*. Não é o que ela diz. ⚠️ **Um teto certo sem
derivação e um teto errado leem exactamente igual no dia em que alguém precisa de o
mover** — ninguém sabe se pode, e a resposta honesta («não sei») custa a mesma
investigação nos dois casos. O produto da medição é a **derivação executável**, e o
valor é um subproduto que pode ou não mudar.

⚠️ **E o recurso certo é o que decide, não o recurso plausível.** A frase antiga
falava de *legibilidade*; uma amostra de 14 px lê-se perfeitamente. O que ela deixa
de ser é **clicável** — cada amostra abre o seletor de cor —, e a régua estava ali ao
lado: a caixa de agarrar que o mesmo editor já declara para os marcadores.

**How to apply:**
1. Ao medir um teto, o entregável é o **gate que refaz a conta**, nunca só o número.
   Se o valor não mudar, o commit continua valendo o que valia.
2. Antes de escrever a derivação, pergunte **de que recurso** o teto é, e prefira a
   régua que a própria casa já declara (um alvo de ponteiro, um `MAX_*` do kernel, um
   `ulp`) a uma que você inventa para a ocasião.
3. Conte contra o **pior caso** (aqui: o painel mais estreito a que o arrasto chega),
   não contra o estado confortável de hoje — a mesma lei que o `motion.spring` aplica
   ao relógio.

*Irmã de [[feedback_the_ceiling_is_the_hardwares_never_the_fallbacks]] pelo outro
lado: aquela diz que o teto não pode ser do caminho lento; esta diz que confirmar um
teto é um resultado.*

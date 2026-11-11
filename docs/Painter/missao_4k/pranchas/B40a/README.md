# B40a — Color Fill e Cut respeitam o alfa travado e a máscara de proteção

Em cada imagem: esquerda = antes, meio = depois, direita = o que mudou (vermelho). Um disco laranja numa camada transparente.
«colorfill_trinco»: com o alfa da camada travado, o Color Fill antes enchia o retângulo inteiro; agora só repinta o disco.
«cut_protegido»: com a metade esquerda protegida pela máscara, o Cut antes apagava o disco todo; agora a metade protegida fica.
O Colar segue a mesma regra (a peça não pousa no protegido nem muda o alfa travado). Sem trinco e sem máscara, nada muda.

## Trim com Offset 0 (B40.2)

«trim_offset_zero»: uma Curva à mão livre que se cruza (um laço), com o Trim ligado e o Offset em 0.
Antes, o Trim achava que o laço era uma dobra do Offset e apagava-o (esquerda); agora o laço fica (meio).
Com o Offset diferente de 0 o Trim continua a cortar as dobras, como antes. Nada mais muda.

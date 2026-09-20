---
name: feedback_the_floor_of_a_limit_can_be_a_theorem_not_the_solver
description: Antes de gastar uma wave a apertar um limite, pergunte se o chão dele é um TEOREMA — achatar um pedaço curvo distorce área por obrigação, achatar uma face plana não distorce nada
metadata:
  type: feedback
---

**Antes de construir a cura de um limite, pergunte de que ele é chão. Se for um
TEOREMA, nenhuma afinação o move — e a saída é trocar de FAMÍLIA, não de solver.**

⛔ **Caso medido (`line/sculpt3d`, atlas de textura, 2026-09-20).** A W5 mediu que
`5 %` da superfície recebia `13 %` dos texels da mediana (`7,7×` mais borrada),
atribuiu o espalhamento ao parametrizador, e shipou **um escalar por peça com um
tecto varrido em duas peças** (`TECTO_DA_IGUALACAO = 3,0`). A lei está certa e o
tecto está medido. O que faltava era a pergunta anterior: **qual é o chão daquela
coluna?**

⭐⭐⭐ **É o Teorema Egregium de Gauss.** Uma superfície com curvatura não-nula não
tem achatamento isométrico: achatar um pedaço **curvo** obriga a distorcer ângulo
ou **área**, e a área **é** a densidade de texels. ⇒ *nenhum parametrizador leva
aquela coluna a `1`* — o LSCM, o SLIM, o BFF, todos pagam; só muda o valor.

⭐ **E a família vizinha não tem o problema, pela mesma matemática:** per-face
(Ptex/Htex) e mesh colors achatam **uma face plana de cada vez**, e um mapa afim
num triângulo preserva a razão de áreas **exactamente**. Ali o espalhamento
inteiro é o **arredondamento** da resolução por face (`√2` se for a potência de
dois mais perto), e quem o escolhe é quem escreve o código.

**Why:** um limite legítimo diz **de que recurso ele é** (CLAUDE.md §0.0). Um
tecto sobre uma grandeza cujo chão é um teorema nomeia o recurso errado: ele
lê-se como *«afinámos até aqui»* quando a frase honesta é *«esta família não vai
mais longe»*. E a diferença decide a wave seguinte — afinar o solver, ou mudar
de onde a tinta mora.

**How to apply:** ao escrever um `MAX_*`/cap/tecto sobre uma grandeza
**geométrica**, escreva ao lado qual é o valor ideal e **porque ele é
inalcançável**. Se a resposta for *«porque a superfície é curva»*, *«porque o
empacotamento desperdiça»*, *«porque o `f32` tem 24 bits» —* isso é um chão, e a
tabela do tecto tem de o mostrar. Se não houver resposta, o tecto é um palpite à
espera de um smoke. ⚠️ A régua que torna isto accionável é comparar a família com
**a família que não tem o problema**, não com uma versão melhor de si mesma.

Medido em [[reference_topic_quad_remesh_rulers]]; o corpus, a triagem e as quatro
recusas estão em `docs/3D/27_o_estado_da_arte_de_onde_a_tinta_mora.md`, e o
instrumento é `docs/3D/ferramentas/onde_a_tinta_mora.py`.
Irmão de [[feedback_a_global_extreme_is_not_a_per_face_ruler]].

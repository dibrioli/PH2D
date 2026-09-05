---
name: a-reconciling-pass-turns-a-delete-into-a-silent-undo-of-another-edit
description: Onde um passe RESTAURA o que ele possui, apagar não é um no-op — é um revert silencioso da edição vizinha; e a guarda tem de estar no GESTO, porque o passe não distingue «nunca existiu» de «foi apagado»
metadata:
  type: feedback
---

Report do dono (2026-09-05): *«ao tentar deletar o objeto, ele não é deletado e volta para sua
posição de origem»*. Reproduzido headless em cinco minutos, e os dois sintomas eram **um** mecanismo:
o `despawn` passava, o passe estrutural das instâncias **re-materializava** a peça no quadro seguinte
(o mestre continua a tê-la), e ela voltava com a pose do MESTRE — **apagando o arrasto que o artista
tinha feito nela**. Havia ainda um terceiro efeito que ele não podia ver: a chave de excepção
sobrevivia a apontar para um valor que já não existe.

⇒ **um gesto que não faz nada é mau; um que DESFAZ outra coisa é pior.**

**Why:** todo passe que reconcilia um derivado contra uma fonte (instância↔receita, cache↔ficheiro,
UI↔documento, index↔árvore) trata *«o alvo não tem o que a fonte tem»* como **falta**, e repõe. Um
apagar produz exactamente esse estado. Nada no passe distingue *«a fonte ganhou isto agora»* de
*«alguém apagou a cópia disto»* — **são o mesmo estado do mundo**; só quem recebeu o clique conhece
a intenção.

**How to apply:**
1. Perante *«apaguei e voltou»*, procure o passe que **repõe**, não o handler do apagar.
2. A guarda entra no **GESTO** (o único caminho que o artista alcança), nunca no passe — e a
   pergunta é *«isto veio da fonte?»*, com uma porta.
3. ⚠️ **Não reutilize a cerca vizinha sem a medir**: a condição *«estou DENTRO do derivado?»* e
   *«a fonte DEU isto?»* leem-se iguais e não são a mesma — o que as separa é o **elo**. Colapsá-las
   recusa o que o artista criou lá dentro (o gate apanhou-o na 1.ª corrida).
4. A recusa **diz onde fazer**. Metade deste report era a instrução: duas linhas com o mesmo nome na
   árvore, e nada a dizer qual era a da fonte.

Relacionado: [[a-fence-can-guard-two-things-and-name-only-one]],
[[a-surface-that-only-counts-is-usually-missing-a-datum-not-a-widget]].

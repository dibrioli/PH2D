//! **As perguntas ao ledger** — ver o doc do `mod consultas` no `lib.rs`.

#[allow(clippy::wildcard_imports)]
use super::*;

impl PreviewDrive {
    /// ⭐⭐⭐ **UM MOTOR ESTÁ A CONDUZIR ESTA ENTIDADE NESTE QUADRO?**
    ///
    /// ⛔⛔ **Ela existe por um laço fechado que a auditoria de 2026-09-08 mediu.** O cabeçalho do
    /// `autokey_pass` declara a invariante que o protege — *«the apply pass has already written the
    /// document's value to the world … world == curve and keys nothing — no feedback loop»* — e ela
    /// exige que **ninguém escreva pose entre o apply e o autokey**. Os passes do esqueleto (o osso
    /// inteligente, a âncora de IK) escrevem exactamente aí. Com o objecto conduzido **seleccionado**
    /// e o AutoKey armado, o autokey lê a saída do motor, ela difere da curva do clip activo, e ele
    /// cunha uma chave **a partir do que o motor está a mostrar**.
    ///
    /// ⇒ *o que um motor conduz é pré-visualização, e pré-visualização não é autoria.* Este ledger
    /// já sabe exactamente quem está sob condução — faltava alguém perguntar-lhe.
    ///
    /// ⚠️ Ela responde por ENTIDADE e não por `(entidade, driver)`: a pergunta do autokey é *«esta
    /// pose é do artista?»*, e basta um motor a conduzir para a resposta ser não.
    ///
    /// ⚠️⚠️ **Ela nasceu no sítio errado, e o defeito foi o que a auditoria acabara de nomear:** a
    /// 1.ª inserção caiu ENTRE o `#[cfg(test)]` da vizinha e a vizinha, e o método herdou-o — só
    /// existia em `cfg(test)`, e o produto não compilava. *Um item novo colado a um atributo rouba-o
    /// ao dono.*
    #[must_use]
    pub fn drives(&self, entity: u64) -> bool {
        self.memo.keys().any(|(bits, _)| *bits == entity)
    }

    /// ⭐⭐⭐ **O VALOR AUTORADO que este motor deslocou** — o que o documento diz, enquanto a cena
    /// mostra o que o motor escreveu.
    ///
    /// ⚠️ **Ela não é um atalho da [`Self::substitute_authored`], e a diferença é a razão de
    /// existir:** aquela põe o MUNDO INTEIRO no estado autorado e obriga a repor a seguir — é a
    /// porta da captura, e usá-la para ler dois ossos deslocaria todos os outros motores a meio do
    /// quadro. Esta responde por **uma** entidade sem tocar em nada.
    ///
    /// ⭐ O 1.º consumidor é o modo MISTO do esqueleto: *«cada osso mantém sua direção inicial»* só
    /// tem sentido se «inicial» for o que o artista DESENHOU. Lido da pose viva, o lado de uma junta
    /// é o que o solver deixou no quadro anterior — e um arrasto que leve o alvo para fora do
    /// alcance deita a corrente na recta e **apaga os lados para sempre**.
    #[must_use]
    pub fn authored(&self, entity: u64, driver: Driver) -> Option<Driven> {
        self.memo.get(&(entity, driver)).map(|e| e.authored)
    }

    /// ⭐⭐⭐ **O QUE ESTE MOTOR DEIXOU no quadro anterior** — a outra metade do par, e a que torna
    /// «outra mão mexeu» uma pergunta que um motor pode fazer **antes** de escrever.
    ///
    /// # ⛔⛔ Porque ela precisou de existir (a ponte da PARALAXE, W1)
    ///
    /// A [`Self::driven`] já responde a esta pergunta — **internamente e tarde demais**: ela
    /// compara `last_written` com o `before` que o motor lhe passa, e só então decide se o
    /// documento mudou de mão. Isso chega a um motor cuja saída **não depende do autorado** (o
    /// HUD: a pose da raiz é função pura da vista, e o autorado é descartado). Não chega a um cuja
    /// lei é `saída = autorada + f(vista)`: ele precisa de saber de que pose PARTIR, e a pose viva
    /// que ele encontra é a que ele próprio deslocou.
    ///
    /// ⚠️⚠️ **E a tentação é re-DERIVAR o deslocamento a partir da vista de AGORA — é errado, e o
    /// modo de falha é mudo.** O que lá está foi escrito com a vista do quadro ANTERIOR; com a
    /// câmera a andar as duas nunca coincidem, toda leitura se lê como *«outra mão mexeu»*, e o
    /// motor «recupera» um autorado que ninguém escreveu — na paralaxe isso lia declive **ZERO**
    /// com todos os outros gates verdes.
    ///
    /// ⭐ Com este par o deslocamento aplicado é `last_written − authored`, **medido e não
    /// re-derivado**: ele não depende da vista nenhuma, logo a recuperação do autorado é exacta
    /// mesmo com a câmera a mover-se entre os dois quadros.
    #[must_use]
    pub fn last_written(&self, entity: u64, driver: Driver) -> Option<Driven> {
        self.memo.get(&(entity, driver)).map(|e| e.last_written)
    }

    /// ⭐ **QUE MOTORES conduzem esta entidade agora** — a lista, para quem precisa de a **largar**
    /// e não só de saber que ela existe.
    ///
    /// ⚠️ Ela é o oráculo do censo que ata o `timeline_preview::DRIVERS` da shell ao que o
    /// `declare_timeline_writes` de facto escreve: *uma lista escrita à mão ao lado de um produtor
    /// é a segunda resposta à mesma pergunta, e a que envelhece é a escrita à mão*.
    ///
    /// ⚠️ `cfg(any(test, feature = "test-support"))` pela razão do [`Self::is_empty`], mais a
    /// fronteira de crate (HOWTO §2.5) — quem o lê é um gate da SHELL, e um `cfg(test)` não atravessa: no produto quem percorre os motores é a
    /// própria lista nomeada, e um método que só os gates usam é exactamente o que o clippy nomeia.
    #[cfg(any(test, feature = "test-support"))]
    #[must_use]
    pub fn drivers_of(&self, entity: u64) -> Vec<Driver> {
        self.memo
            .keys()
            .filter(|(bits, _)| *bits == entity)
            .map(|(_, d)| *d)
            .collect()
    }

    /// ⭐⭐ **QUEM ESTE MOTOR CONDUZ AGORA** — para um motor PERSISTENTE largar quem deixou de
    /// conduzir (append-only, auditoria 26 da paralaxe).
    ///
    /// ⛔⛔ **O defeito que ela fecha:** um condutor persistente que deixa de declarar um objecto
    /// (a câmera sumiu, o factor voltou ao neutro, o componente foi retirado) não pode confiar na
    /// [`Self::settle`] — ela só ESQUECE, e o valor vivo que fica no mundo é o que o motor
    /// escreveu. A captura seguinte grava-o como documento, e quando a condução volta o motor
    /// parte dele e desloca DUAS vezes. ⇒ quem sabe que já não conduz chama
    /// [`Self::release_to_authored`] sobre cada entidade desta lista que não declarou neste quadro.
    #[must_use]
    pub fn driven_by(&self, driver: Driver) -> Vec<Entity> {
        self.memo
            .keys()
            .filter(|(_, d)| *d == driver)
            .map(|(bits, _)| Entity::from_bits(*bits))
            .collect()
    }

    /// ⭐ **OUTRO motor conduz esta entidade?** — a pergunta do painel de um condutor que não
    /// COMPÕE com os outros (a paralaxe parte do autorado; um segundo motor no mesmo `Transform`
    /// corrompe-o). Auditoria 26, §2.7.
    #[must_use]
    pub fn drives_other_than(&self, entity: u64, driver: Driver) -> bool {
        self.memo
            .keys()
            .any(|(bits, d)| *bits == entity && *d != driver)
    }
}

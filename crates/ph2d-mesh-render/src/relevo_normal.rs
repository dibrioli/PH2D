//! ⭐⭐⭐ **A NORMAL INCLINADA PELO RELEVO, e o HORIZONTE que ela não passa** —
//! o dono da lei que o `tinta.wgsl` repete (`tinta_inclina`/`tinta_horizonte`),
//! conferido contra ele na placa (`tests/it/relevo_horizonte.rs`).
//!
//! Tudo em espaço de VISTA: `z > 0` é virado para o olho, que é a convenção do
//! `canvas_normal` do `mesh.wgsl`.
//!
//! ⛔⛔ **O horizonte é o report do dono de 01/10, 2.ª volta** (*«mesma ponta
//! vista de frente e inclinada»*, a seta na ponta do traço): perto do contorno
//! da peça a normal de BASE já está quase de lado, e a encosta da ponta do
//! traço inclina-a para FORA. Ela passa o horizonte (`z < 0`) e o
//! `canvas_normal` — que existe para uma casca vista por trás acender como
//! frente — vira-a INTEIRA (`n = −n`): o `z` volta positivo **e o lado para
//! onde ela aponta no ecrã troca**. A luz salta de uma borda do matcap para a
//! oposta numa linha só, que é a meia-lua dura da foto. De frente a mesma
//! ponta tem a base virada ao olho e nada chega ao horizonte — daí *«de frente
//! bom, inclinado não»*.
//!
//! ⭐⭐ **A cura é COMPRIMIR a inclinação antes do horizonte, nunca cortá-la:**
//! abaixo de `t = min(|n.z|, T)` o `z` da normal inclinada desce por uma
//! exponencial que toca `t` com derivada `1` (sem vinco na luz) e só se
//! aproxima de `K·t` — nunca o atravessa. ⚠️ O limiar sai da BASE e não de um
//! número fixo, e é isso que torna a cura inerte onde não há relevo: com
//! `corpo = 0` a normal inclinada É a base, `z = |n.z| ≥ t`, e sai **ao bit**.
//!
//! ⚠️ `T` e `K` não nomeiam recurso nenhum, e dizê-lo é a forma honesta (§0.0):
//! são a FORMA da compressão. `T = 0,25` (o relevo só é comprimido a menos de
//! ~14° do horizonte) e `K = 0,1` (o chão fica a um décimo do limiar, longe o
//! bastante do zero para o `canvas_normal` nunca hesitar).

/// Onde a compressão começa, em `z` da normal unitária (`cos` do ângulo ao
/// eixo da vista).
pub const HORIZONTE_T: f32 = 0.25;
/// O chão da compressão, em fracção do limiar.
pub const HORIZONTE_K: f32 = 0.1;

fn normaliza(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    [v[0] / l, v[1] / l, v[2] / l]
}

/// ⭐ **A compressão contra o horizonte** — `n` é a normal de BASE (unitária),
/// `nb` a inclinada (unitária). Devolve `nb` intacta se ela não desce abaixo
/// do limiar do lado de `n`.
#[must_use]
pub fn horizonte(n: [f32; 3], nb: [f32; 3]) -> [f32; 3] {
    let s = if n[2] < 0.0 { -1.0 } else { 1.0 };
    let t = n[2].abs().min(HORIZONTE_T);
    let z = nb[2] * s;
    if t <= 0.0 || z >= t {
        return nb;
    }
    let zmin = HORIZONTE_K * t;
    let zc = zmin + (t - zmin) * ((z - t) / (t - zmin)).exp();
    let r_old = (nb[0] * nb[0] + nb[1] * nb[1]).sqrt();
    let (mut x, mut y) = (nb[0], nb[1]);
    if r_old > 0.0 {
        let k = (1.0 - zc * zc).max(0.0).sqrt() / r_old;
        x *= k;
        y *= k;
    }
    [x, y, zc * s]
}

/// ⭐⭐ **A normal de vista inclinada pelo gradiente `gv` da altura** (já em
/// espaço de vista), pesada pelo `corpo` — o *gradiente de superfície* de
/// Mikkelsen (2020) seguido do [`horizonte`].
#[must_use]
pub fn inclina(n_in: [f32; 3], gv: [f32; 3], corpo: f32) -> [f32; 3] {
    let n = normaliza(n_in);
    let c = corpo.clamp(0.0, 1.0);
    // ⚠️ Sem corpo a base sai INTEIRA: `nb / |nb|` com `nb = n` erra um ULP e
    // cairia um ULP abaixo do limiar, que a compressão então tocaria.
    if c <= 0.0 {
        return n;
    }
    let d = n[0] * gv[0] + n[1] * gv[1] + n[2] * gv[2];
    let nb = [
        n[0] - c * (gv[0] - n[0] * d),
        n[1] - c * (gv[1] - n[1] * d),
        n[2] - c * (gv[2] - n[2] * d),
    ];
    let l = (nb[0] * nb[0] + nb[1] * nb[1] + nb[2] * nb[2]).sqrt();
    if l <= 0.0 {
        return n;
    }
    horizonte(n, [nb[0] / l, nb[1] / l, nb[2] / l])
}

#[cfg(test)]
#[path = "relevo_normal_tests.rs"]
mod tests;

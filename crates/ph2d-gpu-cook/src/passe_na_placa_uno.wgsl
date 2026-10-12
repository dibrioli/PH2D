// O passe INTEIRO num so' GRUPO (doc 130 §3.1): poucas pecas e muitas varreduras -- o regime da
// CADEIA (`~n^2` varreduras, doc 115 §18). Um despacho; as fases sincronizam por barreira e a
// paragem SAI do laco (nada de despachos vazios depois do repouso). O `scan` (e a `Ent` da entrada)
// vem de `passe_na_placa_scan_grupo.wgsl` ou, com subgrupos, de `passe_na_placa_scan_subgrupo.wgsl`.

var<workgroup> wg_andou: atomic<u32>;
var<workgroup> wg_parar: u32;

fn pnp_barreira() {
    storageBarrier();
    workgroupBarrier();
}

@compute @workgroup_size(256)
fn main(e: Ent) {
    let lid = e.lid;
    pnp_toca();
    let n = u.n;
    let nb = u.num_buckets;
    let alc_max = bitcast<f32>(atomicLoad(&ctrl[0]));
    let lado = 2.0 * alc_max;
    if (lid == 0u) {
        wg_parar = atomicLoad(&ctrl[2]);
        atomicStore(&wg_andou, 0u);
    }
    var bi = 0u;
    var v = 0u;
    loop {
        if (workgroupUniformLoad(&wg_parar) != 0u || v >= u.varreduras) { break; }
        let bo = n - bi;
        for (var b = lid; b <= nb; b = b + 256u) {
            atomicStore(&starts[b], 0u);
            if (b < nb) { atomicStore(&cursor[b], 0u); }
        }
        pnp_barreira();
        for (var i = lid; i < n; i = i + 256u) { pnp_gira_conta(i, bi, lado); }
        pnp_barreira();
        pnp_scan_grupo(e, nb + 1u);
        pnp_barreira();
        for (var i = lid; i < n; i = i + 256u) { pnp_espalha(i, bi, lado); }
        pnp_barreira();
        for (var b = lid; b < nb; b = b + 256u) { pnp_ordena(b); }
        pnp_barreira();
        for (var i = lid; i < n; i = i + 256u) {
            let a = pnp_corrige(i, bi, bo, alc_max);
            if (a >= 0.0) { atomicMax(&wg_andou, pnp_bits(a)); }
        }
        pnp_barreira();
        if (lid == 0u) {
            atomicAdd(&ctrl[3], 1u);
            let m = bitcast<f32>(atomicLoad(&wg_andou));
            atomicStore(&wg_andou, 0u);
            if (m == 0.0 || m < u.repouso * alc_max) {
                wg_parar = 1u;
                atomicStore(&ctrl[2], 1u);
            }
        }
        bi = bo;
        v = v + 1u;
    }
    // O fim mora na metade `bi`: leva-o a' base 0, que e' a que o `escreve` le.
    if (bi != 0u) {
        for (var i = lid; i < n; i = i + 256u) { est[i] = est[bi + i]; }
    }
}

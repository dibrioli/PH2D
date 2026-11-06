fn main() {
    for (nome, h) in sonda_w2::hashes() {
        println!("{nome}\t{h:016x}");
    }
    println!("conjunto\t{:016x}", sonda_w2::hash_conjunto());
}

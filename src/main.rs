fn main() {
    let asm = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let program = miffed::parse::parse_program(&asm).unwrap();
    dbg!(program);
}

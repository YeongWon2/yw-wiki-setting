// 실행 파일 버전 = 세팅 VERSION 파일 (위키 · 전역 스킬과 같은 번호)
fn main() {
    let v = std::fs::read_to_string("../VERSION").expect("../VERSION");
    println!("cargo:rustc-env=YW_VERSION={}", v.trim());
    println!("cargo:rerun-if-changed=../VERSION");
}

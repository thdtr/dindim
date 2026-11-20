fn main() {
    println!("cargo:rerun-if-changed=../../packaging/windows/dindim.rc");
    println!("cargo:rerun-if-changed=../../assets/icons/dindim.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        embed_resource::compile("../../packaging/windows/dindim.rc", embed_resource::NONE)
            .manifest_optional()
            .expect("failed to embed Windows resources");
    }
}

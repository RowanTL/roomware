fn main() {
    // CARGO_CFG_TARGET_OS reflects the target being built for, unlike
    // #[cfg(windows)], which would reflect the host running this script.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        embed_resource::compile("nix/roomware.rc", embed_resource::NONE)
            .manifest_required()
            .unwrap();
    }
}

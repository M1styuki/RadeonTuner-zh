fn main() {
    println!("cargo:rerun-if-changed=app-icon.rc");
    println!("cargo:rerun-if-changed=RadeonTuner/AppIcon.ico");
    embed_resource::compile("app-icon.rc", embed_resource::NONE)
        .manifest_required()
        .expect("could not embed the RadeonTuner application icon");
}

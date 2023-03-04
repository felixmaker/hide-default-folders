use embed_manifest::{
    embed_manifest,
    manifest::*,
    new_manifest,
};

fn main() {
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        let manifest = new_manifest("Contoso.Sample")
            .supported_os(SupportedOS::Windows10..)
            // .requested_execution_level(ExecutionLevel::RequireAdministrator)        
            ;

        embed_manifest(manifest).expect("unable to embed manifest file");
    }
    println!("cargo:rerun-if-changed=build.rs");
}

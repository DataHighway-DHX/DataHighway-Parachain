use substrate_build_script_utils::{generate_cargo_keys, rerun_if_git_head_changed};
// TODO - do we need this for parachain?
use vergen::{
    generate_cargo_keys as vergen_generate_cargo_keys,
    ConstantsFlags,
};

fn main() {
    // TODO - do we need this for parachain?
    vergen_generate_cargo_keys(ConstantsFlags::SHA_SHORT).expect("Failed to generate metadata files");
	generate_cargo_keys();

	rerun_if_git_head_changed();
}

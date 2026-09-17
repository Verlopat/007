fn main() {
    let mut cfg = cmake::Config::new("openfhe-bridge");
    cfg.profile(if cfg!(debug_assertions) { "Debug" } else { "Release" });
    cfg.out_dir("openfhe-bridge");
    cfg.build_target("openfhe_bridge");
    cfg.build();
}

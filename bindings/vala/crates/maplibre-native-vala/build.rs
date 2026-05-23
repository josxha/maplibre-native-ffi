fn main() {
    pkg_config::Config::new()
        .atleast_version("2.0")
        .probe("glib-2.0")
        .expect("glib-2.0 is required for the Vala binding adapter");
}

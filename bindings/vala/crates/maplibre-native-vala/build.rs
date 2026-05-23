fn main() {
    for package in ["glib-2.0", "gobject-2.0"] {
        pkg_config::Config::new()
            .atleast_version("2.0")
            .probe(package)
            .unwrap_or_else(|_| panic!("{package} is required for the Vala binding adapter"));
    }
}

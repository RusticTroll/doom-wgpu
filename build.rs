use std::path::Path;

fn main() {
    let ajbsp_path = Path::new("src/ajbsp/src");

    cc::Build::new()
        .cpp(true)
        .files([
            ajbsp_path.join("level.cc"),
            ajbsp_path.join("misc.cc"),
            ajbsp_path.join("node.cc"),
            ajbsp_path.join("parse.cc"),
            ajbsp_path.join("utility.cc"),
            ajbsp_path.join("wad.cc"),
        ])
        .compile("ajbsp");
}

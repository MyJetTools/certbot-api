fn main() {
    ci_utils::css::CssCompiler::new("./css")
        .add_file("01-tokens.css")
        .add_file("02-shell.css")
        .add_file("03-atoms.css")
        .add_file("04-tables.css")
        .compile("./public/assets/app.css");
}

use include_dir::{Dir, include_dir};
use minijinja::Environment;

static BUILT_IN_TEMPLATES_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/templates");

pub struct Template {
    pub path: String,
    pub contents: String,
}

pub fn get_built_in_templates() -> impl Iterator<Item = Template> {
    BUILT_IN_TEMPLATES_DIR
        .find("**/*")
        .expect("Failed to traverse embedded templates")
        .filter_map(|entry| {
            entry.as_file().map(|f| Template {
                path: f
                    .path()
                    .to_string_lossy()
                    .into_owned()
                    .replace("templates/", ""),
                contents: f
                    .contents_utf8()
                    .expect("Invalid UTF-8 in embedded template")
                    .to_string(),
            })
        })
}

pub fn build_environment() -> Environment<'static> {
    let mut env = Environment::new();

    for template in get_built_in_templates() {
        let name = template.path;
        env.add_template_owned(name, template.contents)
            .expect("Failed to add template");
    }

    env
}

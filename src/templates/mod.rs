#![allow(dead_code)]

use directories::ProjectDirs;
use include_dir::{Dir, include_dir};
use minijinja::Environment;
use std::{
    fs,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

/// Embedded templates compiled into the binary
static BUILT_IN_TEMPLATES_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/src/templates/built_in");

/// Where a template comes from
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TemplateSource {
    BuiltIn,
    User,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TemplateType {
    Verb,
}

#[derive(Clone, Debug)]
pub struct Template {
    pub path: String,
    pub contents: String,
    pub source: TemplateSource,
    pub template_type: TemplateType,
}

pub fn get_built_in_templates() -> impl Iterator<Item = Template> {
    BUILT_IN_TEMPLATES_DIR
        .find("**/*")
        .expect("Failed to traverse embedded templates")
        .filter_map(|entry| {
            entry.as_file().map(|f| Template {
                template_type: TemplateType::Verb,
                source: TemplateSource::BuiltIn,
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

fn templates_from_dir<P: AsRef<Path>>(
    base: P,
    source: TemplateSource,
) -> impl Iterator<Item = Template> {
    let base: PathBuf = base.as_ref().to_path_buf();

    WalkDir::new(&base)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter_map(move |entry| {
            let abs_path = entry.path();
            let rel = abs_path.strip_prefix(&base).ok()?;
            let contents = fs::read_to_string(abs_path).ok()?;

            Some(Template {
                template_type: TemplateType::Verb,
                source,
                path: rel.to_string_lossy().replace('\\', "/"),
                contents,
            })
        })
}

fn user_template_dir() -> Option<PathBuf> {
    ProjectDirs::from("", "", "lakonik")
        .map(|pd| pd.config_dir().join("templates"))
        .filter(|p| p.exists())
}

pub fn get_user_templates() -> impl Iterator<Item = Template> {
    user_template_dir()
        .into_iter()
        .flat_map(|dir| templates_from_dir(dir, TemplateSource::User))
}

pub fn build_environment() -> Environment<'static> {
    let mut env = Environment::new();

    // Built-ins first; user files can override.
    get_built_in_templates()
        .chain(get_user_templates())
        .for_each(|t| {
            env.add_template_owned(t.path, t.contents)
                .expect("Failed to add template");
        });

    env
}

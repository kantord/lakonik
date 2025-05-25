use minijinja::Environment;

pub fn build_environment() -> Environment<'static> {
    let mut env = Environment::new();

    let _ = env.add_template("create", "create for me a(n) {{description}}");

    env
}

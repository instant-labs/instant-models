use super::*;

#[derive(Debug, PartialEq)]
pub struct StructBuilder {
    pub name: Cow<'static, str>,
    pub columns: IndexMap<Cow<'static, str>, Column>,
    pub constraints: Vec<Constraint>,
}

impl Default for StructBuilder {
    fn default() -> Self {
        Self {
            name: String::new().into(),
            columns: IndexMap::new(),
            constraints: vec![],
        }
    }
}

impl StructBuilder {
    pub fn new(name: Cow<'static, str>) -> Self {
        Self {
            name,
            ..Self::default()
        }
    }

    pub fn add_column(&mut self, val: Column) -> &mut Self {
        self.columns.insert(val.name.clone(), val);
        self
    }

    pub fn build_type(&self) -> String {
        format!("{}", self)
    }

    pub fn build_new_type(&self) -> String {
        let columns = self.columns.values().filter(|c| !c.primary_key).fold(String::new(), |mut acc, col| {
            acc.push_str(&format!(
                "    {}",
                NewValue {
                    val: &col,
                    lifetime: Some("a")
                }
            ));
            acc.push('\n');
            acc
        });

        format!(
            r#"pub struct {}New<'a> {{
{}}}
        "#,
            AsUpperCamelCase(&self.name),
            columns
        )
    }

    pub fn build_new_type_methods(&self) -> String {
        let columns = self.columns.values().fold(String::new(), |mut acc, col| {
            acc.push_str(&format!(
                "    {}",
                NewValue {
                    val: &col,
                    lifetime: Some("a")
                }
            ));
            acc.push('\n');
            acc
        });

        format!(
            r#"pub struct {}New<'a> {{
{}}}
        "#,
            AsUpperCamelCase(&self.name),
            columns
        )
    }
}

impl std::fmt::Display for StructBuilder {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        let columns = self.columns.values().fold(String::new(), |mut acc, col| {
            acc.push_str(&format!("    {}", col));
            acc.push('\n');
            acc
        });
        write!(
            fmt,
            r#"pub struct {} {{
{}}}
        "#,
            AsUpperCamelCase(&self.name),
            columns
        )
    }
}


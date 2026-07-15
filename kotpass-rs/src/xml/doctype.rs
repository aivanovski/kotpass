use super::PrintOptions;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Doctype {
    pub name: String,
    pub system_id: Option<String>,
    pub public_id: Option<String>,
}

impl Doctype {
    pub fn new(
        name: impl Into<String>,
        public_id: Option<String>,
        system_id: Option<String>,
    ) -> Self {
        Self {
            name: name.into(),
            system_id,
            public_id,
        }
    }

    pub(crate) fn render(&self, builder: &mut String, print_options: &PrintOptions) {
        builder.push_str("<!DOCTYPE ");
        builder.push_str(&self.name);

        if let Some(public_id) = &self.public_id {
            builder.push_str(" PUBLIC \"");
            builder.push_str(public_id);
            builder.push('"');
        }

        if let Some(system_id) = &self.system_id {
            if self.public_id.is_none() {
                builder.push_str(" SYSTEM");
            }
            builder.push_str(" \"");
            builder.push_str(system_id);
            builder.push('"');
        }

        builder.push('>');
        builder.push_str(print_options.line_ending());
    }
}

use std::collections::HashMap;

use tera::{Error, Value};

pub fn render(value: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
    let Some(markdown) = value.as_str() else {
        return Err(Error::msg(format!(
            "invalid value: {value}, expected string"
        )));
    };

    Ok(crate::markdown::render(markdown)
        .map_err(|e| format!("{e:?}"))?
        .into())
}

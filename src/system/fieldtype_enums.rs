#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldType {
    Text,
    Email,
    Phone,
    Number,
    Currency,
    Checkbox,
    Date,
    CreatedAt,
    UpdatedAt,
}

impl FieldType {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "text" => Ok(FieldType::Text),
            "email" => Ok(FieldType::Email),
            "phone" => Ok(FieldType::Phone),
            "number" => Ok(FieldType::Number),
            "currency" => Ok(FieldType::Currency),
            "checkbox" => Ok(FieldType::Checkbox),
            "date" => Ok(FieldType::Date),
            "created_at" => Ok(FieldType::CreatedAt),
            "updated_at" => Ok(FieldType::UpdatedAt),
            _ => Err(format!("Unknown field type: {}", s)),
        }
    }

    pub fn column_name(&self) -> &'static str {
        match self {
            FieldType::Text | FieldType::Email | FieldType::Phone => "value_text",
            FieldType::Number | FieldType::Currency => "value_number",
            FieldType::Checkbox => "value_boolean",
            FieldType::Date | FieldType::CreatedAt | FieldType::UpdatedAt => "value_date",
        }
    }

    pub fn insert_query(&self) -> (&'static str, bool) {
        match self {
            FieldType::Text | FieldType::Email | FieldType::Phone => (
                "INSERT INTO cells (row_id, field_id, value_text) VALUES ($1, $2, '')",
                false,
            ),
            FieldType::Number | FieldType::Currency => (
                "INSERT INTO cells (row_id, field_id, value_number) VALUES ($1, $2, 0.00)",
                false,
            ),
            FieldType::Checkbox => (
                "INSERT INTO cells (row_id, field_id, value_boolean) VALUES ($1, $2, 0)",
                false,
            ),
            FieldType::Date => (
                "INSERT INTO cells (row_id, field_id, value_date) VALUES ($1, $2, null)",
                false,
            ),
            FieldType::CreatedAt | FieldType::UpdatedAt => (
                "INSERT INTO cells (row_id, field_id, value_date) VALUES ($1, $2, $3)",
                true,
            ),
        }
    }
}

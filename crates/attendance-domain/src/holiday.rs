use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HolidayType {
    NationalPublic,
    Religious,
    CompanyDeclared,
    OptionalObservance,
}

impl Default for HolidayType {
    fn default() -> Self {
        HolidayType::NationalPublic
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Holiday {
    pub id: Option<i64>,
    pub company_id: String,
    pub name: String,
    pub holiday_date: String, // YYYY-MM-DD
    pub holiday_type: HolidayType,
    pub is_recurring_yearly: bool,
    pub description: Option<String>,
}

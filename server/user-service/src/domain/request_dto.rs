use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    #[validate(length(min = 1, max = 255))]
    pub surname: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8, max = 255))]
    pub password: String,
    pub phone_number: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,
    #[validate(length(min = 1, max = 255))]
    pub surname: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
    pub phone_number: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateAddressRequest {
    #[validate(length(min = 1, max = 255))]
    pub street: String,
    #[validate(length(min = 1, max = 20))]
    pub building_num: String,
    pub floor_num: Option<i32>,
    pub apartment_num: Option<String>,
    #[validate(length(min = 1, max = 20))]
    pub post_index: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateAddressRequest {
    pub street: Option<String>,
    pub building_num: Option<String>,
    pub floor_num: Option<i32>,
    pub apartment_num: Option<String>,
    pub post_index: Option<String>,
}

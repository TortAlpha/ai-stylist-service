use crate::domain::address::Address;
use crate::domain::response_dto::{AddressResponse, UserAuthResponse, UserResponse};
use crate::domain::user::User;

pub fn user_to_response(u: &User) -> UserResponse {
    UserResponse {
        id: u.id,
        name: u.name.clone(),
        surname: u.surname.clone(),
        email: u.email.clone(),
        phone_number: u.phone_number.clone(),
        is_active: u.is_active,
        role: u.role.clone(),
        created_at: u.created_at,
        updated_at: u.updated_at,
    }
}

pub fn user_to_auth_response(u: &User) -> UserAuthResponse {
    UserAuthResponse {
        id: u.id,
        email: u.email.clone(),
        password: u.password.clone(),
        role: u.role.clone(),
        is_active: u.is_active,
    }
}

pub fn address_to_response(a: &Address) -> AddressResponse {
    AddressResponse {
        id: a.id,
        owner_id: a.owner_id,
        street: a.street.clone(),
        building_num: a.building_num.clone(),
        floor_num: a.floor_num,
        apartment_num: a.apartment_num.clone(),
        post_index: a.post_index.clone(),
        created_at: a.created_at,
        updated_at: a.updated_at,
    }
}

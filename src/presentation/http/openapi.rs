use crate::core::error::ErrorResponse;
use crate::domain::{
    auth::dto::*, product_categories::dto::*, product_categories::entity::*,
    product_materials::dto::*, product_materials::entity::*, products::dto::*, products::entity::*,
    users::dto::*, users::entity::*,
};
use crate::presentation::http::*;
use crate::shared::dto::{pagination::*, response::*};
use utoipa::{
    Modify, OpenApi,
    openapi::security::{Http, HttpAuthScheme, SecurityScheme},
};

#[derive(OpenApi)]
#[openapi(
    info(title = "Mebayu API", version = "1.0.0", description = "Mebayu API Documentation"),
    paths(
        auth_controller::get_profile,
        auth_controller::login,
        auth_controller::register,
        auth_controller::refresh_token,
        product_controller::get_all,
        product_controller::create,
        product_controller::get_upload_url,
        product_controller::get_by_id,
        product_controller::update,
        product_controller::delete,
        product_category_controller::get_all,
        product_category_controller::create,
        product_category_controller::get_by_id,
        product_category_controller::get_all_with_product_count,
        product_category_controller::update,
        product_category_controller::delete,
        product_material_controller::get_all,
        product_material_controller::create,
        product_material_controller::get_by_id,
        product_material_controller::update,
        product_material_controller::delete,
        user_controller::get_all,
        user_controller::get_by_id,
        user_controller::create,
        user_controller::update,
        user_controller::delete_user,
    ),
    components(
        schemas(
            AuthResponseDto, LoginDto, RefreshTokenDto, RegisterDto,
            CreateProductRequest, UpdateProductRequest, GetUploadUrlRequest, GetUploadUrlResponse, Product, ProductImage,
            CreateProductCategoryRequest, UpdateProductCategoryRequest, ProductCategory,
            CreateProductMaterialRequest, UpdateProductMaterialRequest, ProductMaterial,
            CreateUserDto, UpdateUserDto, UserResponseDto, UserRole,
            PaginationQuery, SortOrder, ErrorResponse,
            ApiResponse<Product>, ApiResponse<UserResponseDto>, ApiResponse<ProductCategory>, ApiResponse<ProductMaterial>, ApiResponse<GetUploadUrlResponse>,
            PaginationResponse<Vec<Product>>, PaginationResponse<Vec<ProductCategory>>, PaginationResponse<Vec<ProductMaterial>>, PaginationResponse<Vec<UserResponseDto>>
        )
    ),
    modifiers(&SecurityAddon),
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "jwt",
                SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
            )
        }
    }
}

use std::sync::Arc;

use crate::{
    core::config::Config,
    domain::{
        auth::service::AuthService, product_categories::service::ProductCategoryServiceImpl,
        product_materials::service::ProductMaterialServiceImpl,
        products::service::ProductServiceImpl, users::service::UserServiceImpl,
    },
    infrastructure::object_storage::s3::S3Service,
};

#[derive(Clone)]
pub struct AppState {
    pub product_service: Arc<ProductServiceImpl>,
    pub product_category_service: Arc<ProductCategoryServiceImpl>,
    pub product_material_service: Arc<ProductMaterialServiceImpl>,
    pub user_service: Arc<UserServiceImpl>,
    pub auth_service: Arc<AuthService>,
    pub redis_client: redis::Client,
    pub s3_service: Arc<S3Service>,
    pub config: Config,
}

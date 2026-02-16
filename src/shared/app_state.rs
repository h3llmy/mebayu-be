use std::sync::Arc;

use crate::{
    core::config::Config,
    domain::{
        auth::service::AuthService, product_categories::service::ProductCategoryServiceImpl,
        product_materials::service::ProductMaterialServiceImpl,
        products::service::ProductServiceImpl, users::service::UserServiceImpl,
    },
    infrastructure::repository::{
        product_category_repository_impl::ProductCategoryRepositoryImpl,
        product_material_repository_impl::ProductMaterialRepositoryImpl,
        product_repository_impl::ProductRepositoryImpl, user_repository_impl::UserRepositoryImpl,
    },
};

#[derive(Clone)]
pub struct AppState {
    pub product_service: Arc<ProductServiceImpl<ProductRepositoryImpl>>,
    pub product_category_service: Arc<ProductCategoryServiceImpl<ProductCategoryRepositoryImpl>>,
    pub product_material_service: Arc<ProductMaterialServiceImpl<ProductMaterialRepositoryImpl>>,
    pub user_service: Arc<UserServiceImpl<UserRepositoryImpl>>,
    pub auth_service: Arc<AuthService<UserRepositoryImpl>>,
    pub redis_client: redis::Client,
    pub config: Config,
}

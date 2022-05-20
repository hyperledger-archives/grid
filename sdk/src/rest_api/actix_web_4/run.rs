// Copyright 2018-2022 Cargill Incorporated
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use actix_web_4::web;
use actix_web_4::{web::Data, App, HttpServer};

use crate::error::InternalError;
#[cfg(feature = "proxy-run")]
use crate::proxy::ProxyClient;
#[cfg(feature = "rest-api-endpoint-agent")]
use crate::rest_api::actix_web_4::routes::agents;
#[cfg(feature = "rest-api-endpoint-batches")]
use crate::rest_api::actix_web_4::routes::batches;
#[cfg(feature = "rest-api-endpoint-location")]
use crate::rest_api::actix_web_4::routes::locations;
#[cfg(feature = "rest-api-endpoint-organization")]
use crate::rest_api::actix_web_4::routes::organizations;
#[cfg(feature = "rest-api-endpoint-product")]
use crate::rest_api::actix_web_4::routes::products;
#[cfg(feature = "rest-api-endpoint-purchase-order")]
use crate::rest_api::actix_web_4::routes::purchase_orders;
#[cfg(feature = "rest-api-endpoint-record")]
use crate::rest_api::actix_web_4::routes::records;
#[cfg(feature = "rest-api-endpoint-role")]
use crate::rest_api::actix_web_4::routes::roles;
#[cfg(feature = "rest-api-endpoint-schema")]
use crate::rest_api::actix_web_4::routes::schemas;
#[cfg(feature = "rest-api-endpoint-submit")]
use crate::rest_api::actix_web_4::routes::submit;
use crate::rest_api::actix_web_4::{KeyState, StoreState};

#[cfg(feature = "proxy-run")]
use super::routes::proxy_get;

pub async fn run(
    bind: &str,
    store_state: StoreState,
    key_state: KeyState,
    #[cfg(feature = "proxy-run")] proxy_client: Box<dyn ProxyClient>,
) -> Result<(), InternalError> {
    HttpServer::new(move || {
        #[allow(unused_mut)]
        let mut app = App::new()
            .app_data(Data::new(store_state.clone()))
            .app_data(Data::new(key_state.clone()));

        #[cfg(feature = "rest-api-endpoint-submit")]
        {
            app = app.route("/submit", web::post().to(submit::submit));
        }

        #[cfg(feature = "proxy-run")]
        {
            app = app
                .app_data(Data::new(proxy_client.cloned_box()))
                .default_service(web::get().to(proxy_get));
        }

        #[cfg(feature = "rest-api-endpoint-agent")]
        {
            app = app
                .route("/agent", web::get().to(agents::list_agents))
                .route("/agent/{public_key}", web::get().to(agents::get_agent));
        }

        #[cfg(feature = "rest-api-endpoint-batches")]
        {
            app = app
                .service(
                    web::resource("/batch_statuses")
                        .name("get_batch_statuses")
                        .route(web::get().to(batches::get_batch_statuses)),
                )
                .service(
                    web::resource("/batches")
                        .name("get_batch_statuses")
                        .route(web::post().to(batches::submit_batches)),
                )
        }

        #[cfg(feature = "rest-api-endpoint-location")]
        {
            app = app
                .route("/location", web::get().to(locations::list_locations))
                .route("/location/{id}", web::get().to(locations::get_location));
        }

        #[cfg(feature = "rest-api-endpoint-organization")]
        {
            app = app
                .route(
                    "/organization",
                    web::get().to(organizations::list_organizations),
                )
                .route(
                    "/organization/{id}",
                    web::get().to(organizations::get_organization),
                );
        }

        #[cfg(feature = "rest-api-endpoint-product")]
        {
            app = app
                .route("/product", web::get().to(products::list_products))
                .route("/product/{id}", web::get().to(products::get_product));
        }

        #[cfg(feature = "rest-api-endpoint-purchase-order")]
        {
            app = app
                .route(
                    "/purchase_order",
                    web::get().to(purchase_orders::list_purchase_orders),
                )
                .route(
                    "/purchase_order/{uid}",
                    web::get().to(purchase_orders::get_purchase_order),
                )
                .route(
                    "/purchase_order/{uid}/version",
                    web::get().to(purchase_orders::list_purchase_order_versions),
                )
                .route(
                    "/purchase_order/{uid}/version/{version_id}",
                    web::get().to(purchase_orders::get_purchase_order_version),
                )
                .route(
                    "/purchase_order/{uid}/version/{version_id}/revision",
                    web::get().to(purchase_orders::list_purchase_order_version_revisions),
                )
                .route(
                    "/purchase_order/{uid}/version/{version_id}/revision/latest",
                    web::get().to(purchase_orders::get_latest_revision_id),
                )
                .route(
                    "/purchase_order/{uid}/version/{version_id}/revision/{revision_number}",
                    web::get().to(purchase_orders::get_purchase_order_version_revision),
                );
        }

        #[cfg(feature = "rest-api-endpoint-record")]
        {
            app = app
                .route("/record", web::get().to(records::list_records))
                .route("/record/{record_id}", web::get().to(records::get_record))
                .route(
                    "record/{record_id}/property/{property_name}",
                    web::get().to(records::get_record_property_name),
                );
        }

        #[cfg(feature = "rest-api-endpoint-role")]
        {
            app = app
                .route(
                    "/role/{org_id}",
                    web::get().to(roles::list_roles_for_organization),
                )
                .route("/role/{org_id}/{name}", web::get().to(roles::get_role));
        }

        #[cfg(feature = "rest-api-endpoint-schema")]
        {
            app = app
                .route("/schema", web::get().to(schemas::list_schemas))
                .route("/schema/{name}", web::get().to(schemas::get_schema));
        }

        app
    })
    .bind(bind)
    .map_err(|err| InternalError::from_source(Box::new(err)))?
    .run()
    .await
    .map_err(|err| InternalError::from_source(Box::new(err)))
}

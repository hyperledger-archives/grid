// Copyright 2018-2021 Cargill Incorporated
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

use crate::commits::MAX_COMMIT_NUM;
use crate::error::InternalError;
use crate::purchase_order::store::diesel::{
    models::PurchaseOrderAlternateIdModel, schema::purchase_order_alternate_id,
};

use crate::purchase_order::store::PurchaseOrderStoreError;
use diesel::{prelude::*, result::Error::NotFound};

#[cfg(feature = "postgres")]
pub(in crate) mod pg {
    use super::*;

    pub fn get_uid_from_alternate_id(
        conn: &diesel::pg::PgConnection,
        alternate_id: &str,
        service_id: Option<&str>,
    ) -> Result<String, PurchaseOrderStoreError> {
        if !alternate_id.contains(':') {
            return Err(PurchaseOrderStoreError::InternalError(
                InternalError::with_message(format!(
                    "Could not find alternate ID {}.
                    Alternate IDs must be in the format <id_type>:<id>",
                    alternate_id
                )),
            ));
        }
        let split: Vec<&str> = alternate_id.split(':').collect();
        let id_type = split[0];
        let id = split[1];

        let mut query = purchase_order_alternate_id::table.into_boxed().filter(
            purchase_order_alternate_id::alternate_id_type
                .eq(&id_type)
                .and(purchase_order_alternate_id::alternate_id.eq(&id))
                .and(purchase_order_alternate_id::end_commit_num.eq(MAX_COMMIT_NUM)),
        );

        if let Some(service_id) = &service_id {
            query = query.filter(purchase_order_alternate_id::service_id.eq(service_id));
        } else {
            query = query.filter(purchase_order_alternate_id::service_id.is_null());
        }

        let alt_id_model = query
            .first::<PurchaseOrderAlternateIdModel>(conn)
            .map_err(|err| match err {
                NotFound => PurchaseOrderStoreError::NotFoundError(format!(
                    "Could not find alternate ID {}",
                    alternate_id
                )),
                _ => PurchaseOrderStoreError::InternalError(InternalError::from_source(Box::new(
                    err,
                ))),
            })?;

        Ok(alt_id_model.purchase_order_uid)
    }
}

#[cfg(feature = "sqlite")]
pub(in crate) mod sqlite {
    use super::*;

    pub fn get_uid_from_alternate_id(
        conn: &diesel::sqlite::SqliteConnection,
        alternate_id: &str,
        service_id: Option<&str>,
    ) -> Result<String, PurchaseOrderStoreError> {
        if !alternate_id.contains(':') {
            return Err(PurchaseOrderStoreError::InternalError(
                InternalError::with_message(format!(
                    "Could not find alternate ID {}.
                Alternate IDs must be in the format <id_type>:<id>",
                    alternate_id
                )),
            ));
        }
        let split: Vec<&str> = alternate_id.split(':').collect();
        let id_type = split[0];
        let id = split[1];

        let mut query = purchase_order_alternate_id::table.into_boxed().filter(
            purchase_order_alternate_id::alternate_id_type
                .eq(&id_type)
                .and(purchase_order_alternate_id::alternate_id.eq(&id))
                .and(purchase_order_alternate_id::end_commit_num.eq(MAX_COMMIT_NUM)),
        );

        if let Some(service_id) = &service_id {
            query = query.filter(purchase_order_alternate_id::service_id.eq(service_id));
        } else {
            query = query.filter(purchase_order_alternate_id::service_id.is_null());
        }

        let alt_id_model = query
            .first::<PurchaseOrderAlternateIdModel>(conn)
            .map_err(|err| match err {
                NotFound => PurchaseOrderStoreError::NotFoundError(format!(
                    "Could not find alternate ID {}",
                    alternate_id
                )),
                _ => PurchaseOrderStoreError::InternalError(InternalError::from_source(Box::new(
                    err,
                ))),
            })?;

        Ok(alt_id_model.purchase_order_uid)
    }
}

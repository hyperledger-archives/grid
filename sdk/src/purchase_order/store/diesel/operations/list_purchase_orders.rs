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

use super::{get_uid_from_alternate_id, PurchaseOrderStoreOperations};
use crate::commits::MAX_COMMIT_NUM;
use crate::error::InternalError;
use crate::paging::Paging;
use crate::purchase_order::store::diesel::{
    models::{
        PurchaseOrderAlternateIdModel, PurchaseOrderModel, PurchaseOrderVersionModel,
        PurchaseOrderVersionRevisionModel,
    },
    schema::{
        purchase_order, purchase_order_alternate_id, purchase_order_version,
        purchase_order_version_revision,
    },
    ListPOFilters, PurchaseOrder, PurchaseOrderAlternateId, PurchaseOrderList,
    PurchaseOrderVersion,
};

use crate::purchase_order::store::PurchaseOrderStoreError;
use diesel::dsl::*;
use diesel::prelude::*;

pub(in crate::purchase_order::store::diesel) trait PurchaseOrderStoreListPurchaseOrdersOperation {
    fn list_purchase_orders(
        &self,
        filters: ListPOFilters,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<PurchaseOrderList, PurchaseOrderStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> PurchaseOrderStoreListPurchaseOrdersOperation
    for PurchaseOrderStoreOperations<'a, diesel::pg::PgConnection>
{
    fn list_purchase_orders(
        &self,
        filters: ListPOFilters,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<PurchaseOrderList, PurchaseOrderStoreError> {
        self.conn.transaction::<_, PurchaseOrderStoreError, _>(|| {
            let ListPOFilters {
                buyer_org_id,
                seller_org_id,
                has_accepted_version,
                is_open,
                alternate_ids,
            } = filters;

            let mut uids: Vec<String> = Vec::new();

            if let Some(ref alternate_ids) = alternate_ids {
                let id_vec = alternate_ids
                    .split(',')
                    .map(String::from)
                    .collect::<Vec<String>>();
                uids = id_vec
                    .iter()
                    .filter_map(
                        |id| match get_uid_from_alternate_id::pg::get_uid_from_alternate_id(
                            self.conn, id, service_id,
                        ) {
                            Err(PurchaseOrderStoreError::NotFoundError(_)) => None,
                            other => Some(other),
                        },
                    )
                    .collect::<Result<_, _>>()?;
            }

            let mut query = purchase_order::table
                .into_boxed()
                .select(purchase_order::all_columns)
                .offset(offset)
                .limit(limit)
                .filter(purchase_order::end_commit_num.eq(MAX_COMMIT_NUM));

            if !uids.is_empty() {
                query = query.filter(purchase_order::purchase_order_uid.eq(any(uids)));
            } else if uids.is_empty() && alternate_ids.is_some() {
                return Ok(PurchaseOrderList::new(
                    Vec::new(),
                    Paging::new(offset, limit, 0),
                ));
            }

            if let Some(has_accepted_version) = has_accepted_version {
                if has_accepted_version {
                    query = query.filter(purchase_order::accepted_version_id.is_not_null())
                } else {
                    query = query.filter(purchase_order::accepted_version_id.is_null())
                }
            }

            if let Some(is_open) = is_open {
                query = query.filter(purchase_order::is_closed.eq(!is_open))
            }

            if let Some(buyer_org_id) = buyer_org_id {
                query = query.filter(purchase_order::buyer_org_id.eq(buyer_org_id))
            }

            if let Some(seller_org_id) = seller_org_id {
                query = query.filter(purchase_order::seller_org_id.eq(seller_org_id))
            }

            if let Some(service_id) = service_id {
                query = query.filter(purchase_order::service_id.eq(service_id));
            } else {
                query = query.filter(purchase_order::service_id.is_null());
            }

            let purchase_order_models =
                query.load::<PurchaseOrderModel>(self.conn).map_err(|err| {
                    PurchaseOrderStoreError::InternalError(InternalError::from_source(Box::new(
                        err,
                    )))
                })?;

            let mut count_query = purchase_order::table
                .into_boxed()
                .select(purchase_order::all_columns);

            if let Some(service_id) = service_id {
                count_query = count_query.filter(purchase_order::service_id.eq(service_id));
            } else {
                count_query = count_query.filter(purchase_order::service_id.is_null());
            }

            let total = count_query.count().get_result(self.conn)?;

            let mut orders = Vec::new();

            for o in purchase_order_models {
                let mut query = purchase_order_version::table
                    .into_boxed()
                    .select(purchase_order_version::all_columns)
                    .filter(
                        purchase_order_version::purchase_order_uid
                            .eq(&o.purchase_order_uid)
                            .and(purchase_order_version::end_commit_num.eq(MAX_COMMIT_NUM)),
                    );

                if let Some(service_id) = service_id {
                    query = query.filter(purchase_order_version::service_id.eq(service_id));
                } else {
                    query = query.filter(purchase_order_version::service_id.is_null());
                }

                let version_models =
                    query
                        .load::<PurchaseOrderVersionModel>(self.conn)
                        .map_err(|err| {
                            PurchaseOrderStoreError::InternalError(InternalError::from_source(
                                Box::new(err),
                            ))
                        })?;

                let mut versions = Vec::new();
                let mut alternate_ids = Vec::new();

                for v in version_models {
                    let mut query = purchase_order_version_revision::table
                        .into_boxed()
                        .select(purchase_order_version_revision::all_columns)
                        .filter(
                            purchase_order_version_revision::purchase_order_uid
                                .eq(&o.purchase_order_uid)
                                .and(purchase_order_version_revision::version_id.eq(&v.version_id))
                                .and(
                                    purchase_order_version_revision::end_commit_num
                                        .eq(MAX_COMMIT_NUM),
                                ),
                        );

                    if let Some(service_id) = service_id {
                        query = query
                            .filter(purchase_order_version_revision::service_id.eq(service_id));
                    } else {
                        query = query.filter(purchase_order_version_revision::service_id.is_null());
                    }

                    let revision_models = query
                        .load::<PurchaseOrderVersionRevisionModel>(self.conn)
                        .map_err(|err| {
                            PurchaseOrderStoreError::InternalError(InternalError::from_source(
                                Box::new(err),
                            ))
                        })?;

                    versions.push(PurchaseOrderVersion::from((&v, &revision_models)));
                }

                let mut query = purchase_order_alternate_id::table
                    .into_boxed()
                    .select(purchase_order_alternate_id::all_columns)
                    .filter(
                        purchase_order_alternate_id::purchase_order_uid
                            .eq(&o.purchase_order_uid)
                            .and(purchase_order_alternate_id::end_commit_num.eq(MAX_COMMIT_NUM)),
                    );

                if let Some(service_id) = service_id {
                    query = query.filter(purchase_order_alternate_id::service_id.eq(service_id));
                } else {
                    query = query.filter(purchase_order_alternate_id::service_id.is_null());
                }

                let alt_id_models = query
                    .load::<PurchaseOrderAlternateIdModel>(self.conn)
                    .map_err(|err| {
                        PurchaseOrderStoreError::InternalError(InternalError::from_source(
                            Box::new(err),
                        ))
                    })?;

                alt_id_models.iter().for_each(|id| {
                    alternate_ids.push(PurchaseOrderAlternateId::from(id));
                });

                orders.push(PurchaseOrder::from((o, versions, alternate_ids)));
            }

            Ok(PurchaseOrderList::new(
                orders,
                Paging::new(offset, limit, total),
            ))
        })
    }
}

#[cfg(feature = "sqlite")]
impl<'a> PurchaseOrderStoreListPurchaseOrdersOperation
    for PurchaseOrderStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn list_purchase_orders(
        &self,
        filters: ListPOFilters,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<PurchaseOrderList, PurchaseOrderStoreError> {
        self.conn.transaction::<_, PurchaseOrderStoreError, _>(|| {
            let ListPOFilters {
                buyer_org_id,
                seller_org_id,
                has_accepted_version,
                is_open,
                alternate_ids,
            } = filters;

            let mut uids: Vec<String> = Vec::new();

            if let Some(ref alternate_ids) = alternate_ids {
                let id_vec = alternate_ids
                    .split(',')
                    .map(String::from)
                    .collect::<Vec<String>>();
                uids = id_vec
                    .iter()
                    .filter_map(|id| {
                        match get_uid_from_alternate_id::sqlite::get_uid_from_alternate_id(
                            self.conn, id, service_id,
                        ) {
                            Err(PurchaseOrderStoreError::NotFoundError(_)) => None,
                            other => Some(other),
                        }
                    })
                    .collect::<Result<_, _>>()?;
            }

            let mut query = purchase_order::table
                .into_boxed()
                .select(purchase_order::all_columns)
                .offset(offset)
                .limit(limit)
                .filter(purchase_order::end_commit_num.eq(MAX_COMMIT_NUM));

            if !uids.is_empty() {
                query = query.filter(purchase_order::purchase_order_uid.eq_any(uids));
            } else if uids.is_empty() && alternate_ids.is_some() {
                return Ok(PurchaseOrderList::new(
                    Vec::new(),
                    Paging::new(offset, limit, 0),
                ));
            }

            if let Some(has_accepted_version) = has_accepted_version {
                if has_accepted_version {
                    query = query.filter(purchase_order::accepted_version_id.is_not_null())
                } else {
                    query = query.filter(purchase_order::accepted_version_id.is_null())
                }
            }

            if let Some(is_open) = is_open {
                query = query.filter(purchase_order::is_closed.eq(!is_open))
            }

            if let Some(buyer_org_id) = buyer_org_id {
                query = query.filter(purchase_order::buyer_org_id.eq(buyer_org_id))
            }

            if let Some(seller_org_id) = seller_org_id {
                query = query.filter(purchase_order::seller_org_id.eq(seller_org_id))
            }

            if let Some(service_id) = service_id {
                query = query.filter(purchase_order::service_id.eq(service_id));
            } else {
                query = query.filter(purchase_order::service_id.is_null());
            }

            let purchase_order_models =
                query.load::<PurchaseOrderModel>(self.conn).map_err(|err| {
                    PurchaseOrderStoreError::InternalError(InternalError::from_source(Box::new(
                        err,
                    )))
                })?;

            let mut count_query = purchase_order::table
                .into_boxed()
                .select(purchase_order::all_columns);

            if let Some(service_id) = service_id {
                count_query = count_query.filter(purchase_order::service_id.eq(service_id));
            } else {
                count_query = count_query.filter(purchase_order::service_id.is_null());
            }

            let total = count_query.count().get_result(self.conn)?;

            let mut orders = Vec::new();

            for o in purchase_order_models {
                let mut query = purchase_order_version::table
                    .into_boxed()
                    .select(purchase_order_version::all_columns)
                    .filter(
                        purchase_order_version::purchase_order_uid
                            .eq(&o.purchase_order_uid)
                            .and(purchase_order_version::end_commit_num.eq(MAX_COMMIT_NUM)),
                    );

                if let Some(service_id) = service_id {
                    query = query.filter(purchase_order_version::service_id.eq(service_id));
                } else {
                    query = query.filter(purchase_order_version::service_id.is_null());
                }

                let version_models =
                    query
                        .load::<PurchaseOrderVersionModel>(self.conn)
                        .map_err(|err| {
                            PurchaseOrderStoreError::InternalError(InternalError::from_source(
                                Box::new(err),
                            ))
                        })?;

                let mut versions = Vec::new();
                let mut alternate_ids = Vec::new();

                for v in version_models {
                    let mut query = purchase_order_version_revision::table
                        .into_boxed()
                        .select(purchase_order_version_revision::all_columns)
                        .filter(
                            purchase_order_version_revision::purchase_order_uid
                                .eq(&o.purchase_order_uid)
                                .and(purchase_order_version_revision::version_id.eq(&v.version_id))
                                .and(
                                    purchase_order_version_revision::end_commit_num
                                        .eq(MAX_COMMIT_NUM),
                                ),
                        );

                    if let Some(service_id) = service_id {
                        query = query
                            .filter(purchase_order_version_revision::service_id.eq(service_id));
                    } else {
                        query = query.filter(purchase_order_version_revision::service_id.is_null());
                    }

                    let revision_models = query
                        .load::<PurchaseOrderVersionRevisionModel>(self.conn)
                        .map_err(|err| {
                            PurchaseOrderStoreError::InternalError(InternalError::from_source(
                                Box::new(err),
                            ))
                        })?;

                    versions.push(PurchaseOrderVersion::from((&v, &revision_models)));
                }

                let mut query = purchase_order_alternate_id::table
                    .into_boxed()
                    .select(purchase_order_alternate_id::all_columns)
                    .filter(
                        purchase_order_alternate_id::purchase_order_uid
                            .eq(&o.purchase_order_uid)
                            .and(purchase_order_alternate_id::end_commit_num.eq(MAX_COMMIT_NUM)),
                    );

                if let Some(service_id) = service_id {
                    query = query.filter(purchase_order_alternate_id::service_id.eq(service_id));
                } else {
                    query = query.filter(purchase_order_alternate_id::service_id.is_null());
                }

                let alt_id_models = query
                    .load::<PurchaseOrderAlternateIdModel>(self.conn)
                    .map_err(|err| {
                        PurchaseOrderStoreError::InternalError(InternalError::from_source(
                            Box::new(err),
                        ))
                    })?;

                alt_id_models.iter().for_each(|id| {
                    alternate_ids.push(PurchaseOrderAlternateId::from(id));
                });

                orders.push(PurchaseOrder::from((o, versions, alternate_ids)));
            }

            Ok(PurchaseOrderList::new(
                orders,
                Paging::new(offset, limit, total),
            ))
        })
    }
}

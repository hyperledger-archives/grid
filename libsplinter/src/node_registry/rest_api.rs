// Copyright 2018-2020 Cargill Incorporated
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

use std::collections::HashMap;

use crate::actix_web::{error::BlockingError, web, Error, HttpRequest, HttpResponse};
use crate::futures::{future::IntoFuture, stream::Stream, Future};
use crate::protocol;
use crate::rest_api::{
    paging::{get_response_paging_info, Paging, DEFAULT_LIMIT, DEFAULT_OFFSET},
    percent_encode_filter_query, Method, ProtocolVersionRangeGuard, Resource,
};

use super::{
    error::{InvalidNodeError, NodeRegistryError},
    MetadataPredicate, Node, NodeRegistryReader, NodeRegistryWriter,
};

type Filter = HashMap<String, (String, String)>;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ListNodesResponse {
    data: Vec<Node>,
    paging: Paging,
}

pub fn make_nodes_identity_resource<N>(registry: N) -> Resource
where
    N: NodeRegistryReader + NodeRegistryWriter + Clone + 'static,
{
    let registry1 = registry.clone();
    let registry2 = registry.clone();
    Resource::build("/admin/nodes/{identity}")
        .add_request_guard(ProtocolVersionRangeGuard::new(
            protocol::ADMIN_FETCH_NODE_MIN,
            protocol::ADMIN_PROTOCOL_VERSION,
        ))
        .add_method(Method::Get, move |r, _| {
            fetch_node(r, web::Data::new(registry.clone()))
        })
        .add_method(Method::Put, move |r, p| {
            put_node(r, p, web::Data::new(registry1.clone()))
        })
        .add_method(Method::Delete, move |r, _| {
            delete_node(r, web::Data::new(registry2.clone()))
        })
}

pub fn make_nodes_resource<N>(registry: N) -> Resource
where
    N: NodeRegistryReader + NodeRegistryWriter + Clone + 'static,
{
    let registry1 = registry.clone();
    Resource::build("/admin/nodes")
        .add_request_guard(ProtocolVersionRangeGuard::new(
            protocol::ADMIN_LIST_NODES_MIN,
            protocol::ADMIN_PROTOCOL_VERSION,
        ))
        .add_method(Method::Get, move |r, _| {
            list_nodes(r, web::Data::new(registry.clone()))
        })
        .add_method(Method::Post, move |_, p| {
            add_node(p, web::Data::new(registry1.clone()))
        })
}

fn fetch_node<NR>(
    request: HttpRequest,
    registry: web::Data<NR>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>>
where
    NR: NodeRegistryReader + 'static,
{
    let identity = request
        .match_info()
        .get("identity")
        .unwrap_or("")
        .to_string();
    Box::new(
        web::block(move || registry.fetch_node(&identity)).then(|res| match res {
            Ok(node) => Ok(HttpResponse::Ok().json(node)),
            Err(err) => match err {
                BlockingError::Error(err) => match err {
                    NodeRegistryError::NotFoundError(err) => Ok(HttpResponse::NotFound().json(err)),
                    _ => Ok(HttpResponse::InternalServerError().json(format!("{}", err))),
                },
                _ => Ok(HttpResponse::InternalServerError().json(format!("{}", err))),
            },
        }),
    )
}

fn put_node<NW>(
    request: HttpRequest,
    payload: web::Payload,
    registry: web::Data<NW>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>>
where
    NW: NodeRegistryWriter + 'static,
{
    let path_identity = request
        .match_info()
        .get("identity")
        .unwrap_or("")
        .to_string();
    Box::new(
        payload
            .from_err::<Error>()
            .fold(web::BytesMut::new(), move |mut body, chunk| {
                body.extend_from_slice(&chunk);
                Ok::<_, Error>(body)
            })
            .into_future()
            .and_then(move |body| match serde_json::from_slice::<Node>(&body) {
                Ok(node) => Box::new(
                    web::block(move || {
                        if node.identity != path_identity {
                            Err(NodeRegistryError::InvalidNode(
                                InvalidNodeError::InvalidIdentity(
                                    node.identity,
                                    "node identity cannot be changed".into(),
                                ),
                            ))
                        } else {
                            registry.insert_node(node)
                        }
                    })
                    .then(|res| {
                        Ok(match res {
                            Ok(_) => HttpResponse::Ok().finish(),
                            Err(err) => match err {
                                BlockingError::Error(err) => match err {
                                    NodeRegistryError::InvalidNode(err) => {
                                        HttpResponse::Forbidden()
                                            .json(format!("node is invalid: {}", err))
                                    }
                                    _ => {
                                        HttpResponse::InternalServerError().json(format!("{}", err))
                                    }
                                },
                                _ => HttpResponse::InternalServerError().json(format!("{}", err)),
                            },
                        })
                    }),
                )
                    as Box<dyn Future<Item = HttpResponse, Error = Error>>,
                Err(err) => Box::new(
                    HttpResponse::BadRequest()
                        .json(format!("invalid node: {}", err))
                        .into_future(),
                ),
            }),
    )
}

fn delete_node<NW>(
    request: HttpRequest,
    registry: web::Data<NW>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>>
where
    NW: NodeRegistryWriter + 'static,
{
    let identity = request
        .match_info()
        .get("identity")
        .unwrap_or("")
        .to_string();
    Box::new(
        web::block(move || registry.delete_node(&identity)).then(|res| match res {
            Ok(_) => Ok(HttpResponse::Ok().finish()),
            Err(err) => match err {
                BlockingError::Error(err) => match err {
                    NodeRegistryError::NotFoundError(err) => Ok(HttpResponse::NotFound().json(err)),
                    _ => Ok(HttpResponse::InternalServerError().json(format!("{}", err))),
                },
                _ => Ok(HttpResponse::InternalServerError().json(format!("{}", err))),
            },
        }),
    )
}

fn list_nodes<NR>(
    req: HttpRequest,
    registry: web::Data<NR>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>>
where
    NR: NodeRegistryReader + 'static,
{
    let query: web::Query<HashMap<String, String>> =
        if let Ok(q) = web::Query::from_query(req.query_string()) {
            q
        } else {
            return Box::new(
                HttpResponse::BadRequest()
                    .json(json!({
                        "message": "Invalid query"
                    }))
                    .into_future(),
            );
        };

    let offset = match query.get("offset") {
        Some(value) => match value.parse::<usize>() {
            Ok(val) => val,
            Err(err) => {
                return Box::new(
                    HttpResponse::BadRequest()
                        .json(format!(
                            "Invalid offset value passed: {}. Error: {}",
                            value, err
                        ))
                        .into_future(),
                )
            }
        },
        None => DEFAULT_OFFSET,
    };

    let limit = match query.get("limit") {
        Some(value) => match value.parse::<usize>() {
            Ok(val) => val,
            Err(err) => {
                return Box::new(
                    HttpResponse::BadRequest()
                        .json(format!(
                            "Invalid limit value passed: {}. Error: {}",
                            value, err
                        ))
                        .into_future(),
                )
            }
        },
        None => DEFAULT_LIMIT,
    };

    let mut link = format!("{}?", req.uri().path());

    let filters = match query.get("filter") {
        Some(value) => match serde_json::from_str(value) {
            Ok(val) => {
                link.push_str(&format!("filter={}&", percent_encode_filter_query(value)));
                Some(val)
            }
            Err(err) => {
                return Box::new(
                    HttpResponse::BadRequest()
                        .json(format!(
                            "Invalid filter value passed: {}. Error: {}",
                            value, err
                        ))
                        .into_future(),
                )
            }
        },
        None => None,
    };

    let predicates = match to_predicates(filters) {
        Ok(predicates) => predicates,
        Err(err) => return Box::new(HttpResponse::BadRequest().json(err).into_future()),
    };

    Box::new(query_list_nodes(
        registry,
        link,
        predicates,
        Some(offset),
        Some(limit),
    ))
}

fn query_list_nodes<NR>(
    registry: web::Data<NR>,
    link: String,
    filters: Vec<MetadataPredicate>,
    offset: Option<usize>,
    limit: Option<usize>,
) -> impl Future<Item = HttpResponse, Error = Error>
where
    NR: NodeRegistryReader + 'static,
{
    let count_filters = filters.clone();
    web::block(move || match registry.count_nodes(&count_filters) {
        Ok(count) => Ok((registry, count)),
        Err(err) => Err(err),
    })
    .and_then(move |(registry, total_count)| {
        web::block(move || match registry.list_nodes(&filters) {
            Ok(nodes_iter) => Ok(ListNodesResponse {
                data: nodes_iter
                    .skip(offset.as_ref().copied().unwrap_or(0))
                    .take(limit.as_ref().copied().unwrap_or(std::usize::MAX))
                    .collect::<Vec<_>>(),
                paging: get_response_paging_info(limit, offset, &link, total_count as usize),
            }),
            Err(err) => Err(err),
        })
    })
    .then(|res| match res {
        Ok(list_res) => Ok(HttpResponse::Ok().json(list_res)),
        Err(err) => {
            error!("Unable to list nodes: {}", err);
            Ok(HttpResponse::InternalServerError().into())
        }
    })
}

fn to_predicates(filters: Option<Filter>) -> Result<Vec<MetadataPredicate>, String> {
    match filters {
        Some(filters) => filters
            .into_iter()
            .map(|(key, (operator, value))| match operator.as_str() {
                "=" => Ok(MetadataPredicate::Eq(key, value)),
                ">" => Ok(MetadataPredicate::Gt(key, value)),
                "<" => Ok(MetadataPredicate::Lt(key, value)),
                ">=" => Ok(MetadataPredicate::Ge(key, value)),
                "<=" => Ok(MetadataPredicate::Le(key, value)),
                "!=" => Ok(MetadataPredicate::Ne(key, value)),
                _ => Err(format!("{} is not a valid operator", operator)),
            })
            .collect(),
        None => Ok(vec![]),
    }
}

fn add_node<NW>(
    payload: web::Payload,
    registry: web::Data<NW>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>>
where
    NW: NodeRegistryReader + NodeRegistryWriter + 'static,
{
    Box::new(
        payload
            .from_err::<Error>()
            .fold(web::BytesMut::new(), move |mut body, chunk| {
                body.extend_from_slice(&chunk);
                Ok::<_, Error>(body)
            })
            .into_future()
            .and_then(move |body| match serde_json::from_slice::<Node>(&body) {
                Ok(node) => Box::new(
                    web::block(move || {
                        if registry.has_node(&node.identity)? {
                            Err(NodeRegistryError::InvalidNode(
                                InvalidNodeError::DuplicateIdentity(node.identity),
                            ))
                        } else {
                            registry.insert_node(node)
                        }
                    })
                    .then(|res| {
                        Ok(match res {
                            Ok(_) => HttpResponse::Ok().finish(),
                            Err(err) => match err {
                                BlockingError::Error(err) => match err {
                                    NodeRegistryError::InvalidNode(err) => {
                                        HttpResponse::Forbidden()
                                            .json(format!("node is invalid: {}", err))
                                    }
                                    _ => {
                                        HttpResponse::InternalServerError().json(format!("{}", err))
                                    }
                                },
                                _ => HttpResponse::InternalServerError().json(format!("{}", err)),
                            },
                        })
                    }),
                )
                    as Box<dyn Future<Item = HttpResponse, Error = Error>>,
                Err(err) => Box::new(
                    HttpResponse::BadRequest()
                        .json(format!("invalid node: {}", err))
                        .into_future(),
                ),
            }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::env;
    use std::fs::{remove_file, File};
    use std::panic;
    use std::thread;

    use crate::actix_web::{
        http::{header, StatusCode},
        test, web, App,
    };
    use crate::node_registry::yaml::YamlNodeRegistry;

    fn new_yaml_node_registry(file_path: &str) -> YamlNodeRegistry {
        YamlNodeRegistry::new(file_path).expect("Error creating YamlNodeRegistry")
    }

    #[test]
    /// Tests a GET /admin/nodes/{identity} request returns the expected node.
    fn test_fetch_node_ok() {
        run_test(|test_yaml_file_path| {
            write_to_file(&test_yaml_file_path, &[get_node_1(), get_node_2()]);

            let node_registry = new_yaml_node_registry(test_yaml_file_path);

            let mut app = test::init_service(
                App::new().data(node_registry.clone()).service(
                    web::resource("/admin/nodes/{identity}")
                        .route(web::get().to_async(fetch_node::<YamlNodeRegistry>)),
                ),
            );

            let req = test::TestRequest::get()
                .uri(&format!("/admin/nodes/{}", get_node_1().identity))
                .to_request();

            let resp = test::call_service(&mut app, req);

            assert_eq!(resp.status(), StatusCode::OK);
            let node: Node = serde_yaml::from_slice(&test::read_body(resp)).unwrap();
            assert_eq!(node, get_node_1())
        })
    }

    #[test]
    /// Tests a GET /admin/nodes/{identity} request returns NotFound when an invalid identity is passed
    fn test_fetch_node_not_found() {
        run_test(|test_yaml_file_path| {
            write_to_file(&test_yaml_file_path, &[get_node_1(), get_node_2()]);

            let node_registry = new_yaml_node_registry(test_yaml_file_path);

            let mut app = test::init_service(
                App::new().data(node_registry.clone()).service(
                    web::resource("/admin/nodes/{identity}")
                        .route(web::get().to_async(fetch_node::<YamlNodeRegistry>)),
                ),
            );

            let req = test::TestRequest::get()
                .uri("/admin/nodes/Node-not-valid")
                .to_request();

            let resp = test::call_service(&mut app, req);

            assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        })
    }

    #[test]
    /// Test the PUT /admin/nodes/{identity} route for adding or updating a node in the registry.
    fn test_put_node() {
        run_test(|test_yaml_file_path| {
            let mut node = get_node_1();
            write_to_file(&test_yaml_file_path, &[node.clone()]);

            let node_registry = new_yaml_node_registry(test_yaml_file_path);

            let mut app = test::init_service(
                App::new().data(node_registry.clone()).service(
                    web::resource("/admin/nodes/{identity}")
                        .route(web::patch().to_async(put_node::<YamlNodeRegistry>))
                        .route(web::get().to_async(fetch_node::<YamlNodeRegistry>)),
                ),
            );

            // Verify no body (e.g. no updated Node) gets a BAD_REQUEST response
            let req = test::TestRequest::patch()
                .uri(&format!("/admin/nodes/{}", &node.identity))
                .to_request();

            let resp = test::call_service(&mut app, req);

            assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

            // Verify that updating an existing node gets an OK response and the fetched node has
            // the updated values
            node.endpoint = "12.0.0.123:8432".to_string();
            node.metadata
                .insert("location".to_string(), "Minneapolis".to_string());

            let req = test::TestRequest::patch()
                .uri(&format!("/admin/nodes/{}", &node.identity))
                .header(header::CONTENT_TYPE, "application/json")
                .set_json(&node)
                .to_request();

            let resp = test::call_service(&mut app, req);

            assert_eq!(resp.status(), StatusCode::OK);

            let req = test::TestRequest::get()
                .uri(&format!("/admin/nodes/{}", &node.identity))
                .to_request();

            let resp = test::call_service(&mut app, req);

            assert_eq!(resp.status(), StatusCode::OK);
            let updated_node: Node = serde_yaml::from_slice(&test::read_body(resp)).unwrap();
            assert_eq!(updated_node, node);

            // Verify that attempting to change the node identity gets a FORBIDDEN response
            let old_identity = node.identity.clone();
            node.identity = "Node-789".into();

            let req = test::TestRequest::patch()
                .uri(&format!("/admin/nodes/{}", &old_identity))
                .header(header::CONTENT_TYPE, "application/json")
                .set_json(&node)
                .to_request();

            let resp = test::call_service(&mut app, req);

            assert_eq!(resp.status(), StatusCode::FORBIDDEN);
        })
    }

    #[test]
    /// Test the DELETE /admin/nodes/{identity} route for deleting a node from the registry.
    fn test_delete_node() {
        run_test(|test_yaml_file_path| {
            write_to_file(&test_yaml_file_path, &[get_node_1()]);

            let node_registry = new_yaml_node_registry(test_yaml_file_path);

            let mut app = test::init_service(
                App::new().data(node_registry.clone()).service(
                    web::resource("/admin/nodes/{identity}")
                        .route(web::delete().to_async(delete_node::<YamlNodeRegistry>)),
                ),
            );

            // Verify that an existing node gets an OK response
            let req = test::TestRequest::delete()
                .uri(&format!("/admin/nodes/{}", get_node_1().identity))
                .to_request();

            let resp = test::call_service(&mut app, req);

            assert_eq!(resp.status(), StatusCode::OK);

            // Verify that a non-existent node gets a NOT_FOUND response
            let req = test::TestRequest::delete()
                .uri(&format!("/admin/nodes/{}", get_node_1().identity))
                .to_request();

            let resp = test::call_service(&mut app, req);

            assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        })
    }

    #[test]
    /// Tests a GET /admin/nodes request with no filters returns the expected nodes.
    fn test_list_node_ok() {
        run_test(|test_yaml_file_path| {
            write_to_file(&test_yaml_file_path, &[get_node_1(), get_node_2()]);

            let node_registry = new_yaml_node_registry(test_yaml_file_path);

            let mut app = test::init_service(
                App::new().data(node_registry.clone()).service(
                    web::resource("/admin/nodes")
                        .route(web::get().to_async(list_nodes::<YamlNodeRegistry>)),
                ),
            );

            let req = test::TestRequest::get().uri("/admin/nodes").to_request();

            let resp = test::call_service(&mut app, req);

            assert_eq!(resp.status(), StatusCode::OK);
            let nodes: ListNodesResponse = serde_yaml::from_slice(&test::read_body(resp)).unwrap();
            assert_eq!(nodes.data, vec![get_node_1(), get_node_2()]);
            assert_eq!(
                nodes.paging,
                create_test_paging_response(0, 100, 0, 0, 0, 2, "/admin/nodes?")
            )
        })
    }

    #[test]
    /// Tests a GET /admin/nodes request with filters returns the expected node.
    fn test_list_node_with_filters_ok() {
        run_test(|test_yaml_file_path| {
            write_to_file(&test_yaml_file_path, &[get_node_1(), get_node_2()]);

            let node_registry = new_yaml_node_registry(test_yaml_file_path);

            let mut app = test::init_service(
                App::new().data(node_registry.clone()).service(
                    web::resource("/admin/nodes")
                        .route(web::get().to_async(list_nodes::<YamlNodeRegistry>)),
                ),
            );

            let filter = percent_encode_filter_query("{\"company\":[\"=\",\"Bitwise IO\"]}");

            let req = test::TestRequest::get()
                .uri(&format!("/admin/nodes?filter={}", filter))
                .header(header::CONTENT_TYPE, "application/json")
                .to_request();

            let resp = test::call_service(&mut app, req);

            assert_eq!(resp.status(), StatusCode::OK);
            let nodes: ListNodesResponse = serde_yaml::from_slice(&test::read_body(resp)).unwrap();
            assert_eq!(nodes.data, vec![get_node_1()]);
            let link = format!("/admin/nodes?filter={}&", filter);
            assert_eq!(
                nodes.paging,
                create_test_paging_response(0, 100, 0, 0, 0, 1, &link)
            )
        })
    }

    #[test]
    /// Tests a GET /admin/nodes request with invalid filter returns BadRequest response.
    fn test_list_node_with_filters_bad_request() {
        run_test(|test_yaml_file_path| {
            write_to_file(&test_yaml_file_path, &[get_node_1(), get_node_2()]);

            let node_registry = new_yaml_node_registry(test_yaml_file_path);

            let mut app = test::init_service(
                App::new().data(node_registry.clone()).service(
                    web::resource("/admin/nodes")
                        .route(web::get().to_async(list_nodes::<YamlNodeRegistry>)),
                ),
            );

            let filter = percent_encode_filter_query("{\"company\":[\"*\",\"Bitwise IO\"]}");

            let req = test::TestRequest::get()
                .uri(&format!("/admin/nodes?filter={}", filter))
                .header(header::CONTENT_TYPE, "application/json")
                .to_request();

            let resp = test::call_service(&mut app, req);

            assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        })
    }

    #[test]
    /// Test the POST /admin/nodes route for adding a node to the registry.
    fn test_add_node() {
        run_test(|test_yaml_file_path| {
            write_to_file(&test_yaml_file_path, &[]);

            let node_registry = new_yaml_node_registry(test_yaml_file_path);

            let mut app = test::init_service(
                App::new().data(node_registry.clone()).service(
                    web::resource("/admin/nodes")
                        .route(web::post().to_async(add_node::<YamlNodeRegistry>))
                        .route(web::get().to_async(fetch_node::<YamlNodeRegistry>)),
                ),
            );

            // Verify an invalid node gets a BAD_REQUEST response
            let req = test::TestRequest::post()
                .uri("/admin/nodes")
                .header(header::CONTENT_TYPE, "application/json")
                .to_request();

            let resp = test::call_service(&mut app, req);

            assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

            // Verify a valid node gets an OK response
            let req = test::TestRequest::post()
                .uri("/admin/nodes")
                .header(header::CONTENT_TYPE, "application/json")
                .set_json(&get_node_1())
                .to_request();

            let resp = test::call_service(&mut app, req);

            assert_eq!(resp.status(), StatusCode::OK);

            // Verify a duplicate node gets a FORBIDDEN response
            let req = test::TestRequest::post()
                .uri("/admin/nodes")
                .header(header::CONTENT_TYPE, "application/json")
                .set_json(&get_node_1())
                .to_request();

            let resp = test::call_service(&mut app, req);

            assert_eq!(resp.status(), StatusCode::FORBIDDEN);
        })
    }

    fn create_test_paging_response(
        offset: usize,
        limit: usize,
        next_offset: usize,
        previous_offset: usize,
        last_offset: usize,
        total: usize,
        link: &str,
    ) -> Paging {
        let base_link = format!("{}limit={}&", link, limit);
        let current_link = format!("{}offset={}", base_link, offset);
        let first_link = format!("{}offset=0", base_link);
        let next_link = format!("{}offset={}", base_link, next_offset);
        let previous_link = format!("{}offset={}", base_link, previous_offset);
        let last_link = format!("{}offset={}", base_link, last_offset);

        Paging {
            current: current_link,
            offset,
            limit,
            total,
            first: first_link,
            prev: previous_link,
            next: next_link,
            last: last_link,
        }
    }

    fn write_to_file(file_path: &str, nodes: &[Node]) {
        let file = File::create(file_path).expect("Error creating test nodes yaml file.");
        serde_yaml::to_writer(file, nodes).expect("Error writing nodes to file.");
    }

    fn get_node_1() -> Node {
        let mut metadata = HashMap::new();
        metadata.insert("company".to_string(), "Bitwise IO".to_string());
        Node {
            identity: "Node-123".to_string(),
            endpoint: "12.0.0.123:8431".to_string(),
            display_name: "Bitwise IO - Node 1".to_string(),
            metadata,
        }
    }

    fn get_node_2() -> Node {
        let mut metadata = HashMap::new();
        metadata.insert("company".to_string(), "Cargill".to_string());
        Node {
            identity: "Node-456".to_string(),
            endpoint: "13.0.0.123:8434".to_string(),
            display_name: "Cargill - Node 1".to_string(),
            metadata,
        }
    }

    fn run_test<T>(test: T) -> ()
    where
        T: FnOnce(&str) -> () + panic::UnwindSafe,
    {
        let test_yaml_file = temp_yaml_file_path();

        let test_path = test_yaml_file.clone();
        let result = panic::catch_unwind(move || test(&test_path));

        remove_file(test_yaml_file).unwrap();

        assert!(result.is_ok())
    }

    fn temp_yaml_file_path() -> String {
        let mut temp_dir = env::temp_dir();

        let thread_id = thread::current().id();
        temp_dir.push(format!("test_node_endpoint-{:?}.yaml", thread_id));
        temp_dir.to_str().unwrap().to_string()
    }
}

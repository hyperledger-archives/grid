// Copyright 2019 Cargill Incorporated
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

use actix_web::{error::BlockingError, web, Error, HttpResponse};
use futures::Future;
use libsplinter::node_registry::error::NodeRegistryError;
use libsplinter::node_registry::NodeRegistry;

pub fn fetch_node(
    identity: web::Path<String>,
    registry: web::Data<Box<dyn NodeRegistry>>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    web::block(move || registry.fetch_node(&identity.into_inner())).then(|res| match res {
        Ok(node) => Ok(HttpResponse::Ok().json(node)),
        Err(err) => match err {
            BlockingError::Error(err) => match err {
                NodeRegistryError::NotFoundError(err) => Ok(HttpResponse::NotFound().json(err)),
                _ => Ok(HttpResponse::InternalServerError().json(format!("{}", err))),
            },
            _ => Ok(HttpResponse::InternalServerError().json(format!("{}", err))),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node_registry::yaml::YamlNodeRegistry;
    use libsplinter::node_registry::Node;
    use std::collections::HashMap;
    use std::env;
    use std::fs::{remove_file, File};
    use std::panic;
    use std::thread;

    use actix_web::{http::StatusCode, test, web, App};

    #[test]
    /// Tests a GET /node/{identity} request returns the expected node.
    fn test_fetch_node_ok() {
        run_test(|test_yaml_file_path| {
            write_to_file(&test_yaml_file_path);

            let node_registry: Box<dyn NodeRegistry> = Box::new(
                YamlNodeRegistry::new(test_yaml_file_path)
                    .expect("Error creating YamlNodeRegistry"),
            );

            let mut app =
                test::init_service(App::new().data(node_registry.clone()).service(
                    web::resource("/node/{identity}").route(web::get().to_async(fetch_node)),
                ));

            let req = test::TestRequest::get()
                .uri(&format!("/node/{}", get_node_1().identity))
                .to_request();

            let resp = test::call_service(&mut app, req);

            assert_eq!(resp.status(), StatusCode::OK);
            let node: Node = serde_yaml::from_slice(&test::read_body(resp)).unwrap();
            assert_eq!(node, get_node_1())
        })
    }

    #[test]
    /// Tests a GET /node/{identity} request returns NotFound when an invalid identity is passed
    fn test_fetch_node_not_found() {
        run_test(|test_yaml_file_path| {
            write_to_file(&test_yaml_file_path);

            let node_registry: Box<dyn NodeRegistry> = Box::new(
                YamlNodeRegistry::new(test_yaml_file_path)
                    .expect("Error creating YamlNodeRegistry"),
            );

            let mut app =
                test::init_service(App::new().data(node_registry.clone()).service(
                    web::resource("/node/{identity}").route(web::get().to_async(fetch_node)),
                ));

            let req = test::TestRequest::get()
                .uri("/node/Node-not-valid")
                .to_request();

            let resp = test::call_service(&mut app, req);

            assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        })
    }

    fn write_to_file(file_path: &str) {
        let file = File::create(file_path).expect("Error creating test nodes yaml file.");
        serde_yaml::to_writer(file, &vec![get_node_1(), get_node_2()])
            .expect("Error writing nodes to file.");
    }

    fn get_node_1() -> Node {
        let mut metadata = HashMap::new();
        metadata.insert("url".to_string(), "12.0.0.123:8431".to_string());
        metadata.insert("company".to_string(), "Bitwise IO".to_string());
        Node {
            identity: "Node-123".to_string(),
            metadata,
        }
    }

    fn get_node_2() -> Node {
        let mut metadata = HashMap::new();
        metadata.insert("url".to_string(), "13.0.0.123:8434".to_string());
        metadata.insert("company".to_string(), "Cargill".to_string());
        Node {
            identity: "Node-456".to_string(),
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

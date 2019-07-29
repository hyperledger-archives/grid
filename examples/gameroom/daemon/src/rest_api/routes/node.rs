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
use actix_web::{client::Client, http::StatusCode, web, Error, HttpResponse};
use futures::Future;
use libsplinter::node_registry::Node;
pub fn fetch_node(
    identity: web::Path<String>,
    client: web::Data<(Client, String)>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let splinterd_url = &client.1;
    let client = &client.0;

    client
        .get(&format!("{}/nodes/{}", splinterd_url, identity))
        .send()
        .map_err(Error::from)
        .and_then(|mut resp| {
            let body = resp.body().wait()?;
            match resp.status() {
                StatusCode::OK => {
                    let node: Node = serde_json::from_slice(&body)?;
                    Ok(HttpResponse::Ok().json(node))
                }
                StatusCode::NOT_FOUND => {
                    let message: String = serde_json::from_slice(&body)?;
                    Ok(HttpResponse::NotFound().json(message))
                }
                _ => {
                    let message: String = serde_json::from_slice(&body)?;
                    Ok(HttpResponse::InternalServerError().json(message))
                }
            }
        })
}
#[cfg(all(feature = "test-node-endpoint", test))]
mod test {
    use super::*;
    use actix_web::{
        http::{header, StatusCode},
        test, web, App,
    };

    static SPLINTERD_URL: &str = "http://splinterd-node:8085";

    #[test]
    /// Tests a GET /nodes/{identity} request returns the expected node.
    fn test_fetch_node_ok() {
        let mut app = test::init_service(
            App::new()
                .data((Client::new(), SPLINTERD_URL.to_string()))
                .service(web::resource("/nodes/{identity}").route(web::get().to_async(fetch_node))),
        );

        let req = test::TestRequest::get()
            .uri(&format!("/nodes/{}", get_node_1().identity))
            .to_request();

        let resp = test::call_service(&mut app, req);

        assert_eq!(resp.status(), StatusCode::OK);
        let node: Node = serde_json::from_slice(&test::read_body(resp)).unwrap();
        assert_eq!(node, get_node_1())
    }

    #[test]
    /// Tests a GET /nodes/{identity} request returns NotFound when an invalid identity is passed
    fn test_fetch_node_not_found() {
        let mut app = test::init_service(
            App::new()
                .data((Client::new(), SPLINTERD_URL.to_string()))
                .service(web::resource("/nodes/{identity}").route(web::get().to_async(fetch_node))),
        );

        let req = test::TestRequest::get()
            .uri("/nodes/Node-not-valid")
            .to_request();

        let resp = test::call_service(&mut app, req);

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
    fn get_node_1() -> Node {
        let mut metadata = HashMap::new();
        metadata.insert("url".to_string(), "127.0.0.1:8080".to_string());
        metadata.insert("company".to_string(), "Bitwise IO".to_string());
        Node {
            identity: "Node-123".to_string(),
            metadata,
        }
    }
}

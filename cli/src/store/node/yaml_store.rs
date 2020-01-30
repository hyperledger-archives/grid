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

use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{Error as IoError, ErrorKind};
use std::path::PathBuf;

use super::{Node, NodeStore, NodeStoreError};

const DEFAULT_FILE_NAME: &str = "node_alias.yaml";
const DEFAULTS_PATH: &str = ".splinter";

#[derive(Serialize, Deserialize)]
pub struct SerdeNode {
    alias: String,
    endpoint: String,
}

impl Into<Node> for SerdeNode {
    fn into(self) -> Node {
        Node {
            alias: self.alias,
            endpoint: self.endpoint,
        }
    }
}

impl From<&Node> for SerdeNode {
    fn from(node: &Node) -> SerdeNode {
        SerdeNode {
            alias: node.alias.clone(),
            endpoint: node.endpoint.clone(),
        }
    }
}

pub struct FileBackedNodeStore {
    file_name: String,
}

impl Default for FileBackedNodeStore {
    fn default() -> Self {
        FileBackedNodeStore {
            file_name: DEFAULT_FILE_NAME.to_owned(),
        }
    }
}

impl NodeStore for FileBackedNodeStore {
    fn get_node(&self, alias: &str) -> Result<Option<Node>, NodeStoreError> {
        let nodes = self.load()?;

        let node = nodes.into_iter().find_map(|node| {
            if node.alias == alias {
                return Some(node.into());
            }
            None
        });

        Ok(node)
    }

    fn list_nodes(&self) -> Result<Vec<Node>, NodeStoreError> {
        let serde_nodes = self.load()?;
        let nodes = serde_nodes
            .into_iter()
            .map(|node| node.into())
            .collect::<Vec<Node>>();
        Ok(nodes)
    }

    fn add_node(&self, new_node: &Node) -> Result<(), NodeStoreError> {
        let mut serde_nodes = self.load()?;
        let existing_node_index = serde_nodes.iter().enumerate().find_map(|(index, node)| {
            if new_node.alias() == node.alias {
                Some(index)
            } else {
                None
            }
        });
        if let Some(index) = existing_node_index {
            serde_nodes.remove(index);
        }
        serde_nodes.push(SerdeNode::from(new_node));

        self.save(&serde_nodes)?;

        Ok(())
    }

    fn delete_node(&self, alias: &str) -> Result<(), NodeStoreError> {
        let mut serde_nodes = self.load()?;
        let existing_node_index = serde_nodes.iter().enumerate().find_map(|(index, node)| {
            if node.alias == alias {
                Some(index)
            } else {
                None
            }
        });

        match existing_node_index {
            Some(index) => {
                serde_nodes.remove(index);
                self.save(&serde_nodes)?;
            }
            None => {
                return Err(NodeStoreError::NotFound(format!(
                    "Node with alias {} was not found",
                    alias
                )))
            }
        }

        Ok(())
    }
}

impl FileBackedNodeStore {
    fn load(&self) -> Result<Vec<SerdeNode>, NodeStoreError> {
        let file = open_file(false, &self.file_name)?;

        if file.metadata()?.len() == 0 {
            return Ok(vec![]);
        }

        let nodes = serde_yaml::from_reader(file)?;
        Ok(nodes)
    }

    fn save(&self, nodes: &[SerdeNode]) -> Result<(), NodeStoreError> {
        let temp_file_name = format!("{}.tmp", self.file_name);
        let file = open_file(true, &temp_file_name)?;

        serde_yaml::to_writer(file, &nodes)?;

        let temp_file_path = build_file_path(&temp_file_name)?;
        let perm_file_path = build_file_path(&self.file_name)?;
        fs::rename(temp_file_path, perm_file_path)?;
        Ok(())
    }
}

fn build_file_path(file_name: &str) -> Result<PathBuf, NodeStoreError> {
    let mut path = get_file_path()?;
    path.push(file_name);
    Ok(path)
}

fn open_file(truncate: bool, file_name: &str) -> Result<File, NodeStoreError> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(truncate)
        .open(&build_file_path(file_name)?)?;
    Ok(file)
}

fn get_file_path() -> Result<PathBuf, NodeStoreError> {
    let mut path = dirs::home_dir().ok_or_else(|| {
        let err = IoError::new(ErrorKind::NotFound, "Home directory not found");
        NodeStoreError::IoError(err)
    })?;
    path.push(DEFAULTS_PATH);
    fs::create_dir_all(&path)?;
    Ok(path)
}

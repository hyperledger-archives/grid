// Copyright 2021 Cargill Incorporated
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

//! Provides support for using GDSN data to create Grid Product transactions.
//!
//! This module exports a `get_trade_items_from_xml()` function, which can parse
//! and validate GDSN 3.1 XML data. After being processed by this function, the
//! resulting `TradeItem` structs can be converted into Grid Product transaction
//! payloads.
mod error;

use std::io::{Cursor, Read};

use quick_xml::{
    de::from_str,
    events::{BytesEnd, BytesStart, BytesText, Event},
    Reader, Writer,
};
use serde::Deserialize;

use crate::error::InvalidArgumentError;
use crate::protocol::{
    product::{
        payload::{
            ProductCreateAction, ProductCreateActionBuilder, ProductUpdateAction,
            ProductUpdateActionBuilder,
        },
        state::ProductNamespace,
    },
    schema::state::{DataType, PropertyValueBuilder},
};
pub use error::ProductGdsnError;

/// Name of the property where GDSN 3.1 XML data will be stored
pub const GDSN_3_1_PROPERTY_NAME: &str = "GDSN_3_1";

#[derive(Debug, Deserialize, PartialEq)]
/// Struct used for serde deserialization of the GDSN XML elements
pub struct GridTradeItems {
    #[serde(rename = "tradeItem")]
    pub trade_items: Vec<TradeItem>,
}

/// Representation of a GDSN trade item, including the GTIN and GDSN XML element
#[derive(Debug, Deserialize, PartialEq)]
pub struct TradeItem {
    pub gtin: String,
    #[serde(default)]
    pub payload: String,
}

impl TradeItem {
    /// Returns a ProductCreateAction transaction payload
    ///
    /// # Arguments
    ///
    /// * `owner` - The Pike organization ID of the owner of this record in Grid
    pub fn into_create_payload(self, owner: &str) -> Result<ProductCreateAction, ProductGdsnError> {
        let xml_property = PropertyValueBuilder::new()
            .with_name(GDSN_3_1_PROPERTY_NAME.to_string())
            .with_data_type(DataType::String)
            .with_string_value(self.payload)
            .build()?;
        Ok(ProductCreateActionBuilder::new()
            .with_product_id(self.gtin)
            .with_product_namespace(ProductNamespace::Gs1)
            .with_owner(owner.to_string())
            .with_properties(vec![xml_property])
            .build()?)
    }

    /// Returns a ProductUpdateAction transaction payload
    pub fn into_update_payload(self) -> Result<ProductUpdateAction, ProductGdsnError> {
        let gdsn_property = PropertyValueBuilder::new()
            .with_name(GDSN_3_1_PROPERTY_NAME.to_string())
            .with_data_type(DataType::String)
            .with_string_value(self.payload)
            .build()?;
        Ok(ProductUpdateActionBuilder::new()
            .with_product_id(self.gtin)
            .with_product_namespace(ProductNamespace::Gs1)
            .with_properties(vec![gdsn_property])
            .build()?)
    }
}

/// Returns a vector of TradeItem objects from a path to an XML file containing
/// a gridTradeItems XML element. An error will be returned if the XML file
/// fails to validate against the GridTradeItems.xsd schema, or if the file
/// does not contain well-formed XML.
///
/// View the GridTradeItems XML schema definition here:
///     https://github.com/hyperledger/grid/blob/main/sdk/src/product/gdsn/GridTradeItems.xsd
///
/// # Arguments
///
/// * `path` - The path to an XML file containing GDSN trade item definitions
///
pub fn get_trade_items_from_xml(path: &str) -> Result<Vec<TradeItem>, ProductGdsnError> {
    let mut xml_file = std::fs::File::open(path).map_err(|error| {
        ProductGdsnError::InvalidArgument(InvalidArgumentError::new(
            path.to_string(),
            error.to_string(),
        ))
    })?;
    let mut xml_str = String::new();

    xml_file.read_to_string(&mut xml_str).map_err(|error| {
        ProductGdsnError::InvalidArgument(InvalidArgumentError::new(
            path.to_string(),
            error.to_string(),
        ))
    })?;

    let mut grid_trade_items: GridTradeItems = from_str(&xml_str).map_err(|error| {
        ProductGdsnError::InvalidArgument(InvalidArgumentError::new(
            path.to_string(),
            error.to_string(),
        ))
    })?;

    let mut reader = Reader::from_str(&xml_str);
    reader.trim_text(true);
    let mut buf = Vec::new();

    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) if e.name() == b"tradeItem" => {
                let mut writer = Writer::new(Cursor::new(Vec::new()));
                let mut elem = BytesStart::owned(b"tradeItem".to_vec(), "tradeItem".len());
                elem.extend_attributes(e.attributes().map(|attr| attr.unwrap()));
                writer.write_event(Event::Start(elem))?;

                let mut gtin_buf = Vec::new();
                let mut current_gtin = String::new();

                loop {
                    match reader.read_event(&mut buf) {
                        Ok(Event::End(ref e)) if e.name() == b"tradeItem" => {
                            writer.write_event(Event::End(BytesEnd::borrowed(b"tradeItem")))?;
                            let result_bytes = writer.into_inner().into_inner();
                            let result = std::str::from_utf8(&result_bytes)?;

                            for trade_item in grid_trade_items.trade_items.iter_mut() {
                                if trade_item.gtin == current_gtin {
                                    trade_item.payload = result.to_string();
                                }
                            }
                            break;
                        }
                        Ok(Event::Start(ref e)) if e.name() == b"gtin" => {
                            reader.read_text(&e.name(), &mut gtin_buf)?;
                            current_gtin = std::str::from_utf8(&gtin_buf)?
                                .split('/')
                                .next()
                                .ok_or_else(|| {
                                    ProductGdsnError::InvalidArgument(InvalidArgumentError::new(
                                        path.to_string(),
                                        "Unable to parse GTIN from XML".to_string(),
                                    ))
                                })?
                                .to_string();
                            writer.write_event(Event::Start(BytesStart::owned(
                                b"gtin".to_vec(),
                                "gtin".len(),
                            )))?;
                            writer.write_event(Event::Text(BytesText::from_plain(
                                current_gtin.as_bytes(),
                            )))?;
                            writer.write_event(Event::End(BytesEnd::owned(b"gtin".to_vec())))?;
                        }
                        Ok(Event::Eof) => break,
                        Ok(e) => writer.write_event(&e)?,
                        Err(e) => {
                            return Err(ProductGdsnError::InvalidArgument(
                                InvalidArgumentError::new(
                                    path.to_string(),
                                    format!(
                                        "Error at position {}: {:?}",
                                        reader.buffer_position(),
                                        e
                                    ),
                                ),
                            ));
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(ProductGdsnError::InvalidArgument(
                    InvalidArgumentError::new(
                        path.to_string(),
                        format!("Error at position {}: {:?}", reader.buffer_position(), e),
                    ),
                ));
            }
            _ => (),
        }
        buf.clear();
    }
    Ok(grid_trade_items.trade_items)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::{fs::read_to_string, path::PathBuf};

    use crate::error::InvalidArgumentError;

    const TEST_GTIN_1: &str = "00734730437958";
    const TEST_GTIN_2: &str = "10036016500279";

    fn get_expected_trade_item_payload_from_file(path: &str) -> String {
        let mut payload_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        payload_path.push(path);

        let payload = read_to_string(payload_path)
            .unwrap()
            .replace("    ", "")
            .replace("\n", "");

        payload
    }

    /// Test that a well-formed and valid XML file with a gridTradeItems element
    /// containing one product definition element will return a vector with one
    /// TradeItem
    #[test]
    fn test_get_trade_item_from_xml_success() {
        let mut test_gdsn_product_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_gdsn_product_path.push("src/product/gdsn/test_files/gdsn_product.xml");

        let path_str = test_gdsn_product_path.to_str().unwrap();
        let result = get_trade_items_from_xml(path_str);

        assert!(result.is_ok());

        let expected = TradeItem {
            gtin: TEST_GTIN_1.to_string(),
            payload: get_expected_trade_item_payload_from_file(
                "src/product/gdsn/test_files/test_trade_item_payload_1.xml",
            ),
        };

        assert_eq!(result.unwrap(), vec![expected]);
    }

    /// Test that a well-formed and valid XML file with a gridTradeItems element
    /// containing multiple product definitions will return a vector with all of
    /// the TradeItems.
    #[test]
    fn test_get_multiple_trade_items_from_xml_success() {
        let mut test_gdsn_product_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_gdsn_product_path.push("src/product/gdsn/test_files/gdsn_product_multiple.xml");

        let path_str = test_gdsn_product_path.to_str().unwrap();
        let result = get_trade_items_from_xml(path_str);

        assert!(result.is_ok());

        let expected_1 = TradeItem {
            gtin: TEST_GTIN_1.to_string(),
            payload: get_expected_trade_item_payload_from_file(
                "src/product/gdsn/test_files/test_trade_item_payload_1.xml",
            ),
        };

        let expected_2 = TradeItem {
            gtin: TEST_GTIN_2.to_string(),
            payload: get_expected_trade_item_payload_from_file(
                "src/product/gdsn/test_files/test_trade_item_payload_2.xml",
            ),
        };

        assert_eq!(result.unwrap(), vec![expected_1, expected_2]);
    }

    /// Test that a poorly formed product definition will result in an error
    #[test]
    fn test_get_trade_items_from_xml_poorly_formed() {
        let mut test_gdsn_product_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_gdsn_product_path.push("src/product/gdsn/test_files/gdsn_product_poorly_formed.xml");

        let path_str = test_gdsn_product_path.to_str().unwrap();
        let result = get_trade_items_from_xml(path_str);

        assert!(result.is_err());

        let expected_error = InvalidArgumentError::new(
            path_str.to_string(),
            "Expecting </tradeItem> found </gridTradeItems>".to_string(),
        );
        assert_eq!(result.unwrap_err().to_string(), expected_error.to_string());
    }
}

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

pub mod error;

use quick_xml::{
    de::from_str,
    events::{BytesEnd, BytesStart, BytesText, Event},
    Reader, Writer,
};
use serde::Deserialize;
use std::io::Cursor;
use std::io::Read;

use crate::error::InvalidArgumentError;
use error::ProductGdsnError;

use crate::protocol::{
    product::{
        payload::{ProductCreateAction, ProductCreateActionBuilder},
        state::ProductNamespace,
    },
    schema::state::{DataType, PropertyValueBuilder},
};

#[derive(Debug, Deserialize, PartialEq)]
pub struct GridTradeItems {
    #[serde(rename = "tradeItem")]
    pub trade_items: Vec<TradeItem>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct TradeItem {
    pub gtin: String,
    #[serde(default)]
    pub payload: String,
}

impl TradeItem {
    pub fn into_create_payload(self, owner: &str) -> Result<ProductCreateAction, ProductGdsnError> {
        let xml_property = PropertyValueBuilder::new()
            .with_name("xml_data".to_string())
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
}

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
                                &current_gtin.as_bytes(),
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

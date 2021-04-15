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

use libxml::{
    parser::Parser,
    schemas::{SchemaParserContext, SchemaValidationContext},
};

use crate::error::{InternalError, InvalidArgumentError};
use crate::products::gdsn::error::ProductGdsnError;

pub fn validate_product_definitons(xml_path: &str) -> Result<(), ProductGdsnError> {
    static GRID_TRADE_ITEMS_SCHEMA: &str = include_str!("GridTradeItems.xsd");

    let xml = Parser::default().parse_file(&xml_path).map_err(|err| {
        ProductGdsnError::InvalidArgument(InvalidArgumentError::new(
            xml_path.to_string(),
            err.to_string(),
        ))
    })?;

    let mut xsd_parser = SchemaParserContext::from_buffer(GRID_TRADE_ITEMS_SCHEMA);
    let xsd = SchemaValidationContext::from_parser(&mut xsd_parser);

    if let Err(errors) = xsd {
        let mut xsd_errors: Vec<String> = vec![];

        for err in &errors {
            xsd_errors.push(err.message().to_string())
        }

        return Err(ProductGdsnError::Internal(InternalError::with_message(
            format!("Error creating validation context: {:#?}", xsd_errors),
        )));
    }
    let mut xsd = xsd.unwrap();

    if let Err(errors) = xsd.validate_document(&xml) {
        let mut validation_errors: Vec<String> = vec![];

        for err in &errors {
            validation_errors.push(err.message().to_string())
        }

        return Err(ProductGdsnError::InvalidArgument(
            InvalidArgumentError::new(
                xml_path.to_string(),
                format!("Invalid product definition: {:#?}", validation_errors),
            ),
        ));
    }

    Ok(())
}

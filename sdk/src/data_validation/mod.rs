/*
 * Copyright 2021 Cargill Incorporated
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 * -----------------------------------------------------------------------------
*/

mod error;
#[cfg(feature = "purchase-order")]
pub mod purchase_order;
mod xml;

pub use error::DataValidationError;
use xml::{validate_xml, Schema};

/// Checks whether an XML file is valid against the GS1 BMS Order XML 3.4 schema.
/// An error will be returned if the file fails to validate for any reason.
///
/// For more information about this specification, see the documentation here:
///     https://www.gs1.org/standards/edi-xml/xml-order/3-4-1
///
/// This implementation uses the libxml2 C library. For more information about
/// libxml2, see the documentation here:
///     http://www.xmlsoft.org/
///
/// # Arguments
///
/// * `data` - A path to an XML file or an XML string to be validated.
/// * `is_path` - Whether the data provided is a path or a string.
/// * `schema_dir` - References a path to the directory containing schema files
///
pub fn validate_order_xml_3_4(
    data: &str,
    is_path: bool,
    schema_dir: &str,
) -> Result<(), DataValidationError> {
    validate_xml(data, is_path, Schema::OrderXmlV3_4, schema_dir)?;
    Ok(())
}

/// Checks whether an XML file at the provided path validates against the
/// GridTradeItems.xsd XML schema definition. The GridTradeItems schema is a
/// wrapper around the GDSN XML 3.1 TradeItem schema. An error will be returned
/// if the file fails to validate for any reason.
///
/// View the GridTradeItems XML schema definition here:
///     https://github.com/hyperledger/grid/blob/main/sdk/src/products/gdsn/GridTradeItems.xsd
///
/// For more information about the GDSN XML 3.1 schema, view the documentation
/// here:
///     https://www.gs1.org/docs/gdsn/3.1/gdsn_3_1_operations_manual_i2.pdf#page=19
///
/// This implementation uses the libxml2 C library. For more information about
/// libxml2, see the documentation here:
///     http://www.xmlsoft.org/
///
/// # Arguments
///
/// * `xml_path` - The path to an XML file containing GDSN trade item definitions
/// * `schema_dir` - References a path to the directory containing schema files
///
pub fn validate_gdsn_3_1(
    data: &str,
    is_path: bool,
    schema_dir: &str,
) -> Result<(), DataValidationError> {
    validate_xml(data, is_path, Schema::GdsnXmlV3_1, schema_dir)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::Read;
    use std::path::PathBuf;

    use crate::error::InvalidArgumentError;

    // GDSN Product
    /// Test a valid GDSN 3.1 xml string validates successfully
    #[test]
    fn test_validate_gdsn_3_1() {
        let mut test_gdsn_xml_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_gdsn_xml_path.push("src/data_validation/test_files/gdsn_product.xml");

        let path_str = test_gdsn_xml_path.to_str().unwrap();
        let mut data = String::new();
        std::fs::File::open(path_str)
            .unwrap()
            .read_to_string(&mut data)
            .unwrap();
        let mut schema_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        schema_dir.push("src/data_validation/xml/xsd/product");
        let schema_dir = schema_dir
            .into_os_string()
            .into_string()
            .expect("Unable to convert product schema dir to string");

        let result = validate_gdsn_3_1(&data, false, &schema_dir);

        assert!(result.is_ok());
    }

    /// Test a path to a valid GDSN 3.1 xml file validates successfully
    #[test]
    fn test_validate_gdsn_3_1_path() {
        let mut test_gdsn_xml_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_gdsn_xml_path.push("src/data_validation/test_files/gdsn_product.xml");

        let path_str = test_gdsn_xml_path.to_str().unwrap();
        let mut schema_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        schema_dir.push("src/data_validation/xml/xsd/product");
        let schema_dir = schema_dir
            .into_os_string()
            .into_string()
            .expect("Unable to convert product schema dir to string");

        let result = validate_gdsn_3_1(path_str, true, &schema_dir);

        assert!(result.is_ok());
    }

    /// Test an invalid GDSN 3.1 xml string doesn't validate
    #[test]
    fn test_validate_gdsn_3_1_invalid() {
        let mut test_gdsn_xml_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_gdsn_xml_path.push("src/data_validation/test_files/gdsn_product_invalid.xml");

        let path_str = test_gdsn_xml_path.to_str().unwrap();
        let mut data = String::new();
        std::fs::File::open(path_str)
            .unwrap()
            .read_to_string(&mut data)
            .unwrap();
        let mut schema_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        schema_dir.push("src/data_validation/xml/xsd/product");
        let schema_dir = schema_dir
            .into_os_string()
            .into_string()
            .expect("Unable to convert product schema dir to string");

        let result = validate_gdsn_3_1(&data, false, &schema_dir);

        assert!(result.is_err());

        let expected_error = InvalidArgumentError::new(data, "file fails to validate".to_string());

        assert_eq!(result.unwrap_err().to_string(), expected_error.to_string());
    }

    /// Test a path to an invalid GDSN 3.1 xml file doesn't validate
    #[test]
    fn test_validate_gdsn_3_1_path_invalid() {
        let mut test_gdsn_xml_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_gdsn_xml_path.push("src/data_validation/test_files/gdsn_product_invalid.xml");

        let path_str = test_gdsn_xml_path.to_str().unwrap();

        let mut schema_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        schema_dir.push("src/data_validation/xml/xsd/product");
        let schema_dir = schema_dir
            .into_os_string()
            .into_string()
            .expect("Unable to convert product schema dir to string");

        let result = validate_gdsn_3_1(path_str, true, &schema_dir);

        assert!(result.is_err());

        let expected_error =
            InvalidArgumentError::new(path_str.to_string(), "file fails to validate".to_string());

        assert_eq!(result.unwrap_err().to_string(), expected_error.to_string());
    }
}

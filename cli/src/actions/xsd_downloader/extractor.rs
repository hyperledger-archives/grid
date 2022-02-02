/*
 * Copyright 2022 Cargill Incorporated
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

//! Extraction logic for the XSD downloader
//!
//! These functions are specific to the file format provided by GS1.

use std::fs::{self, File};
use std::io::{self, Cursor, Read, Seek};
use std::path::Path;
use zip::{self, ZipArchive};

use crate::error::CliError;

/// Get a file from an archive
///
/// * `file` - Zip archive file
/// * `prefix` - Prefix of the sought file
fn get_file_from_archive(file: impl Read + Seek, prefix: &str) -> Result<Vec<u8>, CliError> {
    let mut archive =
        ZipArchive::new(file).map_err(|err| CliError::InternalError(err.to_string()))?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|err| CliError::InternalError(err.to_string()))?;
        let outpath = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };

        debug!("found path: {:?}", outpath);
        if outpath.to_string_lossy().starts_with(prefix) {
            let mut zip = Vec::new();
            io::copy(&mut file, &mut zip)
                .map_err(|err| CliError::InternalError(err.to_string()))?;
            return Ok(zip);
        }
    }

    Err(CliError::ActionError(
        "zip does not contain necessary files".to_string(),
    ))
}

/// Copy schemas
///
/// * `file` - Zip archive file with schemas
/// * `dest_path` - Path to copy the schemas to
fn copy_schemas(file: impl Read + Seek, dest_path: &Path) -> Result<(), CliError> {
    let mut archive =
        ZipArchive::new(file).map_err(|err| CliError::InternalError(err.to_string()))?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|err| CliError::InternalError(err.to_string()))?;
        debug!("file {}", file.name());

        // Ex: BMS_Package_Order_r3p4p1_d1_7Nov_2019/Schemas/gs1/ecom/eComCommon.xsd
        let full_path = match file.enclosed_name() {
            Some(path) => path,
            None => continue,
        };

        // Ex: BMS_Package_Order_r3p4p1_d1_7Nov_2019/Schemas
        let prefix = Path::new(
            full_path
                .iter()
                .next()
                .ok_or_else(|| CliError::ActionError("error parsing zip".to_string()))?,
        )
        .join("Schemas");

        if full_path.starts_with(prefix.clone()) {
            // Ex: gs1/ecom/eComCommon.xsd
            let outpath = dest_path.join(
                full_path
                    .strip_prefix(prefix)
                    .map_err(|err| CliError::InternalError(err.to_string()))?,
            );

            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(&p)
                        .map_err(|err| CliError::InternalError(err.to_string()))?;
                }
            }

            debug!("extracting {outpath}", outpath = outpath.to_string_lossy());

            if (&*file.name()).ends_with('/') {
                debug!("File {} extracted to \"{}\"", i, outpath.display());
                fs::create_dir_all(&outpath)
                    .map_err(|err| CliError::InternalError(err.to_string()))?;
            } else {
                debug!(
                    "File {} extracted to \"{}\" ({} bytes)",
                    i,
                    outpath.display(),
                    file.size()
                );
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(&p)
                            .map_err(|err| CliError::InternalError(err.to_string()))?;
                    }
                }
                let mut outfile = fs::File::create(&outpath)
                    .map_err(|err| CliError::InternalError(err.to_string()))?;
                io::copy(&mut file, &mut outfile)
                    .map_err(|err| CliError::InternalError(err.to_string()))?;
            }
        }
    }

    Ok(())
}

/// Extract xsd files
///
/// * `zip_path` - Zip archive file path
/// * `dest_path` - Path to copy the schemas to
pub fn extract(zip_path: &Path, dest_path: &Path) -> Result<(), CliError> {
    debug!("parsing root archive {}", zip_path.to_string_lossy());
    let root_file = File::open(&zip_path).map_err(CliError::IoError)?;
    let xml_zip = get_file_from_archive(root_file, "BMS Packages EDI XML")?;

    debug!("parsing order archive");
    let xml_cursor = Cursor::new(xml_zip);
    let order_zip = get_file_from_archive(xml_cursor, "BMS_Package_Order_")?;

    debug!("extracting order schemas");
    let order_cursor = Cursor::new(order_zip);
    copy_schemas(order_cursor, dest_path)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::Write;
    use tempdir::TempDir;
    use zip::{result::ZipResult, ZipWriter};

    fn dummy(zip: &mut ZipWriter<impl Write + Seek>, filename: &str) -> ZipResult<()> {
        zip.start_file(filename, Default::default())?;
        Ok(())
    }

    // Create an example zip that roughly mirrors the expected GS1 format
    fn create_example_zip(path: &Path) -> ZipResult<()> {
        let mut order_file: Vec<u8> = vec![];
        let order_cursor = Cursor::new(&mut order_file);
        let mut order = zip::ZipWriter::new(order_cursor);

        order.add_directory("BMS_Package_Order_r0p0p0_d0_1Dec_2000/", Default::default())?;
        order.add_directory(
            "BMS_Package_Order_r0p0p0_d0_1Dec_2000/Schemas",
            Default::default(),
        )?;
        order.add_directory(
            "BMS_Package_Order_r0p0p0_d0_1Dec_2000/Schemas/gs1",
            Default::default(),
        )?;
        order.add_directory(
            "BMS_Package_Order_r0p0p0_d0_1Dec_2000/Schemas/gs1/ecom",
            Default::default(),
        )?;
        order.add_directory(
            "BMS_Package_Order_r0p0p0_d0_1Dec_2000/Schemas/gs1/shared",
            Default::default(),
        )?;
        order.add_directory(
            "BMS_Package_Order_r0p0p0_d0_1Dec_2000/Schemas/sbdh",
            Default::default(),
        )?;

        dummy(
            &mut order,
            "BMS_Package_Order_r0p0p0_d0_1Dec_2000/Schemas/gs1/ecom/eComCommon.xsd",
        )?;
        dummy(
            &mut order,
            "BMS_Package_Order_r0p0p0_d0_1Dec_2000/Schemas/gs1/ecom/Order.xsd",
        )?;
        dummy(
            &mut order,
            "BMS_Package_Order_r0p0p0_d0_1Dec_2000/Schemas/gs1/shared/SharedCommon.xsd",
        )?;
        dummy(
            &mut order,
            "BMS_Package_Order_r0p0p0_d0_1Dec_2000/Schemas/sbdh/BasicTypes.xsd",
        )?;
        dummy(
            &mut order,
            "BMS_Package_Order_r0p0p0_d0_1Dec_2000/Schemas/sbdh/BusinessScope.xsd",
        )?;
        dummy(
            &mut order,
            "BMS_Package_Order_r0p0p0_d0_1Dec_2000/Schemas/sbdh/DocumentIdentification.xsd",
        )?;
        dummy(
            &mut order,
            "BMS_Package_Order_r0p0p0_d0_1Dec_2000/Schemas/sbdh/Manifest.xsd",
        )?;
        dummy(
            &mut order,
            "BMS_Package_Order_r0p0p0_d0_1Dec_2000/Schemas/sbdh/Partner.xsd",
        )?;
        dummy(
            &mut order,
            "BMS_Package_Order_r0p0p0_d0_1Dec_2000/Schemas/sbdh/StandardBusinessDocumentHeader.xsd",
        )?;
        order.finish()?;
        drop(order);

        let mut xml_file: Vec<u8> = vec![];
        let xml_cursor = Cursor::new(&mut xml_file);
        let mut xml = zip::ZipWriter::new(xml_cursor);
        dummy(&mut xml, "BMS_Package_ATest_r0p0p0_d0_1Dec_2000.zip")?;
        dummy(&mut xml, "BMS_Package_Order_r0p0p0_d0_1Dec_2000.zip")?;
        xml.write_all(&order_file)?;
        dummy(
            &mut xml,
            "BMS_Package_Order_Response_r0p0p0_d0_1Dec_2000.zip",
        )?;
        dummy(&mut xml, "BMS_Package_ZTest_r0p0p0_d0_1Dec_2000.zip")?;
        xml.finish()?;
        drop(xml);

        let file = std::fs::File::create(&path).unwrap();
        let mut root = zip::ZipWriter::new(file);
        dummy(&mut root, "BMS_eCom_Common_Library_r0p0p0_d0_1Jan_2000.pdf")?;
        dummy(&mut root, "BMS EDI XML 0.0.0 PDF.zip")?;
        dummy(&mut root, "BMS Packages EDI XML 0.0.0.zip")?;
        root.write_all(&xml_file)?;
        dummy(
            &mut root,
            "BMS_Shared_Common_Library_r0p0p0_d0_1Jan_2000.pdf",
        )?;

        root.finish()?;

        Ok(())
    }

    #[test]
    // Validate that extract() will extract a zip file in the expected zipception format
    // without failure, and that the resulting files are in the expected locations.
    fn extract_works() {
        let source_temp = TempDir::new("zipstuff").expect("could not create tempdir");
        let source_path = source_temp.into_path().join("test.zip");

        let dest_temp = TempDir::new("dest").expect("could not create tempdir");
        let dest_path = dest_temp.into_path();

        create_example_zip(&source_path).unwrap();
        extract(&source_path, &dest_path).unwrap();
        assert!(dest_path.join("gs1/ecom/eComCommon.xsd").exists());
        assert!(dest_path.join("gs1/ecom/Order.xsd").exists());
        assert!(dest_path.join("gs1/shared/SharedCommon.xsd").exists());
        assert!(dest_path.join("sbdh/BasicTypes.xsd").exists());
        assert!(dest_path.join("sbdh/BusinessScope.xsd").exists());
        assert!(dest_path.join("sbdh/DocumentIdentification.xsd").exists());
        assert!(dest_path.join("sbdh/Manifest.xsd").exists());
        assert!(dest_path.join("sbdh/Partner.xsd").exists());
        assert!(dest_path
            .join("sbdh/StandardBusinessDocumentHeader.xsd")
            .exists());
    }
}

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
use std::fs::{self, File};
use std::io::{self, Cursor};
use std::path::PathBuf;

use reqwest::{blocking::Client, Url};

use crate::error::CliError;

/// Uses the Reqwest library to download a file
pub fn download(url: &Url, file_name: &str) -> Result<(), CliError> {
    let client = Client::new();
    let res = client
        .get(url.clone())
        .send()
        .map_err(|err| CliError::InternalError(err.to_string()))?;
    if res.status().is_server_error() {
        return Err(CliError::ActionError(format!(
            "received server error {:?}",
            res.status()
        )));
    }
    let body = res
        .bytes()
        .map_err(|err| CliError::InternalError(err.to_string()))?;
    let mut file =
        File::create(file_name).map_err(|err| CliError::InternalError(err.to_string()))?;
    let mut cursor = Cursor::new(body);
    io::copy(&mut cursor, &mut file).map_err(|err| CliError::InternalError(err.to_string()))?;

    Ok(())
}

/// Configuration for the caching downloader
#[derive(Debug, PartialEq)]
pub struct CachingDownloadConfig {
    pub url: Url,
    pub file_path: PathBuf,
    pub temp_file_path: PathBuf,
    #[cfg(feature = "xsd-downloader-force-download")]
    pub force_download: bool,
    pub hash: &'static str,
}

/// Cache a file
///
/// * `config` - The configuration for caching and downloading the file
/// * `download` - The function to use to download the file
/// * `validate_hash` - The function to use to validate the hash
pub fn caching_download<T>(
    config: CachingDownloadConfig,
    download: T,
    validate_hash: impl FnOnce(&PathBuf, &str) -> Result<(), CliError>,
) -> Result<(), CliError>
where
    T: FnOnce(&Url, &str) -> Result<(), CliError>,
{
    let cached = config.file_path.exists();

    if cached {
        debug!(
            "file {file_path} is cached",
            file_path = &fs::canonicalize(config.file_path.clone())
                .unwrap_or_else(|_| config.file_path.clone())
                .as_os_str()
                .to_string_lossy()
        );
    }

    #[cfg(not(feature = "xsd-downloader-force-download"))]
    let download_file = !cached;

    #[cfg(feature = "xsd-downloader-force-download")]
    let download_file = !cached || config.force_download;

    if download_file {
        if cached {
            debug!("downloading anyway due to force download option",);
        }

        let temp_path_name: &str = &config.temp_file_path.as_os_str().to_string_lossy();

        if config.temp_file_path.exists() {
            return Err(CliError::ActionError(format!(
                "cannot proceed, as temp file {temp_file_path} already exists. is a \
                download already in progress? if not, please delete the temp file",
                temp_file_path = &fs::canonicalize(config.temp_file_path.clone())
                    .unwrap_or(config.temp_file_path)
                    .as_os_str()
                    .to_string_lossy()
            )));
        }

        info!("downloading file from {url_name}", url_name = config.url);

        let url = &config.url;
        download(url, temp_path_name)?;

        info!("download finished");

        if let Err(result) = (validate_hash)(&config.temp_file_path, config.hash) {
            fs::remove_file(config.temp_file_path)
                .map_err(|err| CliError::InternalError(err.to_string()))?;

            return Err(result);
        }

        debug!("hash valid, moving to cache");

        fs::rename(config.temp_file_path, config.file_path)
            .map_err(|err| CliError::InternalError(err.to_string()))?;
    } else {
        debug!("using cache");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::tests::{MockValidator, MockValidatorCall};
    use super::*;

    use mockito::mock;
    use pretty_assertions::assert_eq;
    use std::fs::File;
    use std::io::Write;
    use tempdir::TempDir;

    const TEST_DATA: &str = "lagomorpha";
    const TEST_HASH: &str = "9b44a9cb40096bf6767dec8e97bdc5a36ead7bc6200025cac801bf445307aba0";

    #[derive(Debug, PartialEq)]
    struct MockDownloaderCall {
        pub url: Url,
        pub file_name: String,
    }

    struct MockDownloader {
        responses: Vec<Result<(), CliError>>,
        calls: Vec<MockDownloaderCall>,
    }

    impl MockDownloader {
        pub fn new(responses: Vec<Result<(), CliError>>) -> MockDownloader {
            MockDownloader {
                responses,
                calls: vec![],
            }
        }

        pub fn download(&mut self, url: &Url, file_name: &str) -> Result<(), CliError> {
            self.calls.push(MockDownloaderCall {
                url: url.clone(),
                file_name: file_name.to_string(),
            });

            let mut output = File::create(file_name).expect("could not create file");
            write!(output, "{}", TEST_DATA).expect("could not write file");

            self.responses.pop().unwrap()
        }

        pub fn get_calls(&self) -> &[MockDownloaderCall] {
            &self.calls
        }
    }

    #[test]
    // Test that ReqwestDownloader downloads from the given url to the given file
    fn reqwest_downloader_downloads() {
        let url = Url::parse(&mockito::server_url()).expect("could not parse url");
        let url = url.join("some_file.zip").expect("could not create url");

        let mock_endpoint = mock("GET", "/some_file.zip")
            .with_status(201)
            .with_header("content-type", "text/plain")
            .with_body("world")
            .create();

        let tmp_dir = TempDir::new("example").expect("could not create tempdir");
        let file_path = tmp_dir.path().join("some_file_location.txt");

        assert!(!file_path.exists());

        download(&url, &file_path.as_os_str().to_string_lossy()).expect("download failed");

        assert!(file_path.exists());
        mock_endpoint.assert();

        let file = fs::read_to_string(file_path).expect("could not read file");
        assert_eq!(file, "world");
    }

    #[test]
    // Test that ReqwestDownloader downloads from the given url to the given file
    fn reqwest_downloader_does_not_leave_file_on_failure() {
        let url = Url::parse(&mockito::server_url()).expect("could not parse url");
        let url = url.join("bad_file.zip").expect("could not create url");

        let mock_endpoint = mock("GET", "/bad_file.zip").with_status(503).create();

        let tmp_dir = TempDir::new("example").expect("could not create tempdir");
        let file_path = tmp_dir.path().join("some_file_location.txt");

        assert!(!file_path.exists());

        let result = download(&url, &file_path.as_os_str().to_string_lossy());

        assert_eq!(
            format!("{:?}", result),
            "Err(ActionError(\"received server error 503\"))"
        );
        assert!(!file_path.exists());
        mock_endpoint.assert();
    }

    #[test]
    // Test that CachingDownloader does not download when file is cached
    fn caching_download_doesnot_download_cached_file() {
        let temp_dir = TempDir::new("example").expect("could not create tempdir");
        let dest_dir = temp_dir.path();

        let file_name = "gs1.zip";
        let temp_file_name = "gs1-temp.zip";

        let file_path = dest_dir.join(file_name);
        let temp_file_path = dest_dir.join(temp_file_name);

        // Create the temporary file
        File::create(file_path.clone()).expect("could not create file");
        assert!(file_path.exists());

        let mut downloader = MockDownloader::new(vec![]);
        let mut validator = MockValidator::new(vec![]);

        let config = CachingDownloadConfig {
            url: Url::parse("http://localhost/fake").expect("could not create url"),
            file_path: file_path.to_path_buf(),
            temp_file_path: temp_file_path.to_path_buf(),
            #[cfg(feature = "xsd-downloader-force-download")]
            force_download: false,
            hash: TEST_HASH,
        };

        // Run the test
        let result = caching_download(
            config,
            |url: &Url, file_name: &str| downloader.download(url, file_name),
            |path_buf: &PathBuf, hash: &str| validator.validate(path_buf, hash),
        );

        assert_eq!(validator.get_calls(), &[]);
        assert!(result.is_ok());
        assert_eq!(downloader.get_calls(), &[]);
    }

    #[test]
    // Test that CachingDownloader downloads if file is not cached
    fn caching_download_downloads_file() -> Result<(), CliError> {
        let temp_dir = TempDir::new("example").expect("could not create tempdir");
        let dest_dir = temp_dir.path();

        let file_name = "gs1.zip";
        let temp_file_name = "gs1-temp.zip";

        let file_path = dest_dir.join(file_name);
        let temp_file_path = dest_dir.join(temp_file_name);

        assert!(!file_path.exists());

        let mut downloader = MockDownloader::new(vec![Ok(())]);
        let mut validator = MockValidator::new(vec![Ok(())]);

        let url = Url::parse("http://localhost/fake").expect("could not create url");

        let config = CachingDownloadConfig {
            url: url.clone(),
            file_path: file_path.to_path_buf(),
            temp_file_path: temp_file_path.to_path_buf(),
            #[cfg(feature = "xsd-downloader-force-download")]
            force_download: false,
            hash: TEST_HASH,
        };

        // Run the test
        caching_download(
            config,
            |url: &Url, file_name: &str| downloader.download(url, file_name),
            |path_buf: &PathBuf, hash: &str| validator.validate(path_buf, hash),
        )?;

        assert_eq!(
            validator.get_calls(),
            &[MockValidatorCall {
                path_buf: temp_file_path.to_path_buf(),
                hash: TEST_HASH.to_string(),
            }]
        );
        assert_eq!(
            downloader.get_calls(),
            &[MockDownloaderCall {
                url,
                file_name: dest_dir.join(temp_file_name).to_string_lossy().to_string()
            }]
        );
        assert!(file_path.exists());
        assert!(!temp_file_path.exists());
        Ok(())
    }

    #[test]
    #[cfg(feature = "xsd-downloader-force-download")]
    // Test that CachingDownloader downloads if file is cached, but force_download is enabled
    fn caching_download_downloads_cached_file_with_force_download() -> Result<(), CliError> {
        let temp_dir = TempDir::new("example").expect("could not create tempdir");
        let dest_dir = temp_dir.path();

        let file_name = "gs1.zip";
        let temp_file_name = "gs1-temp.zip";

        let file_path = dest_dir.join(file_name);
        let temp_file_path = dest_dir.join(temp_file_name);

        // Create the temporary file
        File::create(file_path.clone()).expect("could not create file");
        assert!(file_path.exists());

        let mut downloader = MockDownloader::new(vec![Ok(())]);
        let mut validator = MockValidator::new(vec![Ok(())]);

        let url = Url::parse("http://localhost/fake").expect("could not create url");

        let config = CachingDownloadConfig {
            url: url.clone(),
            file_path: file_path.to_path_buf(),
            temp_file_path: temp_file_path.to_path_buf(),
            force_download: true,
            hash: TEST_HASH,
        };

        // Run the test
        caching_download(
            config,
            |url: &Url, file_name: &str| downloader.download(url, file_name),
            |path_buf: &PathBuf, hash: &str| validator.validate(path_buf, hash),
        )?;

        assert_eq!(
            validator.get_calls(),
            &[MockValidatorCall {
                path_buf: temp_file_path.to_path_buf(),
                hash: TEST_HASH.to_string(),
            }]
        );
        assert_eq!(
            downloader.get_calls(),
            &[MockDownloaderCall {
                url,
                file_name: dest_dir.join(temp_file_name).to_string_lossy().to_string()
            }]
        );
        assert!(!temp_file_path.exists());
        Ok(())
    }

    #[test]
    #[cfg(feature = "xsd-downloader-force-download")]
    // Test that CachingDownloader fails if the temporary file already exists
    fn caching_download_fails_if_temp_file_exists() {
        let temp_dir = TempDir::new("example").expect("could not create tempdir");
        let dest_dir = temp_dir.path();

        let file_name = "gs1.zip";
        let temp_file_name = "gs1-temp.zip";

        let file_path = dest_dir.join(file_name);
        let temp_file_path = dest_dir.join(temp_file_name);

        // Create the temporary file
        File::create(temp_file_path.clone()).expect("could not create temp file");
        assert!(temp_file_path.exists());

        let mut downloader = MockDownloader::new(vec![]);
        let mut validator = MockValidator::new(vec![]);

        let config = CachingDownloadConfig {
            url: Url::parse("http://localhost/fake").expect("could not create url"),
            file_path: file_path.to_path_buf(),
            temp_file_path: temp_file_path.to_path_buf(),
            force_download: true,
            hash: TEST_HASH,
        };

        // Run the test
        let result = caching_download(
            config,
            |url: &Url, file_name: &str| downloader.download(url, file_name),
            |path_buf: &PathBuf, hash: &str| validator.validate(path_buf, hash),
        );

        assert_eq!(validator.get_calls(), &[]);
        assert!(result.is_err());
        assert_eq!(downloader.get_calls(), &[]);
    }

    #[test]
    // Test that CachingDownloader fails if the temporary file already exists
    fn caching_download_fails_if_hash_incorrect() -> Result<(), CliError> {
        let temp_dir = TempDir::new("example").expect("could not create tempdir");
        let dest_dir = temp_dir.path();

        let file_name = "gs1.zip";
        let temp_file_name = "gs1-temp.zip";

        let file_path = dest_dir.join(file_name);
        let temp_file_path = dest_dir.join(temp_file_name);

        assert!(!file_path.exists());

        let mut downloader = MockDownloader::new(vec![Ok(())]);
        let mut validator =
            MockValidator::new(vec![Err(CliError::ActionError("hash failure".to_string()))]);

        let url = Url::parse("http://localhost/fake").expect("could not create url");

        let config = CachingDownloadConfig {
            url: url.clone(),
            file_path: file_path.to_path_buf(),
            temp_file_path: temp_file_path.to_path_buf(),
            #[cfg(feature = "xsd-downloader-force-download")]
            force_download: false,
            hash: TEST_HASH,
        };

        // Run the test
        let result = caching_download(
            config,
            |url: &Url, file_name: &str| downloader.download(url, file_name),
            |path_buf: &PathBuf, hash: &str| validator.validate(path_buf, hash),
        );

        assert_eq!(
            validator.get_calls(),
            &[MockValidatorCall {
                path_buf: temp_file_path.to_path_buf(),
                hash: TEST_HASH.to_string(),
            }]
        );
        assert!(result.is_err());
        assert_eq!(
            downloader.get_calls(),
            &[MockDownloaderCall {
                url,
                file_name: dest_dir.join(temp_file_name).to_string_lossy().to_string()
            }]
        );
        assert!(!temp_file_path.exists());
        Ok(())
    }
}

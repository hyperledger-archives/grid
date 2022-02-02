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
mod downloader;
mod extractor;
mod validator;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use reqwest::Url;

use crate::actions;
use crate::error::CliError;
use downloader::CachingDownloadConfig;

const ENV_GRID_CACHE_DIR: &str = "GRID_CACHE_DIR";
const DEFAULT_GRID_CACHE_DIR: &str = "/var/cache/grid";

#[derive(Clone, Debug, PartialEq)]
struct UrlFile {
    url: &'static str,
    hash: &'static str,
    artifact_name: &'static str,
    extract_to: &'static str,
}

/// Files to be downloaded, and where they go
const DOWNLOADS: &[UrlFile] = &[UrlFile {
    url: "https://www.gs1.org/docs/EDI/xml/3.4.1/GS1_XML_3-4-1_Publication.zip",
    hash: "4a10f96d32fd0f73b39b0d969cb7b822f49b1bd1e7c0782cfe3648b70dd77376",
    artifact_name: "GS1_XML_3-4-1_Publication.zip",
    extract_to: "po",
}];

/// Defines how file downloads will be handled in a cached context
#[derive(Clone, Debug, PartialEq)]
pub enum DownloadConfig {
    #[cfg(feature = "xsd-downloader-force-download")]
    Always,
    CacheOnly,
    IfNotCached,
}

#[derive(Clone, Debug, PartialEq)]
struct FetchAndExtractConfig {
    download_config: DownloadConfig,
    copy_from: Option<String>,
    do_checksum: bool,
    url_files: &'static [UrlFile],
    artifact_dir: PathBuf,
    schema_dir: PathBuf,
}

/// Fetch xsds and extract them
///
/// * `config` - Behavior when downloading the file
/// * `download` - Function to download the files
/// * `validate_hash` - Function to validate file hashes
/// * `extract` - Function to extract the files
fn fetch_and_extract_with_callbacks(
    config: FetchAndExtractConfig,
    mut download: impl FnMut(CachingDownloadConfig) -> Result<(), CliError>,
    mut validate_hash: impl FnMut(&Path, &str) -> Result<(), CliError>,
    mut extract: impl FnMut(&Path, &Path) -> Result<(), CliError>,
) -> Result<(), CliError> {
    let copy_path = if let Some(ref dir) = config.copy_from {
        let path = Path::new(dir);
        if !path.exists() {
            return Err(CliError::ActionError(format!(
                "could not copy from {dir}, as path does not exist"
            )));
        }

        let metadata = fs::metadata(path).map_err(|err| {
            CliError::InternalError(format!(
                "could not read metadata from directory \"{dir}\": {err}",
                dir = path.to_string_lossy()
            ))
        })?;

        if !metadata.is_dir() {
            return Err(CliError::ActionError(format!(
                "could not copy from {dir}, as the specified path is not a directory"
            )));
        }

        Some(path)
    } else {
        None
    };

    for file in config.url_files {
        let filename = file.artifact_name;
        let file_path = config.artifact_dir.join(filename);
        let file_name = file_path
            .file_name()
            .ok_or_else(|| {
                CliError::InternalError(format!(
                    "error getting filename of \"{file_path}\"",
                    file_path = file_path.to_string_lossy()
                ))
            })?
            .to_string_lossy();
        let mut checksum_validated = false;

        let mut validate = |file_path: &Path| {
            let result = (validate_hash)(file_path, file.hash);

            if result.is_ok() {
                info!("Hash of \"{file_name}\" verified: {hash}", hash = file.hash);
            }

            result
        };

        if let Some(ref dir) = config.copy_from {
            let copy_path = copy_path.ok_or_else(|| {
                CliError::InternalError("sanity check fail: path unavailable".to_string())
            })?;

            let copy_file_path = copy_path.join(filename);

            if copy_file_path.exists() {
                if config.do_checksum {
                    // Do a validation of the file before copying to cache
                    validate(&copy_file_path)?;

                    // Make sure we don't revalidate later
                    // (hashing can take a non-trivial amount of time)
                    checksum_validated = true;
                } else {
                    debug!("skipping checksum");
                }

                debug!("skipping download for {filename}, copying from {dir}");
                fs::copy(&copy_file_path, &file_path).map_err(|err| {
                    CliError::InternalError(format!(
                        "could not copy \"{copy_file_path}\" to \"{file_path}\": {err}",
                        copy_file_path = copy_file_path.to_string_lossy(),
                        file_path = file_path.to_string_lossy()
                    ))
                })?;
            } else if config.download_config == DownloadConfig::CacheOnly {
                // If we are copying and not downloading, we expect all
                // necessary files to exist in the copy directory, regardless
                // of whether they exist in cache
                return Err(CliError::ActionError(format!(
                    "expected to find \"{filename}\" \
                in \"{copy_path}\", but the file does not exist",
                    copy_path = copy_path.to_string_lossy(),
                )));
            } else {
                info!("file \"{filename}\" doesn't exist in copy path");
            }
        }

        match config.download_config {
            DownloadConfig::CacheOnly => {
                debug!("skipping download, using cache only");
            }
            _ => {
                let url = Url::parse(file.url).map_err(|err| {
                    CliError::InternalError(format!(
                        "sanity check fail: unable to parse \"{url}\": {err}",
                        url = file.url
                    ))
                })?;

                let config = CachingDownloadConfig {
                    url,
                    file_path: file_path.to_path_buf(),
                    temp_file_path: config.artifact_dir.join(format!("{filename}.download")),
                    #[cfg(feature = "xsd-downloader-force-download")]
                    force_download: config.download_config == DownloadConfig::Always,
                    hash: file.hash,
                };

                (download)(config)?;

                // Make sure we don't revalidate later
                // (hashing can take a non-trivial amount of time)
                checksum_validated = true;
            }
        }

        if !file_path.exists() {
            return Err(CliError::ActionError(format!(
                "file \"{file_path}\" does not exist",
                file_path = file_path.to_string_lossy(),
            )));
        }

        if !checksum_validated && config.do_checksum {
            // Do a validation of the final file
            validate(&file_path)?;
        } else {
            debug!("skipping checksum");
        }

        let dest_dir = config.schema_dir.join(file.extract_to);

        info!(
            "Extracting \"{file_name}\" to \"{dest_dir}\"",
            dest_dir = dest_dir.to_string_lossy()
        );

        (extract)(&file_path, &dest_dir)?;
    }

    Ok(())
}

/// Test that the given path is directory and writable.
/// Return the appropriate error if not.
///
/// * `path` - The path of the directory to check
fn test_directory_writable(ctx: &str, path: &Path) -> Result<(), CliError> {
    if !path.exists() {
        return Err(CliError::ActionError(format!(
            "{ctx} path \"{path}\" does not exist",
            path = path.display()
        )));
    }

    if !path.is_dir() {
        return Err(CliError::ActionError(format!(
            "{ctx} path \"{path}\" is not a directory",
            path = path.display()
        )));
    }

    let permissions = path
        .metadata()
        .map_err(|err| {
            CliError::ActionError(format!(
                "{ctx} path \"{path}\" is not writable: {err}",
                path = path.display(),
                err = err
            ))
        })?
        .permissions();

    if permissions.readonly() {
        return Err(CliError::ActionError(format!(
            "{ctx} path \"{path}\" is not writable",
            path = path.display()
        )));
    }

    Ok(())
}

/// Fetch xsds and extract them
///
/// * `artifact_dir` - Location to store and retrieve artifacts from
/// * `download_config` - Behavior when downloading the file
/// * `do_checksum` - Whether to perform a checksum on the cached file
pub fn fetch_and_extract_xsds(
    artifact_dir: Option<&str>,
    download_config: DownloadConfig,
    do_checksum: bool,
    copy_from: Option<String>,
) -> Result<(), CliError> {
    let artifact_dir = match artifact_dir {
        Some(dir) => {
            let path = PathBuf::from(dir);
            test_directory_writable("cache", &path).map(|_| path)
        }
        None => {
            let parent_path = PathBuf::from(
                env::var(ENV_GRID_CACHE_DIR).unwrap_or_else(|_| DEFAULT_GRID_CACHE_DIR.to_string()),
            );

            test_directory_writable("cache", &parent_path)?;

            let path = parent_path.join("xsd_artifact_cache");

            if !path.exists() {
                fs::create_dir(&path).map_err(|e| {
                    CliError::ActionError(format!(
                        "could not create artifact directory \"{dir}\": {e}",
                        dir = path.display()
                    ))
                })?;
            }

            Ok(path)
        }
    }?;

    debug!("using artifact directory of {}", artifact_dir.display());

    // Make sure the state directory exists and is writable, since the schema directory
    // is contained within this directory
    let state_dir = actions::get_grid_state_dir();
    let state_dir = PathBuf::from(state_dir);
    test_directory_writable("state", &state_dir)?;

    let schema_dir = actions::get_grid_xsd_dir();
    if !schema_dir.exists() {
        fs::create_dir(&schema_dir).map_err(|e| {
            CliError::ActionError(format!(
                "could not create schema directory \"{dir}\": {e}",
                dir = schema_dir.to_string_lossy()
            ))
        })?;
    }

    debug!("using schema directory of {}", schema_dir.display());

    fetch_and_extract_with_callbacks(
        FetchAndExtractConfig {
            download_config,
            copy_from,
            do_checksum,
            url_files: DOWNLOADS,
            artifact_dir,
            schema_dir,
        },
        |config| {
            downloader::caching_download(
                config,
                downloader::download,
                |path_buf: &PathBuf, hash: &str| validator::validate_hash(path_buf, hash),
            )
        },
        validator::validate_hash,
        extractor::extract,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;
    use std::fs::{self, File};
    use std::path::Path;
    use tempdir::TempDir;

    #[derive(Debug, PartialEq)]
    struct MockCacheDownloaderCall {
        pub config: CachingDownloadConfig,
    }

    struct MockCacheDownloader {
        responses: Vec<Result<(), CliError>>,
        calls: Vec<MockCacheDownloaderCall>,
    }

    impl MockCacheDownloader {
        pub fn new(responses: Vec<Result<(), CliError>>) -> MockCacheDownloader {
            MockCacheDownloader {
                responses,
                calls: vec![],
            }
        }

        pub fn download(&mut self, config: CachingDownloadConfig) -> Result<(), CliError> {
            self.calls.push(MockCacheDownloaderCall { config });
            self.responses.pop().unwrap()
        }

        pub fn get_calls(&self) -> &[MockCacheDownloaderCall] {
            &self.calls
        }
    }

    #[derive(Debug, PartialEq)]
    pub struct MockValidatorCall {
        pub path_buf: PathBuf,
        pub hash: String,
    }

    pub struct MockValidator {
        responses: Vec<Result<(), CliError>>,
        calls: Vec<MockValidatorCall>,
    }

    impl MockValidator {
        pub fn new(responses: Vec<Result<(), CliError>>) -> MockValidator {
            MockValidator {
                responses,
                calls: vec![],
            }
        }

        pub fn validate(&mut self, path_buf: &Path, hash: &str) -> Result<(), CliError> {
            self.calls.push(MockValidatorCall {
                path_buf: path_buf.to_path_buf(),
                hash: hash.to_string(),
            });

            self.responses.pop().unwrap()
        }

        pub fn get_calls(&self) -> &[MockValidatorCall] {
            &self.calls
        }
    }

    #[derive(Debug, PartialEq)]
    pub struct MockExtractorCall {
        pub source: PathBuf,
        pub dest: PathBuf,
    }

    pub struct MockExtractor {
        responses: Vec<Result<(), CliError>>,
        calls: Vec<MockExtractorCall>,
    }

    impl MockExtractor {
        pub fn new(responses: Vec<Result<(), CliError>>) -> MockExtractor {
            MockExtractor {
                responses,
                calls: Vec::new(),
            }
        }

        pub fn extract(&mut self, source: &Path, dest: &Path) -> Result<(), CliError> {
            self.calls.push(MockExtractorCall {
                source: source.to_path_buf(),
                dest: dest.to_path_buf(),
            });

            self.responses.pop().unwrap()
        }

        pub fn get_calls(&self) -> &[MockExtractorCall] {
            &self.calls
        }
    }

    #[test]
    fn fae_default_configuration_downloads_and_extracts() {
        let temp_dir = TempDir::new("fae_xsds").expect("could not create tempdir");
        let path = temp_dir.into_path();
        let artifact_dir = path.join("artifact");
        let schema_dir = path.join("schema");

        fs::create_dir(&artifact_dir).expect("could not create directory");
        File::create(&artifact_dir.join("out.zip")).expect("could not create file");

        let mut downloader = MockCacheDownloader::new(vec![Ok(())]);
        let mut validator = MockValidator::new(vec![Ok(())]);
        let mut extractor = MockExtractor::new(vec![Ok(())]);

        let config = FetchAndExtractConfig {
            download_config: DownloadConfig::IfNotCached,
            copy_from: None,
            do_checksum: true,
            url_files: &[UrlFile {
                url: "https://bismuth/zepplin.zip",
                hash: "notarealhash",
                artifact_name: "out.zip",
                extract_to: "edgarallen",
            }],
            artifact_dir: artifact_dir.clone(),
            schema_dir: schema_dir.clone(),
        };

        fetch_and_extract_with_callbacks(
            config.clone(),
            |config| downloader.download(config),
            |path_buf, hash| validator.validate(path_buf, hash),
            |source, dest| extractor.extract(source, dest),
        )
        .expect("failed to fetch and extract");

        assert_eq!(
            downloader.get_calls(),
            &config
                .url_files
                .iter()
                .map(|file| MockCacheDownloaderCall {
                    config: CachingDownloadConfig {
                        url: Url::parse(file.url).expect("could not parse url"),
                        file_path: artifact_dir.join(file.artifact_name).to_path_buf(),
                        temp_file_path: artifact_dir.join(format!(
                            "{filename}.download",
                            filename = file.artifact_name
                        )),
                        #[cfg(feature = "xsd-downloader-force-download")]
                        force_download: false,
                        hash: file.hash,
                    }
                })
                .collect::<Vec<MockCacheDownloaderCall>>()
        );

        assert_eq!(validator.get_calls(), &[]);

        assert_eq!(
            extractor.get_calls(),
            &config
                .url_files
                .iter()
                .map(|file| MockExtractorCall {
                    source: artifact_dir.join(file.artifact_name).to_path_buf(),
                    dest: schema_dir.join(file.extract_to),
                })
                .collect::<Vec<MockExtractorCall>>()
        );
    }

    // Validate that files are copied if that download option is selected
    #[test]
    fn fae_files_copy_if_copy_option_enabled() {
        let temp_dir = TempDir::new("fae_xsds").expect("could not create tempdir");
        let path = temp_dir.into_path();
        let artifact_dir = path.join("artifact");
        fs::create_dir(&artifact_dir).expect("could not create directory");

        let schema_dir = path.join("schema");

        let copy_dir = path.join("copy_from");
        fs::create_dir(&copy_dir).expect("could not create directory");
        File::create(&copy_dir.join("out.zip")).expect("could not create file");

        let mut downloader = MockCacheDownloader::new(vec![]);
        let mut validator = MockValidator::new(vec![Ok(()), Ok(())]);
        let mut extractor = MockExtractor::new(vec![Ok(())]);

        let config = FetchAndExtractConfig {
            download_config: DownloadConfig::CacheOnly,
            copy_from: Some(copy_dir.to_string_lossy().to_string()),
            do_checksum: true,
            url_files: &[UrlFile {
                url: "https://bismuth/zepplin.zip",
                hash: "notarealhash",
                artifact_name: "out.zip",
                extract_to: "edgarallen",
            }],
            artifact_dir: artifact_dir.clone(),
            schema_dir: schema_dir.clone(),
        };

        fetch_and_extract_with_callbacks(
            config.clone(),
            |config| downloader.download(config),
            |path_buf, hash| validator.validate(path_buf, hash),
            |source, dest| extractor.extract(source, dest),
        )
        .expect("failed to fetch and extract");

        assert_eq!(downloader.get_calls(), &[],);

        assert_eq!(
            validator.get_calls(),
            &config
                .url_files
                .iter()
                .map(|file| MockValidatorCall {
                    path_buf: copy_dir.join(file.artifact_name).to_path_buf(),
                    hash: file.hash.to_string(),
                })
                .collect::<Vec<MockValidatorCall>>()
        );

        assert_eq!(
            extractor.get_calls(),
            &config
                .url_files
                .iter()
                .map(|file| MockExtractorCall {
                    source: artifact_dir.join(file.artifact_name).to_path_buf(),
                    dest: schema_dir.join(file.extract_to),
                })
                .collect::<Vec<MockExtractorCall>>()
        );
    }

    // Validate that hash checking is disabled if the option is not selected
    #[test]
    fn fae_files_validation_does_not_happen_if_disabled() {
        let temp_dir = TempDir::new("fae_xsds").expect("could not create tempdir");
        let path = temp_dir.into_path();
        let artifact_dir = path.join("artifact");
        let schema_dir = path.join("schema");

        fs::create_dir(&artifact_dir).expect("could not create directory");
        File::create(&artifact_dir.join("out.zip")).expect("could not create file");

        let mut downloader = MockCacheDownloader::new(vec![Ok(())]);
        let mut validator = MockValidator::new(vec![]);
        let mut extractor = MockExtractor::new(vec![Ok(())]);

        let config = FetchAndExtractConfig {
            download_config: DownloadConfig::IfNotCached,
            copy_from: None,
            do_checksum: false,
            url_files: &[UrlFile {
                url: "https://bismuth/zepplin.zip",
                hash: "notarealhash",
                artifact_name: "out.zip",
                extract_to: "edgarallen",
            }],
            artifact_dir: artifact_dir.clone(),
            schema_dir: schema_dir.clone(),
        };

        fetch_and_extract_with_callbacks(
            config.clone(),
            |config| downloader.download(config),
            |path_buf, hash| validator.validate(path_buf, hash),
            |source, dest| extractor.extract(source, dest),
        )
        .expect("failed to fetch and extract");

        assert_eq!(
            downloader.get_calls(),
            &config
                .url_files
                .iter()
                .map(|file| MockCacheDownloaderCall {
                    config: CachingDownloadConfig {
                        url: Url::parse(file.url).expect("could not parse url"),
                        file_path: artifact_dir.join(file.artifact_name).to_path_buf(),
                        temp_file_path: artifact_dir.join(format!(
                            "{filename}.download",
                            filename = file.artifact_name
                        )),
                        #[cfg(feature = "xsd-downloader-force-download")]
                        force_download: false,
                        hash: file.hash,
                    }
                })
                .collect::<Vec<MockCacheDownloaderCall>>()
        );

        assert_eq!(validator.get_calls(), &[],);

        assert_eq!(
            extractor.get_calls(),
            &config
                .url_files
                .iter()
                .map(|file| MockExtractorCall {
                    source: artifact_dir.join(file.artifact_name).to_path_buf(),
                    dest: schema_dir.join(file.extract_to),
                })
                .collect::<Vec<MockExtractorCall>>()
        );
    }

    #[test]
    fn fae_cache_only_with_missing_artifact_fails() {
        let temp_dir = TempDir::new("fae_xsds").expect("could not create tempdir");
        let path = temp_dir.into_path();
        let schema_dir = path.join("schema");

        let mut downloader = MockCacheDownloader::new(vec![]);
        let mut validator = MockValidator::new(vec![]);
        let mut extractor = MockExtractor::new(vec![]);

        let config = FetchAndExtractConfig {
            download_config: DownloadConfig::CacheOnly,
            copy_from: None,
            do_checksum: true,
            url_files: &[UrlFile {
                url: "https://bismuth/zepplin.zip",
                hash: "notarealhash",
                artifact_name: "out.zip",
                extract_to: "edgarallen",
            }],
            artifact_dir: PathBuf::from("fakedir"),
            schema_dir: schema_dir.clone(),
        };

        let result = fetch_and_extract_with_callbacks(
            config.clone(),
            |config| downloader.download(config),
            |path_buf, hash| validator.validate(path_buf, hash),
            |source, dest| extractor.extract(source, dest),
        );

        assert_eq!(
            format!("{:?}", result),
            "Err(ActionError(\"file \\\"fakedir/out.zip\\\" does not exist\"))"
        );
        assert_eq!(downloader.get_calls(), &[],);
        assert_eq!(validator.get_calls(), &[],);
        assert_eq!(extractor.get_calls(), &[],);
    }

    #[test]
    fn fae_cache_only_copy_from_missing_file_fails() {
        let temp_dir = TempDir::new("fae_xsds").expect("could not create tempdir");
        let path = temp_dir.into_path();
        let schema_dir = path.join("schema");

        let mut downloader = MockCacheDownloader::new(vec![]);
        let mut validator = MockValidator::new(vec![]);
        let mut extractor = MockExtractor::new(vec![]);

        let copy_dir = path.join("copy_from");
        let copy_dir_string = copy_dir.to_string_lossy().to_string();
        fs::create_dir(&copy_dir).expect("could not create directory");

        let config = FetchAndExtractConfig {
            download_config: DownloadConfig::CacheOnly,
            copy_from: Some(copy_dir.to_string_lossy().to_string()),
            do_checksum: true,
            url_files: &[UrlFile {
                url: "https://bismuth/zepplin.zip",
                hash: "notarealhash",
                artifact_name: "out.zip",
                extract_to: "edgarallen",
            }],
            artifact_dir: PathBuf::from("fakedir"),
            schema_dir: schema_dir.clone(),
        };

        let result = fetch_and_extract_with_callbacks(
            config.clone(),
            |config| downloader.download(config),
            |path_buf, hash| validator.validate(path_buf, hash),
            |source, dest| extractor.extract(source, dest),
        );

        assert_eq!(
            format!("{:?}", result),
            format!(
                "Err(ActionError(\"expected to find \\\"out.zip\\\" in \
            \\\"{copy_dir_string}\\\", but the file does not exist\"))"
            )
        );
        assert_eq!(downloader.get_calls(), &[],);
        assert_eq!(validator.get_calls(), &[],);
        assert_eq!(extractor.get_calls(), &[],);
    }

    #[test]
    fn fae_if_not_cached_copy_from_missing_file_downloads() {
        let temp_dir = TempDir::new("fae_xsds").expect("could not create tempdir");
        let path = temp_dir.into_path();
        let schema_dir = path.join("schema");
        let artifact_dir = path.join("artifact");

        fs::create_dir(&artifact_dir).expect("could not create directory");

        let mut downloader = MockCacheDownloader::new(vec![Ok(())]);
        let mut validator = MockValidator::new(vec![Ok(())]);
        let mut extractor = MockExtractor::new(vec![Ok(())]);

        let copy_dir = path.join("copy_from");
        fs::create_dir(&copy_dir).expect("could not create directory");

        let config = FetchAndExtractConfig {
            download_config: DownloadConfig::IfNotCached,
            copy_from: Some(copy_dir.to_string_lossy().to_string()),
            do_checksum: true,
            url_files: &[UrlFile {
                url: "https://bismuth/zepplin.zip",
                hash: "notarealhash",
                artifact_name: "out.zip",
                extract_to: "edgarallen",
            }],
            artifact_dir: artifact_dir.clone(),
            schema_dir: schema_dir.clone(),
        };

        let result = fetch_and_extract_with_callbacks(
            config.clone(),
            |config| {
                File::create(&artifact_dir.join(&config.file_path)).expect("could not create file");
                downloader.download(config)
            },
            |path_buf, hash| validator.validate(path_buf, hash),
            |source, dest| extractor.extract(source, dest),
        );

        assert_eq!(format!("{:?}", result), "Ok(())");
        assert_eq!(
            downloader.get_calls(),
            &config
                .url_files
                .iter()
                .map(|file| MockCacheDownloaderCall {
                    config: CachingDownloadConfig {
                        url: Url::parse(file.url).expect("could not parse url"),
                        file_path: artifact_dir.join(file.artifact_name).to_path_buf(),
                        temp_file_path: artifact_dir.join(format!(
                            "{filename}.download",
                            filename = file.artifact_name
                        )),
                        #[cfg(feature = "xsd-downloader-force-download")]
                        force_download: false,
                        hash: file.hash,
                    }
                })
                .collect::<Vec<MockCacheDownloaderCall>>()
        );

        assert_eq!(validator.get_calls(), &[],);

        assert_eq!(
            extractor.get_calls(),
            &config
                .url_files
                .iter()
                .map(|file| MockExtractorCall {
                    source: artifact_dir.join(file.artifact_name).to_path_buf(),
                    dest: schema_dir.join(file.extract_to),
                })
                .collect::<Vec<MockExtractorCall>>()
        );
    }

    #[test]
    fn test_directory_writable_succeeds_if_writable() {
        let temp_dir = TempDir::new("writable").expect("could not create tempdir");
        let path = temp_dir.into_path();

        test_directory_writable("test", &path).expect("could not write to directory");
    }

    #[test]
    fn test_directory_writable_fails_if_not_directory() {
        let path = Path::new("/fake/directory/is/fake");

        let result = test_directory_writable("test", path);

        assert_eq!(
            format!("{:?}", result),
            "Err(ActionError(\"test path \\\"/fake/directory/is/fake\\\" \
                does not exist\"))"
        );
    }
}

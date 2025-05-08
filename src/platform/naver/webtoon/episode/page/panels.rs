use crate::platform::naver::{Client, errors::DownloadError, webtoon::episode::EpisodeError};
use anyhow::{Context, anyhow};
use image::{GenericImageView, ImageFormat, RgbaImage};
use scraper::{Html, Selector};
use std::path::Path;
use tokio::{fs::File, io::AsyncWriteExt};
use url::Url;

/// Represents a single panel for an episode.
#[derive(Debug, Clone)]
pub struct Panel {
    pub(in crate::platform::naver::webtoon::episode) url: Url,
    pub(in crate::platform::naver::webtoon::episode) bytes: Vec<u8>,
    pub(in crate::platform::naver::webtoon::episode) episode: u16,
    pub(in crate::platform::naver::webtoon::episode) number: u16,
}

impl Panel {
    /// Returns the url for the panel.
    #[must_use]
    pub fn url(&self) -> &str {
        self.url.as_str()
    }

    pub(in crate::platform::naver::webtoon::episode) async fn download(
        &mut self,
        client: &Client,
    ) -> Result<(), EpisodeError> {
        self.bytes = client
            .http
            .get(self.url.as_str())
            .header("Referer", "https://comic.naver.com/")
            .send()
            .await?
            .bytes()
            .await?
            .to_vec();

        Ok(())
    }
}

pub(super) fn from_html(html: &Html, episode: u16) -> Result<Vec<Panel>, EpisodeError> {
    let selector = Selector::parse(r"div.wt_viewer>img") //
        .expect("`div.wt_viewer>img` should be a valid selector");

    let mut panels = Vec::new();

    #[allow(unused, reason = "not all features use `number`")]
    for (number, img) in html.select(&selector).enumerate() {
        let url = img
            .value()
            .attr("src")
            .context("`src` is missing, `img` should always have one")?;

        let url = Url::parse(url).map_err(|err| EpisodeError::Unexpected(err.into()))?;

        panels.push(Panel {
            url,

            episode,
            // Enumerate starts at 0. +1 so that it starts at one.
            number: u16::try_from(number + 1)
                .context("there shouldn't be more than 65,536 panels for an episode")?,
            bytes: Vec::new(),
        });
    }

    if panels.is_empty() {
        return Err(EpisodeError::NoPanelsFound);
    }

    Ok(panels)
}

/// Represents all the panels for an episode.
#[derive(Debug, Clone)]
pub struct Panels {
    pub(in crate::platform::naver::webtoon::episode) images: Vec<Panel>,
    pub(in crate::platform::naver::webtoon::episode) height: u32,
    pub(in crate::platform::naver::webtoon::episode) width: u32,
}

impl Panels {
    /// Saves all the panels of an episode as a single long image file in PNG format.
    ///
    /// # Behavior
    ///
    /// - Combines all panels of the episode vertically into one long image.
    /// - The output image is always saved as a PNG file, even if the original panels are in a different format (e.g., JPEG), due to JPEG's limitations.
    /// - If the directory specified by `path` does not exist, it will be created along with any required parent directories.
    ///
    /// # Parameters
    ///
    /// - `path`: The target directory where the combined image will be saved. If it doesn't exist, it will be created.
    ///
    /// # Errors
    ///
    /// - Returns a [`DownloadError`] if any issues arise during directory creation, image creation, or writing the combined image to disk.
    pub async fn save_single<P>(&self, path: P) -> Result<(), DownloadError>
    where
        P: AsRef<Path> + Send,
    {
        let path = path.as_ref();

        tokio::fs::create_dir_all(path).await?;

        let episode = self.images[0].episode;

        let path = path.join(episode.to_string()).with_extension("png");

        File::create(&path)
            .await
            .context("failed to create download file")?;

        let mut single = RgbaImage::new(self.width, self.height);

        let mut offset = 0;

        for image in &self.images {
            let image = image::load_from_memory(&image.bytes) //
                .context("failed to load image from memory")?;

            for (x, y, pixels) in image.pixels() {
                single.put_pixel(x, y + offset, pixels);
            }

            offset += image.height();
        }

        tokio::task::spawn_blocking(move || single.save_with_format(path, ImageFormat::Png))
            .await
            .context("Failed `spawn_blocking`")?
            .context("Failed to save image to disk")?;

        Ok(())
    }

    /// Saves each panel of the episode to disk, naming the resulting files using the format `EPISODE_NUMBER-PANEL_NUMBER`.
    ///
    /// For example, the first panel of the 34th episode would be saved as `34-1`. The file extension will match the panel's original format.
    ///
    /// # Behavior
    ///
    /// - If the specified directory does not exist, it will be created, along with any necessary parent directories.
    ///
    /// # Parameters
    ///
    /// - `path`: The destination directory where the panels will be saved. If the path does not exist, it will be created automatically.
    ///
    /// # Errors
    ///
    /// - Returns a [`DownloadError`] if there are any issues creating the directory, writing to the files, or processing the filesystem.
    pub async fn save_multiple<P>(&self, path: P) -> Result<(), DownloadError>
    where
        P: AsRef<Path> + Send,
    {
        let path = path.as_ref();

        tokio::fs::create_dir_all(path).await?;

        for panel in &self.images {
            let name = format!("{}-{}", panel.episode, panel.number);
            let path = path.join(name).with_extension("png");

            let mut file = File::create(&path)
                .await
                .context("failed to create download file")?;

            let bytes = panel.bytes.as_slice();

            file.write_all(bytes).await?;
        }

        Ok(())
    }
}

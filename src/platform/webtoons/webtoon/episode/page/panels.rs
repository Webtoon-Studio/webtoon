use crate::platform::webtoons::webtoon::episode::EpisodeError;
use anyhow::{Context, anyhow};
use scraper::{Html, Selector};
use url::Url;

use crate::platform::webtoons::{Client, errors::DownloadError};
use image::{GenericImageView, ImageFormat, RgbaImage};
use std::path::Path;
use tokio::{fs::File, io::AsyncWriteExt};

/// Represents a single panel for an episode.
///
/// This type is not constructed directly, but gotten via [`Episode::panels()`](super::super::Episode::panels()).
#[derive(Debug, Clone)]
pub struct Panel {
    pub(in crate::platform::webtoons::webtoon::episode) url: Url,
    pub(in crate::platform::webtoons::webtoon::episode) episode: u16,
    pub(in crate::platform::webtoons::webtoon::episode) number: u16,
    pub(in crate::platform::webtoons::webtoon::episode) ext: String,
    pub(in crate::platform::webtoons::webtoon::episode) bytes: Vec<u8>,
    pub(in crate::platform::webtoons::webtoon::episode) height: u32,
    pub(in crate::platform::webtoons::webtoon::episode) width: u32,
}

impl Panel {
    /// Returns the url for the panel.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(843910, Type::Canvas).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(1).await? {
    ///     for panel in episode.panels().await? {
    ///         println!("url: {}", panel.url());
    ///     }
    ///     # return Ok(());
    /// }
    /// # unreachable!("should have entered the episode block and returned");
    /// # Ok(())
    /// # }
    /// ```
    pub fn url(&self) -> &str {
        self.url.as_str()
    }

    pub(in crate::platform::webtoons::webtoon::episode) async fn download(
        &mut self,
        client: &Client,
    ) -> Result<(), EpisodeError> {
        let bytes = client
            .http
            .get(self.url.as_str())
            .send()
            .await?
            .bytes()
            .await?;

        self.bytes = bytes.to_vec();

        Ok(())
    }
}

#[allow(unused, reason = "not all features use `episode`")]
pub(super) fn from_html(html: &Html, episode: u16) -> Result<Vec<Panel>, EpisodeError> {
    let panels = if super::is_audio_reader(html) {
        return Ok(Vec::new());
    } else {
        from_normal_reader(html, episode)?
    };

    if panels.is_empty() {
        return Err(EpisodeError::Unexpected(anyhow!(
            "Failed to find a single panel on episode page"
        )));
    }

    Ok(panels)
}

fn from_normal_reader(html: &Html, episode: u16) -> Result<Vec<Panel>, EpisodeError> {
    let selector = Selector::parse(r"img._images") //
        .expect("`img._images` should be a valid selector");

    let mut panels = Vec::new();

    for (number, img) in html.select(&selector).enumerate() {
        let height = img
            .value()
            .attr("height")
            .context("`height` is missing, `img._images` should always have one")?
            .split('.')
            .next()
            .context("`height` attribute should be a float")?
            .parse::<u32>()
            .map_err(|err| EpisodeError::Unexpected(err.into()))?;

        let width = img
            .value()
            .attr("width")
            .context("`width` is missing, `img._images` should always have one")?
            .split('.')
            .next()
            .context("`width` attribute should be a float")?
            .parse::<u32>()
            .map_err(|err| EpisodeError::Unexpected(err.into()))?;

        let url = img
            .value()
            .attr("data-url")
            .context("`data-url` is missing, `img._images` should always have one")?;

        let mut url = Url::parse(url).map_err(|err| EpisodeError::Unexpected(err.into()))?;

        url.set_host(Some("swebtoon-phinf.pstatic.net"))
            .expect("`swebtoon-phinf.pstatic.net` should be a valid host");

        let ext = url
            .path()
            .split('.')
            .nth(1)
            .with_context(|| format!("`{url}` should end in an extension but didn't"))?
            .to_string();

        panels.push(Panel {
            url,

            episode,
            // Enumerate starts at 0. +1 so that it starts at one.
            number: u16::try_from(number + 1)
                .context("there shouldn't be more than 65,536 panels for an episode")?,
            height,
            width,
            ext,
            bytes: Vec::new(),
        });
    }

    Ok(panels)
}

/// Represents all the panels for an episode.
///
/// This type is not constructed directly, but gotten via [`Episode::panels()`](crate::platform::webtoons::webtoon::episode::Episode::panels()).
///
/// # Example
///
/// ```
/// # use webtoon::platform::webtoons::{errors::Error, Client, Type};
/// # #[tokio::main]
/// # async fn main() -> Result<(), Error> {
/// let client = Client::new();
///
/// let Some(webtoon) = client.webtoon(961, Type::Original).await? else {
///     unreachable!("webtoon is known to exist");
/// };
///
/// if let Some(episode) = webtoon.episode(1).await? {
///     let panels = episode.download().await?;
///     assert_eq!(52 , panels.count());
///     # return Ok(());
/// }
/// # unreachable!("should have entered the episode block and returned");
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Panels {
    pub(in crate::platform::webtoons::webtoon::episode) images: Vec<Panel>,
    pub(in crate::platform::webtoons::webtoon::episode) height: u32,
    pub(in crate::platform::webtoons::webtoon::episode) width: u32,
}

impl Panels {
    /// Returns how many `Panels` are on the episode.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(961, Type::Original).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(2).await? {
    ///     let panels = episode.download().await?;
    ///     assert_eq!(99 , panels.count());
    ///     # return Ok(());
    /// }
    /// # unreachable!("should have entered the episode block and returned");
    /// # Ok(())
    /// # }
    /// ```
    pub fn count(&self) -> usize {
        self.images.len()
    }

    /// Saves all the panels of an episode as a single long image file in PNG format.
    ///
    /// # Behavior
    ///
    /// - Combines all panels of the episode vertically into one long image.
    /// - The output image is always saved as a PNG file, even if the original panels are in a different format (e.g., JPEG), due to JPEG's limitations.
    /// - If the directory specified by `path` does not exist, it will be created along with any required parent directories.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(2960, Type::Original).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(1).await? {
    ///     let panels = episode.download().await?;
    ///     panels.save_single("panels/").await?;
    ///     # return Ok(());
    /// }
    /// # unreachable!("should have entered the episode block and returned");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn save_single<P>(&self, path: P) -> Result<(), DownloadError>
    where
        P: AsRef<Path> + Send,
    {
        let path = path.as_ref();

        tokio::fs::create_dir_all(path).await?;

        let ext = &self.images[0].ext;
        let episode = self.images[0].episode;
        let width = self.width;
        let height = self.height;

        let path = path.join(episode.to_string()).with_extension(ext);

        File::create(&path)
            .await
            .context("failed to create download file")?;

        let mut single = RgbaImage::new(width, height);

        let mut offset = 0;

        for panel in &self.images {
            let bytes = panel.bytes.as_slice();

            let image = image::load_from_memory(bytes) //
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
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(2960, Type::Original).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(2).await? {
    ///     let panels = episode.download().await?;
    ///     panels.save_multiple("panels/").await?;
    ///     # return Ok(());
    /// }
    /// # unreachable!("should have entered the episode block and returned");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn save_multiple<P>(&self, path: P) -> Result<(), DownloadError>
    where
        P: AsRef<Path> + Send,
    {
        let path = path.as_ref();

        tokio::fs::create_dir_all(path).await?;

        for panel in &self.images {
            let name = format!("{}-{}", panel.episode, panel.number);
            let path = path.join(name).with_extension(&panel.ext);

            let mut file = File::create(&path)
                .await
                .context("failed to create download file")?;

            let bytes = panel.bytes.as_slice();

            file.write_all(bytes).await?;
        }

        Ok(())
    }
}

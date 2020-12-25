use futures::prelude::*;
use reqwest::Client;
use serde::Deserialize;
use serde_xml_rs::from_str;
use ssdp_client::{search, SearchTarget};
use std::{fmt, time::Duration};
use thiserror::Error;
use url::Url;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to send request")]
    Request(#[from] reqwest::Error),
    #[error("failed to send SSDP request")]
    SSDPRequest(#[from] ssdp_client::Error),
    #[error("failed to parse URL")]
    URLParse(#[from] url::ParseError),
    #[error("failed to parse XML")]
    XMLParse(#[from] serde_xml_rs::Error),
    #[error("argument error `{0}`")]
    Argument(String),
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Device {
    url: Url,
    client: Client,
}

impl Device {
    pub fn new(url: Url) -> Device {
        Device {
            url,
            client: Client::new(),
        }
    }

    pub async fn discover() -> Result<Vec<Device>> {
        let search_target = SearchTarget::Custom("roku".into(), "ecp".into());
        let mut responses = search(&search_target, Duration::from_secs(3), 2).await?;
        let mut devices = vec![];
        while let Some(response) = responses.next().await {
            let url = Url::parse(response?.location())?;
            devices.push(Device {
                url,
                client: Client::new(),
            });
        }
        Ok(devices)
    }

    pub async fn apps(&self) -> Result<Apps> {
        let url = self.url.join("query/apps")?;
        let res = self.client.get(url).send().await?;
        let text = res.text().await?;
        Ok(from_str(&text)?)
    }

    pub async fn active_app(&self) -> Result<ActiveApp> {
        let url = self.url.join("query/active-app")?;
        let res = self.client.get(url).send().await?;
        let text = res.text().await?;
        Ok(from_str(&text)?)
    }

    pub async fn media_player(&self) -> Result<MediaPlayer> {
        let url = self.url.join("query/media-player")?;
        let res = self.client.get(url).send().await?;
        let text = res.text().await?;
        Ok(from_str(&text)?)
    }

    pub async fn keydown(&self, key: &Key) -> Result<()> {
        let url = self.url.join(&format!("keydown/{}", key.to_string()))?;
        self.client.post(url).send().await?;
        Ok(())
    }

    pub async fn keyup(&self, key: &Key) -> Result<()> {
        let url = self.url.join(&format!("keyup/{}", key.to_string()))?;
        self.client.post(url).send().await?;
        Ok(())
    }

    pub async fn keypress(&self, key: &Key) -> Result<()> {
        let url = self.url.join(&format!("keypress/{}", key.to_string()))?;
        println!("{}", url);
        self.client.post(url).send().await?;
        Ok(())
    }

    pub async fn launch(&self, app: &App) -> Result<()> {
        let app_id = app
            .id
            .as_ref()
            .ok_or_else(|| Error::Argument("app.id required".to_string()))?;
        let url = self.url.join(&format!("launch/{}", app_id))?;
        self.client.post(url).send().await?;
        Ok(())
    }

    pub async fn install(&self, app: &App) -> Result<()> {
        let app_id = app
            .id
            .as_ref()
            .ok_or_else(|| Error::Argument("app.id required".to_string()))?;
        let url = self.url.join(&format!("install/{}", app_id))?;
        self.client.post(url).send().await?;
        Ok(())
    }

    pub async fn device_info(&self) -> Result<DeviceInfo> {
        let url = self.url.join("query/device-info")?;
        let res = self.client.get(url).send().await?;
        let text = res.text().await?;
        println!("{}", text);
        Ok(from_str(&text)?)
    }

    pub async fn input(&self, input: &[(String, String)]) -> Result<()> {
        let url = self.url.join("input")?;
        self.client.post(url).query(input).send().await?;
        Ok(())
    }

    pub async fn search(&self, search: Search) -> Result<()> {
        let search = search.build();
        let url = self.url.join("search")?;
        self.client.post(url).query(&search).send().await?;
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct Apps {
    #[serde(rename = "app")]
    pub apps: Vec<App>,
}

#[derive(Debug, Deserialize)]
pub struct ActiveApp {
    pub app: App,
    pub screensaver: Option<Screensaver>,
}

#[derive(Debug, Deserialize)]
pub struct App {
    pub id: Option<String>,
    #[serde(rename = "$value")]
    pub name: String,
    pub version: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Screensaver {
    pub black: Option<bool>,
    pub id: String,
    #[serde(rename = "$value")]
    pub name: String,
    #[serde(rename = "type")]
    pub screensaver_type: String,
    pub version: String,
}

#[derive(Debug, Deserialize)]
pub struct MediaPlayer {
    pub buffering: Option<Buffering>,
    pub duration: Option<String>,
    pub error: bool,
    pub format: Option<Format>,
    pub is_live: Option<bool>,
    pub new_stream: Option<NewStream>,
    pub plugin: Option<Plugin>,
    pub position: Option<String>,
    pub runtime: Option<String>,
    pub state: String,
    pub stream_segment: Option<StreamSegment>,
}

#[derive(Debug, Deserialize)]
pub struct Plugin {
    pub bandwidth: String,
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct Format {
    pub audio: String,
    pub captions: String,
    pub container: String,
    pub drm: String,
    pub video: String,
    pub video_res: String,
}

#[derive(Debug, Deserialize)]
pub struct Buffering {
    pub current: u32,
    pub max: u32,
    pub target: u32,
}

#[derive(Debug, Deserialize)]
pub struct NewStream {
    pub speed: String,
}

#[derive(Debug, Deserialize)]
pub struct StreamSegment {
    pub bitrate: u32,
    pub media_sequence: u32,
    pub segment_type: String,
    pub time: u32,
}

pub enum Key {
    Back,
    Backspace,
    ChannelDown,
    ChannelUp,
    Down,
    Enter,
    FindRemote,
    Fwd,
    Home,
    Info,
    InputAV1,
    InputHDMI1,
    InputHDMI2,
    InputHDMI3,
    InputHDMI4,
    InputTuner,
    InstantReplay,
    Left,
    Play,
    PowerOff,
    Rev,
    Right,
    Search,
    Select,
    Up,
    VolumeDown,
    VolumeMute,
    VolumeUp,
    Lit(char),
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Key::Back => write!(f, "Back"),
            Key::Backspace => write!(f, "Backspace"),
            Key::ChannelDown => write!(f, "ChannelDown"),
            Key::ChannelUp => write!(f, "ChannelUp"),
            Key::Down => write!(f, "Down"),
            Key::Enter => write!(f, "Enter"),
            Key::FindRemote => write!(f, "FindRemote"),
            Key::Fwd => write!(f, "Fwd"),
            Key::Home => write!(f, "Home"),
            Key::Info => write!(f, "Info"),
            Key::InputAV1 => write!(f, "InputAV1"),
            Key::InputHDMI1 => write!(f, "InputHDMI1"),
            Key::InputHDMI2 => write!(f, "InputHDMI2"),
            Key::InputHDMI3 => write!(f, "InputHDMI3"),
            Key::InputHDMI4 => write!(f, "InputHDMI4"),
            Key::InputTuner => write!(f, "InputTuner"),
            Key::InstantReplay => write!(f, "InstantReplay"),
            Key::Left => write!(f, "Left"),
            Key::Play => write!(f, "Play"),
            Key::PowerOff => write!(f, "PowerOff"),
            Key::Rev => write!(f, "Rev"),
            Key::Right => write!(f, "Right"),
            Key::Search => write!(f, "Search"),
            Key::Select => write!(f, "Select"),
            Key::Up => write!(f, "Up"),
            Key::VolumeDown => write!(f, "VolumeDown"),
            Key::VolumeMute => write!(f, "VolumeMute"),
            Key::VolumeUp => write!(f, "VolumeUp"),
            Key::Lit(c) => write!(f, "Lit_{}", c),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DeviceInfo {
    pub advertising_id: String,
    pub build_number: String,
    pub can_use_wifi_extender: bool,
    pub clock_format: String,
    pub country: String,
    pub davinci_version: String,
    pub default_device_name: String,
    pub developer_enabled: bool,
    pub device_id: String,
    pub ethernet_mac: Option<String>,
    pub find_remote_is_possible: bool,
    pub friendly_device_name: String,
    pub friendly_model_name: String,
    pub grandcentral_version: String,
    pub has_mobile_screensaver: bool,
    pub has_play_on_roku: bool,
    #[serde(rename = "has-wifi-5G-support")]
    pub has_wifi_5g_support: bool,
    pub has_wifi_extender: bool,
    pub headphones_connected: bool,
    pub is_stick: bool,
    pub is_tv: bool,
    pub keyed_developer_id: String,
    pub language: String,
    pub locale: String,
    pub model_name: String,
    pub model_number: String,
    pub model_region: String,
    pub network_name: String,
    pub network_type: String,
    pub notifications_enabled: bool,
    pub notifications_first_use: bool,
    pub power_mode: String,
    pub search_channels_enabled: bool,
    pub search_enabled: bool,
    pub secure_device: bool,
    pub serial_number: String,
    pub software_build: String,
    pub software_version: String,
    pub support_url: String,
    pub supports_audio_guide: bool,
    pub supports_ecs_microphone: bool,
    pub supports_ecs_textedit: bool,
    pub supports_ethernet: bool,
    pub supports_find_remote: bool,
    pub supports_private_listening: bool,
    pub supports_rva: bool,
    pub supports_suspend: bool,
    pub supports_wake_on_wlan: bool,
    pub time_zone: String,
    pub time_zone_auto: bool,
    pub time_zone_name: String,
    pub time_zone_offset: i32,
    pub time_zone_tz: String,
    pub udn: String,
    pub uptime: u32,
    pub user_device_location: String,
    pub user_device_name: String,
    pub vendor_name: String,
    pub voice_search_enabled: bool,
    pub wifi_driver: String,
    pub wifi_mac: String,
}

pub struct Search {
    keyword: String,
    launch: Option<bool>,
    match_any: Option<bool>,
    providers: Option<Vec<String>>,
    provider_ids: Option<Vec<String>>,
    search_type: Option<SearchType>,
    season: Option<u32>,
    show_unavailable: Option<bool>,
    title: Option<String>,
    tmsid: Option<String>,
}

impl Search {
    pub fn new(keyword: String) -> Search {
        Search {
            keyword,
            launch: None,
            match_any: None,
            provider_ids: None,
            providers: None,
            search_type: None,
            season: None,
            show_unavailable: None,
            title: None,
            tmsid: None,
        }
    }

    fn build(self) -> Vec<(String, String)> {
        let mut ret = vec![("keyword", self.keyword)];
        if let Some(launch) = self.launch {
            ret.push(("launch", launch.to_string()));
        }
        if let Some(match_any) = self.match_any {
            ret.push(("match-any", match_any.to_string()));
        }
        if let Some(provider_ids) = self.provider_ids {
            ret.push(("provider-id", provider_ids.join(",")));
        }
        if let Some(providers) = self.providers {
            ret.push(("provider", providers.join(",")));
        }
        if let Some(search_type) = self.search_type {
            ret.push((
                "type",
                match search_type {
                    SearchType::Movie => "movie",
                    SearchType::TvShow => "tv-show",
                    SearchType::Person => "person",
                    SearchType::Channel => "channel",
                    SearchType::Game => "game",
                }
                .to_string(),
            ));
        }
        if let Some(season) = self.season {
            ret.push(("season", season.to_string()));
        }
        if let Some(show_unavailable) = self.show_unavailable {
            ret.push(("show-unavailable", show_unavailable.to_string()));
        }
        if let Some(title) = self.title {
            ret.push(("title", title));
        }
        if let Some(tmsid) = self.tmsid {
            ret.push(("tmsid", tmsid));
        }
        ret.into_iter().map(|(k, v)| (k.to_string(), v)).collect()
    }

    pub fn launch(&mut self, launch: bool) -> &mut Self {
        self.launch = Some(launch);
        self
    }

    pub fn match_any(&mut self, match_any: bool) -> &mut Self {
        self.match_any = Some(match_any);
        self
    }

    pub fn provider(&mut self, provider: String) -> &mut Self {
        match &mut self.providers {
            Some(providers) => {
                providers.push(provider);
            }
            None => {
                self.providers = Some(vec![]);
            }
        }
        self
    }

    pub fn provider_id(&mut self, provider_id: String) -> &mut Self {
        match &mut self.provider_ids {
            Some(provider_ids) => {
                provider_ids.push(provider_id);
            }
            None => {
                self.provider_ids = Some(vec![]);
            }
        }
        self
    }

    pub fn search_type(&mut self, search_type: SearchType) -> &mut Self {
        self.search_type = Some(search_type);
        self
    }

    pub fn season(&mut self, season: u32) -> &mut Self {
        self.season = Some(season);
        self
    }
    pub fn show_unavailable(&mut self, show_unavailable: bool) -> &mut Self {
        self.show_unavailable = Some(show_unavailable);
        self
    }

    pub fn title(&mut self, title: String) -> &mut Self {
        self.title = Some(title);
        self
    }

    pub fn tmsid(&mut self, tmsid: String) -> &mut Self {
        self.tmsid = Some(tmsid);
        self
    }
}

pub enum SearchType {
    Movie,
    TvShow,
    Person,
    Channel,
    Game,
}

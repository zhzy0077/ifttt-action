use crate::{Feed, Indexable, State};
use anyhow::Result;
use async_trait::async_trait;
use rss::{Channel, Item};
use serde::Deserialize;

const WEATHER_URL: &str = "https://devapi.heweather.net/v7/weather/3d";

pub struct WeatherFeed {
    pub config: WeatherConfig,
}

impl WeatherFeed {
    pub fn new(key: impl Into<String>, location: impl Into<String>) -> Self {
        let config = WeatherConfig {
            key: key.into(),
            location: location.into(),
        };
        WeatherFeed { config }
    }
}

#[async_trait]
impl Feed for WeatherFeed {
    async fn feed(&self, state: &mut State) -> Result<Vec<Box<dyn Indexable>>> {
        let client = reqwest::ClientBuilder::new().build()?;

        let res: WeatherOutput = client
            .get(WEATHER_URL)
            .query(&[
                ("key", &self.config.key),
                ("location", &self.config.location),
            ])
            .send()
            .await?
            .json()
            .await?;
        Ok(vec![Box::new(res)])
    }
}

#[derive(Deserialize)]
pub struct WeatherConfig {
    pub key: String,
    pub location: String,
}

#[derive(Debug, Deserialize)]
pub struct WeatherOutput {
    #[serde(alias = "fxLink")]
    fx_link: String,
    daily: Vec<Weather>,
}
#[derive(Debug, Deserialize)]
struct Weather {
    #[serde(alias = "tempMax")]
    temp_max: String,
    #[serde(alias = "tempMin")]
    temp_min: String,
    #[serde(alias = "textDay")]
    text_day: String,
    #[serde(alias = "windDirDay")]
    wind_dir_day: String,
    #[serde(alias = "windSpeedDay")]
    wind_speed_day: String,
    #[serde(alias = "windScaleDay")]
    wind_scale_day: String,
}

impl<'a> std::ops::Index<&'a str> for WeatherOutput {
    type Output = str;
    fn index(&self, field: &'a str) -> &Self::Output {
        let weather = self.daily.first().expect("No weather found.");
        match field {
            "fx_link" => &self.fx_link,
            "temp_max" => &weather.temp_max,
            "temp_min" => &weather.temp_min,
            "text_day" => &weather.text_day,
            "wind_dir_day" => &weather.wind_dir_day,
            "wind_speed_day" => &weather.wind_speed_day,
            "wind_scale_day" => &weather.wind_scale_day,
            _ => "",
        }
    }
}

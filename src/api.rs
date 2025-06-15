use firebase_rs::{Firebase, RequestError};
use ratatui::style::Stylize;
use ratatui::text::{Line, Text};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum Item {
    #[serde(rename = "story")]
    Story {
        /// The username of the story's author.
        by: Option<String>,
        dead: Option<bool>,
        /// The total comment count.
        descendants: Option<isize>,
        id: usize,
        /// The ids of the item's comments, in ranked display order.
        kids: Option<Vec<usize>>,
        /// The story's score.
        score: usize,
        text: Option<String>,
        /// Creation date of the item, in Unix Time.
        time: usize,
        /// The title of the story.
        title: Option<String>,
        ///	The URL of the story.
        url: Option<String>,
    },
    #[serde(rename = "comment")]
    Comment {
        /// The username of the comment's author.
        by: Option<String>,
        id: usize,
        /// The comment's parent: either another comment or the relevant story.
        parent: usize,
        text: Option<String>,
        time: usize,
        /// The ids of the item's comments, in ranked display order.
        kids: Option<Vec<usize>>,
    },
    #[serde(other)]
    Unsupported,
}

impl From<&Item> for Text<'_> {
    fn from(value: &Item) -> Self {
        match value {
            Item::Story { by, descendants, time, title, .. } => {
                let datetime = OffsetDateTime::from_unix_timestamp(*time as i64).unwrap();
                Text::from(vec![
                    Line::from(title.clone().unwrap()).bold(),
                    Line::from(format!(
                        "    --{} @ {}.{}.{} {}:{}:{} {} comments",
                        by.clone().unwrap_or("".to_string()),
                        datetime.year(), datetime.month() as usize, datetime.day(),
                        datetime.hour(), datetime.minute(), datetime.second(),
                        descendants.unwrap_or(0),
                    )),
                    Line::from(""),
                ])
            }
            Item::Comment { by, text, time, kids, .. } => {
                let datetime = OffsetDateTime::from_unix_timestamp(*time as i64).unwrap();
                Text::from(format!(
                    "{}\n    --{} @ {}.{}.{} {}:{}:{} {} comments\n ",
                    // TODO: Hard-coded line length
                    textwrap::wrap(text.clone().unwrap_or("".to_string()).as_str(), 100).join("\n"),
                    by.clone().unwrap_or("".to_string()),
                    datetime.year(), datetime.month() as usize, datetime.day(),
                    datetime.hour(), datetime.minute(), datetime.second(),
                    kids.clone().iter().len(),
                ))
            }
            Item::Unsupported => Text::from("Unsupported item type"),
        }
    }
}

pub struct HackerNews {
    endpoint: Firebase,
}

impl HackerNews {
    pub fn new(endpoint: &str) -> Self {
        HackerNews {
            endpoint: Firebase::new(endpoint).unwrap()
        }
    }

    pub fn default() -> Self {
        HackerNews::new("https://hacker-news.firebaseio.com/v0/")
    }

    pub async fn get_new_stories(&self) -> Result<Vec<usize>, RequestError> {
        Ok(self.endpoint.at("newstories").get::<Vec<usize>>().await?)
    }

    pub async fn get_top_stories(&self) -> Result<Vec<usize>, RequestError> {
        Ok(self.endpoint.at("topstories").get::<Vec<usize>>().await?)
    }

    pub async fn get_item(&self, id: usize) -> Result<Item, RequestError> {
        Ok(self.endpoint.at("item").at(&format!("{}", id)).get::<Item>().await?)
    }
}

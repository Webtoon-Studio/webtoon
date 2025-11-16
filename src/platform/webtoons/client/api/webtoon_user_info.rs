use serde::Deserialize;

#[allow(unused)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WebtoonUserInfo {
    author: bool,
    pub favorite: bool,
}

impl WebtoonUserInfo {
    pub fn is_webtoon_creator(&self) -> bool {
        self.author
    }

    #[allow(unused)]
    pub fn did_rate(&self) -> bool {
        self.favorite
    }
}

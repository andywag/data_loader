use serde::{Deserialize, Serialize};


#[derive(Deserialize, Serialize, Debug)]
pub enum SourceFilter {
    #[serde(rename = "json_text")]
    JsonText,
    #[serde(rename = "fast_text")]
    FastText
}

impl SourceFilter {
    pub fn get_text(&self, line:String) -> Option<String> {
        match self {
            SourceFilter::JsonText => super::provider_util::create_json_text(line, "text"),
            SourceFilter::FastText => None,
        }
        
    }
}
use abi_stable::{
    export_root_module,
    prefix_type::PrefixTypeTrait,
    sabi_extern_fn,
    sabi_trait::prelude::TD_Opaque,
    std_types::{RBox, RStr, RString, RVec},
};
use quick_search_lib::{ColoredChar, Log, PluginId, SearchLib, SearchLib_Ref, SearchResult, Searchable, Searchable_TO};
use serde::{Deserialize, Serialize};

static NAME: &str = "DuckDuckGo-Search";

#[export_root_module]
pub fn get_library() -> SearchLib_Ref {
    SearchLib { get_searchable }.leak_into_prefix()
}

#[sabi_extern_fn]
fn get_searchable(id: PluginId, logger: quick_search_lib::ScopedLogger) -> Searchable_TO<'static, RBox<()>> {
    let this = DuckDuckGo::new(id, logger);
    Searchable_TO::from_value(this, TD_Opaque)
}

struct DuckDuckGo {
    id: PluginId,
    client: reqwest::blocking::Client,
    config: quick_search_lib::Config,
    logger: quick_search_lib::ScopedLogger,
}

impl DuckDuckGo {
    fn new(id: PluginId, logger: quick_search_lib::ScopedLogger) -> Self {
        Self {
            id,
            client: reqwest::blocking::Client::new(),
            config: default_config(),
            logger,
        }
    }
}

impl Searchable for DuckDuckGo {
    fn search(&self, query: RString) -> RVec<SearchResult> {
        let mut res: Vec<SearchResult> = vec![];

        let url = format!("https://ac.duckduckgo.com/ac/?q={}", urlencoding::encode(query.as_str()));

        if let Ok(response) = self.client.get(url).send() {
            if let Ok(text) = response.json::<Vec<QueryResult>>() {
                for query in text {
                    res.push(SearchResult::new(&query.phrase));
                }
            }
        }

        res.sort_by(|a, b| a.title().cmp(b.title()));
        res.dedup_by(|a, b| a.title() == b.title());
        if self.config.get("Always return query even if no results found").and_then(|e| e.as_bool()).unwrap_or(true) {
            res.retain(|r| r.title() != query);
            res.insert(0, SearchResult::new(&query));
        }

        res.into()
    }
    fn name(&self) -> RStr<'static> {
        NAME.into()
    }
    fn colored_name(&self) -> RVec<quick_search_lib::ColoredChar> {
        // can be dynamic although it's iffy how it might be used
        vec![
            // 0xde5833FF
            ColoredChar::new('D', 0xDE5833FF),
            ColoredChar::new('u', 0xDE5833FF),
            ColoredChar::new('c', 0xDE5833FF),
            ColoredChar::new('k', 0xDE5833FF),
            // 0xffcc33FF
            ColoredChar::new('D', 0xFFCC33FF),
            ColoredChar::new('u', 0xFFCC33FF),
            ColoredChar::new('c', 0xFFCC33FF),
            ColoredChar::new('k', 0xFFCC33FF),
            // 0x4cba3cFF
            ColoredChar::new('G', 0x4CBA3CFF),
            ColoredChar::new('o', 0x4CBA3CFF),
        ]
        .into()
    }
    fn execute(&self, result: &SearchResult) {
        // let s = result.extra_info();
        // if let Ok::<clipboard::ClipboardContext, Box<dyn std::error::Error>>(mut clipboard) = clipboard::ClipboardProvider::new() {
        //     if let Ok(()) = clipboard::ClipboardProvider::set_contents(&mut clipboard, s.to_owned()) {
        //         println!("copied to clipboard: {}", s);
        //     } else {
        //         println!("failed to copy to clipboard: {}", s);
        //     }
        // } else {
        //     log::error!("failed to copy to clipboard: {}", s);
        // }

        // finish up, above is a clipboard example

        if let Err(e) = webbrowser::open(&format!("https://ac.duckduckgo.com/?q={}", urlencoding::encode(result.title()))) {
            self.logger.error(&format!("failed to open browser: {}", e));
        }
    }
    fn plugin_id(&self) -> PluginId {
        self.id.clone()
    }
    fn get_config_entries(&self) -> quick_search_lib::Config {
        default_config()
    }
    fn lazy_load_config(&mut self, config: quick_search_lib::Config) {
        self.config = config;
    }
}

fn default_config() -> quick_search_lib::Config {
    let mut config = quick_search_lib::Config::new();
    config.insert("Always return query even if no results found".into(), quick_search_lib::EntryType::Bool { value: true });
    config
}

// [
//     {
//         "phrase":"bing"
//     },{
//         "phrase":"brightness"
//     },{
//         "phrase":"best buy"
//     },{
//         "phrase":"bank of america"
//     },{
//         "phrase":"bobbi althoff"
//     },{
//         "phrase":"blackboard"
//     },{
//         "phrase":"bbc news"
//     },{
//         "phrase":"bluetooth"
//     }
// ]

#[derive(Debug, Clone, Serialize, Deserialize)]
struct QueryResult {
    phrase: String,
}
